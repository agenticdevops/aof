# Tutorial: Incident Response Automation with AOF

Build an end-to-end automated incident response pipeline that detects PagerDuty/Opsgenie alerts, investigates root causes, performs automated triage, and generates comprehensive postmortems—all without manual intervention.

**What you'll build:**
- PagerDuty/Opsgenie webhook integration with signature verification
- Multi-agent incident response pipeline
- Automated triage, investigation, and postmortem generation
- Integration with observability tools (Grafana, Prometheus, Loki)

**What you'll learn:**
- Event-driven automation with triggers
- Multi-agent fleet orchestration
- Observability tool integration
- Automated documentation workflows

**Time estimate:** 30 minutes

**Prerequisites:**
- `aofctl` installed ([Installation Guide](https://docs.aof.sh))
- PagerDuty or Opsgenie account
- Kubernetes cluster with observability stack (Prometheus/Grafana/Loki)
- Google Gemini API key (free tier: `export GEMINI_API_KEY=your-key`)

## Architecture Overview

```
PagerDuty/Opsgenie Alert
          ↓
    [Webhook Trigger] ──→ Signature Verification
          ↓
    [Incident Responder] ──→ Initial Triage (Severity, Blast Radius)
          ↓
    [RCA Investigator] ──→ 5 Whys Analysis (Logs, Metrics, Timeline)
          ↓
    [Postmortem Writer] ──→ Google SRE-style Report
          ↓
    [Store in Git] ──→ docs/postmortems/INC-2024-XXX.md
```

## Step 1: Set Up Observability Connections

First, configure connections to your observability tools so agents can query metrics and logs.

Create `config/observability.env`:

```bash
# Prometheus for metrics
export PROMETHEUS_URL=http://prometheus.monitoring.svc.cluster.local:9090

# Loki for logs
export LOKI_URL=http://loki.monitoring.svc.cluster.local:3100

# Grafana for dashboards
export GRAFANA_URL=http://grafana.monitoring.svc.cluster.local:3000
export GRAFANA_API_KEY=your-grafana-api-key

# Kubernetes access
export KUBECONFIG=$HOME/.kube/config
```

**Test your connections:**

```bash
# Load environment
source config/observability.env

# Test Prometheus
curl "$PROMETHEUS_URL/api/v1/query?query=up"

# Test Loki
curl "$LOKI_URL/ready"

# Test Grafana
curl -H "Authorization: Bearer $GRAFANA_API_KEY" \
  "$GRAFANA_URL/api/health"

# Test kubectl
kubectl cluster-info
```

**Expected output:** All commands should return successful responses.

## Step 2: Configure PagerDuty Webhook Integration

### Create Generic Webhook (v3) in PagerDuty

1. **Navigate to Integrations:**
   - Go to **Integrations** → **Generic Webhooks (v3)**
   - Click **+ New Webhook**

2. **Configure webhook:**
   - **Webhook URL:** `https://your-domain.com/webhook/pagerduty`
   - **Scope Type:** Account (for all services) or Service (specific service)
   - **Event Subscription:** Select events to receive
     - ✅ `incident.triggered`
     - ✅ `incident.acknowledged`
     - ✅ `incident.escalated`
     - ✅ `incident.resolved`

3. **Save and copy credentials:**
   - Copy the **Webhook Secret** (for signature verification)
   - Generate a **REST API Token** (for adding notes to incidents)
     - Navigate to **API Access** → **Create New API Key**
     - Grant permissions: `Read/Write` on Incidents

4. **Add to environment:**

```bash
# Add to config/observability.env
export PAGERDUTY_WEBHOOK_SECRET=whsec_xxx...
export PAGERDUTY_API_TOKEN=u+xxx...
export PAGERDUTY_FROM_EMAIL=aof@yourcompany.com
```

### Alternative: Opsgenie Configuration

If using Opsgenie instead:

1. **Create API Integration:**
   - Go to **Settings** → **Integrations** → **API**
   - Create new **Incoming Webhook** integration

2. **Configure webhook:**
   - Copy webhook URL: `https://api.opsgenie.com/v2/integrations/xxx`
   - Copy API key for verification

3. **Add to environment:**

```bash
export OPSGENIE_API_KEY=xxx
export OPSGENIE_WEBHOOK_URL=https://api.opsgenie.com/v2/integrations/xxx
```

## Step 3: Create the PagerDuty Trigger

Create `triggers/pagerduty-incidents.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-production-incidents
  labels:
    platform: pagerduty
    environment: production
    team: sre

spec:
  # Platform configuration
  type: PagerDuty

  config:
    # Webhook endpoint path
    path: /webhook/pagerduty

    # Authentication
    webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
    api_token: ${PAGERDUTY_API_TOKEN}

    # Bot name for incident notes
    bot_name: "aof-incident-bot"

    # Filter by event types (optional)
    event_types:
      - incident.triggered
      - incident.acknowledged
      - incident.escalated

    # Filter by specific services (optional)
    # Get service IDs from PagerDuty: Services → [Service] → URL
    allowed_services:
      - PXYZ123  # Production API
      - PXYZ456  # Payment Service

    # Filter by teams (optional)
    allowed_teams:
      - P456DEF  # Infrastructure Team

    # Only process P1 and P2 incidents (optional)
    # Values: P1 (highest) to P5 (lowest)
    min_priority: "P2"

    # Only process high urgency incidents (optional)
    # Values: "high" or "low"
    min_urgency: "high"

  # Route to incident response fleet
  agent: incident-response-fleet

  # Enable the trigger
  enabled: true
```

**Key configuration options:**

- **`webhook_secret`**: Required for HMAC-SHA256 signature verification
- **`api_token`**: Optional - enables agents to add notes to incidents
- **`event_types`**: Filter which incident events to process
- **`allowed_services`**: Process only specific PagerDuty services
- **`min_priority`**: Ignore low-priority incidents (P3-P5)
- **`min_urgency`**: Ignore low-urgency incidents

## Step 4: Deploy the Incident Response Fleet

AOF provides pre-built library agents for incident response. We'll create a fleet that chains them together.

Create `fleets/incident-response-fleet.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: incident-response-fleet
  labels:
    category: incident
    team: sre

spec:
  # Fleet coordination mode
  coordination:
    mode: sequential  # Run agents in order

  # Fleet members
  agents:
    # 1. First Responder - Initial triage
    - name: incident-responder
      ref: library/incident/incident-responder.yaml
      role: coordinator

    # 2. RCA Investigator - Deep analysis
    - name: rca-investigator
      ref: library/incident/rca-investigator.yaml
      role: specialist

    # 3. Postmortem Writer - Documentation
    - name: postmortem-writer
      ref: library/incident/postmortem-writer.yaml
      role: specialist

  # Shared configuration
  shared:
    # Observability endpoints (all agents share these)
    env:
      PROMETHEUS_URL: ${PROMETHEUS_URL}
      LOKI_URL: ${LOKI_URL}
      GRAFANA_URL: ${GRAFANA_URL}

    # Shared memory for cross-agent context
    memory:
      type: in-memory
      namespace: incident-response

    # Common tools available to all agents
    tools:
      - kubectl
      - prometheus_query
      - loki_query
      - grafana_query
```

**How the fleet works:**

1. **incident-responder**: Receives the PagerDuty webhook, performs initial triage
   - Classifies severity (P0-P4)
   - Determines blast radius
   - Gathers initial context
   - Creates incident timeline

2. **rca-investigator**: Performs deep root cause analysis
   - Applies 5 Whys technique
   - Reconstructs timeline from logs/metrics
   - Identifies contributing factors
   - Provides evidence-based conclusions

3. **postmortem-writer**: Generates comprehensive documentation
   - Creates Google SRE-style postmortem
   - Quantifies impact from metrics
   - Extracts action items
   - Formats as markdown for git commit

## Step 5: Deploy the System

```bash
# 1. Load environment variables
source config/observability.env

# 2. Deploy the fleet
aofctl apply -f fleets/incident-response-fleet.yaml

# 3. Deploy the trigger
aofctl apply -f triggers/pagerduty-incidents.yaml

# 4. Start the trigger server (daemon mode)
aofctl serve \
  --config triggers/pagerduty-incidents.yaml \
  --port 8080

# 5. Verify deployment
aofctl get triggers
aofctl get fleets
```

**Expected output:**

```
TRIGGER                             PLATFORM      AGENT                       STATUS
pagerduty-production-incidents      pagerduty     incident-response-fleet     active

FLEET                              AGENTS    COORDINATION    STATUS
incident-response-fleet            3         sequential      ready
```

## Step 6: Expose Webhook to the Internet

For PagerDuty to send webhooks, you need a public URL.

### Option A: Production (Load Balancer)

```bash
# Create Kubernetes service
kubectl expose deployment aof-trigger-server \
  --type=LoadBalancer \
  --port=443 \
  --target-port=8080 \
  --name=aof-webhooks

# Get external IP
kubectl get svc aof-webhooks

# Configure in PagerDuty
# Webhook URL: https://<EXTERNAL-IP>/webhook/pagerduty
```

### Option B: Development (ngrok)

```bash
# Start ngrok tunnel
ngrok http 8080

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
# Configure in PagerDuty:
# Webhook URL: https://abc123.ngrok.io/webhook/pagerduty
```

### Option C: Production (Ingress)

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: aof-webhooks
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - aof.yourcompany.com
      secretName: aof-tls
  rules:
    - host: aof.yourcompany.com
      http:
        paths:
          - path: /webhook
            pathType: Prefix
            backend:
              service:
                name: aof-trigger-server
                port:
                  number: 8080
```

```bash
kubectl apply -f ingress.yaml

# Webhook URL: https://aof.yourcompany.com/webhook/pagerduty
```

## Step 7: Test End-to-End

### Test with a Real Incident

Trigger a test incident in PagerDuty:

```bash
# Create a test incident via PagerDuty API
curl -X POST https://api.pagerduty.com/incidents \
  -H "Authorization: Token token=${PAGERDUTY_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -H "From: ${PAGERDUTY_FROM_EMAIL}" \
  -d '{
    "incident": {
      "type": "incident",
      "title": "High CPU usage on api-deployment",
      "service": {
        "id": "PXYZ123",
        "type": "service_reference"
      },
      "urgency": "high",
      "body": {
        "type": "incident_body",
        "details": "CPU usage exceeded 90% for 5 minutes"
      }
    }
  }'
```

### Watch the Automation Flow

```bash
# Watch trigger logs
aofctl logs trigger pagerduty-production-incidents --follow

# Watch fleet execution
aofctl logs fleet incident-response-fleet --follow
```

**Expected flow:**

```
[14:05:01] Received PagerDuty webhook: incident.triggered
[14:05:01] Signature verified successfully
[14:05:02] Event: incident.triggered for incident #1234
[14:05:02] Routing to fleet: incident-response-fleet

[14:05:03] AGENT: incident-responder (STARTED)
[14:05:05] ✓ Severity classified: P2 (Medium)
[14:05:05] ✓ Blast radius: ~500 users, api-service degraded
[14:05:07] ✓ Initial findings: High CPU on api-deployment pods
[14:05:08] ✓ Recommended action: Scale deployment to 10 replicas
[14:05:08] AGENT: incident-responder (COMPLETED)

[14:05:09] AGENT: rca-investigator (STARTED)
[14:05:11] ✓ Timeline reconstructed (30 events)
[14:05:15] ✓ 5 Whys analysis completed
[14:05:16] ✓ Root cause: Unoptimized batch job saturating CPU
[14:05:18] ✓ Contributing factors: No resource limits, no HPA
[14:05:18] AGENT: rca-investigator (COMPLETED)

[14:05:19] AGENT: postmortem-writer (STARTED)
[14:05:22] ✓ Postmortem generated (2,500 words)
[14:05:23] ✓ Impact metrics calculated from Prometheus
[14:05:24] ✓ 6 action items extracted
[14:05:25] ✓ Saved to: docs/postmortems/INC-2024-1234.md
[14:05:25] AGENT: postmortem-writer (COMPLETED)

[14:05:26] ✓ Note added to PagerDuty incident
[14:05:26] FLEET EXECUTION COMPLETED (25 seconds)
```

### Verify the Output

Check the PagerDuty incident for agent notes:

1. Open the incident in PagerDuty
2. Scroll to **Notes** section
3. You should see a note from **aof-incident-bot**:

```markdown
🚨 INCIDENT TRIAGE

Severity: P2
Status: INVESTIGATING
Blast Radius: ~500 users | 1 service affected
Started: 2024-12-23T14:05:01Z

## Summary
High CPU usage detected on api-deployment in production.
Analysis shows unoptimized batch job consuming 95% CPU.

## Impact
- Users: ~500 active users experiencing slow response times
- Services: api-service degraded (P95 latency: 50ms → 3000ms)
- Revenue: Estimated $200/hour if not resolved

## Initial Findings
- CPU usage climbed to 95% at 14:00 UTC
- Batch job started at 13:55 UTC (cron schedule)
- No resource limits configured on deployment
- No Horizontal Pod Autoscaler configured

## Recommended Actions
1. **Immediate**: Scale api-deployment to 10 replicas
2. **Short-term**: Add resource limits and HPA
3. **Long-term**: Optimize batch job queries

## Relevant Context
- Dashboard: http://grafana.example.com/d/api-dashboard
- Runbook: https://wiki.example.com/runbooks/high-cpu
- Similar Incident: INC-2024-0987 (3 weeks ago)

## Timeline
[14:00] CPU usage climbed to 95%
[14:02] API latency increased to 3000ms
[14:05] Incident triggered from PagerDuty
[14:05] Initial triage completed
```

### Check the Generated Postmortem

```bash
# View the generated postmortem
cat docs/postmortems/INC-2024-1234.md
```

**Expected format:**

```markdown
# Postmortem: High CPU Usage on API Deployment

**Incident ID**: INC-2024-1234
**Date**: 2024-12-23
**Authors**: AOF Postmortem Writer
**Status**: Final
**Severity**: P2

---

## Executive Summary

On December 23, 2024 at 14:00 UTC, the Production API experienced
degraded performance due to high CPU usage. The incident lasted
25 minutes and affected approximately 500 active users. The root
cause was an unoptimized batch job running full-table scans against
the database. We resolved it by scaling the deployment and
optimizing the batch job queries.

## Impact

- **User Impact**: ~500 users (5% of active users)
- **Duration**: 25 minutes (14:00 - 14:25 UTC)
- **Affected Services**: Production API, Batch Processing
- **Revenue Impact**: ~$200 (estimated)
- **SLA Impact**: No breach (P95 < 5s for 99.5% of time)

### Impact Metrics

| Metric | Normal | During Incident | Peak Impact |
|--------|--------|----------------|-------------|
| Request Success Rate | 99.9% | 98.5% | 97.2% |
| P95 Latency | 50ms | 1800ms | 3000ms |
| Active Users | 10,000 | 9,500 | - |
| Error Rate | 0.1% | 1.5% | 2.8% |

## Timeline

All times in UTC.

### Detection

**14:00** Incident started (CPU climbed to 95%)
**14:02** First Datadog alert fired: High CPU Usage
**14:05** PagerDuty incident created: INC-2024-1234
**14:05** AOF incident-responder acknowledged and began triage

### Investigation

**14:05** incident-responder analyzed pod status and metrics
**14:06** Checked recent deployments (none in past 24 hours)
**14:07** Analyzed logs, discovered batch job correlation
**14:08** Formed hypothesis: Batch job causing CPU saturation
**14:09** rca-investigator confirmed hypothesis with 5 Whys

### Mitigation

**14:10** Scaled api-deployment from 5 to 10 replicas
**14:12** CPU usage dropped to 60% across pods
**14:15** Monitoring confirmed API latency normalized
**14:20** Stopped batch job temporarily
**14:25** Incident resolved

## Root Cause

### The 5 Whys

1. **Why was the API slow?**
   → CPU usage was at 95%, causing request queuing

2. **Why was CPU at 95%?**
   → A batch job was running CPU-intensive queries

3. **Why were the queries CPU-intensive?**
   → The batch job was doing full-table scans without indexes

4. **Why was it doing full-table scans?**
   → The query wasn't optimized and lacked proper indexes

5. **Why wasn't it optimized earlier?**
   → The batch job was added recently without performance review

### Root Cause Statement

The root cause was an unoptimized batch job running full-table
scans against the production database, saturating CPU resources.
This occurred because the batch job was deployed without
performance testing or resource limits. The issue was exacerbated
by the lack of Horizontal Pod Autoscaling, which would have
mitigated the impact.

## Contributing Factors

1. **No Resource Limits**: api-deployment had no CPU/memory limits,
   allowing batch job to consume all available resources
2. **No Horizontal Pod Autoscaler**: No automatic scaling based on
   CPU metrics
3. **Lack of Query Optimization**: Batch job queries weren't reviewed
   for performance before deployment

## Resolution

### Immediate Mitigation

To stop the bleeding, we:
1. Scaled api-deployment from 5 to 10 replicas (14:10)
2. Temporarily stopped the batch job (14:20)

This restored normal service within 15 minutes.

### Permanent Fix

The long-term solution involved:
1. Added database indexes for batch job queries (14:45)
2. Configured HPA for api-deployment (target: 70% CPU) (15:00)
3. Added resource limits to all deployments (15:30)
4. Rescheduled batch job to off-peak hours (16:00)

## What Went Well

- ✅ AOF automated triage within 60 seconds
- ✅ Root cause identified quickly (5 minutes)
- ✅ Mitigation applied automatically
- ✅ Monitoring and alerting worked as expected

## What Went Wrong

- ❌ Batch job deployed without performance testing
- ❌ No resource limits or HPA configured
- ❌ Delayed detection (issue started at 14:00, alert at 14:02)

## Lessons Learned

1. **Always performance test batch jobs**: Resource-intensive jobs
   must be tested under load before production deployment
2. **Resource limits are non-negotiable**: All deployments must have
   CPU/memory limits and HPA configured
3. **Automated triage is effective**: AOF reduced MTTR from typical
   15 minutes to 5 minutes

## Action Items

### Prevention (Stop This from Happening Again)

| Action | Owner | Status | Due Date |
|--------|-------|--------|----------|
| Add performance testing to CI/CD pipeline | DevOps | Open | 2024-12-30 |
| Enforce resource limits via OPA policies | SRE | Open | 2024-12-27 |
| Review all batch jobs for optimization | Backend | Open | 2025-01-10 |

### Detection (Find It Faster Next Time)

| Action | Owner | Status | Due Date |
|--------|-------|--------|----------|
| Add query performance monitoring | Database | Open | 2024-12-28 |
| Reduce alert threshold to 80% CPU | SRE | Completed | 2024-12-23 |

### Mitigation (Fix It Faster Next Time)

| Action | Owner | Status | Due Date |
|--------|-------|--------|----------|
| Enable AOF auto-scaling for all services | SRE | Open | 2025-01-05 |
| Create runbooks for common issues | SRE | Open | 2025-01-15 |

## Appendix

### Relevant Logs

```
[14:00:15] batch-job-abc123: Starting data aggregation...
[14:00:16] batch-job-abc123: Query: SELECT * FROM users WHERE...
[14:00:30] batch-job-abc123: Processing 1.2M records...
[14:02:45] api-pod-xyz789: WARN: Request queue depth: 150
[14:03:12] api-pod-xyz789: ERROR: Request timeout after 3000ms
```

### Metrics Graphs

- [Grafana Dashboard: API Performance](http://grafana.example.com/d/api/incident-2024-1234)
- [Prometheus: CPU Usage Graph](http://prometheus.example.com/graph?g0.expr=...)

### Related Incidents

- [INC-2024-0987]: Similar CPU issue caused by unoptimized queries (3 weeks ago)
- [INC-2024-0654]: Database performance degradation (2 months ago)

### References

- [Runbook: High CPU Troubleshooting](https://wiki.example.com/runbooks/high-cpu)
- [Database Query Optimization Guide](https://wiki.example.com/guides/query-optimization)
```

## Step 8: Customize for Your Environment

### Customize Incident Responder Prompts

Edit the library agent by creating a custom agent:

```yaml
# agents/custom-incident-responder.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: custom-incident-responder
  labels:
    category: incident
    tier: custom

spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.1

  # Base on library agent but customize for your needs
  description: "Custom incident responder for Acme Corp production"

  tools:
    - kubectl
    - prometheus_query
    - loki_query
    - grafana_query

  system_prompt: |
    You are Acme Corp's first-responder SRE agent.

    ## Your Mission
    When an incident is triggered:
    1. Acknowledge and classify severity using Acme's scale
    2. Determine blast radius (customers affected, revenue impact)
    3. Gather context from our observability stack
    4. Identify runbooks from https://wiki.acme.com/runbooks
    5. Notify #incidents Slack channel
    6. Create incident timeline

    ## Acme Severity Classification

    - **SEV-0 (Critical)**: Complete outage, all customers affected
      - Example: "API returns 503 for 100% of requests"
      - Response time: Immediate
      - Escalation: Page CEO, CTO, VP Eng

    - **SEV-1 (High)**: Major degradation, >25% customers affected
      - Example: "Checkout flow failing for 40% of users"
      - Response time: < 15 minutes
      - Escalation: Page on-call manager

    - **SEV-2 (Medium)**: Partial degradation, <25% customers
      - Example: "Search slow in EU region"
      - Response time: < 1 hour
      - Escalation: Notify team lead

    - **SEV-3 (Low)**: Minor issue, minimal customer impact
      - Example: "Admin dashboard widget broken"
      - Response time: Next business day
      - Escalation: None

    ## Acme-Specific Investigation

    1. **Check our services** (priority order):
       - api-gateway (namespace: production)
       - payment-service (namespace: production)
       - auth-service (namespace: production)
       - user-service (namespace: production)

    2. **Check our dependencies**:
       - PostgreSQL cluster (production-db)
       - Redis cache (production-cache)
       - Kafka (production-events)

    3. **Check our metrics**:
       - Revenue impact: `sum(rate(checkout_completed[5m]))`
       - User impact: `count(active_sessions)`
       - Error rate: `rate(http_requests_total{status=~"5.."}[5m])`

    4. **Link to dashboards**:
       - Production Overview: http://grafana.acme.com/d/prod-overview
       - Service Health: http://grafana.acme.com/d/service-health
       - Revenue Metrics: http://grafana.acme.com/d/revenue

    ## Output Format

    Always respond with:

    ```
    🚨 ACME INCIDENT TRIAGE

    Severity: SEV-[0-3]
    Status: INVESTIGATING
    Customer Impact: [X customers | $Y revenue/hour]
    Started: [timestamp]

    ## Summary
    [2-sentence description]

    ## Customer Impact
    - Customers: [affected count and percentage]
    - Services: [which services are down/degraded]
    - Revenue: [estimated $/hour impact]

    ## Initial Findings
    - [Key observation from logs]
    - [Key observation from metrics]
    - [Recent changes in last hour]

    ## Recommended Actions
    1. [Immediate action with owner]
    2. [Next investigation step]
    3. [Escalation if needed]

    ## Links
    - Dashboard: [Grafana link]
    - Runbook: [Wiki link if exists]
    - Slack: #incidents

    ## Timeline
    [HH:MM] Incident triggered
    [HH:MM] Initial triage completed
    ```

  memory: "File:./acme-incident-memory.json:100"
  max_context_messages: 30

  env:
    PROMETHEUS_URL: ${PROMETHEUS_URL}
    LOKI_URL: ${LOKI_URL}
    GRAFANA_URL: ${GRAFANA_URL}
```

Update your fleet to use the custom agent:

```yaml
# fleets/incident-response-fleet.yaml
spec:
  agents:
    - name: incident-responder
      ref: agents/custom-incident-responder.yaml  # Custom agent
      role: coordinator
    # ... rest unchanged
```

### Add Slack Notifications

Install Slack integration to get real-time updates:

```yaml
# agents/slack-notifier.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: slack-notifier

spec:
  model: google:gemini-2.5-flash
  temperature: 0.1

  description: "Send incident notifications to Slack"

  tools:
    - type: HTTP
      config:
        name: slack-webhook

  system_prompt: |
    You send concise incident notifications to Slack.

    Format all messages as:

    ```
    🚨 *&#123;&#123;severity&#125;&#125;* - &#123;&#123;title&#125;&#125;

    *Impact*: &#123;&#123;impact&#125;&#125;
    *Status*: &#123;&#123;status&#125;&#125;

    &lt;&#123;&#123;dashboard_link&#125;&#125;|View Dashboard> | &lt;&#123;&#123;incident_link&#125;&#125;|PagerDuty>
    ```

    Use Slack webhook to post: ${SLACK_WEBHOOK_URL}

  env:
    SLACK_WEBHOOK_URL: ${SLACK_WEBHOOK_URL}
```

Add to your fleet:

```yaml
spec:
  agents:
    - name: incident-responder
      ref: library/incident/incident-responder.yaml

    # Add Slack notifier
    - name: slack-notifier
      ref: agents/slack-notifier.yaml
      triggers:
        - on: incident-responder.complete
        - on: rca-investigator.complete
        - on: postmortem-writer.complete
```

## Step 9: Add Alert Fatigue Reduction

Use the alert-analyzer library agent to reduce noise:

```yaml
# fleets/smart-incident-response.yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: smart-incident-response

spec:
  agents:
    # 0. Alert Analyzer - Filter noise
    - name: alert-analyzer
      ref: library/incident/alert-analyzer.yaml
      role: filter

    # Only proceed if alert is actionable
    - name: incident-responder
      ref: library/incident/incident-responder.yaml
      role: coordinator
      conditions:
        - from: alert-analyzer
          when: ${alert-analyzer.output.actionable} == true

    # ... rest of pipeline
```

The alert-analyzer will:
- Deduplicate related alerts
- Identify alert storms (>10 alerts in 5 minutes)
- Filter known false positives
- Group related incidents

## Next Steps

### Production Hardening

1. **Add rate limiting:**

```yaml
spec:
  rate_limit:
    max_concurrent: 5      # Max 5 incidents at once
    queue_size: 20        # Queue up to 20
    timeout_seconds: 600  # 10 min timeout per incident
```

2. **Add persistent memory:**

```yaml
spec:
  shared:
    memory:
      type: redis
      config:
        url: redis://localhost:6379
        namespace: incident-response
```

3. **Add monitoring:**

```bash
# Prometheus metrics endpoint
aofctl serve --metrics-port 9090

# View metrics
curl localhost:9090/metrics | grep aof_incident
```

4. **Add circuit breaker:**

```yaml
spec:
  circuit_breaker:
    failure_threshold: 5     # Trip after 5 failures
    reset_timeout: 300       # Reset after 5 minutes
    half_open_requests: 2    # Test with 2 requests
```

### Advanced Integrations

- **[Jira Integration](./jira-automation)** - Auto-create Jira tickets for incidents
- **[Slack Bot](./slack-bot)** - Interactive incident management in Slack
- **[GitHub Automation](./github-automation)** - Auto-create PRs for fixes

### Learning Resources

- **[Agent Spec Reference](/docs/reference/agent-spec)** - Complete Agent YAML reference
- **[Fleet Spec Reference](/docs/reference/fleet-spec)** - Fleet orchestration patterns
- **[Trigger Spec Reference](/docs/reference/trigger-spec)** - Webhook configuration
- **[Agent Library](/docs/user-guide/agents/)** - Pre-built agents

---

**🎉 You've built an end-to-end incident response automation system!** Your on-call team now has an AI teammate that:

- ✅ Triages incidents in under 60 seconds
- ✅ Performs 5 Whys root cause analysis
- ✅ Generates comprehensive postmortems
- ✅ Never sleeps, never misses context

**Typical ROI:**
- **MTTR reduction**: 15 min → 5 min (67% faster)
- **On-call burden**: -40% (fewer manual investigations)
- **Postmortem completion**: 100% (vs. ~30% without automation)
- **Knowledge retention**: Perfect (every incident documented)
