# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Approval User Whitelist** - Configure which users can approve destructive commands
  - New `approval_allowed_users` field in platform config (Slack implemented)
  - Platform-agnostic design: supports `U12345678`, `slack:U12345678`, `email:user@company.com` formats
  - Currently implemented: Raw Slack user IDs (e.g., `U015VBH1GTZ`)
  - Planned: Global `spec.approval.allowed_users` for multi-platform deployments
  - If not configured, anyone can approve (backward compatible)
  - Documentation and example config updated

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
