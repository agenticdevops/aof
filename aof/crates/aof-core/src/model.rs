use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::AofResult;

/// Model provider trait - abstraction over LLM providers
///
/// Implementations should minimize allocations and use zero-copy where possible.
#[async_trait]
pub trait Model: Send + Sync {
    /// Generate completion (non-streaming)
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse>;

    /// Generate completion (streaming)
    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>>;

    /// Model configuration
    fn config(&self) -> &ModelConfig;

    /// Provider type
    fn provider(&self) -> ModelProvider;

    /// Count tokens in text (approximate)
    fn count_tokens(&self, text: &str) -> usize {
        // Default: rough approximation (4 chars per token)
        text.len() / 4
    }
}

/// Model provider enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Google,
    Groq,
    Bedrock,
    Azure,
    Ollama,
    Custom,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,

    /// Provider
    pub provider: ModelProvider,

    /// API key (optional, can use env var)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API endpoint (for custom providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Temperature (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Extra provider-specific config
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    60
}

/// Model request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    /// Messages in conversation
    pub messages: Vec<RequestMessage>,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Tools available
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<ToolDefinition>,

    /// Temperature override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Max tokens override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Stream response
    #[serde(default)]
    pub stream: bool,

    /// Extra parameters
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Message in request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::ToolCall>>,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool definition for model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// Generated text
    pub content: String,

    /// Tool calls requested by model
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_calls: Vec<crate::ToolCall>,

    /// Stop reason
    pub stop_reason: StopReason,

    /// Usage statistics
    pub usage: Usage,

    /// Provider-specific metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Stop reason
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    ContentFilter,
}

/// Token usage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    ContentDelta { delta: String },
    ToolCall { tool_call: crate::ToolCall },
    Done { usage: Usage, stop_reason: StopReason },
}

/// Reference-counted model
pub type ModelRef = Arc<dyn Model>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_provider_serialization() {
        let provider = ModelProvider::Anthropic;
        let json = serde_json::to_string(&provider).unwrap();
        assert_eq!(json, "\"anthropic\"");

        let provider: ModelProvider = serde_json::from_str("\"openai\"").unwrap();
        assert_eq!(provider, ModelProvider::OpenAI);
    }

    #[test]
    fn test_model_config_defaults() {
        let json = r#"{
            "model": "gpt-4",
            "provider": "openai"
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.provider, ModelProvider::OpenAI);
        assert_eq!(config.temperature, 0.7); // default
        assert_eq!(config.timeout_secs, 60); // default
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_model_config_full() {
        let config = ModelConfig {
            model: "claude-3-5-sonnet".to_string(),
            provider: ModelProvider::Anthropic,
            api_key: Some("test_key".to_string()),
            endpoint: Some("https://api.anthropic.com".to_string()),
            temperature: 0.3,
            max_tokens: Some(4096),
            timeout_secs: 120,
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Custom".to_string(), "value".to_string());
                h
            },
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ModelConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "claude-3-5-sonnet");
        assert_eq!(deserialized.temperature, 0.3);
        assert_eq!(deserialized.max_tokens, Some(4096));
    }

    #[test]
    fn test_message_role() {
        assert_eq!(
            serde_json::to_string(&MessageRole::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Assistant).unwrap(),
            "\"assistant\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::System).unwrap(),
            "\"system\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Tool).unwrap(),
            "\"tool\""
        );
    }

    #[test]
    fn test_model_request() {
        let request = ModelRequest {
            messages: vec![
                RequestMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    tool_calls: None,
                },
            ],
            system: Some("You are a helpful assistant.".to_string()),
            tools: vec![],
            temperature: Some(0.5),
            max_tokens: Some(1000),
            stream: false,
            extra: HashMap::new(),
        };

        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.system, Some("You are a helpful assistant.".to_string()));
    }

    #[test]
    fn test_stop_reason_serialization() {
        let end_turn = StopReason::EndTurn;
        let json = serde_json::to_string(&end_turn).unwrap();
        assert_eq!(json, "\"end_turn\"");

        let max_tokens: StopReason = serde_json::from_str("\"max_tokens\"").unwrap();
        assert_eq!(max_tokens, StopReason::MaxTokens);
    }

    #[test]
    fn test_usage() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
        };

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);

        let default_usage = Usage::default();
        assert_eq!(default_usage.input_tokens, 0);
        assert_eq!(default_usage.output_tokens, 0);
    }

    #[test]
    fn test_stream_chunk_content_delta() {
        let chunk = StreamChunk::ContentDelta {
            delta: "Hello".to_string(),
        };

        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("content_delta"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_stream_chunk_done() {
        let chunk = StreamChunk::Done {
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
            },
            stop_reason: StopReason::EndTurn,
        };

        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("done"));
        assert!(json.contains("end_turn"));
    }

    #[test]
    fn test_model_response() {
        let response = ModelResponse {
            content: "Hello, world!".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 5,
                output_tokens: 3,
            },
            metadata: HashMap::new(),
        };

        assert_eq!(response.content, "Hello, world!");
        assert!(response.tool_calls.is_empty());
        assert_eq!(response.stop_reason, StopReason::EndTurn);
    }
}
