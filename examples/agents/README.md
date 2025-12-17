# AOF Agent Catalog

Pre-built agent templates for common DevOps tasks. Use these as starting points for your own agents or reference them directly in fleet configurations.

## Directory Structure

```
agents/
├── observability/          # Data collection agents
│   ├── loki-collector.yaml
│   ├── prometheus-collector.yaml
│   ├── k8s-collector.yaml
│   └── git-collector.yaml
├── reasoning/              # Analysis and reasoning agents
│   ├── claude-analyzer.yaml
│   ├── gemini-analyzer.yaml
│   ├── gpt4-analyzer.yaml
│   └── rca-coordinator.yaml
└── README.md               # This file
```

## Observability Agents (Tier 1)

Low-cost data collection agents optimized for speed and structured output.

### loki-collector

**Purpose**: Query Loki for relevant logs during incident investigation.

| Property | Value |
|----------|-------|
| Model | `google:gemini-2.0-flash` |
| Cost | ~$0.075/1M tokens |
| Tier | 1 |
| Tools | `shell` |

**Output**: Structured JSON with error summaries, key logs, and first error identification.

**Usage**:
```yaml
agents:
  - name: loki-collector
    config: agents/observability/loki-collector.yaml
    tier: 1
```

### prometheus-collector

**Purpose**: Query Prometheus metrics to identify resource anomalies.

| Property | Value |
|----------|-------|
| Model | `google:gemini-2.0-flash` |
| Cost | ~$0.075/1M tokens |
| Tier | 1 |
| Tools | `shell` |

**Metrics Collected**:
- Error rates
- Latency percentiles
- Resource usage (CPU, memory, disk)
- Saturation indicators

**Usage**:
```yaml
agents:
  - name: prometheus-collector
    config: agents/observability/prometheus-collector.yaml
    tier: 1
```

### k8s-collector

**Purpose**: Collect Kubernetes cluster state information.

| Property | Value |
|----------|-------|
| Model | `google:gemini-2.0-flash` |
| Cost | ~$0.075/1M tokens |
| Tier | 1 |
| Tools | `shell` |

**Data Collected**:
- Unhealthy pods
- Recent events
- Resource pressure
- Recent deployments

**Usage**:
```yaml
agents:
  - name: k8s-collector
    config: agents/observability/k8s-collector.yaml
    tier: 1
```

### git-collector

**Purpose**: Audit recent Git changes for potential incident correlation.

| Property | Value |
|----------|-------|
| Model | `google:gemini-2.0-flash` |
| Cost | ~$0.075/1M tokens |
| Tier | 1 |
| Tools | `shell`, `git` |

**Data Collected**:
- Recent commits
- Changed files
- Suspicious changes
- Rollback candidates

**Usage**:
```yaml
agents:
  - name: git-collector
    config: agents/observability/git-collector.yaml
    tier: 1
```

## Reasoning Agents (Tier 2)

Multi-model analysis agents for diverse perspectives on incident root causes.

### claude-analyzer

**Purpose**: Root cause analysis using Claude's reasoning capabilities.

| Property | Value |
|----------|-------|
| Model | `anthropic:claude-sonnet-4-20250514` |
| Cost | ~$3/1M tokens |
| Tier | 2 |
| Weight | 1.5 (recommended) |
| Tools | `shell` |

**Analysis Approach**:
- Timeline reconstruction
- Correlation analysis
- 5 Whys technique

**Usage**:
```yaml
agents:
  - name: claude-analyzer
    config: agents/reasoning/claude-analyzer.yaml
    tier: 2
    weight: 1.5
```

### gemini-analyzer

**Purpose**: Root cause analysis using Gemini's systematic approach.

| Property | Value |
|----------|-------|
| Model | `google:gemini-2.5-pro` |
| Cost | ~$1.25/1M tokens |
| Tier | 2 |
| Weight | 1.0 (default) |
| Tools | `shell` |

**Analysis Approach**:
- Data synthesis
- Hypothesis generation
- Evidence evaluation with Occam's Razor

**Usage**:
```yaml
agents:
  - name: gemini-analyzer
    config: agents/reasoning/gemini-analyzer.yaml
    tier: 2
    weight: 1.0
```

### gpt4-analyzer

**Purpose**: Root cause analysis using GPT-4's structured reasoning.

| Property | Value |
|----------|-------|
| Model | `openai:gpt-4o` |
| Cost | ~$5/1M tokens |
| Tier | 2 |
| Weight | 1.0 (default) |
| Tools | `shell` |

**Analysis Approach**:
- Five Whys Analysis
- Fault Tree Analysis
- STAMP (System-Theoretic) Analysis

**Usage**:
```yaml
agents:
  - name: gpt4-analyzer
    config: agents/reasoning/gpt4-analyzer.yaml
    tier: 2
    weight: 1.0
```

## Coordinator Agents (Tier 3)

Manager agents that synthesize findings into final reports.

### rca-coordinator

**Purpose**: Synthesize multi-model analyses into a final RCA report.

| Property | Value |
|----------|-------|
| Model | `anthropic:claude-sonnet-4-20250514` |
| Cost | ~$3/1M tokens |
| Tier | 3 |
| Role | `manager` |
| Tools | `shell` |

**Capabilities**:
- Agreement analysis across models
- Disagreement resolution
- Evidence aggregation
- Structured RCA report generation

**Usage**:
```yaml
agents:
  - name: rca-coordinator
    config: agents/reasoning/rca-coordinator.yaml
    tier: 3
    role: manager
```

## Cost Summary

| Agent | Model | Cost/1M tokens | Typical RCA Usage |
|-------|-------|----------------|-------------------|
| loki-collector | Gemini Flash | $0.075 | ~15K tokens |
| prometheus-collector | Gemini Flash | $0.075 | ~10K tokens |
| k8s-collector | Gemini Flash | $0.075 | ~15K tokens |
| git-collector | Gemini Flash | $0.075 | ~10K tokens |
| claude-analyzer | Claude Sonnet | $3.00 | ~20K tokens |
| gemini-analyzer | Gemini Pro | $1.25 | ~20K tokens |
| gpt4-analyzer | GPT-4o | $5.00 | ~20K tokens |
| rca-coordinator | Claude Sonnet | $3.00 | ~15K tokens |

**Estimated total per RCA**: $0.50-1.00

## Creating Custom Agents

Use these agents as templates for your own:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: my-custom-agent
  labels:
    tier: "2"
    category: custom
spec:
  model: anthropic:claude-sonnet-4-20250514

  instructions: |
    Your custom instructions here...

    ## Output Format
    Return structured JSON...

  tools:
    - shell
    - read_file

  max_iterations: 5
  temperature: 0.4
```

## Using in Fleets

### Reference External Config

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
spec:
  agents:
    - name: my-collector
      config: agents/observability/loki-collector.yaml
      tier: 1
```

### Override Properties

```yaml
spec:
  agents:
    - name: high-priority-analyzer
      config: agents/reasoning/claude-analyzer.yaml
      tier: 2
      weight: 2.0  # Override default weight
```

## See Also

- [Multi-Model RCA Tutorial](../docs/tutorials/multi-model-rca.md)
- [Fleet Concepts](../docs/concepts/fleets.md)
- [Fleet YAML Reference](../docs/reference/fleet-spec.md)
