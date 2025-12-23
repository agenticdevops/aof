---
sidebar_position: 6
sidebar_label: Slack
---

# Slack MCP Server

Send messages and interact with Slack workspaces.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-slack` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/slack) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-slack
```

## Configuration

```yaml
mcp_servers:
  - name: slack
    command: npx
    args: ["-y", "@modelcontextprotocol/server-slack"]
    env:
      SLACK_BOT_TOKEN: ${SLACK_BOT_TOKEN}
      SLACK_TEAM_ID: ${SLACK_TEAM_ID}
```

### Required Environment Variables

- `SLACK_BOT_TOKEN`: Bot User OAuth Token (starts with `xoxb-`)
- `SLACK_TEAM_ID`: Your Slack workspace ID

### Bot Token Scopes

Your Slack app needs these OAuth scopes:
- `channels:history` - View messages in public channels
- `channels:read` - View basic channel info
- `chat:write` - Send messages
- `reactions:write` - Add reactions
- `users:read` - View users

## Tools

### slack_list_channels

List all channels in the workspace.

**Parameters**: None

**Returns**: Array of channel objects with id, name, and topic

### slack_post_message

Post a message to a channel.

**Parameters**:
- `channel_id` (string, required): Channel ID
- `text` (string, required): Message text

**Example**:
```json
{
  "tool": "slack_post_message",
  "arguments": {
    "channel_id": "C01234ABCDE",
    "text": "Deployment completed successfully! :rocket:"
  }
}
```

### slack_reply_to_thread

Reply to a thread.

**Parameters**:
- `channel_id` (string, required): Channel ID
- `thread_ts` (string, required): Thread timestamp
- `text` (string, required): Reply text

### slack_add_reaction

Add a reaction to a message.

**Parameters**:
- `channel_id` (string, required): Channel ID
- `timestamp` (string, required): Message timestamp
- `reaction` (string, required): Emoji name (without colons)

**Example**:
```json
{
  "tool": "slack_add_reaction",
  "arguments": {
    "channel_id": "C01234ABCDE",
    "timestamp": "1234567890.123456",
    "reaction": "white_check_mark"
  }
}
```

### slack_get_channel_history

Get recent messages from a channel.

**Parameters**:
- `channel_id` (string, required): Channel ID
- `limit` (number, optional): Number of messages (default: 10)

### slack_get_thread_replies

Get replies to a thread.

**Parameters**:
- `channel_id` (string, required): Channel ID
- `thread_ts` (string, required): Thread timestamp

### slack_get_users

List users in the workspace.

**Parameters**: None

**Returns**: Array of user objects

### slack_get_user_profile

Get a user's profile.

**Parameters**:
- `user_id` (string, required): User ID

## Use Cases

### Alert Notifier Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: alert-notifier
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: slack
      command: npx
      args: ["-y", "@modelcontextprotocol/server-slack"]
      env:
        SLACK_BOT_TOKEN: ${SLACK_BOT_TOKEN}
        SLACK_TEAM_ID: ${SLACK_TEAM_ID}
  system_prompt: |
    You send alert notifications to Slack:
    1. Format alerts clearly with severity indicators
    2. Use appropriate emojis for alert types
    3. Include actionable information
    4. Thread follow-up messages
```

### Incident Coordinator

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: incident-coordinator
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: slack
      command: npx
      args: ["-y", "@modelcontextprotocol/server-slack"]
      env:
        SLACK_BOT_TOKEN: ${SLACK_BOT_TOKEN}
        SLACK_TEAM_ID: ${SLACK_TEAM_ID}
  system_prompt: |
    You coordinate incidents via Slack:
    - Create incident threads
    - Post status updates
    - Tag relevant team members
    - Track resolution progress
    - Post postmortem summaries
```

### Deployment Announcer

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: deploy-announcer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: slack
      command: npx
      args: ["-y", "@modelcontextprotocol/server-slack"]
      env:
        SLACK_BOT_TOKEN: ${SLACK_BOT_TOKEN}
        SLACK_TEAM_ID: ${SLACK_TEAM_ID}
  system_prompt: |
    You announce deployments to Slack:
    - Pre-deployment notifications
    - Progress updates
    - Success/failure notifications
    - Rollback alerts

    Format messages with:
    - Environment name
    - Version being deployed
    - Expected duration
    - Rollback plan
```

## Message Formatting

### Basic Formatting

```
*bold* _italic_ ~strikethrough~ `code`
```

### Code Blocks

````
```python
print("Hello, World!")
```
````

### Links

```
<https://example.com|Link Text>
```

### User Mentions

```
<@U01234ABCDE> mentioned you
```

### Channel Links

```
Check <#C01234ABCDE|channel-name>
```

## Creating a Slack App

1. Go to https://api.slack.com/apps
2. Click "Create New App"
3. Choose "From scratch"
4. Name your app and select workspace
5. Go to "OAuth & Permissions"
6. Add required Bot Token Scopes
7. Install app to workspace
8. Copy Bot User OAuth Token

## Rate Limits

Slack API has rate limits:
- **Tier 1**: 1+ per minute (basic reads)
- **Tier 2**: 20+ per minute (common operations)
- **Tier 3**: 50+ per minute (posting messages)
- **Tier 4**: 100+ per minute (high-volume)

The server handles rate limiting with automatic retries.

## Troubleshooting

### Invalid Token

Verify your token is correct:
```bash
curl -H "Authorization: Bearer $SLACK_BOT_TOKEN" \
  https://slack.com/api/auth.test
```

### Missing Scopes

Check app permissions at: https://api.slack.com/apps/YOUR_APP_ID/oauth

### Channel Not Found

Ensure the bot is invited to the channel:
```
/invite @your-bot-name
```

### Rate Limited

Wait for the retry-after period or reduce request frequency.
