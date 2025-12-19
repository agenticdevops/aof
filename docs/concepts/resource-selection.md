# Choosing the Right Resource: DaemonConfig vs AgentFlow vs FlowBinding

AOF provides multiple approaches for connecting platforms to agents. This guide helps you choose the right architecture for your use case.

## Quick Decision Tree

```
Do you need multi-step workflows (transform → agent → response)?
├─ No → Use DaemonConfig + Agent (simple)
└─ Yes → Do you need multi-tenant deployment?
         ├─ No → Use AgentFlow (workflow orchestration)
         └─ Yes → Use FlowBinding (composable architecture)
```

## Comparison Table

| Feature | DaemonConfig | AgentFlow | FlowBinding |
|---------|--------------|-----------|-------------|
| **Complexity** | Simple | Medium | Advanced |
| **Use case** | Single agent bots | Multi-step workflows | Enterprise multi-tenant |
| **Agent routing** | `default_agent` + `/agent` command | Flow nodes | Binding composition |
| **Multi-step flows** | No | Yes | Yes (via AgentFlow ref) |
| **Parallel execution** | No | Yes | Yes (via AgentFlow ref) |
| **Conditional routing** | No | Yes | Yes |
| **Context injection** | No | Per-flow | Per-binding |
| **Multi-tenant** | No | Limited | Yes |
| **Approval workflow** | Agent-based | Node-based | Context-based |
| **Resource reuse** | Low | Medium | High |

---

## Option 1: DaemonConfig + Agent (Simple)

**Best for:** Slack/Telegram bots, single-agent interactions, quick setup

### When to Use
- You have one agent (or a few selectable agents)
- Users interact via natural language or commands
- No complex orchestration needed
- MVP or proof-of-concept

### Architecture
```
┌─────────────────────────────────────────────────────┐
│ DaemonConfig                                        │
│ ├─ platforms: telegram, slack                       │
│ ├─ agents: directory: ./agents                      │
│ └─ runtime: default_agent: k8s-ops                  │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │ TriggerHandler   │
              │ /agent, /fleet   │
              │ Natural language │
              └──────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │ Agent: k8s-ops   │
              └──────────────────┘
```

### Example
```yaml
# config/telegram-bot.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-bot
spec:
  server:
    port: 8080
  platforms:
    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN
  agents:
    directory: ./agents
  fleets:
    directory: ./fleets
  runtime:
    default_agent: k8s-ops
```

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
  tools:
    - kubectl
    - helm
```

### User Interaction
```
User: /agent k8s-ops
Bot: Switched to k8s-ops agent

User: show pods in production
Bot: [runs kubectl get pods -n production]
```

### Pros
- Simple to set up
- Easy to understand
- Works today (implemented)

### Cons
- No multi-step workflows
- No parallel execution
- Limited routing logic

---

## Option 2: AgentFlow (Workflow Orchestration)

**Best for:** Multi-step processes, parallel execution, conditional logic

### When to Use
- You need to chain multiple agents
- You want parallel agent execution
- You need conditional branching
- Complex approval workflows with reactions

### Architecture
```
┌─────────────────────────────────────────────────────┐
│ AgentFlow: pr-review-flow                           │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Trigger: Slack (patterns: ["review PR"])        │ │
│ └─────────────────────────────────────────────────┘ │
│                        │                            │
│                        ▼                            │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Node: parse-pr (Transform)                      │ │
│ │ Extract PR URL from message                     │ │
│ └─────────────────────────────────────────────────┘ │
│                   /        \                        │
│                  ▼          ▼                       │
│ ┌──────────────────┐  ┌──────────────────┐         │
│ │ security-review  │  │ perf-review      │  ← Parallel
│ │ (Agent node)     │  │ (Agent node)     │         │
│ └──────────────────┘  └──────────────────┘         │
│                   \        /                        │
│                    ▼      ▼                         │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Node: summarize (Agent)                         │ │
│ │ Combine reviews into final report               │ │
│ └─────────────────────────────────────────────────┘ │
│                        │                            │
│                        ▼                            │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Node: send-response (Slack)                     │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Example
```yaml
# flows/pr-review-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: pr-review-flow
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      patterns: ["review PR", "PR review"]

  context:
    env:
      GITHUB_TOKEN: ${GITHUB_TOKEN}

  nodes:
    - id: parse-pr
      type: Transform
      config:
        script: |
          export PR_URL=$(echo "${event.text}" | grep -oP 'https://github.com/[^ ]+')

    - id: security-review
      type: Agent
      config:
        agent: security-reviewer
        input: "Review PR for security issues: ${PR_URL}"

    - id: perf-review
      type: Agent
      config:
        agent: perf-reviewer
        input: "Review PR for performance: ${PR_URL}"

    - id: summarize
      type: Agent
      config:
        agent: summary-agent
        input: |
          Combine these reviews:
          Security: ${security-review.output}
          Performance: ${perf-review.output}

    - id: respond
      type: Slack
      config:
        channel: ${event.channel_id}
        thread_ts: ${event.thread_ts}
        message: ${summarize.output}

  connections:
    - from: trigger
      to: parse-pr
    - from: parse-pr
      to: security-review
    - from: parse-pr
      to: perf-review    # Parallel with security-review
    - from: security-review
      to: summarize
    - from: perf-review
      to: summarize
    - from: summarize
      to: respond
```

### Node Types
| Node Type | Purpose |
|-----------|---------|
| `Transform` | Extract/transform data with scripts |
| `Agent` | Execute an AI agent |
| `Conditional` | Branch based on conditions |
| `Parallel` | Fan-out to multiple branches |
| `Join` | Merge parallel branches (all/any/majority) |
| `Slack` | Send Slack message |
| `HTTP` | Make HTTP request |
| `Wait` | Delay execution |
| `Approval` | Wait for human approval |
| `End` | Terminate flow |

### Pros
- Multi-step orchestration
- Parallel execution
- Conditional branching
- Rich node types

### Cons
- More complex configuration
- Trigger embedded in flow (less reusable)
- Single-tenant oriented

---

## Option 3: FlowBinding (Composable Architecture)

**Best for:** Enterprise deployments, multi-tenant, maximum reusability

### When to Use
- Same agent/flow used across multiple environments
- Need strict context boundaries (prod vs staging)
- Multi-tenant SaaS deployment
- Complex approval policies per context
- Maximum DRY (Don't Repeat Yourself)

### Architecture
```
┌─────────────────────────────────────────────────────┐
│ Standalone Resources (Define Once)                  │
├─────────────────────────────────────────────────────┤
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │
│ │ Trigger:     │ │ Agent:       │ │ AgentFlow:   │ │
│ │ slack-prod   │ │ k8s-ops      │ │ k8s-flow     │ │
│ └──────────────┘ └──────────────┘ └──────────────┘ │
│ ┌──────────────┐ ┌──────────────┐                  │
│ │ Context:     │ │ Context:     │                  │
│ │ prod         │ │ staging      │                  │
│ └──────────────┘ └──────────────┘                  │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│ FlowBindings (Compose Resources)                    │
├─────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────┐ │
│ │ FlowBinding: prod-k8s-binding                   │ │
│ │ ├─ trigger: slack-prod                          │ │
│ │ ├─ context: prod                                │ │
│ │ ├─ flow: k8s-flow (or agent: k8s-ops)          │ │
│ │ └─ match: patterns: ["kubectl", "k8s"]         │ │
│ └─────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ FlowBinding: staging-k8s-binding                │ │
│ │ ├─ trigger: slack-staging                       │ │
│ │ ├─ context: staging                             │ │
│ │ ├─ flow: k8s-flow (SAME flow!)                 │ │
│ │ └─ match: patterns: ["kubectl", "k8s"]         │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Example Resources

**Trigger (standalone):**
```yaml
# triggers/slack-prod.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
    channels:
      - production
      - prod-alerts
```

**Context (environment boundary):**
```yaml
# contexts/prod.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  env:
    AWS_PROFILE: prod
  approval:
    required_for:
      - "kubectl delete"
      - "kubectl scale"
      - "helm uninstall"
    allowed_users:
      - U12345678  # SRE Lead
```

**FlowBinding (composition):**
```yaml
# bindings/prod-k8s.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-k8s-binding
spec:
  trigger: slack-prod      # Reference to Trigger
  context: prod            # Reference to Context
  flow: k8s-ops-flow       # Reference to AgentFlow
  # OR for simple single-agent:
  # agent: k8s-ops         # Reference to Agent
  match:
    patterns:
      - "kubectl"
      - "k8s"
    priority: 100
  enabled: true
```

### Multi-Tenant Example
```yaml
# Same agent, different contexts for different customers

# Customer A binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: customer-a-k8s
spec:
  trigger: slack-customer-a
  context: customer-a-prod
  agent: k8s-ops           # Same agent!

---
# Customer B binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: customer-b-k8s
spec:
  trigger: slack-customer-b
  context: customer-b-prod
  agent: k8s-ops           # Same agent!
```

### Pros
- Maximum reusability (DRY)
- Clear separation of concerns
- Multi-tenant support
- Context-based approval policies
- Enterprise-scale

### Cons
- More resources to manage
- Higher learning curve
- Requires BindingRouter integration (not yet in handler)

---

## Migration Path

### Start Simple (DaemonConfig)
```bash
# Day 1: Simple bot
aofctl serve --config daemon.yaml --agents-dir ./agents
```

### Add Workflows (AgentFlow)
```bash
# Week 2: Add multi-step flows
aofctl serve --config daemon.yaml --agents-dir ./agents --flows-dir ./flows
```

### Go Enterprise (FlowBinding)
```bash
# Month 2: Full composable architecture
aofctl serve --config daemon.yaml \
  --agents-dir ./agents \
  --flows-dir ./flows \
  --triggers-dir ./triggers \
  --contexts-dir ./contexts \
  --bindings-dir ./bindings
```

---

## Summary

| Approach | Setup Time | Flexibility | Best For |
|----------|------------|-------------|----------|
| **DaemonConfig** | 5 min | Low | MVPs, simple bots |
| **AgentFlow** | 30 min | Medium | Workflows, parallel execution |
| **FlowBinding** | 1 hour | High | Enterprise, multi-tenant |

**Recommendation:** Start with DaemonConfig. Add AgentFlow when you need workflows. Use FlowBinding when you need multi-tenant or strict context separation.

---

## See Also

- [DaemonConfig Reference](../reference/daemon-config.md)
- [Agent Spec Reference](../reference/agent-spec.md)
- [AgentFlow Spec Reference](../reference/agentflow-spec.md)
- [Fleet Spec Reference](../reference/fleet-spec.md)
- [Trigger Reference](../reference/trigger-spec.md)
- [Context Reference](../reference/context-spec.md)
- [FlowBinding Reference](../reference/flowbinding-spec.md)
