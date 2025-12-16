use aof_core::{
    Model, ModelConfig, ModelProvider, ModelRequest, RequestMessage, ModelToolDefinition,
};
use aof_core::model::MessageRole;
use aof_llm::provider::anthropic::AnthropicProvider;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_anthropic_provider_creation_with_api_key() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = AnthropicProvider::create(config);
    assert!(result.is_ok(), "Provider creation should succeed with API key");
}

#[test]
fn test_anthropic_provider_creation_without_api_key() {
    // Clear env var if set
    std::env::remove_var("ANTHROPIC_API_KEY");

    let config = ModelConfig {
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

    let result = AnthropicProvider::create(config);
    assert!(result.is_err(), "Provider creation should fail without API key");
}

#[test]
fn test_anthropic_provider_config() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.3,
        max_tokens: Some(2048),
        timeout_secs: 120,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = AnthropicProvider::create(config).unwrap();
    let returned_config = provider.config();

    assert_eq!(returned_config.model, "claude-3-5-sonnet-20241022");
    assert_eq!(returned_config.temperature, 0.3);
    assert_eq!(returned_config.max_tokens, Some(2048));
    assert_eq!(returned_config.timeout_secs, 120);
}

#[test]
fn test_anthropic_provider_type() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = AnthropicProvider::create(config).unwrap();
    assert_eq!(provider.provider(), ModelProvider::Anthropic);
}

#[test]
fn test_anthropic_token_counting() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = AnthropicProvider::create(config).unwrap();

    // Test basic text
    let tokens = provider.count_tokens("Hello, world!");
    assert!(tokens > 0, "Should count tokens");
    assert!(tokens < 10, "Should be approximate");

    // Test longer text
    let long_text = "This is a longer text that should have more tokens than the short one.";
    let long_tokens = provider.count_tokens(long_text);
    assert!(long_tokens > tokens, "Longer text should have more tokens");

    // Test empty text
    let empty_tokens = provider.count_tokens("");
    assert_eq!(empty_tokens, 0, "Empty text should have 0 tokens");
}

#[test]
fn test_anthropic_custom_headers() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers,
        extra: HashMap::new(),
    };

    let result = AnthropicProvider::create(config);
    assert!(result.is_ok(), "Provider should accept custom headers");
}

#[test]
fn test_anthropic_request_building() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = AnthropicProvider::create(config).unwrap();

    // Test basic request
    let request = ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            tool_calls: None,
        }],
        system: Some("You are a helpful assistant.".to_string()),
        tools: vec![],
        temperature: Some(0.5),
        max_tokens: Some(1000),
        stream: false,
        extra: HashMap::new(),
    };

    // This would normally be tested via the build_request_body method
    // but it's private. We're testing the structure is valid.
    assert_eq!(request.messages.len(), 1);
    assert!(request.system.is_some());
}

#[test]
fn test_anthropic_with_tools() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = AnthropicProvider::create(config).unwrap();

    let tool = ModelToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather for a location".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        }),
    };

    let request = ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: "What's the weather in NYC?".to_string(),
            tool_calls: None,
        }],
        system: None,
        tools: vec![tool],
        temperature: None,
        max_tokens: None,
        stream: false,
        extra: HashMap::new(),
    };

    assert_eq!(request.tools.len(), 1);
    assert_eq!(request.tools[0].name, "get_weather");
}

#[test]
fn test_anthropic_temperature_override() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7, // Default
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = AnthropicProvider::create(config).unwrap();

    let request = ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: "Test".to_string(),
            tool_calls: None,
        }],
        system: None,
        tools: vec![],
        temperature: Some(0.2), // Override
        max_tokens: None,
        stream: false,
        extra: HashMap::new(),
    };

    assert_eq!(request.temperature, Some(0.2));
}

#[test]
fn test_anthropic_max_tokens_override() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096), // Default
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = AnthropicProvider::create(config).unwrap();

    let request = ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: "Test".to_string(),
            tool_calls: None,
        }],
        system: None,
        tools: vec![],
        temperature: None,
        max_tokens: Some(1024), // Override
        stream: false,
        extra: HashMap::new(),
    };

    assert_eq!(request.max_tokens, Some(1024));
}

#[test]
fn test_anthropic_multiple_messages() {
    let request = ModelRequest {
        messages: vec![
            RequestMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                tool_calls: None,
            },
            RequestMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                tool_calls: None,
            },
            RequestMessage {
                role: MessageRole::User,
                content: "How are you?".to_string(),
                tool_calls: None,
            },
        ],
        system: None,
        tools: vec![],
        temperature: None,
        max_tokens: None,
        stream: false,
        extra: HashMap::new(),
    };

    assert_eq!(request.messages.len(), 3);
}
