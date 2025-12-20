# WhatsApp Integration

AOF integrates with the WhatsApp Business Cloud API for mobile-first AI agent interactions. Perfect for on-call engineers, field teams, and anyone who needs AI-powered DevOps from their phone.

## Why WhatsApp?

- **3B+ users worldwide** - Your team already uses it
- **Mobile-first** - Purpose-built for on-the-go operations
- **Rich interactions** - Buttons, lists, and media support
- **Business API** - Enterprise-grade with verified accounts
- **End-to-end encryption** - Secure by default

## How It Works

```
User sends message → WhatsApp Business API → AOF Webhook
                                                   ↓
                                            Parse & Route
                                                   ↓
                                            Execute Agent
                                                   ↓
User receives reply ← WhatsApp Business API ← Format Response
```

AOF receives messages via WhatsApp webhooks, routes them to agents, and sends responses back through the Meta Graph API.

## Key Features

### Interactive Messages

WhatsApp supports rich interactive elements:

| Type | Description | Limit |
|------|-------------|-------|
| **Reply Buttons** | Quick action buttons | Max 3 buttons, 20 chars each |
| **List Messages** | Scrollable selection lists | Max 10 items per section |
| **Text Messages** | Standard text with formatting | Max 4096 characters |

### Message Types Supported

- **Text messages** - Natural language queries
- **Button replies** - Tapped button responses
- **List selections** - Selected list items
- **Media messages** - Images, documents (future)

### Security

- **HMAC-SHA256 signature verification** - All webhooks are cryptographically verified
- **Phone number whitelist** - Restrict to specific numbers
- **App secret validation** - Meta app secret verification

## Architecture

### Platform Adapter

The WhatsApp platform adapter implements `TriggerPlatform`:

```
WhatsAppPlatform
├── parse_message()      # Parse incoming webhooks
├── send_response()      # Send text/interactive messages
├── verify_signature()   # HMAC-SHA256 verification
└── verify_webhook()     # Meta webhook verification (GET)
```

### Webhook Flow

1. **Verification (GET)** - Meta verifies webhook URL ownership
2. **Messages (POST)** - Incoming messages with signature
3. **Response** - AOF sends replies via Graph API

### Response Formatting

AOF automatically:
- Adds status emoji (✅ ❌ ⚠️ ℹ️)
- Converts action buttons to WhatsApp reply buttons
- Truncates long text to WhatsApp limits
- Handles button title length limits (20 chars)

## Use Cases

### On-Call DevOps

```
👤 User: pods crashing in prod
🤖 Bot: ⚠️ Found 2 pods in CrashLoopBackOff:

         api-server-abc12 - OOMKilled (3 restarts)
         worker-xyz98 - ImagePullBackOff

         [View Logs] [Describe Pod] [Check Events]
```

### Field Operations

Perfect for:
- Infrastructure monitoring from anywhere
- Quick status checks
- Alert acknowledgment
- Incident response

### Team Notifications

WhatsApp lists work great for:
- Agent/fleet selection
- Environment switching
- Action menus

## Comparison with Telegram

| Feature | WhatsApp | Telegram |
|---------|----------|----------|
| User Base | 3B+ global | 700M+ global |
| Business API | Meta Business Platform | BotFather (free) |
| Interactive | Buttons (3), Lists (10/section) | Inline keyboards (unlimited) |
| Threading | Not supported | Supported |
| Files | Supported | Supported |
| Setup Complexity | Higher (Meta verification) | Lower |
| Cost | Per-conversation pricing | Free |

## Getting Started

1. **Meta Business Account** - Create at [business.facebook.com](https://business.facebook.com)
2. **WhatsApp Business App** - Set up in Meta Developer Console
3. **Phone Number** - Add and verify a phone number
4. **Configure AOF** - Add WhatsApp platform to daemon config
5. **Set Webhook** - Configure webhook URL in Meta console

See the [WhatsApp Quickstart Guide](../guides/quickstart-whatsapp.md) for step-by-step setup.

## Next Steps

- [WhatsApp Quickstart](../guides/quickstart-whatsapp.md) - 10-minute setup guide
- [WhatsApp Tutorial](../tutorials/whatsapp-ops-bot.md) - Build a complete ops bot
- [WhatsApp Reference](../reference/whatsapp-integration.md) - Full API reference
- [Telegram Tutorial](../tutorials/telegram-ops-bot.md) - Alternative mobile platform
