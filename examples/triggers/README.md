# Event Triggers

Triggers define **message sources**: Slack channels, Telegram groups, webhooks. They are platform-specific but workflow-agnostic.

## Available Triggers

- **`slack-prod.yaml`** - Production Slack channels
  - Channels: #production, #prod-alerts, #sre-oncall
  - Events: app_mention, message
  - Users: Platform team, SRE team

- **`slack-staging.yaml`** - Staging Slack channels
  - Channels: #staging, #staging-alerts, #qa-testing
  - Events: app_mention, message
  - Users: Platform + Dev + QA teams

- **`telegram-oncall.yaml`** - On-call Telegram group
  - Chats: SRE on-call group
  - Events: message, command
  - Commands: /incident, /status, /ack, /resolve

- **`pagerduty.yaml`** - PagerDuty webhook
  - Events: incident.triggered, .acknowledged, .resolved
  - Services: Production platform, database

## Trigger Types

### Slack
\`\`\`yaml
type: Slack
config:
  events: [app_mention, message]
  channels: [production]
  bot_token: ${SLACK_BOT_TOKEN}
  signing_secret: ${SLACK_SIGNING_SECRET}
\`\`\`

### Telegram
\`\`\`yaml
type: Telegram
config:
  events: [message, command]
  chats: [-1001234567890]
  bot_token: ${TELEGRAM_BOT_TOKEN}
  commands: [/incident, /status]
\`\`\`

### Webhook (Generic)
\`\`\`yaml
type: Webhook
config:
  path: /webhook/pagerduty
  auth_type: token
  auth_token: ${WEBHOOK_TOKEN}
  event_types: [incident.triggered]
\`\`\`

## Usage

Triggers are referenced in **bindings**:

\`\`\`yaml
spec:
  trigger: triggers/slack-prod.yaml
  flow: flows/k8s-ops-flow.yaml
  context: contexts/prod.yaml
\`\`\`

## Multi-Platform Pattern

Same flow, different platforms:

\`\`\`yaml
# Slack users
slack-prod:
  trigger: slack-prod
  flow: k8s-ops-flow

# On-call via Telegram
telegram-oncall:
  trigger: telegram-oncall
  flow: incident-flow

# Automated via PagerDuty
pagerduty-auto:
  trigger: pagerduty
  flow: incident-flow
\`\`\`
