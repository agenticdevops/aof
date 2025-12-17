# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- kubectl-style verb-noun CLI commands for Fleet and Flow resources
- `Flow` resource type with short name `fw` (alias for workflow execution)
- `aofctl describe fleet <name>` - Show detailed fleet configuration
- `aofctl describe flow <name>` - Show detailed flow/workflow configuration
- `aofctl describe agent <name>` - Show detailed agent configuration
- Shell completion scaffold (`aofctl completion bash/zsh/fish`)
- Fleet and Flow command modules (hidden for backwards compatibility)

### Changed
- **BREAKING**: CLI now uses kubectl-style verb-noun syntax
  - Old: `aofctl agent run config.yaml`
  - New: `aofctl run agent config.yaml`
- All documentation updated to use `aofctl <verb> <resource>` syntax
- ADR-001 marked as implemented
- Migration guide updated with implementation status

### Fixed
- Consistent command syntax across all documentation and examples
- Example YAML files now use correct `aofctl verb noun` comments

### Deprecated
- Noun-verb subcommands (`aofctl fleet get`, `aofctl flow run`) - use verb-noun instead

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
