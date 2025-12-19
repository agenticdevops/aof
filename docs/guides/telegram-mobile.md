# Telegram Mobile Guide

Safe, read-only access to your DevOps infrastructure from Telegram.

## Quick Start

See [Telegram Quickstart](quickstart-telegram.md) for setup instructions.

## Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help and available agents |
| `/agent` | Switch between agents (interactive) |
| `/agent k8s` | Switch to Kubernetes agent |
| `/agent aws` | Switch to AWS agent |
| `/agent info` | Show current agent details |

## Usage Flow

1. **Send `/help` or `/agent`** - Shows available agents as buttons
2. **Tap an agent** - Switches to that agent
3. **Send natural messages** - Agent handles your query

Example:
```
You: /agent
Bot: Select Agent
     Current: Kubernetes

     [Kubernetes] [AWS] [Docker] [Git]

You: *tap Kubernetes*
Bot: Switched to Kubernetes
     Tools: kubectl, helm

You: list pods in production
Bot: NAME                    READY   STATUS
     api-7d8f9c6b5-x2k3n    1/1     Running
     web-5c4d3b2a1-p9m8l    1/1     Running
```

## How Agents Work

When you ask a question, the selected agent handles your request:

```
You: "check pod logs"
         │
         ▼
    Kubernetes Agent
    (kubectl, helm)
         │
         ▼
    Response with logs
```

## Safety

Telegram is **read-only** by default:
- All write operations are blocked (kubectl apply, docker rm, etc.)
- Use Slack or CLI for write operations
- This protects against accidental commands from mobile

When blocked, you'll see:
```
Write operations are blocked on Telegram.
Use Slack or CLI for this operation.
```

## Built-in Agents

| Agent | Tools | Purpose |
|-------|-------|---------|
| **k8s** | kubectl, helm | Kubernetes operations |
| **aws** | aws cli | AWS cloud operations |
| **docker** | docker | Container management |
| **git** | git | Version control |
| **devops** | kubectl, docker, helm, git, shell | Full-stack DevOps |

## Troubleshooting

**Bot not responding?**
- Check webhook is set correctly
- Check server logs for errors

**Agent not found?**
- Ensure `agents.directory` in config points to agent YAML files
- Check agent file has correct `metadata.name`

**"Write operation blocked"?**
- Expected behavior - Telegram is read-only
- Use Slack or CLI for write operations

## Related

- [Quickstart](quickstart-telegram.md) - Setup guide
- [Agent Switching](agent-switching.md) - Full command reference
- [Safety Layer](safety-layer.md) - Platform policies
