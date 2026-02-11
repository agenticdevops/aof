# Codebase Structure

**Analysis Date:** 2026-02-11

## Directory Layout

```
/aof/
├── crates/                          # All library crates (workspace members)
│   ├── aof-core/                    # Core types, traits, abstractions
│   ├── aof-llm/                     # Multi-provider LLM abstraction
│   ├── aof-mcp/                     # Model Context Protocol client
│   ├── aof-memory/                  # Memory backends (in-memory, file)
│   ├── aof-runtime/                 # Agent/workflow/flow execution engines
│   ├── aof-tools/                   # Built-in tool implementations
│   ├── aof-triggers/                # Webhook-based triggering system
│   ├── aof-skills/                  # Skill loading and hot-reload
│   ├── aof-viz/                     # ASCII visualization for execution
│   ├── aofctl/                      # CLI binary (kubectl-style)
│   ├── smoke-test-mcp/              # MCP initialization tests
│   └── test-trigger-server/         # Trigger server test fixtures
│
├── library/                          # Pre-built agents/workflows
│   ├── kubernetes/                  # K8s troubleshooting agents
│   ├── observability/               # Monitoring agents
│   ├── security/                    # Security scanning agents
│   ├── incident/                    # Incident response agents
│   ├── cloud/                       # Cloud ops agents (AWS, GCP, Azure)
│   └── cicd/                        # CI/CD automation agents
│
├── examples/                         # Example configurations and tutorials
│   ├── agents/                      # Agent YAML specs
│   ├── workflows/                   # Workflow specs
│   ├── flows/                       # AgentFlow specs
│   ├── fleets/                      # Fleet coordination specs
│   ├── triggers/                    # Trigger configurations
│   ├── config/                      # Sample config files
│   ├── contexts/                    # Context definitions (env-specific)
│   └── quickstart/                  # Quick start examples
│
├── skills/                          # Workspace skills (SKILL.md)
│   ├── k8s-debug/                   # Kubernetes debugging
│   ├── argocd-sync/                 # ArgoCD synchronization
│   ├── prometheus-query/            # Prometheus querying
│   ├── loki-search/                 # Loki log searching
│   └── incident-diagnose/           # Incident diagnosis
│
├── docs/                            # Internal/user documentation
│   ├── agent-library/               # Library agent docs
│   ├── agentflow/                   # AgentFlow concepts and examples
│   ├── architecture/                # Design docs
│   ├── dev/                         # Development guides
│   ├── guides/                      # User guides
│   ├── reference/                   # API reference
│   ├── schemas/                     # Config schema documentation
│   ├── tools/                       # Tool documentation
│   ├── triggers/                    # Trigger platform docs
│   ├── skills/                      # Skills documentation
│   ├── concepts/                    # Core concepts
│   └── tutorials/                   # Step-by-step tutorials
│
├── docusaurus-site/                 # Documentation website
│   ├── docs/                        # Markdown docs (mirrored from docs/)
│   ├── src/                         # React components
│   └── sidebars.js                  # Doc navigation
│
├── scripts/                         # Development scripts
│   ├── test-pre-compile.sh          # Fast validation (5s)
│   ├── test-agent.sh                # End-to-end validation
│   └── [other build/test scripts]
│
├── tests/                           # Integration tests
├── coordination/                    # Claude Flow coordination files
├── memory/                          # Session/agent memory storage
├── .planning/codebase/              # GSD planning documents (generated)
│
├── Cargo.toml                       # Workspace manifest
├── Cargo.lock                       # Dependency lock file
├── CHANGELOG.md                     # Release history
├── CLAUDE.md                        # Project instructions (read by Claude)
├── README.md                        # Project overview
├── RELEASE_PROCESS.md               # Release guidelines
├── ROADMAP.md                       # Future plans
└── LICENSE.md                       # Apache 2.0
```

## Directory Purposes

**crates/aof-core:**
- Purpose: Foundation types and trait boundaries for extensibility
- Contains: Agent, Workflow, AgentFlow, Fleet config types; Model, Tool, ToolExecutor, Memory traits; error types
- Key files: `agent.rs`, `workflow.rs`, `agentflow.rs`, `tool.rs`, `model.rs`, `error.rs`

**crates/aof-runtime:**
- Purpose: Execution engines for agents, workflows, AgentFlows, fleets
- Contains: AgentExecutor (request-response loop), WorkflowExecutor (DAG traversal), AgentFlowExecutor (node execution), FleetCoordinator (multi-agent consensus)
- Key files: `executor/agent_executor.rs`, `executor/workflow_executor.rs`, `executor/agentflow_executor.rs`, `fleet/mod.rs`

**crates/aof-llm:**
- Purpose: Multi-provider LLM abstraction (Anthropic, OpenAI, Google, Groq, Bedrock, Azure, Ollama)
- Contains: Trait implementations for each provider, model creation factory
- Key files: `provider/` (one per provider), `stream.rs` (streaming response handling)

**crates/aof-mcp:**
- Purpose: Model Context Protocol client implementation
- Contains: McpClient with multiple transports (stdio, SSE, HTTP)
- Key files: `client/mod.rs`, `transport/` (transport implementations)

**crates/aof-memory:**
- Purpose: Persistent and ephemeral agent state storage
- Contains: InMemoryBackend (DashMap-based), FileBackend (JSON file)
- Key files: `backend/memory.rs`, `backend/file.rs`

**crates/aof-tools:**
- Purpose: Built-in tool implementations for agent actions
- Contains: Unified CLI tools (kubectl, git, docker, terraform, aws, helm), file/shell tools, cloud tools, observability tools
- Key files: `tools/cli.rs` (unified tools), `tools/` (per-tool implementations), `registry.rs` (tool lookup + execution)
- Feature flags: file, shell, kubectl, docker, git, terraform, http, observability, siem, itsm, devops, cloud

**crates/aof-triggers:**
- Purpose: Webhook-based agent invocation system
- Contains: Platform adapters (Telegram, Slack, Discord, WhatsApp), command parsing, safety policies
- Key files: `server.rs` (HTTP server), `platforms/` (per-platform adapters), `safety/` (policy enforcement)

**crates/aof-skills:**
- Purpose: Load executable capabilities from SKILL.md files
- Contains: SkillRegistry, frontmatter parsing, requirements validation, hot-reload
- Key files: `lib.rs` (loader), SKILL.md format documentation

**crates/aofctl:**
- Purpose: kubectl-style CLI for agent orchestration
- Contains: Command handlers (run, get, apply, delete, describe, flow, exec, serve, skills, tools, logs, workflow-ui)
- Key files: `main.rs` (entry), `cli.rs` (command structure), `commands/` (per-command logic), `resources.rs` (resource loading)

**library/:**
- Purpose: Pre-built, production-ready agents for DevOps/SRE
- Contains: Agent YAML specs organized by domain (kubernetes, observability, security, incident, cloud, cicd)
- Usage: Load via `aofctl run agent library://kubernetes/pod-doctor` or `aofctl get agents --library`

**examples/:**
- Purpose: Tutorial configurations and working examples
- Contains: Runnable agent/workflow/flow/fleet/trigger examples with inline documentation
- Usage: Start with `examples/quickstart/` for onboarding

**skills/:**
- Purpose: Workspace-specific skills (executable tribal knowledge)
- Contains: SKILL.md files with frontmatter + markdown content
- Format: `name: skill-name`, `description:`, `metadata: { requires: { bins, env_vars, config_paths } }`
- Usage: Loaded via SkillRegistry, injected into agent context

**docs/:**
- Purpose: User-facing and developer documentation
- Contains: Concepts, guides, API reference, examples, tutorials
- Mirrored: to docusaurus-site/ for website generation
- Sections: agent-library, agentflow, architecture, dev, guides, reference, tools, triggers, skills

## Key File Locations

**Entry Points:**
- `crates/aofctl/src/main.rs`: CLI entry point (Tokio async runtime initialization)
- `crates/aofctl/src/cli.rs`: Clap command structure (run, get, apply, delete, describe, flow, exec, serve, skills, tools, logs, workflow-ui, version)

**Core Abstractions:**
- `crates/aof-core/src/agent.rs`: Agent config types (AgentConfig, AgentContext, ToolSpec)
- `crates/aof-core/src/model.rs`: Model trait, ModelConfig, ModelProvider
- `crates/aof-core/src/tool.rs`: Tool trait, ToolDefinition, ToolInput, ToolResult
- `crates/aof-core/src/workflow.rs`: Workflow config (WorkflowSpec, StepConfig, NextStep)
- `crates/aof-core/src/agentflow.rs`: AgentFlow config (nodes, connections)
- `crates/aof-core/src/error.rs`: AofError enum (Agent, Model, Tool, Memory, etc.)

**Execution:**
- `crates/aof-runtime/src/executor/agent_executor.rs`: Core request-response loop (generate → tool_use → tool_execute → repeat)
- `crates/aof-runtime/src/executor/workflow_executor.rs`: DAG step execution with state transitions
- `crates/aof-runtime/src/executor/agentflow_executor.rs`: Node-based flow execution with variable substitution
- `crates/aof-runtime/src/executor/runtime.rs`: Runtime factory (initializes model, tools, memory)

**Command Handlers:**
- `crates/aofctl/src/commands/run.rs`: `aofctl run agent|workflow|flow` (loads config, creates Runtime, executes)
- `crates/aofctl/src/commands/get.rs`: `aofctl get agents|workflows|tools` (lists resources)
- `crates/aofctl/src/commands/apply.rs`: `aofctl apply -f config.yaml` (registers agents/workflows)
- `crates/aofctl/src/commands/serve.rs`: `aofctl serve` (starts trigger webhook server)
- `crates/aofctl/src/commands/flow.rs`: `aofctl run flow <file>` (AgentFlow execution)
- `crates/aofctl/src/commands/fleet.rs`: Fleet commands
- `crates/aofctl/src/commands/skills.rs`: `aofctl skills list` (skill discovery)

**Configuration:**
- `crates/aofctl/src/resources.rs`: ResourceType enum (Agent, Workflow, Flow, Fleet, Trigger, Tool)
- `crates/aofctl/src/session.rs`: SessionManager (load/save agent sessions for `--resume`)

## Naming Conventions

**Files:**
- `mod.rs`: Module entry point (re-exports public items)
- `lib.rs`: Crate root (public API surface)
- `main.rs`: Binary entry point (CLI)
- `.rs` files: One concept per file (agent.rs, tool.rs, workflow.rs)
- Feature-gated: `#[cfg(feature = "...")]` controls compilation

**Directories:**
- `src/`: Rust source code
- `src/commands/`: CLI command implementations (run.rs, get.rs, apply.rs, etc.)
- `src/executor/`: Execution engines (agent_executor.rs, workflow_executor.rs)
- `src/fleet/`: Fleet coordination logic
- `src/tools/`: Tool implementations by domain (kubectl.rs, docker.rs, shell.rs)
- `src/platforms/`: Trigger platform adapters (telegram.rs, slack.rs, discord.rs)

**Functions/Types:**
- `snake_case`: Function names, variable names
- `PascalCase`: Trait names, struct names, enum names
- `SCREAMING_SNAKE_CASE`: Constants (VERSION, MAX_ITERATIONS)
- Trait methods: Prefixed with verb (execute, generate, register, validate)

**Config Files:**
- `*.yaml`: Agent, Workflow, AgentFlow, Fleet, Trigger specs (Kubernetes-style)
- `*.json`: JSON Schema definitions (output schemas, state schemas)
- `SKILL.md`: Skill definition with YAML frontmatter + markdown content

## Where to Add New Code

**New Agent Tool:**
- Implementation: `crates/aof-tools/src/tools/[tool-name].rs` (struct impl Tool trait)
- Export: Add pub use in `crates/aof-tools/src/lib.rs`
- Registry: Add to `BuiltinToolExecutor::new()` in `crates/aof-tools/src/registry.rs`
- Feature: Add feature flag if optional (e.g., `[features] my_tool = []`)
- Tests: `crates/aof-tools/src/tools/[tool-name]/tests.rs`

**New CLI Command:**
- Implementation: `crates/aofctl/src/commands/[command-name].rs`
- Enum variant: Add to `Commands` enum in `crates/aofctl/src/cli.rs`
- Dispatch: Add handler in `cli.execute()` match statement
- Tests: `crates/aofctl/tests/`

**New Executor Type:**
- Implementation: `crates/aof-runtime/src/executor/[executor-name].rs`
- Export: Add pub use in `crates/aof-runtime/src/lib.rs`
- Runtime: Add factory method in `crates/aof-runtime/src/executor/runtime.rs`

**New Memory Backend:**
- Implementation: `crates/aof-memory/src/backend/[backend-name].rs` (impl MemoryBackend trait)
- Export: Add pub use in `crates/aof-memory/src/lib.rs`
- Factory: Add to `SimpleMemory::with_backend()` in `crates/aof-memory/src/backend/mod.rs`

**New Platform (Triggers):**
- Implementation: `crates/aof-triggers/src/platforms/[platform-name].rs` (impl Platform trait)
- Handler: Implement message parsing and command extraction
- Export: Add pub use in `crates/aof-triggers/src/platforms/mod.rs`
- Integration: Add to `TriggerServer::register_platform()` in `crates/aof-triggers/src/server.rs`

**Shared Utilities:**
- Location: `crates/aof-core/src/` if domain-agnostic, else in consuming crate
- Pattern: Small, focused modules (error.rs, context.rs, binding.rs, activity.rs)

## Special Directories

**coordination/:**
- Purpose: Claude Flow coordination state for multi-agent development
- Generated: Yes (created by `/gsd:orchestrate`)
- Committed: Yes (tracks swarm state)

**memory/:**
- Purpose: Persistent session and agent memory storage
- Generated: Yes (created during execution)
- Committed: No (runtime state, excluded via .gitignore)
- Usage: `memory/agents/` stores per-agent context, `memory/sessions/` stores resumed sessions

**tests/:**
- Purpose: Integration tests
- Pattern: Tests that span multiple crates (end-to-end validation)
- Organization: By concern (agent_executor_tests.rs, workflow_tests.rs)

**.planning/codebase/:**
- Purpose: GSD analysis documents (generated by `/gsd:map-codebase`)
- Generated: Yes (created by this process)
- Committed: Yes (used by `/gsd:plan-phase` and `/gsd:execute-phase`)
- Contents: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, STACK.md, INTEGRATIONS.md, CONCERNS.md

**docusaurus-site/:**
- Purpose: Static documentation website
- Build: `npm run build` generates `build/` directory
- Deploy: From `build/` to hosting (Netlify, Vercel, GitHub Pages)
- Sync: `docs/` is mirrored to `docusaurus-site/docs/` for website generation

---

*Structure analysis: 2026-02-11*
