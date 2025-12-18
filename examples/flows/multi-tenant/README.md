# Multi-Tenant AgentFlow Examples

This directory contains example AgentFlows demonstrating multi-tenant bot architecture. Each flow connects a specific platform/channel combination to a specific Kubernetes cluster with appropriate permissions.

## Architecture Overview

```
                    ┌─────────────────────────────────────────────────────┐
                    │                  AOF Daemon                         │
                    │                                                     │
 Slack #production ─┼─► FlowRouter ─► slack-prod-k8s-bot     ─► Prod K8s │
 Slack #staging    ─┼─► FlowRouter ─► slack-staging-k8s-bot  ─► Staging  │
 Slack #dev-local  ─┼─► FlowRouter ─► slack-dev-local-bot    ─► Local    │
 WhatsApp          ─┼─► FlowRouter ─► whatsapp-oncall-bot    ─► Prod K8s │
                    │                                                     │
                    └─────────────────────────────────────────────────────┘
```

## Available Flows

| Flow | Platform | Channels | Cluster | Permissions |
|------|----------|----------|---------|-------------|
| `slack-prod-k8s-bot` | Slack | #production, #prod-alerts | prod-cluster | Restricted |
| `slack-staging-k8s-bot` | Slack | #staging, #dev-test | staging-cluster | Permissive |
| `slack-dev-local-bot` | Slack | #dev-local, #sandbox | local | Full |
| `whatsapp-oncall-bot` | WhatsApp | On-call phones | prod-cluster | Read-only |

## Quick Start

### 1. Set Environment Variables

```bash
# Slack credentials
export SLACK_BOT_TOKEN=xoxb-xxxxx
export SLACK_SIGNING_SECRET=xxxxx

# LLM API key
export ANTHROPIC_API_KEY=sk-ant-xxxxx

# Kubernetes configs
export KUBECONFIG_PROD=~/.kube/prod-config
export KUBECONFIG_STAGING=~/.kube/staging-config

# WhatsApp (optional)
export WHATSAPP_TOKEN=xxxxx
export WHATSAPP_VERIFY_TOKEN=xxxxx
```

### 2. Start the Daemon with Flows Directory

```bash
# Load all flows from the multi-tenant directory
aofctl serve \
  --flows-dir examples/flows/multi-tenant \
  --agents-dir examples/agents \
  --port 3000
```

### 3. Expose with ngrok (Development)

```bash
ngrok http 3000
```

### 4. Configure Platform Webhooks

**Slack Event Subscriptions:**
- URL: `https://xxxx.ngrok.io/webhook/slack`
- Events: `app_mention`, `message.channels`, `message.im`

**WhatsApp Business API:**
- Webhook URL: `https://xxxx.ngrok.io/webhook/whatsapp`
- Verify token: `${WHATSAPP_VERIFY_TOKEN}`

## Flow Routing

The FlowRouter matches incoming messages to flows based on:

1. **Platform** - Slack, WhatsApp, Discord, etc.
2. **Channels** - Specific channel names or IDs
3. **Users** - Restrict to specific user IDs (optional)
4. **Patterns** - Regex patterns for message content (optional)

### Routing Priority

When multiple flows could match, the router uses this priority:

1. Most specific channel match
2. User restrictions (more restricted = higher priority)
3. Pattern specificity
4. Flow registration order

### Example Routing

```
Message: "@k8s-bot get pods" in #production from U012PLATFORM1
  → Matches: slack-prod-k8s-bot (channel match + user match)

Message: "@k8s-bot delete pod nginx" in #staging from U999NEWDEV
  → Matches: slack-staging-k8s-bot (channel match, no user restriction)

Message: "status" via WhatsApp from +1234567890
  → Matches: whatsapp-oncall-bot (platform + user match)

Message: "@k8s-bot help" in #random from U999USER
  → No match: falls back to default agent or error
```

## Context Configuration

Each flow specifies a `context` that defines the execution environment:

```yaml
context:
  # Kubernetes cluster connection
  kubeconfig: ${KUBECONFIG_PROD:-~/.kube/prod-config}
  namespace: default
  cluster: prod-cluster

  # Environment variables for agent execution
  env:
    ENVIRONMENT: production
    KUBECTL_READONLY: "true"
    REQUIRE_APPROVAL: "true"

  # Working directory
  working_dir: /workspace
```

## Security Considerations

### Production Flow
- Restricted to specific channels and users
- May require approval for mutations
- Full audit logging enabled

### Staging Flow
- Open to development team
- More permissive operations allowed
- Good for testing changes before prod

### Local Dev Flow
- Full permissions
- No approval required
- Fast iteration for experiments

### WhatsApp Flow
- Read-only operations only
- Restricted to on-call phone numbers
- Optimized for mobile output

## Creating Custom Flows

1. Copy an existing flow as a template
2. Update metadata (name, labels, annotations)
3. Configure trigger (platform, channels, patterns)
4. Set context (kubeconfig, namespace, env vars)
5. Adjust permissions as needed
6. Deploy with `aofctl apply`

See individual flow files for detailed examples.
