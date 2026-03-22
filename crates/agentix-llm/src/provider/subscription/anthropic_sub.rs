//! Anthropic subscription provider.
//!
//! Uses the exact same Anthropic Messages API endpoint as the API-key provider
//! (`https://api.anthropic.com/v1/messages`) but authenticates with an OAuth
//! Bearer token instead of the `x-api-key` header.
//!
//! The OAuth token is sourced from `ModelConfig.api_key`. The caller
//! (AuthService) must ensure the token is valid before construction.
//!
//! On HTTP 401: returns `AofError::Model` with a message indicating the
//! subscription token may have expired. No automatic refresh is attempted.

use agentix_core::{AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, StreamChunk};
use async_trait::async_trait;
use futures::Stream;
use reqwest::{Client, RequestBuilder};
use std::pin::Pin;
use std::time::Duration;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic subscription provider — OAuth Bearer auth over the standard Messages API.
pub struct AnthropicSubscriptionProvider;

impl AnthropicSubscriptionProvider {
    /// Create a new Anthropic subscription model.
    ///
    /// `config.api_key` must contain a valid OAuth access token obtained via
    /// the Anthropic PKCE flow. Returns an error if no token is present.
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        let token = config
            .api_key
            .clone()
            .ok_or_else(|| {
                AofError::config(
                    "Anthropic subscription mode requires an OAuth access token in api_key field",
                )
            })?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AofError::model(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Box::new(AnthropicSubscriptionModel {
            config,
            token,
            client,
        }))
    }
}

/// Anthropic subscription model — delegates parsing to the standard Anthropic
/// provider types but overrides the auth header.
struct AnthropicSubscriptionModel {
    config: ModelConfig,
    token: String,
    client: Client,
}

impl AnthropicSubscriptionModel {
    /// Build a request builder with Bearer auth and Anthropic version header.
    ///
    /// The key difference from the standard provider: `Authorization: Bearer {token}`
    /// instead of `x-api-key: {token}`.
    fn build_request(&self) -> RequestBuilder {
        self.client
            .post(ANTHROPIC_API_URL)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
    }
}

#[async_trait]
impl Model for AnthropicSubscriptionModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        // Use our own HTTP client with Bearer auth (differs from standard x-api-key provider)
        let api_request = build_anthropic_request(&self.config, request);

        let response = self
            .build_request()
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Anthropic subscription API request failed: {}", e)))?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(AofError::model(
                "Anthropic subscription token is invalid or expired. \
                 Re-authenticate via `agentix auth login anthropic`.",
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "Anthropic subscription API error {}: {}",
                status, error_text
            )));
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AofError::model(format!("Failed to parse Anthropic response: {}", e)))?;

        Ok(convert_anthropic_response(api_response))
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        use futures::StreamExt;

        let mut api_request = build_anthropic_request(&self.config, request);
        api_request.stream = Some(true);

        let response = self
            .build_request()
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Anthropic subscription stream request failed: {}", e)))?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(AofError::model(
                "Anthropic subscription token is invalid or expired. \
                 Re-authenticate via `agentix auth login anthropic`.",
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "Anthropic subscription streaming error {}: {}",
                status, error_text
            )));
        }

        let byte_stream = response.bytes_stream();

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

                while let Some(pos) = buffer[start..].iter().position(|&b| b == b'\n') {
                    let line_end = start + pos;
                    let line = String::from_utf8_lossy(&buffer[start..line_end]).to_string();
                    start = line_end + 1;

                    if !line.is_empty() {
                        events.push(Ok(line));
                    }
                }

                buffer.drain(..start);

                futures::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter)
            .filter_map(|line_result| async move {
                match line_result {
                    Ok(line) => parse_anthropic_stream_event(&line),
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
        let char_count = text.chars().count();
        (char_count as f32 / 3.5) as usize
    }
}

// ---------------------------------------------------------------------------
// Anthropic API types (mirrored from the standard provider)
// These are private to this module — no cross-module sharing to keep
// each adapter self-contained and independently evolvable.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use agentix_core::model::{StopReason, Usage};
use agentix_core::ToolCall;

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
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum AnthropicStreamEvent {
    MessageStart { message: AnthropicStreamMessage },
    ContentBlockStart { index: usize, content_block: AnthropicContentBlock },
    ContentBlockDelta { index: usize, delta: AnthropicDelta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: AnthropicMessageDelta, usage: AnthropicStreamUsage },
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

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn build_anthropic_request(config: &ModelConfig, request: &ModelRequest) -> AnthropicRequest {
    use agentix_core::model::MessageRole;

    let messages: Vec<AnthropicMessage> = request
        .messages
        .iter()
        .filter_map(|msg| {
            if msg.role == MessageRole::System {
                return None;
            }

            let mut content = vec![AnthropicContent::Text {
                text: msg.content.clone(),
            }];

            if let Some(ref tool_calls) = msg.tool_calls {
                for tool_call in tool_calls {
                    content.push(AnthropicContent::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        content: serde_json::to_string(&tool_call.arguments).unwrap_or_default(),
                    });
                }
            }

            Some(AnthropicMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "user".to_string(),
                    MessageRole::System => return None,
                },
                content,
            })
        })
        .collect();

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
        model: config.model.clone(),
        messages,
        system: request.system.clone(),
        max_tokens: request.max_tokens.or(config.max_tokens).unwrap_or(4096),
        temperature: request.temperature.or(Some(config.temperature)),
        stream: Some(request.stream),
        tools: if tools.is_empty() { None } else { Some(tools) },
    }
}

fn convert_anthropic_response(response: AnthropicResponse) -> ModelResponse {
    let mut content = String::new();
    let mut tool_calls = Vec::new();

    for block in response.content {
        match block {
            AnthropicContentBlock::Text { text } => {
                content.push_str(&text);
            }
            AnthropicContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(ToolCall { id, name, arguments: input });
            }
        }
    }

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

fn parse_anthropic_stream_event(line: &str) -> Option<AofResult<StreamChunk>> {
    if !line.starts_with("data: ") {
        return None;
    }

    let json_str = &line[6..];

    if json_str.trim() == "{\"type\":\"ping\"}" {
        return None;
    }

    let event: AnthropicStreamEvent = match serde_json::from_str(json_str) {
        Ok(e) => e,
        Err(e) => {
            return Some(Err(AofError::model(format!(
                "Failed to parse Anthropic stream event: {}",
                e
            ))));
        }
    };

    match event {
        AnthropicStreamEvent::ContentBlockStart {
            content_block: AnthropicContentBlock::ToolUse { id, name, input },
            ..
        } => Some(Ok(StreamChunk::ToolCall {
            tool_call: ToolCall { id, name, arguments: input },
        })),
        AnthropicStreamEvent::ContentBlockDelta {
            delta: AnthropicDelta::TextDelta { text },
            ..
        } => Some(Ok(StreamChunk::ContentDelta { delta: text })),
        AnthropicStreamEvent::ContentBlockStart { .. } => None,
        AnthropicStreamEvent::ContentBlockDelta { .. } => None,
        AnthropicStreamEvent::MessageDelta { delta, usage: stream_usage } => {
            let stop_reason = match delta.stop_reason.as_deref() {
                Some("end_turn") => StopReason::EndTurn,
                Some("max_tokens") => StopReason::MaxTokens,
                Some("stop_sequence") => StopReason::StopSequence,
                Some("tool_use") => StopReason::ToolUse,
                _ => StopReason::EndTurn,
            };

            Some(Ok(StreamChunk::Done {
                usage: Usage {
                    input_tokens: 0,
                    output_tokens: stream_usage.output_tokens,
                },
                stop_reason,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config(token: Option<&str>) -> ModelConfig {
        ModelConfig {
            model: "claude-3-5-sonnet-20241022".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: token.map(|t| t.to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn create_succeeds_with_token() {
        let config = make_config(Some("oauth-test-token"));
        let result = AnthropicSubscriptionProvider::create(config);
        assert!(result.is_ok(), "Should create provider with valid token");
    }

    #[test]
    fn create_fails_without_token() {
        let config = make_config(None);
        let result = AnthropicSubscriptionProvider::create(config);
        assert!(result.is_err(), "Should fail without OAuth token");
        let err = result.err().unwrap().to_string();
        assert!(err.contains("api_key"), "Error should mention api_key field");
    }

    #[test]
    fn provider_returns_anthropic() {
        let config = make_config(Some("test-token"));
        let model = AnthropicSubscriptionProvider::create(config).unwrap();
        assert_eq!(model.provider(), ModelProvider::Anthropic);
    }

    #[test]
    fn config_is_accessible() {
        let config = make_config(Some("test-token"));
        let model = AnthropicSubscriptionProvider::create(config.clone()).unwrap();
        assert_eq!(model.config().model, "claude-3-5-sonnet-20241022");
        assert_eq!(model.config().provider, ModelProvider::Anthropic);
    }

    #[test]
    fn token_counting_reasonable() {
        let config = make_config(Some("test-token"));
        let model = AnthropicSubscriptionProvider::create(config).unwrap();
        let text = "Hello, world!";
        let tokens = model.count_tokens(text);
        assert!(tokens > 0 && tokens < 10, "Token count should be reasonable");
    }
}
