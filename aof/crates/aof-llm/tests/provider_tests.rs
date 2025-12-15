//! Unit tests for aof-llm provider factory and implementations

use aof_core::{AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, RequestMessage, StopReason, Usage};
use aof_llm::{ProviderFactory};
use std::collections::HashMap;

#[tokio::test]
async fn test_provider_factory_anthropic() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-ant-test123".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(1024),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let model = ProviderFactory::create(config).await;
    assert!(model.is_ok(), "Should create Anthropic provider");

    let model = model.unwrap();
    assert_eq!(model.provider(), ModelProvider::Anthropic);
}

#[tokio::test]
async fn test_provider_factory_openai() {
    let config = ModelConfig {
        model: "gpt-4".to_string(),
        provider: ModelProvider::OpenAI,
        api_key: Some("sk-test123".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(1024),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let model = ProviderFactory::create(config).await;
    assert!(model.is_ok(), "Should create OpenAI provider");

    let model = model.unwrap();
    assert_eq!(model.provider(), ModelProvider::OpenAI);
}

#[tokio::test]
async fn test_provider_factory_unsupported() {
    let config = ModelConfig {
        model: "test-model".to_string(),
        provider: ModelProvider::Azure,
        api_key: None,
        endpoint: None,
        temperature: 0.7,
        max_tokens: None,
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let result = ProviderFactory::create(config).await;
    assert!(result.is_err(), "Should fail for unsupported provider");
}

#[test]
fn test_model_config_serialization() {
    let config = ModelConfig {
        model: "claude-3-5-sonnet".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("secret".to_string()),
        endpoint: None,
        temperature: 0.5,
        max_tokens: Some(2048),
        timeout_secs: 120,
        headers: HashMap::new(),
        extra: HashMap::new(),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ModelConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.model, "claude-3-5-sonnet");
    assert_eq!(deserialized.provider, ModelProvider::Anthropic);
    assert_eq!(deserialized.temperature, 0.5);
    assert_eq!(deserialized.max_tokens, Some(2048));
}

#[test]
fn test_model_request_serialization() {
    let request = ModelRequest {
        messages: vec![
            RequestMessage {
                role: aof_core::model::MessageRole::User,
                content: "Hello".to_string(),
                tool_calls: None,
            },
        ],
        system: Some("You are helpful".to_string()),
        tools: vec![],
        temperature: Some(0.7),
        max_tokens: Some(1000),
        stream: false,
        extra: HashMap::new(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("Hello"));
    assert!(json.contains("You are helpful"));
}

#[test]
fn test_model_response_serialization() {
    let response = ModelResponse {
        content: "Hello, how can I help?".to_string(),
        tool_calls: vec![],
        stop_reason: StopReason::EndTurn,
        usage: Usage {
            input_tokens: 10,
            output_tokens: 5,
        },
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: ModelResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content, "Hello, how can I help?");
    assert_eq!(deserialized.stop_reason, StopReason::EndTurn);
    assert_eq!(deserialized.usage.input_tokens, 10);
    assert_eq!(deserialized.usage.output_tokens, 5);
}

#[test]
fn test_stop_reason_variants() {
    let reasons = vec![
        StopReason::EndTurn,
        StopReason::MaxTokens,
        StopReason::StopSequence,
        StopReason::ToolUse,
        StopReason::ContentFilter,
    ];

    for reason in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: StopReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }
}

#[test]
fn test_usage_default() {
    let usage = Usage::default();
    assert_eq!(usage.input_tokens, 0);
    assert_eq!(usage.output_tokens, 0);
}

#[test]
fn test_provider_enum() {
    let providers = vec![
        ModelProvider::Anthropic,
        ModelProvider::OpenAI,
        ModelProvider::Bedrock,
        ModelProvider::Azure,
        ModelProvider::Ollama,
        ModelProvider::Custom,
    ];

    for provider in providers {
        let json = serde_json::to_string(&provider).unwrap();
        let deserialized: ModelProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(provider, deserialized);
    }
}
