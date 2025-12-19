# Fleet Switching Guide

Switch between different fleets via the `/fleet` command.

## Quick Start

```
/fleet              # Show available fleets (inline buttons)
/fleet devops       # Switch to DevOps fleet
/fleet info         # Show current fleet details
```

## Available Fleets

| Fleet | Agents | Purpose |
|-------|--------|---------|
| **DevOps** | k8s + docker + git + prometheus | Full-stack DevOps |
| **Kubernetes** | k8s + prometheus + loki | K8s cluster operations |
| **AWS** | aws + terraform | AWS cloud infrastructure |
| **Database** | postgres + redis | Database operations |
| **RCA** | collectors + multi-model analysts | Root cause analysis (tiered mode) |
| **RCA Deep** | single investigator + tools | Deep investigation (iterative planning) |

## User Experience

```
User: /fleet

Bot: Select Fleet
     Current: DevOps

     [DevOps]     [Kubernetes]
     [AWS]        [Database]
     [RCA]

User: *taps Kubernetes*

Bot: Switched to Kubernetes

     Agents: k8s, prometheus, loki

     Kubernetes cluster operations with monitoring.

User: why are pods crashing?

Bot: [Fleet routes to k8s-agent, which analyzes...]
```

## How Fleets Work

Fleets compose single-purpose agents and route requests to the right specialist:

```
User: "check pod logs"
         │
         ▼
    ┌─────────────┐
    │ DevOps Fleet│
    │(coordinator)│
    └──────┬──────┘
           │ routes to k8s-agent
           ▼
    ┌─────────────┐
    │  k8s-agent  │ ← focuses on kubectl
    └─────────────┘
           │
           ▼
    Response with logs
```

## Adding Custom Fleets

### 1. Create Single-Purpose Agents

```yaml
# agents/library/my-agent.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: my-agent
  labels:
    library: custom
    domain: mytools
spec:
  model: google:gemini-2.5-flash
  tools:
    - mytool
  system_prompt: |
    You are a specialist for mytool.
    Focus ONLY on mytool operations.
```

### 2. Compose Agents into a Fleet

```yaml
# fleets/my-fleet.yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFleet
metadata:
  name: my-fleet
spec:
  display:
    name: "My Fleet"
    emoji: "🚀"
    description: "Custom operations"

  agents:
    - ref: library/my-agent.yaml
      role: specialist
    - ref: library/k8s-agent.yaml
      role: specialist

  coordination:
    mode: hierarchical
    distribution: skill-based
```

### 3. Run the Fleet

```bash
# CLI
aofctl run fleet fleets/my-fleet.yaml -i "do something"

# Serve with Telegram
aofctl serve --config config.yaml --fleets-dir ./fleets
```

## Fleet Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `hierarchical` | Routes to right specialist | Default for most fleets |
| `peer` | All agents work in parallel | Code review, voting |
| `tiered` | Collectors → Analysts → Synthesizer | Multi-model RCA |
| `pipeline` | Sequential processing | Data transformation |
| `swarm` | Self-organizing, load balanced | High-volume parallel |
| `deep` | Iterative planning + execution loop | Complex investigations |

### Deep Mode (Agentic Investigation)

Deep mode enables iterative, planning-based execution similar to how Claude Code works:

```yaml
coordination:
  mode: deep
  deep:
    max_iterations: 10   # Safety limit
    planning: true       # Enable planning phase
    memory: true         # Persist findings across iterations
```

**How it works:**
1. **Plan** - LLM generates investigation steps
2. **Execute** - Run each step using available tools
3. **Re-plan** - Adjust plan based on findings
4. **Synthesize** - Produce final answer with evidence

**Example:** See `examples/fleets/rca-deep-fleet.yaml`

## Platform Behavior

| Platform | Write Access |
|----------|--------------|
| CLI | Full access |
| Slack | Full access (with approval workflow) |
| Telegram | Read-only |
| WhatsApp | Read-only |

See [Safety Layer Guide](safety-layer.md) for details.

## See Also

- [Core Concepts](../introduction/concepts.md) - Agent → Fleet → Flow model
- [Fleets Deep Dive](../concepts/fleets.md) - Coordination modes and consensus
- [Fleet Reference](../reference/fleet-spec.md) - Fleet specification
