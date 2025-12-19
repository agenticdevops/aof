# Telegram Mobile Guide

Safe, read-only access to your DevOps infrastructure from Telegram.

## Quick Start

See [Telegram Quickstart](quickstart-telegram.md) for setup instructions.

## Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help and available fleets |
| `/fleet` | Switch between fleets |
| `/fleet devops` | Switch to DevOps fleet |
| `/fleet k8s` | Switch to Kubernetes fleet |
| `/fleet info` | Show current fleet details |

## Usage Flow

1. **Send `/help` or `/fleet`** - Shows available fleets as buttons
2. **Tap a fleet** - Switches to that fleet
3. **Send natural messages** - Fleet routes to the right specialist

Example:
```
You: /fleet
Bot: Select Fleet
     Current: DevOps

     [DevOps] [Kubernetes] [AWS] [Database] [RCA]

You: *tap Kubernetes*
Bot: Switched to Kubernetes
     Agents: k8s, prometheus, loki

You: list pods in production
Bot: NAME                    READY   STATUS
     api-7d8f9c6b5-x2k3n    1/1     Running
     web-5c4d3b2a1-p9m8l    1/1     Running
```

## How Fleets Work

Fleets are teams of single-purpose agents. When you ask a question, the fleet routes to the right specialist:

```
You: "check pod logs"
         │
         ▼
    DevOps Fleet
    (coordinator)
         │
         ▼
    k8s-agent  ← kubectl specialist
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

## Built-in Fleets

| Fleet | Agents | Purpose |
|-------|--------|---------|
| **DevOps** | k8s + docker + git + prometheus | Full-stack DevOps |
| **Kubernetes** | k8s + prometheus + loki | K8s cluster operations |
| **AWS** | aws + terraform | AWS cloud infrastructure |
| **Database** | postgres + redis | Database operations |
| **RCA** | collectors + multi-model analysts | Root cause analysis |

## Troubleshooting

**Bot not responding?**
- Check webhook is set correctly
- Check server logs for errors

**Fleet has no agents?**
- Ensure `--fleets-dir` points to fleet YAML files
- Check fleet file has correct `metadata.name`

**"Write operation blocked"?**
- Expected behavior - Telegram is read-only
- Use Slack or CLI for write operations

## Related

- [Quickstart](quickstart-telegram.md) - Setup guide
- [Fleet Switching](agent-switching.md) - Full command reference
- [Safety Layer](safety-layer.md) - Platform policies
