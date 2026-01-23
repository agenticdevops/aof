---
sidebar_position: 14
sidebar_label: Datadog
---

# Datadog MCP Server

Query metrics, monitors, and events from Datadog for observability automation.

## Installation

```bash
# Using npx
npx -y @anthropic/mcp-server-datadog

# Or via npm
npm install -g @anthropic/mcp-server-datadog
```

## Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: datadog-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: datadog
      command: npx
      args: ["-y", "@anthropic/mcp-server-datadog"]
      env:
        DD_API_KEY: ${DD_API_KEY}
        DD_APP_KEY: ${DD_APP_KEY}
        DD_SITE: datadoghq.com  # or datadoghq.eu, us3.datadoghq.com, etc.
```

## Available Tools

### query_metrics

Query timeseries metrics.

```json
{
  "name": "query_metrics",
  "arguments": {
    "query": "avg:system.cpu.user{env:production} by {host}",
    "from": 1705276800,
    "to": 1705320000
  }
}
```

**Parameters**:
- `query` (required): Datadog metrics query
- `from` (required): Start timestamp (epoch seconds)
- `to` (required): End timestamp (epoch seconds)

### get_monitors

List monitors with filters.

```json
{
  "name": "get_monitors",
  "arguments": {
    "tags": ["env:production", "team:platform"],
    "monitor_tags": ["service:api"],
    "group_states": ["Alert", "Warn"]
  }
}
```

**Parameters**:
- `tags` (optional): Filter by tags
- `monitor_tags` (optional): Filter by monitor tags
- `group_states` (optional): Filter by states (Alert, Warn, No Data, OK)

### get_monitor

Get specific monitor details.

```json
{
  "name": "get_monitor",
  "arguments": {
    "monitor_id": 12345678
  }
}
```

### get_events

Get events from event stream.

```json
{
  "name": "get_events",
  "arguments": {
    "start": 1705276800,
    "end": 1705320000,
    "tags": ["env:production"],
    "priority": "normal",
    "sources": ["kubernetes", "cloudwatch"]
  }
}
```

**Parameters**:
- `start` (required): Start timestamp
- `end` (required): End timestamp
- `tags` (optional): Filter by tags
- `priority` (optional): Filter by priority (low, normal)
- `sources` (optional): Filter by source

### post_event

Create an event.

```json
{
  "name": "post_event",
  "arguments": {
    "title": "Deployment: api-v1.2.3",
    "text": "Deployed new version of API service",
    "tags": ["env:production", "service:api"],
    "alert_type": "info",
    "source_type_name": "aof"
  }
}
```

**Parameters**:
- `title` (required): Event title
- `text` (required): Event body (supports markdown)
- `tags` (optional): Event tags
- `alert_type` (optional): error, warning, info, success
- `source_type_name` (optional): Source name

### get_dashboards

List dashboards.

```json
{
  "name": "get_dashboards",
  "arguments": {
    "filter_shared": false,
    "filter_deleted": false
  }
}
```

### get_hosts

Get host information.

```json
{
  "name": "get_hosts",
  "arguments": {
    "filter": "env:production",
    "sort_field": "cpu",
    "sort_dir": "desc",
    "count": 100
  }
}
```

**Parameters**:
- `filter` (optional): Tag filter string
- `sort_field` (optional): Sort by field (cpu, iowait, load)
- `sort_dir` (optional): Sort direction (asc, desc)
- `count` (optional): Max results

### search_logs

Search log data.

```json
{
  "name": "search_logs",
  "arguments": {
    "query": "service:api status:error",
    "from": "now-1h",
    "to": "now",
    "limit": 100,
    "sort": "desc"
  }
}
```

**Parameters**:
- `query` (required): Log search query
- `from` (required): Start time (relative or absolute)
- `to` (required): End time
- `limit` (optional): Max results
- `sort` (optional): Sort direction

## Common Query Patterns

### Infrastructure Metrics

```
# CPU by host
avg:system.cpu.user{env:production} by {host}

# Memory usage
avg:system.mem.used{*} / avg:system.mem.total{*} * 100

# Disk usage
max:system.disk.in_use{*} by {device,host}
```

### Application Metrics

```
# Request rate
sum:trace.servlet.request.hits{env:production}.as_rate()

# Error rate
sum:trace.servlet.request.errors{env:production}.as_rate()
  / sum:trace.servlet.request.hits{env:production}.as_rate() * 100

# P99 latency
p99:trace.servlet.request{env:production}
```

### Container Metrics

```
# Container CPU
avg:docker.cpu.usage{*} by {container_name}

# Kubernetes pod restarts
sum:kubernetes.containers.restarts{*} by {pod_name}
```

## Use Cases

### Datadog Monitor Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dd-monitor
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Monitor Datadog alerts and investigate issues.

    When asked about an alert:
    1. Get monitor details and history
    2. Query related metrics
    3. Search for correlated events
    4. Check affected hosts
    5. Provide root cause analysis
  mcp_servers:
    - name: datadog
      command: npx
      args: ["-y", "@anthropic/mcp-server-datadog"]
      env:
        DD_API_KEY: ${DD_API_KEY}
        DD_APP_KEY: ${DD_APP_KEY}
```

### Log Analyzer Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: log-analyzer
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Analyze logs from Datadog to identify issues.

    Focus on:
    - Error patterns
    - Anomalies
    - Performance degradation
    - Security events
  mcp_servers:
    - name: datadog
      command: npx
      args: ["-y", "@anthropic/mcp-server-datadog"]
      env:
        DD_API_KEY: ${DD_API_KEY}
        DD_APP_KEY: ${DD_APP_KEY}
```

## Security Considerations

1. **API Keys**: Use application keys with limited scope
2. **Scoped Access**: Create keys with specific permissions
3. **Key Rotation**: Rotate application keys regularly
4. **Audit Logs**: Monitor API key usage in Datadog

### Key Permissions

| Use Case | Required Permissions |
|----------|---------------------|
| Read metrics | `metrics_read` |
| Read monitors | `monitors_read` |
| Post events | `events_write` |
| Read logs | `logs_read` |

## Troubleshooting

### Authentication Issues

```bash
# Test API key
curl -X GET "https://api.datadoghq.com/api/v1/validate" \
  -H "DD-API-KEY: ${DD_API_KEY}"

# Test app key
curl -X GET "https://api.datadoghq.com/api/v1/dashboard" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}"
```

### Site Configuration

Different Datadog sites require different endpoints:

| Site | DD_SITE |
|------|---------|
| US1 | datadoghq.com |
| US3 | us3.datadoghq.com |
| US5 | us5.datadoghq.com |
| EU | datadoghq.eu |
| AP1 | ap1.datadoghq.com |

## Related

- [Prometheus MCP Server](./prometheus.md)
- [Grafana MCP Server](./grafana.md)
- [Alert Manager Agent](/docs/agent-library/observability/alert-manager)
