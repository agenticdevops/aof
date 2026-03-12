# Slack Mention Trigger

Mentioning `@agent-name` in any Slack channel fires the agent automatically
with the message content as input.

## Overview

When your Slack app receives an `app_mention` event, OpenAgentiX dispatches
it to the configured agent via its registered webhook endpoint. The agent runs
autonomously and receives the full message content in its TriggerEvent.

## Prerequisites

- A Slack workspace where you have permission to create apps
- The OpenAgentiX gateway accessible from the internet (or Slack's network)
- Slack app with **Events API** enabled and `app_mention` event subscribed

## Agent YAML Configuration

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-optimizer
  description: Optimize database queries on demand via Slack mention
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    You are a database optimization assistant. When mentioned in Slack,
    you receive a TriggerEvent in your input. Read payload.text to
    understand what the user is asking, then provide actionable advice
    for database optimization, query analysis, or performance issues.
  triggers:
    - type: slack
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
```

## Slack App Setup

### 1. Create a Slack App

1. Go to https://api.slack.com/apps and click **Create New App**
2. Choose **From scratch**, give it a name matching your agent
3. Select your workspace

### 2. Enable Events API

1. In your app settings, go to **Event Subscriptions**
2. Toggle **Enable Events** to ON
3. Set **Request URL** to your gateway endpoint:
   ```
   https://your-gateway/webhooks/dba-optimizer-slack-0
   ```
   The trigger_id format is `{agent_name}-slack-{index}`.
4. Wait for Slack to verify the URL (Slack sends a `url_verification` challenge;
   the gateway returns 202 which Slack accepts)

### 3. Subscribe to Events

Under **Subscribe to bot events**, add:
- `app_mention` — fires when `@agent-name` is mentioned in any channel

### 4. Install the App

1. Go to **OAuth & Permissions**
2. Add the `app_mentions:read` scope under **Bot Token Scopes**
3. Install the app to your workspace
4. Copy the **Bot User OAuth Token** (starts with `xoxb-`) for `SLACK_BOT_TOKEN`

### 5. Get Signing Secret

In **Basic Information**, scroll to **App Credentials** and copy the
**Signing Secret** for `SLACK_SIGNING_SECRET`.

### 6. Invite the Bot to Channels

Run `/invite @your-bot-name` in each channel where you want the trigger active.

## TriggerEvent Payload

When the agent is mentioned, it receives:

```json
{
  "source": "slack",
  "payload": {
    "channel": "C456DEF",
    "user": "U123ABC",
    "text": "<@U999BOT> analyze slow queries on the payments table",
    "ts": "1741234567.000001"
  },
  "context": {
    "platform": "slack",
    "channel": "C456DEF"
  },
  "fired_at": "2026-03-13T09:00:00Z",
  "trigger_id": "dba-optimizer-slack-0"
}
```

The agent's system prompt should instruct it to read `payload.text` and
strip the mention prefix (`<@BOT_ID>`) before processing.

## Responding Back to Slack

In v2.0, the triggered agent's output is logged but not automatically sent
back to Slack. Response delivery to Slack channels is planned for Phase 21
(Gateway). In the meantime:

- Configure a `notifications: webhook` pointing to a Slack incoming webhook
  to post the result to a channel
- Or have the agent call the Slack API directly using a tool

## Environment Variables

```bash
export SLACK_BOT_TOKEN="xoxb-your-token-here"
export SLACK_SIGNING_SECRET="your-signing-secret"
```

## See Also

- [Triggers Overview](../concepts/triggers.md)
- [Webhook Triggers](../concepts/webhook-triggers.md)
- [Discord Mention Trigger](discord-mention-trigger.md)
- [Telegram Mention Trigger](telegram-mention-trigger.md)
