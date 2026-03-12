# AOF LLM Providers

Multi-provider LLM abstraction layer for the Agentic Ops Framework.

## Supported Providers

### Anthropic (Claude)
- **Models**: Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku
- **Features**: Streaming, tool calling, system prompts
- **Status**: ✅ Fully implemented

### OpenAI (GPT)
- **Models**: GPT-4 Turbo, GPT-4, GPT-3.5 Turbo
- **Features**: Streaming, function calling, system messages
- **Status**: ✅ Fully implemented

### Amazon Bedrock
- **Models**: Claude via Bedrock, Titan, Llama 2
- **Features**: Streaming, tool use, multi-region support
- **Status**: ✅ Fully implemented (requires `bedrock` feature)

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
# Default: Anthropic + OpenAI
[dependencies]
aof-llm = "0.1"

# With Bedrock support
[dependencies]
aof-llm = { version = "0.1", features = ["bedrock"] }

# All providers
[dependencies]
aof-llm = { version = "0.1", features = ["all-providers"] }
```

### Basic Usage

```rust
use aof_core::{ModelConfig, ModelProvider, ModelRequest, RequestMessage};
use aof_core::model::MessageRole;
use aof_llm::provider::ProviderFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Anthropic provider
    let config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("your-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: Default::default(),
        extra: Default::default(),
    };

    let model = ProviderFactory::create(config).await?;

    // Generate completion
    let request = ModelRequest {
        messages: vec![
            RequestMessage {
                role: MessageRole::User,
                content: "Hello, how are you?".to_string(),
                tool_calls: None,
            }
        ],
        system: Some("You are a helpful assistant.".to_string()),
        tools: vec![],
        temperature: None,
        max_tokens: None,
        stream: false,
        extra: Default::default(),
    };

    let response = model.generate(&request).await?;
    println!("Response: {}", response.content);

    Ok(())
}
```

## Provider Configuration

### Anthropic

```rust
use aof_core::{ModelConfig, ModelProvider};
use std::collections::HashMap;

let config = ModelConfig {
    model: "claude-3-5-sonnet-20241022".to_string(),
    provider: ModelProvider::Anthropic,
    api_key: Some("sk-ant-...".to_string()), // Or set ANTHROPIC_API_KEY env
    endpoint: None, // Uses default https://api.anthropic.com/v1
    temperature: 0.7,
    max_tokens: Some(4096),
    timeout_secs: 60,
    headers: HashMap::new(),
    extra: HashMap::new(),
};
```

**Environment Variables:**
- `ANTHROPIC_API_KEY`: API key (alternative to config)

**Available Models:**
- `claude-3-5-sonnet-20241022` (recommended)
- `claude-3-opus-20240229`
- `claude-3-sonnet-20240229`
- `claude-3-haiku-20240307`

### OpenAI

```rust
use aof_core::{ModelConfig, ModelProvider};
use std::collections::HashMap;

let config = ModelConfig {
    model: "gpt-4-turbo-preview".to_string(),
    provider: ModelProvider::OpenAI,
    api_key: Some("sk-...".to_string()), // Or set OPENAI_API_KEY env
    endpoint: None, // Uses default https://api.openai.com/v1
    temperature: 0.7,
    max_tokens: Some(4096),
    timeout_secs: 60,
    headers: HashMap::new(),
    extra: HashMap::new(),
};
```

**Environment Variables:**
- `OPENAI_API_KEY`: API key (alternative to config)

**Available Models:**
- `gpt-4-turbo-preview`
- `gpt-4-1106-preview`
- `gpt-4`
- `gpt-3.5-turbo`

**Custom Endpoint:**
For Azure OpenAI or compatible APIs:

```rust
let mut config = ModelConfig {
    // ... other fields
    endpoint: Some("https://your-deployment.openai.azure.com/v1".to_string()),
    // ...
};
```

### Amazon Bedrock

```rust
use aof_core::{ModelConfig, ModelProvider};
use serde_json::json;
use std::collections::HashMap;

let mut extra = HashMap::new();
extra.insert("region".to_string(), json!("us-east-1"));

let config = ModelConfig {
    model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
    provider: ModelProvider::Bedrock,
    api_key: None, // Uses AWS credentials from environment
    endpoint: None,
    temperature: 0.7,
    max_tokens: Some(4096),
    timeout_secs: 60,
    headers: HashMap::new(),
    extra, // Region configuration
};
```

**Environment Variables:**
- `AWS_REGION`: AWS region (or use `extra.region` in config)
- Standard AWS credentials environment variables

**Available Models:**
- `anthropic.claude-3-5-sonnet-20241022-v2:0`
- `anthropic.claude-3-sonnet-20240229-v1:0`
- `anthropic.claude-3-haiku-20240307-v1:0`
- `amazon.titan-text-express-v1`
- `meta.llama2-70b-chat-v1`

## Features

### Streaming

All providers support streaming responses:

```rust
use futures::StreamExt;

let mut stream = model.generate_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    match chunk? {
        StreamChunk::ContentDelta { delta } => {
            print!("{}", delta);
        }
        StreamChunk::ToolCall { tool_call } => {
            println!("Tool call: {}", tool_call.name);
        }
        StreamChunk::Done { usage, stop_reason } => {
            println!("\nTokens: {} in, {} out", usage.input_tokens, usage.output_tokens);
        }
    }
}
```

### Tool Calling

Define and use tools with any provider:

```rust
use aof_core::ModelToolDefinition;
use serde_json::json;

let tools = vec![
    ModelToolDefinition {
        name: "get_weather".to_string(),
        description: "Get current weather for a location".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"]
                }
            },
            "required": ["location"]
        }),
    }
];

let request = ModelRequest {
    messages: vec![/* ... */],
    tools,
    // ...
};

let response = model.generate(&request).await?;

for tool_call in response.tool_calls {
    println!("Tool: {} with args: {:?}", tool_call.name, tool_call.arguments);
}
```

### Error Handling

All providers implement automatic retry logic with exponential backoff:

```rust
// Retries up to 3 times with exponential backoff
match model.generate(&request).await {
    Ok(response) => {
        println!("Success: {}", response.content);
    }
    Err(e) => {
        eprintln!("Failed after retries: {}", e);
    }
}
```

### Token Usage Tracking

Monitor token consumption:

```rust
let response = model.generate(&request).await?;

println!("Input tokens: {}", response.usage.input_tokens);
println!("Output tokens: {}", response.usage.output_tokens);
println!("Total tokens: {}",
    response.usage.input_tokens + response.usage.output_tokens);
```

### Token Counting

Estimate tokens before API calls:

```rust
let text = "This is a test message";
let estimated_tokens = model.count_tokens(text);
println!("Estimated tokens: {}", estimated_tokens);
```

## Advanced Usage

### Custom Headers

Add custom HTTP headers (e.g., for tracking):

```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("X-Request-ID".to_string(), "req-123".to_string());
headers.insert("X-User-ID".to_string(), "user-456".to_string());

let config = ModelConfig {
    // ... other fields
    headers,
    // ...
};
```

### Temperature and Max Tokens

Control response randomness and length:

```rust
let request = ModelRequest {
    messages: vec![/* ... */],
    temperature: Some(0.2), // More focused (0.0-1.0)
    max_tokens: Some(500),  // Limit response length
    // ...
};
```

### Multi-turn Conversations

Maintain conversation context:

```rust
use aof_core::model::MessageRole;

let request = ModelRequest {
    messages: vec![
        RequestMessage {
            role: MessageRole::User,
            content: "What's the capital of France?".to_string(),
            tool_calls: None,
        },
        RequestMessage {
            role: MessageRole::Assistant,
            content: "The capital of France is Paris.".to_string(),
            tool_calls: None,
        },
        RequestMessage {
            role: MessageRole::User,
            content: "What's its population?".to_string(),
            tool_calls: None,
        },
    ],
    // ...
};
```

## Performance

### Benchmarks

All providers include:
- **Retry logic**: 3 attempts with exponential backoff (100ms, 200ms, 400ms)
- **Timeouts**: Configurable per request (default 60s)
- **Connection pooling**: Reuses HTTP connections
- **Streaming**: Reduces latency for long responses

### Token Estimation

Approximate token counts (chars per token):
- **Anthropic**: ~3 chars/token (Claude models)
- **OpenAI**: ~4 chars/token (GPT models)
- **Bedrock**: ~3 chars/token (varies by model)

## Error Types

```rust
use aof_core::AofError;

match model.generate(&request).await {
    Ok(response) => { /* success */ }
    Err(AofError::Config(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(AofError::Model(msg)) => {
        eprintln!("Model error: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Testing

Run provider tests:

```bash
# Test default providers (Anthropic + OpenAI)
cargo test --package aof-llm

# Test with Bedrock
cargo test --package aof-llm --features bedrock

# Test all providers
cargo test --package aof-llm --features all-providers

# Test specific provider
cargo test --package aof-llm --test anthropic_tests
cargo test --package aof-llm --test openai_tests
cargo test --package aof-llm --test bedrock_tests --features bedrock
```

## Examples

See the `examples/` directory for complete examples:
- `examples/anthropic_chat.rs` - Basic chat with Claude
- `examples/openai_stream.rs` - Streaming with GPT-4
- `examples/bedrock_tools.rs` - Tool use with Bedrock
- `examples/multi_provider.rs` - Switch between providers

## Contributing

When adding a new provider:

1. Create `src/providers/your_provider.rs`
2. Implement the `Model` trait
3. Add to `ProviderFactory` in `src/provider.rs`
4. Create comprehensive tests in `tests/your_provider_tests.rs`
5. Update this documentation

## License

See LICENSE file in the repository root.
