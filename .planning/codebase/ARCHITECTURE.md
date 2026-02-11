# Architecture

**Analysis Date:** 2026-02-11

## Pattern Overview

**Overall:** Layered Microservices Architecture with Modular Trait-Based Abstraction

**Key Characteristics:**
- Pure Rust library crates providing zero-cost abstractions for agentic systems
- Provider-agnostic design (LLM, MCP, memory, tools) through trait boundaries
- kubectl-style CLI (aofctl) following Kubernetes resource patterns
- Agent execution driven by request-response loops with tool composition
- Kubernetes-inspired configuration format (Agent, Workflow, AgentFlow, Fleet as resource types)

## Layers

**Configuration Layer:**
- Purpose: Parse and validate agent/workflow/fleet specifications (YAML)
- Location: `crates/aofctl/src/commands/run.rs`, `crates/aof-core/src/agent.rs`
- Contains: YAML deserialization, validation, context loading
- Depends on: serde_yaml, serde_path_to_error for precise error messages
- Used by: Runtime initialization, resource loading

**Core Abstraction Layer:**
- Purpose: Define trait boundaries and type contracts for extensibility
- Location: `crates/aof-core/src/`
- Contains: Model trait, Tool trait, ToolExecutor, Memory trait, Agent/Workflow/Fleet types
- Depends on: async_trait, serde (zero serialization overhead)
- Used by: All other crates for interface contracts

**Provider Adapter Layer:**
- Purpose: Implement concrete providers (Anthropic, OpenAI, Google, Groq, Bedrock, Azure, Ollama)
- Location: `crates/aof-llm/src/provider/` (LLM), `crates/aof-mcp/src/` (MCP)
- Contains: Provider-specific clients and protocol adapters
- Depends on: reqwest, hyper for HTTP, provider SDKs
- Used by: Runtime during model initialization

**Memory Layer:**
- Purpose: Persistent and ephemeral state storage with lock-free concurrent access
- Location: `crates/aof-memory/src/backend/`
- Contains: InMemoryBackend (ephemeral), FileBackend (persistent JSON)
- Depends on: DashMap for concurrent writes, tokio for async I/O
- Used by: AgentExecutor for context persistence, session management

**Execution Layer (Orchestration):**
- Purpose: Execute agents, workflows, and AgentFlows with lifecycle management
- Location: `crates/aof-runtime/src/executor/`
- Contains: AgentExecutor, WorkflowExecutor, AgentFlowExecutor, Runtime factory
- Depends on: Model trait, Tool trait, Memory trait, error recovery logic
- Used by: aofctl run commands, trigger servers

**Tool Execution Layer:**
- Purpose: Abstract and execute tools (kubectl, docker, terraform, shell, HTTP, observability)
- Location: `crates/aof-tools/src/`
- Contains: ToolRegistry, built-in tools as separate modules, BuiltinToolExecutor
- Depends on: Tool trait, shell execution, cloud SDKs (AWS, GCP, Azure)
- Used by: AgentExecutor during tool_use phase

**Fleet Coordination Layer:**
- Purpose: Coordinate multiple agent instances with distributed decision-making
- Location: `crates/aof-runtime/src/fleet/`
- Contains: FleetCoordinator, consensus algorithms (Raft, Byzantine), DEEP protocol
- Depends on: Core types, error handling, state management
- Used by: Multi-agent scenarios, consensus-based decisions

**Skills System:**
- Purpose: Load, validate, and inject executable capabilities from SKILL.md files
- Location: `crates/aof-skills/src/`
- Contains: SkillRegistry, frontmatter parsing, requirements gating, hot-reload
- Depends on: File I/O, YAML parsing, pattern matching
- Used by: Runtime, agents for capability discovery

**Trigger Layer:**
- Purpose: Accept agent invocations from messaging platforms via webhooks
- Location: `crates/aof-triggers/src/`
- Contains: TriggerServer, platform adapters (Telegram, Slack, Discord, WhatsApp), SafetyContext
- Depends on: Hyper for HTTP server, Platform-specific message parsing
- Used by: Standalone trigger servers, webhook handlers

**CLI Layer:**
- Purpose: kubectl-style command interface (verb-first: `aofctl run agent <name>`)
- Location: `crates/aofctl/src/`
- Contains: Clap CLI parsing, commands (run, get, apply, delete, describe, flow, exec, serve, skills, tools, logs, etc.)
- Depends on: Runtime, resources, output formatting
- Used by: End users, CI/CD pipelines, kubectl-style workflows

## Data Flow

**Standard Agent Execution Flow:**

1. **Configuration Loading** → User provides `aofctl run agent <file>` or `aofctl run agent <name>`
2. **Parse Config** → `parse_agent_config()` in `crates/aofctl/src/commands/run.rs` validates YAML with serde_path_to_error
3. **Create Runtime** → `Runtime::new()` in `crates/aof-runtime/src/executor/runtime.rs` initializes:
   - LLM model via `aof_llm::create_model()` (provider selection)
   - Tool executor via `ToolRegistry` from `crates/aof-tools/src/registry.rs`
   - Memory backend (InMemoryBackend or FileBackend)
   - Optional MCP client via `McpClientBuilder` if mcp_servers specified
4. **Execute Agent** → `AgentExecutor::execute()` in `crates/aof-runtime/src/executor/agent_executor.rs`:
   - Build ModelRequest with agent instructions + tools + context messages
   - Call `model.generate_stream()` (streaming response)
   - Parse StopReason (EndTurn, ToolUse, MaxTokens, etc.)
   - If ToolUse: execute tool via `ToolExecutor::execute()`
   - Add ToolResult to conversation context
   - Loop until EndTurn or max_iterations
5. **Output Result** → Format response (text, JSON, YAML) and write to stdout/file

**Workflow Execution Flow:**

1. **Load Workflow** → Parse Workflow YAML with WorkflowMetadata + spec
2. **Initialize State** → Create WorkflowState from StateSchema
3. **Execute Steps** → `WorkflowExecutor::execute()` in `crates/aof-runtime/src/executor/workflow_executor.rs`:
   - Start at entrypoint step
   - Execute step (Agent node → AgentExecutor, Script node → direct tool call)
   - Collect step results in state
   - Apply StateReducer if specified (custom state update logic)
   - Evaluate NextStep conditions (conditional routing, joins, parallel branches)
   - Checkpoint state if configured
   - Continue until terminal status (Done, Error, Aborted)
4. **Error Handling** → If error, invoke error_handler step or apply RetryConfig

**AgentFlow Execution Flow:**

1. **Load AgentFlow** → Parse AgentFlow YAML with nodes + connections
2. **Build Graph** → Create DAG from connections (from → to)
3. **Execute Nodes** → `AgentFlowExecutor::execute()` in `crates/aof-runtime/src/executor/agentflow_executor.rs`:
   - Execute nodes respecting graph dependencies
   - Each node streams output as StreamEvent (TextDelta, ToolCallStart, etc.)
   - Substitute output variables (e.g., `${node-id.output}`) into next node inputs
   - Support parallel node execution where dependencies allow
4. **Streaming Output** → Send events via callback or channel for real-time visualization

**State Management:**
- Agent context: `AgentContext` holds messages, tool results, memory references
- Workflow state: `WorkflowState` holds step results, variables, status
- Persistent memory: FileBackend writes JSON snapshots for agent restarts
- Session recovery: `SessionManager` loads previous context for `--resume` or `--session <id>`

## Key Abstractions

**Model Trait:**
- Purpose: Abstract over any LLM provider (Anthropic, OpenAI, Google, etc.)
- Examples: `crates/aof-llm/src/provider/` implementations (anthropic.rs, openai.rs, google.rs)
- Pattern: Implement `generate()` and `generate_stream()` for non-streaming and streaming calls

**Tool Trait:**
- Purpose: Abstract tool operations as (input) → output
- Examples: `KubectlTool`, `GitTool`, `DockerTool`, `ShellTool`, `FileTools`, `HttpTool`
- Pattern: Implement `execute(ToolInput)` → `ToolResult`, provide ToolDefinition for schema

**ToolExecutor Trait:**
- Purpose: Execute multiple tools by name with lookup, error handling, timeouts
- Examples: `BuiltinToolExecutor` in `crates/aof-tools/src/registry.rs`
- Pattern: Registry stores Arc<dyn Tool>, execute by tool_name

**Memory Trait:**
- Purpose: Store/retrieve agent state across execution iterations
- Examples: InMemoryBackend (HashMap in Arc<DashMap>), FileBackend (JSON file)
- Pattern: `insert(key, value)`, `query(key_pattern)` with lock-free reads

**ToolExecutor Trait:**
- Purpose: Execute tools by name, managing concurrency and timeouts
- Pattern: AgentExecutor calls `tool_executor.execute(tool_name, input)` during tool_use phase

## Entry Points

**CLI Entry Point:**
- Location: `crates/aofctl/src/main.rs`
- Triggers: `Cli::parse()` → `cli.execute()` dispatches to commands
- Responsibilities: Parse CLI arguments, initialize tracing, dispatch to command handlers

**Run Agent Command:**
- Location: `crates/aofctl/src/commands/run.rs`
- Triggers: `aofctl run agent <file>` or `aofctl run agent <name>`
- Responsibilities: Load config, initialize Runtime, execute agent, format output, handle interactive mode

**Run Workflow Command:**
- Location: `crates/aofctl/src/commands/run.rs`
- Triggers: `aofctl run workflow <file>`
- Responsibilities: Load Workflow, initialize WorkflowExecutor, execute steps, manage state

**Run Flow Command:**
- Location: `crates/aofctl/src/commands/flow.rs`
- Triggers: `aofctl run flow <file>`
- Responsibilities: Load AgentFlow, build DAG, execute nodes, stream output

**Serve Trigger Server:**
- Location: `crates/aofctl/src/commands/serve.rs`
- Triggers: `aofctl serve`
- Responsibilities: Load TriggerServer config, bind to port, accept webhook requests, dispatch to agents

**Runtime Factory:**
- Location: `crates/aof-runtime/src/executor/runtime.rs`
- Triggers: Called by run/flow/workflow commands
- Responsibilities: Initialize model, tool executor, memory, MCP clients based on config

## Error Handling

**Strategy:** Typed error hierarchy with context preservation and recovery guidance

**Patterns:**
- **AofError Enum** (`crates/aof-core/src/error.rs`): Agent, Model, Tool, Memory, Mcp, Config, Validation, Workflow, Fleet, Runtime, Timeout, ResourceExhausted
- **serde_path_to_error**: Provides field path in YAML/JSON parsing errors (e.g., "Field: spec.memory\nError: invalid type")
- **ErrorKnowledgeBase** (`crates/aof-core/src/error_tracker.rs`): Tracks recurring errors, stores solutions for pattern matching
- **Recovery** in AgentExecutor: Categorize errors as Retryable (network, timeout) vs Terminal (validation, configuration), apply exponential backoff with jitter
- **Context Preservation**: Store error context (iteration count, tool name, step name) for debugging

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` with `tracing_subscriber`
- Pattern: `info!()`, `debug!()`, `warn!()`, `error!()` macros with structured fields
- Config: `RUST_LOG` env var controls level (default: "error" for clean CLI output, "debug" in development)
- Interactive mode: Custom LogWriter layer prevents tracing interference with TUI

**Validation:**
- YAML config: serde_path_to_error with precise field paths
- Output schema: JSON Schema validation with lenient/strict modes
- Agent tools: Tool schemas validated against input at execution time
- Workflow transitions: NextStep conditions evaluated before state update

**Authentication:**
- API Keys: Loaded from env vars (e.g., `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`)
- MCP Auth: mcpServerConfig specifies auth mechanism per server
- Tool Auth: Tool instances carry env-based credentials
- Context-based: `AOFCTL_CONTEXT` selects environment-specific settings (approval, rate limits, env vars)

**Concurrency:**
- Lock-free reads: DashMap for memory (concurrent agents can read simultaneously)
- Bounded parallelism: Semaphore in AgentExecutor limits concurrent tool calls
- Async I/O: tokio runtime for non-blocking I/O across all layers
- Fleet coordination: Raft consensus for multi-agent decisions (crates/aof-runtime/src/fleet/consensus.rs)

---

*Architecture analysis: 2026-02-11*
