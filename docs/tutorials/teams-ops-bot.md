# Building a Teams Ops Bot

Build a complete DevOps bot for Microsoft Teams with Adaptive Cards, approval workflows, and multi-agent coordination.

## What We're Building

A Teams bot that:
- Responds to natural language DevOps queries
- Shows rich status dashboards with Adaptive Cards
- Handles deployment approvals with Action.Submit
- Coordinates multi-agent workflows
- Integrates with Kubernetes, Docker, and cloud services

## Prerequisites

- Completed [Teams Quickstart](../guides/quickstart-teams.md)
- Kubernetes cluster access (local minikube or remote)
- Basic familiarity with Teams app development

## Architecture Overview

```
Teams Channel
     ↓
Azure Bot Service → AOF Webhook
                        ↓
                   Parse Message
                        ↓
              ┌────────┴────────┐
              ↓                 ↓
         Commands          Natural Language
              ↓                 ↓
         Route to           LLM Agent
         Agent/Fleet            ↓
              ↓             Tool Execution
              ↓                 ↓
         Adaptive Card ← Format Response
              ↓
         Teams Reply
```

## Step 1: Enhanced Trigger Configuration

Create a comprehensive trigger with commands and approval workflow:

```yaml
# triggers/teams-ops.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: teams-ops
  labels:
    platform: teams
    environment: production
spec:
  type: Teams
  config:
    app_id: ${TEAMS_APP_ID}
    app_password: ${TEAMS_APP_PASSWORD}

    # Restrict to your organization
    allowed_tenants:
      - ${TEAMS_TENANT_ID}

    # Approval settings
    approval_channel: "19:approvals-channel@thread.tacv2"
    approval_allowed_users:
      - "sre-lead@company.com"
      - "devops-team@company.com"

  # Command definitions
  commands:
    /help:
      agent: devops
      description: "Show available commands"

    /status:
      agent: k8s-ops
      description: "Cluster status dashboard"

    /pods:
      agent: k8s-ops
      description: "List pods in namespace"
      parameters:
        - name: namespace
          required: false
          default: "default"

    /logs:
      agent: k8s-ops
      description: "View pod logs"
      parameters:
        - name: pod
          required: true

    /deploy:
      agent: deployer
      description: "Deploy application"
      requires_approval: true
      parameters:
        - name: app
          required: true
        - name: version
          required: true
        - name: environment
          required: true

    /rollback:
      agent: deployer
      description: "Rollback deployment"
      requires_approval: true

    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis"

    /incident:
      flow: incident-flow
      description: "Start incident response"

  default_agent: devops
```

## Step 2: Specialized Agents

### DevOps Agent

```yaml
# agents/devops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: devops
  labels:
    platform: teams
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "General DevOps assistant for Teams"

  tools:
    - kubectl
    - docker
    - helm
    - aws
    - az

  system_prompt: |
    You are a DevOps assistant in Microsoft Teams.

    ## Response Format
    Structure responses for Adaptive Cards:
    - Use headers with ** for emphasis
    - Use bullet points for lists
    - Include status emoji: ✅ ⚠️ ❌ 🔄 📊
    - Keep responses concise but informative

    ## Available Commands
    - /status - Cluster dashboard
    - /pods [namespace] - List pods
    - /logs <pod> - View logs
    - /deploy <app> <version> <env> - Deploy (requires approval)
    - /rollback - Rollback deployment
    - /diagnose - Root cause analysis
    - /incident - Start incident response

    ## When showing status:
    📊 **Cluster Status**

    | Component | Status |
    |-----------|--------|
    | Nodes | ✅ 3/3 Ready |
    | Pods | ⚠️ 45/48 Running |
    | Services | ✅ All healthy |

    [View Details] [Check Alerts]
```

### Kubernetes Ops Agent

```yaml
# agents/k8s-ops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    platform: teams
    specialty: kubernetes
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "Kubernetes operations specialist"

  tools:
    - kubectl
    - helm

  system_prompt: |
    You are a Kubernetes specialist in Teams.

    ## Response Guidelines
    - Always specify namespace in commands
    - Show pod status with clear indicators
    - Highlight issues prominently
    - Suggest troubleshooting steps

    ## Pod Status Format
    📦 **Pods in {namespace}**

    | Pod | Status | Restarts | Age |
    |-----|--------|----------|-----|
    | api-abc | ✅ Running | 0 | 2d |
    | web-xyz | ⚠️ Pending | 0 | 5m |
    | worker-123 | ❌ CrashLoop | 5 | 1h |

    ⚠️ Issues found: 2 pods need attention

    [View Logs] [Describe Pods] [Events]

    ## For problems, explain:
    1. What's happening
    2. Likely cause
    3. Suggested fix
    4. Available actions
```

### Deployer Agent

```yaml
# agents/deployer.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: deployer
  labels:
    platform: teams
    specialty: deployments
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "Deployment manager with approval workflow"

  tools:
    - kubectl
    - helm
    - argocd

  system_prompt: |
    You are a deployment manager in Teams.

    ## Deployment Workflow
    1. Validate deployment request
    2. Check current state
    3. Request approval if required
    4. Execute deployment
    5. Verify success

    ## Approval Request Format
    🚀 **Deployment Approval Required**

    | Field | Value |
    |-------|-------|
    | Application | {app} |
    | Version | {current} → {new} |
    | Environment | {env} |
    | Requested By | @{user} |

    **Changes:**
    - Feature: New user dashboard
    - Fix: Login timeout issue

    [✅ Approve] [❌ Reject] [💬 Request Changes]

    ## Post-Deployment
    ✅ **Deployment Successful**

    | Metric | Value |
    |--------|-------|
    | Duration | 45s |
    | Replicas | 3/3 Ready |
    | Health | All passing |

    [View Logs] [Rollback]
```

## Step 3: RCA Fleet for Diagnosis

```yaml
# fleets/rca-fleet.yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: rca-fleet
  labels:
    platform: teams
    purpose: diagnosis
spec:
  description: "Multi-agent root cause analysis"

  agents:
    - name: symptom-collector
      role: "Gather symptoms and current state"
      model: google:gemini-2.5-flash
      tools: [kubectl, prometheus]

    - name: log-analyzer
      role: "Analyze logs for errors"
      model: google:gemini-2.5-flash
      tools: [kubectl, loki]

    - name: metric-analyzer
      role: "Check metrics for anomalies"
      model: google:gemini-2.5-flash
      tools: [prometheus, grafana]

    - name: synthesizer
      role: "Synthesize findings into RCA"
      model: google:gemini-2.5-flash

  workflow:
    mode: parallel
    consensus: required
    timeout: 300

  output_format: |
    🔍 **Root Cause Analysis**

    **Summary:** {summary}

    **Timeline:**
    {timeline}

    **Root Cause:**
    {root_cause}

    **Impact:**
    {impact}

    **Remediation:**
    {remediation}

    [Apply Fix] [View Details] [Ignore]
```

## Step 4: Incident Response Flow

```yaml
# flows/incident-flow.yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFlow
metadata:
  name: incident-flow
spec:
  description: "Incident response workflow"

  triggers:
    - type: command
      command: /incident
    - type: alert
      source: pagerduty

  steps:
    - name: triage
      agent: incident-commander
      action: "Assess severity and impact"
      outputs: [severity, impact, affected_services]

    - name: notify
      action: notify_channel
      channel: "19:incidents@thread.tacv2"
      message: |
        🚨 **Incident Declared**
        Severity: {severity}
        Impact: {impact}
        Commander: @{user}

    - name: diagnose
      fleet: rca-fleet
      parallel: true

    - name: remediate
      agent: deployer
      action: "Execute remediation"
      requires_approval: true

    - name: resolve
      agent: incident-commander
      action: "Verify resolution and close"
```

## Step 5: Adaptive Card Templates

Create rich response templates:

### Status Dashboard Card

```yaml
# templates/status-card.yaml
type: AdaptiveCard
version: "1.4"
body:
  - type: TextBlock
    text: "📊 Cluster Dashboard"
    weight: Bolder
    size: Large

  - type: ColumnSet
    columns:
      - type: Column
        width: stretch
        items:
          - type: TextBlock
            text: "Nodes"
            weight: Bolder
          - type: TextBlock
            text: "${nodes_status}"
            color: "${nodes_color}"
      - type: Column
        width: stretch
        items:
          - type: TextBlock
            text: "Pods"
            weight: Bolder
          - type: TextBlock
            text: "${pods_status}"
            color: "${pods_color}"
      - type: Column
        width: stretch
        items:
          - type: TextBlock
            text: "Services"
            weight: Bolder
          - type: TextBlock
            text: "${services_status}"
            color: "${services_color}"

  - type: FactSet
    facts:
      - title: "CPU Usage"
        value: "${cpu_usage}"
      - title: "Memory"
        value: "${memory_usage}"
      - title: "Disk"
        value: "${disk_usage}"

actions:
  - type: Action.Submit
    title: "🔄 Refresh"
    data:
      action: refresh_status
  - type: Action.Submit
    title: "📊 Details"
    data:
      action: show_details
  - type: Action.Submit
    title: "⚠️ Alerts"
    data:
      action: show_alerts
```

## Step 6: Handle Adaptive Card Actions

When users click buttons, AOF receives invoke activities:

```yaml
# In your agent system prompt, add:
system_prompt: |
  ## Handling Button Clicks
  When you receive a message with "action:" prefix,
  it's an Adaptive Card button click.

  Handle these actions:
  - action:refresh_status → Run kubectl get pods,nodes,svc
  - action:show_details → Show detailed component info
  - action:show_alerts → Query Prometheus for alerts
  - action:view_logs → Show recent pod logs
  - action:approve → Process approval
  - action:reject → Process rejection
  - action:request_changes → Ask for clarification
```

## Step 7: Testing the Bot

### Test Commands

In Teams, try:

```
/help
/status
/pods
/pods kube-system
/logs api-server-abc123
check memory usage on production nodes
/diagnose high latency in api
```

### Test Approval Flow

```
/deploy api-server v2.1.0 production
```

Expected flow:
1. Bot sends approval card to approval channel
2. Approver clicks Approve/Reject
3. Bot proceeds with or cancels deployment
4. Bot sends result confirmation

## Step 8: Production Deployment

### Azure App Service

```bash
# Create App Service
az webapp create \
  --name aof-ops-bot \
  --resource-group aof-bot-rg \
  --plan aof-app-plan \
  --runtime "DOCKER|aof/aofctl:latest"

# Configure environment
az webapp config appsettings set \
  --name aof-ops-bot \
  --settings \
    TEAMS_APP_ID=$TEAMS_APP_ID \
    TEAMS_APP_PASSWORD=$TEAMS_APP_PASSWORD
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-daemon
spec:
  replicas: 2
  selector:
    matchLabels:
      app: aof-daemon
  template:
    metadata:
      labels:
        app: aof-daemon
    spec:
      containers:
        - name: aofctl
          image: aof/aofctl:latest
          ports:
            - containerPort: 8080
          env:
            - name: TEAMS_APP_ID
              valueFrom:
                secretKeyRef:
                  name: teams-credentials
                  key: app_id
            - name: TEAMS_APP_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: teams-credentials
                  key: app_password
          volumeMounts:
            - name: config
              mountPath: /app/.aof
      volumes:
        - name: config
          configMap:
            name: aof-config
---
apiVersion: v1
kind: Service
metadata:
  name: aof-daemon
spec:
  ports:
    - port: 80
      targetPort: 8080
  selector:
    app: aof-daemon
```

## Best Practices

### 1. Card Design

- Keep cards focused and scannable
- Use FactSet for structured data
- Limit to 3-4 actions per card
- Use color coding consistently

### 2. Error Handling

- Always provide clear error messages
- Suggest next steps on failure
- Include troubleshooting actions
- Log all interactions for debugging

### 3. Security

- Restrict to specific tenants
- Limit approval users
- Audit all deployments
- Use Azure Key Vault for secrets

### 4. Performance

- Cache cluster state locally
- Batch kubectl commands
- Use streaming for logs
- Implement request timeouts

## Troubleshooting

### Card Not Rendering

```bash
# Validate card JSON
curl -X POST https://adaptivecards.io/api/card \
  -H "Content-Type: application/json" \
  -d @card.json
```

### Actions Not Working

```bash
# Check invoke activities in logs
aofctl daemon logs | grep invoke

# Verify action data format
```

### Slow Responses

```bash
# Profile agent execution
aofctl agent run devops "status" --profile

# Check external tool latency
time kubectl get pods
```

## Next Steps

- [Approval Workflow Guide](../guides/approval-workflow.md)
- [Fleet Configuration](../reference/fleet-spec.md)
- [Custom Tools](../tools/custom-tools.md)
- [Deployment Guide](../guides/deployment.md)
