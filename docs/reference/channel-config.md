# Channel Configuration Reference

Full YAML reference for multi-channel gateway configuration in `agentix.yaml`.

## Workspace Config: `spec.channels`

The `channels` field is an array of channel platform configurations:

```yaml
spec:
  channels:
    - platform: <platform>
      credentials: <credentials>
      routes: <routes[]>
      enabled: <bool>
```

## Platform Types

| Value | Platform |
|-------|----------|
| `slack` | Slack (Events API + Web API) |
| `telegram` | Telegram (Bot API) |
| `discord` | Discord (Gateway + REST API) |

## Credentials

### Slack

```yaml
credentials:
  platform: slack
  bot_token: "${SLACK_BOT_TOKEN}"        # xoxb-... OAuth bot token
  signing_secret: "${SLACK_SIGNING_SECRET}" # For webhook verification
  app_id: "${SLACK_APP_ID}"              # Slack application ID
```

### Telegram

```yaml
credentials:
  platform: telegram
  bot_token: "${TELEGRAM_BOT_TOKEN}"     # From @BotFather
  webhook_secret: "${TELEGRAM_WEBHOOK_SECRET}" # Optional secret token
```

### Discord

```yaml
credentials:
  platform: discord
  bot_token: "${DISCORD_BOT_TOKEN}"      # Bot token from Discord Developer Portal
  application_id: "${DISCORD_APP_ID}"    # Application ID (for mention detection)
  public_key: "${DISCORD_PUBLIC_KEY}"    # Public key (for Ed25519 verification)
```

## Routes

Each route maps a channel to an agent:

```yaml
routes:
  - channel_id: "C_DEVOPS"           # Platform-specific channel identifier
    agent_name: "dba-optimizer"       # Must match a loaded agent name
    direction: bidirectional          # inbound | outbound | bidirectional
    description: "DevOps channel"     # Optional human-readable description
```

### Direction Values

| Value | Inbound Messages | Outbound Notifications |
|-------|------------------|----------------------|
| `inbound` | Yes | No |
| `outbound` | No | Yes |
| `bidirectional` (default) | Yes | Yes |

## Enabled Flag

```yaml
enabled: true  # Default: true. Set to false to skip this platform.
```

## Notification Severity Levels

| Level | Description |
|-------|-------------|
| `info` | Run completed, status update |
| `warning` | Budget threshold, degraded state |
| `error` | Run failed, tool error |
| `critical` | System down, security breach |

## Full Example

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-workspace
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
  gateway:
    host: "0.0.0.0"
    port: 8080
  agents_dir: "./agents"
  channels:
    - platform: slack
      credentials:
        platform: slack
        bot_token: "${SLACK_BOT_TOKEN}"
        signing_secret: "${SLACK_SIGNING_SECRET}"
        app_id: "${SLACK_APP_ID}"
      routes:
        - channel_id: "C_DEVOPS"
          agent_name: dba-optimizer
          direction: bidirectional
          description: "DevOps team channel"
        - channel_id: "C_ALERTS"
          agent_name: cost-monitor
          direction: outbound
          description: "Alerts channel"
      enabled: true
    - platform: telegram
      credentials:
        platform: telegram
        bot_token: "${TELEGRAM_BOT_TOKEN}"
        webhook_secret: "${TELEGRAM_WEBHOOK_SECRET}"
      routes:
        - channel_id: "-100123456"
          agent_name: dba-optimizer
          direction: bidirectional
      enabled: true
    - platform: discord
      credentials:
        platform: discord
        bot_token: "${DISCORD_BOT_TOKEN}"
        application_id: "${DISCORD_APP_ID}"
        public_key: "${DISCORD_PUBLIC_KEY}"
      routes:
        - channel_id: "123456789"
          agent_name: devops-assistant
          direction: bidirectional
      enabled: true
```

## REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/channels` | List all configured channel routes |
| POST | `/api/v1/notify` | Send an outbound notification |
| POST | `/webhooks/channels/:platform` | Receive inbound platform webhooks |

### POST /api/v1/notify Request Body

```json
{
  "platform": "slack",
  "channel_id": "C_ALERTS",
  "agent_name": "cli",
  "title": "Budget Alert",
  "body": "Monthly spend exceeded threshold",
  "severity": "warning"
}
```

## CLI Commands

```bash
# List configured channels
agentix channels

# Send a notification
agentix notify \
  --platform slack \
  --channel-id C_ALERTS \
  --title "Alert" \
  --body "CPU usage high" \
  --severity warning
```
