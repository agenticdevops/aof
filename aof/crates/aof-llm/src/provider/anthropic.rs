use aof_core::model::{StopReason, Usage};
use aof_core::{
    AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse,
    StreamChunk, ToolCall,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic provider
pub struct AnthropicProvider;

impl AnthropicProvider {
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        // Get API key from config or environment
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                AofError::config("ANTHROPIC_API_KEY not found in config or environment")
            })?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AofError::model(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Box::new(AnthropicModel {
            config,
            api_key,
            client,
        }))
    }
}

/// Anthropic model implementation
pub struct AnthropicModel {
    config: ModelConfig,
    api_key: String,
    client: Client,
}

impl AnthropicModel {
    /// Build base request with headers
    fn build_request(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .post(endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
    }

    /// Convert AOF request to Anthropic API format
    fn to_anthropic_request(&self, request: &ModelRequest) -> AnthropicRequest {
        // Convert messages
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .filter_map(|msg| {
                // Skip system messages (handled separately)
                if msg.role == aof_core::model::MessageRole::System {
                    return None;
                }

                // Convert message content
                let mut content = vec![AnthropicContent::Text {
                    text: msg.content.clone(),
                }];

                // Add tool results if present
                if let Some(ref tool_calls) = msg.tool_calls {
                    for tool_call in tool_calls {
                        content.push(AnthropicContent::ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            content: serde_json::to_string(&tool_call.arguments)
                                .unwrap_or_default(),
                        });
                    }
                }

                Some(AnthropicMessage {
                    role: match msg.role {
                        aof_core::model::MessageRole::User => "user".to_string(),
                        aof_core::model::MessageRole::Assistant => "assistant".to_string(),
                        aof_core::model::MessageRole::Tool => "user".to_string(), // Tool results as user
                        aof_core::model::MessageRole::System => return None,
                    },
                    content,
                })
            })
            .collect();

        // Convert tools
        let tools: Vec<AnthropicTool> = request
            .tools
            .iter()
            .map(|tool| AnthropicTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.parameters.clone(),
            })
            .collect();

        AnthropicRequest {
            model: self.config.model.clone(),
            messages,
            system: request.system.clone(),
            max_tokens: request
                .max_tokens
                .or(self.config.max_tokens)
                .unwrap_or(4096),
            temperature: request.temperature.or(Some(self.config.temperature)),
            stream: Some(request.stream),
            tools: if tools.is_empty() {
                None
            } else {
                Some(tools)
            },
        }
    }

    /// Convert Anthropic response to AOF format
    fn convert_anthropic_response(&self, response: AnthropicResponse) -> ModelResponse {
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        // Extract content and tool calls
        for block in response.content {
            match block {
                AnthropicContentBlock::Text { text } => {
                    content.push_str(&text);
                }
                AnthropicContentBlock::ToolUse {
                    id,
                    name,
                    input,
                } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        // Map stop reason
        let stop_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            Some("tool_use") => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        };

        ModelResponse {
            content,
            tool_calls,
            stop_reason,
            usage: Usage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
            },
            metadata: HashMap::new(),
        }
    }

    /// Parse SSE stream chunk (static version for use in async closures)
    fn parse_stream_event_static(line: &str) -> Option<AofResult<StreamChunk>> {
        if !line.starts_with("data: ") {
            return None;
        }

        let json_str = &line[6..]; // Remove "data: " prefix

        // Skip ping events
        if json_str.trim() == "{\"type\":\"ping\"}" {
            return None;
        }

        let event: AnthropicStreamEvent = match serde_json::from_str(json_str) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse stream event: {} - Line: {}", e, json_str);
                return Some(Err(AofError::model(format!(
                    "Failed to parse stream event: {}",
                    e
                ))));
            }
        };

        match event {
            AnthropicStreamEvent::ContentBlockStart { content_block: AnthropicContentBlock::ToolUse { id, name, input }, .. } => {
                Some(Ok(StreamChunk::ToolCall {
                    tool_call: ToolCall {
                        id,
                        name,
                        arguments: input,
                    },
                }))
            }
            AnthropicStreamEvent::ContentBlockDelta { delta: AnthropicDelta::TextDelta { text }, .. } => {
                Some(Ok(StreamChunk::ContentDelta { delta: text }))
            }
            AnthropicStreamEvent::ContentBlockStart { .. } => None,
            AnthropicStreamEvent::ContentBlockDelta { .. } => None,
            AnthropicStreamEvent::MessageDelta {
                delta,
                usage: stream_usage,
            } => {
                let stop_reason = match delta.stop_reason.as_deref() {
                    Some("end_turn") => StopReason::EndTurn,
                    Some("max_tokens") => StopReason::MaxTokens,
                    Some("stop_sequence") => StopReason::StopSequence,
                    Some("tool_use") => StopReason::ToolUse,
                    _ => StopReason::EndTurn,
                };

                Some(Ok(StreamChunk::Done {
                    usage: Usage {
                        input_tokens: 0, // Not provided in delta
                        output_tokens: stream_usage.output_tokens,
                    },
                    stop_reason,
                }))
            }
            _ => None,
        }
    }
}

#[async_trait]
impl Model for AnthropicModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        debug!(
            "Generating completion with model: {}",
            self.config.model
        );

        let api_request = self.to_anthropic_request(request);

        let response = self
            .build_request(ANTHROPIC_API_URL)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AofError::model(format!("API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error {}: {}", status, error_text);
            return Err(AofError::model(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AofError::model(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_anthropic_response(api_response))
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        debug!(
            "Starting streaming generation with model: {}",
            self.config.model
        );

        let mut api_request = self.to_anthropic_request(request);
        api_request.stream = Some(true);

        let response = self
            .build_request(ANTHROPIC_API_URL)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Stream request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error {}: {}", status, error_text);
            return Err(AofError::model(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let byte_stream = response.bytes_stream();

        // Convert byte stream to line stream and parse SSE events
        let stream = byte_stream
            .map(|result| result.map_err(|e| AofError::model(format!("Stream error: {}", e))))
            .scan(Vec::new(), |buffer, chunk_result| {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => return futures::future::ready(Some(vec![Err(e)])),
                };

                buffer.extend_from_slice(&chunk);

                let mut events = Vec::new();
                let mut start = 0;

                // Find complete lines
                while let Some(pos) = buffer[start..].iter().position(|&b| b == b'\n') {
                    let line_end = start + pos;
                    let line = String::from_utf8_lossy(&buffer[start..line_end]).to_string();
                    start = line_end + 1;

                    if !line.is_empty() {
                        events.push(Ok(line));
                    }
                }

                // Keep remaining bytes in buffer
                buffer.drain(..start);

                futures::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter)
            .filter_map(|line_result| async move {
                match line_result {
                    Ok(line) => Self::parse_stream_event_static(&line),
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream))
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Anthropic
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Anthropic uses a similar tokenization to GPT
        // Rough approximation: ~4 chars per token for English text
        // More accurate would use tiktoken or Claude's tokenizer
        let char_count = text.chars().count();
        (char_count as f32 / 3.5) as usize // Slightly more accurate than /4
    }
}

// ============================================================================
// Anthropic API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContent {
    Text { text: String },
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

// Streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum AnthropicStreamEvent {
    MessageStart {
        message: AnthropicStreamMessage,
    },
    ContentBlockStart {
        index: usize,
        content_block: AnthropicContentBlock,
    },
    ContentBlockDelta {
        index: usize,
        delta: AnthropicDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: AnthropicMessageDelta,
        usage: AnthropicStreamUsage,
    },
    MessageStop,
    Ping,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicStreamMessage {
    id: String,
    role: String,
    model: String,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicDelta {
    TextDelta { text: String },
    #[allow(dead_code)]
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageDelta {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamUsage {
    output_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let config = ModelConfig {
            model: "claude-3-5-sonnet-20241022".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: Some("test-key".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let model = AnthropicProvider::create(config).unwrap();

        // Test basic token counting
        let text = "Hello, world!";
        let tokens = model.count_tokens(text);
        assert!(tokens > 0 && tokens < 10, "Token count should be reasonable");

        // Longer text
        let long_text = "The quick brown fox jumps over the lazy dog. ".repeat(10);
        let long_tokens = model.count_tokens(&long_text);
        assert!(long_tokens > tokens, "Longer text should have more tokens");
    }

    #[test]
    fn test_provider_type() {
        let config = ModelConfig {
            model: "claude-3-5-sonnet-20241022".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: Some("test-key".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let model = AnthropicProvider::create(config).unwrap();
        assert_eq!(model.provider(), ModelProvider::Anthropic);
    }

    #[test]
    fn test_api_key_handling() {
        use std::sync::Mutex;
        use std::sync::OnceLock;

        // Use a static mutex to prevent parallel tests from conflicting
        static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();

        // Save original environment
        let original = std::env::var("ANTHROPIC_API_KEY").ok();

        // Test 1: API key from environment variable
        std::env::set_var("ANTHROPIC_API_KEY", "test-env-key");

        let config_with_env = ModelConfig {
            model: "claude-3-5-sonnet-20241022".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: None, // Not in config, should use env
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let result = AnthropicProvider::create(config_with_env);
        assert!(result.is_ok(), "Should create model with env API key");

        // Test 2: Missing API key should fail
        std::env::remove_var("ANTHROPIC_API_KEY");

        let config_no_key = ModelConfig {
            model: "claude-3-5-sonnet-20241022".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let result = AnthropicProvider::create(config_no_key);
        assert!(result.is_err(), "Should fail without API key");

        // Restore original environment
        if let Some(val) = original {
            std::env::set_var("ANTHROPIC_API_KEY", val);
        }
    }
}
