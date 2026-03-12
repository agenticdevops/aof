# Discord Mention Trigger

Mentioning the agent bot in any Discord channel fires the agent with the
message content as input.

## Overview

When a Discord user sends a message containing `<@BOT_USER_ID>` in a channel
where your bot is present, OpenAgentiX dispatches a `MESSAGE_CREATE` event to
the configured agent. The agent runs autonomously with the message content in
its TriggerEvent.

## Prerequisites

- A Discord server where you have Manage Server permissions
- A Discord application with a bot user
- The OpenAgentiX gateway accessible from the internet

## Agent YAML Configuration

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: ops-assistant
  description: Operational assistant available in Discord
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    You are an operations assistant in a Discord server. When mentioned
    in Discord, you receive a TriggerEvent in your input. Read payload.content
    to understand what the user is asking. Respond concisely since responses
    will be sent back to a chat channel.
  triggers:
    - type: discord
      bot_token: ${DISCORD_BOT_TOKEN}
      application_id: ${DISCORD_APPLICATION_ID}
```

## Discord Application Setup

### 1. Create a Discord Application

1. Go to https://discord.com/developers/applications
2. Click **New Application** and give it a name
3. Go to **Bot** section and click **Add Bot**
4. Copy the bot token for `DISCORD_BOT_TOKEN`

### 2. Configure Interactions Endpoint

Discord uses an interactions endpoint (not a webhook subscription like Slack).
To receive `MESSAGE_CREATE` events, you need either:

**Option A: Gateway via Bot WebSocket** (recommended for self-hosted bots)
The bot token is used to connect to the Discord Gateway via WebSocket.
Configure your gateway's outbound bot connector to forward events to:
```
POST /webhooks/ops-assistant-discord-0
```

**Option B: Discord Interactions Endpoint**
For slash commands, set the **Interactions Endpoint URL** in your application
settings. However, `MESSAGE_CREATE` events require a bot WebSocket connection.

### 3. Enable Message Content Intent

1. In your application's **Bot** settings, enable:
   - **Server Members Intent** (if needed)
   - **Message Content Intent** (required to read message content)
2. In production bots (over 100 servers), these intents require verification

### 4. Invite Bot to Server

1. Go to **OAuth2 → URL Generator**
2. Select scopes: `bot`
3. Select bot permissions: `Read Messages/View Channels`, `Send Messages`
4. Use the generated URL to invite the bot to your server

### 5. Get Application ID

In **General Information**, copy the **Application ID** for `DISCORD_APPLICATION_ID`.

## TriggerEvent Payload

When the agent is mentioned (message contains `<@BOT_USER_ID>`), it receives:

```json
{
  "source": "discord",
  "payload": {
    "channel_id": "111222333444555666",
    "guild_id": "777888999000111222",
    "user_id": "333444555666777888",
    "username": "devuser",
    "content": "<@999000111222333444> check the deployment status"
  },
  "context": {
    "platform": "discord",
    "channel_id": "111222333444555666"
  },
  "fired_at": "2026-03-13T10:30:00Z",
  "trigger_id": "ops-assistant-discord-0"
}
```

The `payload.content` field contains the raw message text including the `<@...>` mention. The agent should strip this before processing if needed.

## Environment Variables

```bash
export DISCORD_BOT_TOKEN="your-bot-token-here"
export DISCORD_APPLICATION_ID="your-application-id"
```

## See Also

- [Triggers Overview](../concepts/triggers.md)
- [Slack Mention Trigger](slack-mention-trigger.md)
- [Telegram Mention Trigger](telegram-mention-trigger.md)
