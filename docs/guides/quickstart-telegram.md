# Telegram Bot Quickstart

Get your AOF Telegram bot running in 5 minutes.

## What You Need

**Use the built-in config or create your own:**

```
your-project/
├── config.yaml           # Bot configuration (or use examples/config/daemon.yaml)
├── agents/               # Your agents
└── fleets/               # Your fleets (optional)
```

## Step 1: Use or Create Bot Config

**Option A: Use the built-in config:**
```bash
# Built-in config already has Telegram enabled
aofctl serve --config examples/config/daemon.yaml
```

**Option B: Create your own `config.yaml`:**

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-bot

spec:
  server:
    port: 3000
    host: "0.0.0.0"

  platforms:
    telegram:
      enabled: true
      bot_token_env: "TELEGRAM_BOT_TOKEN"

  agents:
    directory: "./agents"

  runtime:
    default_agent: "devops"
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
# Using built-in example config (Telegram already enabled)
aofctl serve --config examples/config/daemon.yaml

# Or with your own config
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
/agent                   # Switch agents (interactive)
/agent k8s               # Switch to Kubernetes agent
list pods                # Natural language query
kubectl get deployments  # Direct commands
```

## Built-in Agents

The default config includes these agents from `examples/agents/`:

| Agent | Tools | Use Case |
|-------|-------|----------|
| `devops` | kubectl, docker, helm, git, shell | Full-stack DevOps |
| `k8s-ops` | kubectl, helm | Kubernetes operations |
| `sre-agent` | prometheus, loki, kubectl | Observability & SRE |
| `security` | security tools | Security scanning |

## How Agents Work

The daemon routes messages to agents based on the `default_agent` setting or AgentFlow routing:

```
You: "why are pods crashing?"
         │
         ▼
    devops agent
    (kubectl, docker, etc.)
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

**Agent not found?**
- Ensure `agents.directory` in config points to directory with agent YAML files
- Check agent file has correct `metadata.name` matching `default_agent`

**"Write operation blocked"?**
- This is expected! Telegram is read-only for safety
- Use Slack or CLI for write operations
