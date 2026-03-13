# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0-alpha.5] — 2026-03-13

### Phase 17: Cost Tracking + Budgets

#### New: Token and Cost Tracking (COST-01, COST-02, COST-03, COST-07)
- Every LLM call records input tokens, output tokens, estimated cost, and actual provider cost when reported
- Per-call cost records stored in `data/cost.db` (SQLite, WAL mode)
- Costs aggregated per run and per agent with `COALESCE(actual_cost_usd, cost_usd)` — actual provider costs always take precedence over estimates
- `CostRecord`, `CostSummary`, `RunCostSummary`, `ModelPricing` types in `agentix-core::cost`
- `default_model_pricing()` covers 9 models across Anthropic, OpenAI, Google, and Groq

#### New: Budget Enforcement (COST-04, COST-05)
- `budget.daily_limit_usd` in agent YAML blocks new runs when daily USD spend is exceeded
- `budget.max_tokens_per_run` stops the ReAct loop mid-run when cumulative token usage exceeds the limit
- `BudgetStopReason` enum: `DailyLimitExceeded { limit_usd, spent_usd }` and `TokenLimitExceeded { limit, used }`
- `RunResult` extended with `total_input_tokens`, `total_output_tokens`, `stopped_reason`
- `BudgetConfig` field added to `AgentConfig`, `AgentDefinition`, and all agent loader paths

#### New: Smart Model Routing (CORE-05)
- `ModelComplexityScore::score()` scores messages 0-100 based on length, complexity keywords, and question depth
- `ModelComplexityScore::tier()` maps scores to `ModelTier::Flash` (0-30), `Standard` (31-70), or `Pro` (71-100)
- Scoring keywords: analyze, research, architecture, investigate, implement, compare, explain, design, optimize, debug, refactor

#### New: CLI — `agentix costs` (CLI-09, COST-06)
- `agentix costs` — prints table of all agents with total runs, token counts, and USD costs sorted by spend
- `agentix costs agent <name>` — prints agent summary plus per-run cost breakdown table
- `--limit N` flag to control number of recent runs shown (default: 20)
- `--output json` for machine-readable output

#### New: REST API (COST-06)
- `GET /api/v1/costs` — all agent cost summaries, sorted by total spend descending
- `GET /api/v1/costs/agents/:name` — single agent cost summary (404 if no data)
- `GET /api/v1/costs/agents/:name/runs?limit=N` — paginated per-run cost history

#### Infrastructure
- `CostStore` SQLite backend in `agentix-runtime` with 7 methods: open, insert_record, get_run_summary, get_agent_summary, list_run_summaries, list_all_summaries, get_today_spend
- `AgentManager::new_with_data_dir()` creates persistent `data/runs.db` and `data/cost.db` on startup
- `Gateway::start` now uses `new_with_data_dir(Path::new("data"))` for production persistence

#### New: Examples and Documentation
- `quickstart/agents/budget-example.yaml` — agent demonstrating budget configuration with daily limit and token cap
- `docs/guides/cost-budgets.md` — setup guide for cost tracking, budgets, and smart model routing
- `docs/reference/agent-spec-budget.md` — full budget field reference with behavior description
- `docs/reference/cli-costs.md` — CLI command reference with output examples
- `docs/concepts/cost-tracking.md` — cost data model, pricing table, and API overview

### Requirements Completed
- COST-01, COST-02, COST-03, COST-04, COST-05, COST-06, COST-07 (Cost Tracking + Budgets)
- CLI-09 (`agentix costs` command)
- CORE-05 (Smart model routing by complexity score)

---

## [2.0.0-alpha.4] - 2026-03-13

### Added

#### Agent Coordination (Phase 16 — COORD-01 through COORD-05)
- **AgentInbox**: bounded mpsc channel (capacity 256) per agent for message passing
- **Delegation API**: coordinator agents delegate tasks to specialist agents via `POST /api/v1/agents/:name/delegate`; returns structured `DelegationResult` with output, status, and delegation_id
- **Parallel fan-out**: `CoordinatorProtocol::delegate_parallel()` fans out N tasks concurrently and returns results in order
- **Audit trail for delegations**: every agent-to-agent delegation is recorded in the run audit log with `delegation_id`, `from_agent`, and child run linkage
- **CoordinatorProtocol trait**: pluggable coordinator interface in `agentix-core`
- **`DelegationMessage`** and **`DelegationResult`** types in `agentix-core::coordination`

#### Vector Memory (Phase 16 — MEM-01 through MEM-04, CORE-04)
- **Vector memory**: agents with `spec.vector_memory.enabled=true` store run outputs as embeddings and recall top_k similar past contexts before each run
- **SQLite vector backend**: `SqliteVectorBackend` stores embeddings as BLOBs in SQLite; cosine-similarity search computed in Rust
- **Per-agent isolation**: all memory queries and stores are scoped by `agent_id` — agents cannot access each other's memories
- **`VectorMemoryBackend` trait**: pluggable backend interface in `agentix-core` for custom embedding stores
- **`format_memory_context()`**: formats recalled entries as a `## Relevant context from past runs` system prompt block
- **Deterministic hash embedding**: built-in FNV-1a hash fallback generates 256-dim embeddings without an external embedding API

#### Research Phase (Phase 16 — CORE-06)
- **Research phase pre-loop**: agents with `spec.research_phase.enabled=true` fire a fact-gathering LLM call before the main ReAct loop
- Gathered facts are injected as `## Research Context` into the system prompt
- Configurable `max_iterations` (default: 3)

#### CLI Memory Commands
- **`agentix memory list <agent>`**: lists all stored vector memory entries for an agent (id, run_id, text snippet, stored_at)
- **`agentix memory clear <agent>`**: deletes all memory entries for an agent (with confirmation prompt)

#### REST Memory API
- **`GET /api/v1/agents/:name/memory`**: returns all memory entries for the agent as a JSON array
- **`DELETE /api/v1/agents/:name/memory`**: clears all memory entries; returns `{"cleared": true, "entries_deleted": N}`

#### Quickstart Examples
- `quickstart/agents/ops-coordinator.yaml` — coordinator agent with memory, delegates to dba-specialist
- `quickstart/agents/dba-specialist.yaml` — specialist agent with memory and research phase

#### New Documentation
- `docs/guides/agent-memory.md` — complete memory configuration reference
- `docs/guides/coordinator-agents.md` — coordinator pattern guide with YAML examples
- `docs/concepts/agent-coordination.md` — coordination primitives reference
- `docs/concepts/vector-memory.md` — vector memory architecture guide
- `docs/guides/quickstart-coordination.md` — step-by-step coordinator quickstart

### Requirements Completed
- COORD-01, COORD-02, COORD-03, COORD-04, COORD-05 (Agent Coordination)
- MEM-01, MEM-02, MEM-03, MEM-04 (Memory & Knowledge)
- CORE-04 (Agent memory recall and storage)
- CORE-06 (Agent research phase pre-loop)

---

## [2.0.0-alpha.3] - 2026-03-13

### Added

#### Triggers + Scheduling (Phase 15 — TRIG-01 through TRIG-11)
- **Cron trigger**: agents fire automatically on cron schedules (`triggers: [{type: cron, expression: "0 9 * * 1"}]`)
- **Webhook trigger**: generic HTTP POST at `/webhooks/:trigger_id` fires an agent; optional HMAC-SHA256 signature verification
- **GitHub trigger**: HMAC-verified GitHub webhook fires agents on `pull_request`, `push`, `release`, and other events
- **Jira trigger**: Jira webhook fires agents on `issue_updated`, `comment_created`, and other events (event from `webhookEvent` body field)
- **Channel mention trigger**: @mentioning the agent in Slack (`app_mention`), Discord (message with `<@`), or Telegram fires the agent
- **Agent-to-agent trigger**: one agent fires another via `POST /api/v1/agents/:name/trigger`
- **CLI trigger**: `agentix run <agent> --input "..."` for one-shot manual invocation
- `TriggerTrait` pluggable interface in `agentix-core` — add new trigger types without modifying core
- `TriggerEvent` envelope type carrying source, payload, context, fired_at, and trigger_id through the entire pipeline
- `TriggerSource` enum: `Cron`, `Webhook`, `GitHub`, `Jira`, `Slack`, `Discord`, `Telegram`, `Agent`, `Cli`
- `POST /api/v1/agents/:name/trigger` — REST endpoint for agent and CLI triggers
- `POST /webhooks/:trigger_id` — REST endpoint for webhook, GitHub, and Jira triggers

#### Run Persistence (TRIG-10)
- Every agent run is stored in SQLite (`./agentix-runs.db`) with trigger_source, timestamps, duration, and output summaries
- `GET /api/v1/runs` — global run history endpoint, filterable by `?agent=name` and `?limit=N`
- `agentix runs` now shows a **TRIGGER** column with the trigger source for each run

#### Notification Routing (TRIG-11)
- `webhook` notification type: POST run summary to a configured URL after every triggered run
- `log` notification type: emit structured log line after every triggered run
- Configured via `notifications:` field in agent YAML spec

#### Quickstart Examples
- `quickstart/agents/scheduled-reporter.yaml` — weekly cron report with Slack webhook notification
- `quickstart/agents/github-pr-reviewer.yaml` — automatic PR review on GitHub `pull_request` events

#### New Documentation
- `docs/concepts/triggers.md` — trigger system overview and architecture
- `docs/concepts/cron-triggers.md` — cron scheduling reference
- `docs/concepts/webhook-triggers.md` — HTTP webhooks, GitHub, and Jira trigger reference
- `docs/concepts/run-persistence.md` — run history, SQLite storage, and notification routing
- `docs/guides/github-integration.md` — end-to-end GitHub webhook setup
- `docs/guides/jira-integration.md` — Jira webhook setup
- `docs/guides/slack-mention-trigger.md` — Slack app_mention trigger setup
- `docs/guides/discord-mention-trigger.md` — Discord mention trigger setup
- `docs/guides/telegram-mention-trigger.md` — Telegram message trigger setup
- `docs/guides/agent-to-agent-triggers.md` — agent orchestration via triggers
- `docs/guides/cli-triggers.md` — manual CLI invocation reference
- `docs/guides/triggers-quickstart.md` — 5-minute quickstart guide for all trigger types

---

## [2.0.0-alpha.2] - 2026-03-13

### Added

#### Skills Composition (SKILL-01 to SKILL-05, CLI-08)
- **8 built-in skill packs** embedded in the binary: `aws`, `kubernetes`, `terraform`, `docker`, `git`, `database`, `security`, `observability`
- `agentix skills list` — list all available skill packs with descriptions
- `agentix skills show <name>` — print a skill pack's full instruction content
- Custom skills: place `SKILL.md` in `agents/<name>/skills/<skill-name>/SKILL.md`
- Skill injection is deterministic (directory matching) — no LLM routing call required
- Skills are injected into the agent's system prompt under `## Skill: <name>` headings
- JSON output support: `agentix skills list --output json`

#### CLI and Shell Tools (TOOL-01, TOOL-03)
- `CliToolExecutor` — async CLI and shell tool execution with timeout enforcement
- Replaces blocking `ShellToolExecutor` stub from Phase 13 with proper async implementation
- `{{variable}}` template substitution in tool commands and args
- Default 30-second timeout; configurable per tool via `timeout_secs` in tool YAML
- Tool failures return descriptive error observations — agent decides how to proceed

#### MCP Server Integration (TOOL-02)
- `McpToolExecutor` — connects agents to MCP servers for tool calls
- `CompositeToolExecutor` — routes to CLI, MCP, or WASM executors based on tool type
- stdio, SSE, and HTTP transport support via `agentix-mcp` crate
- Per-run connection lifecycle — MCP server subprocess started/stopped with each run
- Error observations on connection failure (no run abort)

#### WASM Sandbox (TOOL-04, TOOL-05)
- `WasmSandbox` — executes WASM modules via `wasmtime` (feature-gated: `agentix-runtime/wasm`)
- `CapabilityManifest` — declared permissions in `tools/<name>.yaml` for `type: wasm`
- `WasmCapability` types: `network`, `filesystem`, `secrets`, `http_endpoints`, `resource_limits`
- Undeclared capability access returns `WasmViolation` as tool observation
- Default resource limits: 64 MB memory, 30-second timeout

### Documentation
- `docs/guides/skills.md` — skills composition guide with examples
- `docs/guides/tools.md` — CLI, shell, and MCP tool configuration reference
- `docs/guides/mcp.md` — MCP server setup, transports, and popular servers
- `docs/guides/wasm-tools.md` — WASM tools, capabilities, and module interface

## [2.0.0-alpha.1] - 2026-03-13

### BREAKING CHANGES
- All crates renamed from `aof-*` to `agentix-*`
- CLI binary renamed from `aofctl` to `agentix`
- Agent format changed to GitAgent-compatible directories (agent.yaml + SOUL.md)
- Workspace config changed to `agentix.yaml` with `kind: Workspace`

### Added
- GitAgent-compatible agent directory format (agent.yaml + SOUL.md + RULES.md + skills/)
- ReAct (Reason + Act) loop engine: plan -> act -> observe -> reflect
- Streaming output: SSE for HTTP clients, text with [Plan]/[Act]/[Observe]/[Reflect] labels, NDJSON
- Gateway service with REST API for agent lifecycle management
- `agentix onboard` — interactive workspace setup
- `agentix init` — scaffold new agent directories
- `agentix gateway start/status` — gateway lifecycle
- `agentix agents` — list agents
- `agentix runs` — run history
- `agentix logs` — execution output
- `agentix stop` — cancel running agents
- `agentix apply -f` — register agents via gateway API
- `agentix validate` — validates both directories and flat YAML files
- Workspace defaults (max_iterations, timeout, mode) applied to all agents
- DirectoryLoader: assembles system prompt from SOUL.md + RULES.md + skills/
- Multi-agent via agents/ subdirectory (hierarchical)
- LLM provider factory: resolves "provider/model" strings to concrete provider instances
- AgentManager: DashMap-based concurrent agent state management
- ShellToolExecutor: executes shell/cli tools with template variable substitution

### Removed
- All v1.0 code: React web-app, agent personas, fleet coordination, workflows, agentflows
- 12 v1.0 crates deleted
- Monolithic agent YAML as primary format (now backward-compat only)

---

## [Unreleased]

### Enhanced
- **TUI Professional Layout** - Complete redesign of interactive mode
  - Header bar with agent status, model, tool count, LLM calls, session timer
  - Current tool indicator shows which tool is being executed
  - Activity panel shows detailed tool information (name, arguments, duration)
  - Token counts displayed in activity log for LLM responses
  - Timestamped chat messages with role indicators (YOU/AI/SYS/ERR)
  - Color-coded token gauge (green/yellow/red based on usage)
  - Professional footer with context-aware keyboard shortcuts

- **Full Input Editing** - Claude Code-like input experience
  - Cursor movement with ←/→ arrow keys
  - Word-by-word navigation with Ctrl+←/→
  - Home/End keys to jump to start/end of input
  - Ctrl+A/E for bash-style start/end navigation
  - Backspace/Delete work at cursor position
  - Ctrl+W to delete word before cursor
  - Ctrl+U to clear entire input
  - Multi-line input with Alt+Enter, Ctrl+J (cross-terminal compatible)
  - Animated cursor shows position in text

- **Double-ESC to Exit** - Vim-style exit
  - Press ESC twice within 500ms to quit (when not busy)
  - Single ESC still cancels running agent

- **Header Tool Count Fix**
  - Now shows "Tools: X (Y used)" where X = available, Y = executed

- **Real-time Tool Activity Events**
  - Activity panel now shows tool executions in real-time
  - Tool name, arguments (truncated), and execution duration displayed
  - Streaming events from runtime for accurate tool tracking
  - Current tool indicator in header during execution

## [0.4.0-beta] - 2026-01-23

### Added
- **Interactive TUI Mode** - Full-featured terminal user interface for agent conversations
  - Launch with `aofctl run agent <config.yaml>` (no `--input` flag)
  - Chat panel with syntax-highlighted conversation history
  - Activity log showing real-time agent events (thinking, analyzing, tool use, LLM calls)
  - Context gauge displaying token usage and execution time
  - Help overlay with keyboard shortcuts (press `?`)
  - LazyGit-inspired styling with clear visual hierarchy

- **Agent Cancellation** - Stop running agents with ESC key
  - Graceful cancellation using tokio CancellationToken
  - Clean abort of LLM calls and tool executions
  - Status indicator shows "Cancelling..." during abort

- **Session Persistence** - Conversation history saved automatically
  - Sessions stored in `~/.aof/sessions/<agent-name>/`
  - Includes complete message history, token usage, activity logs
  - JSON format for easy inspection and backup

- **Session Resume** - Continue previous conversations
  - `--resume` flag to continue latest session: `aofctl run agent config.yaml --resume`
  - `--session <id>` flag to resume specific session
  - Restored sessions show previous context to the agent

- **Session Management Commands**
  - `aofctl get sessions` - List all saved sessions across agents
  - `aofctl get sessions <agent>` - List sessions for specific agent
  - Output shows session ID, agent, model, message count, tokens, age
  - Supports `-o json` and `-o yaml` output formats

- **Activity Event System** - Real-time agent activity tracking
  - New `ActivityEvent` enum in aof-core with event types:
    - Thinking, Analyzing, LlmCall, ToolUse, ToolComplete, Warning, Error
  - `ActivitySender` for emitting events from runtime
  - `ActivityReceiver` for consuming events in TUI

### Changed
- TUI keyboard shortcuts updated:
  - `ESC` now cancels running agent (was: do nothing)
  - `Ctrl+S` saves session manually
  - `Ctrl+L` clears chat and starts new session
  - `Shift+↑/↓` scrolls chat history
  - `PageUp/Down` scrolls 5 lines

### Documentation
- Updated getting-started guide with interactive mode examples
- Added TUI keyboard shortcuts to CLI reference
- Added session management documentation
- Updated aofctl reference with --resume and --session flags

## [0.3.2-beta] - 2026-01-02

### Added
- Built-in command handler support via `agent: builtin` in trigger command bindings
  - Use `agent: builtin` for `/help`, `/agent`, `/fleet` to get interactive menus
  - Interactive menus include fleet/agent selection buttons (Telegram/Slack)
  - Keeps built-in UI handlers separate from LLM-routed commands
- Stale message filtering for webhook handlers
  - Messages older than 60 seconds are silently dropped
  - Prevents processing of queued messages when daemon restarts
  - Configurable via `max_message_age_secs` in handler config
- `cargo install aofctl` support via crates.io publishing
  - All AOF crates now published to crates.io
  - Automated publishing on tagged releases
- New documentation: Built-in Commands Guide (`docs/guides/builtin-commands.md`)

### Fixed
- `aofctl serve` now produces visible startup output
  - Changed from tracing (default level: error) to println for critical startup messages
  - Users can now see server bind address, registered platforms, loaded agents/flows
  - Error messages use stderr for proper output separation
- GitHub/GitLab/Bitbucket PR reviews now post a single response instead of multiple comments
  - Intermediate acknowledgment messages ("Thinking...", "Processing...") are skipped for Git platforms
  - Only the final response is posted, keeping PR threads clean
  - Slack/Telegram/Discord still show real-time progress indicators
- Improved `library://` URI path resolution for agent library

## [0.3.1-beta] - 2025-12-26

### Added
- Token usage tracking for AgentFlow execution
  - Agent nodes now report input/output tokens
  - Flow completion summary shows total token usage
  - Script nodes correctly show 0 tokens (no LLM usage)
- `--library` flag for `aofctl get agents` to list built-in agents
  - Shows all 30 production-ready agents from the library
  - Displays domain (category), agent name, status, and model
  - Supports filtering by agent name: `aofctl get agents pod-doctor --library`
  - Supports JSON/YAML output formats
- `library://` URI syntax for running agents from the built-in library
  - Format: `library://domain/agent-name`
  - Example: `aofctl run agent library://kubernetes/pod-doctor --prompt "debug CrashLoopBackOff"`
  - Helpful error messages showing available agents when agent not found
- `--prompt` as an alias for `--input` in the run command
  - More intuitive for LLM-style interactions

### Fixed
- Script node YAML field naming (`scriptConfig` camelCase)
- Flow completion display formatting for token line

## [0.3.0-beta] - 2025-12-24

### Added

#### Agent Library (30 Production-Ready Agents)
- **Kubernetes Domain** (5 agents)
  - deploy-guardian: Validates deployments before production rollout
  - node-doctor: Diagnoses and auto-heals node problems
  - resource-optimizer: Right-sizes containers based on actual usage
  - pod-debugger: Troubleshoots pod failures with context
  - rollout-manager: Manages progressive deployments
- **Observability Domain** (5 agents)
  - alert-manager: Manages Prometheus alerts with runbook automation
  - slo-guardian: Monitors SLI/SLO compliance and error budgets
  - log-analyzer: Analyzes logs for patterns and anomalies
  - metrics-explorer: Queries and visualizes Prometheus metrics
  - trace-investigator: Analyzes distributed traces
- **Incident Domain** (5 agents)
  - rca-agent: Performs automated root cause analysis
  - incident-commander: Orchestrates incident response
  - escalation-manager: Routes incidents to appropriate teams
  - postmortem-writer: Generates blameless postmortems
  - runbook-executor: Executes runbooks with safety checks
- **CI/CD Domain** (5 agents)
  - pipeline-fixer: Diagnoses and fixes failing pipelines
  - build-optimizer: Optimizes build performance
  - release-manager: Coordinates release workflows
  - test-analyzer: Analyzes test failures and flakiness
  - artifact-manager: Manages build artifacts and images
- **Security Domain** (5 agents)
  - vuln-scanner: Scans for vulnerabilities with Trivy
  - secret-auditor: Audits secrets management with Vault
  - compliance-checker: Validates compliance with OPA policies
  - access-reviewer: Reviews RBAC permissions
  - security-responder: Responds to security incidents
- **Cloud Domain** (5 agents)
  - cost-optimizer: Analyzes cloud costs and recommends savings
  - drift-detector: Detects infrastructure drift with Terraform
  - capacity-planner: Forecasts capacity needs
  - backup-validator: Validates backup integrity
  - multi-cloud-coordinator: Coordinates across AWS/Azure/GCP

#### MCP Server Catalog (10 Documented Servers)
- **Core**: filesystem, fetch, puppeteer
- **Development**: github, gitlab
- **Databases**: postgres (read-only), sqlite (read/write)
- **Communication**: slack
- **Search**: brave-search

Each catalog entry includes:
- Configuration examples for AOF agents
- Full tool reference with parameters
- Use case examples with agent specs
- Troubleshooting guides

#### Comprehensive Documentation
- Agent Library user guide with domain overviews
- MCP integration guide with tested servers
- Wired both sections into Docusaurus sidebar

### Changed
- Improved getting-started with zero-setup examples
- Cleaned up architecture documentation
- Updated all agent specs to use correct tool naming (underscore-separated)

## [0.2.0-beta] - 2025-12-20

### Added

#### Composable Architecture (Major Refactor)
- **Simplified to 4 Core Concepts**: Agent, Fleet, Flow, Trigger
- **Composable Design**: Mix and match agents, tools, and triggers
- Removed complex FlowBinding in favor of direct trigger→agent mapping

#### New Trigger Platforms (5 New)
- **Microsoft Teams** - Bot Framework integration with Adaptive Cards
  - JWT Bearer token authentication
  - Tenant and channel restrictions
  - Action.Submit handling for button clicks
- **WhatsApp Business** - Cloud API integration
  - HMAC-SHA256 signature verification
  - Interactive buttons and lists
  - Template message support
- **GitHub** - Webhook integration for PR/Issue automation
  - PR opened/updated/merged events
  - Issue created/updated events
  - Comment triggers with @mention detection
- **GitLab** - Webhook integration for MR automation
  - Merge request events
  - Pipeline status triggers
  - Note (comment) events
- **Bitbucket** - Webhook integration for PR automation
  - Pull request events
  - Repository push events
- **Jira** - Issue tracking platform abstraction
  - Issue created/updated/transitioned events
  - JQL query support
  - Comment and attachment handling

#### Comprehensive Documentation
- **Concepts**: Teams, Discord, WhatsApp, Jira integration overviews
- **Reference**: Full API reference for each platform
- **Tutorials**: Step-by-step ops bot tutorials
- **Quickstart Guides**: 10-15 minute setup guides

#### Platform Capabilities System
- Thread support detection
- Interactive element support
- File attachment support
- Reaction support
- Rich text support
- Approval workflow support

### Changed
- **Simplified Agent Switching** - Easy agent switching via `/agent` and `/help` commands
- **Platform-Based Safety Layer** - Read-only mode for mobile platforms
- **Simplified Output** - Text-only responses for better mobile display
- Daemon configuration simplified with direct platform webhook paths

### Fixed
- Telegram inline keyboard callback handling
- Agent loading when flows directory doesn't exist
- System prompts correctly loaded from agent YAML
- Model configuration from agent YAML (was hardcoded)

### Technical Details
- 9 trigger platforms: Slack, Discord, Telegram, WhatsApp, Teams, GitHub, GitLab, Bitbucket, Jira
- 7 built-in tools: kubectl, docker, aws, terraform, git, shell, http
- Platform registry with factory pattern for extensibility
- Ed25519 (Discord) and HMAC-SHA256 (others) signature verification
- ~60,000 lines of Rust code

## [0.1.15] - 2025-12-18

### Added
- **Human-in-the-Loop Approval Workflow** - Reaction-based command approval for Slack
  - ✅/❌ reactions for approve/deny destructive operations
  - Automatic ✅ ❌ reaction buttons added to approval messages
  - Support for multiple approval reactions (thumbs up, checkmark variants)
  - Configurable `approval_allowed_users` for role-based approval
- **Conversation Memory System** - Context persistence across Slack messages
  - Per-channel and per-thread memory isolation
  - Automatic context injection for follow-up messages
  - Supports "delete it", "scale that down" style contextual commands
  - Configurable memory limits (20 messages default, 10 in context)
- **AgentFlow Multi-Tenant Routing** - Route messages to different agents based on patterns
  - Pattern-based agent selection from incoming messages
  - Support for multiple agents in a single flow
- **Bot Self-Approval Prevention** - Auto-detects `bot_user_id` at startup
  - Uses Slack's `auth.test` API to get bot's own user ID
  - Filters out bot's own reactions to prevent self-approval
- New documentation guides:
  - `docs/guides/approval-workflow.md` - Complete approval workflow guide
  - `docs/guides/conversation-memory.md` - Conversation memory guide

### Changed
- kubectl-style verb-noun CLI commands for Fleet and Flow resources
- `Flow` resource type with short name `fw` (alias for workflow execution)
- All documentation updated to use `aofctl <verb> <resource>` syntax
- Updated slack-k8s-bot agent with explicit kubectl syntax guidance

### Fixed
- **Telegram inline keyboard callback handling** - Fixed "Invalid selection" error when tapping agent/flow buttons
  - Improved callback format parsing to handle both `callback:agent:name` and `agent:name` formats
  - Added better debug logging for callback troubleshooting
  - More descriptive error messages when callback parsing fails
- System prompts from agent YAML files now correctly loaded and used
- Agent execution error handling with proper response to platforms
- Bracket structure in `handle_natural_language` function
- **Agent loading flag logic** - Fixed issue where agents weren't loaded when flows directory didn't exist
  - Introduced proper `agents_loaded` boolean tracking
  - Agents now correctly pre-load regardless of flows configuration
- **`/run agent` using wrong model** - Fixed hardcoded Anthropic model to use pre-loaded agent's configuration
  - Now uses `google:gemini-2.5-flash` from agent YAML
  - Tools and system prompts correctly applied

### Notes
- Config changes to `approval_allowed_users` require server restart (hot-reload coming in [Issue #22](https://github.com/agenticdevops/aof/issues/22))

## [0.1.14] - 2025-12-17

### Added
- Initial release with core aofctl functionality
- Agent execution with multiple LLM providers (OpenAI, Anthropic, Google, Ollama, Groq)
- MCP (Model Context Protocol) server support
- Memory backends (InMemory, File, SQLite)
- Built-in tools (Shell, HTTP, FileSystem)
- AgentFleet multi-agent coordination
- AgentFlow workflow orchestration
- Trigger system (Webhook, Schedule, FileWatch, Slack, GitHub, PagerDuty)

### Commands
- `aofctl run agent <file>` - Execute an agent
- `aofctl run workflow <file>` - Execute a workflow
- `aofctl run fleet <file>` - Execute a fleet
- `aofctl get <resource>` - List resources
- `aofctl describe <resource> <name>` - Show resource details
- `aofctl apply -f <file>` - Apply configuration
- `aofctl delete <resource> <name>` - Delete resources
- `aofctl logs <resource> <name>` - View logs
- `aofctl api-resources` - List available resource types
- `aofctl version` - Show version information

### Resource Types
| Resource | Short Name | Description |
|----------|------------|-------------|
| agents | ag | AI agents |
| workflows | wf | Multi-step workflows |
| fleets | fl | Multi-agent coordination |
| flows | fw | Workflow aliases |
| tools | tl | MCP tools |

---

## Release Notes Format

### Version Number Meaning
- **MAJOR**: Breaking changes to CLI or API
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, documentation updates

### Categories
- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features to be removed in future
- **Removed**: Features removed in this release
- **Fixed**: Bug fixes
- **Security**: Security-related changes
