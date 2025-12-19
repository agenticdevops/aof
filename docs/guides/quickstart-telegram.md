# Telegram Bot Quickstart

Get your AOF Telegram bot running in 5 minutes.

## What You Need

**3 files only:**

```
your-project/
├── config.yaml      # Bot configuration
└── agents/
    └── k8s-ops.yaml # Your agent (or use built-in)
```

## Step 1: Create Bot Config

Create `config.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-bot

spec:
  server:
    port: 8080
    host: "0.0.0.0"

  platforms:
    telegram:
      enabled: true
      bot_token_env: "TELEGRAM_BOT_TOKEN"

  agents:
    directory: "./agents"

  runtime:
    default_agent: "k8s-ops"
```

## Step 2: Get Telegram Bot Token

1. Open Telegram, search for `@BotFather`
2. Send `/newbot`
3. Follow prompts to name your bot
4. Copy the token (looks like `123456789:ABCdef...`)

## Step 3: Set Environment Variables

```bash
export TELEGRAM_BOT_TOKEN="your-bot-token-here"
export GOOGLE_API_KEY="your-google-ai-key"  # or ANTHROPIC_API_KEY
```

## Step 4: Run the Bot

```bash
# Using built-in agents
aofctl serve --config config.yaml --agents-dir /path/to/aof/examples/agents

# Or with your own agents directory
aofctl serve --config config.yaml
```

## Step 5: Set Webhook (for production)

```bash
# Use ngrok for local testing
ngrok http 8080

# Set webhook with Telegram
curl "https://api.telegram.org/bot$TELEGRAM_BOT_TOKEN/setWebhook?url=https://your-ngrok-url.ngrok.io/webhook/telegram"
```

## Using the Bot

Open your bot in Telegram and try:

```
/help                    # Show commands
/agent                   # Switch agents (k8s, aws, docker, devops)
list pods                # Natural language query
kubectl get deployments  # Direct commands
```

## Built-in Agents

| Command | Agent | Tools |
|---------|-------|-------|
| `/agent k8s` | k8s-ops | kubectl, helm |
| `/agent aws` | aws-agent | aws cli |
| `/agent docker` | docker-ops | docker, shell |
| `/agent devops` | devops | kubectl, docker, helm, terraform, git, shell |

## Safety

- **Telegram is read-only** - write operations are blocked
- Use Slack or CLI for write operations
- This protects against accidental destructive commands from mobile

## Troubleshooting

**Bot not responding?**
- Check webhook is set: `curl "https://api.telegram.org/bot$TELEGRAM_BOT_TOKEN/getWebhookInfo"`
- Check server logs: look for "Registered platform: telegram"

**Agent has no tools?**
- Ensure `--agents-dir` points to directory with agent YAML files
- Check agent file has correct `metadata.name`

**"Write operation blocked"?**
- This is expected! Telegram is read-only for safety
- Use Slack or CLI for write operations
