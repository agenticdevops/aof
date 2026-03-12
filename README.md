# OpenAgentiX

**Enterprise Agent Automation Platform**

Define AI agents as directories. Run them on demand or on schedule.
Stream real-time ReAct loop output. Track everything.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://rustup.rs)
[![GitHub](https://img.shields.io/badge/github-agenticdevops%2Faof-black?logo=github)](https://github.com/agenticdevops/aof)

## Quick Start

```bash
# Install
cargo install agentix

# Set up a workspace
agentix onboard

# Start the gateway
agentix gateway start

# List agents
agentix agents

# Create a new agent
agentix init --name my-agent
```

## Agent Directories

Agents are directories, not single files:

```
agents/my-agent/
├── agent.yaml    # Metadata + model
└── SOUL.md       # Identity + system prompt
```

That's all you need. Add tools, skills, and sub-agents as your needs grow.

## Features

- **GitAgent-compatible** — Agent directories with SOUL.md, RULES.md, skills/, tools/
- **ReAct loop** — Plan -> Act -> Observe -> Reflect execution model
- **Streaming output** — SSE for HTTP, text/JSON for CLI
- **Multi-provider** — Anthropic, OpenAI, Google, Ollama
- **Gateway architecture** — REST API, run management, agent lifecycle
- **Skills composition** — Reusable capability modules (Phase 14)
- **Cost tracking** — Per-run and per-agent cost monitoring (Phase 17)
- **Approval workflows** — Human-in-the-loop gate (Phase 20)

## Example: Run an Agent

```bash
# Start the gateway
agentix gateway start

# Run via API (streaming)
curl -X POST http://localhost:7777/api/v1/agents/my-agent/run \
  -H 'Content-Type: application/json' \
  -d '{"input": "What is the status of the cluster?"}' \
  --no-buffer
```

## Documentation

- [Quickstart Guide](docs/guides/quickstart.md)
- [Agent Directory Format](docs/spec/agent-directory-structure.md)
- [Agent Manifest (agent.yaml)](docs/spec/agent-yaml-v1.md)
- [Workspace Configuration](docs/spec/workspace-config.md)

## Architecture

```
agentix CLI
    |
    v
Gateway (HTTP :7777)
    |
    +--> AgentManager (loads agent directories)
    |        |
    |        v
    |    AgentDefinition (SOUL.md + RULES.md + skills/ + tools/)
    |
    +--> ReActEngine (Plan -> Act -> Observe -> Reflect)
             |
             +--> LLM Provider (anthropic/claude-sonnet-4-6)
             |
             +--> ToolExecutor (shell, cli, mcp)
```

## Crates

| Crate | Description |
|-------|-------------|
| `agentix-core` | Core traits, types, agent definitions |
| `agentix-llm` | LLM provider abstraction (Anthropic, OpenAI, Google, Ollama) |
| `agentix-runtime` | ReAct loop engine, gateway, agent manager |
| `agentix-memory` | Conversation and working memory |
| `agentix-triggers` | Webhook handlers (Slack, Telegram, GitHub, etc.) |
| `agentix-mcp` | Model Context Protocol client |
| `agentix` | CLI binary |

## License

Apache 2.0 — See [LICENSE](LICENSE)
