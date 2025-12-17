# Fleet Examples

This directory contains AgentFleet examples for multi-agent coordination.

## Directory Structure

```
fleets/
├── mock/                           # Mock/demo fleets (no infrastructure required)
│   ├── README.md
│   └── multi-model-rca-mock.yaml   # Simulated RCA demo
│
├── real/                           # Production fleets (real infrastructure)
│   ├── README.md
│   └── multi-model-rca-real.yaml   # Real Prometheus/Loki/K8s RCA
│
├── code-review-team.yaml           # Code review with consensus
├── incident-response-team.yaml     # Incident response coordination
├── data-pipeline-team.yaml         # Data transformation pipeline
├── k8s-rca-team.yaml               # Kubernetes-specific RCA
├── application-rca-team.yaml       # Application-level RCA
├── database-rca-team.yaml          # Database-specific RCA
└── ...
```

## Quick Start

### Option 1: Mock Fleet (No Infrastructure)

Perfect for learning and demos:

```bash
# Only needs an API key
export GOOGLE_API_KEY=your-key

aofctl run fleet examples/fleets/mock/multi-model-rca-mock.yaml \
  --input "Investigate: API returning 500 errors"
```

### Option 2: Real Fleet (With Infrastructure)

For actual incident response:

```bash
# Needs Prometheus, Loki, Kubernetes
export GOOGLE_API_KEY=your-key

aofctl run fleet examples/fleets/real/multi-model-rca-real.yaml \
  --input "Investigate: High memory in monitoring namespace"
```

## Fleet Categories

### 🔍 Root Cause Analysis (RCA)

| Fleet | Mode | Description |
|-------|------|-------------|
| `mock/multi-model-rca-mock.yaml` | Tiered | Multi-model RCA with simulated data |
| `real/multi-model-rca-real.yaml` | Tiered | Multi-model RCA with real Prometheus/Loki |
| `k8s-rca-team.yaml` | Hierarchical | Kubernetes-specific RCA |
| `application-rca-team.yaml` | Hierarchical | Application-level RCA |
| `database-rca-team.yaml` | Hierarchical | Database-specific RCA |

### 👀 Code Review

| Fleet | Mode | Description |
|-------|------|-------------|
| `code-review-team.yaml` | Peer | Multi-perspective code review |
| `code-review-fleet.yaml` | Peer | Security + performance + quality |

### 🚨 Incident Response

| Fleet | Mode | Description |
|-------|------|-------------|
| `incident-response-team.yaml` | Hierarchical | Full incident response workflow |
| `sre-oncall-fleet.yaml` | Hierarchical | SRE on-call automation |

### 🔧 DevOps

| Fleet | Mode | Description |
|-------|------|-------------|
| `dockerizer-team.yaml` | Pipeline | Dockerfile generation pipeline |
| `data-pipeline-team.yaml` | Pipeline | Data transformation workflow |

## Coordination Modes

| Mode | Best For | Example |
|------|----------|---------|
| **Peer** | Consensus, voting | Code review (3 reviewers vote) |
| **Hierarchical** | Complex orchestration | Incident response (manager delegates) |
| **Pipeline** | Sequential processing | Data transformation (A→B→C) |
| **Tiered** | Multi-model consensus | RCA (collectors→reasoning→synthesis) |
| **Swarm** | High-volume tasks | Log analysis at scale |

## Consensus Algorithms

| Algorithm | Use Case | Configuration |
|-----------|----------|---------------|
| `majority` | General consensus | `min_votes: 2` |
| `unanimous` | Critical decisions | - |
| `weighted` | Mixed expertise | `weight: 1.5` per agent |
| `first_wins` | Speed priority | - |
| `human_review` | High-stakes | `min_confidence: 0.9` |

## Testing Fleets

### Validate Configuration

```bash
aofctl validate fleet examples/fleets/code-review-team.yaml
```

### Dry Run

```bash
aofctl run fleet examples/fleets/mock/multi-model-rca-mock.yaml \
  --input "Test input" \
  --dry-run
```

### Verbose Execution

```bash
aofctl run fleet examples/fleets/real/multi-model-rca-real.yaml \
  --input "Investigate: Issue" \
  --verbose
```

## See Also

- **Fleet Concepts**: `docs/concepts/fleets.md`
- **Fleet Spec Reference**: `docs/reference/fleet-spec.md`
- **Multi-Model RCA Tutorial**: `docs/tutorials/multi-model-rca.md`
- **Architecture Guide**: `docs/architecture/multi-model-consensus.md`
