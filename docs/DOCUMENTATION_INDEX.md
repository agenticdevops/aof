# AOF Documentation Index

Documentation for the Agentic Ops Framework (AOF).

## Quick Start

- **[README.md](../README.md)** - Project overview and installation
- **[Core Concepts](introduction/concepts.md)** - Agent → Fleet → Flow architecture
- **[Telegram Quickstart](guides/quickstart-telegram.md)** - Set up a Telegram bot in 5 minutes

## Core Concepts

AOF uses a simple, composable model:

| Concept | What It Is | Example |
|---------|------------|---------|
| **Agent** | Single-purpose specialist | `k8s-agent`, `prometheus-agent` |
| **Fleet** | Team of agents for a purpose | `devops-fleet`, `rca-fleet` |
| **Flow** | Multi-step workflow with nodes | `deploy-flow`, `incident-flow` |
| **Trigger** | Platform + command routing | `slack-prod`, `telegram-oncall` |

**One way to do it**: Build focused agents → Compose into fleets → Define workflows as flows → Connect to chat via triggers.

## User Guides

### Essential
- **[Core Concepts](introduction/concepts.md)** - Agent, Fleet, Flow explained
- **[Fleet Switching](guides/agent-switching.md)** - Switch between fleets in chat
- **[Safety Layer](guides/safety-layer.md)** - Platform safety (Telegram read-only, Slack full access)
- **[Telegram Mobile](guides/telegram-mobile.md)** - Mobile companion guide

### Platform Setup
- **[Slack Bot Tutorial](tutorials/slack-bot.md)** - Complete Slack bot setup with approval workflow
- **[Approval Workflow](guides/approval-workflow.md)** - Human-in-the-loop approvals for Slack
- **[Conversation Memory](guides/conversation-memory.md)** - Context persistence

## Reference

### Core Specifications
- **[aofctl CLI](reference/aofctl.md)** - Command reference
- **[Agent Spec](reference/agent-spec.md)** - Agent YAML specification
- **[Fleet Spec](reference/fleet-spec.md)** - Fleet YAML specification
- **[Trigger Spec](reference/trigger-spec.md)** - Trigger YAML specification (platform + command routing)
- **[AgentFlow Spec](reference/agentflow-spec.md)** - Multi-step workflow specification
- **[Context Spec](reference/context-spec.md)** - Execution environment specification
- **[DaemonConfig](reference/daemon-config.md)** - Server configuration
- **[Platform Policies](reference/platform-policies.md)** - Safety rules per platform

### Platform Integrations
- **[GitHub Integration](reference/github-integration.md)** - GitHub webhooks, events, API actions

## Tutorials

1. **[Build Your First Agent](tutorials/first-agent.md)** - 15 minutes
2. **[Create a Slack Bot](tutorials/slack-bot.md)** - 20 minutes
3. **[Telegram Ops Bot](tutorials/telegram-ops-bot.md)** - Mobile DevOps
4. **[Multi-Model RCA](tutorials/multi-model-rca-quickstart.md)** - Consensus analysis

## Agent Library

Single-purpose building blocks in `examples/agents/library/`:

| Agent | Tools | Purpose |
|-------|-------|---------|
| `k8s-agent` | kubectl, helm | Kubernetes operations |
| `docker-agent` | docker | Container management |
| `git-agent` | git | Version control |
| `aws-agent` | aws cli | AWS cloud operations |
| `terraform-agent` | terraform | Infrastructure as Code |
| `prometheus-agent` | prometheus_query | Metrics analysis |
| `loki-agent` | loki_query | Log analysis |
| `postgres-agent` | psql | PostgreSQL operations |
| `redis-agent` | redis-cli | Redis operations |

## Built-in Fleets

Composed teams in `examples/fleets/`:

| Fleet | Agents | Purpose |
|-------|--------|---------|
| **DevOps** | k8s + docker + git + prometheus | Full-stack DevOps |
| **Kubernetes** | k8s + prometheus + loki | K8s cluster operations |
| **AWS** | aws + terraform | AWS cloud infrastructure |
| **Database** | postgres + redis | Database operations |
| **RCA** | collectors + multi-model analysts | Root cause analysis |

## Platform Safety

| Platform | Read | Write |
|----------|------|-------|
| CLI | Yes | Yes |
| Slack | Yes | Yes (with approval) |
| Telegram | Yes | No |
| WhatsApp | Yes | No |

## Commands

```
/agent               # List and switch agents (interactive)
/agent k8s           # Switch to Kubernetes agent
/agent info          # Show current agent details
/help                # Show help
```

After selecting a fleet, just type naturally:
```
list pods in production
show deployment status
what's the memory usage?
```

## Architecture (Advanced)

- **[Fleets Deep Dive](concepts/fleets.md)** - Coordination modes and consensus
- **[AgentFlow Routing](guides/agentflow-routing.md)** - Message routing patterns
- **[Multi-Tenant Architecture](architecture/multi-tenant-agentflows.md)** - Enterprise patterns
- **[Multi-Model Consensus](architecture/multi-model-consensus.md)** - Multi-AI coordination

## Examples

- `examples/agents/library/` - Single-purpose agent library
- `examples/fleets/` - Fleet compositions
- `examples/triggers/` - Trigger configurations with command routing
- `examples/flows/` - Multi-step workflow examples
- `examples/config/daemon.yaml` - Unified daemon config

## Support

- GitHub Issues: https://github.com/agenticdevops/aof/issues
- Documentation: https://docs.aof.sh

---

Last updated: 2025-12-20
