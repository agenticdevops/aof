# Tutorial: Build a Telegram Ops Bot with AOF

Build a mobile-friendly Telegram bot for DevOps operations using AOF. Perfect for on-call engineers who need quick access to cluster status from their phone.

**What you'll learn:**
- Set up AOF with Telegram integration
- Create agents and fleets for different use cases
- Use inline keyboards for agent/fleet switching
- Understand read-only safety for mobile platforms

**Time:** 10 minutes

## Quick Start

```bash
# Set up environment
export TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrSTUvwxYZ
export GOOGLE_API_KEY=xxxxx

# Start AOF server
aofctl serve \
  --config examples/configs/telegram-bot.yaml \
  --agents-dir examples/agents \
  --fleets-dir examples/fleets

# In another terminal, expose with ngrok
ngrok http 8080

# Set webhook with Telegram
curl -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -d "url=https://your-ngrok-url.ngrok.io/webhook/telegram"
```

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Telegram account
- Google AI API key (or Anthropic/OpenAI)
- Optional: Kubernetes cluster for kubectl commands

## Step 1: Create Telegram Bot

### Talk to BotFather

1. Open Telegram and search for `@BotFather`
2. Send `/newbot`
3. Follow the prompts:
   - Bot name: `My Ops Bot` (display name)
   - Bot username: `my_ops_bot` (must end in `bot`)
4. Copy the **HTTP API token**:

```
Done! Your new bot is created.
Use this token to access the HTTP API:
123456789:ABCdefGHIjklMNOpqrSTUvwxYZ
```

### Set Bot Commands (Optional)

Send to @BotFather:

```
/setcommands
```

Select your bot and paste:

```
help - Show available commands
agent - Switch agent
fleet - Switch fleet
```

## Step 2: Create Agent Configuration

Create `agents/k8s-ops.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    platform: telegram
    capability: kubernetes

spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "Kubernetes operations assistant"

  tools:
    - kubectl
    - helm

  system_prompt: |
    You are a Kubernetes operations assistant on Telegram mobile.

    ## Your Role
    - Answer K8s questions clearly and concisely
    - Run kubectl/helm commands when requested
    - Troubleshoot cluster issues
    - Keep responses SHORT - this is mobile chat

    ## Response Format
    - Be extremely concise (mobile screens are small)
    - Use code blocks for command output
    - Use emoji for status: ✅ ⚠️ ❌
    - Truncate long outputs, offer to show more

    ## Safety
    This is a READ-ONLY platform. If user requests write operations,
    explain they should use Slack or CLI instead.
```

## Step 3: Create Daemon Configuration

Create `config/telegram-bot.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-ops-bot

spec:
  server:
    port: 8080
    host: "0.0.0.0"

  platforms:
    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN

      # Optional: Restrict to specific users/groups
      # allowed_users:
      #   - 123456789  # Telegram user IDs
      # allowed_groups:
      #   - -1001234567890  # Telegram group IDs

  agents:
    directory: "./agents"

  fleets:
    directory: "./fleets"

  runtime:
    default_agent: k8s-ops
    max_concurrent_tasks: 5
    task_timeout_secs: 120
```

## Step 4: Start the Server

```bash
# Set environment variables
export TELEGRAM_BOT_TOKEN="123456789:ABCdefGHIjklMNOpqrSTUvwxYZ"
export GOOGLE_API_KEY="your-google-api-key"

# Start server
aofctl serve \
  --config config/telegram-bot.yaml \
  --agents-dir agents/

# In another terminal, start ngrok
ngrok http 8080
# Note the HTTPS URL: https://abc123.ngrok.io

# Set webhook
curl -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -d "url=https://abc123.ngrok.io/webhook/telegram"

# Verify webhook
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo"
```

## Step 5: Test the Bot

Open your bot in Telegram and try these commands:

### Basic Commands

```
/help
```

Shows available commands and current agent:

```
AOF Bot - DevOps from mobile

Current agent: 🎯 K8s Ops
Tools: kubectl, helm

Commands:
/fleet - Switch fleet (recommended)
/agent - Switch agent
/help - Show this help

Just type naturally after selecting an agent.
```

### Switch Agent

```
/agent
```

Shows inline keyboard with available agents:

```
Select an agent:

[🎯 K8s Ops] [🐳 Docker]
[☁️ AWS] [🔧 DevOps]
```

Tap a button to switch. Or switch directly:

```
/agent docker
```

### Natural Language Queries

After selecting an agent, just type naturally:

```
show pods in production
```

Response:
```
🔍 Pods in production:

NAME                    STATUS   AGE
nginx-abc123            Running  5d
api-xyz789              Running  3d
worker-def456           Running  1d

✅ 3 pods healthy
```

### Use Fleets

Fleets are teams of agents with automatic routing:

```
/fleet
```

Shows available fleets:

```
Select a fleet:

[🚀 DevOps Fleet] [☸️ K8s Fleet]
[🔍 RCA Fleet] [☁️ AWS Fleet]
```

Switch to a fleet:

```
/fleet devops
```

Now queries are routed to the right specialist:

```
why is the API slow?
```

The DevOps fleet routes to prometheus-agent for metrics, then k8s-agent for pod analysis.

## Available Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help and current agent |
| `/agent` | List agents with inline buttons |
| `/agent <name>` | Switch to specific agent |
| `/agent info` | Show current agent details |
| `/fleet` | List fleets with inline buttons |
| `/fleet <name>` | Switch to specific fleet |
| `/fleet info` | Show current fleet details |
| `/run agent <name> <query>` | Run specific agent once |
| `/status task <id>` | Check task status |

## Built-in Agents

| Agent | Tools | Use Case |
|-------|-------|----------|
| `k8s-ops` | kubectl, helm | Kubernetes operations |
| `docker-ops` | docker, shell | Container management |
| `aws-agent` | aws | AWS CLI operations |
| `devops` | kubectl, docker, terraform, git | Full-stack DevOps |

## Built-in Fleets

| Fleet | Agents | Use Case |
|-------|--------|----------|
| `devops-fleet` | k8s, docker, prometheus, git | General DevOps |
| `k8s-fleet` | k8s, prometheus, loki | Kubernetes focus |
| `rca-fleet` | collectors + analysts | Root cause analysis |
| `aws-fleet` | aws, terraform | AWS operations |

## Safety: Read-Only Mode

Telegram is configured as a **read-only platform** by default. This means:

- ✅ Read operations work: `kubectl get`, `docker ps`, `aws describe-*`
- ❌ Write operations blocked: `kubectl delete`, `docker rm`, `aws terminate-*`

This protects against accidental destructive commands from mobile.

### Why Read-Only?

1. **Mobile context** - Easy to make mistakes on small screens
2. **No approval UI** - Telegram lacks reaction-based approval like Slack
3. **Safety first** - Critical operations should use Slack or CLI

### Override (Not Recommended)

If you need write access, configure a separate daemon:

```yaml
# telegram-write-enabled.yaml (USE WITH CAUTION)
platforms:
  telegram:
    enabled: true
    bot_token_env: TELEGRAM_BOT_TOKEN
    # This removes read-only protection:
    safety:
      read_only: false
```

## Production Setup

### 1. Use Cloudflared (Free, No Signup)

```bash
brew install cloudflared
cloudflared tunnel --url http://localhost:8080

# Use the URL for webhook:
# https://random-words.trycloudflare.com/webhook/telegram
```

### 2. Restrict Users

Only allow specific Telegram users:

```yaml
platforms:
  telegram:
    enabled: true
    bot_token_env: TELEGRAM_BOT_TOKEN

    # Only these users can use the bot
    allowed_users:
      - 123456789  # Your Telegram user ID
      - 987654321  # Team member

    # Or restrict to specific groups
    allowed_groups:
      - -1001234567890  # Your ops group
```

Find your Telegram user ID: Message `@userinfobot`

### 3. Run as Service

```bash
# systemd service
sudo tee /etc/systemd/system/aof-telegram.service << 'EOF'
[Unit]
Description=AOF Telegram Bot
After=network.target

[Service]
Type=simple
User=aof
Environment=TELEGRAM_BOT_TOKEN=xxx
Environment=GOOGLE_API_KEY=xxx
ExecStart=/usr/local/bin/aofctl serve --config /etc/aof/telegram-bot.yaml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable --now aof-telegram
```

## Troubleshooting

### Bot not responding

```bash
# Check webhook is set
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo"

# Should show:
# "url": "https://your-url/webhook/telegram"
# "pending_update_count": 0

# Check server logs
RUST_LOG=debug aofctl serve --config config/telegram-bot.yaml
```

### "Write operation blocked"

This is expected! Telegram is read-only for safety. Use:
- **Slack** for write operations with approval workflow
- **CLI** for direct command execution

### Webhook SSL errors

Telegram requires valid HTTPS. Use:
- `cloudflared` (free, automatic SSL)
- `ngrok` (free tier available)
- Proper SSL certificate in production

### Inline buttons not showing

1. Check bot has no pending updates: restart server
2. Verify agents/fleets directories have YAML files
3. Check server logs for parsing errors

## Next Steps

- [Slack Bot Tutorial](./slack-bot.md) - Full read/write with approval workflow
- [Fleets Guide](../concepts/fleets.md) - Understanding fleet routing
- [Multi-Model RCA](./multi-model-rca.md) - Advanced root cause analysis
- [Safety Layer Guide](../guides/safety-layer.md) - Platform safety configuration
