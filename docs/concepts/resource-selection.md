# Choosing the Right Resource

AOF provides a composable architecture with four core concepts. This guide helps you choose the right approach for your use case.

## Quick Decision Tree

```
What do you need to do?
├─ Single-purpose task → Use Agent
├─ Multi-agent coordination → Use Fleet
├─ Multi-step workflow → Use AgentFlow (Flow)
└─ Connect chat platforms → Use Trigger (with command bindings)
```

## The Four Core Concepts

| Concept | What It Is | Example |
|---------|------------|---------|
| **Agent** | Single-purpose specialist | `k8s-agent`, `prometheus-agent` |
| **Fleet** | Team of agents for a purpose | `devops-fleet`, `rca-fleet` |
| **Flow** | Multi-step workflow with nodes | `deploy-flow`, `incident-flow` |
| **Trigger** | Platform + command routing | `slack-prod`, `telegram-oncall` |

**One way to do it**: Build focused agents → Compose into fleets → Define workflows as flows → Connect to chat via triggers.

---

## Option 1: Agent (Single-Purpose)

**Best for:** Focused tasks, testing, building blocks

### When to Use
- Single tool domain (kubectl, docker, git)
- Reusable building block
- Testing a specific capability
- Simple Q&A interactions

### Example
```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-agent
spec:
  model: google:gemini-2.5-flash
  tools:
    - kubectl
    - helm
  system_prompt: |
    You are a Kubernetes specialist.
    Focus ONLY on Kubernetes operations.
```

### CLI Usage
```bash
aofctl run agent library/k8s-agent.yaml -i "list pods in production"
```

---

## Option 2: Fleet (Multi-Agent Team)

**Best for:** Complex tasks requiring multiple specialists

### When to Use
- Task spans multiple tool domains
- Need multi-model consensus (RCA)
- Complex investigation requiring different perspectives
- Parallel data collection and analysis

### Example
```yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFleet
metadata:
  name: devops-fleet
spec:
  display:
    name: "DevOps"
    emoji: "🔧"
  agents:
    - ref: library/k8s-agent.yaml
    - ref: library/docker-agent.yaml
    - ref: library/prometheus-agent.yaml
  coordination:
    mode: hierarchical
```

### CLI Usage
```bash
aofctl run fleet fleets/devops-fleet.yaml -i "why are pods crashing?"
```

---

## Option 3: AgentFlow (Workflow)

**Best for:** Multi-step processes with approval gates

### When to Use
- Sequential steps (validate → approve → deploy)
- Conditional branching based on results
- Human approval gates
- Complex orchestration

### Example
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
spec:
  description: "Deployment with approval"
  nodes:
    - id: validate
      type: Agent
      config:
        agent: validator-agent
    - id: approval
      type: HumanApproval
      config:
        message: "Approve deployment?"
    - id: deploy
      type: Agent
      config:
        agent: k8s-agent
  connections:
    - from: start
      to: validate
    - from: validate
      to: approval
    - from: approval
      to: deploy
      condition: approved
```

### CLI Usage
```bash
aofctl run flow flows/deploy-flow.yaml -i "deploy v2.1.0"
```

---

## Option 4: Trigger (Platform Routing)

**Best for:** Connecting chat platforms to handlers

### When to Use
- Chat bots (Slack, Telegram, Discord)
- Webhook integrations (PagerDuty, GitHub)
- Command routing (/kubectl, /diagnose, /deploy)
- Platform-specific configuration

### Example
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}

  commands:
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis"
    /deploy:
      flow: deploy-flow
      description: "Deployment workflow"

  default_agent: devops
```

### Command Targets

Each command routes to exactly one target:

| Target | When to Use | Example |
|--------|-------------|---------|
| `agent` | Single-purpose task | `/kubectl → k8s-agent` |
| `fleet` | Multi-agent coordination | `/diagnose → rca-fleet` |
| `flow` | Multi-step workflow | `/deploy → deploy-flow` |

---

## Comparison Table

| Feature | Agent | Fleet | Flow | Trigger |
|---------|-------|-------|------|---------|
| **Purpose** | Single task | Multi-agent | Workflow | Platform routing |
| **Complexity** | Simple | Medium | Medium | Simple |
| **Reusability** | High | High | Medium | Per-platform |
| **Multi-step** | No | No | Yes | Routes to others |
| **Approval gates** | No | No | Yes | Via flow |
| **Multi-model** | No | Yes | Via fleet | Via fleet |

---

## Architecture Summary

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Complete Architecture                           │
├──────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  AGENTS              FLEETS              FLOWS        TRIGGERS        │
│  (building blocks)   (compositions)      (workflows)  (routing)       │
│                                                                        │
│  ┌─────────────┐   ┌─────────────┐   ┌───────────┐  ┌────────────┐   │
│  │ k8s-agent   │──▶│devops-fleet │   │deploy-flow│◀─│slack-prod  │   │
│  ├─────────────┤   └─────────────┘   └───────────┘  ├────────────┤   │
│  │docker-agent │                                    │            │   │
│  ├─────────────┤   ┌─────────────┐                  │ /deploy    │──▶│
│  │ git-agent   │──▶│  rca-fleet  │◀────────────────│ /diagnose  │   │
│  ├─────────────┤   └─────────────┘                  │ /kubectl   │   │
│  │prometheus   │                                    └────────────┘   │
│  └─────────────┘                                                      │
│                                                                        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Migration Path

### Start Simple
```bash
# Test a single agent
aofctl run agent library/k8s-agent.yaml -i "list pods"
```

### Add Fleet Composition
```bash
# Use a fleet for complex tasks
aofctl run fleet fleets/devops-fleet.yaml -i "diagnose pod crash"
```

### Add Workflows
```bash
# Use flows for approval workflows
aofctl run flow flows/deploy-flow.yaml -i "deploy v2.1.0"
```

### Connect to Chat
```bash
# Start daemon with triggers
aofctl serve --config daemon.yaml
```

---

## Summary

| Approach | Setup Time | Use Case |
|----------|------------|----------|
| **Agent** | 2 min | Single tool testing |
| **Fleet** | 10 min | Multi-agent tasks |
| **Flow** | 15 min | Approval workflows |
| **Trigger** | 5 min | Chat platform bots |

**Recommendation:** Build focused agents → Compose into fleets → Define workflows as flows → Connect to chat via triggers.

---

## See Also

- [Core Concepts](../introduction/concepts.md) - Mental model
- [Agent Spec Reference](../reference/agent-spec.md)
- [Fleet Spec Reference](../reference/fleet-spec.md)
- [AgentFlow Spec Reference](../reference/agentflow-spec.md)
- [Trigger Reference](../reference/trigger-spec.md)
- [DaemonConfig Reference](../reference/daemon-config.md)
