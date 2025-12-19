# Composable Architecture

AOF follows a simple, composable model with four core concepts.

## Overview

The architecture is organized into four layers, from atomic resources to platform routing:

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: TRIGGERS (Platform + Command Routing)                  │
│ Self-contained routing: platform config + command bindings      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Trigger: slack-prod                                         │ │
│ │ ├─ Platform: Slack (bot_token, channels)                   │ │
│ │ ├─ Commands: /kubectl → k8s-agent                          │ │
│ │ │           /diagnose → rca-fleet                          │ │
│ │ │           /deploy → deploy-flow                          │ │
│ │ └─ Default: devops-agent                                   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: FLOWS (Multi-Step Workflows)                           │
│ Orchestration logic with nodes and connections                  │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ AgentFlow: deploy-flow                                      │ │
│ │ validate → approval → deploy → notify                       │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: FLEETS (Agent Composition)                            │
│ Compose multiple agents for collaboration                       │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ AgentFleet: rca-fleet                                       │ │
│ │ k8s-collector + prometheus-collector → analysts → synthesis │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: AGENTS (Atomic Units)                                 │
│ Single-purpose AI specialists                                   │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│ │ k8s-agent│  │prometheus│  │ loki     │  │ docker   │        │
│ └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## The Four Concepts

| Concept | Purpose | Example |
|---------|---------|---------|
| **Agent** | Single-purpose specialist | `k8s-agent`, `prometheus-agent` |
| **Fleet** | Team of agents | `devops-fleet`, `rca-fleet` |
| **Flow** | Multi-step workflow | `deploy-flow`, `incident-flow` |
| **Trigger** | Platform + command routing | `slack-prod`, `telegram-oncall` |

## How Commands Are Routed

Triggers contain command bindings that route to agents, fleets, or flows:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}

  commands:
    /kubectl:
      agent: k8s-agent        # Single agent
    /diagnose:
      fleet: rca-fleet        # Multi-agent team
    /deploy:
      flow: deploy-flow       # Multi-step workflow

  default_agent: devops       # Fallback for natural language
```

When a user sends `/diagnose pod crashing`, the message is routed to `rca-fleet`.

## Reusability

Agents are defined once and referenced by multiple triggers:

```yaml
# agents/k8s-agent.yaml - Define once
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-agent
spec:
  model: google:gemini-2.5-flash
  tools: [kubectl, helm]
```

```yaml
# triggers/slack-prod.yaml - Reference in production
commands:
  /kubectl:
    agent: k8s-agent

# triggers/telegram-oncall.yaml - Reference for on-call
commands:
  /k8s:
    agent: k8s-agent   # Same agent, different platform
```

## Context Injection

Contexts provide environment boundaries for agent execution:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  approval:
    required: true
    allowed_users: [U12345678]
```

Flows reference contexts:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
spec:
  context:
    ref: prod
  nodes:
    - id: deploy
      type: Agent
      config:
        agent: k8s-agent
```

## Multi-Platform Deployment

Same agents, different platforms:

```yaml
# Slack trigger
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
  commands:
    /kubectl:
      agent: k8s-agent
---
# Telegram trigger
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-oncall
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
  commands:
    /k8s:
      agent: k8s-agent   # Same agent!
```

## See Also

- [Core Concepts](../introduction/concepts.md) - Mental model
- [Trigger Reference](../reference/trigger-spec.md) - Complete trigger specification
- [AgentFlow Reference](../reference/agentflow-spec.md) - Workflow specification
- [Context Reference](../reference/context-spec.md) - Environment configuration
