# Credential Auditing Guide

**Monitor and detect suspicious credential access patterns in AOF.**

## Overview

AOF's credential auditing system tracks every time an agent accesses credentials (kubeconfig, AWS keys, etc.), establishes behavioral baselines, and alerts on anomalous patterns that may indicate credential exfiltration attempts.

## Quick Start

### 1. Enable Credential Auditing

In your daemon configuration (`daemon.yaml`):

```yaml
credential_auditing:
  enabled: true
  audit_log_path: /var/log/aof/credential-audit.log
  learning_period_days: 7
  alert_threshold: 0.8
  block_threshold: 0.95
```

### 2. Start the Daemon

```bash
aofctl serve --config daemon.yaml
```

### 3. Monitor Access Patterns

```bash
# View recent credential accesses
tail -f /var/log/aof/credential-audit.log | jq

# Query specific agent
jq 'select(.agent_id == "agent-1")' /var/log/aof/credential-audit.log

# Find high anomaly scores
jq 'select(.anomaly_score > 0.7)' /var/log/aof/credential-audit.log
```

## Understanding the Audit Log

### Log Format

Each credential access generates a JSON log entry:

```json
{
  "event_id": "evt-1708012345678-42",
  "timestamp": "2026-02-14T12:30:00Z",
  "agent_id": "agent-incident-responder",
  "credential_type": "Kubernetes",
  "file_path": "/home/aof/.kube/config",
  "access_mode": "Read",
  "tool_context": {
    "tool_name": "kubectl",
    "operation": "get pods",
    "arguments": ["get", "pods", "-n", "production"],
    "risk_level": "Low"
  },
  "anomaly_score": 0.15,
  "sequence_number": 42,
  "session_id": "session-20260214-001"
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | String | Unique identifier for this access event |
| `timestamp` | ISO 8601 | When the access occurred (UTC) |
| `agent_id` | String | Which agent accessed the credential |
| `credential_type` | Enum | Type of credential (Kubernetes, AWS, GCP, Azure, Git, Database, Vault, Custom) |
| `file_path` | String | Path to the credential file |
| `access_mode` | Enum | How credential was accessed (Read, Write, Execute) |
| `tool_context.tool_name` | String | Tool that accessed the credential (kubectl, aws, gcloud, etc.) |
| `tool_context.operation` | String | What operation was performed |
| `tool_context.arguments` | Array | Tool arguments |
| `tool_context.risk_level` | Enum | Operation risk (Low, Medium, High, Critical) |
| `anomaly_score` | Float | Behavioral anomaly score (0.0-1.0) |
| `sequence_number` | Integer | Monotonically increasing number for tamper detection |
| `session_id` | String | Groups related accesses in a session |

### Credential Types Detected

AOF automatically detects credential requirements based on tool name:

| Tool | Credential Type | File Paths Monitored |
|------|----------------|----------------------|
| `kubectl`, `k9s` | Kubernetes | `~/.kube/config`, `/etc/kubernetes/admin.conf` |
| `aws`, `aws-cli` | AWS | `~/.aws/credentials`, `~/.aws/config` |
| `gcloud`, `gsutil` | GCP | `~/.config/gcloud/`, `$GOOGLE_APPLICATION_CREDENTIALS` |
| `az` | Azure | `~/.azure/` |
| `git`, `gh` | Git | `~/.gitconfig`, `~/.git-credentials` |
| `psql`, `mysql`, `redis-cli`, `mongosh` | Database | `~/.pgpass`, `~/.my.cnf`, etc. |
| `vault` | Vault | `~/.vault-token` |

## Anomaly Detection

### How Baselines Are Established

After **10+ accesses** for a given agent+credential pair, AOF establishes a behavioral baseline:

1. **Frequency baseline**: Mean time between accesses (e.g., every 30 minutes)
2. **Volume baseline**: Mean accesses per day (e.g., 20 accesses/day)
3. **Active hours**: Hours of day when accesses normally occur (e.g., 9am-6pm UTC)

### Anomaly Scoring

Each access is scored on **4 dimensions**:

| Dimension | Weight | Trigger Condition | Example |
|-----------|--------|-------------------|---------|
| **Frequency** | 0.0-0.4 | Access interval < 10% of baseline | Access every 3 minutes when baseline is 30 minutes |
| **Volume** | 0.0-0.3 | Daily accesses > 3x baseline | 60 accesses today when baseline is 20/day |
| **Time-of-day** | 0.0-0.2 | Access outside active hours | Access at 3am when baseline is 9am-6pm |
| **Burst** | 0.0-0.3 | >5 accesses within 60 seconds | 10 accesses in 10 seconds |

**Total score**: Sum of all dimensions, capped at 1.0

### Actions by Score

| Score Range | Action | What Happens | Example |
|-------------|--------|--------------|---------|
| **0.0-0.5** | Allow | Access proceeds normally | Normal kubectl usage |
| **0.5-0.7** | Log | Access logged with score | Slightly elevated frequency |
| **0.7-0.8** | Alert | Logged + administrator notified | Off-hours access detected |
| **0.8-0.95** | RequireApproval | Access blocked until approved | 10x frequency spike |
| **>0.95** | Block | Access denied, alert sent | Burst of 20 accesses in 10 seconds |

### Learning Period

**First 7 days** (or until >= 10 samples per agent+credential):

- All accesses score **0.0** (no false positives)
- Baselines are being established
- All access is allowed

After learning period:

- Baselines are applied
- Anomalous access triggers alerts/blocks

**Check if learning mode is active:**

```bash
# Look for learning mode messages in daemon logs
grep "Learning mode" /var/log/aof/daemon.log
```

## Configuring Thresholds

### Alert Threshold

Controls when administrators are notified:

```yaml
credential_auditing:
  alert_threshold: 0.8  # Alert when score >= 0.8
```

**Tuning**:
- **Too many alerts?** Increase to 0.85 or 0.9
- **Missing attacks?** Decrease to 0.75 or 0.7

### Block Threshold

Controls when access is automatically denied:

```yaml
credential_auditing:
  block_threshold: 0.95  # Block when score >= 0.95
```

**Tuning**:
- **False positives blocking legitimate access?** Increase to 0.98
- **Need stricter protection?** Decrease to 0.9

### Learning Period

How long to collect data before enforcing baselines:

```yaml
credential_auditing:
  learning_period_days: 7  # 7 days is default
```

**Tuning**:
- **Stable access patterns?** Use 3-5 days
- **Highly variable workload?** Use 14-30 days
- **Emergency deployment?** Use 1-2 days (higher false positive rate)

## Monitoring Credential Access

### View Real-Time Access

```bash
# Stream audit log with pretty JSON
tail -f /var/log/aof/credential-audit.log | jq -C

# Filter by agent
tail -f /var/log/aof/credential-audit.log | jq 'select(.agent_id == "agent-1")'

# Filter by credential type
tail -f /var/log/aof/credential-audit.log | jq 'select(.credential_type == "Kubernetes")'

# Show only high anomaly scores
tail -f /var/log/aof/credential-audit.log | jq 'select(.anomaly_score > 0.7)'
```

### Query Historical Access

```bash
# Count accesses by agent
jq -s 'group_by(.agent_id) | map({agent: .[0].agent_id, count: length})' /var/log/aof/credential-audit.log

# Count accesses by credential type
jq -s 'group_by(.credential_type) | map({type: .[0].credential_type, count: length})' /var/log/aof/credential-audit.log

# Find accesses in time range
jq 'select(.timestamp >= "2026-02-14T00:00:00Z" and .timestamp <= "2026-02-14T23:59:59Z")' /var/log/aof/credential-audit.log

# Calculate average anomaly score by agent
jq -s 'group_by(.agent_id) | map({agent: .[0].agent_id, avg_score: (map(.anomaly_score) | add / length)})' /var/log/aof/credential-audit.log
```

### Tamper Detection

Audit log entries have **sequence numbers** that are monotonically increasing. Gaps indicate deleted events (log tampering).

```bash
# Check for sequence gaps
jq -s 'sort_by(.sequence_number) | .[0:-1] as $prev | .[-1:][0] as $curr | $prev | to_entries | map(select(.value.sequence_number + 1 != $curr.sequence_number)) | map({gap_at: .value.sequence_number, next: ($curr.sequence_number)})' /var/log/aof/credential-audit.log
```

**If gaps found**: Investigate immediately. Check for:
- Log rotation issues (should preserve sequence across rotations)
- Manual file editing
- Compromised system

## WebSocket Event Streaming

High anomaly scores trigger real-time events on the WebSocket interface.

### Subscribe to Anomaly Alerts

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'credential_anomaly') {
    console.log('Anomaly detected:', data.payload);
    // {
    //   agent_id: "agent-1",
    //   credential_type: "Kubernetes",
    //   anomaly_score: 0.85,
    //   reasons: ["Frequency spike: 180 seconds vs baseline 1800 seconds"],
    //   recommended_action: "RequireApproval"
    // }
  }
};
```

### Integrate with Alerting Systems

Forward anomaly events to PagerDuty, Slack, etc.:

```bash
# Listen for anomaly events and send to Slack
while read line; do
  score=$(echo $line | jq -r '.anomaly_score')
  if (( $(echo "$score > 0.8" | bc -l) )); then
    agent=$(echo $line | jq -r '.agent_id')
    cred=$(echo $line | jq -r '.credential_type')
    curl -X POST $SLACK_WEBHOOK_URL \
      -H 'Content-Type: application/json' \
      -d "{\"text\": \"Credential anomaly: agent=$agent, type=$cred, score=$score\"}"
  fi
done < <(tail -f /var/log/aof/credential-audit.log)
```

## Responding to Anomalies

### Score 0.7-0.8 (Alert)

**What it means**: Mildly anomalous pattern (e.g., off-hours access)

**Action**:
1. Review the access in audit log
2. Confirm it's expected (e.g., on-call engineer investigating incident)
3. If unexpected, investigate further

### Score 0.8-0.95 (RequireApproval)

**What it means**: Significant anomaly (e.g., 10x frequency spike)

**Action**:
1. Access is **paused** until manually approved
2. Administrator receives alert
3. Review context: which agent, what operation, why the spike?
4. **Approve**: If legitimate (e.g., automated remediation loop)
5. **Deny**: If suspicious (e.g., unknown source)

**Approving access**:

```bash
# Via CLI (future feature)
aofctl credential approve --event-id evt-123

# Via API
curl -X POST http://localhost:8080/api/credential/approve \
  -H 'Content-Type: application/json' \
  -d '{"event_id": "evt-123", "approver": "admin@example.com"}'
```

### Score >0.95 (Block)

**What it means**: Extremely anomalous (e.g., 20 accesses in 10 seconds)

**Action**:
1. Access is **denied immediately**
2. Administrator receives critical alert
3. **Investigate**:
   - Which agent triggered it?
   - Is the agent compromised?
   - Is there a runaway loop in agent code?
4. **Remediate**:
   - Stop the agent: `aofctl agent stop <agent-id>`
   - Review agent code for bugs or malicious logic
   - Check for compromised dependencies
5. **Reset baseline** (if it was a false positive):

```bash
aofctl credential reset-baseline --agent-id agent-1 --credential-type Kubernetes
```

## Tuning for False Positives

### Symptom: Legitimate bursts flagged

**Example**: Incident response agent queries K8s 10 times in 30 seconds

**Solution 1**: Increase burst threshold

```yaml
# In daemon config (requires code change to expose this)
anomaly_detection:
  burst_threshold: 10  # Default is 5 accesses in 60 seconds
```

**Solution 2**: Exempt specific agents

```yaml
credential_auditing:
  exempt_agents:
    - agent-incident-responder  # No anomaly detection for this agent
```

### Symptom: Off-hours access always alerts

**Example**: Global team works across timezones

**Solution**: Extend active hours

The baseline learns active hours from observed access patterns. If your team works 24/7:

- Let the learning period run longer (14-30 days)
- Access will occur at all hours, establishing a 24/7 baseline
- Future off-hours access won't trigger anomalies

### Symptom: Volume spikes after code changes

**Example**: New agent feature causes 3x more K8s queries

**Solution**: Reset baseline after known changes

```bash
# Reset baseline to trigger re-learning
aofctl credential reset-baseline --agent-id agent-1 --credential-type Kubernetes
```

Then let the agent run for 7 days to establish a new baseline.

## Security Best Practices

### 1. Rotate Credentials Regularly

Anomaly detection helps detect exfiltration, but can't prevent it. Limit blast radius:

```bash
# Kubernetes: Rotate service account tokens
kubectl create token <service-account> --duration=24h

# AWS: Use temporary credentials
aws sts get-session-token --duration-seconds 3600
```

### 2. Separate Credentials by Agent

Never share credentials across agents:

```bash
# Bad: All agents use same kubeconfig
-v /shared/kubeconfig:/creds/kubeconfig:ro

# Good: Each agent has its own kubeconfig
-v /var/aof/creds/agent-1/kubeconfig:/creds/kubeconfig:ro
-v /var/aof/creds/agent-2/kubeconfig:/creds/kubeconfig:ro
```

This way, if one agent is compromised, others are unaffected.

### 3. Monitor Audit Log Growth

Audit logs can grow large (1KB per access event):

```bash
# Check log size
ls -lh /var/log/aof/credential-audit.log

# Rotate logs weekly
logrotate /etc/logrotate.d/aof-credential-audit
```

**Logrotate config** (`/etc/logrotate.d/aof-credential-audit`):

```
/var/log/aof/credential-audit.log {
    weekly
    rotate 12
    compress
    delaycompress
    missingok
    notifempty
    postrotate
        # Signal daemon to reopen log file
        killall -SIGHUP aofctl
    endscript
}
```

### 4. Archive Logs for Forensics

Keep audit logs for at least 90 days:

```bash
# Compress and archive old logs
gzip /var/log/aof/credential-audit.log.1
mv /var/log/aof/credential-audit.log.1.gz /var/archive/aof/

# Or send to centralized logging (Elasticsearch, Splunk, etc.)
filebeat -c /etc/filebeat/filebeat.yml
```

## Troubleshooting

### Audit Log Not Growing

**Symptom**: `/var/log/aof/credential-audit.log` is empty or not updating

**Causes**:
1. Credential auditing not enabled
2. Log file permissions incorrect
3. No credential accesses occurring

**Fix**:

```bash
# Check if enabled
grep "credential_auditing.enabled" daemon.yaml

# Check log file permissions
ls -la /var/log/aof/credential-audit.log
# Should be writable by daemon user

# Manually trigger an access
kubectl get pods  # Should generate a log entry
tail -1 /var/log/aof/credential-audit.log
```

### Learning Mode Never Exits

**Symptom**: All anomaly scores are 0.0 even after 7 days

**Cause**: Not enough accesses to establish baseline (need >= 10 per agent+credential)

**Fix**:

```bash
# Check how many accesses recorded
jq -s 'group_by("\(.agent_id):\(.credential_type)") | map({key: .[0].agent_id + ":" + .[0].credential_type, count: length})' /var/log/aof/credential-audit.log

# If count < 10, wait for more accesses or manually trigger some
```

### High False Positive Rate

**Symptom**: Legitimate accesses scoring > 0.7

**Causes**:
1. Learning period too short (workload not fully observed)
2. Thresholds too strict
3. Highly variable workload

**Fixes**:
1. Extend learning period to 14-30 days
2. Increase alert_threshold from 0.8 to 0.85 or 0.9
3. Reset baselines and re-learn with longer observation window

## See Also

- [Sandbox Security](/docs/concepts/sandbox-security.md) — Defense-in-depth overview
- [Security Hardening (Technical)](/docs/dev/security-hardening.md) — Implementation details
- [WebSocket API](/docs/api/websocket.md) — Real-time event streaming
- [Decision Logging](/docs/concepts/decision-logging.md) — Audit trail for agent actions
