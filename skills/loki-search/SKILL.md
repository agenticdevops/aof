---
name: loki-search
description: "Loki log searching, LogQL queries, and log analysis"
homepage: "https://docs.aof.sh/skills/loki-search"
metadata:
  emoji: "📜"
  version: "1.0.0"
  author: "AOF Team"
  license: "Apache-2.0"
  requires:
    any_bins:
      - logcli
      - curl
  install:
    - id: brew-logcli
      kind: brew
      package: logcli
      bins:
        - logcli
  tags:
    - loki
    - logging
    - logql
    - observability
    - troubleshooting
---

# Loki Search Skill

Expert guidance for querying logs with Loki, writing LogQL queries, and analyzing log patterns.

## When to Use This Skill

- Searching logs for errors or specific events
- Correlating logs across services
- Building log-based alerts
- Analyzing log patterns and frequencies
- Investigating incidents with log data

## LogQL Basics

### Stream Selectors

```logql
# Select by label
{job="api-server"}

# Multiple labels
{job="api-server", namespace="production"}

# Regex matching
{job=~"api.*"}

# Not equal
{job!="test"}

# Regex not matching
{namespace!~"dev|staging"}
```

### Log Pipeline

```logql
# Filter lines containing text
{job="api-server"} |= "error"

# Filter lines NOT containing text
{job="api-server"} != "debug"

# Regex filter
{job="api-server"} |~ "error|warn"

# Case-insensitive
{job="api-server"} |~ "(?i)error"
```

### Parser Stages

```logql
# JSON parser
{job="api-server"} | json

# Logfmt parser
{job="api-server"} | logfmt

# Regex parser
{job="api-server"} | regexp `level=(?P<level>\w+)`

# Pattern parser
{job="api-server"} | pattern `<ip> - - <_> "<method> <path> <_>" <status>`
```

### Label Filters (after parsing)

```logql
# Filter by extracted label
{job="api-server"} | json | level="error"

# Numeric comparison
{job="api-server"} | json | status >= 500

# Multiple conditions
{job="api-server"} | json | level="error" and duration > 1000
```

## Common Query Patterns

### Error Searching

```logql
# Find all errors
{namespace="production"} |= "error"

# JSON logs with error level
{namespace="production"} | json | level="error"

# Errors in specific service
{app="payment-service"} | json | level=~"error|fatal"

# Stack traces (multi-line)
{app="api"} |~ "(?s)Exception.*?at .*"
```

### Request/Response Analysis

```logql
# Slow requests (JSON logs)
{job="api"} | json | response_time > 1000

# 5xx errors
{job="api"} | json | status >= 500

# Specific endpoint errors
{job="api"} | json | path="/api/users" | status >= 400
```

### Application-Specific

```logql
# Kubernetes pod logs
{namespace="production", pod=~"api-.*"}

# Container logs
{namespace="production", container="app"}

# Specific deployment
{namespace="production"} | json | kubernetes_labels_app="my-app"
```

## Metric Queries

### Log-Based Metrics

```logql
# Count of errors per minute
sum(count_over_time({job="api"} |= "error" [1m]))

# Rate of requests
rate({job="api"} | json | path="/api/users" [5m])

# Errors by service
sum by (service) (count_over_time({namespace="prod"} | json | level="error" [5m]))
```

### Aggregations

```logql
# Sum
sum(count_over_time({job="api"} [5m]))

# Average
avg(bytes_over_time({job="api"} [5m]))

# Max/Min
max(count_over_time({job="api"} [5m]))

# Top by label
topk(5, sum by (service) (count_over_time({namespace="prod"} [5m])))
```

### Quantiles (from extracted values)

```logql
# P99 latency from logs
quantile_over_time(0.99, {job="api"} | json | unwrap response_time [5m]) by (endpoint)

# P95 by service
quantile_over_time(0.95, {job="api"} | json | unwrap duration [5m]) by (service)
```

## LogCLI Usage

### Basic Queries

```bash
# Set Loki address
export LOKI_ADDR=http://loki:3100

# Query logs
logcli query '{job="api"}'

# Query with time range
logcli query '{job="api"}' --from="2h" --to="now"

# Limit results
logcli query '{job="api"}' --limit=100

# Output format
logcli query '{job="api"}' --output=jsonl
```

### Time Ranges

```bash
# Last hour
logcli query '{job="api"}' --from="1h"

# Specific time
logcli query '{job="api"}' --from="2024-01-15T10:00:00Z" --to="2024-01-15T11:00:00Z"

# Relative time
logcli query '{job="api"}' --from="2024-01-15T10:00:00Z" --to="1h"
```

### Follow Logs (Tail)

```bash
# Tail logs
logcli query '{job="api"}' --tail

# Tail with delay
logcli query '{job="api"}' --tail --delay-for=2s
```

## Troubleshooting Queries

### No Results

1. **Check label names exist:**
```logql
{job="api"}  # Returns nothing?
# Try browsing labels first
```

2. **Verify time range:**
```bash
logcli query '{job="api"}' --from="24h"
```

3. **Check label values:**
```bash
logcli labels job
logcli labels namespace
```

### Query Too Slow

1. **Add more selective labels:**
```logql
# Too broad
{namespace="production"} |= "error"

# Better
{namespace="production", app="api"} |= "error"
```

2. **Reduce time range**

3. **Avoid complex regex when possible:**
```logql
# Slower
{job="api"} |~ "error|warn|fatal"

# Faster
{job="api", level=~"error|warn|fatal"}
```

### Parser Not Working

```logql
# Debug: see raw lines first
{job="api"} | limit 10

# Test JSON parser
{job="api"} | json | __error__=""

# See parse errors
{job="api"} | json | __error__!=""
```

## Alert Examples

### Error Rate Alert

```yaml
groups:
  - name: loki-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(count_over_time({namespace="production"} | json | level="error" [5m])) > 100
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate in production"
```

### Missing Logs Alert

```yaml
      - alert: NoLogs
        expr: |
          absent(count_over_time({job="critical-service"} [5m]))
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "No logs from critical-service"
```

## Performance Tips

1. **Use specific labels** - More labels = faster queries
2. **Avoid `.*` regex** when possible
3. **Use line filters before parsers** - Filter early
4. **Prefer `|=` over `|~`** for literal strings
5. **Set reasonable time ranges** - Shorter = faster

## Best Practices

1. **Structure your logs** - Use JSON for easy parsing
2. **Add context labels** - Service, environment, version
3. **Include trace IDs** - For distributed tracing correlation
4. **Consistent field names** - `level`, `message`, `error`, etc.
5. **Avoid high cardinality** - Don't use request IDs as labels

## Useful Query Templates

| Use Case | Query |
|----------|-------|
| All errors | `{namespace="prod"} \|= "error"` |
| Errors by service | `sum by (app) (count_over_time({namespace="prod"} \| json \| level="error" [5m]))` |
| Slow requests | `{job="api"} \| json \| response_time > 1000` |
| Recent exceptions | `{job="api"} \|~ "Exception\|Error" \| limit 50` |
| Specific user activity | `{job="api"} \| json \| user_id="12345"` |
| HTTP 5xx errors | `{job="api"} \| json \| status >= 500` |
| Request rate | `rate({job="api"} \| json \| path="/api/v1/users" [1m])` |
