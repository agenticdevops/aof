# Tutorial: Incident Response Auto-Remediation

Build an intelligent incident response system that detects issues, diagnoses problems, attempts auto-remediation, and escalates to humans when needed.

**What you'll learn:**
- PagerDuty webhook integration
- Multi-stage diagnostic workflows
- Conditional remediation logic
- Human-in-the-loop approvals
- Post-incident analysis

**Time:** 30 minutes

## Prerequisites

- `aofctl` installed
- PagerDuty account (free tier works)
- Slack workspace (for notifications)
- Kubernetes cluster
- OpenAI or Anthropic API key

## Architecture Overview

```
PagerDuty Alert
      ↓
  [Webhook Trigger]
      ↓
  [Diagnostic Agent] ─→ Analyze logs, metrics, events
      ↓
  [Severity Check]
      ├─→ Critical: Request Human Approval
      └─→ Non-Critical: Auto-Remediate
            ↓
      [Remediation Agent] ─→ Fix the issue
            ↓
      [Verify Fix]
            ↓
      [Notify Slack] ─→ Report outcome
            ↓
      [Close PagerDuty]
```

## Step 1: Create Diagnostic Agent

Create `diagnostic-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: incident-diagnostic
  labels:
    purpose: diagnostics
    team: sre

spec:
  model: anthropic:claude-3-5-sonnet-20241022

  model_config:
    temperature: 0.2  # Low temperature for deterministic analysis
    max_tokens: 3000

  instructions: |
    You are an expert SRE performing incident diagnostics.

    Your role:
    - Analyze the incident alert details
    - Check pod status, logs, and events
    - Identify root cause
    - Classify severity (critical, high, medium, low)
    - Recommend remediation steps

    Diagnostic process:
    1. Understand the alert (service, error, metrics)
    2. Check current state (kubectl get/describe)
    3. Review recent logs (kubectl logs)
    4. Check events (kubectl get events)
    5. Analyze patterns and correlations
    6. Determine root cause
    7. Assess impact and severity
    8. Recommend fix

    Output format:
    ```json
    {
      "severity": "critical|high|medium|low",
      "root_cause": "Brief description",
      "affected_components": ["pod-name", "service-name"],
      "impact": "User-facing impact description",
      "recommended_action": "Specific remediation step",
      "requires_approval": true|false,
      "confidence": 0.0-1.0
    }
    ```

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        timeout_seconds: 60

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"

    - type: HTTP
      config:
        # For checking service health
        timeout_seconds: 10

  memory:
    type: SQLite
    config:
      path: ./incident-diagnostics.db
```

## Step 2: Create Remediation Agent

Create `remediation-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: incident-remediation
  labels:
    purpose: remediation
    team: sre

spec:
  model: openai:gpt-4

  model_config:
    temperature: 0.1  # Very low - we want predictable fixes
    max_tokens: 2000

  instructions: |
    You are an expert SRE performing incident remediation.

    Your role:
    - Execute the recommended remediation action
    - Verify the fix worked
    - Document what was done
    - Rollback if fix fails

    Available remediation actions:
    - Restart pods (kubectl rollout restart)
    - Scale deployments (kubectl scale)
    - Update resources (kubectl patch)
    - Clear stuck resources (kubectl delete pod)
    - Rollback deployments (kubectl rollout undo)

    Safety rules:
    - Always use --dry-run first for destructive ops
    - Verify current state before changes
    - Take snapshots of resources before modification
    - Monitor for 60 seconds after remediation
    - Rollback if health checks fail

    Output format:
    ```json
    {
      "action_taken": "Specific command executed",
      "result": "success|failed|partial",
      "verification": "Health check results",
      "rollback_needed": true|false,
      "logs": "Relevant output"
    }
    ```

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        timeout_seconds: 120

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]

  memory:
    type: SQLite
    config:
      path: ./incident-remediation.db
```

## Step 3: Create Incident Response Flow

Create `incident-response-flow.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-auto-response

spec:
  # Triggered by PagerDuty webhook
  trigger:
    type: Webhook
    config:
      path: /pagerduty/webhook
      methods: [POST]
      auth:
        type: Bearer
        token: ${PAGERDUTY_WEBHOOK_TOKEN}

  nodes:
    # 1. Parse PagerDuty alert
    - id: parse-alert
      type: Transform
      config:
        script: |
          # Extract incident details
          export INCIDENT_ID="${event.incident.id}"
          export INCIDENT_TITLE="${event.incident.title}"
          export INCIDENT_SERVICE="${event.incident.service.name}"
          export INCIDENT_URGENCY="${event.incident.urgency}"
          export INCIDENT_URL="${event.incident.html_url}"

          # Extract K8s context if available
          export K8S_NAMESPACE="${event.incident.custom_details.namespace:-default}"
          export K8S_RESOURCE="${event.incident.custom_details.resource}"

    # 2. Run diagnostics
    - id: diagnose
      type: Agent
      config:
        agent: incident-diagnostic
        input: |
          Incident: ${INCIDENT_TITLE}
          Service: ${INCIDENT_SERVICE}
          Urgency: ${INCIDENT_URGENCY}
          Namespace: ${K8S_NAMESPACE}
          Resource: ${K8S_RESOURCE}

          Diagnose this incident and recommend remediation.
        timeout_seconds: 180

    # 3. Notify Slack immediately
    - id: notify-diagnosis
      type: Slack
      config:
        channel: "#incidents"
        message: |
          🚨 **Incident Detected**

          **Incident**: ${INCIDENT_TITLE}
          **Service**: ${INCIDENT_SERVICE}
          **Severity**: ${diagnose.output.severity}
          **Root Cause**: ${diagnose.output.root_cause}

          **Impact**: ${diagnose.output.impact}
          **Recommended Action**: ${diagnose.output.recommended_action}

          **Status**: Analyzing...
          **Link**: ${INCIDENT_URL}

    # 4. Check if critical severity
    - id: check-severity
      type: Conditional
      config:
        conditions:
          - name: is_critical
            expression: ${diagnose.output.severity} == "critical"
          - name: needs_approval
            expression: ${diagnose.output.requires_approval} == true

    # 5a. Request human approval for critical incidents
    - id: request-approval
      type: Slack
      config:
        channel: "#incidents"
        message: |
          ⚠️ **CRITICAL: Human Approval Required**

          **Incident**: ${INCIDENT_TITLE}
          **Root Cause**: ${diagnose.output.root_cause}
          **Proposed Fix**: ${diagnose.output.recommended_action}
          **Confidence**: ${diagnose.output.confidence}

          React with ✅ to approve auto-remediation
          React with ⏸️ to pause and investigate manually
          React with ❌ to skip auto-remediation

          cc: @oncall @sre-lead
        wait_for_reaction: true
        timeout_seconds: 600  # 10 minutes
      conditions:
        - from: check-severity
          when: is_critical == true OR needs_approval == true

    # 5b. Auto-proceed for non-critical
    - id: auto-approve
      type: Transform
      config:
        script: export APPROVED=true
      conditions:
        - from: check-severity
          when: is_critical == false AND needs_approval == false

    # 6. Execute remediation
    - id: remediate
      type: Agent
      config:
        agent: incident-remediation
        input: |
          Execute the following remediation:

          Action: ${diagnose.output.recommended_action}
          Namespace: ${K8S_NAMESPACE}
          Resource: ${K8S_RESOURCE}

          Verify the fix works and monitor for 60 seconds.
        timeout_seconds: 300
      conditions:
        # Run if auto-approved OR human-approved
        - from: auto-approve
          when: APPROVED == true
        - from: request-approval
          when: reaction == "white_check_mark"

    # 7. Verify fix worked
    - id: verify-fix
      type: Agent
      config:
        agent: incident-diagnostic
        input: |
          Verify the incident is resolved:

          Original Issue: ${INCIDENT_TITLE}
          Remediation: ${remediate.output.action_taken}

          Check if the problem is fixed.
        timeout_seconds: 120

    # 8. Handle failed remediation
    - id: remediation-failed
      type: Conditional
      config:
        condition: ${remediate.output.result} != "success"

    - id: rollback
      type: Agent
      config:
        agent: incident-remediation
        input: "Rollback the failed remediation: ${remediate.output.action_taken}"
      conditions:
        - from: remediation-failed
          when: true

    - id: escalate
      type: Slack
      config:
        channel: "#incidents"
        message: |
          🔴 **Auto-Remediation FAILED - Manual Intervention Required**

          **Incident**: ${INCIDENT_TITLE}
          **Attempted Fix**: ${remediate.output.action_taken}
          **Result**: ${remediate.output.result}
          **Rollback**: ${rollback.output.result}

          @oncall please investigate immediately

          **Link**: ${INCIDENT_URL}
      conditions:
        - from: remediation-failed
          when: true

    # 9. Success path - notify and close
    - id: notify-success
      type: Slack
      config:
        channel: "#incidents"
        message: |
          ✅ **Incident Auto-Resolved**

          **Incident**: ${INCIDENT_TITLE}
          **Root Cause**: ${diagnose.output.root_cause}
          **Fix Applied**: ${remediate.output.action_taken}
          **Verification**: ${verify-fix.output}

          **Resolution Time**: ${flow.duration_seconds}s
          **Status**: Resolved automatically
      conditions:
        - from: remediation-failed
          when: false

    # 10. Close PagerDuty incident
    - id: close-pagerduty
      type: HTTP
      config:
        method: PUT
        url: https://api.pagerduty.com/incidents/${INCIDENT_ID}
        headers:
          Authorization: "Token token=${PAGERDUTY_API_KEY}"
          Content-Type: application/json
        body: |
          {
            "incident": {
              "type": "incident_reference",
              "status": "resolved",
              "resolution": "Auto-resolved by AOF: ${remediate.output.action_taken}"
            }
          }
      conditions:
        - from: remediation-failed
          when: false

    # 11. Log incident for analysis
    - id: log-incident
      type: Transform
      config:
        script: |
          # Store incident data for post-mortem
          cat > /tmp/incident-${INCIDENT_ID}.json <<EOF
          {
            "incident_id": "${INCIDENT_ID}",
            "title": "${INCIDENT_TITLE}",
            "severity": "${diagnose.output.severity}",
            "root_cause": "${diagnose.output.root_cause}",
            "remediation": "${remediate.output.action_taken}",
            "result": "${remediate.output.result}",
            "duration_seconds": ${flow.duration_seconds},
            "auto_resolved": true,
            "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
          }
          EOF

  # Define execution flow
  connections:
    - from: parse-alert
      to: diagnose

    - from: diagnose
      to: notify-diagnosis

    - from: notify-diagnosis
      to: check-severity

    - from: check-severity
      to: request-approval
      to: auto-approve

    - from: request-approval
      to: remediate

    - from: auto-approve
      to: remediate

    - from: remediate
      to: verify-fix

    - from: verify-fix
      to: remediation-failed

    - from: remediation-failed
      to: rollback
      to: notify-success

    - from: rollback
      to: escalate

    - from: notify-success
      to: close-pagerduty

    - from: close-pagerduty
      to: log-incident
```

## Step 4: Configure PagerDuty

### Create Webhook in PagerDuty

1. Go to **Integrations** → **Generic Webhooks (v3)**
2. Add webhook: `https://your-domain.com/pagerduty/webhook`
3. Select events:
   - Incident Triggered
   - Incident Acknowledged
   - Incident Resolved
4. Copy webhook token

```bash
export PAGERDUTY_WEBHOOK_TOKEN=your-token
export PAGERDUTY_API_KEY=your-api-key
```

### Add Custom Fields

Add to your PagerDuty service:
- `namespace` - K8s namespace
- `resource` - K8s resource (deployment/pod/service)

## Step 5: Deploy the System

```bash
# Deploy agents
aofctl agent apply -f diagnostic-agent.yaml
aofctl agent apply -f remediation-agent.yaml

# Deploy flow
aofctl flow apply -f incident-response-flow.yaml

# Start the flow
aofctl flow run incident-auto-response --daemon

# Verify it's running
aofctl flow status incident-auto-response
```

## Step 6: Test the System

### Test 1: Simulate Non-Critical Incident

```bash
# Trigger a test PagerDuty incident
curl -X POST https://your-domain.com/pagerduty/webhook \
  -H "Authorization: Bearer ${PAGERDUTY_WEBHOOK_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "event": {
      "incident": {
        "id": "TEST001",
        "title": "High memory usage on api-deployment",
        "service": {
          "name": "API Service"
        },
        "urgency": "high",
        "html_url": "https://yourcompany.pagerduty.com/incidents/TEST001",
        "custom_details": {
          "namespace": "production",
          "resource": "deployment/api"
        }
      }
    }
  }'
```

Expected flow:
1. Alert parsed
2. Diagnostic agent analyzes (finds high memory pods)
3. Slack notification sent
4. Auto-approves (not critical)
5. Remediation agent restarts pods
6. Verification succeeds
7. PagerDuty incident closed
8. Success notification

### Test 2: Critical Incident (Requires Approval)

```bash
curl -X POST https://your-domain.com/pagerduty/webhook \
  -H "Authorization: Bearer ${PAGERDUTY_WEBHOOK_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "event": {
      "incident": {
        "id": "TEST002",
        "title": "Database cluster down - all replicas failing",
        "service": {
          "name": "PostgreSQL"
        },
        "urgency": "high",
        "html_url": "https://yourcompany.pagerduty.com/incidents/TEST002",
        "custom_details": {
          "namespace": "production",
          "resource": "statefulset/postgres"
        }
      }
    }
  }'
```

Expected flow:
1. Alert parsed
2. Diagnostic agent classifies as CRITICAL
3. Slack notification sent
4. Approval requested (waits for reaction)
5. Human approves ✅
6. Remediation attempts fix
7. Verification check
8. Result notification

## Step 7: Add Post-Incident Analysis

Create `post-incident-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: post-incident-analyzer

spec:
  model: anthropic:claude-3-5-sonnet-20241022

  instructions: |
    You are an SRE performing post-incident analysis.

    Analyze incident logs and generate:
    - Timeline of events
    - Root cause analysis
    - Contributing factors
    - Remediation effectiveness
    - Recommendations to prevent recurrence

    Format as a markdown report suitable for a post-mortem doc.

  tools:
    - type: Shell
      config:
        allowed_commands: [cat, jq, grep]
```

Add to flow:

```yaml
# Daily post-incident report
- id: daily-analysis
  type: Agent
  config:
    agent: post-incident-analyzer
    input: "Analyze all incidents from the past 24 hours: /tmp/incident-*.json"

- id: send-report
  type: Slack
  config:
    channel: "#sre-postmortems"
    message: ${daily-analysis.output}
```

## Step 8: Monitor the System

```bash
# View all incident responses
aofctl flow logs incident-auto-response

# Get metrics
aofctl flow metrics incident-auto-response

# Check success rate
aofctl flow describe incident-auto-response | grep "success_rate"

# View diagnostic logs
aofctl agent logs incident-diagnostic -f

# View remediation logs
aofctl agent logs incident-remediation -f
```

## Production Best Practices

### 1. Add Rate Limiting

```yaml
spec:
  rate_limit:
    max_concurrent: 3        # Max 3 incidents at once
    queue_size: 10          # Queue up to 10
    timeout_seconds: 1800   # 30 min max per incident
```

### 2. Add Monitoring

```yaml
- id: track-metrics
  type: HTTP
  config:
    method: POST
    url: https://metrics.yourcompany.com/incidents
    body: |
      {
        "incident_id": "${INCIDENT_ID}",
        "duration": ${flow.duration_seconds},
        "severity": "${diagnose.output.severity}",
        "auto_resolved": ${remediate.output.result == "success"}
      }
```

### 3. Add Runbook Integration

```yaml
- id: fetch-runbook
  type: HTTP
  config:
    url: https://runbooks.yourcompany.com/api/${INCIDENT_SERVICE}
    method: GET

- id: diagnose-with-runbook
  type: Agent
  config:
    agent: incident-diagnostic
    input: |
      Incident: ${INCIDENT_TITLE}
      Runbook Steps: ${fetch-runbook.output}

      Follow runbook for diagnosis.
```

## Troubleshooting

### Diagnostics timeout

```bash
# Increase timeout
timeout_seconds: 300  # 5 minutes

# Or split into multiple steps
```

### Remediation fails

```bash
# Check agent logs
aofctl agent logs incident-remediation --tail 100

# Verify kubectl access
kubectl cluster-info

# Test remediation manually
aofctl agent exec incident-remediation "Check pod status in production"
```

### Approval reactions not working

```bash
# Check Slack integration
aofctl flow logs incident-auto-response | grep slack

# Verify bot scopes include reactions:read
```

## Next Steps

- **[AgentFlow Reference](../reference/agentflow-spec.md)** - Advanced patterns
- **[Example Flows](../examples/)** - More automation examples
- **Production Deployment** - Scale to handle all incidents

---

**🎉 You've built an intelligent incident response system!** Your on-call just got a lot easier.
