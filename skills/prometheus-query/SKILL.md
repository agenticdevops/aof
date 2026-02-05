---
name: prometheus-query
description: "Prometheus/PromQL querying, alerting analysis, and metrics exploration"
homepage: "https://docs.aof.sh/skills/prometheus-query"
metadata:
  emoji: "📊"
  version: "1.0.0"
  author: "AOF Team"
  license: "Apache-2.0"
  requires:
    any_bins:
      - promtool
      - curl
  tags:
    - prometheus
    - monitoring
    - metrics
    - promql
    - alerting
    - observability
---

# Prometheus Query Skill

Expert guidance for writing PromQL queries, analyzing metrics, and troubleshooting Prometheus alerting.

## When to Use This Skill

- Building PromQL queries for dashboards or alerts
- Investigating metric anomalies
- Debugging alerting rules
- Analyzing application performance metrics
- Capacity planning with historical data

## PromQL Fundamentals

### Basic Query Types

```promql
# Instant vector - current value
http_requests_total

# Range vector - values over time
http_requests_total[5m]

# Scalar - single numeric value
scalar(http_requests_total)
```

### Common Selectors

```promql
# Label matching
http_requests_total{job="api-server"}
http_requests_total{job="api-server", method="POST"}

# Regex matching
http_requests_total{job=~"api.*"}
http_requests_total{status!~"2.."}

# Multiple values
http_requests_total{method=~"GET|POST"}
```

## Essential Query Patterns

### Rate and Increase

```promql
# Per-second rate over 5 minutes
rate(http_requests_total[5m])

# Total increase over time window
increase(http_requests_total[1h])

# Use irate for volatile, short-term rates
irate(http_requests_total[1m])
```

### Aggregation

```promql
# Sum across all instances
sum(rate(http_requests_total[5m]))

# Sum by label
sum by (method) (rate(http_requests_total[5m]))

# Average
avg(rate(http_requests_total[5m]))

# Count
count(up{job="api-server"})

# Percentiles
histogram_quantile(0.95, sum(rate(http_request_duration_bucket[5m])) by (le))
```

### Filtering and Comparison

```promql
# Keep only high values
http_requests_total > 1000

# Top 5 by value
topk(5, sum by (instance) (rate(http_requests_total[5m])))

# Bottom 5
bottomk(5, sum by (instance) (rate(http_requests_total[5m])))
```

## Common Operational Queries

### Error Rates

```promql
# Error rate percentage
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100

# Error rate by endpoint
sum by (path) (rate(http_requests_total{status=~"5.."}[5m])) / sum by (path) (rate(http_requests_total[5m])) * 100
```

### Latency

```promql
# 95th percentile latency
histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))

# Average latency
sum(rate(http_request_duration_seconds_sum[5m])) / sum(rate(http_request_duration_seconds_count[5m]))

# Latency by service
histogram_quantile(0.99, sum by (service, le) (rate(http_request_duration_seconds_bucket[5m])))
```

### Resource Usage

```promql
# CPU usage by container
sum by (container) (rate(container_cpu_usage_seconds_total[5m]))

# Memory usage percentage
container_memory_working_set_bytes / container_spec_memory_limit_bytes * 100

# Disk usage
node_filesystem_avail_bytes / node_filesystem_size_bytes * 100
```

### Kubernetes-Specific

```promql
# Pod restarts
increase(kube_pod_container_status_restarts_total[1h])

# Pods not ready
kube_pod_status_ready{condition="false"}

# Deployment replicas mismatch
kube_deployment_spec_replicas - kube_deployment_status_replicas_available

# PVC usage
kubelet_volume_stats_used_bytes / kubelet_volume_stats_capacity_bytes * 100
```

## Alerting Rule Patterns

### High Error Rate Alert

```yaml
groups:
  - name: api-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"
```

### Latency Alert

```yaml
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P95 latency is {{ $value }}s"
```

### Resource Alert

```yaml
      - alert: PodMemoryHigh
        expr: |
          container_memory_working_set_bytes / container_spec_memory_limit_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Pod memory usage high"
          description: "{{ $labels.pod }} memory at {{ $value | humanizePercentage }}"
```

## Debugging Alerts

### Check Current Alert State

```bash
# Query Prometheus API
curl -s 'http://prometheus:9090/api/v1/alerts' | jq '.data.alerts[] | select(.state=="firing")'

# Check specific alert
curl -s 'http://prometheus:9090/api/v1/rules' | jq '.data.groups[].rules[] | select(.name=="HighErrorRate")'
```

### Test Alert Expression

```bash
# Instant query
curl -s 'http://prometheus:9090/api/v1/query?query=<expr>' | jq

# Range query
curl -s 'http://prometheus:9090/api/v1/query_range?query=<expr>&start=<start>&end=<end>&step=60s' | jq
```

## Performance Tips

1. **Use recording rules** for expensive queries used in dashboards
2. **Avoid high-cardinality labels** in aggregations
3. **Use `rate()` not `irate()`** for alerting (more stable)
4. **Set appropriate time ranges** - 5m is common default
5. **Use `without()` instead of `by()`** when excluding few labels

### Recording Rule Example

```yaml
groups:
  - name: api-recording
    rules:
      - record: job:http_requests:rate5m
        expr: sum by (job) (rate(http_requests_total[5m]))

      - record: job:http_request_latency_seconds:p95
        expr: histogram_quantile(0.95, sum by (job, le) (rate(http_request_duration_seconds_bucket[5m])))
```

## Useful Functions Reference

| Function | Description | Example |
|----------|-------------|---------|
| `rate()` | Per-second rate | `rate(counter[5m])` |
| `increase()` | Total increase | `increase(counter[1h])` |
| `histogram_quantile()` | Percentile from histogram | `histogram_quantile(0.99, ...)` |
| `sum()` | Sum values | `sum by (label) (metric)` |
| `avg()` | Average values | `avg(metric)` |
| `max()` / `min()` | Max/min values | `max by (instance) (metric)` |
| `topk()` / `bottomk()` | Top/bottom N | `topk(5, metric)` |
| `absent()` | Check if metric exists | `absent(up{job="api"})` |
| `changes()` | Number of value changes | `changes(metric[1h])` |
| `delta()` | Difference between first and last | `delta(gauge[1h])` |
| `deriv()` | Per-second derivative | `deriv(gauge[5m])` |
