# AgentFlow Routing Guide

This guide explains how AOF routes messages to agents using AgentFlows, and how to configure routing for your bots.

## Overview

AgentFlow routing is the system that determines **which agent handles which message**. Instead of hardcoding a single agent for all messages, you can:

- Route different message patterns to different agents
- Restrict agents to specific channels or users
- Run multiple bots on different platforms from a single daemon
- Implement multi-tenant deployments for enterprises

## How Routing Works

When a message arrives, AOF follows this decision tree:

```
Message Arrives (Telegram/Slack/Discord/WhatsApp)
        │
        ▼
┌───────────────────────────────────────┐
│ Step 1: FlowRouter.route_best()       │
│ - Load all flows from flows.directory │
│ - Score each flow against message     │
│ - Pick highest scoring match          │
└───────────────┬───────────────────────┘
                │
        ┌───────┴───────┐
        │               │
        ▼               ▼
   [Match Found]   [No Match]
        │               │
        ▼               ▼
┌───────────────┐ ┌─────────────────────┐
│ Execute       │ │ Step 2: Parse as    │
│ AgentFlow     │ │ command (/run, etc) │
└───────────────┘ └──────────┬──────────┘
                             │
                     ┌───────┴───────┐
                     │               │
                     ▼               ▼
                [Command]      [Not Command]
                     │               │
                     ▼               ▼
              ┌────────────┐ ┌──────────────────┐
              │ Execute    │ │ Step 3: Route to │
              │ Command    │ │ default_agent    │
              │ Handler    │ │ from config      │
              └────────────┘ └──────────────────┘
```

## Flow Scoring Algorithm

When multiple flows could match a message, the FlowRouter scores each one:

| Filter Type | Score | Description |
|-------------|-------|-------------|
| Exact channel match | +100 | Message from a configured channel |
| User whitelist match | +80 | Message from an allowed user |
| Pattern regex match | +60 | Message text matches a pattern |
| Platform match | +40 | Correct platform (telegram, slack, etc.) |
| No filters (catch-all) | +10 | Flow with no restrictions |

**Highest score wins.** If scores tie, first defined flow wins.

### Example Scoring

```yaml
# Flow A: score = 40 (platform only)
trigger:
  type: Telegram

# Flow B: score = 40 + 60 = 100 (platform + pattern)
trigger:
  type: Telegram
  config:
    patterns: ["kubectl"]

# Flow C: score = 40 + 100 = 140 (platform + channel)
trigger:
  type: Telegram
  config:
    chat_ids: [-1001234567890]
```

Message "kubectl get pods" in chat -1001234567890:
- Flow A: 40 points
- Flow B: 100 points
- Flow C: 140 points ← **Winner**

## Directory Structure

```
your-project/
├── daemon-config.yaml      # Main config file
├── agents/                 # Agent definitions
│   ├── k8s-ops.yaml       # metadata.name: k8s-ops
│   ├── incident-agent.yaml # metadata.name: incident-responder
│   └── dev-assistant.yaml  # metadata.name: dev-assistant
│
└── flows/                  # AgentFlow definitions
    ├── telegram-k8s.yaml   # Routes K8s messages
    ├── telegram-incidents.yaml
    └── slack-default.yaml
```

## Configuration Reference

### Daemon Config

```yaml
# daemon-config.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: my-bot-server

spec:
  # Agent discovery
  agents:
    directory: "./agents"    # Path to agent YAML files
    watch: false             # Hot-reload on changes (optional)

  # AgentFlow routing
  flows:
    directory: "./flows"     # Path to flow YAML files
    watch: false             # Hot-reload on changes (optional)
    enabled: true            # Enable flow routing

  runtime:
    # Fallback when no flow matches
    default_agent: "k8s-ops" # Must exist in agents directory
```

### AgentFlow Spec

```yaml
# flows/my-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: my-flow-name        # Unique identifier
  labels:                   # Optional labels for organization
    platform: telegram
    environment: production

spec:
  # Trigger configuration
  trigger:
    type: Telegram          # Platform: Telegram, Slack, Discord, WhatsApp
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}

      # FILTERS (all optional - omit for catch-all)
      chat_ids: []          # Telegram chat IDs
      channels: []          # Slack channel names
      users: []             # User IDs/usernames
      patterns: []          # Regex patterns to match message text

  # Agent(s) to handle matched messages
  agents:
    - name: k8s-ops         # Must match metadata.name in agents/
      patterns: []          # Optional: sub-routing within flow
      description: ""       # Optional: for documentation

  # Environment context (passed to agent)
  context:
    kubeconfig: ${KUBECONFIG}
    namespace: default
    env:
      CUSTOM_VAR: "value"

  # Approval workflow (optional)
  approval:
    enabled: true
    allowed_users: ["U12345"]
    require_for:
      - "kubectl delete"
```

## Common Patterns

### Pattern 1: Single Agent for All Messages

The simplest setup - one agent handles everything:

```yaml
# flows/default.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: default-flow

spec:
  trigger:
    type: Telegram
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}
      # No filters = matches ALL messages

  agents:
    - name: k8s-ops
```

### Pattern 2: Pattern-Based Routing

Route different types of requests to specialized agents:

```yaml
# flows/k8s-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: k8s-flow

spec:
  trigger:
    type: Telegram
    config:
      patterns:
        - "kubectl"
        - "k8s"
        - "pod"
        - "deploy"
        - "namespace"

  agents:
    - name: k8s-ops
---
# flows/incident-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-flow

spec:
  trigger:
    type: Telegram
    config:
      patterns:
        - "incident"
        - "outage"
        - "alert"
        - "page"

  agents:
    - name: incident-responder
---
# flows/default-flow.yaml (catch-all, lowest priority)
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: default-flow

spec:
  trigger:
    type: Telegram
    # No patterns = catch-all for unmatched messages

  agents:
    - name: general-assistant
```

### Pattern 3: Channel-Based Routing (Slack)

Route different Slack channels to different agents/clusters:

```yaml
# flows/prod-k8s.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: prod-k8s-flow

spec:
  trigger:
    type: Slack
    config:
      channels:
        - production
        - prod-alerts
        - sre-oncall

  agents:
    - name: k8s-ops

  context:
    kubeconfig: ${KUBECONFIG_PROD}
    env:
      CLUSTER: "production"
      REQUIRE_APPROVAL: "true"
---
# flows/staging-k8s.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: staging-k8s-flow

spec:
  trigger:
    type: Slack
    config:
      channels:
        - staging
        - dev-test

  agents:
    - name: k8s-ops

  context:
    kubeconfig: ${KUBECONFIG_STAGING}
    env:
      CLUSTER: "staging"
      REQUIRE_APPROVAL: "false"  # No approval in staging
```

### Pattern 4: User-Based Routing

Restrict certain agents to specific users:

```yaml
# flows/admin-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: admin-flow

spec:
  trigger:
    type: Slack
    config:
      users:
        - U015ADMIN      # Slack user IDs
        - U016SRELEAD

  agents:
    - name: admin-agent  # Has dangerous tools access

  approval:
    allowed_users:
      - U015ADMIN        # Self-approval for admins
---
# flows/developer-flow.yaml (everyone else)
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: developer-flow

spec:
  trigger:
    type: Slack
    # No user filter = matches all users

  agents:
    - name: dev-assistant  # Read-only access
```

### Pattern 5: Multi-Platform Same Agent

Share an agent across platforms:

```yaml
# flows/telegram-assistant.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: telegram-assistant

spec:
  trigger:
    type: Telegram
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}

  agents:
    - name: general-assistant
---
# flows/slack-assistant.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-assistant

spec:
  trigger:
    type: Slack
    config:
      bot_token: ${SLACK_BOT_TOKEN}

  agents:
    - name: general-assistant  # Same agent!
---
# flows/discord-assistant.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: discord-assistant

spec:
  trigger:
    type: Discord
    config:
      bot_token: ${DISCORD_BOT_TOKEN}

  agents:
    - name: general-assistant  # Same agent!
```

### Pattern 6: Combining Filters

Combine multiple filters for precise routing:

```yaml
# flows/prod-k8s-admin.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: prod-k8s-admin-flow

spec:
  trigger:
    type: Slack
    config:
      # ALL conditions must match (AND logic)
      channels:
        - production
      users:
        - U015ADMIN
        - U016SRELEAD
      patterns:
        - "kubectl"
        - "helm"

  agents:
    - name: k8s-admin-agent

  context:
    kubeconfig: ${KUBECONFIG_PROD}
```

## Agent Configuration

Agents referenced in flows must exist in the `agents.directory`:

```yaml
# agents/k8s-ops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-ops           # ← Referenced by flows as "k8s-ops"
  labels:
    category: infrastructure

spec:
  model: google:gemini-2.5-flash  # Required: LLM model

  tools:                   # Available tools
    - kubectl
    - helm
    - docker
    - shell

  system_prompt: |         # Agent behavior instructions
    You are a Kubernetes expert...

  max_tokens: 4096
  temperature: 0.3
```

**Important:** The `metadata.name` in the agent file must match the `agents[].name` in the flow.

## Debugging Routing

### Check What's Loaded

Start the server with debug logging:

```bash
RUST_LOG=debug aofctl serve --config daemon-config.yaml
```

Look for these log messages:

```
INFO  Pre-loaded 3 agents from "./agents"
INFO  Loaded AgentFlow 'telegram-k8s-flow' for platform 'telegram'
INFO  Loaded AgentFlow 'slack-default-flow' for platform 'slack'
```

### Check Routing Decisions

When a message arrives:

```
INFO  Matched AgentFlow 'telegram-k8s-flow' for message (score: 100, reason: PatternMatch)
INFO  Using pre-loaded agent: k8s-ops
```

### Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| "No tool executor, tools will be empty" | Agent not pre-loaded | Check agents directory path |
| "Routing to default agent" | No flow matched | Add a catch-all flow |
| "Agent not found" | metadata.name mismatch | Ensure flow references correct agent name |
| "Using fallback model" | Agent not in runtime | Verify agent YAML is valid |

## Best Practices

### 1. Always Have a Catch-All Flow

```yaml
# flows/default.yaml
spec:
  trigger:
    type: Telegram
    # No filters = lowest priority catch-all
  agents:
    - name: general-assistant
```

### 2. Use Specific Patterns Over Broad Ones

```yaml
# ❌ Too broad - matches everything with "get"
patterns: ["get"]

# ✅ Specific - matches K8s commands
patterns: ["kubectl get", "kubectl describe", "k8s"]
```

### 3. Organize Flows by Environment

```
flows/
├── production/
│   ├── k8s-flow.yaml
│   └── incident-flow.yaml
├── staging/
│   └── dev-flow.yaml
└── shared/
    └── default-flow.yaml
```

### 4. Document Your Flows

```yaml
metadata:
  name: prod-k8s-flow
  labels:
    environment: production
    team: platform
    on-call: sre-team
  annotations:
    description: "Production K8s operations for SRE team"
    owner: "platform-team@company.com"
```

### 5. Test Routing Before Production

```bash
# Dry-run to see which flow would match
aofctl flow test --message "kubectl get pods" --platform telegram
# Output: Would route to 'telegram-k8s-flow' (score: 100)
```

## Complete Example

Here's a complete multi-agent setup:

```
my-bot/
├── daemon-config.yaml
├── agents/
│   ├── k8s-ops.yaml
│   ├── incident-responder.yaml
│   └── general-assistant.yaml
└── flows/
    ├── k8s-flow.yaml
    ├── incident-flow.yaml
    └── default-flow.yaml
```

**daemon-config.yaml:**
```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-agent-bot

spec:
  platforms:
    telegram:
      enabled: true
      bot_token_env: "TELEGRAM_BOT_TOKEN"

  agents:
    directory: "./agents"

  flows:
    directory: "./flows"
    enabled: true

  runtime:
    default_agent: "general-assistant"
```

**Start the bot:**
```bash
export TELEGRAM_BOT_TOKEN="your-token"
export GOOGLE_API_KEY="your-api-key"
aofctl serve --config daemon-config.yaml
```

## Related Documentation

- [Multi-Tenant Architecture](../architecture/multi-tenant-agentflows.md)
- [Agent Configuration Reference](../reference/agent-spec.md)
- [Trigger Reference](../reference/trigger-spec.md)
- [FlowBinding Reference](../reference/flowbinding-spec.md)
