use aof_core::{
    model::MessageRole, AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, ToolCall, Usage,
};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

/// OpenAI provider
pub struct OpenAIProvider;

impl OpenAIProvider {
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        Ok(Box::new(OpenAIModel::new(config)?))
    }
}

/// OpenAI model implementation
pub struct OpenAIModel {
    config: ModelConfig,
    client: Client,
    api_key: String,
    endpoint: String,
}

impl OpenAIModel {
    /// Create new OpenAI model
    pub fn new(config: ModelConfig) -> AofResult<Self> {
        // Get API key from config or environment
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| {
                AofError::config("OPENAI_API_KEY not found in config or environment")
            })?;

        // Use custom endpoint or default
        let endpoint = config
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Build HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AofError::model(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            client,
            api_key,
            endpoint,
        })
    }

    /// Build request payload
    fn build_request(&self, request: &ModelRequest) -> OpenAIRequest {
        // Convert messages
        let messages: Vec<OpenAIMessage> = if let Some(system) = &request.system {
            // Add system message first
            let mut msgs = vec![OpenAIMessage {
                role: "system".to_string(),
                content: Some(system.clone()),
                tool_calls: None,
                tool_call_id: None,
            }];
            msgs.extend(request.messages.iter().map(|m| OpenAIMessage {
                role: match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Tool => "tool",
                }
                .to_string(),
                content: Some(m.content.clone()),
                tool_calls: m.tool_calls.as_ref().map(|calls| {
                    calls
                        .iter()
                        .map(|tc| OpenAIToolCall {
                            id: tc.id.clone(),
                            function: OpenAIFunction {
                                name: tc.name.clone(),
                                arguments: serde_json::to_string(&tc.arguments)
                                    .unwrap_or_default(),
                            },
                            r#type: "function".to_string(),
                        })
                        .collect()
                }),
                tool_call_id: None,
            }));
            msgs
        } else {
            request
                .messages
                .iter()
                .map(|m| OpenAIMessage {
                    role: match m.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                        MessageRole::Tool => "tool",
                    }
                    .to_string(),
                    content: Some(m.content.clone()),
                    tool_calls: m.tool_calls.as_ref().map(|calls| {
                        calls
                            .iter()
                            .map(|tc| OpenAIToolCall {
                                id: tc.id.clone(),
                                function: OpenAIFunction {
                                    name: tc.name.clone(),
                                    arguments: serde_json::to_string(&tc.arguments)
                                        .unwrap_or_default(),
                                },
                                r#type: "function".to_string(),
                            })
                            .collect()
                    }),
                    tool_call_id: None,
                })
                .collect()
        };

        // Convert tools
        let tools = if !request.tools.is_empty() {
            Some(
                request
                    .tools
                    .iter()
                    .map(|t| OpenAITool {
                        r#type: "function".to_string(),
                        function: OpenAIFunctionDef {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            parameters: t.parameters.clone(),
                        },
                    })
                    .collect(),
            )
        } else {
            None
        };

        OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            temperature: request.temperature.or(Some(self.config.temperature)),
            max_tokens: request.max_tokens.or(self.config.max_tokens),
            stream: Some(request.stream),
            tools,
        }
    }

    /// Parse OpenAI response to ModelResponse
    fn parse_response(&self, response: OpenAIResponse) -> AofResult<ModelResponse> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AofError::model("No choices in OpenAI response"))?;

        let content = choice
            .message
            .content
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Parse tool calls
        let tool_calls = choice
            .message
            .tool_calls
            .as_ref()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|tc| {
                        serde_json::from_str(&tc.function.arguments)
                            .ok()
                            .map(|args| ToolCall {
                                id: tc.id.clone(),
                                name: tc.function.name.clone(),
                                arguments: args,
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Map finish reason
        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("tool_calls") | Some("function_call") => StopReason::ToolUse,
            Some("content_filter") => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        };

        let usage = Usage {
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
        };

        Ok(ModelResponse {
            content,
            tool_calls,
            stop_reason,
            usage,
            metadata: HashMap::new(),
        })
    }

}

/// Parse OpenAI streaming chunk (free function to avoid lifetime issues)
fn parse_openai_stream_chunk(line: &str) -> Option<AofResult<StreamChunk>> {
    // Skip empty lines and comments
    if line.is_empty() || !line.starts_with("data: ") {
        return None;
    }

    let data = line.strip_prefix("data: ")?;

    // Check for [DONE] marker
    if data.trim() == "[DONE]" {
        return None;
    }

    // Parse JSON
    let chunk: OpenAIStreamChunk = match serde_json::from_str(data) {
        Ok(c) => c,
        Err(e) => return Some(Err(AofError::model(format!("Failed to parse chunk: {}", e)))),
    };

    let choice = chunk.choices.first()?;

    // Handle content delta
    if let Some(content) = &choice.delta.content {
        return Some(Ok(StreamChunk::ContentDelta {
            delta: content.clone(),
        }));
    }

    // Handle tool calls
    if let Some(tool_calls) = &choice.delta.tool_calls {
        for tc in tool_calls {
            if let Some(func) = &tc.function {
                if let (Some(name), Some(args)) = (&func.name, &func.arguments) {
                    if let Ok(arguments) = serde_json::from_str(args) {
                        return Some(Ok(StreamChunk::ToolCall {
                            tool_call: ToolCall {
                                id: tc.id.clone().unwrap_or_default(),
                                name: name.clone(),
                                arguments,
                            },
                        }));
                    }
                }
            }
        }
    }

    // Handle finish
    if let Some(finish_reason) = &choice.finish_reason {
        let stop_reason = match finish_reason.as_str() {
            "stop" => StopReason::EndTurn,
            "length" => StopReason::MaxTokens,
            "tool_calls" | "function_call" => StopReason::ToolUse,
            "content_filter" => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        };

        return Some(Ok(StreamChunk::Done {
            usage: Usage {
                input_tokens: 0, // OpenAI doesn't provide usage in stream
                output_tokens: 0,
            },
            stop_reason,
        }));
    }

    None
}

#[async_trait]
impl Model for OpenAIModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        let payload = self.build_request(request);

        tracing::debug!(
            "Sending OpenAI request: model={}, messages={}",
            payload.model,
            payload.messages.len()
        );

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::model(format!("OpenAI API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| AofError::model(format!("Failed to parse OpenAI response: {}", e)))?;

        self.parse_response(openai_response)
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        let mut payload = self.build_request(request);
        payload.stream = Some(true);

        tracing::debug!(
            "Sending OpenAI streaming request: model={}, messages={}",
            payload.model,
            payload.messages.len()
        );

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::model(format!("OpenAI streaming request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "OpenAI streaming error ({}): {}",
                status, error_text
            )));
        }

        // Convert bytes stream to SSE stream
        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(|result| {
                result.map_err(|e| AofError::model(format!("Stream error: {}", e)))
            })
            .scan(String::new(), |buffer, chunk_result| {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => return futures::future::ready(Some(vec![Err(e)])),
                };

                // Append to buffer
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Split by newlines and process complete lines
                let mut results = Vec::new();
                let lines: Vec<&str> = buffer.split('\n').collect();

                // Keep the last incomplete line in buffer
                if let Some((last, complete)) = lines.split_last() {
                    for line in complete {
                        if let Some(chunk) = parse_openai_stream_chunk(line) {
                            results.push(chunk);
                        }
                    }
                    *buffer = last.to_string();
                }

                futures::future::ready(Some(results))
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(stream))
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::OpenAI
    }

    fn count_tokens(&self, text: &str) -> usize {
        // GPT models use ~4 chars per token on average
        // For more accurate counting, use tiktoken library
        (text.len() as f32 / 4.0).ceil() as usize
    }
}

// OpenAI API types

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    r#type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    r#type: String,
    function: OpenAIFunctionDef,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

/// Streaming tool call (OpenAI sends partial data in streams)
#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCall {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    r#type: Option<String>,
    #[serde(default)]
    function: Option<OpenAIStreamFunction>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamFunction {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let config = ModelConfig {
            model: "gpt-4".to_string(),
            provider: ModelProvider::OpenAI,
            api_key: Some("test".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(1000),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let model = OpenAIModel::new(config).unwrap();

        // "Hello, world!" is roughly 3 tokens
        let count = model.count_tokens("Hello, world!");
        assert!(count >= 3 && count <= 4);
    }

    #[test]
    fn test_stream_chunk_parsing() {
        // Test content delta
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk = parse_openai_stream_chunk(line);
        assert!(chunk.is_some());

        // Test [DONE] marker
        let line = "data: [DONE]";
        let chunk = parse_openai_stream_chunk(line);
        assert!(chunk.is_none());
    }
}
