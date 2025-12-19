# Telegram Mobile Companion

The Telegram Mobile Companion provides a safe, read-first interface for interacting with AOF agents from your mobile device. It's built on AOF's **platform-agnostic safety layer**, which means the same safety framework that protects Telegram also protects Slack, WhatsApp, Discord, and any future platform.

## Overview

Telegram is designed for on-the-go DevOps:
- Quick status checks from anywhere
- Read-only by default for safety
- Interactive agent selection via inline keyboards
- Approval workflow for sensitive operations
- **Platform-agnostic safety**: Same rules apply across all messaging platforms

## Quick Start

### 1. Connect Your Bot

```bash
# Set your Telegram bot token
export TELEGRAM_BOT_TOKEN="your-bot-token"

# Start AOF with Telegram trigger
aofctl serve --platform telegram --agents ./agents
```

### 2. Start Chatting

Open Telegram and message your bot:

```
/agents
```

This shows available agents as inline buttons. Tap one to select it.

## Commands

| Command | Description |
|---------|-------------|
| `/agents` | Show available agents (inline keyboard) |
| `/flows` | Show available flows (inline keyboard) |
| `/help` | Show help information |
| `/run agent <name> <message>` | Run specific agent |
| `/status task <id>` | Check task status |
| `/list tasks` | List active tasks |

## Agent Selection Flow

1. **Send `/agents`** - Bot displays inline keyboard with agent buttons
2. **Tap an agent** - Bot confirms: "Switched to agent: *k8s-status*"
3. **Send natural messages** - All messages route to selected agent
4. **Switch agents anytime** - Send `/agents` again to change

Example:
```
You: /agents
Bot: Select an Agent
     [k8s-status] [docker-status] [git-status]

You: *tap k8s-status*
Bot: Switched to agent: *k8s-status*
     You can now send messages and I'll respond using this agent.

You: what pods are running in production?
Bot: 🤔 Thinking...
     Here are the pods in the production namespace:
     NAME                    READY   STATUS    RESTARTS
     api-7d8f9c6b5-x2k3n    1/1     Running   0
     web-5c4d3b2a1-p9m8l    1/1     Running   0
```

## Safety Features

### Read-Only by Default

Telegram operations are classified and filtered:

| Action Class | Telegram Policy |
|--------------|-----------------|
| Read | ✅ Allowed |
| Write | ⚠️ Requires approval |
| Delete | 🚫 Blocked |
| Dangerous | 🚫 Blocked |

### Blocked Operations

These operations are blocked on Telegram:
- `kubectl delete`
- `kubectl exec`
- `docker rm`
- `helm uninstall`
- Any `--force` flags

When blocked, you'll see:
```
🚫 Delete operations are blocked on Telegram.
Use Slack or kubectl directly for this operation.
```

### Approval Workflow

For operations that need approval (configured per context):

1. Agent proposes the action
2. Approval message appears (on Slack/CLI)
3. Authorized user approves
4. Action executes
5. Result sent back to Telegram

## Example Agents

### k8s-status (Read-Only)

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-status
  labels:
    mobile-safe: "true"
spec:
  model:
    provider: anthropic
    name: claude-3-5-sonnet-20241022
  tools:
    - kubectl:get
    - kubectl:describe
    - kubectl:logs
  system_prompt: |
    You are a Kubernetes status agent for mobile users.
    Only use READ operations: get, describe, logs.
    Never suggest delete or exec operations.
```

### docker-status (Read-Only)

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: docker-status
  labels:
    mobile-safe: "true"
spec:
  model:
    provider: anthropic
    name: claude-3-5-sonnet-20241022
  tools:
    - docker:ps
    - docker:images
    - docker:logs
    - docker:stats
  system_prompt: |
    You are a Docker status agent for mobile monitoring.
    Only use READ operations: ps, images, logs, stats.
    Never suggest rm, stop, or exec operations.
```

## Configuration

### Context for Telegram

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-mobile
spec:
  namespace: production
  default_agent: k8s-status

  platform_policies:
    telegram:
      blocked_classes:
        - delete
        - dangerous
      approval_classes:
        - write
      allowed_classes:
        - read
      blocked_message: |
        This operation requires CLI or Slack access.
```

### Environment Variables

```bash
# Required
TELEGRAM_BOT_TOKEN=your-bot-token

# Optional
TELEGRAM_ALLOWED_USERS=user_id_1,user_id_2  # Restrict access
TELEGRAM_DEFAULT_AGENT=k8s-status           # Default agent
```

## Best Practices

1. **Use read-only agents** - Create mobile-specific agents with only read tools
2. **Label mobile-safe agents** - Use `mobile-safe: "true"` label
3. **Keep responses short** - Mobile screens are small
4. **Use structured output** - Tables and lists work well
5. **Avoid long-running tasks** - Mobile connections are unreliable

## Troubleshooting

### Bot not responding

1. Check bot token is correct
2. Verify bot is added to the chat
3. Check server logs for errors

### Commands not working

1. Ensure commands start with `/`
2. Check agent is loaded: `/agents`
3. Verify agent has appropriate tools

### Operations blocked

1. Check your context's platform_policies
2. Verify the action class classification
3. Use CLI or Slack for write/delete operations

## Testing the Implementation

### Run Unit Tests

```bash
# Test safety module (18 tests)
cargo test --package aof-triggers -- safety

# Test visualization crate (27 tests)
cargo test --package aof-viz

# Test all
cargo test --all
```

### Available Test Examples

| Example | Location | Purpose |
|---------|----------|---------|
| k8s-status | `examples/agents/mobile-read-only/k8s-status.yaml` | Read-only K8s monitoring |
| docker-status | `examples/agents/mobile-read-only/docker-status.yaml` | Container status checks |
| git-status | `examples/agents/mobile-read-only/git-status.yaml` | Repository status |
| prometheus-query | `examples/agents/mobile-read-only/prometheus-query.yaml` | Metrics queries |
| helm-status | `examples/agents/mobile-read-only/helm-status.yaml` | Helm release status |

### Platform Policy Examples

| Example | Location | Policy |
|---------|----------|--------|
| Production | `examples/contexts/telegram-prod.yaml` | Read-only, blocks all writes |
| Development | `examples/contexts/telegram-dev.yaml` | Allows writes with approval |
| Personal | `examples/contexts/telegram-personal.yaml` | Relaxed for personal use |

### End-to-End Test

```bash
# 1. Set your Telegram bot token
export TELEGRAM_BOT_TOKEN="your-bot-token"

# 2. Start server with mobile agents
aofctl serve \
  --platform telegram \
  --agents examples/agents/mobile-read-only \
  --context examples/contexts/telegram-prod.yaml

# 3. Test in Telegram:
#    - Send /agents → Select k8s-status
#    - Send "what pods are running" → Should work
#    - Send "delete the pod" → Should be blocked
```

## Platform-Agnostic Architecture

The Telegram implementation uses AOF's shared safety components:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          SAFETY LAYER ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐                                                   │
│  │ Incoming Message │  (Telegram, Slack, WhatsApp, Discord, etc.)       │
│  └────────┬─────────┘                                                   │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ TriggerHandler                                                    │   │
│  │ ├── Parse commands (/agents, /flows, /help)                      │   │
│  │ └── Route to safety layer                                        │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Safety Layer (platform-agnostic)                                  │   │
│  │ ├── ToolClassifier.classify(command) → ActionClass               │   │
│  │ ├── PolicyEngine.evaluate(platform, class) → Decision            │   │
│  │ └── Same logic for ALL platforms                                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Response Rendering (aof-viz)                                      │   │
│  │ └── Platform-specific RenderConfig (width, colors, compact)      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

This means:
- **Same safety rules** apply to Telegram, Slack, WhatsApp, Discord
- **Configuration-driven**: Change policies in YAML, not code
- **Extensible**: Add new platforms without modifying safety logic

## Related

- [Safety Layer](/docs/guides/safety-layer.md) - Platform-agnostic safety framework
- [Agents Reference](/docs/reference/agent.md)
- [Context Reference](/docs/reference/context.md)
- [Approval Workflows](/docs/guides/approvals.md)
- [Platform Policies Reference](/docs/reference/platform-policies.md)
