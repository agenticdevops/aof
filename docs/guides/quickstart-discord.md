# Discord Bot Quickstart

Get AOF running with Discord in 10 minutes.

## Prerequisites

- Discord account
- A Discord server where you have admin permissions
- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Public HTTPS endpoint (or ngrok for development)

## Step 1: Create Discord Application

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application"
3. Name it "AOF Bot" (or your preferred name)
4. Click "Create"

## Step 2: Get Application Credentials

In your application settings:

1. **General Information**:
   - Copy **Application ID**
   - Copy **Public Key**

2. **Bot** section:
   - Click "Add Bot"
   - Click "Reset Token" and copy the **Bot Token**
   - Enable "Message Content Intent" if needed

```bash
# Set environment variables
export DISCORD_APPLICATION_ID="your-application-id"
export DISCORD_PUBLIC_KEY="your-public-key"
export DISCORD_BOT_TOKEN="your-bot-token"
```

## Step 3: Configure Interactions Endpoint

### For Development (ngrok)

```bash
# Start ngrok
ngrok http 8080

# Note the HTTPS URL: https://xxx.ngrok.io
```

### Set Webhook URL

1. In Developer Portal, go to "General Information"
2. Find "Interactions Endpoint URL"
3. Enter: `https://your-domain.com/webhook/discord`
4. Discord will verify the endpoint (make sure AOF is running first)

## Step 4: Invite Bot to Server

1. Go to **OAuth2** → **URL Generator**
2. Select scopes:
   - `bot`
   - `applications.commands`
3. Select bot permissions:
   - Send Messages
   - Embed Links
   - Use Slash Commands
4. Copy the generated URL
5. Open URL in browser and select your server

## Step 5: Create AOF Configuration

```bash
mkdir -p ~/.aof
```

### Trigger Configuration

```yaml
# ~/.aof/triggers/discord-starter.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-starter
  labels:
    platform: discord
spec:
  type: Discord
  config:
    bot_token: ${DISCORD_BOT_TOKEN}
    application_id: ${DISCORD_APPLICATION_ID}
    public_key: ${DISCORD_PUBLIC_KEY}

    # Optional: Restrict to specific servers
    # guild_ids:
    #   - "your-guild-id"

  commands:
    /help:
      agent: devops
      description: "Show available commands"

    /status:
      agent: devops
      description: "Check system status"

    /agent:
      agent: devops
      description: "Manage agents"

  default_agent: devops
```

### Agent Configuration

```yaml
# ~/.aof/agents/devops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: devops
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "DevOps assistant for Discord"

  tools:
    - kubectl
    - docker
    - helm

  system_prompt: |
    You are a DevOps assistant in Discord.

    ## Response Guidelines
    - Use Discord embed formatting
    - Include status indicators: ✅ ⚠️ ❌
    - Keep responses concise
    - Offer button actions when appropriate

    ## For status queries:
    📊 **System Status**

    ✅ All systems operational

    • API: Running
    • Database: Connected
    • Cache: Healthy

    Use [Refresh] [View Logs] buttons for actions.
```

### Daemon Configuration

```yaml
# ~/.aof/daemon.yaml
log_level: info
http_port: 8080

platforms:
  discord:
    enabled: true
    webhook_path: /webhook/discord
```

## Step 6: Start AOF

```bash
# Validate configuration
aofctl validate ~/.aof/triggers/discord-starter.yaml
aofctl validate ~/.aof/agents/devops.yaml

# Start daemon
aofctl daemon start

# Check status
aofctl daemon status
```

## Step 7: Register Commands

```bash
# Register slash commands globally (takes up to 1 hour)
aofctl discord register-commands

# Or register to specific guild (instant)
aofctl discord register-commands --guild-id YOUR_GUILD_ID
```

## Step 8: Test Your Bot

In Discord, try:

```
/help
/status
/agent action:run agent_id:devops
```

### Expected Response

```
📊 System Status

✅ All systems operational

┌────────────────────────────┐
│ Component │ Status         │
├───────────┼────────────────┤
│ API       │ ✅ Running     │
│ Database  │ ✅ Connected   │
│ Cache     │ ✅ Healthy     │
└────────────────────────────┘

[Refresh] [View Logs] [Details]
```

## Troubleshooting

### Endpoint Verification Failed

```bash
# Make sure AOF is running first
aofctl daemon status

# Check logs
aofctl daemon logs --follow

# Verify public key is correct
echo $DISCORD_PUBLIC_KEY
```

### Commands Not Showing

```bash
# Global commands can take up to 1 hour
# Use guild-specific commands for instant updates
aofctl discord register-commands --guild-id YOUR_GUILD_ID

# List registered commands
aofctl discord list-commands
```

### Bot Not Responding

```bash
# Check webhook is accessible
curl -X POST https://your-domain.com/webhook/discord \
  -H "Content-Type: application/json" \
  -d '{"type": 1}'

# Should return {"type": 1}

# Check daemon logs
aofctl daemon logs | grep discord
```

### Signature Verification Errors

```bash
# Verify your public key matches the one in Developer Portal
# The public key should be 64 hex characters

# Check for timing issues
# Discord requires response within 3 seconds
```

## Next Steps

- [Discord Tutorial](../tutorials/discord-ops-bot.md) - Build a complete ops bot
- [Discord Reference](../reference/discord-integration.md) - Full API reference
- [Custom Tools](../tools/custom-tools.md) - Add your own tools
- [Deployment Guide](../guides/deployment.md) - Production deployment
