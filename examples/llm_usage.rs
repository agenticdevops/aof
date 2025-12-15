use my_framework::llm::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Example 1: Using OpenAI
    example_openai().await?;

    // Example 2: Using Anthropic
    example_anthropic().await?;

    // Example 3: Using the registry with model strings
    example_registry().await?;

    // Example 4: Tool calling
    example_tools().await?;

    // Example 5: Vision
    example_vision().await?;

    // Example 6: Streaming
    example_streaming().await?;

    Ok(())
}

async fn example_openai() -> Result<()> {
    println!("\n=== OpenAI Example ===");

    // Configure OpenAI
    let config = config::OpenAiConfig {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap()),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        default_model: "gpt-4o".to_string(),
        temperature: 0.7,
        max_tokens: None,
    };

    // Create provider
    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    // Create a simple chat request
    let request = ChatRequest::new(vec![
        Message::system("You are a helpful assistant."),
        Message::user("What is the capital of France?"),
    ])
    .with_temperature(0.5)
    .with_max_tokens(100);

    // Send request
    let response = provider.chat(request).await?;

    // Print response
    println!("Model: {}", response.model);
    if let Some(choice) = response.choices.first() {
        if let MessageContent::Text(text) = &choice.message.content {
            println!("Response: {}", text);
        }
    }
    println!("Tokens used: {}", response.usage.total_tokens);
    println!(
        "Cost: ${:.6}",
        response.usage.calculate_cost(provider.model_info())
    );

    Ok(())
}

async fn example_anthropic() -> Result<()> {
    println!("\n=== Anthropic Example ===");

    // Configure Anthropic
    let config = config::AnthropicConfig {
        api_key: Some(std::env::var("ANTHROPIC_API_KEY").unwrap()),
        base_url: "https://api.anthropic.com/v1".to_string(),
        default_model: "claude-3-5-sonnet-20241022".to_string(),
        temperature: 0.7,
        max_tokens: 4096,
    };

    // Create provider
    let provider = AnthropicProvider::new(config, None)?;

    // Create request
    let request = ChatRequest::new(vec![
        Message::system("You are a helpful coding assistant."),
        Message::user("Write a simple 'hello world' in Rust."),
    ]);

    // Send request
    let response = provider.chat(request).await?;

    // Print response
    if let Some(choice) = response.choices.first() {
        if let MessageContent::Text(text) = &choice.message.content {
            println!("Response: {}", text);
        }
    }

    Ok(())
}

async fn example_registry() -> Result<()> {
    println!("\n=== Registry Example ===");

    // Create configuration
    let mut config = LlmConfig::default();
    config.providers.openai = config::OpenAiConfig::from_env();
    config.providers.anthropic = config::AnthropicConfig::from_env();
    config.providers.google = config::GoogleConfig::from_env();

    // Create registry with auto-discovery
    let registry = create_default_registry(config).await?;

    // Use model strings to get providers
    let test_models = vec![
        "openai:gpt-4o",
        "anthropic:claude-3-5-sonnet-20241022",
        "google:gemini-2.0-flash-exp",
        "gpt-4o",        // Auto-detect OpenAI
        "claude-opus",   // Auto-detect Anthropic
        "gemini-flash",  // Auto-detect Google
    ];

    for model_string in test_models {
        match registry.parse_model_string(model_string) {
            Ok((provider, model)) => {
                println!("Model string '{}' -> provider: {}, model: {}", model_string, provider, model);
            }
            Err(e) => {
                println!("Failed to parse '{}': {}", model_string, e);
            }
        }
    }

    // List all available providers
    println!("\nAvailable providers: {:?}", registry.list_providers());

    // Use a provider
    if let Ok(provider) = registry.get_provider_for_model("openai:gpt-4o") {
        let request = ChatRequest::new(vec![Message::user("Say hello!")]);
        let response = provider.chat(request).await?;
        println!("\nGot response from: {}", response.model);
    }

    Ok(())
}

async fn example_tools() -> Result<()> {
    println!("\n=== Tool Calling Example ===");

    let config = config::OpenAiConfig::from_env()
        .ok_or_else(|| LlmError::ConfigurationError("No OpenAI config".to_string()))?;

    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    // Define a tool
    let weather_tool = Tool {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "get_weather".to_string(),
            description: "Get the current weather in a location".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"]
                    }
                },
                "required": ["location"]
            }),
        },
    };

    // Create request with tools
    let request = ChatRequest::new(vec![
        Message::system("You are a helpful assistant with access to tools."),
        Message::user("What's the weather in San Francisco?"),
    ])
    .with_tools(vec![weather_tool]);

    // Send request
    let response = provider.chat(request).await?;

    // Check for tool calls
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = &choice.message.tool_calls {
            println!("Assistant wants to call tools:");
            for tool_call in tool_calls {
                println!("  - {}: {}", tool_call.function.name, tool_call.function.arguments);
            }
        }
    }

    Ok(())
}

async fn example_vision() -> Result<()> {
    println!("\n=== Vision Example ===");

    let config = config::OpenAiConfig::from_env()
        .ok_or_else(|| LlmError::ConfigurationError("No OpenAI config".to_string()))?;

    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    if !provider.supports_vision() {
        println!("Model doesn't support vision");
        return Ok(());
    }

    // Create multimodal message
    let message = Message {
        role: Role::User,
        content: MessageContent::Parts(vec![
            ContentPart::Text {
                text: "What's in this image?".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "https://example.com/image.jpg".to_string(),
                    detail: Some("high".to_string()),
                },
            },
        ]),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    let request = ChatRequest::new(vec![message]);
    let response = provider.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        if let MessageContent::Text(text) = &choice.message.content {
            println!("Vision response: {}", text);
        }
    }

    Ok(())
}

async fn example_streaming() -> Result<()> {
    println!("\n=== Streaming Example ===");

    let config = config::OpenAiConfig::from_env()
        .ok_or_else(|| LlmError::ConfigurationError("No OpenAI config".to_string()))?;

    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    let request = ChatRequest::new(vec![
        Message::system("You are a helpful assistant."),
        Message::user("Count to 10."),
    ])
    .with_stream(true);

    let mut stream = provider.chat_stream(request).await?;

    print!("Streaming: ");
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                if let Some(content) = chunk.delta.content {
                    print!("{}", content);
                }
            }
            Err(e) => {
                eprintln!("\nStreaming error: {}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
