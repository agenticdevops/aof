# Fleet Examples

Fleets are **compositions of multiple agents** working together on complex tasks. This directory contains working examples for all coordination modes.

## Quick Start

```bash
# Set your API key
export GOOGLE_API_KEY=AIza...

# Run any fleet
aofctl run fleet examples/fleets/<fleet-name>.yaml --input "your task"
```

## Examples by Coordination Mode

### Peer Mode (Parallel + Consensus/Aggregation)

| Fleet | Description | Aggregation |
|-------|-------------|-------------|
| `simple-test-fleet.yaml` | Basic peer coordination test | Consensus |
| `simple-team.yaml` | Simple team coordination | Consensus |
| **`../quickstart/code-review-fleet.yaml`** | Security + Quality reviewers | **Merge (recommended)** |

**Try it:**
```bash
# Peer + Aggregation (collects ALL findings)
aofctl run fleet examples/quickstart/code-review-fleet.yaml \
  --input "Review: function login(u,p) { return db.query('SELECT * FROM users WHERE name=' + u); }"
```

### Hierarchical Mode (Manager + Workers)

| Fleet | Description |
|-------|-------------|
| `incident-response-team.yaml` | Coordinator + specialists |
| `devops-fleet.yaml` | DevOps manager + workers |
| `k8s-fleet.yaml` | K8s manager + specialists |
| `aws-fleet.yaml` | AWS manager + workers |
| `database-fleet.yaml` | DB manager + specialists |

**Try it:**
```bash
aofctl run fleet examples/fleets/incident-response-team.yaml \
  --input "Investigate why API is returning 500 errors"
```

### Pipeline Mode (Sequential Handoff)

| Fleet | Description |
|-------|-------------|
| `data-pipeline-team.yaml` | Collect → Analyze → Report |
| `dockerizer-team.yaml` | Analyze → Dockerfile → Build |

**Try it:**
```bash
aofctl run fleet examples/fleets/data-pipeline-team.yaml \
  --input "Process this data: {users: 100, errors: 5}"
```

### Tiered Mode (Multi-Tier + Consensus)

| Fleet | Description |
|-------|-------------|
| `multi-model-rca-fleet.yaml` | Tier 1: Collectors → Tier 2: Reasoners |
| `rca-fleet.yaml` | Multi-tier RCA |
| `mock-tiered-fleet.yaml` | Test tiered execution |
| `code-review-team.yaml` | Tiered code review |

**Try it:**
```bash
aofctl run fleet examples/fleets/multi-model-rca-fleet.yaml \
  --input "Why are pods crashing in production?"
```

### Deep Mode (Iterative Planning + Execution)

| Fleet | Description |
|-------|-------------|
| `deep-analysis-fleet.yaml` | Iterative investigation loop |
| `rca-deep-fleet.yaml` | Deep RCA with re-planning |

**Try it:**
```bash
aofctl run fleet examples/fleets/rca-deep-fleet.yaml \
  --input "Find root cause of memory leak"
```

### Swarm Mode (Self-Organizing)

| Fleet | Description |
|-------|-------------|
| `high-volume-fleet.yaml` | Load-balanced parallel processing |

**Try it:**
```bash
aofctl run fleet examples/fleets/high-volume-fleet.yaml \
  --input "Process batch of 100 items"
```

## Choosing the Right Mode

```
Need multiple perspectives on SAME task?
  └── Yes: Peer + Consensus
  └── No: Different specialists providing complementary info?
        └── Yes: Peer + Aggregation
        └── No: Sequential data transformation?
              └── Yes: Pipeline
              └── No: Manager coordinates workers?
                    └── Yes: Hierarchical
                    └── No: Iterative investigation?
                          └── Yes: Deep
                          └── No: High-volume parallel?
                                └── Yes: Swarm
                                └── No: Multi-tier RCA?
                                      └── Yes: Tiered
```

## Best Practices

1. **Use Aggregation for Specialists** - When agents provide different info, use `aggregation: merge`
2. **Use Consensus for Validation** - When agents compete on same task, use `consensus`
3. **Keep Instructions Focused** - Each agent should have a clear, specific role
4. **Start Simple** - Begin with `simple-test-fleet.yaml` to understand the pattern
