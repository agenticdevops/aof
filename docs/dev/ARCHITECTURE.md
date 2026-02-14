# AOF Architecture

**Last Updated:** 2026-02-14 (Phase 6 complete)

This document describes the internal architecture of AOF for developers contributing to the project.

## Crate Structure

```
aof/
├── aof-core              # Core traits, types, coordination events
├── aof-llm               # LLM provider abstraction (Anthropic, OpenAI, Google, Ollama, Groq, Bedrock)
├── aof-mcp               # MCP client implementation
├── aof-runtime           # Agent execution runtime
├── aof-memory            # Memory backends
├── aof-tools             # Built-in tool implementations
├── aof-triggers          # Event trigger system
├── aof-coordination      # Event broadcasting and session persistence
├── aof-personas          # Agent persona system (AGENTS.md/SOUL.md)
├── aof-conversational    # Intent classification & specialist handlers (Phase 6)
└── aofctl                # CLI binary with REST API & WebSocket
```

**New in Phase 6:** `aof-conversational` crate for conversational agent configuration (intent classification, 4 specialist handlers, session management)

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

## AgentFlow Architecture

### Flow Router

The FlowRouter routes incoming events to matching AgentFlows based on:

```rust
pub struct FlowRouter {
    registry: Arc<FlowRegistry>,
}

impl FlowRouter {
    pub async fn route(&self, platform: &str, channel: Option<&str>,
                        user: Option<&str>, text: Option<&str>) -> Option<Arc<AgentFlowSpec>> {
        // Match flows by platform, channel, user, and message pattern
    }
}
```

### Flow Registry

```rust
pub struct FlowRegistry {
    flows: HashMap<String, Arc<AgentFlowSpec>>,
}

impl FlowRegistry {
    pub async fn from_directory(path: &Path) -> Result<Self> {
        // Load all AgentFlow YAML files from directory
    }
}
```

### Execution Context

Each AgentFlow can specify its own execution context:

```rust
pub struct FlowContext {
    pub kubeconfig: Option<String>,
    pub namespace: Option<String>,
    pub cluster: Option<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}
```

### Trigger Config Filtering

```rust
pub struct TriggerConfig {
    pub events: Vec<String>,
    pub channels: Option<Vec<String>>,    // Channel filtering
    pub users: Option<Vec<String>>,       // User filtering
    pub patterns: Option<Vec<String>>,    // Regex pattern matching
    pub bot_token: Option<String>,
    pub signing_secret: Option<String>,
}
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

## Phase 5: Agent Personas

The persona system gives agents distinct personalities, communication styles, and visual identities.

### Crate: aof-personas

```
crates/aof-personas/
├── src/
│   ├── lib.rs          # Module declarations and re-exports
│   ├── types.rs        # Agent, Soul, SoulFrontmatter, AgentsFile
│   ├── loader.rs       # AgentLoader, SoulLoader, AgentCache
│   ├── composer.rs     # PromptComposer (7-layer instruction composition)
│   ├── events.rs       # Introduction event builders
│   ├── metrics.rs      # ReliabilityMetrics, ReliabilityCache
│   ├── validation.rs   # Structural validation, prompt injection detection
│   └── watcher.rs      # File watching with debounce
└── tests/              # 7 integration test files
```

### Data Flow

```
AGENTS.md ──> AgentLoader ──> Vec<Agent>
                                │
                ┌───────────────┼───────────────┐
                ▼               ▼               ▼
         PromptComposer   IntroEvents      REST API
         (system prompt)  (broadcast)      (/api/config)
                │               │               │
                ▼               ▼               ▼
         AgentExecutor    WebSocket ──> Mission Control UI
         (LLM context)   subscribers    (AgentCard, toasts)
```

### Key Types

- `Agent` -- Structured agent identity from AGENTS.md
- `Soul` -- Personality guidance from SOUL.md (YAML frontmatter + prose)
- `PromptComposer` -- 7-layer system prompt composition with caching
- `ReliabilityCache` -- Thread-safe metrics computation from event history
- `PersonaWatcher` -- Filesystem change detection with debounced reload

See [persona-system.md](./persona-system.md) for the complete developer guide.

## Phase 6: Conversational Configuration (aof-conversational)

### Overview

Phase 6 adds a **conversational interface** for creating agents via natural language. The `aof-conversational` crate implements:

1. **Intent Classification** - Understands what user wants (create agent, build squad, configure schedule, teach skill)
2. **Orchestrator** - Routes requests to specialized handlers with confidence-based routing
3. **4 Specialist Handlers** - Implement specific agent/squad/schedule/skill creation
4. **Session Management** - In-memory LRU cache (100 sessions, 30-min TTL)
5. **REST API** - 5 endpoints for session management and file persistence
6. **React UI** - Chat interface with file preview and confirmation workflow

### Architecture

```
User Input (Chat)
        │
        ▼
┌───────────────────────┐
│ Intent Classifier     │  (LLM-based classification with few-shot examples)
├───────────────────────┤
│ Confidence Routing    │
├───────────────────────┤
│ ├─ HIGH (≥0.8)       │  → Route to Specialist
│ ├─ MEDIUM (0.5-0.79) │  → Ask clarifying questions
│ └─ LOW (<0.5)        │  → Show error with examples
└───────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ Specialist Handlers (one per intent)                    │
├─────────────────────────────────────────────────────────┤
│ • AgentCreator (06-02)      - Create single agents     │
│ • SquadBuilder (06-03)      - Build agent squads       │
│ • SkillTeacher (06-03)      - Create skill.md files    │
│ • Scheduler (06-04)         - Configure cron schedules │
└─────────────────────────────────────────────────────────┘
        │
        ▼
File Generation & Preview
        │
        ▼
User Confirmation (Confirm or Cancel)
        │
        ▼
Atomic File Persistence (AGENTS.md, SOUL.md, SKILL.md, triggers.yaml)
```

### Provider-Agnostic LLM Design

**All LLM calls route through `aof-llm` `Model` trait:**
- No hardcoded model strings (e.g., no `"claude-3-5-sonnet"` in code)
- Uses `llm.default_model()` for provider flexibility
- Default: Gemini 2.5 Flash (cost-efficient, low latency)
- Supported: Anthropic, OpenAI, Google, Ollama, Groq, Bedrock

### Key Types

```rust
pub enum IntentType {
    CreateAgent,        // "I need a K8s monitoring agent"
    BuildSquad,         // "Build incident response squad"
    ConfigureSchedule,  // "Check cluster every 30 min"
    TeachSkill,         // "Learn how to debug Postgres"
    Unknown,            // Unrecognized
}

pub struct ConversationSession {
    pub id: String,
    pub messages: Vec<ConversationMessage>,
    pub pending_files: HashMap<String, String>,  // path -> content
    pub last_intent: Option<IntentType>,
}

pub struct OrchestratorResponse {
    pub intent: IntentType,
    pub response_type: ResponseType,  // SpecialistResult | ClarifyingQuestions | Error
    pub files: HashMap<String, String>,
    pub message: String,
}
```

### Input Sanitization

Detects 6 prompt injection patterns:
1. `ignore/disregard/forget` + `previous/above/prior` + `instructions`
2. `you are now` / `act as` / `pretend to be`
3. `override/bypass/ignore` + `system/safety/constraint`
4. `system prompt`
5. `new instructions`
6. `ignore the above`

### Session Management

- **Storage:** In-memory LRU cache (bounded at 100 sessions)
- **Memory:** ~1MB worst case (100 sessions × ~10KB each)
- **TTL:** 30 minutes of inactivity (lazy cleanup on `get()`)
- **Eviction:** LRU removes oldest when at capacity
- **Thread Safety:** `Arc<RwLock<>>` for concurrent access

### REST API Endpoints

```
POST   /api/conversation/session           → Create session
GET    /api/conversation/session/{id}      → Get session + history
POST   /api/conversation/message           → Send message
POST   /api/conversation/confirm           → Persist files
POST   /api/conversation/cancel            → Discard pending
```

### Testing

- **MockModel Pattern:** All tests use `MockModel` (no API keys)
- **Coverage:** 47+ unit tests across all intents
- **Integration:** Multi-turn conversations, file confirmation, prompt injection blocking
- **Test Framework:** `#[tokio::test]` with `#[ignore]` for optional real API tests

### Performance

| Operation | Latency |
|-----------|---------|
| Intent Classification | 100-500ms (LLM call) |
| File Persistence | <1ms (filesystem) |
| Session CRUD | <1ms (in-memory LRU) |
| WebSocket Notification | <10ms (broadcast) |

### Related Documentation

- **[PHASE-6-IMPLEMENTATION-SUMMARY.md](./PHASE-6-IMPLEMENTATION-SUMMARY.md)** - Complete overview of all 5 plans
- **[conversational-architecture.md](./conversational-architecture.md)** - Detailed technical design
- **[conversation-api.md](./conversation-api.md)** - REST API reference & testing
- **[squad-templates.md](./squad-templates.md)** - Squad template specifications
- **[agent-generation-pipeline.md](./agent-generation-pipeline.md)** - Agent creation flow

### Future Extensions (Phase 7+)

- Agent modification intents (`modify_agent`, `delete_agent`, `list_agents`)
- Deployment intents (`deploy_agent`, `rollback_agent`)
- Persistent session storage (Redis, PostgreSQL)
- Streaming responses (SSE for real-time feedback)
- Voice interface (speech-to-text)

## Contributing

1. Fork the repository
2. Create a feature branch from `dev`
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Submit PR against `dev` branch

See [CONTRIBUTING.md](./CONTRIBUTING.md) for full guidelines.
