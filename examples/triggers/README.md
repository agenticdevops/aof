# Triggers

Triggers define **message sources** and **command routing**: which platforms to listen on, what commands to recognize, and where to route them.

## Architecture

Each Trigger is a self-contained unit that includes:
- **Platform Configuration** - Slack, Telegram, Discord, webhook settings
- **Command Bindings** - Maps slash commands to agents, fleets, or flows
- **Default Agent** - Fallback for natural language messages

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}

  # Route commands to specific handlers
  commands:
    /aof:
      agent: devops
      description: "General DevOps assistant"
    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis"
    /deploy:
      flow: deploy-flow
      description: "Deployment workflow"

  # Fallback for @mentions and natural language
  default_agent: devops
```

## Available Examples

### `slack-prod.yaml` - Production Slack
Routes commands in production channels to appropriate handlers.
- Commands: `/aof`, `/diagnose`, `/deploy`, `/rca`
- Fleets: `rca-fleet` for multi-model consensus analysis
- Flows: `deploy-flow` for approval workflows

### `slack-staging.yaml` - Staging Slack
Simplified staging configuration for development/QA.
- Commands: `/aof`, `/test`
- Default agent for all natural language

### `telegram-oncall.yaml` - On-Call Telegram
Mobile-friendly incident management.
- Commands: `/incident`, `/status`, `/ack`, `/resolve`
- Fleets: `incident-fleet` for on-call workflows

### `pagerduty.yaml` - PagerDuty Webhook
Automated incident response from PagerDuty events.
- Event types: `incident.triggered`, `incident.acknowledged`
- Routes to flows and agents based on event type

## Trigger Types

### Slack
```yaml
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
  commands:
    /kubectl:
      agent: k8s-admin
      description: "Kubernetes operations"
  default_agent: devops
```

### Telegram
```yaml
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
  commands:
    /start:
      agent: devops
      description: "Get started"
    /status:
      agent: monitoring
      description: "Check system status"
  default_agent: devops
```

### PagerDuty (Webhook)
```yaml
spec:
  type: PagerDuty
  config:
    path: /webhook/pagerduty
    webhook_secret: ${PAGERDUTY_WEBHOOK_TOKEN}
    service_ids:
      - PXXXXXX
  commands:
    incident.triggered:
      flow: incident-flow
      description: "Auto-triggered incident response"
  default_agent: devops
```

## Command Binding Options

Each command can route to one of three targets:

```yaml
commands:
  # Route to a single agent
  /ask:
    agent: devops
    description: "Ask the DevOps agent"

  # Route to a fleet (multi-agent coordination)
  /diagnose:
    fleet: rca-fleet
    description: "Multi-model RCA with consensus"

  # Route to a flow (multi-step workflow)
  /deploy:
    flow: deploy-flow
    description: "Deployment with approval gates"
```

## Usage with DaemonConfig

The daemon loads triggers from a configured directory:

```yaml
# daemon.yaml
triggers:
  directory: ./examples/triggers/
  watch: false  # Enable for hot-reload
```

When a message arrives:
1. Platform is matched (Slack, Telegram, etc.)
2. Command is extracted (e.g., `/diagnose`)
3. Command binding routes to agent/fleet/flow
4. If no command match, `default_agent` handles it
