# Telegram Integration

AOF integrates with Telegram through the Bot API using **long polling** by default. No webhook URLs, ngrok, or public server required - just a bot token from @BotFather.

## Why Telegram?

- **900M+ monthly active users** - One of the most popular messaging platforms globally
- **Bot API** - First-class bot support with rich features
- **Inline keyboards** - Interactive button responses
- **Reply threads** - Organized conversation context
- **Groups & channels** - Team-based agent interactions
- **Free** - No per-message costs

## How It Works

AOF uses **long polling** (getUpdates API) instead of webhooks:

```
AOF Server                         Telegram API
    |                                   |
    |--- POST getUpdates(timeout=30) -->|
    |         (holds connection)        |
    |                                   |
    |<-- updates (when available) ------|
    |                                   |
    |--- parse & route to agent ------->|
    |                                   |
    |--- POST sendMessage ------------->|
    |                                   |
    |--- POST getUpdates(offset=N+1) -->|
    |         (next poll cycle)         |
```

### Why Long Polling?

| | Long Polling | Webhooks |
|---|---|---|
| **Setup** | Just a bot token | Public URL + SSL + DNS |
| **Local dev** | Works immediately | Requires ngrok or similar |
| **Latency** | ~instant (connection held open) | ~instant (push) |
| **Reliability** | Built-in offset tracking | Must handle retries |
| **Firewall** | Outbound only | Must allow inbound |

Long polling is the default because it removes all infrastructure requirements. For production deployments with high message volume, webhooks can be enabled as an optimization.

## Quick Start

### 1. Create a Bot

1. Open Telegram and message [@BotFather](https://t.me/BotFather)
2. Send `/newbot` and follow the prompts
3. Copy the bot token (looks like `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`)

### 2. Configure AOF

Set the bot token as an environment variable:

```bash
export TELEGRAM_BOT_TOKEN="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
```

Add Telegram to your `serve-config.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: ServerConfig
spec:
  platforms:
    telegram:
      enabled: true
      bot_token_env: "TELEGRAM_BOT_TOKEN"
      polling: true  # default - no webhook needed
```

### 3. Start the Server

```bash
aofctl serve --config serve-config.yaml
```

Output:
```
  Registered platform: telegram
  Telegram polling started (getUpdates mode - no webhook required)
  Server listening on 127.0.0.1:7777
```

### 4. Chat with Your Bot

Open Telegram, find your bot, and send a message. AOF routes it to the configured agent and sends back the response.

## Configuration Reference

```yaml
platforms:
  telegram:
    # Enable/disable Telegram integration
    enabled: true

    # Bot token (direct value or env var reference)
    bot_token: "123456:ABC-..."        # Direct value
    bot_token_env: "TELEGRAM_BOT_TOKEN" # Or from environment

    # Use long polling (default: true)
    # Set to false if using webhooks with a public URL
    polling: true

    # Optional: restrict to a single user ID
    # Get your ID from @userinfobot on Telegram
    allowed_user_id: 123456789

    # Optional: webhook secret (only for webhook mode)
    webhook_secret: "your-secret"
```

### Resource Spec

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Telegram platform |
| `bot_token` | string | - | Bot token (direct) |
| `bot_token_env` | string | - | Env var containing bot token |
| `polling` | bool | `true` | Use getUpdates polling |
| `allowed_user_id` | int | - | Restrict to single user |
| `webhook_secret` | string | - | Secret for webhook verification |

## Security

### User Whitelisting

Restrict bot access to specific users:

```yaml
platforms:
  telegram:
    enabled: true
    bot_token_env: "TELEGRAM_BOT_TOKEN"
    allowed_user_id: 123456789
```

To find your user ID:
1. Message [@userinfobot](https://t.me/userinfobot) on Telegram
2. It replies with your numeric ID (e.g., `123456789`)
3. Use this numeric ID, not your @username

### Bot Token Security

- Never commit bot tokens to version control
- Use `bot_token_env` to reference environment variables
- Rotate tokens if compromised via @BotFather `/revoke`

## Supported Features

| Feature | Status |
|---|---|
| Text messages | Supported |
| /commands | Supported |
| Inline keyboards (buttons) | Supported |
| Callback queries | Supported |
| Reply threads | Supported |
| File attachments | Supported |
| Group chats | Supported |
| MarkdownV2 formatting | Supported |
| Inline queries | Planned |

## Bot Commands

AOF registers these default commands:

| Command | Description |
|---|---|
| `/agent` | List or switch agents |
| `/agent <name>` | Switch to specific agent |
| `/agent info` | Show current agent details |
| `/run agent <name> <input>` | Run agent directly |
| `/status task <id>` | Check task status |
| `/help` | Show help text |

## Polling vs Webhook Mode

### Polling (Default)

Best for:
- Local development
- Single-instance deployments
- Getting started quickly
- Behind firewalls/NAT

### Webhook Mode

Best for:
- High-volume production deployments
- Multi-instance setups
- When you already have a public endpoint

To switch to webhook mode:

```yaml
platforms:
  telegram:
    enabled: true
    bot_token_env: "TELEGRAM_BOT_TOKEN"
    polling: false
    webhook_secret: "your-random-secret"
```

Then set the webhook URL via API:
```bash
curl -X POST "https://api.telegram.org/bot$TOKEN/setWebhook" \
  -d "url=https://your-domain.com/webhook/telegram" \
  -d "secret_token=your-random-secret"
```

## Troubleshooting

### Bot doesn't respond

1. Check the bot token is correct: `echo $TELEGRAM_BOT_TOKEN`
2. Verify the server is running: look for "Telegram polling started" in logs
3. Make sure you're messaging the right bot
4. Check server logs for errors

### 409 Conflict Error

Another instance is already polling with the same token. Only one process can use getUpdates at a time. Stop the other instance or switch to webhook mode.

### Messages are delayed

Long polling has a 30-second timeout. If you see delays longer than that, check network connectivity to `api.telegram.org`.

### Bot responds to everyone

Set `allowed_user_id` to restrict access to your user ID only.
