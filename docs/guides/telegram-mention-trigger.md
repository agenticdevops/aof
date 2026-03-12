# Telegram Mention Trigger

Sending a message to the Telegram bot fires the agent with the message text
as input.

## Overview

When a user sends a message to your Telegram bot (directly or in a group),
OpenAgentiX dispatches the message to the configured agent. Unlike Slack and
Discord which require an explicit @mention, any message to the Telegram bot
triggers the agent.

## Prerequisites

- A Telegram account
- A bot created via @BotFather
- The OpenAgentiX gateway accessible from the internet (Telegram requires HTTPS)

## Agent YAML Configuration

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: infra-helper
  description: Infrastructure helper bot on Telegram
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    You are an infrastructure helper accessible via Telegram. When a user
    sends you a message, you receive a TriggerEvent in your input. Read
    payload.text to understand their request. Provide brief, actionable
    responses suitable for a chat interface.

    Common tasks: checking service status, explaining errors, suggesting
    commands, and answering infrastructure questions.
  triggers:
    - type: telegram
      bot_token: ${TELEGRAM_BOT_TOKEN}
```

## Telegram Bot Setup

### 1. Create a Bot with @BotFather

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` and follow the prompts
3. Choose a name (display name) and username (must end in `bot`)
4. Copy the **HTTP API token** for `TELEGRAM_BOT_TOKEN`

### 2. Register Webhook with Telegram

Telegram requires you to register a webhook URL so it knows where to send
updates. Call the Telegram Bot API:

```bash
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -d "url=https://your-gateway/webhooks/infra-helper-telegram-0"
```

The trigger_id format is `{agent_name}-telegram-{index}`.

Verify the webhook is set:
```bash
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo"
```

### 3. Configure Privacy (for Group Chats)

By default, bots in groups only receive messages that start with `/` or
explicitly mention them. To receive all messages in groups:

1. Send `/setprivacy` to @BotFather
2. Select your bot
3. Choose **Disable** to receive all messages

Or, keep privacy mode enabled and users must mention the bot as
`@your_bot_username` in group messages.

## TriggerEvent Payload

When a message is received, the agent gets:

```json
{
  "source": "telegram",
  "payload": {
    "chat_id": 111222333,
    "user_id": 987654321,
    "username": "devuser",
    "text": "@infra_helper_bot check replication lag on db-primary"
  },
  "context": {
    "platform": "telegram",
    "chat_id": "111222333"
  },
  "fired_at": "2026-03-13T11:00:00Z",
  "trigger_id": "infra-helper-telegram-0"
}
```

The `payload.chat_id` can be used to send replies back via the Telegram API.

## HTTPS Requirement

Telegram only sends webhooks to HTTPS endpoints. For local development:

- Use [ngrok](https://ngrok.com/) to expose a local port: `ngrok http 7777`
- Register the ngrok HTTPS URL as the webhook URL

For production, use a reverse proxy (nginx, Caddy, or similar) with TLS.

## Environment Variables

```bash
export TELEGRAM_BOT_TOKEN="123456789:ABCdefGHIjklMNOPqrstUVWXyz"
```

## See Also

- [Triggers Overview](../concepts/triggers.md)
- [Slack Mention Trigger](slack-mention-trigger.md)
- [Discord Mention Trigger](discord-mention-trigger.md)
