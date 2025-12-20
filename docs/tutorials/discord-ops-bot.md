# Building a Discord Ops Bot

Build a complete DevOps bot for Discord with slash commands, embeds, buttons, and multi-agent coordination.

## What We're Building

A Discord bot that:
- Responds to slash commands for DevOps operations
- Shows rich status embeds with color-coded information
- Uses buttons for interactive workflows
- Coordinates multi-agent tasks
- Supports role-based access control

## Prerequisites

- Completed [Discord Quickstart](../guides/quickstart-discord.md)
- Kubernetes cluster access (local minikube or remote)
- Basic familiarity with Discord bot development

## Architecture Overview

```
Discord Server
      ↓
Discord Gateway → AOF Webhook
                       ↓
                  Ed25519 Verify
                       ↓
                  Parse Interaction
                       ↓
             ┌────────┴────────┐
             ↓                 ↓
        Slash Command     Component
             ↓                 ↓
        Route to           Handle
        Agent/Fleet        Action
             ↓                 ↓
        Embed + Buttons ← Format Response
             ↓
        Discord Reply
```

## Step 1: Enhanced Trigger Configuration

Create a comprehensive trigger with commands and role restrictions:

```yaml
# triggers/discord-ops.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-ops
  labels:
    platform: discord
    environment: production
spec:
  type: Discord
  config:
    bot_token: ${DISCORD_BOT_TOKEN}
    application_id: ${DISCORD_APPLICATION_ID}
    public_key: ${DISCORD_PUBLIC_KEY}

    # Restrict to specific servers
    guild_ids:
      - ${DISCORD_GUILD_ID}

    # Role restrictions
    allowed_roles:
      - ${DISCORD_ADMIN_ROLE}
      - ${DISCORD_DEVOPS_ROLE}

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

    /logs:
      agent: k8s-ops
      description: "View pod logs"

    /deploy:
      agent: deployer
      description: "Deploy application"

    /scale:
      agent: k8s-ops
      description: "Scale deployment"

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
    platform: discord
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "General DevOps assistant for Discord"

  tools:
    - kubectl
    - docker
    - helm
    - aws

  system_prompt: |
    You are a DevOps assistant in Discord.

    ## Response Format
    Use Discord embed formatting:
    - Title with emoji prefix
    - Color-coded status (green=success, yellow=warning, red=error)
    - Inline fields for metrics
    - Footer with timestamp

    ## Available Commands
    - /status - Cluster dashboard
    - /pods [namespace] - List pods
    - /logs <pod> - View logs
    - /deploy <app> <version> - Deploy
    - /scale <deployment> <replicas> - Scale
    - /diagnose - Root cause analysis
    - /incident - Start incident response

    ## Example Status Response
    Title: 📊 Cluster Status
    Color: 0x00FF00 (green for healthy)

    Fields:
    - Nodes: ✅ 3/3 Ready (inline)
    - Pods: ⚠️ 45/48 Running (inline)
    - Services: ✅ 12/12 Active (inline)

    Description:
    All core services healthy. 3 pods pending in staging namespace.

    Buttons: [Refresh] [View Pods] [View Logs]
```

### Kubernetes Ops Agent

```yaml
# agents/k8s-ops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    platform: discord
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
    You are a Kubernetes specialist in Discord.

    ## Response Guidelines
    - Use embed format for structured data
    - Color code by severity:
      - 0x00FF00 (green) - All healthy
      - 0xFFFF00 (yellow) - Warnings present
      - 0xFF0000 (red) - Errors/failures
    - Show pod status in table format
    - Highlight issues prominently

    ## Pod Status Format
    Title: 📦 Pods in {namespace}

    ```
    Pod              Status      Restarts  Age
    ─────────────────────────────────────────
    api-abc123       ✅ Running  0         2d
    web-xyz789       ⚠️ Pending  0         5m
    worker-def456    ❌ Error    5         1h
    ```

    Footer: {count} pods | {healthy} healthy | {issues} issues

    Buttons: [Describe] [Logs] [Events] [Refresh]

    ## Component Handling
    When receiving component:approve_restart:
    - Execute kubectl rollout restart
    - Update message with result
```

### Deployer Agent

```yaml
# agents/deployer.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: deployer
  labels:
    platform: discord
    specialty: deployments
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "Deployment manager"

  tools:
    - kubectl
    - helm
    - argocd

  system_prompt: |
    You are a deployment manager in Discord.

    ## Deployment Request Format
    Title: 🚀 Deployment Request

    Fields:
    - Application: {app} (inline)
    - Version: {current} → {new} (inline)
    - Environment: {env} (inline)

    Description:
    Changes:
    • Feature: New user dashboard
    • Fix: Login timeout issue

    Requested by: @{user}

    Color: 0x0099FF (blue for pending)

    Buttons:
    [✅ Approve] [❌ Reject] [📋 View Changes]

    ## Post-Deployment
    Title: ✅ Deployment Complete

    Fields:
    - Duration: 45s
    - Replicas: 3/3 Ready
    - Status: ✅ Healthy

    Color: 0x00FF00 (green)

    Buttons: [View Logs] [Rollback]
```

## Step 3: RCA Fleet

```yaml
# fleets/rca-fleet.yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: rca-fleet
  labels:
    platform: discord
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
    Title: 🔍 Root Cause Analysis

    Fields:
    - Severity: {severity} (inline)
    - Duration: {duration} (inline)
    - Services: {affected_count} affected (inline)

    Description:
    **Summary:** {summary}

    **Root Cause:**
    {root_cause}

    **Timeline:**
    {timeline}

    **Remediation:**
    {remediation_steps}

    Color: Based on severity

    Buttons: [Apply Fix] [View Details] [Dismiss]
```

## Step 4: Custom Slash Commands

Define more detailed commands with options:

```yaml
# Full command definitions
commands:
  - name: agent
    description: "Manage AOF agents"
    options:
      - name: action
        type: 3  # STRING
        description: "Action to perform"
        required: true
        choices:
          - name: run
            value: run
          - name: status
            value: status
          - name: stop
            value: stop
      - name: agent_id
        type: 3
        description: "Agent ID"
        required: true

  - name: deploy
    description: "Deploy application"
    options:
      - name: application
        type: 3
        description: "Application name"
        required: true
        autocomplete: true
      - name: version
        type: 3
        description: "Version to deploy"
        required: true
      - name: environment
        type: 3
        description: "Target environment"
        required: true
        choices:
          - name: development
            value: dev
          - name: staging
            value: staging
          - name: production
            value: prod

  - name: scale
    description: "Scale deployment"
    options:
      - name: deployment
        type: 3
        description: "Deployment name"
        required: true
      - name: replicas
        type: 4  # INTEGER
        description: "Number of replicas"
        required: true
        min_value: 0
        max_value: 100
```

## Step 5: Component Handlers

Handle button clicks and select menus:

```yaml
# In agent system prompt
system_prompt: |
  ## Component Handling
  When you receive a message with "component:" prefix,
  it's a button or select menu interaction.

  Handle these component IDs:
  - component:refresh_status → Re-run status check
  - component:view_pods → List all pods
  - component:view_logs → Show recent logs
  - component:approve_deploy → Execute deployment
  - component:reject_deploy → Cancel deployment
  - component:apply_fix → Apply recommended fix
  - component:rollback → Rollback last deployment

  Response format for component interactions:
  - Update the original message if possible
  - Use ephemeral message for confirmations
  - Add new buttons for next actions
```

## Step 6: Embed Templates

### Status Dashboard

```json
{
  "embeds": [{
    "title": "📊 Cluster Dashboard",
    "description": "Real-time cluster health overview",
    "color": 65280,
    "fields": [
      { "name": "🖥️ Nodes", "value": "✅ 3/3 Ready", "inline": true },
      { "name": "📦 Pods", "value": "⚠️ 45/48 Running", "inline": true },
      { "name": "🔗 Services", "value": "✅ 12/12 Active", "inline": true },
      { "name": "💾 PVCs", "value": "✅ 8/8 Bound", "inline": true },
      { "name": "🔐 Secrets", "value": "✅ 15 Active", "inline": true },
      { "name": "⚙️ ConfigMaps", "value": "✅ 23 Active", "inline": true }
    ],
    "footer": { "text": "Last updated" },
    "timestamp": "2024-01-15T10:30:00.000Z"
  }],
  "components": [{
    "type": 1,
    "components": [
      { "type": 2, "style": 1, "label": "🔄 Refresh", "custom_id": "refresh_status" },
      { "type": 2, "style": 2, "label": "📦 Pods", "custom_id": "view_pods" },
      { "type": 2, "style": 2, "label": "📋 Logs", "custom_id": "view_logs" },
      { "type": 2, "style": 2, "label": "⚠️ Alerts", "custom_id": "view_alerts" }
    ]
  }]
}
```

### Deployment Approval

```json
{
  "embeds": [{
    "title": "🚀 Deployment Approval Required",
    "description": "A new deployment is waiting for approval",
    "color": 255,
    "fields": [
      { "name": "Application", "value": "api-server", "inline": true },
      { "name": "Version", "value": "v2.1.0 → v2.2.0", "inline": true },
      { "name": "Environment", "value": "production", "inline": true },
      { "name": "Requested By", "value": "<@123456789>", "inline": true },
      { "name": "Changes", "value": "• Bug fixes\n• Performance improvements", "inline": false }
    ],
    "footer": { "text": "Deployment ID: deploy-abc123" }
  }],
  "components": [
    {
      "type": 1,
      "components": [
        { "type": 2, "style": 3, "label": "✅ Approve", "custom_id": "approve_deploy_abc123" },
        { "type": 2, "style": 4, "label": "❌ Reject", "custom_id": "reject_deploy_abc123" },
        { "type": 2, "style": 2, "label": "📋 View Changes", "custom_id": "view_changes_abc123" }
      ]
    }
  ]
}
```

## Step 7: Role-Based Access

Implement role checks in your configuration:

```yaml
# triggers/discord-ops.yaml
spec:
  config:
    # Role IDs from Discord server
    allowed_roles:
      - "111111111111111111"  # Admin
      - "222222222222222222"  # DevOps
      - "333333333333333333"  # SRE

  # Command-specific role requirements
  commands:
    /deploy:
      agent: deployer
      description: "Deploy application"
      required_roles:
        - "111111111111111111"  # Admin only
        - "222222222222222222"  # DevOps

    /status:
      agent: k8s-ops
      description: "View status"
      # No role restriction - all allowed_roles can use
```

## Step 8: Testing

### Test Slash Commands

```
/help
/status
/pods namespace:default
/deploy application:api-server version:v2.2.0 environment:staging
```

### Test Button Interactions

1. Run `/status` to get a status embed
2. Click the "Refresh" button
3. Verify the embed updates

### Test Deployment Flow

1. Run `/deploy application:test version:v1.0.0 environment:dev`
2. See approval embed appear
3. Click "Approve" button
4. Verify deployment proceeds

## Step 9: Production Deployment

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/aofctl /usr/local/bin/
COPY config/ /app/config/
WORKDIR /app
CMD ["aofctl", "daemon", "start"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-discord-bot
spec:
  replicas: 2
  selector:
    matchLabels:
      app: aof-discord-bot
  template:
    metadata:
      labels:
        app: aof-discord-bot
    spec:
      containers:
        - name: aofctl
          image: aof/aofctl:latest
          ports:
            - containerPort: 8080
          env:
            - name: DISCORD_BOT_TOKEN
              valueFrom:
                secretKeyRef:
                  name: discord-credentials
                  key: bot_token
            - name: DISCORD_APPLICATION_ID
              valueFrom:
                secretKeyRef:
                  name: discord-credentials
                  key: application_id
            - name: DISCORD_PUBLIC_KEY
              valueFrom:
                secretKeyRef:
                  name: discord-credentials
                  key: public_key
          volumeMounts:
            - name: config
              mountPath: /app/.aof
      volumes:
        - name: config
          configMap:
            name: aof-config
```

## Best Practices

### 1. Embed Design

- Use consistent color coding
- Keep embeds scannable
- Limit fields to essential info
- Use inline fields for related data

### 2. Button Organization

- Max 5 buttons per row
- Group related actions
- Use appropriate styles (colors)
- Clear, concise labels

### 3. Error Handling

- Show user-friendly error messages
- Include troubleshooting hints
- Log errors for debugging
- Provide retry options

### 4. Performance

- Respond within 3 seconds
- Use deferred responses for long operations
- Cache frequently accessed data
- Batch API calls when possible

## Next Steps

- [Discord Reference](../reference/discord-integration.md) - Full API reference
- [Fleet Configuration](../reference/fleet-spec.md) - Multi-agent coordination
- [Custom Tools](../tools/custom-tools.md) - Add your own tools
- [Deployment Guide](../guides/deployment.md) - Production deployment
