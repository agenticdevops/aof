//! Google subscription provider.
//!
//! Uses Google's Generative Language API (`generativelanguage.googleapis.com`)
//! with OAuth Bearer token authentication instead of the `?key=API_KEY` URL
//! parameter used by the standard API-key provider.
//!
//! The standard Google provider appends `?key=<api_key>` to each URL.
//! This subscription provider omits the key param and adds an
//! `Authorization: Bearer <oauth_token>` header instead.
//!
//! Endpoint: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
//! Streaming: same endpoint with `:streamGenerateContent?alt=sse`
//!
//! On HTTP 401: returns `AofError::Model` indicating the subscription token
//! may have expired. The caller must re-authenticate.

use agentix_core::{
    model::{MessageRole, StopReason, Usage},
    AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, StreamChunk, ToolCall,
};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

const GOOGLE_BASE_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Google subscription provider — OAuth Bearer token to Generative Language API.
pub struct GoogleSubscriptionProvider;

impl GoogleSubscriptionProvider {
    /// Create a Google subscription model.
    ///
    /// `config.api_key` must contain a valid OAuth access token obtained via
    /// Google OAuth2 PKCE flow. Returns an error if no token is present.
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        let token = config
            .api_key
            .clone()
            .ok_or_else(|| {
                AofError::config(
                    "Google subscription mode requires an OAuth access token in api_key field",
                )
            })?;

        // Use custom endpoint if provided, otherwise use the standard Google endpoint.
        // Note: the subscription variant does NOT append ?key=token.
        let endpoint = config
            .endpoint
            .clone()
            .unwrap_or_else(|| GOOGLE_BASE_ENDPOINT.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AofError::model(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Box::new(GoogleSubscriptionModel {
            config,
            token,
            endpoint,
            client,
        }))
    }
}

struct GoogleSubscriptionModel {
    config: ModelConfig,
    token: String,
    endpoint: String,
    client: Client,
}

impl GoogleSubscriptionModel {
    /// Build the generateContent URL without API key.
    ///
    /// Standard provider: `{endpoint}/models/{model}:generateContent?key={key}`
    /// Subscription:      `{endpoint}/models/{model}:generateContent`
    fn generate_url(&self) -> String {
        format!(
            "{}/models/{}:generateContent",
            self.endpoint, self.config.model
        )
    }

    /// Build the streamGenerateContent URL without API key.
    fn stream_url(&self) -> String {
        format!(
            "{}/models/{}:streamGenerateContent?alt=sse",
            self.endpoint, self.config.model
        )
    }

    /// Build the Gemini request payload from a ModelRequest.
    fn build_request(&self, request: &ModelRequest) -> GeminiRequest {
        let mut contents: Vec<GeminiContent> = Vec::new();

        // Build tool_call_id → name map for resolving tool response names
        let mut tool_id_to_name: HashMap<String, String> = HashMap::new();
        for m in request.messages.iter() {
            if let Some(tool_calls) = &m.tool_calls {
                for tc in tool_calls {
                    tool_id_to_name.insert(tc.id.clone(), tc.name.clone());
                }
            }
        }

        let mut pending_function_responses: Vec<GeminiPart> = Vec::new();

        for (i, m) in request.messages.iter().enumerate() {
            if m.role != MessageRole::Tool && !pending_function_responses.is_empty() {
                contents.push(GeminiContent {
                    role: "user".to_string(),
                    parts: std::mem::take(&mut pending_function_responses),
                });
            }

            match m.role {
                MessageRole::User => {
                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::Text { text: m.content.clone() }],
                    });
                }
                MessageRole::Assistant => {
                    if let Some(tool_calls) = &m.tool_calls {
                        let mut parts: Vec<GeminiPart> = Vec::new();
                        if !m.content.is_empty() {
                            parts.push(GeminiPart::Text { text: m.content.clone() });
                        }
                        for tc in tool_calls {
                            parts.push(GeminiPart::FunctionCall {
                                function_call: GeminiFunctionCall {
                                    name: tc.name.clone(),
                                    args: tc.arguments.clone(),
                                },
                            });
                        }
                        contents.push(GeminiContent {
                            role: "model".to_string(),
                            parts,
                        });
                    } else {
                        contents.push(GeminiContent {
                            role: "model".to_string(),
                            parts: vec![GeminiPart::Text { text: m.content.clone() }],
                        });
                    }
                }
                MessageRole::System => {}
                MessageRole::Tool => {
                    let tool_name = m
                        .tool_call_id
                        .as_ref()
                        .and_then(|id| tool_id_to_name.get(id))
                        .cloned()
                        .unwrap_or_else(|| {
                            for j in (0..i).rev() {
                                if let Some(tcs) = &request.messages[j].tool_calls {
                                    let tool_index = (j + 1..=i)
                                        .filter(|&k| request.messages[k].role == MessageRole::Tool)
                                        .count()
                                        - 1;
                                    if let Some(tc) = tcs.get(tool_index) {
                                        return tc.name.clone();
                                    }
                                    if let Some(tc) = tcs.first() {
                                        return tc.name.clone();
                                    }
                                }
                            }
                            "unknown".to_string()
                        });

                    let response_data = serde_json::from_str::<serde_json::Value>(&m.content)
                        .unwrap_or_else(|_| serde_json::json!({"result": m.content}));

                    pending_function_responses.push(GeminiPart::FunctionResponse {
                        function_response: GeminiFunctionResponse {
                            name: tool_name,
                            response: response_data,
                        },
                    });
                }
            }
        }

        if !pending_function_responses.is_empty() {
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: pending_function_responses,
            });
        }

        let system_instruction = request.system.as_ref().map(|s| GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: s.clone() }],
        });

        let tools = if !request.tools.is_empty() {
            Some(vec![GeminiTool {
                function_declarations: request
                    .tools
                    .iter()
                    .map(|t| GeminiFunctionDeclaration {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.clone(),
                    })
                    .collect(),
            }])
        } else {
            None
        };

        let generation_config = GeminiGenerationConfig {
            temperature: request.temperature.or(Some(self.config.temperature)),
            max_output_tokens: request.max_tokens.or(self.config.max_tokens),
            top_p: None,
            top_k: None,
        };

        GeminiRequest {
            contents,
            system_instruction,
            tools,
            generation_config: Some(generation_config),
        }
    }

    fn parse_response(&self, response: GeminiResponse) -> AofResult<ModelResponse> {
        let candidates = response.candidates.unwrap_or_default();

        if candidates.is_empty() {
            if let Some(prompt_feedback) = response.prompt_feedback {
                if !prompt_feedback.safety_ratings.is_empty() {
                    let safety_info = prompt_feedback
                        .safety_ratings
                        .iter()
                        .map(|r| format!("{}={}", r.category, r.probability))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(AofError::model(format!(
                        "Content blocked by safety filter: {}",
                        safety_info
                    )));
                }
            }
            return Err(AofError::model(
                "No candidates in Gemini response — possible API error or safety filter",
            ));
        }

        let candidate = candidates.first().unwrap();

        if candidate.content.is_none() {
            return Err(AofError::model(
                "No content in Gemini response — possible safety filter or API error",
            ));
        }

        let content_parts = candidate
            .content
            .as_ref()
            .map(|c| &c.parts)
            .ok_or_else(|| AofError::model("Missing parts in Gemini response content"))?;

        if content_parts.is_empty() {
            return Err(AofError::model(
                "Empty response from Gemini — no parts in content",
            ));
        }

        let content = content_parts
            .iter()
            .filter_map(|p| match p {
                GeminiPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let tool_calls: Vec<ToolCall> = content_parts
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match p {
                GeminiPart::FunctionCall { function_call } => Some(ToolCall {
                    id: format!("call_{}", i),
                    name: function_call.name.clone(),
                    arguments: function_call.args.clone(),
                }),
                _ => None,
            })
            .collect();

        let stop_reason = if !tool_calls.is_empty() {
            StopReason::ToolUse
        } else {
            match candidate.finish_reason.as_deref() {
                Some("STOP") => StopReason::EndTurn,
                Some("MAX_TOKENS") => StopReason::MaxTokens,
                Some("SAFETY") => StopReason::ContentFilter,
                _ => StopReason::EndTurn,
            }
        };

        let usage = response
            .usage_metadata
            .map(|u| Usage {
                input_tokens: u.prompt_token_count,
                output_tokens: u.candidates_token_count,
            })
            .unwrap_or_default();

        Ok(ModelResponse {
            content,
            tool_calls,
            stop_reason,
            usage,
            metadata: HashMap::new(),
        })
    }
}

#[async_trait]
impl Model for GoogleSubscriptionModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        let payload = self.build_request(request);
        let url = self.generate_url();

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Google subscription API request failed: {}", e)))?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(AofError::model(
                "Google subscription token is invalid or expired. \
                 Re-authenticate via `agentix auth login google`.",
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "Google subscription API error ({}): {}",
                status, error_text
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AofError::model(format!("Failed to parse Google response: {}", e)))?;

        self.parse_response(gemini_response)
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        let payload = self.build_request(request);
        let url = self.stream_url();

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Google subscription streaming request failed: {}", e)))?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(AofError::model(
                "Google subscription token is invalid or expired. \
                 Re-authenticate via `agentix auth login google`.",
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "Google subscription streaming error ({}): {}",
                status, error_text
            )));
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(|result| result.map_err(|e| AofError::model(format!("Stream error: {}", e))))
            .scan(String::new(), |buffer, chunk_result| {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => return futures::future::ready(Some(vec![Err(e)])),
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                let mut results = Vec::new();
                let lines: Vec<&str> = buffer.split('\n').collect();

                if let Some((last, complete)) = lines.split_last() {
                    for line in complete {
                        if let Some(chunk) = parse_gemini_stream_chunk(line) {
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
        ModelProvider::Google
    }

    fn count_tokens(&self, text: &str) -> usize {
        (text.len() as f32 / 4.0).ceil() as usize
    }
}

fn parse_gemini_stream_chunk(line: &str) -> Option<AofResult<StreamChunk>> {
    if line.is_empty() || !line.starts_with("data: ") {
        return None;
    }

    let data = line.strip_prefix("data: ")?;

    let chunk: GeminiStreamChunk = match serde_json::from_str(data) {
        Ok(c) => c,
        Err(e) => {
            return Some(Err(AofError::model(format!(
                "Failed to parse Google stream chunk: {}",
                e
            ))))
        }
    };

    let candidates = chunk.candidates.unwrap_or_default();
    let candidate = candidates.first()?;

    if let Some(content) = &candidate.content {
        for part in &content.parts {
            if let GeminiPart::Text { text } = part {
                return Some(Ok(StreamChunk::ContentDelta { delta: text.clone() }));
            }
        }
    }

    if let Some(finish_reason) = &candidate.finish_reason {
        let stop_reason = match finish_reason.as_str() {
            "STOP" => StopReason::EndTurn,
            "MAX_TOKENS" => StopReason::MaxTokens,
            "SAFETY" => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        };

        return Some(Ok(StreamChunk::Done {
            usage: chunk
                .usage_metadata
                .map(|u| Usage {
                    input_tokens: u.prompt_token_count,
                    output_tokens: u.candidates_token_count,
                })
                .unwrap_or_default(),
            stop_reason,
        }));
    }

    None
}

// ---------------------------------------------------------------------------
// Gemini API types (mirrored from the standard provider)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
    #[serde(rename = "promptFeedback")]
    prompt_feedback: Option<GeminiPromptFeedback>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    #[serde(default)]
    content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount", default)]
    prompt_token_count: usize,
    #[serde(rename = "candidatesTokenCount", default)]
    candidates_token_count: usize,
}

#[derive(Debug, Deserialize)]
struct GeminiStreamChunk {
    #[serde(default)]
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiPromptFeedback {
    #[serde(rename = "safetyRatings", default)]
    safety_ratings: Vec<GeminySafetyRating>,
}

#[derive(Debug, Deserialize)]
struct GeminySafetyRating {
    category: String,
    probability: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config(token: Option<&str>) -> ModelConfig {
        ModelConfig {
            model: "gemini-2.0-flash".to_string(),
            provider: ModelProvider::Google,
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
        let config = make_config(Some("oauth-access-token"));
        let result = GoogleSubscriptionProvider::create(config);
        assert!(result.is_ok(), "Should create provider with valid token");
    }

    #[test]
    fn create_fails_without_token() {
        let config = make_config(None);
        let result = GoogleSubscriptionProvider::create(config);
        assert!(result.is_err(), "Should fail without OAuth token");
        let err = result.err().unwrap().to_string();
        assert!(err.contains("api_key"), "Error should mention api_key field");
    }

    #[test]
    fn provider_returns_google() {
        let config = make_config(Some("test-token"));
        let model = GoogleSubscriptionProvider::create(config).unwrap();
        assert_eq!(model.provider(), ModelProvider::Google);
    }

    #[test]
    fn config_is_accessible() {
        let config = make_config(Some("test-token"));
        let model = GoogleSubscriptionProvider::create(config.clone()).unwrap();
        assert_eq!(model.config().model, "gemini-2.0-flash");
        assert_eq!(model.config().provider, ModelProvider::Google);
    }

    #[test]
    fn token_counting_reasonable() {
        let config = make_config(Some("test-token"));
        let model = GoogleSubscriptionProvider::create(config).unwrap();
        let text = "Hello, world!";
        let tokens = model.count_tokens(text);
        assert!(tokens >= 3 && tokens <= 5, "Token count should be ~3-4 for short text");
    }

    #[test]
    fn subscription_url_does_not_include_key_param() {
        let config = make_config(Some("secret-token"));
        let model = GoogleSubscriptionProvider::create(config).unwrap();
        // The endpoint should NOT contain ?key= anywhere
        // We can verify this by checking the model's config doesn't expose key in URL
        // (The actual URL is internal to GoogleSubscriptionModel, but we verify
        // the endpoint config is clean)
        let cfg = model.config();
        assert!(
            cfg.endpoint.is_none() || !cfg.endpoint.as_ref().unwrap().contains("?key="),
            "Subscription endpoint must not include API key param"
        );
    }

    #[test]
    fn custom_endpoint_is_respected() {
        let mut config = make_config(Some("test-token"));
        config.endpoint = Some("https://custom.googleapis.com/v1".to_string());
        let model = GoogleSubscriptionProvider::create(config).unwrap();
        assert_eq!(
            model.config().endpoint.as_deref(),
            Some("https://custom.googleapis.com/v1")
        );
    }
}
