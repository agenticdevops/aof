# Real Fleet Examples

This directory contains **production-ready** fleet examples that use actual infrastructure.

## Purpose

Real examples are designed for:
- **Production incident response**
- **Actual RCA with real data**
- **Integration with your observability stack**
- **Automated runbook execution**

## Available Real Fleets

| Fleet | Description | Prerequisites |
|-------|-------------|---------------|
| `multi-model-rca-real.yaml` | Multi-model RCA with real Prometheus/Loki/K8s | Full monitoring stack |

## Prerequisites

### Required Infrastructure

| Component | Purpose | Default Endpoint |
|-----------|---------|------------------|
| Kubernetes | Cluster state | kubectl configured |
| Prometheus | Metrics | `http://localhost:30400` |
| Loki | Logs | `http://localhost:30700` |

### API Keys

```bash
# At least one required
export GOOGLE_API_KEY=your-key
export ANTHROPIC_API_KEY=your-key  # Optional, for Claude
```

### Verify Your Setup

```bash
# Check Kubernetes
kubectl get nodes

# Check Prometheus
curl -s "http://localhost:30400/api/v1/query?query=up" | jq '.status'

# Check Loki
curl -s "http://localhost:30700/loki/api/v1/labels" | jq '.status'
```

## Usage

### Basic Usage

```bash
# Run against real infrastructure
aofctl run fleet examples/fleets/real/multi-model-rca-real.yaml \
  --input "Investigate: High memory usage in monitoring namespace"
```

### With Custom Endpoints

```bash
# If your Prometheus/Loki are on different ports
PROMETHEUS_URL=http://localhost:9090 \
LOKI_URL=http://localhost:3100 \
aofctl run fleet examples/fleets/real/multi-model-rca-real.yaml \
  --input "Investigate: Pod crashes in production"
```

### Verbose Mode

```bash
# See detailed execution
aofctl run fleet examples/fleets/real/multi-model-rca-real.yaml \
  --input "Investigate: API latency spike" \
  --verbose
```

## What Real Fleets Do

```
┌─────────────────────────────────────────────────────────────────┐
│                      REAL FLEET                                 │
│                                                                 │
│  Tier 1 collectors actually run:                               │
│  ─────────────────────────────────                             │
│  curl http://prometheus:30400/api/v1/query?query=...           │
│  curl http://loki:30700/loki/api/v1/query_range?query=...      │
│  kubectl get pods -A                                           │
│  kubectl get events --sort-by='.lastTimestamp'                 │
│                                                                 │
│  Real data → Real analysis → Actionable report                 │
└─────────────────────────────────────────────────────────────────┘
```

## Output Example

Real fleets produce actionable reports with actual data:

```markdown
# 🔴 Root Cause Analysis Report

## 🎯 Root Cause
**Category**: config
**Description**: Memory limit too low for prometheus-0 pod
**Confidence**: HIGH (2/2 models agreed)

### Evidence
1. **From Prometheus**: container_memory_working_set_bytes{pod="prometheus-0"} = 1.8GB (limit: 2GB)
2. **From Loki**: 47 OOM warning logs in last hour
3. **From K8s**: Pod restarted 3 times, last reason: OOMKilled

## 🚨 Immediate Actions
1. [ ] **Increase memory limit** (Priority: CRITICAL)
   ```bash
   kubectl patch deployment prometheus -n monitoring \
     -p '{"spec":{"template":{"spec":{"containers":[{"name":"prometheus","resources":{"limits":{"memory":"4Gi"}}}]}}}}'
   ```
```

## Cost Considerations

Real fleets process more data and cost slightly more:

| Tier | Agents | Est. Tokens | Est. Cost |
|------|--------|-------------|-----------|
| 1 (Collectors) | 4 | ~30K | ~$0.02 |
| 2 (Reasoning) | 2 | ~40K | ~$0.10 |
| 3 (Coordinator) | 1 | ~20K | ~$0.05 |
| **Total** | 7 | ~90K | **~$0.17** |

## Customizing for Your Environment

### Different Prometheus/Loki URLs

Edit the agent instructions:

```yaml
instructions: |
  ## Prometheus Endpoint
  URL: http://your-prometheus:9090
```

### Add More Data Sources

Add agents for your specific tools:

```yaml
- name: datadog-collector
  tier: 1
  spec:
    model: google:gemini-2.0-flash
    instructions: |
      Query Datadog metrics using the API...
```

### Adjust for Your Namespace

```yaml
instructions: |
  Focus on namespace: production
  kubectl get pods -n production
```

## Security Considerations

- Real fleets execute actual commands in your cluster
- Agents only have read access (no mutations by default)
- Consider RBAC if running in production
- API keys are used for LLM calls only

## See Also

- **Mock examples**: `../mock/` - For testing without infrastructure
- **Multi-Model RCA Tutorial**: `../../docs/tutorials/multi-model-rca.md`
- **Architecture Guide**: `../../docs/architecture/multi-model-consensus.md`
