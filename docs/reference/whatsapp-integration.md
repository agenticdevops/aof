# WhatsApp Integration Reference

Complete reference for AOF's WhatsApp Business Cloud API integration.

## Configuration

### DaemonConfig

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

      # Optional: API version (default v18.0)
      api_version: "v18.0"

      # Optional: Phone number whitelist
      allowed_numbers:
        - "14155551234"
        - "14155555678"

  agents:
    directory: "./agents"

  runtime:
    default_agent: "devops"
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `WHATSAPP_PHONE_NUMBER_ID` | Phone Number ID from Meta Business | Yes |
| `WHATSAPP_ACCESS_TOKEN` | Permanent access token | Yes |
| `WHATSAPP_VERIFY_TOKEN` | Custom string for webhook verification | Yes |
| `WHATSAPP_APP_SECRET` | App secret for signature verification | Yes |

### WhatsAppConfig Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `phone_number_id` | string | Yes | - | Meta Phone Number ID |
| `access_token` | string | Yes | - | Graph API access token |
| `verify_token` | string | Yes | - | Webhook verification token |
| `app_secret` | string | Yes | - | HMAC signature secret |
| `business_account_id` | string | No | - | WhatsApp Business Account ID |
| `allowed_numbers` | string[] | No | all | Phone number whitelist |
| `api_version` | string | No | v18.0 | Graph API version |

## Webhook Endpoints

### Verification (GET)

Meta verifies webhook ownership via GET request:

```
GET /webhook/whatsapp?hub.mode=subscribe&hub.verify_token=YOUR_TOKEN&hub.challenge=CHALLENGE
```

AOF returns the challenge if token matches.

### Messages (POST)

```
POST /webhook/whatsapp
Content-Type: application/json
X-Hub-Signature-256: sha256=<signature>

{
  "object": "whatsapp_business_account",
  "entry": [{
    "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
    "changes": [{
      "value": {
        "messaging_product": "whatsapp",
        "metadata": {
          "display_phone_number": "15550000000",
          "phone_number_id": "123456789"
        },
        "contacts": [{
          "profile": { "name": "John Doe" },
          "wa_id": "14155551234"
        }],
        "messages": [{
          "from": "14155551234",
          "id": "wamid.xxx",
          "timestamp": "1234567890",
          "type": "text",
          "text": { "body": "show pods" }
        }]
      },
      "field": "messages"
    }]
  }]
}
```

## Message Types

### Text Messages

Incoming text messages:

```json
{
  "type": "text",
  "text": {
    "body": "show pods in production"
  }
}
```

### Button Replies

When user taps a reply button:

```json
{
  "type": "interactive",
  "interactive": {
    "type": "button_reply",
    "button_reply": {
      "id": "view_logs",
      "title": "View Logs"
    }
  }
}
```

Parsed as: `button:view_logs`

### List Replies

When user selects from a list:

```json
{
  "type": "interactive",
  "interactive": {
    "type": "list_reply",
    "list_reply": {
      "id": "agent_k8s",
      "title": "K8s Agent",
      "description": "Kubernetes operations"
    }
  }
}
```

Parsed as: `list:agent_k8s`

## Response Types

### Text Response

```rust
TriggerResponse {
    text: "Pods are healthy",
    status: ResponseStatus::Success,
    actions: vec![],
}
```

Sent as:
```json
{
  "messaging_product": "whatsapp",
  "recipient_type": "individual",
  "to": "14155551234",
  "type": "text",
  "text": {
    "preview_url": false,
    "body": "✅ Pods are healthy"
  }
}
```

### Interactive Buttons

When response has actions (max 3):

```rust
TriggerResponse {
    text: "Found CrashLoopBackOff",
    status: ResponseStatus::Warning,
    actions: vec![
        Action { id: "view_logs", label: "View Logs" },
        Action { id: "describe", label: "Describe Pod" },
        Action { id: "events", label: "Check Events" },
    ],
}
```

Sent as:
```json
{
  "messaging_product": "whatsapp",
  "recipient_type": "individual",
  "to": "14155551234",
  "type": "interactive",
  "interactive": {
    "type": "button",
    "body": {
      "text": "⚠️ Found CrashLoopBackOff"
    },
    "action": {
      "buttons": [
        { "type": "reply", "reply": { "id": "view_logs", "title": "View Logs" } },
        { "type": "reply", "reply": { "id": "describe", "title": "Describe Pod" } },
        { "type": "reply", "reply": { "id": "events", "title": "Check Events" } }
      ]
    }
  }
}
```

### Interactive Lists

For larger selections, use list messages:

```rust
platform.send_interactive_list(
    "14155551234",
    "Select Agent",           // header
    "Choose an agent to use", // body
    "View Agents",            // button text
    vec![
        ListSection {
            title: "DevOps".to_string(),
            rows: vec![
                ListRow { id: "k8s", title: "K8s Agent", description: Some("Kubernetes ops") },
                ListRow { id: "docker", title: "Docker Agent", description: Some("Container ops") },
            ],
        },
    ],
).await?;
```

## API Methods

### WhatsAppPlatform

```rust
impl WhatsAppPlatform {
    /// Create new WhatsApp platform adapter
    pub fn new(config: WhatsAppConfig) -> Result<Self, PlatformError>;

    /// Verify webhook subscription (GET request handler)
    pub fn verify_webhook(&self, mode: &str, token: &str, challenge: &str) -> Option<String>;

    /// Send text message
    pub async fn send_text_message(&self, to: &str, text: &str) -> Result<String, PlatformError>;

    /// Send interactive buttons (max 3, 20 chars each)
    pub async fn send_interactive_buttons(
        &self,
        to: &str,
        body_text: &str,
        buttons: Vec<(String, String)>,  // (id, title)
    ) -> Result<String, PlatformError>;

    /// Send interactive list
    pub async fn send_interactive_list(
        &self,
        to: &str,
        header: &str,
        body_text: &str,
        button_text: &str,
        sections: Vec<ListSection>,
    ) -> Result<String, PlatformError>;
}
```

### TriggerPlatform Implementation

```rust
#[async_trait]
impl TriggerPlatform for WhatsAppPlatform {
    /// Parse incoming webhook
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError>;

    /// Send response to user
    async fn send_response(
        &self,
        channel: &str,  // Phone number
        response: TriggerResponse,
    ) -> Result<(), PlatformError>;

    /// Verify HMAC-SHA256 signature
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool;

    fn platform_name(&self) -> &'static str { "whatsapp" }
    fn supports_threading(&self) -> bool { false }
    fn supports_interactive(&self) -> bool { true }
    fn supports_files(&self) -> bool { true }
}
```

## Security

### HMAC-SHA256 Verification

All webhooks include signature in `X-Hub-Signature-256` header:

```
sha256=<hex_encoded_hmac>
```

Verification:
```rust
let mut mac = HmacSha256::new_from_slice(app_secret.as_bytes())?;
mac.update(payload);
let computed = hex::encode(mac.finalize().into_bytes());
computed == provided_signature
```

### Phone Number Whitelist

Restrict access to specific phone numbers:

```yaml
platforms:
  whatsapp:
    allowed_numbers:
      - "14155551234"  # Only this number can use the bot
```

Non-whitelisted numbers receive no response (silent fail for security).

## WhatsApp Limits

| Limit | Value |
|-------|-------|
| Text message body | 4096 characters |
| Reply buttons | 3 maximum |
| Button title | 20 characters |
| List sections | 10 maximum |
| Items per section | 10 maximum |
| Item title | 24 characters |
| Item description | 72 characters |
| List button text | 20 characters |

## Error Handling

### PlatformError Types

| Error | Cause |
|-------|-------|
| `InvalidSignature` | HMAC verification failed or phone not allowed |
| `ParseError` | Malformed webhook payload |
| `UnsupportedMessageType` | Non-text/interactive message |
| `ApiError` | Graph API request failed |

### WhatsApp API Errors

Common error codes from Meta:

| Code | Description |
|------|-------------|
| 100 | Invalid parameter |
| 131030 | Recipient not on WhatsApp |
| 131047 | Re-engagement message required |
| 131051 | Message type not supported |
| 368 | Temporarily blocked |

## Trigger Configuration

### Basic WhatsApp Trigger

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: whatsapp-ops
spec:
  platform: whatsapp
  webhook:
    path: /webhook/whatsapp
  routing:
    default_agent: devops
```

### Multi-Agent Routing

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: whatsapp-multi-agent
spec:
  platform: whatsapp
  webhook:
    path: /webhook/whatsapp
  routing:
    rules:
      - match:
          text_contains: ["pod", "deployment", "kubectl"]
        agent: k8s-ops
      - match:
          text_contains: ["docker", "container", "image"]
        agent: docker-ops
    default_agent: devops
```

## Rate Limits

WhatsApp Business API has per-phone-number limits:

| Tier | Messages/day |
|------|--------------|
| Unverified | 250 |
| Verified (Tier 1) | 1,000 |
| Verified (Tier 2) | 10,000 |
| Verified (Tier 3) | 100,000 |
| Verified (Tier 4) | Unlimited |

## Meta Setup

### Prerequisites

1. **Meta Business Account** at [business.facebook.com](https://business.facebook.com)
2. **Meta Developer Account** at [developers.facebook.com](https://developers.facebook.com)
3. **WhatsApp Business App** created in Developer Console
4. **Phone Number** added to WhatsApp Business Account

### Webhook Configuration

In Meta Developer Console:

1. Go to **WhatsApp** → **Configuration**
2. Set **Webhook URL**: `https://your-domain.com/webhook/whatsapp`
3. Set **Verify Token**: Same as `WHATSAPP_VERIFY_TOKEN`
4. Subscribe to **messages** field

### Access Token

Generate permanent token:

1. Go to **WhatsApp** → **API Setup**
2. Under **Permanent token**, click **Generate**
3. Copy and save as `WHATSAPP_ACCESS_TOKEN`

## Comparison: Test vs Production

| Aspect | Test Mode | Production |
|--------|-----------|------------|
| Phone numbers | Limited (5) | Unlimited |
| Verification | Not required | Business verification required |
| Template messages | Not required | Required for initiating |
| Rate limits | 250/day | Tiered (Up to unlimited) |
| Cost | Free | Per-conversation pricing |

## See Also

- [WhatsApp Concepts](../concepts/whatsapp-integration.md) - Overview and use cases
- [WhatsApp Tutorial](../tutorials/whatsapp-ops-bot.md) - Step-by-step guide
- [WhatsApp Quickstart](../guides/quickstart-whatsapp.md) - 5-minute setup
- [Meta WhatsApp Docs](https://developers.facebook.com/docs/whatsapp/cloud-api) - Official API docs
