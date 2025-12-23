# Incident Agents

Production-ready agents for incident response, investigation, and documentation.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [incident-commander](#incident-commander) | Coordinate response | Multiple |
| [rca-agent](#rca-agent) | Root cause analysis | kubectl, grafana, loki |
| [postmortem-writer](#postmortem-writer) | Generate postmortems | None (analysis) |
| [runbook-executor](#runbook-executor) | Execute runbooks | kubectl, shell |
| [escalation-manager](#escalation-manager) | Manage escalations | None (coordination) |

## incident-commander

Coordinates incident response activities, assigns roles, and tracks timeline.

### Usage

```bash
# Declare an incident
aofctl run agent library://incident/incident-commander \
  --prompt "Declare SEV2 incident: Payment API returning 500 errors"

# Get incident status
aofctl run agent library://incident/incident-commander \
  --prompt "What's the current status of INC-2024-1220-001?"
```

### Capabilities

- Incident declaration and severity assessment
- Role assignment (IC, comms, scribe)
- Timeline tracking
- Stakeholder notification
- Status page updates
- Post-incident handoff

### Example Output

```markdown
## Incident Declared: INC-2024-1220-001

**Title**: Payment API 500 Errors
**Severity**: SEV2 (Major - Customer Impact)
**Status**: INVESTIGATING
**Declared**: 2024-12-20 14:32 UTC
**Incident Commander**: @oncall-engineer

### Impact Assessment
- **Affected Service**: payment-api
- **User Impact**: ~15% of checkout attempts failing
- **Revenue Impact**: Estimated $X/hour
- **Regions**: US-East, EU-West

### Assigned Roles
| Role | Assignee | Status |
|------|----------|--------|
| Incident Commander | @oncall-engineer | Active |
| Communications | @platform-lead | Standby |
| Scribe | @sre-team | Active |
| Subject Expert | @payments-team | Paged |

### Initial Timeline
- 14:25 - Alert fired: PaymentAPI5xxRate
- 14:28 - On-call acknowledged
- 14:32 - Incident declared (SEV2)
- 14:33 - War room created: #inc-payment-500

### Next Actions
1. [ ] Investigate payment-api logs
2. [ ] Check recent deployments
3. [ ] Verify database health
4. [ ] Prepare customer communication

### Communication Channels
- Slack: #inc-payment-500
- Bridge: meet.google.com/abc-defg-hij
- Status Page: Investigating
```

---

## rca-agent

Performs root cause analysis by correlating events, logs, and metrics.

### Usage

```bash
# Investigate an incident
aofctl run agent library://incident/rca-agent \
  --prompt "Investigate the cause of the payment API outage at 14:30"

# Analyze specific symptoms
aofctl run agent library://incident/rca-agent \
  --prompt "What caused the database connection exhaustion?"
```

### Capabilities

- Event correlation across time/space
- Five Whys analysis
- Hypothesis testing
- Log and metric analysis
- Impact assessment
- Evidence collection

### Tools Used
- `kubectl` - Kubernetes state
- `grafana_dashboard_get` - Dashboards
- `grafana_query` - Metrics
- `loki_query` - Logs
- `datadog_log_query` - Datadog logs

### Example Output

```markdown
## Root Cause Analysis: Payment API Outage

**Incident**: INC-2024-1220-001
**Duration**: 14:30 - 15:15 UTC (45 minutes)
**Severity**: SEV2

### Executive Summary
The outage was caused by a database connection pool exhaustion triggered
by a new feature that held connections longer than expected.

### Timeline of Events

```
T-15min (14:15): Feature flag enabled for "async-payment-processing"
T-10min (14:20): Connection pool usage begins climbing (50% → 70%)
T-05min (14:25): First timeout errors in logs
T+00min (14:30): Alert fires, pool at 95%
T+05min (14:35): Pool exhausted, all requests failing
T+15min (14:45): Feature flag disabled
T+20min (14:50): Connections begin releasing
T+45min (15:15): Full recovery confirmed
```

### Five Whys Analysis

**Why 1**: Why did the payment API return 500 errors?
→ Database connections were exhausted

**Why 2**: Why were connections exhausted?
→ Connections weren't being released properly

**Why 3**: Why weren't connections released?
→ New async feature holds connection during long-running operation

**Why 4**: Why does the feature hold connections?
→ Missing connection.release() in async callback path

**Why 5**: Why wasn't this caught in testing?
→ Load testing didn't simulate production connection concurrency

### Root Cause
Missing `connection.release()` call in the async payment processing
feature's error handling path, causing connections to leak under load.

### Evidence
1. Connection pool metrics show linear growth after feature enabled
2. Error logs show "Connection pool exhausted" at 14:35
3. Feature flag toggle timestamp correlates with symptom start
4. Code review confirms missing release() in catch block

### Contributing Factors
1. Insufficient connection pool size (50 vs recommended 100)
2. No connection leak detection/alerting
3. Load testing didn't cover async error paths

### Recommendations
1. **Immediate**: Fix connection.release() in async handler
2. **Short-term**: Add connection pool exhaustion alert at 80%
3. **Medium-term**: Implement connection leak detection
4. **Long-term**: Add chaos testing for connection failures
```

---

## postmortem-writer

Generates comprehensive incident postmortems from investigation data.

### Usage

```bash
# Generate postmortem
aofctl run agent library://incident/postmortem-writer \
  --prompt "Generate postmortem for INC-2024-1220-001"

# Generate from notes
aofctl run agent library://incident/postmortem-writer \
  --prompt "Create postmortem from these incident notes: [paste notes]"
```

### Capabilities

- Structures postmortem document
- Extracts timeline from logs
- Identifies action items
- Calculates impact metrics
- Suggests preventive measures
- Formats for stakeholder review

### Example Output

```markdown
# Postmortem: Payment API Outage
**Date**: December 20, 2024
**Authors**: SRE Team, Payments Team
**Status**: Draft

## Incident Summary
| Field | Value |
|-------|-------|
| Incident ID | INC-2024-1220-001 |
| Duration | 45 minutes (14:30 - 15:15 UTC) |
| Severity | SEV2 |
| Impact | 15% checkout failure rate |
| Customers Affected | ~12,000 |
| Revenue Impact | $45,000 estimated |

## What Happened
On December 20, 2024, the payment API experienced a 45-minute outage
affecting approximately 15% of checkout attempts. The incident was
caused by a database connection pool exhaustion triggered by a newly
enabled feature.

## Timeline
| Time (UTC) | Event |
|------------|-------|
| 14:15 | Feature flag "async-payment-processing" enabled |
| 14:20 | Connection pool usage began increasing |
| 14:25 | First timeout errors appeared in logs |
| 14:30 | Alert fired, incident declared |
| 14:35 | Full outage - all requests failing |
| 14:45 | Feature flag disabled |
| 14:50 | Recovery began |
| 15:15 | Full service restored |

## Root Cause
Missing `connection.release()` call in async payment processing error
handling path caused database connections to leak under load.

## Impact
- **Customer Impact**: 12,000 customers experienced checkout failures
- **Revenue Impact**: Estimated $45,000 in lost/delayed transactions
- **SLA Impact**: 99.9% availability SLO breached (11 minutes of budget consumed)

## What Went Well
- Alert fired within 5 minutes of issue start
- On-call responded within 3 minutes
- Root cause identified within 15 minutes
- Feature flag allowed quick mitigation

## What Went Poorly
- Issue not caught in staging testing
- Connection pool metrics not monitored
- No automated rollback for connection exhaustion

## Action Items
| Priority | Action | Owner | Due Date |
|----------|--------|-------|----------|
| P0 | Fix connection leak in async handler | @payments-dev | Dec 21 |
| P1 | Add connection pool alert at 80% | @sre-team | Dec 22 |
| P2 | Add connection leak detection | @platform | Jan 5 |
| P2 | Update load test with async errors | @qa-team | Jan 10 |
| P3 | Document feature flag rollback procedure | @docs | Jan 15 |

## Lessons Learned
1. New features should include resource cleanup validation
2. Connection pool monitoring is critical infrastructure
3. Feature flags are valuable for quick mitigation

## Appendix
- [Incident Slack Thread](#)
- [Grafana Dashboard](#)
- [Related PR Fix](#)
```

---

## runbook-executor

Executes structured runbook procedures with safety checks.

### Usage

```bash
# Execute a runbook
aofctl run agent library://incident/runbook-executor \
  --prompt "Execute runbook: database-failover-procedure"

# Guided execution
aofctl run agent library://incident/runbook-executor \
  --prompt "Guide me through the pod-restart runbook for api-server"
```

### Capabilities

- Step-by-step execution
- Pre-flight safety checks
- Progress tracking
- Rollback on failure
- Execution logging
- Human-in-the-loop confirmation

### Tools Used
- `kubectl` - Kubernetes operations
- `shell` - Generic commands

### Example Output

```markdown
## Runbook Execution: pod-restart-graceful

**Runbook**: pod-restart-graceful
**Target**: deployment/api-server (namespace: production)
**Executor**: runbook-executor
**Started**: 2024-12-20 15:30:00 UTC

### Pre-Flight Checks

| Check | Status | Details |
|-------|--------|---------|
| Cluster Access | PASS | Connected to prod-cluster |
| Namespace Exists | PASS | production namespace found |
| Deployment Exists | PASS | api-server deployment found |
| Replicas | PASS | 5/5 pods ready |
| PDB Exists | PASS | min available: 3 |

### Execution Steps

**Step 1/5**: Cordon node for maintenance
```bash
# Skipped - not required for pod restart
```
Status: SKIPPED

**Step 2/5**: Scale down by 1 replica
```bash
kubectl scale deployment api-server -n production --replicas=4
```
Status: COMPLETED
Output: deployment.apps/api-server scaled

**Step 3/5**: Wait for termination
```bash
kubectl wait --for=delete pod -l app=api-server -n production --timeout=60s
```
Status: COMPLETED
Duration: 45 seconds

**Step 4/5**: Scale back up
```bash
kubectl scale deployment api-server -n production --replicas=5
```
Status: COMPLETED
Output: deployment.apps/api-server scaled

**Step 5/5**: Verify health
```bash
kubectl rollout status deployment api-server -n production
```
Status: COMPLETED
Output: deployment "api-server" successfully rolled out

### Execution Summary

- **Total Steps**: 5
- **Completed**: 4
- **Skipped**: 1
- **Failed**: 0
- **Duration**: 2 minutes 15 seconds

**Result**: SUCCESS

### Post-Execution Verification
- All 5 pods running
- No errors in last 60 seconds
- Latency within normal range
```

---

## escalation-manager

Manages escalation chains and stakeholder notifications.

### Usage

```bash
# Escalate an incident
aofctl run agent library://incident/escalation-manager \
  --prompt "Escalate INC-001 to SEV1 - need VP Engineering"

# Check escalation status
aofctl run agent library://incident/escalation-manager \
  --prompt "Who has been notified for the current incident?"
```

### Capabilities

- Escalation chain management
- Stakeholder notification
- On-call routing
- Escalation tracking
- De-escalation handling
- Handoff coordination

### Example Output

```markdown
## Escalation Status: INC-2024-1220-001

### Current Severity: SEV1 (escalated from SEV2)
**Escalation Reason**: Duration > 1 hour, revenue impact increasing

### Notification Status

| Level | Role | Person | Notified | Acknowledged |
|-------|------|--------|----------|--------------|
| L1 | On-Call Engineer | @alice | 14:30 | 14:33 |
| L2 | Team Lead | @bob | 14:45 | 14:48 |
| L3 | Engineering Manager | @carol | 15:00 | 15:05 |
| L4 | VP Engineering | @dave | 15:30 | 15:35 |
| - | Customer Success | @eve | 15:00 | 15:02 |

### Escalation Timeline

```
14:30 - L1 On-Call paged (auto)
14:45 - L2 Team Lead escalated (15min threshold)
15:00 - L3 Eng Manager escalated (30min threshold)
15:30 - L4 VP Engineering escalated (1hr threshold + revenue impact)
```

### Next Escalation
If not resolved by 16:30, CTO will be notified.

### Communication Sent
- Status Page: Updated at 14:35, 15:00, 15:30
- Customer Email: Sent to enterprise customers at 15:15
- Internal Slack: #incidents channel updated every 15min

### Recommended Actions
1. Prepare executive summary for VP sync
2. Draft customer communication for extended outage
3. Identify additional subject matter experts
```

---

## Environment Setup

```bash
# Kubernetes access
export KUBECONFIG=~/.kube/config

# Grafana (for RCA agent)
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_TOKEN=your-api-token

# Loki (for log analysis)
export LOKI_URL=https://loki.example.com

# PagerDuty (for escalation)
export PAGERDUTY_API_KEY=your-api-key

# Slack (for notifications)
export SLACK_BOT_TOKEN=xoxb-xxx
```

## Next Steps

- [CI/CD Agents](./cicd.md) - Pipeline management
- [Security Agents](./security.md) - Security operations
- [Incident Response Tutorial](../tutorials/incident-response.md) - Full workflow
