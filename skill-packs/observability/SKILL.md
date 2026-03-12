# Observability Operations Skill

You have expertise in monitoring, alerting, log analysis, and distributed tracing.

## Log Analysis
- Filter by time range first, then narrow by content
- Look for correlated errors across services using correlation IDs or trace IDs
- Use structured log fields for reliable filtering (avoid grep on free-text logs)
- Check log volume anomalies — sudden spike or drop often indicates a problem

## Metrics Interpretation
- Four Golden Signals: Latency, Traffic, Errors, Saturation
- Latency: check p95/p99, not just mean — outliers matter
- Error rate: calculate as percentage of total traffic, not absolute count
- Saturation: CPU/memory/disk/queue depth — look for approaching limits (>80%)

## Tracing
- Find the critical path: look for the slowest span in a trace waterfall
- Database queries: check N+1 patterns (many identical short-duration spans)
- External calls: look for missing timeouts causing cascade failures
- Trace sampling: be aware that sampled traces may miss infrequent errors

## Common Tools
- Prometheus: `promql` queries for metrics; use `rate()` for counters
- Grafana: dashboards for visualization; use `explore` for ad-hoc queries
- Jaeger/Tempo: distributed trace inspection
- `kubectl logs` / `docker logs`: container log access
- CloudWatch Insights: `fields @timestamp, @message | filter @message like /ERROR/ | limit 50`
