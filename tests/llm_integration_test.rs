use my_framework::llm::*;

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_openai_integration() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set for integration tests");

    let config = config::OpenAiConfig {
        api_key: Some(api_key),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        default_model: "gpt-4o-mini".to_string(),
        temperature: 0.7,
        max_tokens: Some(100),
    };

    let provider = OpenAiProvider::new(config, None).expect("Failed to create provider");

    let request = ChatRequest::new(vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ])
    .with_temperature(0.0)
    .with_max_tokens(50);

    let response = provider.chat(request).await.expect("Failed to get response");

    assert_eq!(response.choices.len(), 1);
    assert!(response.usage.total_tokens > 0);

    if let MessageContent::Text(text) = &response.choices[0].message.content {
        assert!(text.contains("Hello"));
    } else {
        panic!("Expected text response");
    }

    // Test cost calculation
    let cost = response.usage.calculate_cost(provider.model_info());
    assert!(cost > 0.0);
    println!("Cost: ${:.6}", cost);
}

#[tokio::test]
#[ignore]
async fn test_anthropic_integration() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set for integration tests");

    let config = config::AnthropicConfig {
        api_key: Some(api_key),
        base_url: "https://api.anthropic.com/v1".to_string(),
        default_model: "claude-3-haiku-20240307".to_string(),
        temperature: 0.7,
        max_tokens: 100,
    };

    let provider = AnthropicProvider::new(config, None).expect("Failed to create provider");

    let request = ChatRequest::new(vec![
        Message::system("You are a helpful assistant."),
        Message::user("Count from 1 to 5."),
    ]);

    let response = provider.chat(request).await.expect("Failed to get response");

    assert_eq!(response.choices.len(), 1);
    assert!(response.usage.total_tokens > 0);
}

#[tokio::test]
#[ignore]
async fn test_registry() {
    let mut config = LlmConfig::default();
    config.providers.openai = config::OpenAiConfig::from_env();
    config.providers.anthropic = config::AnthropicConfig::from_env();

    let registry = create_default_registry(config).await.expect("Failed to create registry");

    // Test parsing
    let (provider, model) = registry
        .parse_model_string("openai:gpt-4o")
        .expect("Failed to parse");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4o");

    // Test auto-detection
    let (provider, model) = registry.parse_model_string("gpt-4o").expect("Failed to parse");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4o");

    // Test getting provider
    if std::env::var("OPENAI_API_KEY").is_ok() {
        let provider = registry
            .get_provider_for_model("gpt-4o-mini")
            .expect("Failed to get provider");

        let request = ChatRequest::new(vec![Message::user("Hello!")]);
        let response = provider.chat(request).await.expect("Failed to chat");

        assert!(response.choices.len() > 0);
    }
}

#[tokio::test]
#[ignore]
async fn test_tool_calling() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set for integration tests");

    let config = config::OpenAiConfig {
        api_key: Some(api_key),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        default_model: "gpt-4o-mini".to_string(),
        temperature: 0.0,
        max_tokens: Some(200),
    };

    let provider = OpenAiProvider::new(config, None).expect("Failed to create provider");

    let tool = Tool {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "get_weather".to_string(),
            description: "Get the current weather".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city name"
                    }
                },
                "required": ["location"]
            }),
        },
    };

    let request = ChatRequest::new(vec![Message::user(
        "What's the weather in San Francisco?",
    )])
    .with_tools(vec![tool]);

    let response = provider.chat(request).await.expect("Failed to get response");

    // Should trigger a tool call
    if let Some(tool_calls) = &response.choices[0].message.tool_calls {
        assert!(tool_calls.len() > 0);
        assert_eq!(tool_calls[0].function.name, "get_weather");
        println!("Tool call: {:?}", tool_calls[0]);
    }
}

#[tokio::test]
async fn test_token_counting() {
    use my_framework::llm::tokens::*;

    let counter = SimpleTokenCounter;

    // Test simple text
    let tokens = counter.count_tokens("Hello, world!");
    assert!(tokens > 0);
    assert!(tokens < 10);

    // Test messages
    let messages = vec![
        Message::system("You are helpful."),
        Message::user("Hello!"),
        Message::assistant("Hi there!"),
    ];

    let total = counter.count_message_tokens(&messages);
    assert!(total > 10); // Should include overhead
}

#[test]
fn test_model_info() {
    use my_framework::llm::providers::*;

    // Test OpenAI models
    let config = config::OpenAiConfig {
        api_key: Some("test".to_string()),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        default_model: "gpt-4o".to_string(),
        temperature: 0.7,
        max_tokens: None,
    };

    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string())).unwrap();
    let info = provider.model_info();

    assert_eq!(info.provider, "openai");
    assert_eq!(info.model_id, "gpt-4o");
    assert!(info.supports_tools);
    assert!(info.supports_vision);
    assert_eq!(info.context_window, 128_000);
}

#[test]
fn test_error_types() {
    use my_framework::llm::error::*;

    // Test retryable errors
    let network_error = LlmError::NetworkError("Connection failed".to_string());
    assert!(network_error.is_retryable());

    let rate_limit = LlmError::RateLimitError("Too many requests".to_string());
    assert!(rate_limit.is_retryable());
    assert!(rate_limit.is_rate_limit());

    // Test non-retryable
    let auth_error = LlmError::AuthenticationError("Invalid key".to_string());
    assert!(!auth_error.is_retryable());
    assert!(auth_error.is_authentication());
}

#[test]
fn test_cost_calculation() {
    let info = ModelInfo {
        provider: "test".to_string(),
        model_id: "test-model".to_string(),
        display_name: "Test".to_string(),
        context_window: 8192,
        max_output_tokens: 4096,
        supports_tools: false,
        supports_vision: false,
        supports_structured_output: false,
        cost_per_1k_input_tokens: 0.01,
        cost_per_1k_output_tokens: 0.03,
    };

    let usage = Usage::new(1000, 500);
    let cost = usage.calculate_cost(&info);

    // (1000 / 1000) * 0.01 + (500 / 1000) * 0.03 = 0.01 + 0.015 = 0.025
    assert!((cost - 0.025).abs() < 0.0001);
}
