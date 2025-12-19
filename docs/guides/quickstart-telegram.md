# Telegram Bot Quickstart

Get your AOF Telegram bot running in 5 minutes.

## What You Need

**3 files only:**

```
your-project/
├── config.yaml           # Bot configuration
└── fleets/
    └── devops-fleet.yaml # Your fleet (or use built-in)
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

  fleets:
    directory: "./fleets"

  runtime:
    default_fleet: "devops-fleet"
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
# Using built-in fleets
aofctl serve --config config.yaml --fleets-dir /path/to/aof/examples/fleets

# Or with your own fleets directory
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
/fleet                   # Switch fleets
/fleet devops            # Switch to DevOps fleet
list pods                # Natural language query
kubectl get deployments  # Direct commands
```

## Built-in Fleets

| Command | Fleet | Agents |
|---------|-------|--------|
| `/fleet devops` | devops-fleet | k8s, docker, git, prometheus |
| `/fleet k8s` | k8s-fleet | k8s, prometheus, loki |
| `/fleet aws` | aws-fleet | aws, terraform |
| `/fleet database` | database-fleet | postgres, redis |
| `/fleet rca` | rca-fleet | collectors + multi-model analysts |

## How Fleets Work

Fleets are teams of single-purpose agents. When you ask a question, the fleet routes to the right specialist:

```
You: "why are pods crashing?"
         │
         ▼
    DevOps Fleet
    (coordinator)
         │
         ▼
    k8s-agent  ← kubectl specialist
         │
         ▼
    Response with analysis
```

## Safety

- **Telegram is read-only** - write operations are blocked
- Use Slack or CLI for write operations
- This protects against accidental destructive commands from mobile

## Troubleshooting

**Bot not responding?**
- Check webhook is set: `curl "https://api.telegram.org/bot$TELEGRAM_BOT_TOKEN/getWebhookInfo"`
- Check server logs: look for "Registered platform: telegram"

**Fleet has no agents?**
- Ensure `--fleets-dir` points to directory with fleet YAML files
- Check fleet file has correct `metadata.name`

**"Write operation blocked"?**
- This is expected! Telegram is read-only for safety
- Use Slack or CLI for write operations
