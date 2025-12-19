# AOF Documentation Index

Documentation for the Agentic Ops Framework (AOF).

## Quick Start

- **[README.md](../README.md)** - Project overview and installation
- **[Telegram Quickstart](guides/quickstart-telegram.md)** - Set up a Telegram bot in 5 minutes

## User Guides

### Essential
- **[Agent Switching](guides/agent-switching.md)** - Switch between agents (k8s, aws, docker, devops)
- **[Safety Layer](guides/safety-layer.md)** - Platform safety (Telegram read-only, Slack full access)
- **[Telegram Mobile](guides/telegram-mobile.md)** - Mobile companion guide

### Platform Setup
- **[Slack App Setup](guides/slack-app-setup.md)** - Configure Slack bot
- **[Approval Workflow](guides/approval-workflow.md)** - Human-in-the-loop approvals for Slack
- **[Conversation Memory](guides/conversation-memory.md)** - Context persistence

## Reference

- **[aofctl CLI](reference/aofctl.md)** - Command reference
- **[Agent Spec](reference/agent-spec.md)** - Agent YAML specification
- **[Platform Policies](reference/platform-policies.md)** - Safety rules per platform

## Tutorials

1. **[Build Your First Agent](tutorials/first-agent.md)** - 15 minutes
2. **[Create a Slack Bot](tutorials/slack-bot.md)** - 20 minutes
3. **[Telegram Ops Bot](tutorials/telegram-ops-bot.md)** - Mobile DevOps

## Built-in Agents

| Agent | Tools | Use For |
|-------|-------|---------|
| k8s | kubectl, helm | Kubernetes operations |
| aws | aws cli | AWS cloud operations |
| docker | docker, shell | Container management |
| devops | kubectl, docker, helm, terraform, git | Full-stack DevOps |

## Platform Safety

| Platform | Read | Write |
|----------|------|-------|
| CLI | Yes | Yes |
| Slack | Yes | Yes (with approval) |
| Telegram | Yes | No |
| WhatsApp | Yes | No |

## Commands

```
/help                # Show help and switch agents
/agent               # List and switch agents
/agent k8s           # Switch to Kubernetes agent
/agent info          # Show current agent details
```

After selecting an agent, just type naturally:
```
list pods in production
show deployment status
what's the memory usage?
```

## Architecture (Advanced)

- **[AgentFlow Routing](guides/agentflow-routing.md)** - Message routing patterns
- **[Multi-Tenant Architecture](architecture/multi-tenant-agentflows.md)** - Enterprise patterns
- **[Multi-Model Consensus](architecture/multi-model-consensus.md)** - Multi-AI coordination

## Examples

- `examples/configs/telegram-bot.yaml` - Telegram bot config
- `examples/configs/slack-k8s-bot.yaml` - Slack bot config
- `examples/agents/` - Agent YAML files

## Support

- GitHub Issues: https://github.com/agenticdevops/aof/issues
- Documentation: https://docs.aof.sh

---

Last updated: 2025-12-19
