# LLM Provider Abstraction - Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
my-framework = { path = "." } # or version from crates.io
tokio = { version = "1.35", features = ["full"] }
```

## 30-Second Start

```rust
use my_framework::llm::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Configure (uses environment variables)
    let config = config::OpenAiConfig::from_env().unwrap();

    // 2. Create provider
    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    // 3. Send request
    let response = provider.chat(
        ChatRequest::new(vec![
            Message::user("What is 2+2?")
        ])
    ).await?;

    // 4. Get result
    println!("{:?}", response.choices[0].message.content);

    Ok(())
}
```

## Environment Setup

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Google
export GOOGLE_API_KEY="AIza..."

# Ollama (local - no key needed)
# Just run: ollama serve
```

## Common Use Cases

### 1. Simple Chat

```rust
let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

let response = provider.chat(
    ChatRequest::new(vec![
        Message::system("You are a helpful coding assistant."),
        Message::user("Write a Fibonacci function in Rust."),
    ])
    .with_temperature(0.7)
    .with_max_tokens(500)
).await?;
```

### 2. Using Model Strings (Recommended)

```rust
// Create registry once
let config = LlmConfig::default();
let registry = create_default_registry(config).await?;

// Switch between providers easily
let providers = vec![
    "openai:gpt-4o",
    "anthropic:claude-3-5-sonnet-20241022",
    "google:gemini-2.0-flash-exp",
    "ollama:llama3",
];

for model_string in providers {
    let provider = registry.get_provider_for_model(model_string)?;
    let response = provider.chat(request.clone()).await?;
    println!("{}: {:?}", model_string, response);
}
```

### 3. Tool Calling

```rust
let tool = Tool {
    tool_type: "function".to_string(),
    function: FunctionDefinition {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            }
        }),
    },
};

let response = provider.chat(
    ChatRequest::new(vec![
        Message::user("Search for Rust tutorials")
    ])
    .with_tools(vec![tool])
).await?;

if let Some(tool_calls) = &response.choices[0].message.tool_calls {
    println!("Tool: {}", tool_calls[0].function.name);
}
```

### 4. Streaming Responses

```rust
use futures::StreamExt;

let mut stream = provider.chat_stream(
    ChatRequest::new(vec![
        Message::user("Write a story")
    ])
    .with_stream(true)
).await?;

while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.delta.content {
        print!("{}", content);
    }
}
```

### 5. Vision/Images

```rust
let message = Message {
    role: Role::User,
    content: MessageContent::Parts(vec![
        ContentPart::Text {
            text: "What's in this image?".to_string()
        },
        ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.jpg".to_string(),
                detail: Some("high".to_string()),
            },
        },
    ]),
    // ... other fields
};

let response = provider.chat(
    ChatRequest::new(vec![message])
).await?;
```

### 6. Cost Tracking

```rust
let response = provider.chat(request).await?;

println!("Tokens used: {}", response.usage.total_tokens);
println!("Input tokens: {}", response.usage.prompt_tokens);
println!("Output tokens: {}", response.usage.completion_tokens);

let cost = response.usage.calculate_cost(provider.model_info());
println!("Cost: ${:.6}", cost);
```

## Model Selection Guide

### For Speed (Cheapest)
- `openai:gpt-4o-mini` - Fast, cheap, good quality
- `anthropic:claude-3-haiku-20240307` - Very fast
- `google:gemini-2.0-flash-exp` - FREE during preview
- `ollama:llama3` - FREE, local

### For Quality (Best)
- `openai:o1` - Reasoning tasks
- `anthropic:claude-3-5-sonnet-20241022` - Coding, analysis
- `google:gemini-1.5-pro` - Large context

### For Balance
- `openai:gpt-4o` - General purpose
- `anthropic:claude-3-5-sonnet-20241022` - Coding
- `google:gemini-2.0-flash-exp` - Fast + good

### For Local/Free
- `ollama:llama3` - General
- `ollama:mistral` - Fast
- `ollama:llava` - Vision support

## Provider Comparison

| Provider | Best For | Context | Tools | Vision | Cost/1M |
|----------|----------|---------|-------|--------|---------|
| OpenAI GPT-4o | General purpose | 128k | ✅ | ✅ | $2.50/$10 |
| OpenAI o1 | Reasoning | 128k | ❌ | ✅ | $15/$60 |
| Claude 3.5 | Coding | 200k | ✅ | ✅ | $3/$15 |
| Gemini Flash | Speed | 1M | ✅ | ✅ | FREE* |
| Ollama | Privacy | 8k+ | ⚠️ | ⚠️ | FREE |

*During preview period

## Error Handling

```rust
match provider.chat(request).await {
    Ok(response) => {
        // Success
    },
    Err(LlmError::AuthenticationError(_)) => {
        eprintln!("Check your API key!");
    },
    Err(LlmError::RateLimitError(_)) => {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // Retry
    },
    Err(LlmError::NetworkError(_)) => {
        // Connection issue, retry
    },
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Configuration Best Practices

### 1. Use Environment Variables

```rust
// Automatic from env vars
let config = config::OpenAiConfig::from_env().unwrap();
```

### 2. Or Explicit Configuration

```rust
let config = config::OpenAiConfig {
    api_key: Some("sk-...".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
    organization_id: None,
    default_model: "gpt-4o".to_string(),
    temperature: 0.7,
    max_tokens: None,
};
```

### 3. Global Configuration

```rust
let mut config = LlmConfig::default();
config.timeout = Duration::from_secs(60);
config.retry_enabled = true;
config.max_retries = 3;

// Add providers
config.providers.openai = Some(openai_config);
config.providers.anthropic = Some(anthropic_config);
```

## Testing

```bash
# Unit tests
cargo test --lib llm

# Integration tests (requires API keys)
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
cargo test --test llm_integration_test -- --ignored

# Run examples
cargo run --example llm_usage
```

## Advanced Topics

### Retry with Backoff

```rust
use tokio::time::{sleep, Duration};

async fn chat_with_retry(
    provider: &impl LlmProvider,
    request: ChatRequest,
    max_retries: usize,
) -> Result<ChatResponse> {
    let mut retries = 0;

    loop {
        match provider.chat(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_retryable() && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_millis(1000 * 2_u64.pow(retries as u32));
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Fallback Chain

```rust
async fn chat_with_fallback(
    registry: &ProviderRegistry,
    request: ChatRequest,
) -> Result<ChatResponse> {
    let providers = vec![
        "openai:gpt-4o-mini",
        "anthropic:claude-3-haiku-20240307",
        "ollama:llama3",
    ];

    for model in providers {
        if let Ok(provider) = registry.get_provider_for_model(model) {
            if let Ok(response) = provider.chat(request.clone()).await {
                return Ok(response);
            }
        }
    }

    Err(LlmError::ProviderError("All providers failed".to_string()))
}
```

### Parallel Requests

```rust
use futures::future::join_all;

let requests = vec![request1, request2, request3];
let futures: Vec<_> = requests
    .into_iter()
    .map(|req| provider.chat(req))
    .collect();

let responses = join_all(futures).await;
```

## Next Steps

1. Read [LLM_PROVIDER_DESIGN.md](./LLM_PROVIDER_DESIGN.md) for architecture details
2. Check [examples/llm_usage.rs](../examples/llm_usage.rs) for full examples
3. Review [tests/llm_integration_test.rs](../tests/llm_integration_test.rs) for test patterns
4. See Cargo.toml for dependency versions

## Common Issues

### "API key not found"
- Set environment variables: `export OPENAI_API_KEY="sk-..."`
- Or pass explicitly in config

### "Connection refused" (Ollama)
- Start Ollama: `ollama serve`
- Check URL: default is `http://localhost:11434`

### "Rate limit exceeded"
- Implement exponential backoff
- Use cheaper model
- Add delays between requests

### "Context window exceeded"
- Reduce message history
- Use model with larger context
- Implement summarization

## Support

- Issues: GitHub Issues
- Docs: `/docs/LLM_*.md`
- Examples: `/examples/llm_*.rs`
