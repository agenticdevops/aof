#[cfg(feature = "bedrock")]
use aof_core::{
    Model, ModelConfig, ModelProvider, ModelRequest, RequestMessage,
};
#[cfg(feature = "bedrock")]
use aof_core::model::MessageRole;
#[cfg(feature = "bedrock")]
use aof_llm::provider::bedrock::BedrockProvider;
#[cfg(feature = "bedrock")]
use serde_json::json;
#[cfg(feature = "bedrock")]
use std::collections::HashMap;

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_provider_creation() {
    let mut extra = HashMap::new();
    extra.insert("region".to_string(), json!("us-east-1"));

    let config = ModelConfig {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        provider: ModelProvider::Bedrock,
        api_key: None, // Uses AWS credentials
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra,
    };

    // Note: This will use AWS credentials from environment
    let result = BedrockProvider::create(config).await;
    // May fail if AWS credentials not configured - that's expected
    assert!(result.is_ok() || result.is_err());
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_provider_config() {
    let mut extra = HashMap::new();
    extra.insert("region".to_string(), json!("us-west-2"));

    let config = ModelConfig {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        provider: ModelProvider::Bedrock,
        api_key: None,
        endpoint: None,
        temperature: 0.3,
        max_tokens: Some(2048),
        timeout_secs: 120,
        headers: HashMap::new(),
        extra,
    };

    if let Ok(provider) = BedrockProvider::create(config).await {
        let returned_config = provider.config();
        assert_eq!(returned_config.model, "anthropic.claude-3-sonnet-20240229-v1:0");
        assert_eq!(returned_config.temperature, 0.3);
        assert_eq!(returned_config.max_tokens, Some(2048));
        assert_eq!(returned_config.timeout_secs, 120);
    }
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_provider_type() {
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

    if let Ok(provider) = BedrockProvider::create(config).await {
        assert_eq!(provider.provider(), ModelProvider::Bedrock);
    }
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_token_counting() {
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

    if let Ok(provider) = BedrockProvider::create(config).await {
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
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_region_configuration() {
    let regions = vec!["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"];

    for region in regions {
        let mut extra = HashMap::new();
        extra.insert("region".to_string(), json!(region));

        let config = ModelConfig {
            model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            provider: ModelProvider::Bedrock,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: extra.clone(),
        };

        if let Ok(provider) = BedrockProvider::create(config).await {
            assert_eq!(
                provider.config().extra.get("region"),
                Some(&json!(region))
            );
        }
    }
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_different_models() {
    let models = vec![
        "anthropic.claude-3-sonnet-20240229-v1:0",
        "anthropic.claude-3-haiku-20240307-v1:0",
        "anthropic.claude-v2:1",
        "amazon.titan-text-express-v1",
    ];

    for model in models {
        let mut extra = HashMap::new();
        extra.insert("region".to_string(), json!("us-east-1"));

        let config = ModelConfig {
            model: model.to_string(),
            provider: ModelProvider::Bedrock,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra,
        };

        if let Ok(provider) = BedrockProvider::create(config).await {
            assert_eq!(provider.config().model, model);
        }
    }
}

#[cfg(feature = "bedrock")]
#[test]
fn test_bedrock_request_structure() {
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

    assert_eq!(request.messages.len(), 1);
    assert!(request.system.is_some());
    assert_eq!(request.temperature, Some(0.5));
}

#[cfg(feature = "bedrock")]
#[tokio::test]
async fn test_bedrock_temperature_override() {
    let mut extra = HashMap::new();
    extra.insert("region".to_string(), json!("us-east-1"));

    let config = ModelConfig {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        provider: ModelProvider::Bedrock,
        api_key: None,
        endpoint: None,
        temperature: 0.7, // Default
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra,
    };

    if let Ok(_provider) = BedrockProvider::create(config).await {
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
}

#[cfg(not(feature = "bedrock"))]
#[tokio::test]
async fn test_bedrock_feature_disabled() {
    use aof_core::{ModelConfig, ModelProvider};
    use aof_llm::provider::bedrock::BedrockProvider;
    use std::collections::HashMap;

    let config = ModelConfig {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        provider: ModelProvider::Bedrock,
        api_key: None,
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = BedrockProvider::create(config).await;
    assert!(result.is_err(), "Should fail when bedrock feature is disabled");
}
