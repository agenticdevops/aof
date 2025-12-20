# WhatsApp Bot Quickstart

Get your AOF WhatsApp bot running in 10 minutes.

## What You Need

**Use the built-in config or create your own:**

```
your-project/
├── config.yaml           # Bot configuration
├── agents/               # Your agents
└── fleets/               # Your fleets (optional)
```

## Step 1: Meta Business Setup

### Create Developer App

1. Go to [developers.facebook.com](https://developers.facebook.com)
2. Click **My Apps** → **Create App**
3. Select **Business** → Connect to a Meta Business Account
4. Name your app → **Create App**

### Add WhatsApp

1. In app dashboard, find **WhatsApp** → **Set Up**
2. This creates a test phone number

### Get Credentials

From **WhatsApp** → **API Setup**:
- **Phone Number ID**: `123456789012345`
- **Access Token**: Click **Generate** for permanent token

From **App Settings** → **Basic**:
- **App Secret**: `abcdef1234567890`

## Step 2: Create Bot Config

Create `config.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: whatsapp-bot

spec:
  server:
    port: 3000
    host: "0.0.0.0"

  platforms:
    whatsapp:
      enabled: true
      phone_number_id_env: "WHATSAPP_PHONE_NUMBER_ID"
      access_token_env: "WHATSAPP_ACCESS_TOKEN"
      verify_token_env: "WHATSAPP_VERIFY_TOKEN"
      app_secret_env: "WHATSAPP_APP_SECRET"

  agents:
    directory: "./agents"

  runtime:
    default_agent: "devops"
```

## Step 3: Set Environment Variables

```bash
export WHATSAPP_PHONE_NUMBER_ID="123456789012345"
export WHATSAPP_ACCESS_TOKEN="EAAxxxxxxxxxxxxxxxxxxxxxxx"
export WHATSAPP_VERIFY_TOKEN="my-random-verify-token"
export WHATSAPP_APP_SECRET="abcdef1234567890"
export GOOGLE_API_KEY="your-google-ai-key"
```

The verify token can be any random string you create.

## Step 4: Run the Bot

```bash
# Start AOF server
aofctl serve --config config.yaml

# In another terminal, expose with ngrok
ngrok http 3000
```

Note the HTTPS URL from ngrok (e.g., `https://abc123.ngrok.io`).

## Step 5: Configure Webhook

In Meta Developer Console:

1. Go to **WhatsApp** → **Configuration**
2. Click **Edit** on Webhook
3. **Callback URL**: `https://abc123.ngrok.io/webhook/whatsapp`
4. **Verify Token**: `my-random-verify-token` (same as env var)
5. Click **Verify and Save**
6. Subscribe to **messages** field

## Step 6: Add Your Phone Number

1. Go to **WhatsApp** → **API Setup**
2. Under **Send and receive messages**, add your phone number
3. Enter the verification code sent via WhatsApp

## Using the Bot

Send messages to the test phone number:

```
You: show pods
Bot: 🔍 Pods in default namespace:

     api-server   ✅ Running
     worker       ✅ Running

     [View Logs] [Check Nodes]
```

Tap buttons for quick actions!

## Built-in Agents

| Agent | Tools | Use Case |
|-------|-------|----------|
| `devops` | kubectl, docker, helm, git | Full-stack DevOps |
| `k8s-ops` | kubectl, helm | Kubernetes operations |
| `sre-agent` | prometheus, loki, kubectl | Observability & SRE |

## How It Works

```
WhatsApp → Meta Cloud API → Your Server → AOF Agent → Response
```

AOF handles:
- Webhook verification (GET request)
- Message parsing (POST request)
- HMAC signature verification
- Interactive buttons/lists
- Response formatting

## Safety

Like Telegram, WhatsApp is configured as **read-only** by default:
- ✅ Read operations work: `kubectl get`, `docker ps`
- ❌ Write operations blocked: `kubectl delete`, `docker rm`

This protects against accidental commands from mobile.

## Troubleshooting

**Webhook not verifying?**
- Verify token must match exactly (case-sensitive)
- Server must be running and accessible
- Check ngrok is forwarding correctly

**Messages not arriving?**
- Ensure **messages** field is subscribed in webhook
- Your phone must be added and verified
- Check server logs for errors

**Invalid signature?**
- App secret must be from **App Settings** → **Basic**
- Check `WHATSAPP_APP_SECRET` env var

## Production Checklist

- [ ] Get permanent access token (not test token)
- [ ] Complete Meta business verification
- [ ] Use proper domain (not ngrok)
- [ ] Set up phone number whitelist
- [ ] Run as systemd service

## Next Steps

- [Full WhatsApp Tutorial](../tutorials/whatsapp-ops-bot.md) - Deep dive
- [WhatsApp Reference](../reference/whatsapp-integration.md) - API details
- [Telegram Quickstart](./quickstart-telegram.md) - Free alternative
