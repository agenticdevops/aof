# Enterprise Multi-Tenant Example

This is a **production-ready, multi-tenant AOF setup** demonstrating the full composable architecture.

## Architecture

```
Production Environment
├── Slack (#production) → K8s Ops Flow → Production Cluster
├── Telegram (on-call) → Incident Flow → Production Cluster
└── PagerDuty Webhooks → Auto Incident Response

Staging Environment
└── Slack (#staging) → K8s Ops Flow → Staging Cluster

Development Environment
└── Slack (#dev) → K8s Ops Flow → Local Cluster
```

## What You Get

- **3 environments**: Production, Staging, Development
- **4 platforms**: Slack, Telegram, PagerDuty, WhatsApp
- **5 agents**: k8s-ops, security, incident, devops, general-assistant
- **2 fleets**: code-review-team, incident-team
- **3 flows**: k8s-ops, incident, deployment
- **Approval workflows**: Environment-specific approval rules
- **Audit logging**: Complete audit trail for production
- **Multi-tenant routing**: Same bot, different behaviors per environment

## Directory Structure

```
enterprise/
├── agents/           # Symlinks to ../../agents/*.yaml
├── fleets/           # Symlinks to ../../fleets/*.yaml
├── flows/            # Symlinks to ../../flows/*.yaml
├── contexts/         # Symlinks to ../../contexts/*.yaml
├── triggers/         # Symlinks to ../../triggers/*.yaml
├── bindings/         # Symlinks to ../../bindings/*.yaml
└── daemon-config.yaml # Complete configuration
```

## Prerequisites

### 1. Kubernetes Clusters

```bash
# Production kubeconfig
export KUBECONFIG_PROD=~/.kube/prod-config

# Staging kubeconfig
export KUBECONFIG_STAGING=~/.kube/staging-config

# Dev kubeconfig (optional, uses default)
export KUBECONFIG_DEV=~/.kube/config
```

### 2. Platform Credentials

```bash
# Slack
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_SIGNING_SECRET=your-secret

# Telegram
export TELEGRAM_BOT_TOKEN=your-telegram-token

# PagerDuty
export PAGERDUTY_WEBHOOK_TOKEN=your-pd-token

# LLM Provider
export GOOGLE_API_KEY=your-gemini-key
```

### 3. User Allowlists

Edit `contexts/prod.yaml` to add your user IDs:
```yaml
approval:
  allowed_users:
    - U012YOURUSER  # Add your Slack user ID
```

## Quick Start

### 1. Create Symlinks

```bash
cd enterprise

# Link to shared resources
ln -s ../../agents agents
ln -s ../../fleets fleets
ln -s ../../flows flows
ln -s ../../contexts contexts
ln -s ../../triggers triggers
ln -s ../../bindings bindings
```

### 2. Start the Daemon

```bash
# From enterprise/ directory
aofctl serve --config daemon-config.yaml
```

### 3. Expose Webhooks

```bash
# Production (use real domain)
# Configure DNS and SSL for: https://aof.yourcompany.com

# Development (use ngrok)
ngrok http 3000
# Configure webhooks to use ngrok URL
```

### 4. Configure Platform Webhooks

**Slack:**
- Go to your Slack App settings
- Event Subscriptions → Enable Events
- Request URL: `https://your-domain.com/webhook/slack`
- Subscribe to: `app_mention`, `message.im`

**Telegram:**
```bash
# Set webhook
curl -X POST https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook \
  -d "url=https://your-domain.com/webhook/telegram"
```

**PagerDuty:**
- Go to PagerDuty → Services → Your Service → Integrations
- Add Webhook Integration
- URL: `https://your-domain.com/webhook/pagerduty`
- Custom Headers: `X-PagerDuty-Signature: ${PAGERDUTY_WEBHOOK_TOKEN}`

## Usage Examples

### Production - Kubernetes Operations

```slack
# In #production channel
@k8s-bot get pods in api namespace
@k8s-bot why is api-deployment crashing?
@k8s-bot scale api-deployment to 5 replicas

# Approval prompt appears (because it's production)
✅ Click to approve
```

### Staging - Faster Iteration

```slack
# In #staging channel
@k8s-bot restart frontend deployment

# No approval needed (staging is more permissive)
✅ Restarted immediately
```

### Telegram - On-Call Incident Response

```telegram
# In on-call group
/incident investigate high latency in prod

# Auto-investigates with incident response team
🚨 Incident Response
Severity: P1
Status: Investigating...
```

### PagerDuty - Automatic Response

```
# PagerDuty triggers incident
→ Webhook received
→ incident-flow auto-starts investigation
→ Results posted to #prod-alerts
→ On-call engineers notified
```

## Multi-Tenant Routing

The same daemon handles all environments:

| Channel | Binding | Context | Behavior |
|---------|---------|---------|----------|
| #production | prod-slack-k8s | prod | Strict approvals, full audit |
| #staging | staging-slack-k8s | staging | Relaxed approvals, basic audit |
| #dev | dev-slack-k8s | dev | No approvals, no audit |

Each binding references the **same flow** (`k8s-ops-flow.yaml`) but different contexts.

## Approval Workflows

### Production
- **Required for**: delete, scale-down, rollback
- **Approvers**: Platform team leads, SRE team
- **Timeout**: 5 minutes
- **Audit**: Full logging to S3

### Staging
- **Required for**: delete namespace, delete PV
- **Approvers**: Platform team + Dev team
- **Timeout**: 3 minutes
- **Audit**: Command logging only

### Development
- **Required for**: Nothing
- **Approvers**: N/A
- **Timeout**: N/A
- **Audit**: Disabled

## Security Features

### Authentication
- Slack signing secret validation
- Telegram bot token verification
- PagerDuty webhook token validation

### Authorization
- Per-environment user allowlists
- Per-channel access controls
- Command approval workflows

### Audit Trail
- All commands logged
- User attribution
- Timestamp tracking
- Outcome recording
- S3 archival (production)

## Monitoring

### Metrics

The daemon exposes Prometheus metrics at `/metrics`:

```prometheus
# Request count by environment
aof_requests_total{environment="production"} 1234

# Approval rate
aof_approvals_total{environment="production",action="approved"} 45

# Agent execution time
aof_agent_duration_seconds{agent="k8s-ops",quantile="0.99"} 2.5
```

### Health Checks

```bash
# Liveness
curl http://localhost:3000/health

# Readiness
curl http://localhost:3000/ready
```

### Logs

Structured JSON logs:

```json
{
  "level": "info",
  "msg": "Command executed",
  "environment": "production",
  "user": "U012PLATFORM1",
  "command": "kubectl get pods",
  "approved": true,
  "duration_ms": 145
}
```

## Customization

### Add New Environment

1. Create context: `contexts/qa.yaml`
2. Create trigger: `triggers/slack-qa.yaml`
3. Create binding: `bindings/qa-slack-k8s.yaml`
4. Add to `daemon-config.yaml`

### Add New Platform

1. Create trigger: `triggers/discord-prod.yaml`
2. Create binding: `bindings/prod-discord-k8s.yaml`
3. Add to `daemon-config.yaml`

### Add New Capability

1. Create flow: `flows/database-ops-flow.yaml`
2. Create bindings for each environment
3. Add to `daemon-config.yaml`

## Deployment

### Docker

```dockerfile
FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y kubectl curl
COPY --from=builder /app/target/release/aofctl /usr/local/bin/
COPY examples/complete/enterprise /config
CMD ["aofctl", "serve", "--config", "/config/daemon-config.yaml"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-daemon
spec:
  replicas: 2  # High availability
  template:
    spec:
      containers:
      - name: aof
        image: yourregistry/aof:latest
        ports:
        - containerPort: 3000
        env:
        - name: SLACK_BOT_TOKEN
          valueFrom:
            secretKeyRef:
              name: aof-secrets
              key: slack-token
        # ... other env vars from secrets
```

### Helm

```bash
helm install aof-daemon ./helm/aof \
  --set image.tag=latest \
  --set-file secrets.slackToken=$SLACK_BOT_TOKEN \
  --set contexts.prod.kubeconfig=/kubeconfigs/prod
```

## Troubleshooting

### Bot not responding

```bash
# Check logs
aofctl logs --follow

# Check webhook configuration
curl -X POST https://your-domain.com/webhook/slack \
  -H "Content-Type: application/json" \
  -d '{"type":"url_verification","challenge":"test"}'
```

### Approval not working

```bash
# Verify user ID in allowlist
aofctl config get contexts.prod.approval.allowed_users

# Check approval timeout
aofctl config get contexts.prod.approval.timeout_seconds
```

### Wrong environment accessed

```bash
# Check binding configuration
aofctl bindings list

# Verify channel mapping
aofctl config get triggers.slack-prod.channels
```

## Support

- Documentation: https://docs.aof.sh
- Examples: https://github.com/agenticdevops/aof/tree/main/examples
- Issues: https://github.com/agenticdevops/aof/issues
