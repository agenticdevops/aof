# Core Concepts

AOF uses a simple, composable model: **Agents** are single-purpose building blocks, **Fleets** compose agents into teams, **Flows** define multi-step workflows, and **Triggers** route messages to handlers.

```
┌─────────────────────────────────────────────────────────────┐
│                    AOF Building Blocks                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   AGENT          FLEET           FLOW          TRIGGER      │
│   ┌────┐       ┌─────────┐    ┌──────────┐   ┌──────────┐  │
│   │ 🔧 │       │ 🔧  🔧  │    │ Node     │   │ Platform │  │
│   └────┘       │ 🔧  🔧  │    │   ↓      │   │    ↓     │  │
│   Single       └─────────┘    │ Node     │   │ Commands │  │
│   Purpose      Composition    │   ↓      │   │    ↓     │  │
│                               │ End      │   │ Handler  │  │
│                               └──────────┘   └──────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## The Mental Model

| Concept | What It Is | Example |
|---------|------------|---------|
| **Agent** | Single-purpose specialist | `kubectl-agent`, `prometheus-agent` |
| **Fleet** | Team of agents for a purpose | `devops-fleet`, `rca-fleet` |
| **Flow** | Multi-step workflow with nodes | `deploy-flow`, `incident-flow` |
| **Trigger** | Platform + command routing | `slack-prod`, `telegram-oncall` |

**One way to do it**: Build single-purpose agents, compose them into fleets, define workflows as flows, connect to chat platforms via triggers.

---

## 1. Agent

An **Agent** is a single-purpose AI specialist. It does one thing well.

### Key Principle: Single Responsibility

```yaml
# ✅ GOOD: Single-purpose agent
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
    When asked about non-K8s topics, indicate another specialist should handle it.
```

```yaml
# ❌ BAD: "Super agent" with too many responsibilities
spec:
  tools:
    - kubectl
    - docker
    - terraform
    - git
    - aws
  # This agent tries to do everything - hard to maintain, not reusable
```

### Why Single-Purpose Agents?

| Benefit | Description |
|---------|-------------|
| **Reusable** | Same `postgres-agent` works in `database-fleet` and `rca-fleet` |
| **Focused** | Better prompts, better results |
| **Composable** | Mix and match to build any fleet |
| **Testable** | Test each agent in isolation |
| **Multi-model** | Different agents can use different LLMs |

### Agent Library

AOF provides a library of pre-built single-purpose agents:

| Agent | Tools | Purpose |
|-------|-------|---------|
| `k8s-agent` | kubectl, helm | Kubernetes operations |
| `docker-agent` | docker | Container management |
| `git-agent` | git | Version control |
| `aws-agent` | aws | AWS cloud operations |
| `terraform-agent` | terraform | Infrastructure as Code |
| `prometheus-agent` | prometheus_query | Metrics analysis |
| `loki-agent` | loki_query | Log analysis |
| `postgres-agent` | psql | PostgreSQL operations |
| `redis-agent` | redis-cli | Redis operations |

### Agent YAML Structure

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-agent
  labels:
    library: core
    domain: kubernetes
spec:
  # LLM model
  model: google:gemini-2.5-flash

  # Tools this agent can use (focused set)
  tools:
    - kubectl
    - helm

  # Focused system prompt
  system_prompt: |
    You are a Kubernetes operations specialist.
    Focus ONLY on Kubernetes cluster management.

  # Optional: Memory configuration
  memory:
    type: InMemory
    config:
      max_messages: 20
```

---

## 2. Fleet

A **Fleet** is a composition of agents that work together for a specific purpose.

### Key Principle: Composition Over Configuration

Instead of one "super agent" with 20 tools, compose single-purpose agents:

```yaml
# Fleet = Composition of single-purpose agents
apiVersion: aof.dev/v1alpha1
kind: AgentFleet
metadata:
  name: devops-fleet
spec:
  display:
    name: "DevOps"
    emoji: "🔧"
    description: "Full-stack DevOps operations"

  # Compose agents from the library
  agents:
    - ref: library/k8s-agent.yaml
      role: specialist
    - ref: library/docker-agent.yaml
      role: specialist
    - ref: library/git-agent.yaml
      role: specialist
    - ref: library/prometheus-agent.yaml
      role: specialist

  coordination:
    mode: hierarchical
    distribution: skill-based
```

### Built-in Fleets

| Fleet | Agents | Purpose |
|-------|--------|---------|
| **DevOps** | k8s + docker + git + prometheus | Full-stack DevOps |
| **Kubernetes** | k8s + prometheus + loki | K8s cluster operations |
| **AWS** | aws + terraform | AWS cloud infrastructure |
| **Database** | postgres + redis | Database operations |
| **RCA** | collectors + multi-model analysts | Root cause analysis |

### Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `hierarchical` | Routes to right specialist | Default for most fleets |
| `peer` | All agents work in parallel | Code review, voting |
| `tiered` | Collectors → Analysts → Synthesizer | Multi-model RCA |
| `pipeline` | Sequential processing | Data transformation |
| `swarm` | Self-organizing, load balanced | High-volume parallel |
| `deep` | Iterative planning + execution loop | Complex investigations |

### How Fleets Work

```
User: "Why are pods crashing?"
            │
            ▼
    ┌───────────────┐
    │  DevOps Fleet │
    │ (coordinator) │
    └───────┬───────┘
            │ routes to specialist
            ▼
    ┌───────────────┐
    │   k8s-agent   │  ← Single-purpose specialist
    └───────────────┘
            │
            ▼
    Response: "Pods crashing due to OOMKilled..."
```

### Multi-Model RCA Fleet

For critical analysis, use multiple LLM models for consensus:

```yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFleet
metadata:
  name: rca-fleet
spec:
  agents:
    # Tier 1: Data collectors (cheap, fast)
    - ref: library/k8s-agent.yaml
      tier: 1
    - ref: library/prometheus-agent.yaml
      tier: 1
    - ref: library/loki-agent.yaml
      tier: 1

    # Tier 2: Analysts (multiple models for diverse perspectives)
    - name: claude-analyst
      tier: 2
      spec:
        model: anthropic:claude-sonnet-4-20250514

    - name: gemini-analyst
      tier: 2
      spec:
        model: google:gemini-2.5-pro

  coordination:
    mode: tiered
    consensus: weighted
```

---

## 3. Flow

A **Flow** is a multi-step workflow with nodes and connections. Flows are pure workflow logic - they define *what happens* in a sequence of steps.

### Key Principle: Declarative Workflows

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
spec:
  description: "Deployment workflow with approval gate"

  nodes:
    - id: validate
      type: Agent
      config:
        agent: validator-agent

    - id: approval
      type: HumanApproval
      config:
        timeout: 300
        message: "Approve deployment to production?"

    - id: deploy
      type: Agent
      config:
        agent: k8s-agent

    - id: notify
      type: End

  connections:
    - from: start
      to: validate
    - from: validate
      to: approval
    - from: approval
      to: deploy
      condition: approved
    - from: deploy
      to: notify
```

### Flow Nodes

| Node Type | Purpose | Example |
|-----------|---------|---------|
| `Agent` | Execute single agent | k8s-agent, docker-agent |
| `Fleet` | Execute agent fleet | rca-fleet, devops-fleet |
| `HumanApproval` | Wait for human approval | Deployment gates |
| `Conditional` | Branch based on conditions | Success/failure paths |
| `End` | Terminal node | Final response |

---

## 4. Trigger

A **Trigger** defines message sources and command routing. Triggers are self-contained units that include platform configuration and command bindings.

### Key Principle: Self-Contained Routing

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

  # Route commands to handlers
  commands:
    /diagnose:
      fleet: rca-fleet
      description: "Multi-model root cause analysis"
    /deploy:
      flow: deploy-flow
      description: "Deployment workflow with approvals"
    /kubectl:
      agent: k8s-agent
      description: "Direct Kubernetes operations"

  # Fallback for natural language
  default_agent: devops
```

### Trigger Types

| Trigger | Events | Use Case |
|---------|--------|----------|
| `Telegram` | message, command | Mobile DevOps |
| `Slack` | message, app_mention, slash_command | Team chat |
| `WhatsApp` | message | Customer support |
| `PagerDuty` | incident events | Automated response |
| `HTTP` | webhook POST | Generic integrations |

### Command Binding Options

Each command routes to one target:

| Target | When to Use | Example |
|--------|-------------|---------|
| `agent` | Single-purpose task | `/kubectl → k8s-agent` |
| `fleet` | Multi-agent coordination | `/diagnose → rca-fleet` |
| `flow` | Multi-step workflow | `/deploy → deploy-flow` |

### Platform Safety

Triggers automatically apply platform-appropriate safety:

| Platform | Read Operations | Write Operations |
|----------|-----------------|------------------|
| CLI | ✅ Allowed | ✅ Allowed |
| Slack | ✅ Allowed | ✅ With approval |
| Telegram | ✅ Allowed | ❌ Blocked |
| WhatsApp | ✅ Allowed | ❌ Blocked |

---

## Putting It Together

### The Full Picture

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
│  ├─────────────┤   ┌─────────────┐                  ┌────────────┐   │
│  │ loki-agent  │──▶│ k8s-fleet   │◀────────────────│telegram    │   │
│  ├─────────────┤   └─────────────┘                  └────────────┘   │
│  │postgres     │                                                      │
│  └─────────────┘                                                      │
│                                                                        │
└──────────────────────────────────────────────────────────────────────┘
```

### CLI Usage

```bash
# Run a single agent (for testing)
aofctl run agent library/k8s-agent.yaml -i "list pods"

# Run a fleet (composed agents)
aofctl run fleet examples/fleets/devops-fleet.yaml -i "check cluster health"

# Start the daemon (connects triggers to handlers)
aofctl serve --config examples/config/daemon.yaml
```

### Chat Usage

Via Slack or Telegram, use slash commands defined in your triggers:

```
/diagnose pod is crashing     # → Routes to rca-fleet
/deploy v2.1.0                # → Routes to deploy-flow
/kubectl get pods             # → Routes to k8s-agent

# Or just chat naturally with the default agent:
"what's the memory usage in production?"
```

---

## When to Use What

| Scenario | Use This | Why |
|----------|----------|-----|
| Test a single tool | **Agent** | Quick, focused testing |
| Kubernetes operations | **Fleet** (k8s-fleet) | K8s + monitoring agents |
| Full DevOps | **Fleet** (devops-fleet) | K8s + Docker + Git + monitoring |
| Root cause analysis | **Fleet** (rca-fleet) | Multi-model consensus |
| Multi-step workflow | **Flow** | Approval gates, pipelines |
| Chat platform bot | **Trigger** | Slack, Telegram with command routing |

---

## Summary

| Concept | Purpose | Remember |
|---------|---------|----------|
| **Agent** | Single-purpose building block | One tool domain, reusable |
| **Fleet** | Composition of agents | Teams of specialists |
| **Flow** | Multi-step workflow | Nodes, connections, approval gates |
| **Trigger** | Platform + command routing | Maps commands to handlers |

**The simple rule**: Build focused agents → Compose into fleets → Define workflows as flows → Connect to chat via triggers.

---

## Next Steps

- **[Telegram Quickstart](../guides/quickstart-telegram.md)** - Get a bot running in 5 minutes
- **[Fleet Reference](../reference/fleet-spec.md)** - Complete fleet specification
- **[Agent Reference](../reference/agent-spec.md)** - Agent YAML specification
- **[Trigger Reference](../reference/trigger-spec.md)** - Trigger YAML specification
