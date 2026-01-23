---
sidebar_position: 11
sidebar_label: Prometheus
---

# Prometheus MCP Server

Query Prometheus metrics and alerts for observability automation.

## Installation

```bash
# Using npx
npx -y @anthropic/mcp-server-prometheus

# Or via npm
npm install -g @anthropic/mcp-server-prometheus
```

## Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: metrics-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: prometheus
      command: npx
      args: ["-y", "@anthropic/mcp-server-prometheus"]
      env:
        PROMETHEUS_URL: http://prometheus:9090
        # Optional: For authentication
        PROMETHEUS_USERNAME: ${PROM_USER}
        PROMETHEUS_PASSWORD: ${PROM_PASS}
```

### With Thanos/Cortex

```yaml
mcp_servers:
  - name: prometheus
    command: npx
    args: ["-y", "@anthropic/mcp-server-prometheus"]
    env:
      PROMETHEUS_URL: http://thanos-query:9090
      PROMETHEUS_TIMEOUT: "60s"  # Longer timeout for Thanos
```

## Available Tools

### query

Execute instant PromQL queries.

```json
{
  "name": "query",
  "arguments": {
    "query": "up{job='kubernetes-pods'}",
    "time": "2024-01-15T12:00:00Z"
  }
}
```

**Parameters**:
- `query` (required): PromQL query string
- `time` (optional): Evaluation timestamp (default: now)

### query_range

Execute range queries for time series data.

```json
{
  "name": "query_range",
  "arguments": {
    "query": "rate(http_requests_total[5m])",
    "start": "2024-01-15T11:00:00Z",
    "end": "2024-01-15T12:00:00Z",
    "step": "1m"
  }
}
```

**Parameters**:
- `query` (required): PromQL query string
- `start` (required): Start timestamp
- `end` (required): End timestamp
- `step` (optional): Query resolution step (default: 15s)

### get_alerts

Get current alerts from Prometheus.

```json
{
  "name": "get_alerts",
  "arguments": {
    "state": "firing",
    "labels": {"severity": "critical"}
  }
}
```

**Parameters**:
- `state` (optional): Filter by state (firing, pending, inactive)
- `labels` (optional): Filter by labels

### get_rules

List alerting and recording rules.

```json
{
  "name": "get_rules",
  "arguments": {
    "type": "alert"
  }
}
```

**Parameters**:
- `type` (optional): Rule type (alert, record)

### get_targets

Get scrape target status.

```json
{
  "name": "get_targets",
  "arguments": {
    "state": "active"
  }
}
```

**Parameters**:
- `state` (optional): Filter by state (active, dropped, any)

### get_labels

Get all label names or values.

```json
{
  "name": "get_labels",
  "arguments": {
    "label": "job"
  }
}
```

**Parameters**:
- `label` (optional): Get values for specific label

## Common PromQL Patterns

### Resource Usage

```promql
# CPU usage by pod
sum(rate(container_cpu_usage_seconds_total{namespace="production"}[5m])) by (pod)

# Memory usage percentage
100 * sum(container_memory_usage_bytes{namespace="production"})
  / sum(machine_memory_bytes)

# Disk usage
100 - (node_filesystem_avail_bytes / node_filesystem_size_bytes * 100)
```

### Request Metrics

```promql
# Request rate
sum(rate(http_requests_total[5m])) by (service)

# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m]))
  / sum(rate(http_requests_total[5m])) * 100

# P99 latency
histogram_quantile(0.99,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, service))
```

### SLO Calculations

```promql
# Availability (uptime)
avg_over_time(up{job="api"}[30d]) * 100

# Error budget consumed
1 - (
  sum(rate(http_requests_total{status!~"5.."}[30d]))
  / sum(rate(http_requests_total[30d]))
) / 0.001  # 99.9% SLO
```

## Use Cases

### SLO Guardian Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: slo-guardian
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Monitor SLO compliance using Prometheus.

    Track these SLIs:
    - Availability: 99.9% uptime
    - Latency: P99 < 200ms
    - Error rate: < 0.1%

    Report on error budget consumption and burn rate.
  mcp_servers:
    - name: prometheus
      command: npx
      args: ["-y", "@anthropic/mcp-server-prometheus"]
      env:
        PROMETHEUS_URL: ${PROMETHEUS_URL}
```

### Alert Investigator Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: alert-investigator
spec:
  model: google:gemini-2.5-flash
  instructions: |
    When investigating alerts:
    1. Query current alert status
    2. Check related metrics history
    3. Identify anomalies and patterns
    4. Correlate with recent deployments
    5. Suggest remediation steps
  mcp_servers:
    - name: prometheus
      command: npx
      args: ["-y", "@anthropic/mcp-server-prometheus"]
      env:
        PROMETHEUS_URL: ${PROMETHEUS_URL}
```

## Security Considerations

1. **Authentication**: Use basic auth or bearer tokens for secured Prometheus
2. **Query Limits**: Set timeout and max query length
3. **Read-Only**: MCP server only supports read operations
4. **Network**: Restrict access to internal Prometheus instances

## Troubleshooting

### Connection Issues

```bash
# Test Prometheus connectivity
curl ${PROMETHEUS_URL}/api/v1/status/config

# Verify MCP server
PROMETHEUS_URL=http://localhost:9090 npx -y @anthropic/mcp-server-prometheus
```

### Query Performance

- Use `step` parameter for range queries
- Limit time ranges for historical queries
- Use recording rules for complex queries

## Related

- [Grafana MCP Server](./grafana.md)
- [SLO Guardian Agent](/docs/agent-library/observability/slo-guardian)
- [Metrics Explorer Agent](/docs/agent-library/observability/metrics-explorer)
