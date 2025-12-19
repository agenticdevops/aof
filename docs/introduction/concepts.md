# Core Concepts

AOF uses a simple, composable model: **Agents** are single-purpose building blocks, **Fleets** compose agents into teams, and **Flows** handle event-driven routing.

```
┌─────────────────────────────────────────────────────────────┐
│                    AOF Building Blocks                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   AGENT          FLEET              FLOW                    │
│   ┌────┐       ┌─────────┐       ┌──────────┐              │
│   │ 🔧 │       │ 🔧  🔧  │       │ Trigger  │              │
│   └────┘       │ 🔧  🔧  │       │    ↓     │              │
│   Single       └─────────┘       │  Agent   │              │
│   Purpose      Composition       │    ↓     │              │
│                                  │ Response │              │
│                                  └──────────┘              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## The Mental Model

| Concept | What It Is | Example |
|---------|------------|---------|
| **Agent** | Single-purpose specialist | `kubectl-agent`, `prometheus-agent` |
| **Fleet** | Team of agents for a purpose | `devops-fleet`, `rca-fleet` |
| **Flow** | Event routing to agents/fleets | Telegram → Fleet → Response |

**One way to do it**: Build single-purpose agents, compose them into fleets, connect fleets to chat platforms via flows.

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

A **Flow** connects triggers (Telegram, Slack, webhooks) to agents or fleets.

### Key Principle: Event-Driven Routing

```yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFlow
metadata:
  name: telegram-ops
spec:
  trigger:
    type: Telegram
    config:
      token: ${TELEGRAM_BOT_TOKEN}

  # Route to a Fleet (not individual agent)
  nodes:
    - id: agent
      type: Fleet
      config:
        fleet: devops-fleet

    - id: respond
      type: Telegram
      config:
        message: ${agent.output}
```

### Flow Triggers

| Trigger | Events | Use Case |
|---------|--------|----------|
| `Telegram` | message, command | Mobile DevOps |
| `Slack` | message, app_mention | Team chat |
| `WhatsApp` | message | Customer support |
| `HTTP` | webhook POST | Integrations |
| `Schedule` | cron | Scheduled jobs |

### Platform Safety

Flows automatically apply platform-appropriate safety:

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
┌─────────────────────────────────────────────────────────────┐
│                    Complete Architecture                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  AGENT LIBRARY              FLEETS               FLOWS      │
│  (building blocks)          (compositions)       (routing)  │
│                                                              │
│  ┌─────────────┐          ┌─────────────┐     ┌──────────┐ │
│  │ k8s-agent   │────┬────▶│devops-fleet │◀────│ Telegram │ │
│  ├─────────────┤    │     └─────────────┘     └──────────┘ │
│  │docker-agent │────┤                                       │
│  ├─────────────┤    │     ┌─────────────┐     ┌──────────┐ │
│  │ git-agent   │────┘     │  rca-fleet  │◀────│  Slack   │ │
│  ├─────────────┤          └─────────────┘     └──────────┘ │
│  │prometheus   │────┬────▶│ k8s-fleet   │                   │
│  ├─────────────┤    │     └─────────────┘                   │
│  │ loki-agent  │────┘                                       │
│  ├─────────────┤          ┌─────────────┐                   │
│  │postgres     │─────────▶│database-fleet│                   │
│  ├─────────────┤          └─────────────┘                   │
│  │ redis       │                                            │
│  └─────────────┘                                            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### CLI Usage

```bash
# Run a single agent (for testing)
aofctl run agent library/k8s-agent.yaml -i "list pods"

# Run a fleet (composed agents)
aofctl run fleet examples/fleets/devops-fleet.yaml -i "check cluster health"

# Serve flows (connect to chat platforms)
aofctl serve --flows-dir ./flows --agents-dir ./agents
```

### Telegram Usage

```
/fleet               # List available fleets
/fleet devops        # Switch to DevOps fleet
/fleet info          # Show current fleet

# Then just chat naturally:
"list pods in production"
"show deployment status"
"what's the memory usage?"
```

---

## When to Use What

| Scenario | Use This | Why |
|----------|----------|-----|
| Test a single tool | **Agent** | Quick, focused testing |
| Kubernetes operations | **Fleet** (k8s-fleet) | K8s + monitoring agents |
| Full DevOps | **Fleet** (devops-fleet) | K8s + Docker + Git + monitoring |
| Database work | **Fleet** (database-fleet) | Postgres + Redis agents |
| Root cause analysis | **Fleet** (rca-fleet) | Multi-model consensus |
| Telegram/Slack bot | **Flow** | Connect fleet to chat platform |

---

## Summary

| Concept | Purpose | Remember |
|---------|---------|----------|
| **Agent** | Single-purpose building block | One tool domain, reusable |
| **Fleet** | Composition of agents | Teams of specialists |
| **Flow** | Event-driven routing | Connects triggers to fleets |

**The simple rule**: Build focused agents → Compose into fleets → Connect via flows.

---

## Next Steps

- **[Telegram Quickstart](../guides/quickstart-telegram.md)** - Get a bot running in 5 minutes
- **[Fleet Reference](../reference/fleet-spec.md)** - Complete fleet specification
- **[Agent Library](../../examples/agents/library/)** - Pre-built agents
- **[Example Fleets](../../examples/fleets/)** - Ready-to-use fleet compositions
