# AOF Architecture

This document describes the internal architecture of AOF for developers contributing to the project.

## Crate Structure

```
aof/
├── aof-core       # Core traits and types
├── aof-llm        # LLM provider abstraction
├── aof-mcp        # MCP client implementation
├── aof-runtime    # Agent execution runtime
├── aof-memory     # Memory backends
├── aof-tools      # Built-in tool implementations
├── aof-triggers   # Event trigger system
└── aofctl         # CLI binary
```

## Crate Dependencies

```
                    ┌──────────┐
                    │  aofctl  │
                    └────┬─────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │aof-runtime│ │aof-triggers│ │aof-memory│
        └────┬─────┘ └────┬─────┘ └────┬─────┘
              │           │           │
    ┌─────────┼───────────┼───────────┤
    ▼         ▼           ▼           ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│ aof-llm  │ │ aof-mcp  │ │aof-tools │
└────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │
     └────────────┼────────────┘
                  ▼
            ┌──────────┐
            │ aof-core │
            └──────────┘
```

## Core Traits

### aof-core

```rust
// Agent trait - the foundation
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, ctx: &mut AgentContext) -> AofResult<String>;
    fn metadata(&self) -> &AgentMetadata;
    async fn init(&mut self) -> AofResult<()>;
    async fn cleanup(&mut self) -> AofResult<()>;
}

// Tool trait - executable capabilities
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;
    fn config(&self) -> &ToolConfig;
}

// ToolExecutor trait - manages tool execution
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult>;
    fn list_tools(&self) -> Vec<ToolDefinition>;
}

// Model trait - LLM abstraction
#[async_trait]
pub trait Model: Send + Sync {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse>;
    async fn generate_stream(&self, request: &ModelRequest)
        -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>>;
}

// Memory trait - state persistence
#[async_trait]
pub trait Memory: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> AofResult<()>;
    async fn retrieve(&self, query: MemoryQuery) -> AofResult<Vec<MemoryEntry>>;
}
```

## Agent Execution Flow

```
┌─────────────────────────────────────────────────────────┐
│                    AgentExecutor                         │
│                                                          │
│  1. Initialize                                           │
│     └─→ Load config, create model, setup tools           │
│                                                          │
│  2. Execute Loop (max_iterations)                        │
│     ┌──────────────────────────────────────────┐        │
│     │  a. Build messages (system + history)    │        │
│     │  b. Call model.generate()                │        │
│     │  c. Parse response                       │        │
│     │     ├─→ Text: Add to history             │        │
│     │     └─→ Tool calls: Execute tools        │        │
│     │         └─→ Add results to history       │        │
│     │  d. Check stop condition                 │        │
│     └──────────────────────────────────────────┘        │
│                                                          │
│  3. Cleanup                                              │
│     └─→ Close connections, cleanup resources             │
└─────────────────────────────────────────────────────────┘
```

## Tool System Architecture

### Built-in Tools (aof-tools)

```rust
// Tool registration
let mut registry = ToolRegistry::new();
registry.register(ShellTool::new());
registry.register(KubectlGetTool::new());

// Create executor from registry
let executor = registry.into_executor();

// Execute tool
let result = executor.execute_tool("shell", input).await?;
```

### Tool Categories

Each tool category is behind a feature flag:

```toml
[features]
file = []
shell = []
kubectl = []
docker = []
git = []
terraform = []
http = ["reqwest"]
observability = ["reqwest"]
aws = []
all = ["file", "shell", "kubectl", "docker", "git", "terraform", "http", "observability", "aws"]
```

### ToolSpec Type

```rust
pub enum ToolSpec {
    Simple(String),           // "shell" - backward compat
    Qualified(QualifiedToolSpec),  // Full specification
}

pub struct QualifiedToolSpec {
    pub name: String,
    pub source: ToolSource,   // Builtin or Mcp
    pub server: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
    pub timeout_secs: Option<u64>,
}
```

## MCP Integration

### Transport Types

```rust
pub enum McpTransport {
    Stdio,  // stdin/stdout with subprocess
    Sse,    // Server-Sent Events
    Http,   // HTTP request/response
}
```

### MCP Client Flow

```
┌─────────────────────────────────────────────────────┐
│                   McpClient                          │
│                                                      │
│  1. Connect                                          │
│     ├─→ Stdio: spawn process, setup pipes            │
│     ├─→ SSE: HTTP connection with EventSource        │
│     └─→ HTTP: REST client setup                      │
│                                                      │
│  2. Initialize                                       │
│     └─→ Send initialize request, receive caps        │
│                                                      │
│  3. List Tools                                       │
│     └─→ tools/list → Vec<ToolDefinition>            │
│                                                      │
│  4. Execute Tool                                     │
│     └─→ tools/call → ToolResult                     │
│                                                      │
│  5. Cleanup                                          │
│     └─→ Close transport, terminate process           │
└─────────────────────────────────────────────────────┘
```

## AgentFleet Architecture

### Coordination Modes

```rust
pub enum CoordinationMode {
    Hierarchical,  // Manager coordinates workers
    Peer,          // All agents as equals
    Swarm,         // Self-organizing
}
```

### Fleet Execution

```
┌─────────────────────────────────────────────────────┐
│                 FleetCoordinator                     │
│                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   Agent 1   │  │   Agent 2   │  │   Agent N   │ │
│  │  (Manager)  │  │  (Worker)   │  │  (Worker)   │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                │                │         │
│         └────────────────┼────────────────┘         │
│                          ▼                          │
│                 ┌─────────────────┐                 │
│                 │  SharedMemory   │                 │
│                 │  Communication  │                 │
│                 └─────────────────┘                 │
│                          ▼                          │
│                 ┌─────────────────┐                 │
│                 │    Consensus    │                 │
│                 │    Algorithm    │                 │
│                 └─────────────────┘                 │
└─────────────────────────────────────────────────────┘
```

## Workflow Engine

### Step Types

```rust
pub enum StepType {
    Agent,      // LLM-based step
    Tool,       // Direct tool execution
    Condition,  // Conditional branching
    Parallel,   // Parallel execution
    Loop,       // Iterative execution
}
```

### Workflow State Machine

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Pending │ →  │ Running │ →  │Completed│    │  Failed │
└─────────┘    └────┬────┘    └─────────┘    └─────────┘
                    │                              ▲
                    └──────────────────────────────┘
                         (on error/timeout)
```

## Error Handling

### Error Types

```rust
pub enum AofError {
    Config(String),      // Configuration errors
    Model(String),       // LLM errors
    Tool(String),        // Tool execution errors
    Agent(String),       // Agent execution errors
    Mcp(String),         // MCP protocol errors
    Memory(String),      // Memory backend errors
    Workflow(String),    // Workflow errors
    Io(std::io::Error),  // I/O errors
    Json(serde_json::Error),
}
```

### Error Knowledge Base

```rust
// Track and learn from errors
let kb = ErrorKnowledgeBase::new();
kb.record(error_record);
let similar = kb.find_similar("MCP", &["timeout"]);
```

## Testing Strategy

### Unit Tests

```bash
cargo test --lib                    # All unit tests
cargo test --lib -p aof-core        # Core crate only
cargo test --lib -p aof-tools       # Tools crate only
```

### Integration Tests

```bash
cargo test --test integration       # Integration tests
./scripts/test-agent.sh             # End-to-end test
```

### Pre-compile Validation

```bash
./scripts/test-pre-compile.sh       # Quick validation (5s)
```

## Contributing

1. Fork the repository
2. Create a feature branch from `dev`
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Submit PR against `dev` branch

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for full guidelines.
