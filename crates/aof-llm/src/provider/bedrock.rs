use aof_core::{
    model::MessageRole, AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, ToolCall, Usage,
};
use async_trait::async_trait;
use futures::Stream;
use serde_json::json;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{debug, error, warn};

#[cfg(feature = "bedrock")]
use aws_sdk_bedrockruntime::{
    operation::converse::ConverseOutput,
    types::{
        ContentBlock, ConversationRole, ConverseStreamOutput as StreamOutputEnum, Message as BedrockMessage,
        SystemContentBlock, Tool as BedrockTool, ToolConfiguration, ToolInputSchema, ToolSpecification,
    },
    Client,
};

#[cfg(feature = "bedrock")]
use aws_smithy_types::Document;

const MAX_RETRIES: u32 = 3;

#[cfg(feature = "bedrock")]
fn json_to_document(value: &serde_json::Value) -> Document {
    match value {
        serde_json::Value::Object(map) => {
            let mut doc_map = std::collections::HashMap::new();
            for (k, v) in map {
                doc_map.insert(k.clone(), json_to_document(v));
            }
            Document::Object(doc_map)
        }
        serde_json::Value::Array(arr) => {
            Document::Array(arr.iter().map(json_to_document).collect())
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Document::Number(aws_smithy_types::Number::PosInt(i as u64))
            } else if let Some(f) = n.as_f64() {
                Document::Number(aws_smithy_types::Number::Float(f))
            } else {
                Document::Number(aws_smithy_types::Number::PosInt(0))
            }
        }
        serde_json::Value::String(s) => Document::String(s.clone()),
        serde_json::Value::Bool(b) => Document::Bool(*b),
        serde_json::Value::Null => Document::Null,
    }
}

#[cfg(feature = "bedrock")]
fn document_to_json(doc: &Document) -> Result<serde_json::Value, String> {
    match doc {
        Document::Object(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), document_to_json(v)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        Document::Array(arr) => {
            let values: Result<Vec<_>, _> = arr.iter().map(document_to_json).collect();
            Ok(serde_json::Value::Array(values?))
        }
        Document::Number(n) => {
            let f = n.to_f64_lossy();
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| "Invalid float".to_string())
        }
        Document::String(s) => Ok(serde_json::Value::String(s.clone())),
        Document::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Document::Null => Ok(serde_json::Value::Null),
    }
}

/// Bedrock provider implementation
#[allow(dead_code)]
pub struct BedrockProvider {
    config: ModelConfig,
    #[cfg(feature = "bedrock")]
    client: Client,
    region: String,
}

impl BedrockProvider {
    #[cfg(feature = "bedrock")]
    pub async fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        let region = config
            .extra
            .get("region")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| std::env::var("AWS_REGION").ok())
            .unwrap_or_else(|| "us-east-1".to_string());

        // Initialize AWS SDK
        let config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()));

        let sdk_config = config_loader.load().await;
        let client = Client::new(&sdk_config);

        Ok(Box::new(Self {
            config,
            client,
            region,
        }))
    }

    #[cfg(not(feature = "bedrock"))]
    pub async fn create(_config: ModelConfig) -> AofResult<Box<dyn Model>> {
        Err(AofError::config(
            "Bedrock provider not enabled. Enable the 'bedrock' feature",
        ))
    }

    #[cfg(feature = "bedrock")]
    fn convert_messages(&self, request: &ModelRequest) -> AofResult<Vec<BedrockMessage>> {
        let mut messages = Vec::new();

        for msg in &request.messages {
            if msg.role == MessageRole::System {
                continue; // System messages handled separately
            }

            let role = match msg.role {
                MessageRole::User => ConversationRole::User,
                MessageRole::Assistant => ConversationRole::Assistant,
                MessageRole::Tool => ConversationRole::User, // Tool results as user
                MessageRole::System => continue,
            };

            let content = vec![ContentBlock::Text(msg.content.clone())];

            messages.push(
                BedrockMessage::builder()
                    .role(role)
                    .set_content(Some(content))
                    .build()
                    .map_err(|e| AofError::model(format!("Failed to build message: {}", e)))?,
            );
        }

        Ok(messages)
    }

    #[cfg(feature = "bedrock")]
    fn convert_tools(&self, tools: &[aof_core::ModelToolDefinition]) -> AofResult<ToolConfiguration> {
        let mut tool_specs = Vec::new();

        for tool in tools {
            // Convert JSON Value to AWS Document
            let input_schema = ToolInputSchema::Json(json_to_document(&tool.parameters));

            let spec = ToolSpecification::builder()
                .name(&tool.name)
                .description(&tool.description)
                .input_schema(input_schema)
                .build()
                .map_err(|e| AofError::model(format!("Failed to build tool spec: {}", e)))?;

            tool_specs.push(BedrockTool::ToolSpec(spec));
        }

        ToolConfiguration::builder()
            .set_tools(Some(tool_specs))
            .build()
            .map_err(|e| AofError::model(format!("Failed to build tool config: {}", e)))
    }

    #[cfg(feature = "bedrock")]
    fn parse_response(&self, output: ConverseOutput) -> AofResult<ModelResponse> {
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        let stop_reason = match output.stop_reason().as_str() {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            "tool_use" => StopReason::ToolUse,
            "content_filtered" => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        };

        let usage = output.usage().map(|u| Usage {
            input_tokens: u.input_tokens() as usize,
            output_tokens: u.output_tokens() as usize,
        }).unwrap_or_default();

        if let Some(message_output) = output.output {
            if let Ok(message) = message_output.as_message() {
                for block in message.content() {
                    match block {
                        ContentBlock::Text(text) => {
                            if !content.is_empty() {
                                content.push('\n');
                            }
                            content.push_str(text.as_str());
                        }
                        ContentBlock::ToolUse(tool_use) => {
                            // Convert Document to JSON Value
                            let input: serde_json::Value = tool_use.input()
                                .as_object()
                                .map(|obj| {
                                    let mut map = serde_json::Map::new();
                                    for (k, v) in obj {
                                        if let Ok(json_val) = document_to_json(v) {
                                            map.insert(k.clone(), json_val);
                                        }
                                    }
                                    serde_json::Value::Object(map)
                                })
                                .unwrap_or(json!({}));

                            tool_calls.push(ToolCall {
                                id: tool_use.tool_use_id().to_string(),
                                name: tool_use.name().to_string(),
                                arguments: input,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(ModelResponse {
            content,
            tool_calls,
            stop_reason,
            usage,
            metadata: HashMap::new(),
        })
    }

    #[cfg(feature = "bedrock")]
    async fn retry_request<F, Fut, T>(&self, f: F) -> AofResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = AofResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 {
                        let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempt));
                        warn!(
                            "Request failed (attempt {}/{}): {}. Retrying in {:?}",
                            attempt + 1,
                            MAX_RETRIES,
                            e,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AofError::model("Request failed".to_string())))
    }
}

#[async_trait]
impl Model for BedrockProvider {
    #[cfg(feature = "bedrock")]
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        debug!("Generating completion with Bedrock: {:?}", request);

        let messages = self.convert_messages(request)?;

        let response = self
            .retry_request(|| async {
                let mut builder = self
                    .client
                    .converse()
                    .model_id(&self.config.model)
                    .set_messages(Some(messages.clone()));

                // Add system prompt if present
                if let Some(system) = &request.system {
                    builder = builder.system(SystemContentBlock::Text(system.clone()));
                }

                // Add inference config
                let temperature = request.temperature.unwrap_or(self.config.temperature);
                let max_tokens = request.max_tokens.or(self.config.max_tokens).unwrap_or(4096);

                builder = builder
                    .inference_config(
                        aws_sdk_bedrockruntime::types::InferenceConfiguration::builder()
                            .temperature(temperature)
                            .max_tokens(max_tokens as i32)
                            .build()
                    );

                // Add tools if present
                if !request.tools.is_empty() {
                    let tool_config = self.convert_tools(&request.tools)?;
                    builder = builder.tool_config(tool_config);
                }

                let result = builder
                    .send()
                    .await
                    .map_err(|e| AofError::model(format!("Bedrock API error: {}", e)))?;

                Ok(result)
            })
            .await?;

        self.parse_response(response)
    }

    #[cfg(not(feature = "bedrock"))]
    async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
        Err(AofError::config("Bedrock feature not enabled"))
    }

    #[cfg(feature = "bedrock")]
    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        debug!("Generating streaming completion with Bedrock");

        let messages = self.convert_messages(request)?;

        let mut builder = self
            .client
            .converse_stream()
            .model_id(&self.config.model)
            .set_messages(Some(messages));

        // Add system prompt if present
        if let Some(system) = &request.system {
            builder = builder.system(SystemContentBlock::Text(system.clone()));
        }

        // Add inference config
        let temperature = request.temperature.unwrap_or(self.config.temperature);
        let max_tokens = request.max_tokens.or(self.config.max_tokens).unwrap_or(4096);

        builder = builder
            .inference_config(
                aws_sdk_bedrockruntime::types::InferenceConfiguration::builder()
                    .temperature(temperature)
                    .max_tokens(max_tokens as i32)
                    .build()
            );

        // Add tools if present
        if !request.tools.is_empty() {
            let tool_config = self.convert_tools(&request.tools)?;
            builder = builder.tool_config(tool_config);
        }

        let mut stream = builder
            .send()
            .await
            .map_err(|e| AofError::model(format!("Bedrock streaming API error: {}", e)))?;

        let output_stream = async_stream::stream! {
            while let Some(event) = stream.stream.recv().await.transpose() {
                match event {
                    Ok(output) => {
                        match output {
                            StreamOutputEnum::ContentBlockDelta(delta) => {
                                if let Some(text_delta) = delta.delta().and_then(|d| d.as_text().ok()) {
                                    yield Ok(StreamChunk::ContentDelta {
                                        delta: text_delta.to_string(),
                                    });
                                }
                            }
                            StreamOutputEnum::MessageStop(stop) => {
                                let stop_reason = match stop.stop_reason().as_str() {
                                    "end_turn" => StopReason::EndTurn,
                                    "max_tokens" => StopReason::MaxTokens,
                                    "stop_sequence" => StopReason::StopSequence,
                                    "tool_use" => StopReason::ToolUse,
                                    "content_filtered" => StopReason::ContentFilter,
                                    _ => StopReason::EndTurn,
                                };

                                yield Ok(StreamChunk::Done {
                                    usage: Usage::default(),
                                    stop_reason,
                                });
                            }
                            StreamOutputEnum::Metadata(metadata) => {
                                if let Some(_usage) = metadata.usage() {
                                    // Store usage for final Done event
                                    continue;
                                }
                            }
                            _ => continue,
                        }
                    }
                    Err(e) => {
                        error!("Bedrock stream error: {}", e);
                        yield Err(AofError::model(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }

    #[cfg(not(feature = "bedrock"))]
    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        Err(AofError::config("Bedrock feature not enabled"))
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Bedrock
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Bedrock uses different tokenizers per model
        // Claude models: ~3.5 chars per token
        // Titan models: ~4 chars per token
        // Use conservative estimate
        text.len() / 3
    }
}

#[cfg(all(test, feature = "bedrock"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let mut extra = HashMap::new();
        extra.insert("region".to_string(), json!("us-east-1"));

        let config = ModelConfig {
            model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            provider: ModelProvider::Bedrock,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra,
        };

        // Note: This will fail without AWS credentials configured
        // Just testing the creation logic, not actual API calls
        let result = BedrockProvider::create(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_token_counting() {
        let mut extra = HashMap::new();
        extra.insert("region".to_string(), json!("us-east-1"));

        let config = ModelConfig {
            model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            provider: ModelProvider::Bedrock,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra,
        };

        let provider = BedrockProvider::create(config).await.unwrap();
        let tokens = provider.count_tokens("Hello, world!");
        assert!(tokens > 0);
    }
}
