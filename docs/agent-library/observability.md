---
sidebar_position: 3
sidebar_label: Observability
---

# Observability Agents

Production-ready agents for monitoring, alerting, and observability operations.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [alert-manager](#alert-manager) | Manage alerting rules | grafana_*, prometheus |
| [slo-guardian](#slo-guardian) | Monitor SLO compliance | grafana_*, datadog_* |
| [dashboard-generator](#dashboard-generator) | Auto-generate dashboards | grafana_dashboard_* |
| [log-analyzer](#log-analyzer) | Analyze logs for patterns | loki_query, aws_logs |
| [trace-investigator](#trace-investigator) | Investigate traces | kubectl |
| [newrelic-ops](#newrelic-ops) | New Relic observability | newrelic_* |
| [splunk-analyst](#splunk-analyst) | Splunk log analysis & SIEM | splunk_* |

## alert-manager

Manages and optimizes alerting rules to reduce alert fatigue and improve signal quality.

### Usage

```bash
# Analyze alert fatigue
aofctl run agent library://observability/alert-manager \
  --prompt "Analyze alerting rules for noise and fatigue"

# Optimize specific alerts
aofctl run agent library://observability/alert-manager \
  --prompt "Why is the high-cpu-usage alert firing constantly?"
```

### Capabilities

- Analyzes alert frequency and patterns
- Identifies noisy/flapping alerts
- Recommends threshold adjustments
- Groups related alerts
- Suggests silence rules
- Optimizes notification routing

### Example Output

```markdown
## Alert Analysis Report

### Alert Fatigue Score: HIGH (7.2/10)

**Analysis Period**: Last 7 days
**Total Alerts Fired**: 1,247
**Unique Alert Rules**: 45
**Actionable Alerts**: 23%

### Top Noisy Alerts

| Alert | Fires/Day | Actionable | Recommendation |
|-------|-----------|------------|----------------|
| HighCPUUsage | 89 | 5% | Increase threshold to 90% |
| PodRestarts | 45 | 12% | Add rate limit (3 in 10min) |
| DiskSpace80 | 32 | 8% | Increase to 85% |

### Recommended Changes

**1. HighCPUUsage Alert**
Current: CPU > 80% for 1m
Problem: Brief spikes trigger constantly

```yaml
# Recommended
expr: avg(cpu_usage) > 90
for: 5m
```

**2. Alert Grouping**
Group these related alerts:
- DatabaseConnectionError
- DatabaseSlowQuery
- DatabaseReplicationLag

Into: `DatabaseHealth` alert group

### Silence Recommendations
- Silence `DiskSpace80` during nightly backup (02:00-03:00)
- Silence `HighMemory` during deployments
```

---

## slo-guardian

Monitors Service Level Objective (SLO) compliance and error budget consumption.

### Usage

```bash
# Check SLO status
aofctl run agent library://observability/slo-guardian \
  --prompt "What is the current SLO status for the API service?"

# Analyze error budget
aofctl run agent library://observability/slo-guardian \
  --prompt "How much error budget remains for checkout-service?"
```

### Capabilities

- Tracks SLI metrics (latency, availability, throughput)
- Calculates error budget burn rate
- Predicts SLO breaches
- Alerts on budget exhaustion
- Generates SLO reports
- Recommends SLO targets

### Example Output

```markdown
## SLO Status Report: api-service

### Current SLOs

| SLO | Target | Current | Status |
|-----|--------|---------|--------|
| Availability | 99.9% | 99.87% | AT RISK |
| Latency p99 | 200ms | 185ms | HEALTHY |
| Error Rate | <0.1% | 0.08% | HEALTHY |

### Error Budget Analysis

**Availability SLO (99.9%)**
- Monthly Budget: 43.2 minutes downtime
- Budget Consumed: 38.5 minutes (89%)
- Budget Remaining: 4.7 minutes (11%)
- Days Remaining: 8

**Burn Rate**: 2.3x normal
Current pace will exhaust budget in 2 days.

### Recent Incidents Affecting Budget

| Time | Duration | Impact | Budget Used |
|------|----------|--------|-------------|
| Dec 20 14:32 | 12min | Full outage | 12 min |
| Dec 18 09:15 | 8min | Degraded | 8 min |
| Dec 15 22:00 | 18min | Partial outage | 18 min |

### Recommendations

1. **URGENT**: Address root cause of Dec 20 incident
2. Consider reducing deployment frequency until budget recovers
3. Enable canary deployments to catch issues earlier
4. Add circuit breaker for downstream dependency
```

---

## dashboard-generator

Auto-generates monitoring dashboards based on service metrics.

### Usage

```bash
# Generate dashboard for a service
aofctl run agent library://observability/dashboard-generator \
  --prompt "Create a monitoring dashboard for the payment-service"

# Generate golden signals dashboard
aofctl run agent library://observability/dashboard-generator \
  --prompt "Create a golden signals dashboard for namespace production"
```

### Capabilities

- Discovers available metrics
- Creates golden signals dashboards
- Generates RED/USE method dashboards
- Auto-detects service relationships
- Creates custom panels
- Exports Grafana JSON

### Example Output

```markdown
## Dashboard Generated: payment-service

### Dashboard Panels

**Row 1: Golden Signals**
- Request Rate (RPS)
- Error Rate (%)
- Latency Percentiles (p50, p95, p99)
- Saturation (CPU, Memory)

**Row 2: Business Metrics**
- Transactions per minute
- Transaction value ($)
- Payment success rate
- Fraud detection rate

**Row 3: Dependencies**
- Database latency
- External API latency
- Cache hit ratio
- Queue depth

### Grafana JSON Export

```json
{
  "dashboard": {
    "title": "Payment Service",
    "tags": ["payment", "production"],
    "panels": [
      {
        "title": "Request Rate",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(http_requests_total{service=\"payment\"}[5m])"
          }
        ]
      }
    ]
  }
}
```

### Import Instructions
1. Open Grafana → Dashboards → Import
2. Paste the JSON above
3. Select data source: Prometheus
4. Click Import
```

---

## log-analyzer

Analyzes logs for patterns, anomalies, and error correlation.

### Usage

```bash
# Analyze recent errors
aofctl run agent library://observability/log-analyzer \
  --prompt "Analyze errors in the api-server logs from the last hour"

# Find patterns
aofctl run agent library://observability/log-analyzer \
  --prompt "What patterns correlate with the latency spike at 14:30?"
```

### Capabilities

- Pattern detection in log streams
- Error clustering and categorization
- Anomaly detection
- Correlation with metrics/events
- Root cause hints
- Log query optimization

### Tools Used
- `loki_query` - Grafana Loki
- `elasticsearch_query` - Elasticsearch
- `aws_logs` - CloudWatch Logs
- `gcp_logging` - Google Cloud Logging
- `kubectl` - Kubernetes logs

### Example Output

```markdown
## Log Analysis: api-server (last 1 hour)

### Summary
- **Total Log Lines**: 245,890
- **Error Lines**: 1,247 (0.5%)
- **Warning Lines**: 3,456 (1.4%)
- **Unique Error Patterns**: 8

### Top Error Patterns

| Pattern | Count | First Seen | Last Seen |
|---------|-------|------------|-----------|
| Connection timeout to db | 456 | 14:32:15 | 14:58:23 |
| Rate limit exceeded | 312 | 14:35:00 | 15:02:45 |
| Invalid JWT token | 234 | 14:00:00 | 15:15:00 |
| Null pointer exception | 89 | 14:45:12 | 14:52:33 |

### Error Correlation

The spike in "Connection timeout to db" errors at 14:32 correlates with:
- Database CPU spike to 95% (14:31:45)
- Increased p99 latency (14:32:00)
- Alert: DatabaseHighCPU fired (14:32:30)

### Root Cause Hypothesis

Database became overloaded at 14:31, causing connection timeouts.
Likely cause: Long-running query or missing index.

### Recommended Queries

```logql
# Find slow database queries
{app="api-server"} |~ "query.*duration" | duration > 1s

# Trace affected requests
{app="api-server"} |= "Connection timeout" | json | request_id != ""
```
```

---

## trace-investigator

Investigates distributed traces to identify performance bottlenecks.

### Usage

```bash
# Investigate slow requests
aofctl run agent library://observability/trace-investigator \
  --prompt "Why are checkout requests taking 5+ seconds?"

# Analyze trace for specific request
aofctl run agent library://observability/trace-investigator \
  --prompt "Analyze trace ID abc123xyz"
```

### Capabilities

- Trace analysis and visualization
- Latency breakdown by service
- Critical path identification
- Service dependency mapping
- Bottleneck detection
- Comparison with baseline

### Example Output

```markdown
## Trace Investigation: Slow Checkout Requests

### Trace Summary
- **Trace ID**: abc123xyz
- **Total Duration**: 5,234ms
- **Spans**: 23
- **Services**: 7

### Critical Path Analysis

```
[checkout-api] 5234ms total
├── [inventory-service] 2100ms (40%)  ← BOTTLENECK
│   └── [database] 1950ms
├── [payment-service] 1800ms (34%)
│   ├── [fraud-check] 1200ms
│   └── [payment-gateway] 580ms
├── [notification-service] 890ms (17%)
└── [order-service] 444ms (9%)
```

### Bottleneck Identified

**Service**: inventory-service
**Operation**: checkStock
**Duration**: 2100ms (target: <200ms)

**Root Cause**: Database query in inventory-service is slow
- Query: `SELECT * FROM inventory WHERE sku IN (...)`
- Missing index on `sku` column
- 15,000 rows scanned

### Recommendations

1. Add index on inventory.sku
   ```sql
   CREATE INDEX idx_inventory_sku ON inventory(sku);
   ```

2. Implement caching for inventory lookups
3. Consider async inventory check for non-blocking checkout
```

---

## newrelic-ops

Comprehensive New Relic agent for NRQL queries, incident response, and entity discovery.

### Usage

```bash
# Query metrics with NRQL
aofctl run agent library://observability/newrelic-ops \
  --prompt "Check API error rate for the last hour"

# Investigate incidents
aofctl run agent library://observability/newrelic-ops \
  --prompt "List active incidents and acknowledge critical ones"

# Discover entities
aofctl run agent library://observability/newrelic-ops \
  --prompt "Find all production hosts with alertSeverity CRITICAL"
```

### Capabilities

- Execute NRQL queries for metrics, logs, and traces
- List and manage alert policies
- List and acknowledge incidents
- Search entities by type, tags, and alert status
- Query detailed metric timeslices

### Tools Used

- `newrelic_nrql_query` - Execute NRQL queries
- `newrelic_alerts_list` - List alert policies
- `newrelic_incidents_list` - List incidents
- `newrelic_entity_search` - Search entities
- `newrelic_metrics_query` - Query metrics
- `newrelic_incident_ack` - Acknowledge incidents

### Example Output

```markdown
## New Relic Analysis: API Service

### Current Status
- **Active Incidents**: 2
- **Alerting Entities**: 3
- **Error Rate**: 2.3% (above 1% threshold)

### NRQL Query Results

Query: `SELECT percentage(count(*), WHERE error IS true) FROM Transaction WHERE appName = 'api' SINCE 1 hour ago TIMESERIES`

| Time | Error Rate |
|------|------------|
| 14:00 | 0.8% |
| 14:15 | 1.2% |
| 14:30 | 2.3% ← Spike |
| 14:45 | 2.1% |

### Correlated Incidents

| Issue ID | Priority | Title | Status |
|----------|----------|-------|--------|
| INC-123 | CRITICAL | High error rate | ACTIVATED |
| INC-124 | HIGH | Database latency | ACTIVATED |

### Recommended Actions

1. Acknowledge INC-123 and begin investigation
2. Query database metrics: `SELECT average(duration) FROM DatabaseQuery`
3. Check deployment events for correlation
```

### Agent Definition

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: newrelic-ops
  labels:
    category: observability
    platform: newrelic
spec:
  model: google:gemini-2.5-flash
  tools:
    - newrelic_nrql_query
    - newrelic_alerts_list
    - newrelic_incidents_list
    - newrelic_entity_search
    - newrelic_incident_ack
  environment:
    NEWRELIC_API_KEY: "${NEWRELIC_API_KEY}"
    NEWRELIC_ACCOUNT_ID: "${NEWRELIC_ACCOUNT_ID}"
    NEWRELIC_REGION: "${NEWRELIC_REGION:-us}"
```

> **Tool Reference**: See [New Relic Tools](../tools/newrelic.md) for detailed API documentation.

---

## splunk-analyst

Comprehensive Splunk agent for SPL queries, security analysis, and event management.

### Usage

```bash
# Search logs with SPL
aofctl run agent library://observability/splunk-analyst \
  --prompt "Search for error logs in the last hour"

# Security analysis
aofctl run agent library://observability/splunk-analyst \
  --prompt "List all fired security alerts"

# Run saved searches
aofctl run agent library://observability/splunk-analyst \
  --prompt "Run the daily-error-report saved search"
```

### Capabilities

- Execute SPL (Search Processing Language) queries
- Monitor and list fired alerts
- Run saved searches on-demand
- Send events via HTTP Event Collector
- Discover available indexes

### Tools Used

- `splunk_search` - Execute SPL queries
- `splunk_alerts_list` - List fired alerts
- `splunk_saved_searches` - List saved searches
- `splunk_saved_search_run` - Run saved searches
- `splunk_hec_send` - Send events via HEC
- `splunk_indexes_list` - List indexes

### Example Output

```markdown
## Splunk Security Analysis

### Alert Summary (Last 24 Hours)
- **Total Fired Alerts**: 47
- **Critical**: 3
- **High**: 12
- **Medium**: 32

### Top Alerts by Frequency

| Alert Name | Fires | Severity |
|------------|-------|----------|
| Failed SSH Login | 23 | HIGH |
| Suspicious Network Traffic | 12 | CRITICAL |
| Privilege Escalation | 8 | HIGH |

### SPL Query Results

Query: `index=security action=failure | stats count by user | where count > 10`

| User | Failed Attempts |
|------|-----------------|
| admin | 45 |
| service_account | 23 |
| unknown | 156 ← Suspicious |

### Security Recommendations

1. **URGENT**: Investigate 156 failed logins from unknown user
2. Review `admin` account for brute force attempts
3. Enable MFA for privileged accounts
4. Block suspicious source IPs
```

### Agent Definition

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: splunk-analyst
  labels:
    category: observability
    platform: splunk
spec:
  model: google:gemini-2.5-flash
  tools:
    - splunk_search
    - splunk_alerts_list
    - splunk_saved_searches
    - splunk_saved_search_run
    - splunk_hec_send
    - splunk_indexes_list
  environment:
    SPLUNK_BASE_URL: "${SPLUNK_BASE_URL}"
    SPLUNK_TOKEN: "${SPLUNK_TOKEN}"
    SPLUNK_HEC_URL: "${SPLUNK_HEC_URL}"
    SPLUNK_HEC_TOKEN: "${SPLUNK_HEC_TOKEN}"
```

> **Tool Reference**: See [Splunk Tools](../tools/splunk.md) for detailed API documentation.

---

## Environment Setup

```bash
# Grafana
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_TOKEN=your-api-token

# Prometheus
export PROMETHEUS_URL=https://prometheus.example.com

# Datadog
export DD_API_KEY=your-api-key
export DD_APP_KEY=your-app-key
export DD_SITE=datadoghq.com

# New Relic
export NEWRELIC_API_KEY=NRAK-xxxxxxxxxxxx
export NEWRELIC_ACCOUNT_ID=1234567
export NEWRELIC_REGION=us  # or eu

# Splunk
export SPLUNK_BASE_URL=https://splunk.example.com:8089
export SPLUNK_TOKEN=your-auth-token
export SPLUNK_HEC_URL=https://splunk.example.com:8088
export SPLUNK_HEC_TOKEN=your-hec-token

# Loki
export LOKI_URL=https://loki.example.com

# AWS CloudWatch
export AWS_REGION=us-east-1
export AWS_PROFILE=production

# Elasticsearch
export ELASTICSEARCH_URL=https://elasticsearch.example.com
```

## Next Steps

- [Incident Agents](./incident.md) - Respond to incidents
- [CI/CD Agents](./cicd.md) - Pipeline optimization
- [SLO Concepts](../concepts.md#slos) - Understanding SLOs
