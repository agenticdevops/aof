# LLM Provider Abstraction Layer - Design Document

## Overview

The LLM Provider Abstraction Layer provides a unified, high-performance interface for interacting with multiple Large Language Model providers in Rust. It supports the `provider:model_id` format for easy model selection and switching.

## Architecture

### Core Components

1. **Traits** (`src/llm/core.rs`)
   - `LlmProvider`: Main trait all providers implement
   - Unified request/response types
   - Streaming support
   - Tool/function calling support

2. **Configuration** (`src/llm/config.rs`)
   - Per-provider configuration structs
   - Environment variable fallbacks
   - Sensible defaults

3. **Registry** (`src/llm/registry.rs`)
   - Model string parsing (`provider:model_id`)
   - Auto-detection from model names
   - Provider management

4. **Providers** (`src/llm/providers/`)
   - OpenAI (GPT-4o, o1, o3)
   - Anthropic (Claude 3.5)
   - Google (Gemini 2.0)
   - Ollama (local models)

5. **Utilities**
   - Token counting
   - Error handling
   - Cost estimation

## Usage

### Basic Example

```rust
use my_framework::llm::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Method 1: Direct provider usage
    let config = config::OpenAiConfig::from_env().unwrap();
    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    let request = ChatRequest::new(vec![
        Message::system("You are helpful."),
        Message::user("Hello!"),
    ]);

    let response = provider.chat(request).await?;
    println!("{}", response.choices[0].message.content);

    // Method 2: Using registry with model strings
    let config = LlmConfig::default();
    let registry = create_default_registry(config).await?;

    let provider = registry.get_provider_for_model("openai:gpt-4o")?;
    let response = provider.chat(request).await?;

    Ok(())
}
```

### Model String Format

```rust
// Explicit provider:model format
"openai:gpt-4o"
"anthropic:claude-3-5-sonnet-20241022"
"google:gemini-2.0-flash-exp"
"ollama:llama3"

// Auto-detection (infers provider from model name)
"gpt-4o"           // -> openai
"claude-opus"      // -> anthropic
"gemini-flash"     // -> google
```

### Advanced Features

#### Tool Calling

```rust
let tool = Tool {
    tool_type: "function".to_string(),
    function: FunctionDefinition {
        name: "get_weather".to_string(),
        description: "Get weather for a location".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }),
    },
};

let request = ChatRequest::new(messages).with_tools(vec![tool]);
let response = provider.chat(request).await?;

if let Some(tool_calls) = &response.choices[0].message.tool_calls {
    for call in tool_calls {
        println!("Tool: {}", call.function.name);
    }
}
```

#### Vision/Multimodal

```rust
let message = Message {
    role: Role::User,
    content: MessageContent::Parts(vec![
        ContentPart::Text { text: "What's in this image?".to_string() },
        ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.jpg".to_string(),
                detail: Some("high".to_string()),
            },
        },
    ]),
    // ... other fields
};
```

#### Streaming

```rust
let request = ChatRequest::new(messages).with_stream(true);
let mut stream = provider.chat_stream(request).await?;

while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.delta.content {
        print!("{}", content);
    }
}
```

#### Cost Tracking

```rust
let response = provider.chat(request).await?;
let cost = response.usage.calculate_cost(provider.model_info());
println!("Cost: ${:.6}", cost);
println!("Tokens: {}", response.usage.total_tokens);
```

## Supported Providers

### OpenAI

**Models:**
- GPT-4o (128k context, vision, tools)
- GPT-4o-mini (128k context, vision, tools)
- o1/o1-preview (128k context, vision)
- o1-mini (128k context, vision)

**Configuration:**
```rust
OpenAiConfig {
    api_key: Some("sk-...".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
    organization_id: None,
    default_model: "gpt-4o".to_string(),
    temperature: 0.7,
    max_tokens: None,
}
```

**Environment Variables:**
- `OPENAI_API_KEY` (required)
- `OPENAI_ORG_ID` (optional)

### Anthropic

**Models:**
- Claude 3.5 Sonnet (200k context, vision, tools)
- Claude 3 Opus (200k context, vision, tools)
- Claude 3 Haiku (200k context, vision, tools)

**Configuration:**
```rust
AnthropicConfig {
    api_key: Some("sk-ant-...".to_string()),
    base_url: "https://api.anthropic.com/v1".to_string(),
    default_model: "claude-3-5-sonnet-20241022".to_string(),
    temperature: 0.7,
    max_tokens: 4096,
}
```

**Environment Variables:**
- `ANTHROPIC_API_KEY` (required)

### Google

**Models:**
- Gemini 2.0 Flash (1M context, vision, tools)
- Gemini 1.5 Pro (2M context, vision, tools)

**Configuration:**
```rust
GoogleConfig {
    api_key: Some("AIza...".to_string()),
    base_url: "https://generativelanguage.googleapis.com/v1".to_string(),
    default_model: "gemini-2.0-flash-exp".to_string(),
    temperature: 0.7,
}
```

**Environment Variables:**
- `GOOGLE_API_KEY` (required)

### Ollama

**Models:**
- Any locally available model (llama3, mistral, etc.)
- Vision models (llava, bakllava)

**Configuration:**
```rust
OllamaConfig {
    base_url: "http://localhost:11434".to_string(),
    default_model: "llama3".to_string(),
    temperature: 0.7,
}
```

**Features:**
- No API key required
- Free to use
- Runs locally

## Performance Features

### Connection Pooling

Uses `reqwest::Client` with connection pooling for efficient HTTP connections.

### Retry Logic

```rust
// Built into LlmConfig
LlmConfig {
    retry_enabled: true,
    max_retries: 3,
    retry_delay: Duration::from_millis(1000),
    // ...
}
```

### Circuit Breaker

```rust
LlmConfig {
    circuit_breaker_enabled: true,
    circuit_breaker_threshold: 5,
    // ...
}
```

### Token Counting

```rust
use my_framework::llm::tokens::*;

let counter = get_token_counter("openai", Some("gpt-4"));
let tokens = counter.count_tokens("Hello world");
let message_tokens = counter.count_message_tokens(&messages);
```

## Error Handling

```rust
match provider.chat(request).await {
    Ok(response) => { /* handle success */ },
    Err(LlmError::AuthenticationError(msg)) => {
        eprintln!("Auth error: {}", msg);
    },
    Err(LlmError::RateLimitError(msg)) => {
        // Implement backoff
    },
    Err(LlmError::NetworkError(msg)) => {
        // Retry if transient
    },
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Future Providers

### Planned

1. **Docker Model Runner** - Local models via Docker
2. **Grok** - xAI's models
3. **Azure OpenAI** - Enterprise deployment
4. **AWS Bedrock** - AWS-hosted models

### Extension Points

To add a new provider:

1. Create `src/llm/providers/yourprovider.rs`
2. Implement `LlmProvider` trait
3. Add configuration to `config.rs`
4. Register in `mod.rs`
5. Add model detection to `registry.rs`

## Testing

```bash
# Unit tests
cargo test --lib llm

# Integration tests (requires API keys)
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
cargo test --test llm_integration
```

## Examples

See `/examples/llm_usage.rs` for comprehensive examples:

```bash
cargo run --example llm_usage
```

## Cost Optimization

```rust
// Track costs across requests
let mut total_cost = 0.0;

for request in requests {
    let response = provider.chat(request).await?;
    let cost = response.usage.calculate_cost(provider.model_info());
    total_cost += cost;
    println!("Request cost: ${:.6}", cost);
}

println!("Total cost: ${:.6}", total_cost);
```

## Best Practices

1. **Use the Registry**: Easier to switch between providers
2. **Handle Errors**: Implement retry logic for transient failures
3. **Track Costs**: Monitor token usage and costs
4. **Choose Right Model**: Balance cost, speed, and capability
5. **Use Streaming**: For better UX in interactive applications
6. **Token Limits**: Check context window before sending large inputs

## Dependencies

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
thiserror = "1.0"
```

## License

Same as parent project.
