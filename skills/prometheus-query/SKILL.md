---
name: prometheus-query
description: "Query Prometheus metrics for monitoring and alerting"
homepage: "https://docs.aof.sh/skills/prometheus-query"
metadata:
  emoji: "📊"
  version: "1.0.0"
  requires:
    bins: ["curl", "jq"]
    env: []
    config: []
  tags: ["monitoring", "prometheus", "metrics"]
---

# Prometheus Query Skill

Query Prometheus metrics to analyze system performance, troubleshoot issues, and verify SLOs.

## When to Use This Skill

- Need to check metrics for a service
- Investigating performance degradation
- Verifying SLO compliance
- Analyzing historical trends
- Debugging autoscaling decisions

## Steps

1. **Query current value** — `curl 'http://prometheus:9090/api/v1/query?query=...'`
2. **Query range** — Use start/end timestamps for historical data
3. **Execute PromQL** — Use expressions like `rate(requests[5m])`
4. **Parse results** — Extract value and timestamp with jq
5. **Analyze trend** — Check for increasing/decreasing patterns
