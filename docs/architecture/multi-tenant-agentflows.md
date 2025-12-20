---
sidebar_label: Multi-Tenant Flows
sidebar_position: 5
---

# Multi-Tenant AgentFlow Architecture

This document describes the architecture for running multiple AgentFlows simultaneously to support different projects, bots, organizations, and divisions within a single AOF deployment.

## Overview

AOF supports **multi-tenant AgentFlow deployments** where a single daemon can route messages to different agents based on:

- **Platform** (Slack, Telegram, Discord, WhatsApp)
- **Channel/Group** (production vs staging, team-specific channels)
- **User/Role** (admins, SRE team, developers)
- **Pattern** (kubectl commands, deploy requests, incident reports)
- **Organization** (multi-org enterprise deployments)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         AOF Daemon (aofctl serve)                    │
│                                                                      │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                │
│  │   Slack     │   │  Telegram   │   │  Discord    │   ...more     │
│  │  Platform   │   │  Platform   │   │  Platform   │                │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                │
│         │                 │                 │                        │
│         └────────────────┬┴─────────────────┘                        │
│                          │                                           │
│                   ┌──────▼──────┐                                    │
│                   │ FlowRouter  │  ← Pattern/Channel/User matching   │
│                   └──────┬──────┘                                    │
│                          │                                           │
│    ┌─────────────────────┼─────────────────────┐                    │
│    │                     │                     │                     │
│    ▼                     ▼                     ▼                     │
│ ┌──────────┐      ┌──────────┐          ┌──────────┐                │
│ │ AgentFlow│      │ AgentFlow│          │ AgentFlow│                │
│ │ prod-k8s │      │ staging  │          │ incident │                │
│ └────┬─────┘      └────┬─────┘          └────┬─────┘                │
│      │                 │                     │                       │
│      ▼                 ▼                     ▼                       │
│ ┌────────┐        ┌────────┐           ┌────────┐                   │
│ │ k8s-ops│        │ dev-ops│           │incident│                   │
│ │ Agent  │        │ Agent  │           │ Agent  │                   │
│ └────────┘        └────────┘           └────────┘                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### 1. AgentFlow Routing

Each AgentFlow defines:
- **Trigger filters** - Which messages it handles
- **Context** - Environment variables, kubeconfig, namespace
- **Agents** - Which agent(s) process matched messages
- **Approval rules** - Who can approve destructive commands

### 2. Routing Priority

When multiple AgentFlows could match a message, the FlowRouter uses scoring:

| Factor | Weight | Description |
|--------|--------|-------------|
| Exact channel match | 100 | Message from configured channel |
| User whitelist match | 80 | Message from allowed user |
| Pattern match | 60 | Regex pattern matches message |
| Platform match | 40 | Correct platform type |
| Default (no filters) | 10 | Catch-all flow |

Higher score wins. First match breaks ties.

### 3. Flow Isolation

Each AgentFlow is isolated with its own:
- **Execution context** (environment variables)
- **Agent configuration** (model, tools, system prompt)
- **Approval workflow** (allowed approvers)
- **Memory namespace** (conversation history)

## Directory Structure

```
aof-deployment/
├── daemon-config.yaml          # Main daemon configuration
├── agents/                     # Agent definitions
│   ├── k8s-ops.yaml           # Kubernetes operations agent
│   ├── incident-responder.yaml # Incident response agent
│   ├── dev-assistant.yaml     # Developer assistant agent
│   └── security-scanner.yaml  # Security scanning agent
│
└── flows/                      # AgentFlow definitions
    ├── prod-cluster/
    │   ├── k8s-flow.yaml      # Production K8s ops
    │   └── incident-flow.yaml # Production incidents
    │
    ├── staging-cluster/
    │   └── dev-flow.yaml      # Staging environment
    │
    └── enterprise/
        ├── org-a-flow.yaml    # Organization A
        └── org-b-flow.yaml    # Organization B
```

## Configuration Patterns

### Pattern 1: Channel-Based Routing

Route different Slack channels to different clusters:

```yaml
# flows/prod-cluster/k8s-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: prod-k8s-flow
  labels:
    environment: production
    cluster: prod-us-east-1
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention, message]
      channels: [production, prod-alerts, sre-oncall]  # Only these channels
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: default
    cluster_name: prod-us-east-1
    env:
      REQUIRE_APPROVAL: "true"
      APPROVAL_TIMEOUT: "300"

  agents:
    - name: k8s-ops
      patterns: ["kubectl", "k8s", "pod", "deploy", "scale"]
    - name: incident-responder
      patterns: ["incident", "outage", "alert", "pagerduty"]

  approval:
    allowed_users:
      - U015VBH1GTZ    # SRE Lead
      - U012ADMIN      # Platform Admin
    require_for:
      - kubectl delete
      - kubectl scale --replicas=0
      - helm uninstall
```

```yaml
# flows/staging-cluster/dev-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: staging-dev-flow
  labels:
    environment: staging
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention, message]
      channels: [staging, dev-test, qa-testing]
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  context:
    kubeconfig: ${KUBECONFIG_STAGING}
    namespace: staging
    env:
      REQUIRE_APPROVAL: "false"  # No approval needed in staging

  agents:
    - name: dev-assistant
```

### Pattern 2: User-Based Routing

Route based on user roles:

```yaml
# flows/enterprise/admin-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: admin-only-flow
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      users: [U015ADMIN, U016ADMIN, U017ADMIN]  # Admin users only

  agents:
    - name: admin-agent
      tools: [kubectl, helm, terraform, aws]  # Full access

  approval:
    allowed_users: [U015ADMIN]  # Self-approval for admins
```

```yaml
# flows/enterprise/developer-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: developer-flow
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      # No user filter = all users (developers get this)

  agents:
    - name: dev-assistant
      tools: [kubectl]  # Read-only kubectl
      tool_config:
        kubectl:
          allowed_commands: [get, describe, logs]  # No destructive ops
```

### Pattern 3: Pattern-Based Routing

Route based on message content:

```yaml
# flows/incident-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-flow
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      patterns:
        - "^(incident|outage|alert|page)"
        - "CRITICAL|HIGH|URGENT"
        - "PagerDuty|OpsGenie"

  agents:
    - name: incident-responder
      priority: critical
      tools: [kubectl, prometheus_query, loki_query, pagerduty]
```

### Pattern 4: Multi-Platform Same Agent

Share agents across platforms:

```yaml
# flows/multi-platform/shared-assistant.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: shared-assistant-slack
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  agents:
    - name: general-assistant
---
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: shared-assistant-telegram
spec:
  trigger:
    type: Telegram
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}

  agents:
    - name: general-assistant  # Same agent, different platform
---
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: shared-assistant-discord
spec:
  trigger:
    type: Discord
    config:
      bot_token: ${DISCORD_BOT_TOKEN}

  agents:
    - name: general-assistant
```

### Pattern 5: Enterprise Multi-Org

Support multiple organizations in single deployment:

```yaml
# flows/enterprise/org-a-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: org-a-flow
  labels:
    organization: org-a
    tier: enterprise
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      # Org A's Slack workspace
      bot_token: ${SLACK_BOT_TOKEN_ORG_A}
      signing_secret: ${SLACK_SIGNING_SECRET_ORG_A}

  context:
    organization: org-a
    kubeconfig: ${KUBECONFIG_ORG_A}
    env:
      ORG_ID: "org-a"
      BILLING_ACCOUNT: "acct-001"

  agents:
    - name: org-a-assistant

  limits:
    max_requests_per_hour: 1000
    max_tokens_per_request: 4096
```

## Daemon Configuration

The main daemon config enables multi-tenant routing:

```yaml
# daemon-config.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-tenant-server
spec:
  server:
    port: 8080
    host: "0.0.0.0"

  platforms:
    slack:
      enabled: true
      bot_token_env: "SLACK_BOT_TOKEN"
      signing_secret_env: "SLACK_SIGNING_SECRET"

    telegram:
      enabled: true
      bot_token_env: "TELEGRAM_BOT_TOKEN"

    discord:
      enabled: true
      bot_token_env: "DISCORD_BOT_TOKEN"

  # Agent discovery
  agents:
    directory: "./agents"
    watch: true  # Hot-reload agent changes

  # AgentFlow routing
  flows:
    enabled: true
    directory: "./flows"
    watch: true  # Hot-reload flow changes
    recursive: true  # Scan subdirectories

  routing:
    default_agent: "general-assistant"  # Fallback if no flow matches
    strict_mode: false  # Allow unmatched messages to use default

  runtime:
    max_concurrent_tasks: 50
    task_timeout_secs: 300
    max_tasks_per_user: 5
```

## Message Flow

1. **Message arrives** from platform (Slack, Telegram, etc.)
2. **FlowRouter.route_best()** finds the best matching AgentFlow:
   - Check channel filters
   - Check user filters
   - Check pattern filters
   - Score and rank matches
3. **AgentFlow executes**:
   - Apply context (env vars, kubeconfig)
   - Select agent based on message pattern
   - Execute agent with tools
4. **Approval workflow** (if destructive command):
   - Post approval message with reactions
   - Wait for authorized user reaction
   - Execute or deny based on reaction
5. **Response sent** back to platform

## Best Practices

### 1. Separation of Concerns

- One AgentFlow per environment (prod/staging/dev)
- One AgentFlow per use case (k8s ops, incidents, dev help)
- Shared agents across flows when appropriate

### 2. Security Layers

```
Platform Auth → FlowRouter → User Filter → Approval → Tool Safety
     │              │             │           │           │
  Bot token    Channel match   User ID    Reaction    Command
  validation   pattern match   whitelist  from admin  allowlist
```

### 3. Naming Conventions

| Resource | Convention | Example |
|----------|------------|---------|
| AgentFlow | `{env}-{purpose}-flow` | `prod-k8s-flow` |
| Agent | `{purpose}-agent` | `k8s-ops-agent` |
| Channel filter | Explicit list | `[production, prod-alerts]` |
| User filter | Slack/Platform IDs | `[U015VBH1GTZ]` |

### 4. Environment Isolation

Never mix production and staging in the same flow:

```yaml
# WRONG - Mixed environments
spec:
  trigger:
    config:
      channels: [production, staging]  # DON'T DO THIS

# CORRECT - Separate flows
# File: flows/prod/k8s-flow.yaml
spec:
  trigger:
    config:
      channels: [production]
  context:
    kubeconfig: ${KUBECONFIG_PROD}

# File: flows/staging/k8s-flow.yaml
spec:
  trigger:
    config:
      channels: [staging]
  context:
    kubeconfig: ${KUBECONFIG_STAGING}
```

### 5. Approval Chains

For enterprise, implement approval chains:

```yaml
spec:
  approval:
    chains:
      - name: standard
        approvers: [U015SRE, U016SRE]
        require: 1  # Any 1 approver

      - name: critical
        approvers: [U015LEAD, U016MANAGER]
        require: 2  # Both must approve

    rules:
      - pattern: "kubectl delete namespace"
        chain: critical
      - pattern: "kubectl delete"
        chain: standard
```

## Scaling Considerations

### Horizontal Scaling

For high-volume deployments:

```yaml
# Deploy multiple daemon instances behind load balancer
spec:
  server:
    cluster_mode: true
    redis_url: ${REDIS_URL}  # Shared state
    node_id: ${HOSTNAME}

  runtime:
    distributed: true
    task_queue: redis
```

### Rate Limiting

Per-organization limits:

```yaml
spec:
  limits:
    global:
      max_requests_per_minute: 100

    per_organization:
      max_requests_per_hour: 1000
      max_tokens_per_day: 1000000

    per_user:
      max_requests_per_minute: 10
```

## Monitoring

### Metrics to Track

- `agentflow_requests_total{flow, platform, agent}`
- `agentflow_latency_seconds{flow, agent}`
- `agentflow_approvals_total{flow, status}`
- `agentflow_errors_total{flow, error_type}`

### Logging

Each request logs:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "flow": "prod-k8s-flow",
  "platform": "slack",
  "channel": "production",
  "user": "U015VBH1GTZ",
  "agent": "k8s-ops",
  "action": "kubectl get pods",
  "latency_ms": 1250,
  "tokens_used": 512,
  "status": "success"
}
```

## Migration Path

### From Single-Agent to Multi-Tenant

1. **Create flows directory**
2. **Move existing agent to flows/**
3. **Add channel/user filters**
4. **Test with staging first**
5. **Gradually add more flows**

```bash
# Step 1: Create structure
mkdir -p flows/{prod,staging,enterprise}

# Step 2: Create initial flow from existing config
aofctl generate flow --from-agent agents/k8s-ops.yaml --output flows/prod/k8s-flow.yaml

# Step 3: Test
aofctl serve --config daemon-config.yaml --flows-dir ./flows
```

## Related Documentation

- [AgentFlow Reference](../reference/agentflow-spec.md)
- [Context Reference](../reference/context-spec.md)
- [Agent Configuration](../reference/agent-spec.md)
- [Trigger Reference](../reference/trigger-spec.md)
