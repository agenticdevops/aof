# Tutorial: Multi-Model RCA with Tiered Execution

Build a production-grade Root Cause Analysis fleet that combines multiple LLMs for consensus-based incident diagnosis.

## What You'll Build

A tiered fleet architecture where:
- **Tier 1**: Cheap, fast models collect observability data in parallel
- **Tier 2**: Multiple reasoning models (Claude, Gemini, GPT-4) analyze with diverse perspectives
- **Tier 3**: A coordinator synthesizes findings into a final RCA report

```
┌─────────────────────────────────────────────────────────────────┐
│                  MULTI-MODEL RCA ARCHITECTURE                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TIER 1: Data Collectors (~$0.075/1M tokens)                   │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                   │
│  │  Loki  │ │Promethe│ │  K8s   │ │  Git   │                   │
│  │  Logs  │ │Metrics │ │ State  │ │Changes │                   │
│  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘                   │
│      └──────────┼──────────┼──────────┘                        │
│                 ▼ (parallel execution)                         │
│                                                                 │
│  TIER 2: Reasoning Models (multi-model consensus)              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐                  │
│  │   Claude   │ │   Gemini   │ │   GPT-4    │                  │
│  │ (wt: 1.5)  │ │ (wt: 1.0)  │ │ (wt: 1.0)  │                  │
│  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘                  │
│        └──────────────┼──────────────┘                         │
│                       ▼ (weighted consensus)                   │
│                                                                 │
│  TIER 3: Coordinator                                           │
│  ┌──────────────────────────────────────────┐                  │
│  │            RCA Coordinator               │                  │
│  │     (synthesize final report)            │                  │
│  └──────────────────────────────────────────┘                  │
│                       ▼                                         │
│                Final RCA Report                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Why Multi-Model RCA?

### The Problem with Single-Model Analysis

When a production incident occurs, relying on a single LLM has risks:

| Risk | Impact |
|------|--------|
| **Model Bias** | Each LLM has different training data and reasoning patterns |
| **Hallucination** | No cross-validation means false conclusions go unchecked |
| **Blind Spots** | One model might miss what another catches |
| **Single Point of Failure** | If the model is wrong, your RCA is wrong |

### The Multi-Model Solution

By using multiple LLMs with weighted consensus:

- **Diverse Perspectives**: Claude, Gemini, and GPT-4 approach problems differently
- **Cross-Validation**: Areas of agreement have higher confidence
- **Disagreement Detection**: Conflicting conclusions are surfaced for human review
- **Cost Optimization**: Cheap models for data collection, premium for reasoning

## Prerequisites

- AOF installed (`aofctl version`)
- API keys for multiple providers:
  ```bash
  export ANTHROPIC_API_KEY=sk-ant-...
  export GOOGLE_API_KEY=AIza...
  export OPENAI_API_KEY=sk-...
  ```

## Step 1: Create Data Collector Agents

These Tier 1 agents use cheap, fast models to gather observability data.

### Loki Log Collector

Create `agents/observability/loki-collector.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: loki-collector
  labels:
    tier: "1"
    category: observability
    cost: low
spec:
  model: google:gemini-2.0-flash  # ~$0.075/1M tokens

  instructions: |
    You are a Log Collector agent that queries Loki for relevant logs.

    ## Your Task
    Extract logs related to the incident using LogQL queries.

    ## Query Strategy
    1. Start broad: {job=".*"} |~ "error|Error|ERROR"
    2. Filter by timeframe around incident
    3. Group by service/container
    4. Extract key error patterns

    ## Output Format
    Return structured JSON:
    ```json
    {
      "source": "loki",
      "timeframe": {"start": "...", "end": "..."},
      "error_summary": {
        "total_errors": 123,
        "by_service": {"api": 80, "worker": 43},
        "top_patterns": ["Connection refused", "Timeout"]
      },
      "key_logs": [
        {"timestamp": "...", "service": "...", "message": "..."}
      ],
      "first_error": {"timestamp": "...", "message": "..."}
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.2
```

### Prometheus Metrics Collector

Create `agents/observability/prometheus-collector.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: prometheus-collector
  labels:
    tier: "1"
    category: observability
    cost: low
spec:
  model: google:gemini-2.0-flash

  instructions: |
    You are a Metrics Collector agent that queries Prometheus.

    ## Key Metrics to Check
    - Error rates: rate(http_requests_total{status=~"5.."}[5m])
    - Latency: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
    - Resource usage: container_cpu_usage_seconds_total, container_memory_working_set_bytes
    - Saturation: container_cpu_throttled_seconds_total

    ## Output Format
    ```json
    {
      "source": "prometheus",
      "timeframe": {"start": "...", "end": "..."},
      "anomalies": [
        {
          "metric": "error_rate",
          "baseline": 0.01,
          "current": 0.15,
          "deviation": "15x normal"
        }
      ],
      "resource_status": {
        "cpu_throttled": false,
        "memory_pressure": true,
        "disk_pressure": false
      },
      "correlated_events": ["deploy at 14:02", "traffic spike at 14:05"]
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.2
```

### Kubernetes State Collector

Create `agents/observability/k8s-collector.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-collector
  labels:
    tier: "1"
    category: observability
    cost: low
spec:
  model: google:gemini-2.0-flash

  instructions: |
    You are a Kubernetes State Collector.

    ## Commands to Run
    - kubectl get pods -A | grep -v Running
    - kubectl get events --sort-by='.lastTimestamp' | tail -50
    - kubectl top pods --all-namespaces
    - kubectl describe deployment <affected-service>

    ## Output Format
    ```json
    {
      "source": "kubernetes",
      "cluster_health": "degraded|healthy|critical",
      "unhealthy_pods": [
        {"name": "...", "status": "CrashLoopBackOff", "restarts": 5}
      ],
      "recent_events": [
        {"type": "Warning", "reason": "...", "message": "..."}
      ],
      "resource_pressure": {
        "cpu_constrained_pods": [],
        "memory_constrained_pods": []
      },
      "recent_deployments": []
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.2
```

### Git Change Auditor

Create `agents/observability/git-collector.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: git-collector
  labels:
    tier: "1"
    category: observability
    cost: low
spec:
  model: google:gemini-2.0-flash

  instructions: |
    You are a Git Change Auditor.

    ## Commands to Run
    - git log --oneline --since="24 hours ago"
    - git diff HEAD~10 --stat
    - git log --oneline --all --graph | head -20

    ## Output Format
    ```json
    {
      "source": "git",
      "recent_commits": [
        {
          "hash": "abc123",
          "author": "...",
          "message": "...",
          "timestamp": "...",
          "files_changed": 5
        }
      ],
      "suspicious_changes": [
        {
          "commit": "abc123",
          "reason": "Config file modified",
          "files": ["config/database.yml"]
        }
      ],
      "rollback_candidates": ["abc123", "def456"]
    }
    ```

  tools:
    - shell
    - git

  max_iterations: 3
  temperature: 0.2
```

## Step 2: Create Reasoning Agents

These Tier 2 agents use different LLMs to analyze the collected data.

### Claude Analyzer

Create `agents/reasoning/claude-analyzer.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: claude-analyzer
  labels:
    tier: "2"
    category: reasoning
    cost: medium
spec:
  model: anthropic:claude-sonnet-4-20250514

  instructions: |
    You are a Root Cause Analysis Reasoning Agent powered by Claude.

    ## Your Role
    Analyze collected data from tier 1 agents to identify the root cause.

    ## Analysis Approach

    ### 1. Timeline Reconstruction
    - Order events chronologically
    - Identify the trigger event
    - Map the cascade of failures

    ### 2. Correlation Analysis
    - What changed before the incident?
    - Which metrics correlate with errors?
    - Are there common services/components?

    ### 3. Root Cause Identification
    - Apply the "5 Whys" technique
    - Distinguish symptoms from causes
    - Consider both technical and process factors

    ## Output Format
    ```json
    {
      "analysis_summary": "One paragraph summary",
      "confidence": 0.0-1.0,
      "root_cause": {
        "category": "code|config|infrastructure|dependency|capacity",
        "description": "Clear description",
        "evidence": ["evidence 1", "evidence 2"],
        "timeline_position": "What triggered the cascade"
      },
      "contributing_factors": [
        {"factor": "...", "impact": "high|medium|low", "evidence": "..."}
      ],
      "immediate_actions": [
        {"action": "...", "priority": "critical|high|medium", "expected_impact": "..."}
      ],
      "prevention_recommendations": ["..."]
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.4
```

### Gemini Analyzer

Create `agents/reasoning/gemini-analyzer.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: gemini-analyzer
  labels:
    tier: "2"
    category: reasoning
    cost: medium
spec:
  model: google:gemini-2.5-pro

  instructions: |
    You are a Root Cause Analysis Reasoning Agent powered by Gemini.

    ## Analysis Approach
    Use a structured, methodical approach:

    ### 1. Data Synthesis
    - Combine information from all data sources
    - Build a unified timeline
    - Identify correlations across sources

    ### 2. Hypothesis Generation
    - Generate multiple possible root causes
    - Rank by likelihood based on evidence
    - Consider both obvious and non-obvious causes

    ### 3. Evidence Evaluation
    - Evaluate supporting evidence for each hypothesis
    - Look for contradicting evidence
    - Apply Occam's Razor

    ## Output Format
    ```json
    {
      "analysis_summary": "One paragraph summary",
      "confidence": 0.0-1.0,
      "root_cause": {
        "category": "code|config|infrastructure|dependency|capacity",
        "description": "Clear description",
        "evidence": ["evidence 1", "evidence 2"]
      },
      "alternative_hypotheses": [
        {"hypothesis": "...", "likelihood": "low|medium", "missing_evidence": "..."}
      ],
      "immediate_actions": [
        {"action": "...", "priority": "critical|high|medium", "expected_impact": "..."}
      ],
      "verification_steps": ["..."],
      "prevention_recommendations": ["..."]
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.4
```

### GPT-4 Analyzer

Create `agents/reasoning/gpt4-analyzer.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: gpt4-analyzer
  labels:
    tier: "2"
    category: reasoning
    cost: medium
spec:
  model: openai:gpt-4o

  instructions: |
    You are a Root Cause Analysis Reasoning Agent powered by GPT-4.

    ## Analytical Framework

    ### Five Whys Analysis
    - Start with the symptom
    - Ask "why" iteratively
    - Drill down to the fundamental cause

    ### Fault Tree Analysis
    - Work backwards from the failure
    - Identify all possible causes
    - Evaluate each branch

    ### STAMP Analysis (System-Theoretic)
    - Consider the system as a whole
    - Look for control loop failures
    - Identify missing constraints

    ## Output Format
    ```json
    {
      "analysis_summary": "One paragraph summary",
      "confidence": 0.0-1.0,
      "root_cause": {
        "category": "code|config|infrastructure|dependency|capacity",
        "description": "Clear description",
        "evidence": ["evidence 1", "evidence 2"],
        "five_whys": ["Why 1", "Why 2", "Why 3", "Why 4", "Root cause"]
      },
      "system_weaknesses": [
        {"weakness": "...", "recommendation": "..."}
      ],
      "immediate_actions": [
        {"action": "...", "priority": "critical|high|medium", "risk": "..."}
      ],
      "long_term_fixes": [
        {"fix": "...", "effort": "low|medium|high", "impact": "..."}
      ]
    }
    ```

  tools:
    - shell

  max_iterations: 3
  temperature: 0.4
```

## Step 3: Create the RCA Coordinator

This Tier 3 manager synthesizes all analyses into a final report.

Create `agents/reasoning/rca-coordinator.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: rca-coordinator
  labels:
    tier: "3"
    category: coordinator
    cost: medium
spec:
  model: anthropic:claude-sonnet-4-20250514

  instructions: |
    You are the RCA Coordinator, responsible for synthesizing analyses from
    multiple reasoning agents into a coherent, actionable report.

    ## Your Role
    - Review analyses from all tier 2 reasoning agents
    - Identify areas of agreement and disagreement
    - Synthesize a consensus view with confidence levels
    - Produce the final RCA report

    ## Synthesis Process

    ### 1. Agreement Analysis
    - What do all/most agents agree on? (high confidence)
    - Where is there strong consensus?

    ### 2. Disagreement Resolution
    - Where do agents disagree?
    - Evaluate the evidence for each position
    - Note unresolved disagreements (lower confidence)

    ### 3. Evidence Aggregation
    - Combine evidence from all analyses
    - Remove duplicates, merge similar points
    - Rank evidence by strength

    ## Final Report Format

    ```markdown
    # Root Cause Analysis Report

    ## Incident Summary
    - **Incident**: [description]
    - **Duration**: [start to resolution]
    - **Impact**: [what was affected]
    - **Severity**: [P1/P2/P3/P4]

    ## Executive Summary
    [2-3 paragraphs summarizing the incident]

    ## Root Cause
    **Primary Cause**: [description]
    **Category**: [code|config|infrastructure|dependency|capacity]
    **Confidence**: [high|medium|low] ([X]% of analyzers agreed)

    ### Evidence
    1. [evidence point 1]
    2. [evidence point 2]

    ## Contributing Factors
    | Factor | Impact | Evidence |
    |--------|--------|----------|
    | [factor] | High/Med/Low | [evidence] |

    ## Timeline
    | Time | Event | Significance |
    |------|-------|--------------|
    | [time] | [event] | [why it matters] |

    ## Immediate Actions
    - [ ] [action 1]
    - [ ] [action 2]

    ## Follow-up Actions
    | Action | Priority | Owner | Due Date |
    |--------|----------|-------|----------|
    | [action] | P1/P2/P3 | TBD | TBD |

    ## Prevention Measures
    ### Short-term
    - [measure 1]

    ### Long-term
    - [measure 1]

    ## Appendix
    ### Analyzer Agreement Matrix
    | Finding | Claude | Gemini | GPT-4 | Consensus |
    |---------|--------|--------|-------|-----------|

    ---
    *Generated by AOF Multi-Model RCA Fleet*
    ```

  tools:
    - shell

  max_iterations: 5
  temperature: 0.5
```

## Step 4: Create the Fleet Definition

Create `fleets/multi-model-rca-fleet.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: multi-model-rca
  labels:
    purpose: incident-response
    type: rca
    multi-model: "true"

spec:
  agents:
    # ============================================
    # TIER 1: Data Collectors (cheap, parallel)
    # ============================================
    - name: loki-collector
      config: ../agents/observability/loki-collector.yaml
      tier: 1
      role: specialist

    - name: prometheus-collector
      config: ../agents/observability/prometheus-collector.yaml
      tier: 1
      role: specialist

    - name: k8s-collector
      config: ../agents/observability/k8s-collector.yaml
      tier: 1
      role: specialist

    - name: git-collector
      config: ../agents/observability/git-collector.yaml
      tier: 1
      role: specialist

    # ============================================
    # TIER 2: Reasoning Agents (multi-model)
    # ============================================
    - name: claude-analyzer
      config: ../agents/reasoning/claude-analyzer.yaml
      tier: 2
      weight: 1.5  # Higher weight for Claude

    - name: gemini-analyzer
      config: ../agents/reasoning/gemini-analyzer.yaml
      tier: 2
      weight: 1.0

    - name: gpt4-analyzer
      config: ../agents/reasoning/gpt4-analyzer.yaml
      tier: 2
      weight: 1.0

    # ============================================
    # TIER 3: Coordinator (synthesis)
    # ============================================
    - name: rca-coordinator
      config: ../agents/reasoning/rca-coordinator.yaml
      tier: 3
      role: manager

  coordination:
    mode: tiered
    distribution: round-robin

    # Global consensus configuration
    consensus:
      algorithm: weighted
      min_votes: 2
      timeout_ms: 180000  # 3 minutes
      allow_partial: true
      min_confidence: 0.6

    # Tiered execution configuration
    tiered:
      pass_all_results: true
      final_aggregation: manager_synthesis

      # Per-tier consensus
      tier_consensus:
        "1":
          algorithm: first_wins  # Just collect data fast
        "2":
          algorithm: weighted    # Multi-model analysis
          min_votes: 2
          min_confidence: 0.5
        "3":
          algorithm: first_wins  # Single coordinator

  # Shared memory for cross-agent communication
  shared:
    memory:
      type: inmemory
      namespace: rca-session
      ttl: 3600
```

## Step 5: Run the Fleet

```bash
# Set API keys
export ANTHROPIC_API_KEY=sk-ant-...
export GOOGLE_API_KEY=AIza...
export OPENAI_API_KEY=sk-...

# Run the multi-model RCA
aofctl run fleet fleets/multi-model-rca-fleet.yaml \
  --input "Investigate: Users reporting 500 errors on checkout API since 2pm UTC"
```

## Understanding the Output

### Execution Flow

```
[FLEET] Initializing multi-model-rca with 8 agents
[TIER 1] Starting 4 data collectors in parallel...
  [AGENT] loki-collector: Querying Loki for error logs
  [AGENT] prometheus-collector: Querying Prometheus metrics
  [AGENT] k8s-collector: Checking Kubernetes state
  [AGENT] git-collector: Auditing recent changes
[TIER 1] Complete. Consensus: first_wins (4 results)

[TIER 2] Starting 3 reasoning agents in parallel...
  [AGENT] claude-analyzer: Analyzing with Claude
  [AGENT] gemini-analyzer: Analyzing with Gemini
  [AGENT] gpt4-analyzer: Analyzing with GPT-4
[TIER 2] Complete. Weighted consensus reached (confidence: 0.85)
  - Root cause agreement: 3/3 models
  - Contributing factors: 2/3 agreement

[TIER 3] Starting coordinator synthesis...
  [AGENT] rca-coordinator: Generating final report
[TIER 3] Complete.

[FLEET] Final RCA Report generated
```

### Consensus Results

The fleet tracks agreement between models:

```
Analyzer Agreement Matrix:
| Finding              | Claude | Gemini | GPT-4 | Consensus |
|----------------------|--------|--------|-------|-----------|
| DB connection issue  | ✓      | ✓      | ✓     | HIGH      |
| Memory pressure      | ✓      | ✓      |       | MEDIUM    |
| Config change        |        | ✓      | ✓     | MEDIUM    |
| Network latency      | ✓      |        |       | LOW       |
```

## Cost Optimization

### Estimated Costs per RCA

| Tier | Agents | Model | Cost/1M tokens | Typical Usage |
|------|--------|-------|----------------|---------------|
| 1 | 4 | Gemini Flash | $0.075 | ~50K tokens |
| 2 | 3 | Claude/Gemini/GPT-4 | $3-15 | ~20K tokens each |
| 3 | 1 | Claude Sonnet | $3 | ~10K tokens |

**Total estimated cost**: ~$0.50-1.00 per RCA (vs $5-10 for single GPT-4 analysis)

### Cost Reduction Strategies

1. **Tier 1 Caching**: Cache observability data for similar incidents
2. **Selective Tier 2**: Use 2 models instead of 3 for lower-severity incidents
3. **Model Selection**: Use cheaper models for routine analysis

## Customization

### Adding Custom Data Sources

```yaml
agents:
  - name: splunk-collector
    tier: 1
    spec:
      model: google:gemini-2.0-flash
      instructions: |
        Query Splunk for application logs...
      tools:
        - shell
```

### Adjusting Consensus Weights

```yaml
agents:
  - name: claude-analyzer
    tier: 2
    weight: 2.0  # Claude counts as 2 votes

  - name: junior-model
    tier: 2
    weight: 0.5  # Less experienced model counts as 0.5
```

### Using Human Review for Critical Incidents

```yaml
coordination:
  consensus:
    algorithm: human_review  # Always flag for human decision
    min_confidence: 0.9
```

## Best Practices

### 1. Keep Tier 1 Agents Fast and Cheap

Tier 1 should collect data, not analyze it:
- Use the cheapest models available
- Keep instructions simple and focused
- Output structured JSON for downstream agents

### 2. Diversify Tier 2 Models

Use models from different providers:
- Anthropic (Claude): Strong reasoning, safety-focused
- Google (Gemini): Good at structured data
- OpenAI (GPT-4): Broad knowledge base

### 3. Weight Based on Track Record

Adjust weights based on your historical accuracy:
```yaml
weight: 1.5  # This model has been more accurate for your use case
```

### 4. Set Appropriate Timeouts

```yaml
timeout_ms: 180000  # 3 minutes for full RCA
tier_consensus:
  "1":
    timeout_ms: 30000   # 30s for data collection
  "2":
    timeout_ms: 120000  # 2 min for reasoning
```

## Troubleshooting

### Models Disagree on Root Cause

This is actually valuable information! The disagreement matrix in the final report helps you:
- Identify where human judgment is needed
- Understand the uncertainty in the analysis
- Prioritize follow-up investigation

### Tier 1 Agents Timing Out

Check connectivity to observability tools:
```bash
# Test Loki
curl -G "http://loki:3100/loki/api/v1/query" --data-urlencode 'query={job="test"}'

# Test Prometheus
curl "http://prometheus:9090/api/v1/query?query=up"
```

### Low Confidence Results

Increase data quality:
- Add more Tier 1 collectors
- Extend time window for data collection
- Add specific instructions for your stack

## Summary

You've built a production-grade multi-model RCA system that:

- ✅ Collects data from multiple observability sources in parallel
- ✅ Analyzes with diverse LLM perspectives
- ✅ Uses weighted consensus for reliable conclusions
- ✅ Produces actionable RCA reports
- ✅ Optimizes costs with tiered model selection

## Next Steps

- **[Fleet Concepts](../concepts/fleets.md)** - Deep dive into fleet coordination modes
- **[Fleet YAML Reference](../reference/fleet-spec.md)** - Complete specification
- **[Example Agents](../../examples/agents/README.md)** - Pre-built agent catalog

---

**Production incident?** Deploy your multi-model RCA fleet and let diverse AI perspectives find the root cause while you focus on mitigation!
