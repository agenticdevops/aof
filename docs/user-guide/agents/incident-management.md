---
id: incident-management
title: Incident Management Agents
sidebar_label: Incident Management
format: md
---

# Incident Management Agents

The incident management agent library provides four specialized agents for handling production incidents from initial triage through postmortem documentation. These agents implement industry best practices from Google SRE, follow blameless postmortem principles, and integrate seamlessly with your existing incident response tools.

## Overview

The incident management library includes four coordinated agents:

| Agent | Purpose | Best For | Execution Time |
|-------|---------|----------|----------------|
| **incident-responder** | First-line triage and classification | Immediate incident response | 2-5 minutes |
| **alert-analyzer** | Alert correlation and deduplication | Reducing alert fatigue | 3-8 minutes |
| **rca-investigator** | Root cause analysis using 5 Whys | Deep incident investigation | 30-60 minutes |
| **postmortem-writer** | Blameless postmortem generation | Post-incident documentation | 10-20 minutes |

These agents can work independently or as a coordinated fleet for end-to-end incident management.

## Agent Descriptions

### 1. Incident Responder

**Location**: `library/incident/incident-responder.yaml`

The incident responder is your first-line agent for triaging incoming incidents from PagerDuty, Opsgenie, or other alerting systems.

#### Capabilities

- **Severity Classification**: Automatically classifies incidents as P0-P4 using consistent criteria
- **Blast Radius Determination**: Identifies affected users, services, and business impact
- **Initial Context Gathering**: Collects logs, metrics, and recent changes
- **Runbook Identification**: Matches incident patterns to existing runbooks
- **Incident Timeline Creation**: Starts timeline tracking from incident detection

#### When to Use

- Webhook triggers from PagerDuty or Opsgenie
- Slack commands (`/triage`, `/incident`)
- Manual incident investigation
- First response to any production alert

#### Configuration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: incident-responder
spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.1  # Low temperature for consistent triage

  tools:
    - kubectl           # Kubernetes resource inspection
    - prometheus_query  # Metrics analysis
    - loki_query        # Log correlation
    - grafana_query     # Dashboard context

  memory: "File:./incident-responder-memory.json:100"
  max_context_messages: 30
```

#### Example Usage

**Direct Execution:**
```bash
aofctl run agent library/incident/incident-responder \
  "API is returning 503 errors for 100% of requests in production"
```

**PagerDuty Webhook:**
```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  source:
    type: webhook
    config:
      path: /webhooks/pagerduty

  filter:
    expression: .payload.event == "incident.triggered"

  actions:
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: |
        Triage this incident:
        - Summary: &#123;&#123; .payload.incident.summary &#125;&#125;
        - Service: &#123;&#123; .payload.incident.service.name &#125;&#125;
        - Urgency: &#123;&#123; .payload.incident.urgency &#125;&#125;
```

#### Sample Output

```
🚨 INCIDENT TRIAGE

Severity: P0
Status: INVESTIGATING
Blast Radius: ~10,000 users | 3 services affected
Started: 2024-01-15 10:07:00 UTC

## Summary
The API gateway is returning 503 errors for 100% of requests. All backend
pods are marked as unhealthy by Kubernetes readiness probes.

## Impact
- Users: All production users cannot access the application
- Services: API gateway, backend-api, payment-service
- Revenue: Estimated $500/minute revenue loss

## Initial Findings
- All 6 backend-api pods failing readiness probes
- Prometheus shows database CPU at 98%
- Loki logs indicate "connection timeout to database"
- Recent deployment: backend-api v2.3.4 (deployed 10 min ago)

## Recommended Actions
1. IMMEDIATE: Rollback backend-api to v2.3.3 (SRE on-call)
2. Investigate database connection pool exhaustion
3. Escalate to database team if rollback doesn't resolve

## Relevant Context
- Dashboard: https://grafana.acme.com/d/api-health
- Runbook: https://runbooks.acme.com/api-503
- Similar Incident: INC-2024-003 (database connection pool)

## Timeline
[10:07] Incident triggered from PagerDuty
[10:08] Initial triage completed
[10:09] Rollback recommended to SRE on-call
```

### 2. Alert Analyzer

**Location**: `library/incident/alert-analyzer.yaml`

The alert analyzer reduces alert fatigue by correlating, deduplicating, and prioritizing alerts to surface actionable insights.

#### Capabilities

- **Alert Correlation**: Groups related alerts using temporal, spatial, and causal analysis
- **Deduplication**: Identifies and removes redundant alerts
- **Root Cause Identification**: Distinguishes root causes from symptoms
- **Business Impact Assessment**: Prioritizes based on user impact and business context
- **Alert Rule Improvement**: Suggests tuning and optimization

#### When to Use

- Scheduled runs (every 5-15 minutes)
- After major incidents to analyze alert patterns
- Alert fatigue troubleshooting
- Alert configuration optimization

#### Configuration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: alert-analyzer
spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.2  # Slightly higher for pattern recognition

  tools:
    - prometheus_query
    - grafana_query
    - datadog_metric_query

  memory: "File:./alert-analyzer-memory.json:200"
  max_context_messages: 50  # Large context for pattern learning
```

#### Example Usage

**Scheduled Analysis:**
```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: alert-analysis-cron
spec:
  source:
    type: schedule
    config:
      cron: "*/5 * * * *"  # Every 5 minutes

  actions:
    - type: agent
      ref: library/incident/alert-analyzer.yaml
      input: "Analyze alerts from the last 5 minutes"
```

**Manual Analysis:**
```bash
aofctl run agent library/incident/alert-analyzer \
  "Analyze all alerts from the last hour and identify patterns"
```

#### Sample Output

```
🔔 ALERT ANALYSIS

Period: 2024-01-15 10:00-11:00 UTC
Total Alerts: 47
Unique Issues: 3

## Critical Clusters (Immediate Action Required)

### Cluster 1: Database Connection Pool Exhaustion
Severity: P0
Alerts: 23
Services: api-gateway, backend-api, payment-service, user-service

Root Cause: Database connection pool saturated
Symptoms: API timeouts, pod readiness failures, 503 errors

Recommended Action:
1. Scale database connection pool from 100 to 200 connections
2. Restart affected pods to reset connection state

Alerts:
- [10:05] DatabaseConnectionPoolHigh (database)
- [10:06] APILatencyHigh (api-gateway)
- [10:07] PodNotReady (backend-api)
- [10:07] PodNotReady (payment-service)
- [10:08] HTTPErrorRateHigh (api-gateway)
- ... 18 more related alerts

---

### Cluster 2: High Memory Usage (us-east-1)
Severity: P2
Alerts: 8
Services: cache-redis nodes in us-east-1

Root Cause: Cache eviction rate high due to traffic spike
Symptoms: Increased cache misses, higher database load

Recommended Action:
1. Scale Redis cluster by 2 nodes
2. Review cache TTL settings

Alerts:
- [10:15] RedisMemoryHigh (cache-redis-1)
- [10:16] RedisMemoryHigh (cache-redis-2)
- [10:18] CacheHitRateLow (cache-redis-1)
- ... 5 more related alerts

## Low Priority Alerts (Can Wait)
- DiskUsageWarning (monitoring-server) - At 76%, threshold 75%, trend stable

## Noise (Recommend Tuning)
- PodCPUThrottling - Fired 16 times in last hour, never actionable
  Suggestion: Increase threshold from 50% to 70% or add business hours filter

## Alert Rule Improvements
1. Combine APILatencyHigh and HTTPErrorRateHigh into single SLO alert
2. Add dependency check to PodNotReady (don't alert if database is down)
3. Silence RedisMemoryHigh during known traffic spikes (marketing campaigns)
```

### 3. RCA Investigator

**Location**: `library/incident/rca-investigator.yaml`

The RCA investigator performs deep root cause analysis using the 5 Whys technique and systematic evidence gathering.

#### Capabilities

- **5 Whys Analysis**: Structured root cause investigation
- **Timeline Reconstruction**: Builds precise event timeline
- **Evidence Collection**: Gathers logs, metrics, and configuration changes
- **Hypothesis Testing**: Evaluates potential causes with supporting evidence
- **Contributing Factor Identification**: Identifies factors that exacerbated the issue

#### When to Use

- After incident resolution for deep investigation
- Complex incidents requiring systematic analysis
- Post-incident review preparation
- Manual investigation requests

#### Configuration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: rca-investigator
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192  # Large context for deep investigations
  temperature: 0.1

  tools:
    - kubectl
    - prometheus_query
    - loki_query
    - git

  memory: "File:./rca-investigator-memory.json:100"
  max_context_messages: 40
```

#### Example Usage

**Post-Incident Investigation:**
```bash
aofctl run agent library/incident/rca-investigator \
  "Investigate incident INC-2024-001: API 503 errors from 10:00-10:30 UTC"
```

**Fleet Coordination:**
```yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: incident-investigation
spec:
  agents:
    - name: responder
      ref: library/incident/incident-responder.yaml

    - name: investigator
      ref: library/incident/rca-investigator.yaml

  workflow:
    - step: triage
      agent: responder
      input: "&#123;&#123; .trigger.data &#125;&#125;"

    - step: deep-dive
      agent: investigator
      input: "Investigate: &#123;&#123; .steps.triage.output &#125;&#125;"
      condition: "&#123;&#123; .steps.triage.severity | in 'P0' 'P1' &#125;&#125;"
```

#### Sample Output

```
🔍 ROOT CAUSE ANALYSIS

Incident: API 503 Errors - Production Outage
Duration: 10:07:00 → 10:28:00 UTC (21 minutes)
Severity: P0

## Executive Summary
On January 15, 2024, the production API experienced a complete outage due to
database connection pool exhaustion triggered by a newly deployed batch job
running unoptimized queries. The incident affected all 10,000 active users
and was resolved by killing the batch job and scaling the connection pool.

## Timeline of Events

[09:55] Normal baseline established (API latency 50ms, 0.1% errors)
[10:00] Batch job "user-export" started via cron
[10:02] Database CPU climbed from 15% to 95%
[10:05] API latency increased to 3000ms
[10:06] Backend pods began failing readiness probes
[10:07] All pods marked unhealthy, 503 errors started
[10:07] PagerDuty incident triggered
[10:10] On-call SRE began investigation
[10:15] Root cause identified (batch job)
[10:16] Batch job terminated
[10:18] Database CPU returned to normal
[10:20] Pods became healthy
[10:22] 503 errors stopped
[10:28] Incident declared resolved

## The 5 Whys

Problem: API returning 503 errors for 100% of requests

1. Why are we getting 503 errors?
   → Because Kubernetes load balancer has no healthy backend pods
   Evidence: kubectl get pods shows 0/6 pods ready

2. Why are no pods healthy?
   → Because all pods are failing readiness probes
   Evidence: kubectl describe pod shows "Readiness probe failed"

3. Why are readiness probes failing?
   → Because the /health endpoint times out after 5 seconds
   Evidence: Pod logs show "GET /health timeout after 5000ms"

4. Why does /health timeout?
   → Because it queries the database, which is not responding
   Evidence: Prometheus shows database query latency >10s

5. Why is the database not responding?
   → Because connection pool is exhausted by batch job queries
   Evidence: Database logs show "max_connections (100) reached",
            batch job running full table scan on 50M row table

ROOT CAUSE: Batch job "user-export" running unoptimized full table scan
saturating database connection pool (100 connections), preventing API
health checks from completing within 5s timeout.

## Contributing Factors

1. **No Connection Pool Isolation**: Batch jobs share the same connection
   pool as the API, allowing them to starve the API of connections.

2. **Missing Query Timeout**: The batch job query had no timeout configured,
   allowing it to hold connections indefinitely.

3. **Inadequate Health Check**: The /health endpoint queries the database
   unnecessarily. A database connection issue causes all pods to fail.

## Evidence Summary

### Metrics
- Database CPU: Spiked from 15% to 95% at 10:02
- Database Connections: Maxed at 100/100 at 10:05
- API Latency P95: Increased from 50ms to 15000ms
- API Error Rate: Went from 0.1% to 100%
- Pod Ready Count: Dropped from 6/6 to 0/6

### Logs
- Database: "max_connections (100) reached" (10:05)
- Batch job: "SELECT * FROM users" (no WHERE clause)
- API: "health check timeout" (10:06-10:28)

### Changes
- Batch job "user-export" added to cron (deployed Jan 14)
- No recent API or infrastructure changes

## What Worked
- PagerDuty alert fired immediately when 503s started
- On-call SRE had access to all necessary tools
- Prometheus metrics clearly showed database as bottleneck
- Killing batch job immediately resolved the issue

## What Didn't Work
- No pre-deployment testing of batch job at scale
- Health check depends on database (single point of failure)
- No connection pool monitoring/alerting
- No query timeout on batch job

## Recommendations

### Immediate (Do Today)
1. Add connection pool monitoring and alerting
2. Remove database dependency from /health endpoint

### Short-term (This Week)
1. Add query timeout (30s) to all batch jobs
2. Create separate connection pool for batch jobs (max 20 connections)
3. Optimize user-export query with proper indexes and pagination
4. Add pre-production testing for batch jobs at scale

### Long-term (This Quarter)
1. Implement read replicas for batch job queries
2. Design health check strategy that doesn't depend on external services
3. Implement connection pool autoscaling
4. Add circuit breakers between API and database
```

### 4. Postmortem Writer

**Location**: `library/incident/postmortem-writer.yaml`

The postmortem writer generates comprehensive, blameless postmortem reports following Google SRE best practices.

#### Capabilities

- **Google SRE-Style Postmortems**: Follows industry standard template
- **Impact Quantification**: Extracts metrics-based impact from Prometheus
- **Timeline Construction**: Builds detailed timeline from logs and events
- **Blameless Writing**: Focuses on systems and processes, not individuals
- **Action Item Extraction**: Identifies concrete follow-up tasks

#### When to Use

- After RCA investigation completes
- Post-incident documentation
- Incident review meetings
- Learning library contributions

#### Configuration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: postmortem-writer
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192  # Large output for detailed reports
  temperature: 0.3  # Slightly creative for clear writing

  tools:
    - prometheus_query
    - loki_query

  memory: "File:./postmortem-writer-memory.json:50"
  max_context_messages: 20
```

#### Example Usage

**Standalone Execution:**
```bash
aofctl run agent library/incident/postmortem-writer \
  "Write postmortem for incident INC-2024-001 based on the RCA investigation"
```

**Fleet Integration:**
```yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: full-incident-lifecycle
spec:
  agents:
    - name: responder
      ref: library/incident/incident-responder.yaml

    - name: investigator
      ref: library/incident/rca-investigator.yaml

    - name: writer
      ref: library/incident/postmortem-writer.yaml

  workflow:
    - step: triage
      agent: responder
      input: "&#123;&#123; .trigger.data &#125;&#125;"

    - step: investigate
      agent: investigator
      input: "&#123;&#123; .steps.triage.output &#125;&#125;"

    - step: document
      agent: writer
      input: "&#123;&#123; .steps.investigate.output &#125;&#125;"
```

The postmortem writer generates a complete Markdown document ready to commit to your documentation repository or share with your team.

## Fleet Orchestration

### Complete Incident Response Fleet

Coordinate all four agents for end-to-end incident management:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: incident-response-complete
  labels:
    category: incident
    domain: sre

spec:
  agents:
    - name: triager
      ref: library/incident/incident-responder.yaml

    - name: analyzer
      ref: library/incident/alert-analyzer.yaml

    - name: investigator
      ref: library/incident/rca-investigator.yaml

    - name: documenter
      ref: library/incident/postmortem-writer.yaml

  workflow:
    # Step 1: Immediate triage
    - step: triage
      agent: triager
      input: "&#123;&#123; .trigger.data.incident &#125;&#125;"

    # Step 2: Correlate with recent alerts (parallel)
    - step: alert-context
      agent: analyzer
      input: "Analyze alerts from last 30 minutes related to: &#123;&#123; .steps.triage.summary &#125;&#125;"

    # Step 3: Deep investigation (only for P0/P1)
    - step: investigate
      agent: investigator
      input: |
        Investigate incident:
        Triage: &#123;&#123; .steps.triage.output &#125;&#125;
        Alert Context: &#123;&#123; .steps.alert-context.output &#125;&#125;
      condition: "&#123;&#123; .steps.triage.severity | in 'P0' 'P1' &#125;&#125;"

    # Step 4: Generate postmortem (after investigation)
    - step: postmortem
      agent: documenter
      input: |
        Write postmortem for:
        &#123;&#123; .steps.investigate.output &#125;&#125;
      depends_on:
        - investigate

  config:
    # Share memory across agents
    shared_memory: true

    # Timeout for entire fleet
    timeout: 3600  # 1 hour

    # Retry strategy
    retry:
      max_attempts: 3
      backoff: exponential
```

## Integration Examples

### PagerDuty Integration

Complete PagerDuty webhook integration with incident triage:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-webhook
spec:
  source:
    type: webhook
    config:
      path: /webhooks/pagerduty
      port: 8080

      # Verify PagerDuty signatures
      signature_header: X-PagerDuty-Signature
      signature_secret: "${PAGERDUTY_WEBHOOK_SECRET}"

  # Filter for incident.triggered events
  filter:
    expression: .payload.event == "incident.triggered"

  actions:
    # Run incident responder
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: |
        PagerDuty Incident:
        - ID: &#123;&#123; .payload.incident.id &#125;&#125;
        - Summary: &#123;&#123; .payload.incident.summary &#125;&#125;
        - Service: &#123;&#123; .payload.incident.service.name &#125;&#125;
        - Urgency: &#123;&#123; .payload.incident.urgency &#125;&#125;
        - URL: &#123;&#123; .payload.incident.html_url &#125;&#125;

      # Post triage result back to PagerDuty
      output:
        type: pagerduty_note
        incident_id: "&#123;&#123; .payload.incident.id &#125;&#125;"
        note: "&#123;&#123; .agent.output &#125;&#125;"
```

### Opsgenie Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: opsgenie-webhook
spec:
  source:
    type: webhook
    config:
      path: /webhooks/opsgenie
      port: 8080

  filter:
    expression: .action == "Create"

  actions:
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: |
        Opsgenie Alert:
        - Message: &#123;&#123; .alert.message &#125;&#125;
        - Priority: &#123;&#123; .alert.priority &#125;&#125;
        - Tags: &#123;&#123; .alert.tags | join ", " &#125;&#125;
```

### Slack Command Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: slack-triage-command
spec:
  source:
    type: slack_command
    config:
      command: /triage
      signing_secret: "${SLACK_SIGNING_SECRET}"

  actions:
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: "&#123;&#123; .command.text &#125;&#125;"

      # Post formatted response to Slack
      output:
        type: slack_message
        channel: "&#123;&#123; .command.channel_id &#125;&#125;"
        format: blocks  # Use Slack Block Kit
```

### Scheduled Alert Analysis

Run alert analyzer on a schedule to proactively reduce noise:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: alert-analysis-schedule
spec:
  source:
    type: schedule
    config:
      # Every 5 minutes
      cron: "*/5 * * * *"

  actions:
    - type: agent
      ref: library/incident/alert-analyzer.yaml
      input: "Analyze alerts from the last 5 minutes"

      # Save analysis to file
      output:
        type: file
        path: /var/log/aof/alert-analysis-&#123;&#123; .timestamp &#125;&#125;.json
```

## Customization Examples

### Custom Severity Thresholds

Override severity classification for your organization:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: acme-incident-responder
spec:
  # Inherit from library agent
  base: library/incident/incident-responder.yaml

  # Override system prompt section
  system_prompt_override: |
    ## Severity Classification (Acme Corp Custom)

    - **P0 (Critical)**: Revenue-impacting outage
      - >$1000/min revenue loss OR >50% user impact

    - **P1 (High)**: Major degradation
      - >$500/min revenue loss OR >20% user impact

    - **P2 (Medium)**: Partial degradation
      - <$500/min revenue loss OR <20% user impact

    - **P3 (Low)**: Minor issue with workaround

    - **P4 (Info)**: No user impact
```

### Custom Tools Integration

Add custom tools specific to your infrastructure:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: acme-rca-investigator
spec:
  base: library/incident/rca-investigator.yaml

  # Add custom tools
  tools:
    - kubectl
    - prometheus_query
    - loki_query
    - git
    - datadog_metric_query    # Custom: Datadog integration
    - splunk_search           # Custom: Splunk logs
    - acme_runbook_search     # Custom: Internal runbook DB

  env:
    DATADOG_API_KEY: "${DATADOG_API_KEY}"
    DATADOG_APP_KEY: "${DATADOG_APP_KEY}"
    SPLUNK_URL: "${SPLUNK_URL}"
    SPLUNK_TOKEN: "${SPLUNK_TOKEN}"
    RUNBOOK_DB_URL: "https://runbooks.acme.com/api"
```

### Custom Output Format

Customize postmortem format for your wiki system:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: acme-postmortem-writer
spec:
  base: library/incident/postmortem-writer.yaml

  system_prompt_append: |
    ## Acme Corp Custom Format

    After generating the postmortem, also create:

    1. Executive summary (max 3 sentences) for leadership
    2. Customer communication draft for support team
    3. Jira tickets for each action item in this format:
       ```
       Title: [Action item summary]
       Description: [Detailed description]
       Labels: incident, postmortem, &#123;&#123; .incident.severity &#125;&#125;
       Priority: &#123;&#123; .action.priority &#125;&#125;
       ```
```

## Best Practices

### 1. Use Consistent Environment Configuration

Store environment variables in a central configuration:

```bash
# .env.production
PROMETHEUS_URL=https://prometheus.acme.com
LOKI_URL=https://loki.acme.com
GRAFANA_URL=https://grafana.acme.com
PAGERDUTY_API_KEY=your-key-here
SLACK_WEBHOOK=your-webhook-url

# Run with environment
aofctl run agent library/incident/incident-responder \
  --env-file .env.production \
  "Triage incident..."
```

### 2. Enable Memory Persistence

Allow agents to learn from past incidents:

```yaml
spec:
  # Use persistent file-based memory
  memory: "File:/var/lib/aof/memory/incident-responder.json:500"

  # Or use SQLite for querying
  memory: "SQLite:/var/lib/aof/memory/incident-responder.db"
```

### 3. Monitor Agent Performance

Track agent execution metrics:

```bash
# View recent executions
aofctl get executions --agent incident-responder --limit 20

# Analyze token usage trends
aofctl analyze usage --agent incident-responder --timeframe 30d

# Get performance metrics
aofctl metrics agent incident-responder
```

### 4. Test Before Production

Always test agent configurations in non-production:

```bash
# Test with sample incident data
cat << EOF | aofctl run agent library/incident/incident-responder -
Simulate incident: API gateway returning 502 errors intermittently.
Affects: payment-api, user-api
Region: us-east-1
Started: 5 minutes ago
EOF
```

### 5. Version Control Agent Customizations

Store customized agents in version control:

```bash
# Repository structure
.aof/
├── agents/
│   └── incident/
│       ├── custom-responder.yaml
│       ├── custom-rca.yaml
│       └── custom-postmortem.yaml
├── triggers/
│   ├── pagerduty.yaml
│   └── slack-commands.yaml
├── fleets/
│   └── incident-response.yaml
└── .env.production
```

### 6. Implement Gradual Rollout

Test new configurations gradually:

```yaml
# Use canary deployment for new agent config
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incidents-canary
spec:
  source:
    type: webhook
    config:
      path: /webhooks/pagerduty

  # Route 10% of traffic to new config
  actions:
    - type: agent
      ref: library/incident/incident-responder-v2.yaml
      weight: 10

    - type: agent
      ref: library/incident/incident-responder.yaml
      weight: 90
```

## Troubleshooting

### Agent Not Producing Expected Output

**Check memory context:**
```bash
# View agent memory state
aofctl get memory incident-responder

# Clear memory if stale
aofctl clear memory incident-responder
```

**Verify tool access:**
```bash
# Test Prometheus connectivity
aofctl test tool prometheus_query --env-file .env.production

# Test Kubernetes access
kubectl get pods  # Verify kubectl works
```

### High Token Usage

**Monitor token consumption:**
```bash
# Get token usage breakdown
aofctl analyze tokens --agent incident-responder --timeframe 7d

# Optimize by reducing context
```

**Reduce max_tokens or context:**
```yaml
spec:
  max_tokens: 2048  # Reduce from 4096
  max_context_messages: 15  # Reduce from 30
```

### Slow Agent Execution

**Enable parallel tool execution:**
```yaml
spec:
  tools:
    - kubectl
    - prometheus_query
    - loki_query

  tool_config:
    parallel_execution: true  # Execute tools concurrently
    timeout: 30s
```

## Next Steps

- [Fleet Orchestration Guide](/docs/concepts/fleets) - Coordinate multiple agents
- [Core Concepts](/docs/concepts) - Learn fundamental AOF concepts
- [Built-in Tools](/docs/tools/builtin-tools) - Available tools for agents
- [Trigger Specification](/docs/reference/trigger-spec) - Automate agent execution

## Support

Need help with incident management agents?

- [GitHub Discussions](https://github.com/agenticdevops/aof/discussions)
- [Discord Community](https://discord.gg/aof)
- [Documentation](https://docs.aof.sh)
