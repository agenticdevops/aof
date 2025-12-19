# Enterprise Setup Guide

This guide shows how to deploy AOF in enterprise environments with multiple clusters, organizations, and environments using best practices for security, isolation, and scalability.

## Architecture Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                     Enterprise AOF Deployment                      │
├───────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐           │
│  │   Slack     │    │  Telegram   │    │  WhatsApp   │           │
│  │  Workspace  │    │    Bots     │    │  Business   │           │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘           │
│         │                  │                  │                   │
│         └──────────────────┴──────────────────┘                   │
│                            │                                      │
│                     ┌──────▼────────┐                             │
│                     │  AOF Daemon   │                             │
│                     │  (aofctl)     │                             │
│                     └──────┬────────┘                             │
│                            │                                      │
│         ┌──────────────────┼──────────────────┐                   │
│         │                  │                  │                   │
│    ┌────▼────┐       ┌────▼────┐       ┌────▼────┐               │
│    │ Prod    │       │ Staging │       │   Dev   │               │
│    │ K8s     │       │  K8s    │       │  K8s    │               │
│    │ Cluster │       │ Cluster │       │ Cluster │               │
│    └─────────┘       └─────────┘       └─────────┘               │
│                                                                    │
│    ┌─────────────────────────────────────────────────┐            │
│    │         PostgreSQL Memory Backend                │            │
│    │  - Conversation history                          │            │
│    │  - Audit logs                                   │            │
│    │  - Approval tracking                             │            │
│    └─────────────────────────────────────────────────┘            │
│                                                                    │
└───────────────────────────────────────────────────────────────────┘
```

## Directory Structure

For enterprise deployments, organize your configuration like this:

```
/etc/aof/                                    # System-wide config
├── daemon.yaml                              # Main daemon config
├── secrets/                                 # Credentials (gitignored)
│   ├── .env.prod                           # Production secrets
│   ├── .env.staging                        # Staging secrets
│   └── .env.dev                            # Development secrets
│
├── agents/                                  # Agent definitions
│   ├── k8s-ops.yaml                        # Kubernetes operations
│   ├── incident-responder.yaml             # Incident management
│   ├── security-scanner.yaml               # Security auditing
│   └── cost-optimizer.yaml                 # Resource optimization
│
├── flows/                                   # AgentFlow configurations
│   ├── prod/                               # Production flows
│   │   ├── slack-k8s-bot.yaml             # Prod Slack bot
│   │   ├── telegram-ops.yaml              # Prod Telegram bot
│   │   └── incident-response.yaml          # Auto-remediation
│   │
│   ├── staging/                            # Staging flows
│   │   ├── slack-k8s-bot.yaml             # Staging Slack bot
│   │   └── deployment-testing.yaml         # Deploy validation
│   │
│   ├── dev/                                # Development flows
│   │   ├── slack-k8s-bot.yaml             # Dev Slack bot
│   │   └── debug-helper.yaml              # Developer tools
│   │
│   └── shared/                             # Shared workflows
│       ├── cost-report-flow.yaml          # Weekly cost reports
│       └── security-scan-flow.yaml         # Nightly security scans
│
├── contexts/                                # Environment contexts (future)
│   ├── prod-us-east.yaml                   # Prod US East cluster
│   ├── prod-eu-west.yaml                   # Prod EU West cluster
│   ├── staging.yaml                        # Staging environment
│   └── dev.yaml                            # Development environment
│
└── triggers/                                # Trigger definitions with command routing
    ├── slack-prod.yaml                     # Slack production (routes to prod flows)
    ├── slack-staging.yaml                  # Slack staging
    ├── telegram-oncall.yaml                # Telegram for on-call
    └── webhooks.yaml                       # HTTP webhooks for CI/CD
```

## Setup 1: Multi-Environment (Dev/Staging/Prod)

### Step 1: Create Environment-Specific Contexts

**File: `contexts/prod-cluster.yaml`**
```yaml
apiVersion: aof.dev/v1alpha2  # Future syntax
kind: Context
metadata:
  name: prod-us-east-1
  labels:
    environment: production
    region: us-east-1
    criticality: high
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  cluster: prod-us-east-1

  env:
    ENVIRONMENT: "production"
    REQUIRE_APPROVAL: "true"
    APPROVAL_TIMEOUT: "300"
    MAX_REPLICAS: "100"
    ALERT_CHANNEL: "#prod-alerts"
    PAGERDUTY_ROUTING_KEY: ${PAGERDUTY_KEY_PROD}

  limits:
    max_requests_per_hour: 1000
    max_concurrent_tasks: 50
    max_tokens_per_request: 8192
```

**File: `contexts/staging-cluster.yaml`**
```yaml
apiVersion: aof.dev/v1alpha2
kind: Context
metadata:
  name: staging
  labels:
    environment: staging
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  namespace: staging
  cluster: staging-us-east-1

  env:
    ENVIRONMENT: "staging"
    REQUIRE_APPROVAL: "false"  # No approval in staging
    MAX_REPLICAS: "20"
    ALERT_CHANNEL: "#staging-alerts"

  limits:
    max_requests_per_hour: 500
```

**File: `contexts/dev-local.yaml`**
```yaml
apiVersion: aof.dev/v1alpha2
kind: Context
metadata:
  name: dev-local
  labels:
    environment: development
spec:
  kubeconfig: ~/.kube/minikube
  namespace: default
  cluster: minikube

  env:
    ENVIRONMENT: "development"
    REQUIRE_APPROVAL: "false"
    MAX_REPLICAS: "5"
    DEBUG: "true"
```

### Step 2: Create Environment-Specific Flows

**File: `flows/prod/slack-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-k8s-bot
  labels:
    environment: production
    platform: slack
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention, message]
      channels:
        - production        # #production channel only
        - prod-alerts
        - sre-oncall
      users:               # Restrict to SRE team
        - U015SRE_LEAD
        - U016SRE_ENG
        - U017PLATFORM
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  # Embedded context (until v1alpha2)
  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: production
    env:
      REQUIRE_APPROVAL: "true"
      APPROVAL_TIMEOUT: "300"

  approval:
    allowed_users:
      - U015SRE_LEAD      # Only SRE lead can approve
    require_for:
      - "kubectl delete"
      - "kubectl scale --replicas=0"
      - "helm uninstall"
      - "kubectl drain"

  nodes:
    - id: parse
      type: Transform
      config:
        script: |
          export MESSAGE="${event.text}"
          export CHANNEL="${event.channel}"
          export USER_ID="${event.user}"
          export THREAD="${event.ts}"

    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${MESSAGE}
        context:
          cluster: "prod-us-east-1"
          environment: "production"

    - id: approval-check
      type: Conditional
      config:
        condition: ${agent.requires_approval}
        routes:
          true: request-approval
          false: respond

    - id: request-approval
      type: Slack
      config:
        channel: ${CHANNEL}
        thread_ts: ${THREAD}
        message: |
          ⚠️ **Approval Required**

          Command: `${agent.command}`
          Reason: ${agent.reason}
          Requested by: <@${USER_ID}>

          React with ✅ to approve or ❌ to deny.
        reactions: ["white_check_mark", "x"]

    - id: wait-approval
      type: Wait
      config:
        for: reaction
        timeout: 300
        allowed_reactors: ${approval.allowed_users}

    - id: execute-approved
      type: Agent
      config:
        agent: k8s-ops
        input: "Execute: ${agent.command}"
        execute_directly: true

    - id: respond
      type: Slack
      config:
        channel: ${CHANNEL}
        thread_ts: ${THREAD}
        message: |
          ${execute-approved.output}

          Executed by: <@${USER_ID}>
          Approved by: <@${wait-approval.approver}>

  connections:
    - from: trigger
      to: parse
    - from: parse
      to: agent
    - from: agent
      to: approval-check
    - from: approval-check
      to: request-approval
      condition: requires_approval
    - from: approval-check
      to: respond
      condition: !requires_approval
    - from: request-approval
      to: wait-approval
    - from: wait-approval
      to: execute-approved
    - from: execute-approved
      to: respond
```

**File: `flows/staging/slack-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-staging-k8s-bot
  labels:
    environment: staging
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention, message]
      channels: [staging, staging-test]  # Different channels
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  context:
    kubeconfig: ${KUBECONFIG_STAGING}
    namespace: staging
    env:
      REQUIRE_APPROVAL: "false"  # No approval in staging

  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${event.text}

    - id: respond
      type: Slack
      config:
        channel: ${event.channel}
        thread_ts: ${event.ts}
        message: ${agent.output}

  connections:
    - from: trigger
      to: agent
    - from: agent
      to: respond
```

### Step 3: Configure Environment Variables

**File: `secrets/.env.prod`**
```bash
# Production Environment
GOOGLE_API_KEY=AIza...
SLACK_BOT_TOKEN=xoxb-prod-token
SLACK_SIGNING_SECRET=prod-secret
KUBECONFIG_PROD=/etc/aof/kubeconfig/prod-us-east-1.yaml
PAGERDUTY_KEY_PROD=R012PROD
```

**File: `secrets/.env.staging`**
```bash
# Staging Environment
GOOGLE_API_KEY=AIza...
SLACK_BOT_TOKEN=xoxb-staging-token
SLACK_SIGNING_SECRET=staging-secret
KUBECONFIG_STAGING=/etc/aof/kubeconfig/staging.yaml
```

**File: `secrets/.env.dev`**
```bash
# Development Environment
GOOGLE_API_KEY=AIza...
SLACK_BOT_TOKEN=xoxb-dev-token
SLACK_SIGNING_SECRET=dev-secret
KUBECONFIG_DEV=~/.kube/minikube
```

### Step 4: Start Daemon

```bash
# Source environment
source /etc/aof/secrets/.env.prod

# Start daemon
aofctl serve \
  --config /etc/aof/daemon.yaml \
  --agents-dir /etc/aof/agents \
  --flows-dir /etc/aof/flows \
  --port 3000 \
  --log-level info \
  --log-file /var/log/aof/daemon.log
```

## Setup 2: Multi-Cluster Kubernetes

Route different channels to different Kubernetes clusters:

### Directory Structure

```
flows/
├── prod-us-east/
│   └── slack-k8s-bot.yaml    # Routes #prod-us-east → US East cluster
├── prod-eu-west/
│   └── slack-k8s-bot.yaml    # Routes #prod-eu-west → EU West cluster
└── prod-ap-south/
    └── slack-k8s-bot.yaml    # Routes #prod-ap-south → AP South cluster
```

### Configuration Per Cluster

**File: `flows/prod-us-east/slack-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-us-east-k8s
  labels:
    region: us-east-1
spec:
  trigger:
    type: Slack
    config:
      channels: [prod-us-east, us-east-alerts]  # Specific channels
      bot_token: ${SLACK_BOT_TOKEN}

  context:
    kubeconfig: ${KUBECONFIG_PROD_US_EAST}
    cluster: prod-us-east-1
    region: us-east-1

  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${event.text}
        context:
          cluster: "prod-us-east-1"
```

**File: `flows/prod-eu-west/slack-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-eu-west-k8s
  labels:
    region: eu-west-1
spec:
  trigger:
    type: Slack
    config:
      channels: [prod-eu-west, eu-west-alerts]  # Different channels
      bot_token: ${SLACK_BOT_TOKEN}

  context:
    kubeconfig: ${KUBECONFIG_PROD_EU_WEST}
    cluster: prod-eu-west-1
    region: eu-west-1

  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${event.text}
        context:
          cluster: "prod-eu-west-1"
```

## Setup 3: Multi-Organization

Support multiple organizations with separate Slack workspaces and clusters:

```
flows/
├── org-acme/
│   ├── slack-k8s-bot.yaml       # Acme Corp workspace
│   └── incident-flow.yaml
├── org-globex/
│   ├── slack-k8s-bot.yaml       # Globex workspace
│   └── incident-flow.yaml
└── org-initech/
    ├── slack-k8s-bot.yaml       # Initech workspace
    └── incident-flow.yaml
```

**File: `flows/org-acme/slack-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: acme-slack-k8s-bot
  labels:
    organization: acme
    tier: enterprise
spec:
  trigger:
    type: Slack
    config:
      # Acme's Slack workspace
      bot_token: ${SLACK_BOT_TOKEN_ACME}
      signing_secret: ${SLACK_SIGNING_SECRET_ACME}
      channels: [production, staging]

  context:
    kubeconfig: ${KUBECONFIG_ACME}
    organization: acme
    env:
      ORG_ID: "acme-corp"
      BILLING_ACCOUNT: "acct-acme-001"

  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${event.text}

  limits:
    max_requests_per_hour: 1000
    max_tokens_per_request: 8192
```

**Environment Variables:**
```bash
# Acme Corp
export SLACK_BOT_TOKEN_ACME=xoxb-acme-token
export SLACK_SIGNING_SECRET_ACME=acme-secret
export KUBECONFIG_ACME=/etc/aof/kubeconfig/acme.yaml

# Globex Corp
export SLACK_BOT_TOKEN_GLOBEX=xoxb-globex-token
export SLACK_SIGNING_SECRET_GLOBEX=globex-secret
export KUBECONFIG_GLOBEX=/etc/aof/kubeconfig/globex.yaml
```

## Security Best Practices

### 1. Secret Management

**Use environment variables, never hardcode:**
```yaml
# ❌ WRONG - Hardcoded secret
bot_token: "xoxb-123456789-abcdefg"

# ✅ CORRECT - Environment variable
bot_token: ${SLACK_BOT_TOKEN}
```

**Use a secrets manager in production:**
```bash
# AWS Secrets Manager
export SLACK_BOT_TOKEN=$(aws secretsmanager get-secret-value \
  --secret-id prod/aof/slack-bot-token \
  --query SecretString --output text)

# HashiCorp Vault
export SLACK_BOT_TOKEN=$(vault kv get -field=token secret/aof/slack)

# Kubernetes Secrets (if running in K8s)
# Mount secrets as environment variables via deployment spec
```

### 2. Access Control Layers

Implement defense-in-depth:

```
┌─────────────────────────────────────────┐
│ Layer 1: Platform Authentication        │
│ - Slack bot token validation            │
│ - Telegram chat ID verification         │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ Layer 2: Flow Routing                   │
│ - Channel whitelisting                  │
│ - User ID filtering                     │
│ - Pattern matching                      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ Layer 3: Approval Workflow              │
│ - Authorized approver list              │
│ - Command pattern matching              │
│ - Timeout enforcement                   │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ Layer 4: Tool Safety                    │
│ - Command allowlist/blocklist           │
│ - Dry-run mode                          │
│ - Read-only operations                  │
└─────────────────────────────────────────┘
```

### 3. User Whitelisting

**Restrict flows to specific users:**
```yaml
spec:
  trigger:
    config:
      users:
        - U015ADMIN      # Admin 1
        - U016ADMIN      # Admin 2
        - U017SRE        # SRE engineer
```

**Restrict approvers:**
```yaml
spec:
  approval:
    allowed_users:
      - U015ADMIN      # Only admins can approve
```

### 4. Command Restrictions

**Block dangerous commands:**
```yaml
spec:
  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        tool_config:
          kubectl:
            blocked_commands:
              - "delete namespace"
              - "delete pvc"
              - "drain --force"
          shell:
            blocked_commands:
              - "rm -rf /"
              - "mkfs"
```

**Allow only specific commands:**
```yaml
spec:
  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        tool_config:
          kubectl:
            allowed_commands:
              - "get"
              - "describe"
              - "logs"
              - "top"
            # Everything else is denied
```

## Audit Logging

### Enable Comprehensive Logging

**File: `daemon.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
spec:
  logging:
    level: info
    format: json
    destination: file
    file:
      path: /var/log/aof/daemon.log
      max_size_mb: 100
      max_backups: 10
      compress: true

  audit:
    enabled: true
    log_all_requests: true
    log_tool_executions: true
    log_approvals: true
    destination: postgresql
    postgresql:
      url: ${AUDIT_DB_URL}
      table: audit_logs
```

### Audit Log Schema

```sql
CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    organization TEXT,
    environment TEXT,
    flow_name TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    channel TEXT,
    user_id TEXT NOT NULL,
    user_name TEXT,
    message TEXT,
    command TEXT,
    requires_approval BOOLEAN,
    approver_id TEXT,
    approval_status TEXT,
    execution_status TEXT,
    latency_ms INTEGER,
    tokens_used INTEGER,
    cost_usd NUMERIC(10,4),
    error_message TEXT,
    metadata JSONB
);

CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_org ON audit_logs(organization);
```

### Query Audit Logs

```bash
# List all commands by user
aofctl audit logs --user U015ADMIN --since 24h

# Show approval history
aofctl audit approvals --status approved --since 7d

# Cost analysis
aofctl audit cost --by organization --since 30d
```

## Rate Limiting

### Per-Organization Limits

**File: `daemon.yaml`**
```yaml
spec:
  limits:
    global:
      max_requests_per_minute: 100
      max_concurrent_tasks: 50

    per_organization:
      acme:
        max_requests_per_hour: 1000
        max_tokens_per_day: 1000000
        max_concurrent_tasks: 20
      globex:
        max_requests_per_hour: 500
        max_tokens_per_day: 500000
        max_concurrent_tasks: 10

    per_user:
      max_requests_per_minute: 10
      max_requests_per_hour: 100
```

## Monitoring & Alerting

### Metrics to Track

```yaml
# Prometheus metrics
- agentflow_requests_total{flow, platform, agent, status}
- agentflow_latency_seconds{flow, agent, percentile}
- agentflow_approvals_total{flow, status}
- agentflow_errors_total{flow, error_type}
- agentflow_tokens_used{flow, agent}
- agentflow_cost_usd{organization, flow}

# Alert rules
- agentflow_error_rate > 5%
- agentflow_latency_p99 > 10s
- agentflow_approval_timeout_rate > 10%
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "AOF Enterprise Monitoring",
    "panels": [
      {
        "title": "Requests by Organization",
        "targets": [
          "sum(rate(agentflow_requests_total[5m])) by (organization)"
        ]
      },
      {
        "title": "Approval Rate",
        "targets": [
          "sum(rate(agentflow_approvals_total{status='approved'}[5m])) / sum(rate(agentflow_approvals_total[5m]))"
        ]
      },
      {
        "title": "Cost by Organization",
        "targets": [
          "sum(increase(agentflow_cost_usd[24h])) by (organization)"
        ]
      }
    ]
  }
}
```

## High Availability

### Run Multiple Instances

**Behind a load balancer:**
```bash
# Instance 1
aofctl serve --config daemon.yaml --node-id node-1 --redis ${REDIS_URL}

# Instance 2
aofctl serve --config daemon.yaml --node-id node-2 --redis ${REDIS_URL}

# Instance 3
aofctl serve --config daemon.yaml --node-id node-3 --redis ${REDIS_URL}
```

**Docker Compose:**
```yaml
version: '3.8'
services:
  aof-1:
    image: aof:latest
    command: serve --config /etc/aof/daemon.yaml --node-id node-1
    environment:
      - REDIS_URL=redis://redis:6379
    volumes:
      - /etc/aof:/etc/aof:ro

  aof-2:
    image: aof:latest
    command: serve --config /etc/aof/daemon.yaml --node-id node-2
    environment:
      - REDIS_URL=redis://redis:6379
    volumes:
      - /etc/aof:/etc/aof:ro

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data

  loadbalancer:
    image: nginx:alpine
    ports:
      - "3000:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - aof-1
      - aof-2
```

## Backup & Recovery

### Backup Configuration

```bash
#!/bin/bash
# backup-aof-config.sh
BACKUP_DIR="/backups/aof/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup configs
tar czf "$BACKUP_DIR/config.tar.gz" \
  /etc/aof/agents \
  /etc/aof/flows \
  /etc/aof/contexts \
  /etc/aof/daemon.yaml

# Backup database
pg_dump $AUDIT_DB_URL > "$BACKUP_DIR/audit_logs.sql"

# Upload to S3
aws s3 cp "$BACKUP_DIR" s3://aof-backups/$(date +%Y%m%d)/ --recursive
```

### Restore Procedure

```bash
# 1. Stop daemon
systemctl stop aof

# 2. Restore configs
tar xzf backup/config.tar.gz -C /

# 3. Restore database
psql $AUDIT_DB_URL < backup/audit_logs.sql

# 4. Verify configs
aofctl validate --config /etc/aof/daemon.yaml

# 5. Restart daemon
systemctl start aof
```

## Migration Path

### From Single-Tenant to Multi-Tenant

**Week 1: Setup Infrastructure**
1. Create directory structure
2. Set up PostgreSQL for memory/audit
3. Configure monitoring
4. Set up secrets management

**Week 2: Migration**
1. Convert existing agent to flows/
2. Add channel/user filters
3. Test in staging
4. Deploy to production

**Week 3: Expansion**
1. Add more environments (staging, dev)
2. Create organization-specific flows
3. Implement approval workflows
4. Set up rate limiting

**Week 4: Optimization**
1. Tune performance
2. Add dashboards
3. Set up alerts
4. Document runbooks

## Related Documentation

- [Core Concepts](../introduction/concepts.md) - Resource types
- [Multi-Tenant Architecture](../architecture/multi-tenant-agentflows.md) - Design patterns
- [Approval Workflow Guide](approval-workflow.md) - Human-in-the-loop
- [Deployment Guide](deployment.md) - Production deployment

---

**Questions?** Open an issue at [github.com/agenticdevops/aof/issues](https://github.com/agenticdevops/aof/issues)
