# LLM Provider Abstraction - Architecture Diagram

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Your Application                             │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Application Layer                                │ │
│  │  - Agent orchestration                                        │ │
│  │  - Workflow management                                        │ │
│  │  - Business logic                                            │ │
│  └──────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LLM Abstraction Layer                            │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Provider Registry                                │ │
│  │  - parse_model_string("provider:model")                      │ │
│  │  - get_provider_for_model()                                  │ │
│  │  - Auto-detection from model names                           │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│                              ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Unified Provider Trait                           │ │
│  │  async fn chat(request) -> Result<response>                  │ │
│  │  async fn chat_stream(request) -> Result<Stream>             │ │
│  │  fn supports_tools() -> bool                                 │ │
│  │  fn supports_vision() -> bool                                │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│          ┌───────────────────┼───────────────────┐                │
│          ▼                   ▼                   ▼                 │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐          │
│  │   OpenAI     │   │  Anthropic   │   │   Google     │   ...    │
│  │  Provider    │   │   Provider   │   │  Provider    │          │
│  └──────────────┘   └──────────────┘   └──────────────┘          │
└─────────────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    External LLM Services                            │
│                                                                     │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐          │
│  │   OpenAI     │   │  Anthropic   │   │   Google     │          │
│  │     API      │   │     API      │   │     API      │          │
│  │              │   │              │   │              │          │
│  │  GPT-4o      │   │  Claude 3.5  │   │  Gemini 2.0  │          │
│  │  o1, o3      │   │  Opus, Haiku │   │  Flash, Pro  │          │
│  └──────────────┘   └──────────────┘   └──────────────┘          │
│                                                                     │
│  ┌──────────────┐                                                  │
│  │   Ollama     │   (Local - No external API)                     │
│  │   Server     │                                                  │
│  │              │                                                  │
│  │  llama3      │                                                  │
│  │  mistral     │                                                  │
│  └──────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Request Flow

```
User Request
    │
    ▼
┌─────────────────────────────────┐
│  ChatRequest                    │
│  ├─ messages: Vec<Message>      │
│  ├─ temperature: f32            │
│  ├─ max_tokens: usize           │
│  └─ tools: Vec<Tool>            │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  Provider Registry              │
│  parse("openai:gpt-4o")         │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  OpenAI Provider                │
│  ├─ Convert to OpenAI format    │
│  ├─ Add authentication          │
│  └─ HTTP POST request           │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  OpenAI API                     │
│  ├─ Process request             │
│  ├─ Generate response           │
│  └─ Return JSON                 │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  OpenAI Provider                │
│  ├─ Parse response              │
│  ├─ Convert to unified format   │
│  └─ Calculate usage & cost      │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  ChatResponse                   │
│  ├─ choices: Vec<Choice>        │
│  ├─ usage: Usage                │
│  └─ cost: f64                   │
└─────────────────────────────────┘
    │
    ▼
User receives response
```

## Component Interaction

```
┌────────────────────────────────────────────────────────────────┐
│                         Core Module                            │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Traits     │  │    Types     │  │   Messages   │        │
│  │              │  │              │  │              │        │
│  │ LlmProvider  │  │ ChatRequest  │  │   Message    │        │
│  │              │  │ ChatResponse │  │   Content    │        │
│  │              │  │  ChatChunk   │  │    Role      │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
└────────────────────────────────────────────────────────────────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             │
        ┌────────────────────┴────────────────────┐
        │                                         │
        ▼                                         ▼
┌────────────────┐                       ┌────────────────┐
│  Providers     │                       │   Registry     │
│                │                       │                │
│  ┌──────────┐ │                       │  ┌──────────┐  │
│  │ OpenAI   │ │                       │  │  Parse   │  │
│  └──────────┘ │                       │  │  model   │  │
│  ┌──────────┐ │                       │  │  string  │  │
│  │Anthropic │ │◄──────────────────────┤  └──────────┘  │
│  └──────────┘ │                       │  ┌──────────┐  │
│  ┌──────────┐ │                       │  │  Manage  │  │
│  │  Google  │ │                       │  │providers │  │
│  └──────────┘ │                       │  └──────────┘  │
│  ┌──────────┐ │                       └────────────────┘
│  │  Ollama  │ │
│  └──────────┘ │
└────────────────┘
        │
        ▼
┌────────────────────────────────────────────────────────────────┐
│                      Utility Modules                           │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Config     │  │    Tokens    │  │    Errors    │        │
│  │              │  │              │  │              │        │
│  │ LlmConfig    │  │TokenCounter  │  │  LlmError    │        │
│  │ProviderCfg   │  │count_tokens  │  │is_retryable  │        │
│  │  from_env    │  │calc_cost     │  │is_rate_limit │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
└────────────────────────────────────────────────────────────────┘
```

## Data Flow - Streaming

```
User Request (stream=true)
    │
    ▼
Provider.chat_stream(request)
    │
    ▼
HTTP POST with stream flag
    │
    ▼
Server-Sent Events (SSE)
    │
    ▼
┌─────────────────────────────────┐
│  Event Stream                   │
│                                 │
│  data: {"delta": "Hello"}       │
│  data: {"delta": " world"}      │
│  data: {"delta": "!"}           │
│  data: [DONE]                   │
└─────────────────────────────────┘
    │
    ▼
Parse each event
    │
    ▼
┌─────────────────────────────────┐
│  Stream<ChatChunk>              │
│                                 │
│  ChatChunk { delta: "Hello" }   │
│  ChatChunk { delta: " world" }  │
│  ChatChunk { delta: "!" }       │
└─────────────────────────────────┘
    │
    ▼
User processes stream
```

## Tool Calling Flow

```
User Request with Tools
    │
    ▼
┌─────────────────────────────────┐
│  ChatRequest                    │
│  ├─ messages: [...]             │
│  └─ tools: [                    │
│       {                         │
│         name: "get_weather"     │
│         description: "..."      │
│         parameters: {...}       │
│       }                         │
│     ]                           │
└─────────────────────────────────┘
    │
    ▼
Provider processes request
    │
    ▼
┌─────────────────────────────────┐
│  ChatResponse                   │
│  └─ tool_calls: [               │
│       {                         │
│         id: "call_123"          │
│         function: {             │
│           name: "get_weather"   │
│           arguments: "{...}"    │
│         }                       │
│       }                         │
│     ]                           │
└─────────────────────────────────┘
    │
    ▼
Application executes tool
    │
    ▼
┌─────────────────────────────────┐
│  Tool Result                    │
│  {                              │
│    temperature: 72,             │
│    condition: "sunny"           │
│  }                              │
└─────────────────────────────────┘
    │
    ▼
New request with tool result
    │
    ▼
┌─────────────────────────────────┐
│  ChatRequest                    │
│  messages: [                    │
│    ...previous messages,        │
│    {                            │
│      role: "tool",              │
│      content: "{...}",          │
│      tool_call_id: "call_123"   │
│    }                            │
│  ]                              │
└─────────────────────────────────┘
    │
    ▼
Final response with context
```

## Error Handling Flow

```
Request
    │
    ▼
try {
  provider.chat(request)
}
    │
    ├─ Success ──────────► Return response
    │
    └─ Error
        │
        ▼
    Classify error
        │
        ├─ Authentication ──► Return immediately
        │                     (check API key)
        │
        ├─ Rate Limit ──────► Exponential backoff
        │                     │
        │                     ▼
        │                  Sleep 1s, 2s, 4s...
        │                     │
        │                     └─► Retry
        │
        ├─ Network ─────────► Retry with backoff
        │                     (if retryable)
        │
        ├─ Timeout ─────────► Retry with backoff
        │
        └─ Other ───────────► Return error
                              (not retryable)
```

## Cost Calculation Flow

```
ChatResponse
    │
    ▼
┌─────────────────────────────────┐
│  Usage                          │
│  ├─ prompt_tokens: 150          │
│  ├─ completion_tokens: 50       │
│  └─ total_tokens: 200           │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  ModelInfo                      │
│  ├─ cost_per_1k_input: $0.0025  │
│  └─ cost_per_1k_output: $0.01   │
└─────────────────────────────────┘
    │
    ▼
Calculate:
  input_cost = (150 / 1000) * $0.0025
             = $0.000375

  output_cost = (50 / 1000) * $0.01
              = $0.0005

  total_cost = $0.000875
    │
    ▼
Return: $0.000875
```

## Module Dependencies

```
┌─────────────┐
│   mod.rs    │  ◄── Public exports
└─────────────┘
      │
      ├─► core.rs       (traits, types)
      │     │
      │     └─► Used by all providers
      │
      ├─► config.rs     (configuration)
      │     │
      │     └─► Used by providers
      │
      ├─► error.rs      (error types)
      │     │
      │     └─► Used everywhere
      │
      ├─► registry.rs   (provider management)
      │     │
      │     ├─► Uses: config, core, error
      │     └─► Manages: providers
      │
      ├─► tokens.rs     (token counting)
      │     │
      │     └─► Uses: core
      │
      └─► providers/
            │
            ├─► openai.rs
            │     └─► Uses: core, config, error
            │
            ├─► anthropic.rs
            │     └─► Uses: core, config, error
            │
            ├─► google.rs
            │     └─► Uses: core, config, error
            │
            └─► ollama.rs
                  └─► Uses: core, config, error
```

## Multi-Provider Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    Application Layer                       │
└────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────┐
│                  Provider Registry                         │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  HashMap<String, Arc<dyn LlmProvider>>               │ │
│  │  ├─ "openai"    → OpenAiProvider                     │ │
│  │  ├─ "anthropic" → AnthropicProvider                  │ │
│  │  ├─ "google"    → GoogleProvider                     │ │
│  │  └─ "ollama"    → OllamaProvider                     │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
    │OpenAI  │    │Anthro  │    │Google  │    │Ollama  │
    │Provider│    │pic     │    │Provider│    │Provider│
    └────────┘    └────────┘    └────────┘    └────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
    │OpenAI  │    │Anthro  │    │Google  │    │Local   │
    │  API   │    │pic API │    │  API   │    │Server  │
    └────────┘    └────────┘    └────────┘    └────────┘
```

This architecture provides:
- **Unified interface** across all providers
- **Easy provider switching** via model strings
- **Fallback support** for resilience
- **Performance optimization** with connection pooling
- **Cost tracking** across all providers
- **Extensibility** for new providers
