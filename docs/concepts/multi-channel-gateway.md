# Multi-Channel Gateway

OpenAgentiX supports bi-directional multi-channel communication. Agents receive messages from Slack, Telegram, and Discord channels and respond in the same channel. Agents can also send outbound notifications — run results, alerts, approval requests — to configured channels without a user-initiated trigger.

## Supported Platforms

| Platform | Inbound | Outbound | Threads | Rich Formatting | Auth Method |
|----------|---------|----------|---------|-----------------|-------------|
| Slack | Events API webhooks | chat.postMessage | Thread replies via `thread_ts` | Block Kit blocks | Bot token + signing secret |
| Telegram | Bot API webhooks | sendMessage | Reply via `reply_to_message_id` | MarkdownV2 | Bot token + webhook secret (optional) |
| Discord | Gateway events / interactions | REST API messages | Message references | Embeds with color | Bot token + Ed25519 public key |

## Channel Configuration

Channels are configured in the `spec.channels` section of your `agentix.yaml` workspace config:

```yaml
spec:
  channels:
    - platform: slack
      credentials:
        platform: slack
        bot_token: "${SLACK_BOT_TOKEN}"
        signing_secret: "${SLACK_SIGNING_SECRET}"
        app_id: "A12345"
      routes:
        - channel_id: "C_DEVOPS"
          agent_name: dba-optimizer
          direction: bidirectional
        - channel_id: "C_ALERTS"
          agent_name: cost-anomaly-detector
          direction: outbound
      enabled: true

    - platform: telegram
      credentials:
        platform: telegram
        bot_token: "${TELEGRAM_BOT_TOKEN}"
        webhook_secret: "${TELEGRAM_WEBHOOK_SECRET}"
      routes:
        - channel_id: "${TELEGRAM_CHAT_ID}"
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
        - channel_id: "${DISCORD_CHANNEL_ID}"
          agent_name: devops-assistant
          direction: bidirectional
      enabled: true
```

## Routing

Each channel config contains one or more **routes** that map a platform channel to an agent:

- **`inbound`** — Messages in this channel trigger the specified agent. The agent cannot send notifications back to this channel.
- **`outbound`** — The agent can send notifications to this channel but will not receive messages from it.
- **`bidirectional`** (default) — Both inbound triggers and outbound notifications. This is the most common mode.

Multiple routes per channel config allow different channels to route to different agents:

```yaml
routes:
  - channel_id: "C_DEVOPS"
    agent_name: dba-optimizer
    direction: bidirectional
    description: "DevOps team channel"
  - channel_id: "C_SECURITY"
    agent_name: security-scanner
    direction: inbound
    description: "Security reports channel"
  - channel_id: "C_ALERTS"
    agent_name: cost-anomaly-detector
    direction: outbound
    description: "Alerts channel — outbound only"
```

The `agent_name` must match a loaded agent in the gateway.

## Notification System

Agents can send outbound notifications to configured channels. Notifications carry structured payloads:

```json
{
  "agent_name": "cost-monitor",
  "title": "Budget Alert",
  "body": "Monthly spend exceeded $500 threshold",
  "severity": "warning",
  "run_id": "run-abc-123",
  "metadata": { "env": "production", "amount": 523.45 }
}
```

### Severity Levels

| Level | When to Use | Slack Color | Discord Color | Telegram Emoji |
|-------|------------|-------------|---------------|----------------|
| `info` | Run completed, status update | #36a64f (green) | 0x36a64f | info |
| `warning` | Budget threshold, degraded state | #ffc107 (amber) | 0xffc107 | warning |
| `error` | Run failed, tool error | #dc3545 (red) | 0xdc3545 | error |
| `critical` | System down, security breach | #dc3545 (red, bold) | 0xff0000 | critical |

### When Notifications Fire

- Run completion (success or failure)
- Budget threshold exceeded
- Approval needed
- Agent errors or tool failures
- Custom notifications via REST API or CLI

## Message Flow

### Inbound Flow

```
Platform Webhook → Gateway HTTP endpoint
  → ChannelGateway.parse_webhook()
  → ChannelGatewayManager.route_inbound(platform, channel_id)
  → Agent identified by name
  → ReAct loop executes with message as input
  → Response text
  → ChannelGateway.send_message(channel_id, response, thread_id)
  → Platform API delivers response
```

### Outbound Flow

```
Agent run produces result/alert
  → NotificationPayload constructed
  → ChannelGatewayManager.send_notification(target, payload)
  → ChannelGateway for target platform
  → Platform-specific formatting (Block Kit / Markdown / Embeds)
  → Platform API delivers notification
```

## Security

### Webhook Signature Verification

Each platform uses a different signature scheme to verify that incoming webhooks are authentic:

- **Slack**: HMAC-SHA256 of `v0:{timestamp}:{body}` using `signing_secret`. The signature is in the `X-Slack-Signature` header.
- **Telegram**: Secret token in `X-Telegram-Bot-Api-Secret-Token` header, matched against configured `webhook_secret`.
- **Discord**: Ed25519 signature of `{timestamp}{body}` using `public_key`. The signature is in the `X-Signature-Ed25519` header.

### Credential Storage

All credentials should be stored as environment variables, not hardcoded in config files. Use `${VAR_NAME}` syntax in `agentix.yaml` for environment variable substitution.

## CLI

### List Channels

```bash
agentix channels
```

Displays all configured channel routes in a table:

```
Platform   Channel ID    Agent              Direction      Description
slack      C_DEVOPS      dba-optimizer      bidirectional  DevOps team channel
slack      C_ALERTS      cost-monitor       outbound       Alerts channel
telegram   -100123456    dba-optimizer      bidirectional  On-call group
```

### Send Notification

```bash
agentix notify \
  --platform slack \
  --channel-id C_ALERTS \
  --title "Budget Alert" \
  --body "Monthly spend exceeded threshold" \
  --severity warning
```

## REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/channels` | List all configured channel routes |
| POST | `/api/v1/notify` | Send an outbound notification |
| POST | `/webhooks/channels/:platform` | Receive inbound webhooks from platforms |

### Webhook Setup

- **Slack**: Set your Events API Request URL to `https://your-gateway/webhooks/channels/slack`
- **Telegram**: Call `setWebhook` with URL `https://your-gateway/webhooks/channels/telegram`
- **Discord**: Set your Interactions Endpoint URL to `https://your-gateway/webhooks/channels/discord`
