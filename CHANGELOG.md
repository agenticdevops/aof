# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
