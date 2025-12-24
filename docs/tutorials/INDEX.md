# AOF Tutorials

Hands-on tutorials for building AI-powered DevOps bots with AOF.

## Platform Tutorials

### Messaging Bots

- **[Slack Bot Tutorial](./slack-bot.md)** - Build a Slack bot with approval workflow
  - Human-in-the-loop approval for destructive commands
  - Reaction-based approve/deny workflow
  - Conversation memory per channel/thread
  - Full read/write operations

- **[Telegram Ops Bot](./telegram-ops-bot.md)** - Mobile-friendly bot for on-call engineers
  - Inline keyboard buttons for agent/fleet switching
  - Read-only safety mode for mobile
  - Quick status checks from your phone
  - `/agent` and `/fleet` commands

### GitHub Automation

- **[Automated PR Review](./pr-review-automation.md)** - Set up AI-powered PR review automation
  - Multi-agent review fleet (security, performance, quality)
  - GitHub webhook trigger integration
  - Consensus-based review with weighted votes
  - Automatic PR comments and labeling
  - Complete step-by-step setup guide

### Jira Automation

- **[Jira Automation](./jira-automation.md)** - Automate Jira workflows with AI
  - Automatic bug triage with priority and assignee suggestions
  - Sprint planning assistance and capacity calculation
  - Daily standup reports and burndown analysis
  - Sprint retrospectives with actionable insights
  - Slash commands: `/triage`, `/standup`, `/retro`, `/blockers`
  - Saves 5-10 hours per week for agile teams

### Getting Started

- **[Your First Agent](./first-agent.md)** - Create and run your first agent
  - Basic Agent YAML structure
  - Running with `aofctl run agent`
  - Adding tools (kubectl, shell)

### Advanced Topics

- **[Incident Response Automation](./incident-response-automation.md)** - End-to-end incident automation
  - PagerDuty/Opsgenie webhook integration
  - Multi-agent incident response pipeline
  - Automated triage, RCA, and postmortem generation
  - Observability tool integration (Grafana/Prometheus/Loki)
  - Production-ready with signature verification

- **[Incident Response](./incident-response.md)** - Automated incident workflows
  - Multi-step incident handling
  - Integration with PagerDuty/Slack
  - Escalation workflows

- **[RCA Fleet Tutorial](./rca-fleet.md)** - Root Cause Analysis with multiple agents
  - Fleet-based agent coordination
  - Automatic routing to specialists
  - Collecting and synthesizing findings

- **[Multi-Model RCA Quickstart](./multi-model-rca-quickstart.md)** - Quick introduction to multi-model consensus
  - Different AI models for diverse perspectives
  - Consensus-based analysis

- **[Multi-Model RCA](./multi-model-rca.md)** - Deep dive into multi-model analysis
  - Advanced consensus patterns
  - Model selection strategies

## Quick Start

### 1. Install AOF

```bash
curl -sSL https://docs.aof.sh/install.sh | bash
```

### 2. Create an Agent

```yaml
# agents/k8s-ops.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Kubernetes operations assistant.
    Help users run kubectl commands and troubleshoot issues.
  tools:
    - kubectl
    - helm
```

### 3. Run the Agent

```bash
# Interactive mode
aofctl run agent k8s-ops --agents-dir ./agents

# Or start as a server for Slack/Telegram
aofctl serve \
  --config config/daemon.yaml \
  --agents-dir ./agents
```

## Available Platforms

| Platform | Commands | Features |
|----------|----------|----------|
| **Slack** | Natural language | Approval workflow, reactions, threads |
| **Telegram** | `/help`, `/agent`, `/fleet` | Inline keyboards, read-only mode |
| **CLI** | `aofctl run agent` | Interactive, scriptable |

## Bot Commands

All platform bots support these commands:

| Command | Description |
|---------|-------------|
| `/help` | Show help and current agent |
| `/agent` | List agents with selection UI |
| `/agent <name>` | Switch to specific agent |
| `/fleet` | List fleets with selection UI |
| `/fleet <name>` | Switch to specific fleet |
| `/run agent <name> <query>` | Run specific agent once |
| `/status` | Show task status |

## Resources

- [Agent Spec Reference](../reference/agent-spec.md) - Complete Agent YAML reference
- [Fleet Spec Reference](../reference/fleet-spec.md) - Fleet configuration
- [DaemonConfig Reference](../reference/daemon-config.md) - Server configuration
- [aofctl CLI Reference](../reference/aofctl.md) - All CLI commands

## Getting Help

- [GitHub Issues](https://github.com/agenticdevops/aof/issues) - Report bugs or request features
- [Documentation](https://docs.aof.sh) - Full documentation
- [Examples](https://github.com/agenticdevops/aof/tree/main/examples) - Ready-to-use configurations
