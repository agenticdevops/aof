# Tutorial: Build a WhatsApp Ops Bot with AOF

Build a mobile-first WhatsApp bot for DevOps operations using AOF. Ideal for on-call engineers who need AI-powered infrastructure access from their phone.

**What you'll learn:**
- Set up Meta Business and WhatsApp Cloud API
- Configure AOF with WhatsApp integration
- Create agents optimized for mobile
- Use interactive buttons and lists
- Handle button and list responses

**Time:** 20 minutes

## Quick Start

```bash
# Set environment variables
export WHATSAPP_PHONE_NUMBER_ID=123456789012345
export WHATSAPP_ACCESS_TOKEN=EAAxxxxxxxxxxxxxxxxxxxxxxx
export WHATSAPP_VERIFY_TOKEN=my-secret-verify-token
export WHATSAPP_APP_SECRET=abcdef1234567890
export GOOGLE_API_KEY=xxxxx

# Start AOF server
aofctl serve \
  --config examples/configs/whatsapp-bot.yaml \
  --agents-dir examples/agents

# Expose with ngrok
ngrok http 3000
# Configure webhook URL in Meta Developer Console
```

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Meta Business Account (free at [business.facebook.com](https://business.facebook.com))
- Google AI API key (or Anthropic)
- Phone number for WhatsApp Business

## Step 1: Set Up Meta Business

### Create Meta Business Account

1. Go to [business.facebook.com](https://business.facebook.com)
2. Click **Create Account**
3. Follow the setup wizard
4. Verify your business (optional for testing)

### Create WhatsApp Business App

1. Go to [developers.facebook.com](https://developers.facebook.com)
2. Click **My Apps** → **Create App**
3. Select **Business** type
4. Choose your Meta Business Account
5. Name your app (e.g., "AOF Ops Bot")

### Add WhatsApp Product

1. In your app dashboard, click **Add Product**
2. Find **WhatsApp** and click **Set Up**
3. This creates a test phone number for development

### Get Credentials

From **WhatsApp** → **API Setup**:

```
Phone Number ID: 123456789012345
Access Token: EAAxxxxxxxx... (click Generate for permanent token)
```

From **App Settings** → **Basic**:

```
App Secret: abcdef1234567890
```

Create your own verify token (any random string):

```
Verify Token: my-secret-verify-token-2024
```

## Step 2: Create Agent Configuration

Create `agents/whatsapp-ops.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: whatsapp-ops
  labels:
    platform: whatsapp
    capability: devops

spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 1024

  description: "Mobile DevOps assistant for WhatsApp"

  tools:
    - kubectl
    - docker

  system_prompt: |
    You are a DevOps assistant on WhatsApp mobile.

    ## Response Guidelines
    - Be EXTREMELY concise (small mobile screens)
    - Use emoji for status: ✅ ⚠️ ❌ 🔍
    - Max 3-4 lines per response when possible
    - Use code blocks sparingly (hard to read on mobile)
    - Offer buttons for common follow-up actions

    ## Action Buttons
    When appropriate, suggest up to 3 actions:
    - view_logs: View Logs
    - describe_pod: Describe Pod
    - check_events: Check Events
    - list_pods: List Pods
    - check_nodes: Check Nodes

    ## Safety
    This is a read-only platform. For write operations,
    suggest using Slack or CLI instead.
```

## Step 3: Create Daemon Configuration

Create `config/whatsapp-bot.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: whatsapp-ops-bot

spec:
  server:
    port: 3000
    host: "0.0.0.0"

  platforms:
    whatsapp:
      enabled: true
      phone_number_id_env: WHATSAPP_PHONE_NUMBER_ID
      access_token_env: WHATSAPP_ACCESS_TOKEN
      verify_token_env: WHATSAPP_VERIFY_TOKEN
      app_secret_env: WHATSAPP_APP_SECRET

  agents:
    directory: "./agents"

  runtime:
    default_agent: whatsapp-ops
    max_concurrent_tasks: 5
    task_timeout_secs: 60
```

## Step 4: Start the Server

```bash
# Set environment variables
export WHATSAPP_PHONE_NUMBER_ID="123456789012345"
export WHATSAPP_ACCESS_TOKEN="EAAxxxxxxxxxxxxxxxxxxxxxxx"
export WHATSAPP_VERIFY_TOKEN="my-secret-verify-token"
export WHATSAPP_APP_SECRET="abcdef1234567890"
export GOOGLE_API_KEY="your-google-api-key"

# Start server
aofctl serve \
  --config config/whatsapp-bot.yaml \
  --agents-dir agents/

# In another terminal, start ngrok
ngrok http 3000
# Note the HTTPS URL: https://abc123.ngrok.io
```

## Step 5: Configure Webhook

In Meta Developer Console:

1. Go to **WhatsApp** → **Configuration**
2. Click **Edit** on Webhook
3. Set **Callback URL**: `https://abc123.ngrok.io/webhook/whatsapp`
4. Set **Verify Token**: `my-secret-verify-token` (same as env var)
5. Click **Verify and Save**
6. Under **Webhook Fields**, subscribe to **messages**

## Step 6: Add Test Phone Number

1. Go to **WhatsApp** → **API Setup**
2. Under **Send and receive messages**, add your phone number
3. You'll receive a verification code via WhatsApp
4. Enter the code to verify

## Step 7: Test the Bot

Send a message to the test phone number from WhatsApp:

### Basic Query

```
You: show pods
Bot: 🔍 Pods in default namespace:

     api-server-abc12   ✅ Running
     worker-xyz98       ✅ Running
     db-primary-qrs45   ✅ Running

     [List All] [Check Nodes] [View Events]
```

### Interactive Buttons

Tap a button to trigger follow-up action:

```
You: *taps [Check Nodes]*
Bot: 🖥️ Cluster Nodes:

     node-1   Ready   8 CPU   32GB
     node-2   Ready   8 CPU   32GB
     node-3   Ready   8 CPU   32GB

     ✅ All nodes healthy
```

### Agent Switching

```
You: /agent
Bot: Select an agent:

     📋 View Agents

     [Tap to open list]
```

List shows:
- ☸️ K8s Agent - Kubernetes operations
- 🐳 Docker Agent - Container management
- 🔧 DevOps Agent - Full-stack DevOps

## Interactive Message Examples

### Reply Buttons

When agent suggests actions:

```rust
TriggerResponse {
    text: "Pod api-server is in CrashLoopBackOff",
    status: ResponseStatus::Warning,
    actions: vec![
        Action { id: "view_logs", label: "View Logs" },
        Action { id: "describe_pod", label: "Describe" },
        Action { id: "restart_pod", label: "Restart" },
    ],
}
```

User sees:
```
⚠️ Pod api-server is in CrashLoopBackOff

[View Logs] [Describe] [Restart]
```

### List Messages

For agent selection:

```yaml
# Message with list
Header: "Select Agent"
Body: "Choose an agent for your task"
Button: "View Agents"

Sections:
  - title: "DevOps"
    items:
      - K8s Agent: "Kubernetes operations"
      - Docker Agent: "Container management"
  - title: "Monitoring"
    items:
      - Prometheus Agent: "Metrics and alerts"
      - Loki Agent: "Log analysis"
```

## Handling Button Responses

When user taps a button, AOF receives:

```
Text: button:view_logs
```

Configure agent to handle:

```yaml
system_prompt: |
  ## Button Handling
  When you receive a message starting with "button:",
  it's a button tap. Handle these:

  - button:view_logs → Show recent logs
  - button:describe_pod → Show pod details
  - button:restart_pod → Explain this requires Slack/CLI
```

## Available Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help and current agent |
| `/agent` | List agents with interactive list |
| `/agent <name>` | Switch to specific agent |
| `/fleet` | List fleets with interactive list |
| `/fleet <name>` | Switch to specific fleet |
| `/status` | Check service status |

## Built-in Agents

| Agent | Tools | Use Case |
|-------|-------|----------|
| `k8s-ops` | kubectl, helm | Kubernetes operations |
| `docker-ops` | docker, shell | Container management |
| `devops` | kubectl, docker, terraform, git | Full-stack DevOps |

## Safety: Read-Only Mode

Like Telegram, WhatsApp is **read-only** by default:

- ✅ Read operations work: `kubectl get`, `docker ps`
- ❌ Write operations blocked: `kubectl delete`, `docker rm`

This protects against accidental commands from mobile.

## Production Setup

### 1. Verify Business

For production use:

1. Go to Meta Business Suite
2. Complete business verification
3. Submit WhatsApp display name for approval
4. Wait for Meta review (1-3 business days)

### 2. Phone Number Whitelist

Restrict access in config:

```yaml
platforms:
  whatsapp:
    enabled: true
    phone_number_id_env: WHATSAPP_PHONE_NUMBER_ID
    access_token_env: WHATSAPP_ACCESS_TOKEN
    verify_token_env: WHATSAPP_VERIFY_TOKEN
    app_secret_env: WHATSAPP_APP_SECRET

    # Only allow these phone numbers
    allowed_numbers:
      - "14155551234"   # On-call lead
      - "14155555678"   # Team member
```

### 3. Use Permanent Token

Test tokens expire. For production:

1. Go to **WhatsApp** → **API Setup**
2. Click **Add system user** in Business Settings
3. Generate permanent token for system user
4. Use this token in production

### 4. Run as Service

```bash
# systemd service
sudo tee /etc/systemd/system/aof-whatsapp.service << 'EOF'
[Unit]
Description=AOF WhatsApp Bot
After=network.target

[Service]
Type=simple
User=aof
Environment=WHATSAPP_PHONE_NUMBER_ID=xxx
Environment=WHATSAPP_ACCESS_TOKEN=xxx
Environment=WHATSAPP_VERIFY_TOKEN=xxx
Environment=WHATSAPP_APP_SECRET=xxx
Environment=GOOGLE_API_KEY=xxx
ExecStart=/usr/local/bin/aofctl serve --config /etc/aof/whatsapp-bot.yaml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable --now aof-whatsapp
```

### 5. Use Cloudflare Tunnel

Free alternative to ngrok for production:

```bash
brew install cloudflared

# Create tunnel
cloudflared tunnel create aof-whatsapp
cloudflared tunnel route dns aof-whatsapp whatsapp-bot.yourdomain.com

# Run tunnel
cloudflared tunnel run aof-whatsapp
```

## WhatsApp Limits

Keep these in mind when building agents:

| Limit | Value |
|-------|-------|
| Message body | 4096 characters |
| Reply buttons | 3 maximum |
| Button text | 20 characters |
| List sections | 10 maximum |
| Items per section | 10 maximum |

## Troubleshooting

### Webhook not verifying

```bash
# Check server is running
curl http://localhost:3000/health

# Check ngrok is forwarding
curl https://your-ngrok-url.ngrok.io/health

# Verify token matches
echo $WHATSAPP_VERIFY_TOKEN
```

### Messages not arriving

1. Check webhook subscribed to **messages** field
2. Verify phone number is added and verified
3. Check server logs for signature errors

```bash
RUST_LOG=debug aofctl serve --config config/whatsapp-bot.yaml
```

### Invalid signature errors

```bash
# Verify app secret is correct
echo $WHATSAPP_APP_SECRET

# Check signature header in logs
# Should be: X-Hub-Signature-256: sha256=...
```

### Buttons not showing

- Max 3 buttons allowed
- Button titles max 20 characters
- Check response has `actions` array

## Cost Considerations

WhatsApp Business API has per-conversation pricing:

| Conversation Type | Cost (approximate) |
|-------------------|-------------------|
| User-initiated | $0.005 - $0.08 |
| Business-initiated | $0.01 - $0.15 |

First 1,000 user-initiated conversations/month are free.

## Next Steps

- [WhatsApp Reference](../reference/whatsapp-integration.md) - Full API documentation
- [Telegram Tutorial](./telegram-ops-bot.md) - Free alternative platform
- [Slack Bot Tutorial](./slack-bot.md) - Full read/write with approvals
- [Fleets Guide](../concepts/fleets.md) - Multi-agent routing
