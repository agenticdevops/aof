# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Context Switching (`/context`)** - Unified project/cluster switching command
  - `/context` command with inline keyboard selection for context switching
  - `/context <name>` to switch directly (e.g., `/context cluster-a`)
  - `/context info` for detailed current context information
  - **Context = Agent + Connection Parameters** - Bundles agent, kubeconfig, AWS profile, etc.
  - Replaces both `/env` and `/agents` commands with unified `/context`
  - Per-user context session tracking
  - Example context files: k8s-cluster-a.yaml, aws-dev.yaml, database.yaml, prometheus.yaml
  - New documentation guide: `docs/guides/context-switching.md`
  - Framework-first design: Context is platform-agnostic, works across Telegram, Slack, CLI
- **Comprehensive DevOps Read-Only Agent** - Single agent with all DevOps tools
  - New `devops-readonly` agent combining kubectl, docker, helm, git, terraform, aws, prometheus
  - Designed as default mobile agent (no constant switching needed)
  - Complete tool restrictions for read-only access across all tools
  - Located at `examples/agents/mobile-read-only/devops-readonly.yaml`
- **Environment Context Files** - Pre-built environment configurations
  - `examples/contexts/env-prod.yaml` - Production (read-only from mobile)
  - `examples/contexts/env-staging.yaml` - Staging (write with approval)
  - `examples/contexts/env-dev.yaml` - Development (full access)
  - Each includes kubeconfig, namespace, AWS settings, and platform policies
- **Sample Approval Flow for kubectl** - Example write operation workflow
  - `examples/flows/kubectl-apply-approval.yaml` - Complete approval workflow
  - Environment-aware policies (prod stricter, dev auto-approve)
  - Dry-run validation before approval request
  - Timeout handling with auto-deny
  - Comprehensive variable interpolation
- **Telegram Mobile Companion** - Safe mobile interface for AOF agents
  - `/agents` command with inline keyboard selection for switching agents
  - `/flows` command with inline keyboard selection for triggering flows
  - Session-based agent tracking per user
  - Callback handling for Telegram inline buttons
- **MVP Safety Layer** - Platform-based read-only mode for mobile
  - Telegram/WhatsApp: Read-only (all writes blocked)
  - Slack: Full access with existing approval workflow
  - CLI: Full access (no restrictions)
  - Pattern-based write detection for kubectl, docker, helm, terraform, aws, git
  - Natural language intent detection (create, deploy, delete, scale, etc.)
  - Clear error messages explaining why operation was blocked
- **ASCII Visualization Crate** - `aof-viz` for mobile-friendly output
  - Status rendering with emoji and ASCII styles
  - Flow visualization (linear, inline, branching)
  - Tool call result formatting with tables
  - Safety decision rendering
  - Progress bars and spinners
  - Platform-specific render configs (Telegram, Slack, terminal)
- **Read-Only Mobile Agents** - Pre-built agents for Telegram
  - k8s-status, docker-status, git-status, prometheus-query, helm-status
  - All labeled with `mobile-safe: "true"`
  - Only read operations exposed
- **Agent Tool Execution System** - Full tool execution support for agents
  - Built-in tools: `kubectl`, `docker`, `helm`, `terraform`, `aws`, `git`, `shell`
  - File tools: `read_file`, `write_file`, `list_directory`, `search_files`
  - Observability tools: `prometheus_query`, `loki_query`
  - Tools registered and passed to LLM with definitions
- **Kubernetes-Style Agent Discovery** - Load agents by `kind: Agent` and `metadata.name`
  - Agents indexed by `metadata.name`, not filename
  - Support for both K8s-style and flat YAML formats via `AgentConfigInput` enum
  - `--agents-dir` flag for pre-loading agents on server startup
- **Approval User Whitelist** - Configure which users can approve destructive commands
  - New `approval_allowed_users` field in platform config (Slack implemented)
  - Platform-agnostic design: supports `U12345678`, `slack:U12345678`, `email:user@company.com` formats
  - Currently implemented: Raw Slack user IDs (e.g., `U015VBH1GTZ`)
  - Planned: Global `spec.approval.allowed_users` for multi-platform deployments
  - If not configured, anyone can approve (backward compatible)
  - Documentation and example config updated
- **Multi-Tenant AgentFlow Architecture** - Enterprise-ready multi-tenant deployments
  - Route different channels/users/patterns to different agents
  - Support for multiple organizations in single daemon
  - Environment isolation per AgentFlow (kubeconfig, namespace, env vars)
  - Comprehensive architecture documentation at `docs/architecture/multi-tenant-agentflows.md`
- **Telegram AgentFlow Example** - Complete Telegram bot setup following Slack pattern
  - New `examples/configs/telegram-k8s-flow.yaml` example
  - Same architecture as Slack bot with channel/user/pattern routing
- **AgentFlow Routing Documentation** - Complete guide on how routing works
  - Flow scoring algorithm explained
  - Common routing patterns (pattern-based, channel-based, user-based)
  - Debugging tips and best practices
  - New guide at `docs/guides/agentflow-routing.md`

### Changed
- All example agents updated to use `google:gemini-2.5-flash` as the primary model
- Improved debug logging for agent parsing and tool registration
- Documentation updated with correct model names for all providers
- **`/run agent` command now routes through AgentFlow** - Uses pre-loaded agents with correct model and tools
  - No longer creates hardcoded Anthropic fallback
  - Follows same code path as natural language messages
  - Ensures consistent behavior across commands and chat
- **`/help` command updated** - Now includes `/agents` and `/flows` in quick start section
  - Shows chat mode usage (select agent then type naturally)
  - Updated examples with interactive workflow
  - Fixed support URL to point to correct GitHub repository

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
- Agent instructions now correctly guide LLM to use `kubectl create deployment` instead of `kubectl run` for deployments with replicas
- Consistent command syntax across all documentation and examples
- Example YAML files now use correct `aofctl verb noun` comments

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
