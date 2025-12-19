# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Simplified Agent Switching** - Easy agent switching via `/agent` and `/help` commands
  - `/help` now shows agent selection buttons (tap to switch)
  - `/agent` command with inline keyboard selection
  - Built-in agents: Kubernetes, AWS, Docker, DevOps
  - Greeting message shows current agent and usage info
  - New quickstart guide: `docs/guides/quickstart-telegram.md`
- **Platform-Based Safety Layer** - Simple read-only mode for mobile platforms
  - Telegram/WhatsApp: Read-only (all writes blocked automatically)
  - Slack: Full access with existing approval workflow
  - CLI: Full access (no restrictions)
  - Pattern-based write detection for kubectl, docker, helm, terraform, aws, git
  - Plain text error messages (no markdown for better mobile display)

### Changed
- **Simplified Output** - Text-only responses for better Telegram display
  - Removed markdown formatting from agent responses
  - Cleaner, simpler messages without asterisks or backticks
  - Agent info shows only relevant details (tools, description)
- **Simplified Documentation** - MVP-focused docs
  - Removed complex context/policy YAML examples
  - Archived enterprise-setup.md to internal/future
  - Updated DOCUMENTATION_INDEX.md with simple structure
  - Cleaned up obsolete telegram-specific examples

### Removed
- Complex context YAML files (telegram-prod.yaml, telegram-dev.yaml, telegram-personal.yaml)
- Complex telegram-k8s-flow.yaml example
- mobile-read-only agent directory (platform safety makes this unnecessary)
- Complex platform_policies configuration (now handled automatically by platform detection)

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
