# Multi-Model Consensus Architecture

## Overview

Multi-Model Consensus is a core AOF capability that enables multiple AI models to collaboratively analyze problems and reach consensus-based conclusions. This architecture is particularly powerful for Root Cause Analysis (RCA), code review, and any scenario where diverse perspectives improve accuracy.

## The Problem: Single-Model Limitations

```
┌─────────────────────────────────────────────────────────────────┐
│                    SINGLE MODEL ANALYSIS                        │
│                                                                 │
│                      ┌──────────────┐                          │
│                      │              │                          │
│     Input ──────────▶│  Single LLM  │──────────▶ Output        │
│                      │              │                          │
│                      └──────────────┘                          │
│                                                                 │
│  Problems:                                                      │
│  ✗ Single perspective - one model's biases                     │
│  ✗ No cross-validation - hallucinations go unchecked           │
│  ✗ Blind spots - may miss domain-specific issues               │
│  ✗ Single point of failure - if wrong, entirely wrong          │
│  ✗ No confidence measure - hard to know when to trust          │
└─────────────────────────────────────────────────────────────────┘
```

## The Solution: Multi-Model Consensus

```
┌─────────────────────────────────────────────────────────────────┐
│                  MULTI-MODEL CONSENSUS                          │
│                                                                 │
│                    ┌──────────────┐                            │
│               ┌───▶│   Claude     │───┐                        │
│               │    │  (wt: 1.5)   │   │                        │
│               │    └──────────────┘   │                        │
│               │                       │                        │
│               │    ┌──────────────┐   │    ┌──────────────┐    │
│     Input ────┼───▶│   Gemini     │───┼───▶│  Consensus   │──▶ Output
│               │    │  (wt: 1.0)   │   │    │   Engine     │    │
│               │    └──────────────┘   │    └──────────────┘    │
│               │                       │                        │
│               │    ┌──────────────┐   │                        │
│               └───▶│   GPT-4      │───┘                        │
│                    │  (wt: 1.0)   │                            │
│                    └──────────────┘                            │
│                                                                 │
│  Benefits:                                                      │
│  ✓ Multiple perspectives - diverse training data               │
│  ✓ Cross-validation - agreements have high confidence          │
│  ✓ Blind spot coverage - models complement each other          │
│  ✓ Fault tolerance - one wrong model is outvoted               │
│  ✓ Confidence scoring - disagreement flags uncertainty         │
└─────────────────────────────────────────────────────────────────┘
```

## Architecture Components

### 1. Tiered Execution Model

AOF implements a tiered execution model that optimizes both cost and quality:

```
┌─────────────────────────────────────────────────────────────────────┐
│                       TIERED EXECUTION                               │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ TIER 1: Data Collection                                      │    │
│  │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐             │    │
│  │ │  Loki   │ │ Prom    │ │  K8s    │ │  Git    │             │    │
│  │ │  Logs   │ │ Metrics │ │ State   │ │ Changes │             │    │
│  │ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘             │    │
│  │      │           │           │           │                   │    │
│  │ Model: Gemini Flash (~$0.075/1M tokens)                      │    │
│  │ Consensus: first_wins (speed optimized)                      │    │
│  └──────┼───────────┼───────────┼───────────┼───────────────────┘    │
│         └───────────┴───────────┴───────────┘                        │
│                           │                                          │
│                           ▼ collected data                           │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ TIER 2: Reasoning & Analysis                                 │    │
│  │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │    │
│  │ │   Claude    │ │   Gemini    │ │   GPT-4     │             │    │
│  │ │  Sonnet     │ │    Pro      │ │     o       │             │    │
│  │ │ (wt: 1.5)   │ │ (wt: 1.0)   │ │ (wt: 1.0)   │             │    │
│  │ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘             │    │
│  │        │               │               │                     │    │
│  │ Models: Premium reasoning models (~$3-5/1M tokens)           │    │
│  │ Consensus: weighted (quality optimized)                      │    │
│  └────────┼───────────────┼───────────────┼─────────────────────┘    │
│           └───────────────┼───────────────┘                          │
│                           │                                          │
│                           ▼ weighted consensus                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ TIER 3: Synthesis                                            │    │
│  │ ┌─────────────────────────────────────────────────────┐     │    │
│  │ │                  RCA Coordinator                     │     │    │
│  │ │                                                      │     │    │
│  │ │  • Synthesizes multi-model analyses                  │     │    │
│  │ │  • Highlights agreements (high confidence)           │     │    │
│  │ │  • Flags disagreements (needs review)                │     │    │
│  │ │  • Produces actionable report                        │     │    │
│  │ └─────────────────────────────────────────────────────┘     │    │
│  │                                                              │    │
│  │ Role: manager (final aggregation: manager_synthesis)         │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                           │                                          │
│                           ▼                                          │
│                    Final RCA Report                                  │
│                    (with confidence scores)                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 2. Consensus Engine

The consensus engine supports five algorithms, each optimized for different scenarios:

#### Algorithm Selection Guide

| Algorithm | How It Works | When to Use | Configuration |
|-----------|--------------|-------------|---------------|
| **Majority** | >50% must agree | General analysis, code review | `min_votes: 2` |
| **Unanimous** | 100% must agree | Critical decisions, security | - |
| **Weighted** | Votes × agent weight | Mixed expertise levels | `weights: {senior: 2.0}` |
| **FirstWins** | First response wins | Time-critical, data collection | - |
| **HumanReview** | Always flag for human | High-stakes decisions | `min_confidence: 0.9` |

#### Weighted Consensus Algorithm

```
Weighted Consensus Formula:

For each unique result R:
  score(R) = Σ weight(agent) for all agents that returned R

Winner = R with highest score
Confidence = score(winner) / total_weight

Example:
  Claude (wt: 1.5) → "Database timeout"
  Gemini (wt: 1.0) → "Database timeout"
  GPT-4  (wt: 1.0) → "Network issue"

  Score("Database timeout") = 1.5 + 1.0 = 2.5
  Score("Network issue")    = 1.0
  Total weight              = 3.5

  Winner: "Database timeout"
  Confidence: 2.5 / 3.5 = 0.71 (71%)
```

### 3. Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DATA FLOW                                     │
│                                                                      │
│  User Input                                                          │
│      │                                                               │
│      ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Fleet Coordinator                                             │   │
│  │                                                               │   │
│  │  1. Parse fleet configuration                                 │   │
│  │  2. Group agents by tier                                      │   │
│  │  3. Initialize shared memory                                  │   │
│  │  4. Begin tiered execution                                    │   │
│  └──────────────────────────────────────────────────────────────┘   │
│      │                                                               │
│      ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Tier 1 Execution (Parallel)                                   │   │
│  │                                                               │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐             │   │
│  │  │ Agent 1 │ │ Agent 2 │ │ Agent 3 │ │ Agent 4 │             │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘             │   │
│  │       │           │           │           │                   │   │
│  │       └───────────┴───────────┴───────────┘                   │   │
│  │                       │                                       │   │
│  │                       ▼                                       │   │
│  │            ┌─────────────────────┐                           │   │
│  │            │ Tier 1 Consensus    │                           │   │
│  │            │ (algorithm: first)  │                           │   │
│  │            └─────────────────────┘                           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│      │                                                               │
│      │ pass_all_results: true                                        │
│      ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Tier 2 Execution (Parallel)                                   │   │
│  │                                                               │   │
│  │  Input: Combined Tier 1 results                               │   │
│  │                                                               │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │   │
│  │  │   Claude    │ │   Gemini    │ │   GPT-4     │             │   │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘             │   │
│  │         │               │               │                     │   │
│  │         └───────────────┼───────────────┘                     │   │
│  │                         ▼                                     │   │
│  │            ┌─────────────────────┐                           │   │
│  │            │ Tier 2 Consensus    │                           │   │
│  │            │ (algorithm: weight) │                           │   │
│  │            │ (min_votes: 2)      │                           │   │
│  │            └─────────────────────┘                           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│      │                                                               │
│      │ final_aggregation: manager_synthesis                          │
│      ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Tier 3 Execution (Manager)                                    │   │
│  │                                                               │   │
│  │  Input: All Tier 2 analyses + consensus result                │   │
│  │                                                               │   │
│  │  ┌─────────────────────────────────────────────────────┐     │   │
│  │  │              RCA Coordinator                         │     │   │
│  │  │                                                      │     │   │
│  │  │  • Reviews all Tier 2 outputs                        │     │   │
│  │  │  • Builds agreement matrix                           │     │   │
│  │  │  • Synthesizes final report                          │     │   │
│  │  └─────────────────────────────────────────────────────┘     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│      │                                                               │
│      ▼                                                               │
│  Final Output with Confidence Scores                                 │
└─────────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### Fleet Configuration

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: multi-model-rca
spec:
  agents:
    # Tier 1: Data collectors (cheap, fast)
    - name: loki-collector
      config: agents/observability/loki-collector.yaml
      tier: 1

    # Tier 2: Reasoning models (diverse perspectives)
    - name: claude-analyzer
      config: agents/reasoning/claude-analyzer.yaml
      tier: 2
      weight: 1.5  # Higher confidence in Claude

    - name: gemini-analyzer
      config: agents/reasoning/gemini-analyzer.yaml
      tier: 2
      weight: 1.0

    # Tier 3: Synthesis
    - name: rca-coordinator
      config: agents/reasoning/rca-coordinator.yaml
      tier: 3
      role: manager

  coordination:
    mode: tiered
    consensus:
      algorithm: weighted
      min_votes: 2
      min_confidence: 0.6
    tiered:
      pass_all_results: true
      final_aggregation: manager_synthesis
      tier_consensus:
        "1": { algorithm: first_wins }
        "2": { algorithm: weighted, min_votes: 2 }
        "3": { algorithm: first_wins }
```

### Consensus Engine Implementation

```rust
// crates/aof-runtime/src/fleet/consensus.rs

pub struct ConsensusEngine;

impl ConsensusEngine {
    pub fn evaluate(
        results: &[AgentResult],
        config: &ConsensusConfig,
        weights: &HashMap<String, f32>,
    ) -> ConsensusResult {
        match config.algorithm {
            ConsensusAlgorithm::Majority => Self::majority(results, config),
            ConsensusAlgorithm::Unanimous => Self::unanimous(results),
            ConsensusAlgorithm::Weighted => Self::weighted(results, weights, config),
            ConsensusAlgorithm::FirstWins => Self::first_wins(results),
            ConsensusAlgorithm::HumanReview => Self::human_review(results, config),
        }
    }

    fn weighted(
        results: &[AgentResult],
        weights: &HashMap<String, f32>,
        config: &ConsensusConfig,
    ) -> ConsensusResult {
        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut total_weight = 0.0;

        for result in results {
            let weight = weights.get(&result.agent_name)
                .copied()
                .unwrap_or(1.0);
            total_weight += weight;
            *scores.entry(result.output.clone()).or_insert(0.0) += weight;
        }

        let (winner, score) = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();

        let confidence = score / total_weight;
        let reached = confidence >= config.min_confidence.unwrap_or(0.5);

        ConsensusResult {
            result: winner.clone(),
            confidence,
            votes: results.len(),
            algorithm: ConsensusAlgorithm::Weighted,
            reached,
        }
    }
}
```

## Cost Optimization

### Cost Breakdown by Tier

| Tier | Purpose | Model Class | Cost/1M tokens | Strategy |
|------|---------|-------------|----------------|----------|
| 1 | Data Collection | Gemini Flash | ~$0.075 | Cheap, fast, parallel |
| 2 | Reasoning | Claude/Gemini/GPT-4 | $1-5 | Premium, diverse |
| 3 | Synthesis | Claude Sonnet | ~$3 | Single coordinator |

### Estimated Costs

| Scenario | Single Model | Multi-Model Fleet | Savings |
|----------|--------------|-------------------|---------|
| Simple RCA | $5 (GPT-4) | $1.50 | 70% |
| Complex RCA | $15 (multiple calls) | $3-5 | 67-80% |
| Code Review | $10 | $2 | 80% |

### Why Multi-Model Can Be Cheaper

1. **Tier 1 uses cheapest models** - Data collection doesn't need reasoning power
2. **Parallel execution** - 3 models in parallel ≈ same time as 1 model
3. **Early termination** - If Tier 1 finds nothing, skip expensive Tier 2
4. **Right-sized models** - Each tier uses appropriate model size

## Use Cases

### 1. Incident Root Cause Analysis

```
Input: "Users reporting 500 errors on checkout since 2pm"

Tier 1 Output:
- Loki: 1,247 errors, first at 14:02, pattern: "connection refused"
- Prometheus: Error rate 15x normal, memory pressure detected
- K8s: 3 pods in CrashLoopBackOff, OOMKilled events
- Git: Config change at 13:58, memory limit reduced

Tier 2 Analysis:
- Claude: "Memory limit change caused OOM, cascading failures" (0.85)
- Gemini: "Database connection pool exhausted by restarts" (0.72)
- GPT-4: "Memory configuration change is root cause" (0.88)

Consensus: Memory configuration (confidence: 85%)

Tier 3 Report:
# RCA Report
Root Cause: Memory limit reduction in commit abc123
Confidence: HIGH (3/3 models agree on category)
Immediate Action: Rollback config change
```

### 2. Security Code Review

```
Input: Pull request #456 - new authentication endpoint

Tier 1 Output:
- Security scanner: Found potential SQL injection
- Dependency check: 2 CVEs in new packages
- Static analysis: Missing input validation

Tier 2 Analysis:
- Claude: "Critical SQL injection in login function"
- Gemini: "SQL injection + missing rate limiting"
- GPT-4: "SQL injection, needs parameterized queries"

Consensus: SQL injection vulnerability (confidence: 100%)
Action: Block merge, require fixes
```

### 3. Performance Analysis

```
Input: "API latency increased 300% this week"

Multi-model analysis identifies:
- Database query optimization needed (3/3 agree)
- Missing index on user_id (2/3 agree)
- Connection pool sizing (1/3 - flagged for review)
```

## Best Practices

### 1. Model Selection

| Use Case | Recommended Models | Why |
|----------|-------------------|-----|
| RCA | Claude + Gemini + GPT-4 | Diverse reasoning styles |
| Code Review | Claude + Gemini | Strong at code analysis |
| Data Analysis | Gemini + GPT-4 | Good at structured data |

### 2. Weight Calibration

Start with equal weights, then adjust based on accuracy:

```yaml
# Initial
weights:
  claude: 1.0
  gemini: 1.0
  gpt4: 1.0

# After calibration (based on your domain)
weights:
  claude: 1.5   # Better at your use case
  gemini: 1.0
  gpt4: 0.8     # Less accurate for your domain
```

### 3. Confidence Thresholds

| Scenario | Recommended `min_confidence` |
|----------|------------------------------|
| Informational | 0.5 |
| Actionable recommendations | 0.6 |
| Automated actions | 0.8 |
| Critical/destructive actions | 0.9 + human_review |

### 4. Handling Disagreement

When models disagree, the system should:
1. Report the disagreement clearly
2. Show each model's reasoning
3. Flag for human review if confidence < threshold
4. Log for future model calibration

## Monitoring & Observability

### Key Metrics

```yaml
# Fleet execution metrics
fleet_execution_duration_seconds
fleet_tier_duration_seconds{tier="1|2|3"}
fleet_consensus_confidence{algorithm="weighted"}
fleet_model_agreement_rate
fleet_cost_per_execution_usd

# Per-model metrics
model_response_time_seconds{model="claude|gemini|gpt4"}
model_agreement_rate{model="claude|gemini|gpt4"}
model_accuracy_rate{model="claude|gemini|gpt4"}  # Requires ground truth
```

### Example Dashboard Query

```promql
# Consensus confidence over time
histogram_quantile(0.50,
  rate(fleet_consensus_confidence_bucket[5m])
)

# Model agreement rate
sum(rate(model_agreement_total[1h])) by (model) /
sum(rate(model_response_total[1h])) by (model)
```

## Future Enhancements

1. **Adaptive Weights** - Automatically adjust weights based on historical accuracy
2. **Model Routing** - Route to specific models based on query type
3. **Cascade Execution** - Only invoke Tier 2 if Tier 1 indicates complexity
4. **Confidence Calibration** - Learn confidence thresholds from feedback
5. **Cost Budgets** - Enforce per-query cost limits

## See Also

- [Fleet Concepts](../concepts/fleets.md) - Overview of fleet coordination
- [Multi-Model RCA Tutorial](../tutorials/multi-model-rca.md) - Step-by-step guide
- [Fleet YAML Reference](../reference/fleet-spec.md) - Complete specification
- [Consensus Algorithms](../reference/consensus-algorithms.md) - Algorithm details
