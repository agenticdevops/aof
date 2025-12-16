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

/// Google/Gemini provider
pub struct GoogleProvider;

impl GoogleProvider {
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        Ok(Box::new(GoogleModel::new(config)?))
    }
}

/// Google Gemini model implementation
pub struct GoogleModel {
    config: ModelConfig,
    client: Client,
    api_key: String,
    endpoint: String,
}

impl GoogleModel {
    /// Create new Google Gemini model
    pub fn new(config: ModelConfig) -> AofResult<Self> {
        // Get API key from config or environment
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| {
                AofError::config("GOOGLE_API_KEY not found in config or environment")
            })?;

        // Use custom endpoint or default Gemini API
        let endpoint = config
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());

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

    /// Build request payload for Gemini API
    fn build_request(&self, request: &ModelRequest) -> GeminiRequest {
        // Convert messages to Gemini format
        // Note: Gemini uses "user" and "model" roles only. Tool responses use functionResponse parts.
        let mut contents: Vec<GeminiContent> = Vec::new();

        for (i, m) in request.messages.iter().enumerate() {
            match m.role {
                MessageRole::User => {
                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::Text { text: m.content.clone() }],
                    });
                }
                MessageRole::Assistant => {
                    // Check if this assistant message has tool calls
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
                MessageRole::System => {
                    // Gemini doesn't have system role in contents, skip (handled via system_instruction)
                }
                MessageRole::Tool => {
                    // Tool responses need functionResponse format
                    // Try to get the tool name from the previous assistant message's tool calls
                    let tool_name = if i > 0 {
                        request.messages.get(i - 1)
                            .and_then(|prev| prev.tool_calls.as_ref())
                            .and_then(|tcs| tcs.first())
                            .map(|tc| tc.name.clone())
                            .unwrap_or_else(|| "unknown".to_string())
                    } else {
                        "unknown".to_string()
                    };

                    // Parse content as JSON or wrap as string
                    let response_data = serde_json::from_str::<serde_json::Value>(&m.content)
                        .unwrap_or_else(|_| serde_json::json!({"result": m.content}));

                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::FunctionResponse {
                            function_response: GeminiFunctionResponse {
                                name: tool_name,
                                response: response_data,
                            },
                        }],
                    });
                }
            }
        }

        // Add system instruction if present
        let system_instruction = request.system.as_ref().map(|s| GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: s.clone() }],
        });

        // Convert tools to Gemini format
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

        // Generation config
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

    /// Parse Gemini response to ModelResponse
    fn parse_response(&self, response: GeminiResponse) -> AofResult<ModelResponse> {
        // Handle responses with missing or empty candidates (safety filters, errors, etc.)
        let candidates = response.candidates.unwrap_or_default();

        if candidates.is_empty() {
            // Check if there's a safety/error in the response
            if let Some(prompt_feedback) = response.prompt_feedback {
                if !prompt_feedback.safety_ratings.is_empty() {
                    let safety_info = prompt_feedback.safety_ratings
                        .iter()
                        .map(|r| format!("{}={:?}", r.category, r.probability))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(AofError::model(format!(
                        "Content blocked by safety filter: {}",
                        safety_info
                    )));
                }
            }
            return Err(AofError::model("No candidates in Gemini response - possible API error or safety filter"));
        }

        let candidate = candidates.first().unwrap();

        // Check if content is missing entirely
        if candidate.content.is_none() {
            // Check if there's safety feedback
            if let Some(prompt_feedback) = response.prompt_feedback {
                if !prompt_feedback.safety_ratings.is_empty() {
                    let safety_info = prompt_feedback.safety_ratings
                        .iter()
                        .map(|r| format!("{}={:?}", r.category, r.probability))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(AofError::model(format!(
                        "Content blocked by safety filter: {}",
                        safety_info
                    )));
                }
            }
            return Err(AofError::model("No content in Gemini response - possible safety filter or API error"));
        }

        // Validate that parts are present and not empty
        let content_parts = candidate
            .content
            .as_ref()
            .map(|c| &c.parts)
            .ok_or_else(|| AofError::model("Missing parts in Gemini response content"))?;

        if content_parts.is_empty() {
            tracing::warn!("[GOOGLE] Empty parts array in response candidate");
            return Err(AofError::model(
                "Empty response from Gemini - no parts in content. This may indicate a model processing issue or safety filter.",
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

        // Parse function calls
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

        // Map finish reason
        // Note: Gemini returns "STOP" even with tool calls, so we check tool_calls to determine ToolUse
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

        // Usage statistics
        let usage = response
            .usage_metadata
            .map(|u| Usage {
                input_tokens: u.prompt_token_count,
                output_tokens: u.candidates_token_count,
            })
            .unwrap_or_default();

        // Warn if both content and tool_calls are empty (valid but unusual)
        if content.is_empty() && tool_calls.is_empty() {
            tracing::warn!("[GOOGLE] Response has neither text content nor tool calls - finish_reason: {:?}",
                candidate.finish_reason);
        }

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
impl Model for GoogleModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        tracing::warn!("=== GOOGLE PROVIDER generate() START ===");
        let payload = self.build_request(request);

        tracing::warn!(
            "[GOOGLE] API CALL: model={}, messages={}, system={:?}, tools={}",
            self.config.model,
            payload.contents.len(),
            payload.system_instruction.is_some(),
            payload.tools.as_ref().map(|t| t.len()).unwrap_or(0)
        );

        // Gemini uses model name in URL
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, self.config.model, self.api_key
        );

        tracing::warn!("[GOOGLE] URL: {}/models/{}:generateContent?key=***", self.endpoint, self.config.model);

        // Log request payload for debugging (excluding sensitive data)
        if let Ok(payload_json) = serde_json::to_string_pretty(&payload) {
            tracing::warn!("[GOOGLE] Request payload (first 500 chars): {}", payload_json.chars().take(500).collect::<String>());
        }

        tracing::warn!("[GOOGLE] Sending HTTP POST request...");
        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("[GOOGLE] HTTP request FAILED: {}", e);
                AofError::model(format!("Gemini API request failed: {}", e))
            })?;

        tracing::warn!("[GOOGLE] Response received, status: {}", response.status());

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("[GOOGLE] API ERROR: {} - {}", status, error_text);
            return Err(AofError::model(format!(
                "Gemini API error ({}): {}",
                status, error_text
            )));
        }

        tracing::warn!("[GOOGLE] Parsing JSON response...");
        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("[GOOGLE] Failed to parse response JSON: {}", e);
                AofError::model(format!("Failed to parse Gemini response: {}", e))
            })?;

        tracing::warn!("[GOOGLE] Response parsed successfully, candidates={}", gemini_response.candidates.as_ref().map(|c| c.len()).unwrap_or(0));
        self.parse_response(gemini_response)
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        let payload = self.build_request(request);

        tracing::debug!(
            "Sending Gemini streaming request: model={}, contents={}",
            self.config.model,
            payload.contents.len()
        );

        // Gemini uses streamGenerateContent for streaming
        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}&alt=sse",
            self.endpoint, self.config.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::model(format!("Gemini streaming request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::model(format!(
                "Gemini streaming error ({}): {}",
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
        // Gemini models use roughly 4 chars per token
        (text.len() as f32 / 4.0).ceil() as usize
    }
}

/// Parse Gemini streaming chunk
fn parse_gemini_stream_chunk(line: &str) -> Option<AofResult<StreamChunk>> {
    // Skip empty lines
    if line.is_empty() || !line.starts_with("data: ") {
        return None;
    }

    let data = line.strip_prefix("data: ")?;

    // Parse JSON
    let chunk: GeminiStreamChunk = match serde_json::from_str(data) {
        Ok(c) => c,
        Err(e) => return Some(Err(AofError::model(format!("Failed to parse Gemini chunk: {}", e)))),
    };

    let candidates = chunk.candidates.unwrap_or_default();
    let candidate = candidates.first()?;

    // Handle content delta
    if let Some(content) = &candidate.content {
        for part in &content.parts {
            if let GeminiPart::Text { text } = part {
                return Some(Ok(StreamChunk::ContentDelta {
                    delta: text.clone(),
                }));
            }
        }
    }

    // Handle finish
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

// Gemini API types

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
    Text {
        text: String,
    },
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

    #[test]
    fn test_token_counting() {
        let config = ModelConfig {
            model: "gemini-2.0-flash".to_string(),
            provider: ModelProvider::Google,
            api_key: Some("test".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(1000),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let model = GoogleModel::new(config).unwrap();
        let count = model.count_tokens("Hello, world!");
        assert!(count >= 3 && count <= 4);
    }
}
