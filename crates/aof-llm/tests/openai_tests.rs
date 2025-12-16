use aof_core::{
    Model, ModelConfig, ModelProvider, ModelRequest, RequestMessage, ModelToolDefinition,
};
use aof_core::model::MessageRole;
use aof_llm::provider::openai::OpenAIProvider;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_openai_provider_creation_with_api_key() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::create(config);
    assert!(result.is_ok(), "Provider creation should succeed with API key");
}

#[test]
fn test_openai_provider_creation_without_api_key() {
    // Clear env var if set
    std::env::remove_var("OPENAI_API_KEY");

    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: None,
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::create(config);
    assert!(result.is_err(), "Provider creation should fail without API key");
}

#[test]
fn test_openai_provider_config() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.3,
        max_tokens: Some(2048),
        timeout_secs: 120,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::create(config).unwrap();
    let returned_config = provider.config();

    assert_eq!(returned_config.model, "gpt-4-turbo-preview");
    assert_eq!(returned_config.temperature, 0.3);
    assert_eq!(returned_config.max_tokens, Some(2048));
    assert_eq!(returned_config.timeout_secs, 120);
}

#[test]
fn test_openai_provider_type() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::create(config).unwrap();
    assert_eq!(provider.provider(), ModelProvider::OpenAI);
}

#[test]
fn test_openai_token_counting() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::create(config).unwrap();

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
fn test_openai_custom_endpoint() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: Some("https://custom.openai.com/v1".to_string()),
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::create(config).unwrap();
    assert_eq!(
        provider.config().endpoint,
        Some("https://custom.openai.com/v1".to_string())
    );
}

#[test]
fn test_openai_custom_headers() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers,
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::create(config);
    assert!(result.is_ok(), "Provider should accept custom headers");
}

#[test]
fn test_openai_with_tools() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = OpenAIProvider::create(config).unwrap();

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
        system: Some("You are a helpful assistant.".to_string()),
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
fn test_openai_system_message_handling() {
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
        temperature: None,
        max_tokens: None,
        stream: false,
        extra: HashMap::new(),
    };

    assert!(request.system.is_some());
    assert_eq!(request.system.unwrap(), "You are a helpful assistant.");
}

#[test]
fn test_openai_temperature_override() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7, // Default
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = OpenAIProvider::create(config).unwrap();

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
fn test_openai_multiple_messages() {
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

#[test]
fn test_openai_different_models() {
    let models = vec![
        "gpt-4-turbo-preview",
        "gpt-4",
        "gpt-3.5-turbo",
        "gpt-4-1106-preview",
    ];

    for model in models {
        let config = ModelConfig {
            model: model.to_string(),
            provider: ModelProvider::OpenAI,
            api_key: Some("sk-test-key".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let provider = OpenAIProvider::create(config).unwrap();
        assert_eq!(provider.config().model, model);
    }
}

#[test]
fn test_openai_timeout_configuration() {
    let config = ModelConfig {
        model: "gpt-4-turbo-preview".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 30, // Custom timeout
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::create(config).unwrap();
    assert_eq!(provider.config().timeout_secs, 30);
}
