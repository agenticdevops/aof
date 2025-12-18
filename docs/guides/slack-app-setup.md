# Slack App Setup Guide

This guide walks you through creating a Slack App for use with AOF's AgentFlow Slack integration.

## Step 1: Create Slack App

1. Go to [https://api.slack.com/apps](https://api.slack.com/apps)
2. Click **Create New App**
3. Choose **From scratch**
4. Enter:
   - **App Name**: `K8s Assistant Bot` (or your preferred name)
   - **Workspace**: Select your workspace
5. Click **Create App**

## Step 2: Configure Bot Permissions

1. In the left sidebar, click **OAuth & Permissions**
2. Scroll to **Scopes** → **Bot Token Scopes**
3. Add these scopes:
   - `chat:write` - Send messages
   - `app_mentions:read` - Receive mentions
   - `commands` - Slash commands (optional)
   - `reactions:read` - Read reactions for approvals
   - `reactions:write` - Add reactions

## Step 3: Install App to Workspace

1. In **OAuth & Permissions**, click **Install to Workspace**
2. Authorize the app
3. Copy the **Bot User OAuth Token** (starts with `xoxb-`)
4. Save it as an environment variable:
   ```bash
   export SLACK_BOT_TOKEN=xoxb-your-token-here
   ```

## Step 4: Get Signing Secret

1. In the left sidebar, click **Basic Information**
2. Scroll to **App Credentials**
3. Copy the **Signing Secret**
4. Save it as an environment variable:
   ```bash
   export SLACK_SIGNING_SECRET=your-signing-secret-here
   ```

## Step 5: Enable Event Subscriptions

1. In the left sidebar, click **Event Subscriptions**
2. Toggle **Enable Events** to **On**
3. For **Request URL**, you'll enter your webhook URL:
   - Development: `https://your-ngrok-url.ngrok.io/webhook/slack`
   - Production: `https://your-domain.com/webhook/slack`

4. Under **Subscribe to bot events**, add:
   - `app_mention` - When someone @mentions your bot
   - `message.channels` - Messages in public channels
   - `message.im` - Direct messages to bot
   - `reaction_added` - **Required for approval workflow** - when users react to approve/deny commands

5. Click **Save Changes**

> **Note**: The `reaction_added` event is required for the human-in-the-loop approval workflow.
> When an agent requests approval for a destructive command, users approve by reacting with ✅ or deny with ❌.
> See the [Approval Workflow Guide](approval-workflow.md) for details.

## Step 6: (Optional) Create Slash Command

1. In the left sidebar, click **Slash Commands**
2. Click **Create New Command**
3. Enter:
   - **Command**: `/k8s`
   - **Request URL**: Same as event subscriptions URL
   - **Short Description**: "Ask the K8s assistant"
4. Click **Save**

## Step 7: Invite Bot to Channels

1. In Slack, go to the channel where you want the bot
2. Type `/invite @your-bot-name`
3. Or click the channel name → Integrations → Add apps

## Development Setup with ngrok

For local development, use ngrok to expose your local server:

```bash
# Install ngrok (macOS)
brew install ngrok

# Start your AOF server
aofctl serve --port 3000

# In another terminal, start ngrok
ngrok http 3000

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
# Update your Slack app's Request URL to: https://abc123.ngrok.io/webhook/slack
```

## Environment Variables Summary

Create a `.env` file or export these variables:

```bash
# Required
export SLACK_BOT_TOKEN=xoxb-xxxxx-xxxxx-xxxxx
export SLACK_SIGNING_SECRET=xxxxx

# LLM Provider (choose one)
export ANTHROPIC_API_KEY=sk-ant-xxxxx       # For Claude
export OPENAI_API_KEY=sk-xxxxx              # For GPT
export GOOGLE_API_KEY=xxxxx                 # For Gemini (recommended for speed)

# Optional
export KUBECONFIG=~/.kube/config            # For kubectl access
```

## Testing Your Setup

1. Start the AOF server:
   ```bash
   aofctl serve --port 3000 --config examples/config/slack-daemon.yaml
   ```

2. In Slack, mention your bot:
   ```
   @K8s Assistant Bot what pods are running?
   ```

3. The bot should respond in a thread!

## Multi-Tenant Bot Setup

For routing different channels to different clusters, use AgentFlows with trigger filtering:

```bash
# Start with flows directory for multi-tenant routing
aofctl serve \
  --flows-dir ./examples/flows/multi-tenant/ \
  --agents-dir ./examples/agents/ \
  --port 3000

# Or use a daemon config
aofctl serve --config examples/config/slack-daemon.yaml
```

Example daemon config with flows:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-tenant-bot

spec:
  server:
    port: 3000

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

  agents:
    directory: ./examples/agents/

  # AgentFlow-based routing
  flows:
    directory: ./examples/flows/multi-tenant/
    enabled: true

  runtime:
    default_agent: fallback-bot  # Used when no flow matches
```

See [AgentFlow Spec - Multi-Tenant Routing](../reference/agentflow-spec.md#trigger-filtering-multi-tenant-routing) for configuration options.

## Troubleshooting

### Bot doesn't respond

- Check that the bot is invited to the channel
- Verify the Request URL is correct in Event Subscriptions
- Check AOF server logs: `RUST_LOG=info aofctl serve ...`
- Ensure ngrok is running and URL is correct

### "URL verification failed"

- Make sure your server is running before setting the Request URL
- Check that the URL ends with `/webhook/slack`
- Verify your signing secret is correct

### "Invalid signature"

- Double-check your `SLACK_SIGNING_SECRET`
- Ensure you're using the signing secret, not the bot token

### Rate limiting

- Slack has API rate limits
- Add delays between rapid requests
- Use threading to reduce message noise

## Production Deployment

For production:

1. Use a proper HTTPS endpoint (not ngrok)
2. Set up a reverse proxy (nginx, Caddy)
3. Use environment variables or secrets management
4. Enable logging and monitoring
5. Consider using a persistent database for agent memory

Example with Docker:
```bash
docker run -d \
  -e SLACK_BOT_TOKEN=$SLACK_BOT_TOKEN \
  -e SLACK_SIGNING_SECRET=$SLACK_SIGNING_SECRET \
  -e GOOGLE_API_KEY=$GOOGLE_API_KEY \
  -p 3000:3000 \
  aof:latest serve --port 3000 --config /config/slack-bot-flow.yaml
```
