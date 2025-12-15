use aof_core::{
    Model, ModelConfig, ModelProvider, ModelRequest, RequestMessage, ModelToolDefinition,
};
use aof_core::model::MessageRole;
use aof_llm::provider::google::GoogleProvider;
use serde_json::json;
use std::collections::HashMap;

// === Unit Tests (no API calls) ===

#[test]
fn test_google_provider_creation_with_api_key() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = GoogleProvider::create(config);
    assert!(result.is_ok(), "Provider creation should succeed with API key");
}

#[test]
fn test_google_provider_creation_without_api_key() {
    // Clear env var if set
    std::env::remove_var("GOOGLE_API_KEY");

    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: None,
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = GoogleProvider::create(config);
    assert!(result.is_err(), "Provider creation should fail without API key");
}

#[test]
fn test_google_provider_config() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.3,
        max_tokens: Some(2048),
        timeout_secs: 120,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = GoogleProvider::create(config).unwrap();
    let returned_config = provider.config();

    assert_eq!(returned_config.model, "gemini-2.0-flash-exp");
    assert_eq!(returned_config.temperature, 0.3);
    assert_eq!(returned_config.max_tokens, Some(2048));
    assert_eq!(returned_config.timeout_secs, 120);
}

#[test]
fn test_google_provider_type() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = GoogleProvider::create(config).unwrap();
    assert_eq!(provider.provider(), ModelProvider::Google);
}

#[test]
fn test_google_token_counting() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let provider = GoogleProvider::create(config).unwrap();

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
fn test_google_custom_endpoint() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: Some("https://custom.googleapis.com/v1beta".to_string()),
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = GoogleProvider::create(config);
    assert!(result.is_ok(), "Provider should accept custom endpoint");
}

#[test]
fn test_google_with_tools() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = GoogleProvider::create(config).unwrap();

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
fn test_google_temperature_override() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7, // Default
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = GoogleProvider::create(config).unwrap();

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
fn test_google_max_tokens_override() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096), // Default
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = GoogleProvider::create(config).unwrap();

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
fn test_google_multiple_messages() {
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
fn test_google_different_models() {
    let models = vec![
        "gemini-2.0-flash-exp",
        "gemini-2.0-flash-thinking-exp",
        "gemini-1.5-flash",
        "gemini-1.5-pro",
        "gemini-exp-1206",
    ];

    for model in models {
        let config = ModelConfig {
            model: model.to_string(),
            provider: ModelProvider::Google,
            api_key: Some("test-api-key".to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let result = GoogleProvider::create(config);
        assert!(result.is_ok(), "Provider should be created for model: {}", model);

        let provider = result.unwrap();
        assert_eq!(provider.config().model, model);
    }
}

#[test]
fn test_google_system_instruction() {
    let config = ModelConfig {
        model: "gemini-2.0-flash-exp".to_string(),
        provider: ModelProvider::Google,
        api_key: Some("test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let _provider = GoogleProvider::create(config).unwrap();

    let request = ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            tool_calls: None,
        }],
        system: Some("You are a helpful assistant.".to_string()),
        tools: vec![],
        temperature: None,
        max_tokens: None,
        stream: false,
        extra: HashMap::new(),
    };

    assert!(request.system.is_some());
    assert_eq!(request.system.as_ref().unwrap(), "You are a helpful assistant.");
}

// === Integration Tests (require API key) ===

#[cfg(feature = "integration_tests")]
mod integration {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_google_generate_simple() {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .expect("GOOGLE_API_KEY must be set for integration tests");

        let config = ModelConfig {
            model: "gemini-2.0-flash-exp".to_string(),
            provider: ModelProvider::Google,
            api_key: Some(api_key),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(100),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let provider = GoogleProvider::create(config).unwrap();

        let request = ModelRequest {
            messages: vec![RequestMessage {
                role: MessageRole::User,
                content: "Say 'hello' and nothing else.".to_string(),
                tool_calls: None,
            }],
            system: None,
            tools: vec![],
            temperature: Some(0.0),
            max_tokens: Some(10),
            stream: false,
            extra: HashMap::new(),
        };

        let response = provider.generate(&request).await;
        assert!(response.is_ok(), "Generate should succeed: {:?}", response.err());

        let response = response.unwrap();
        assert!(!response.content.is_empty(), "Response should have content");
        assert!(response.content.to_lowercase().contains("hello"));
    }

    #[tokio::test]
    async fn test_google_generate_with_tools() {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .expect("GOOGLE_API_KEY must be set for integration tests");

        let config = ModelConfig {
            model: "gemini-2.0-flash-exp".to_string(),
            provider: ModelProvider::Google,
            api_key: Some(api_key),
            endpoint: None,
            temperature: 0.0,
            max_tokens: Some(256),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        };

        let provider = GoogleProvider::create(config).unwrap();

        let tool = ModelToolDefinition {
            name: "get_weather".to_string(),
            description: "Get the current weather for a location".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city name"
                    }
                },
                "required": ["location"]
            }),
        };

        let request = ModelRequest {
            messages: vec![RequestMessage {
                role: MessageRole::User,
                content: "What's the weather in San Francisco?".to_string(),
                tool_calls: None,
            }],
            system: Some("Use the get_weather tool to answer weather questions.".to_string()),
            tools: vec![tool],
            temperature: Some(0.0),
            max_tokens: Some(256),
            stream: false,
            extra: HashMap::new(),
        };

        let response = provider.generate(&request).await;
        assert!(response.is_ok(), "Generate with tools should succeed: {:?}", response.err());

        let response = response.unwrap();
        // Either we get tool calls or a response
        assert!(!response.tool_calls.is_empty() || !response.content.is_empty());
    }
}
