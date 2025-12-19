# Trigger Resource Reference

Complete reference for standalone Trigger resource specifications. Triggers define message sources that can be shared across multiple flows via FlowBindings.

## Overview

A Trigger represents a decoupled message source (Slack, Telegram, HTTP, etc.) that can be reused across multiple flows. This enables:
- Reusing the same trigger configuration across flows
- Separating trigger concerns from flow logic
- Multi-tenant deployments with different routing

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value
  annotations:              # Optional: Additional metadata
    key: value

spec:
  type: TriggerType         # Required: Platform type
  config:                   # Required: Platform-specific config
    bot_token: string
    channels: [string]
    # ... platform-specific fields
  enabled: bool             # Optional: Enable/disable trigger
```

---

## Trigger Types

| Type | Description | Required Config |
|------|-------------|-----------------|
| `Slack` | Slack bot events | `bot_token` |
| `Telegram` | Telegram bot events | `bot_token` |
| `Discord` | Discord bot events | `bot_token` |
| `WhatsApp` | WhatsApp Business API | `bot_token`, `phone_number_id` |
| `HTTP` | Generic HTTP webhook | `path` (optional) |
| `Schedule` | Cron-based trigger | `cron` |
| `PagerDuty` | PagerDuty incidents | `api_key` or `routing_key` |
| `GitHub` | GitHub webhooks | `webhook_secret` (optional) |
| `Jira` | Jira webhooks | None required |
| `Manual` | CLI invocation | None required |

---

## Platform Configurations

### Slack Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod-channel
spec:
  type: Slack
  config:
    # Required
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}

    # Optional filters
    channels:
      - production
      - prod-alerts
    users:
      - U12345678  # Only respond to specific users
    events:
      - app_mention
      - message
    patterns:
      - "kubectl.*"  # Regex patterns
      - "k8s"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Bot token (xoxb-...) or env var reference |
| `signing_secret` | string | No | Signing secret for verification |
| `channels` | array | No | Channel names or IDs to listen on |
| `users` | array | No | User IDs to respond to |
| `events` | array | No | Event types (app_mention, message) |
| `patterns` | array | No | Message patterns (regex) |

### Telegram Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-oncall
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}

    # Optional filters
    chat_ids:
      - -1001234567890  # Group chat ID
    users:
      - "123456789"     # User IDs as strings
    patterns:
      - "/kubectl"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Bot token from @BotFather |
| `chat_ids` | array | No | Chat/group IDs to listen on |
| `users` | array | No | User IDs to respond to |
| `patterns` | array | No | Message patterns (regex) |

### Discord Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-ops-bot
spec:
  type: Discord
  config:
    bot_token: ${DISCORD_BOT_TOKEN}
    app_secret: ${DISCORD_APP_SECRET}

    guild_ids:
      - "123456789012345678"
    channels:
      - ops-channel
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Discord bot token |
| `app_secret` | string | No | Application secret |
| `guild_ids` | array | No | Server IDs to listen on |
| `channels` | array | No | Channel names/IDs |

### HTTP Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-webhook
spec:
  type: HTTP
  config:
    path: /webhook/github
    methods:
      - POST
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    required_headers:
      X-GitHub-Event: "*"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | No | URL path pattern |
| `methods` | array | No | HTTP methods (GET, POST, etc.) |
| `webhook_secret` | string | No | Secret for signature verification |
| `required_headers` | map | No | Headers required for authentication |
| `port` | int | No | Port to listen on |
| `host` | string | No | Host to bind to |

### Schedule Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: daily-report
spec:
  type: Schedule
  config:
    cron: "0 9 * * *"      # 9 AM daily
    timezone: "America/New_York"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cron` | string | Yes | Cron expression |
| `timezone` | string | No | Timezone (default: UTC) |

### PagerDuty Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  type: PagerDuty
  config:
    api_key: ${PAGERDUTY_API_KEY}
    routing_key: ${PAGERDUTY_ROUTING_KEY}
    service_ids:
      - P123ABC
      - P456DEF
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string | Yes* | PagerDuty API key |
| `routing_key` | string | Yes* | PagerDuty routing key |
| `service_ids` | array | No | Service IDs to monitor |

*One of `api_key` or `routing_key` required.

### GitHub Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-pr-events
spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    github_events:
      - pull_request
      - push
      - issues
    repositories:
      - myorg/myrepo
      - myorg/other-repo
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `webhook_secret` | string | No | Webhook secret for verification |
| `github_events` | array | No | Event types to listen for |
| `repositories` | array | No | Repository filter (owner/repo) |

### WhatsApp Trigger

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: whatsapp-oncall
spec:
  type: WhatsApp
  config:
    bot_token: ${WHATSAPP_ACCESS_TOKEN}
    phone_number_id: ${WHATSAPP_PHONE_NUMBER_ID}
    verify_token: ${WHATSAPP_VERIFY_TOKEN}
    business_account_id: ${WHATSAPP_BUSINESS_ID}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | WhatsApp access token |
| `phone_number_id` | string | No | Phone number ID |
| `verify_token` | string | No | Webhook verification token |
| `business_account_id` | string | No | Business account ID |

---

## Environment Variables

Triggers support `${VAR_NAME}` syntax for environment variable expansion:

```yaml
spec:
  config:
    bot_token: ${SLACK_BOT_TOKEN}  # Expanded at runtime
```

**Security:** Never hardcode tokens in YAML files. Always use environment variable references.

---

## Matching & Filtering

Triggers support multiple levels of filtering:

### Channel Filter
```yaml
config:
  channels:
    - production      # Exact match
    - "#prod-alerts"  # Slack channel name format
```

### User Filter
```yaml
config:
  users:
    - U12345678       # Specific user IDs
    - "*"             # Any user (default)
```

### Pattern Filter (Regex)
```yaml
config:
  patterns:
    - "kubectl.*"     # Commands starting with kubectl
    - "^/deploy"      # Commands starting with /deploy
    - "(?i)help"      # Case-insensitive "help"
```

### Match Scoring

When multiple triggers match, the most specific wins:

| Filter | Score |
|--------|-------|
| Channel match | +100 |
| User match | +80 |
| Pattern match | +60 |
| Platform only | +10 |

---

## Complete Examples

### Production Slack Bot

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
  labels:
    environment: production
    team: platform
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
    channels:
      - production
      - prod-alerts
      - C0123456789   # Channel ID
    events:
      - app_mention
      - message
  enabled: true
```

### Multi-Pattern Telegram Bot

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-k8s
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
    patterns:
      - "^/kubectl"
      - "^/k8s"
      - "^/pods"
      - "^/deploy"
    users:
      - "123456789"   # On-call engineer
      - "987654321"   # SRE lead
```

### Scheduled Health Check

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: hourly-health-check
spec:
  type: Schedule
  config:
    cron: "0 * * * *"        # Every hour
    timezone: "UTC"
```

---

## Validation

Triggers are validated when loaded:

```bash
# Validate trigger YAML
aofctl validate -f trigger.yaml

# List loaded triggers
aofctl get triggers
```

### Validation Rules
- Name is required and must be DNS-compatible
- Type must be a valid trigger type
- Platform-specific required fields must be present
- Environment variable references are validated at runtime

---

## Usage with FlowBinding

Triggers are referenced in FlowBindings:

```yaml
# triggers/slack-prod.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}

---
# bindings/prod-k8s.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-k8s-binding
spec:
  trigger: slack-prod        # Reference to Trigger
  context: prod
  flow: k8s-ops-flow
```

---

## See Also

- [FlowBinding Reference](./flowbinding-spec.md) - Compose triggers with flows
- [Context Reference](./context-spec.md) - Execution environment configuration
- [DaemonConfig Reference](./daemon-config.md) - Server configuration
- [Resource Selection Guide](../concepts/resource-selection.md) - When to use what
