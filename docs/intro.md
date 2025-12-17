---
sidebar_position: 1
title: Documentation Index
---

# AOF Documentation Index

Complete documentation for the Agentic Ops Framework (AOF).

## Documentation Structure

### Main README
- **[Project README](https://github.com/agenticdevops/aof)** - Project overview, quick install, 30-second example

### Getting Started
- **[Getting Started](/docs/getting-started)** - 5-minute quickstart guide
  - Installation options (cargo, binary, source)
  - API key configuration
  - First agent creation and execution
  - Common troubleshooting

### Core Concepts
- **[Core Concepts](/docs/concepts)** - Understanding AOF fundamentals
  - Agents - Single AI assistants
  - AgentFleets - Teams of agents (planned)
  - AgentFlows - Workflow automation (planned)
  - Tools - MCP, Shell, HTTP, integrations
  - Models - Multi-provider support
  - Memory - Context persistence

### Tutorials (Step-by-Step)
1. **[Build Your First Agent](/docs/tutorials/first-agent)** (15 min)
   - Agent definition and configuration
   - Adding tools (Shell, MCP)
   - Memory management
   - Deployment and testing

2. **[Create a Slack Bot](/docs/tutorials/slack-bot)** (20 min)
   - Slack app setup
   - Event handling
   - Human-in-the-loop approvals
   - Interactive features

3. **[Incident Response Flow](/docs/tutorials/incident-response)** (30 min)
   - PagerDuty integration
   - Auto-diagnostics
   - Conditional remediation
   - Post-incident analysis

### Reference Documentation
- **[Agent YAML Spec](/docs/reference/agent-spec)** - Complete Agent specification
  - Metadata fields
  - Model configuration
  - Instructions best practices
  - All tool types (Shell, HTTP, MCP, Slack, GitHub, etc.)
  - Memory types and configuration
  - Complete examples

- **[AgentFlow YAML Spec](/docs/reference/agentflow-spec)** - Complete AgentFlow specification
  - Trigger types (Webhook, Schedule, Slack, GitHub, etc.)
  - Node types (Agent, Fleet, HTTP, Shell, Conditional, etc.)
  - Connections and conditions
  - Variable interpolation
  - Error handling

- **[aofctl CLI Reference](/docs/reference/aofctl)** - Complete CLI command reference
  - Agent commands (apply, get, run, describe, logs, etc.)
  - Examples and troubleshooting

### Examples (Copy-Paste Ready)
- **[Examples Overview](/docs/examples)** - Overview of all examples with copy-paste YAML

## Recommended Reading Path

### For First-Time Users:
1. Start with **[Project README](https://github.com/agenticdevops/aof)** - Understand what AOF is
2. Follow **[Getting Started](/docs/getting-started)** - Get up and running
3. Read **[Core Concepts](/docs/concepts)** - Understand the building blocks
4. Try **[First Agent Tutorial](/docs/tutorials/first-agent)** - Hands-on practice

### For Production Deployment:
1. Review **[Agent Spec](/docs/reference/agent-spec)** - Understand all options
2. Study **[Examples](/docs/examples)** - See production patterns
3. Check **[CLI Reference](/docs/reference/aofctl)** - Master the tools

## Documentation by Role

### DevOps Engineers
Essential reading:
- [Getting Started](/docs/getting-started)
- [Examples](/docs/examples)
- [Agent Spec](/docs/reference/agent-spec) (Tools section)

### SRE Teams
Essential reading:
- [Core Concepts](/docs/concepts)
- [Incident Response Tutorial](/docs/tutorials/incident-response)
- [Examples](/docs/examples)

### Platform Engineers
Essential reading:
- [AgentFlow Spec](/docs/reference/agentflow-spec)
- [Examples](/docs/examples)
- [CLI Reference](/docs/reference/aofctl)
- All tutorials

## Quick Reference

### Common Tasks

| Task | Documentation |
|------|---------------|
| Install AOF | [Getting Started](/docs/getting-started) |
| Create first agent | [First Agent Tutorial](/docs/tutorials/first-agent) |
| Add kubectl tools | [Agent Spec - Tools](/docs/reference/agent-spec#tool-shell) |
| Build Slack bot | [Slack Bot Tutorial](/docs/tutorials/slack-bot) |
| Setup auto-remediation | [Incident Response Tutorial](/docs/tutorials/incident-response) |
| CLI commands | [aofctl Reference](/docs/reference/aofctl) |

### Model Providers

| Provider | Format | Env Variable |
|----------|--------|--------------|
| Google | `google:gemini-2.5-flash` | `GOOGLE_API_KEY` |
| OpenAI | `openai:gpt-4o` | `OPENAI_API_KEY` |
| Anthropic | `anthropic:claude-3-5-sonnet-20241022` | `ANTHROPIC_API_KEY` |
| Ollama | `ollama:llama3` | None (local) |

### Built-in Tools

| Tool | Description |
|------|-------------|
| `shell` | Execute shell commands |
| `read_file` | Read file contents |
| `list_directory` | List directory contents |
| `git` | Execute git commands |

## Getting Help

1. **Check documentation** - Search this index
2. **Review examples** - See [Examples](/docs/examples)
3. **Troubleshooting** - Check each tutorial's troubleshooting section
4. **GitHub Issues** - [Report bugs or request features](https://github.com/agenticdevops/aof/issues)
5. **Discussions** - [Ask questions](https://github.com/agenticdevops/aof/discussions)

---

**Questions?** Start with [Getting Started](/docs/getting-started) or jump to a [Tutorial](/docs/tutorials/first-agent).

**Building something?** Check the [Examples](/docs/examples) for copy-paste templates.

**Need details?** See the [Reference Documentation](/docs/reference/agent-spec).
