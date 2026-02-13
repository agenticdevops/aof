# Phase 3: Messaging Gateway - Research

**Researched:** 2026-02-13
**Domain:** Hub-and-spoke messaging gateway, channel adapters, multi-platform bot integration, rate limiting
**Confidence:** HIGH

## Executive Summary

Phase 3 implements a hub-and-spoke messaging gateway that routes human messages from Slack, Discord, Telegram, and WhatsApp to the agent runtime in real-time. The gateway acts as a central control plane with platform-specific channel adapters that normalize message format, threading, and rich media differences into a standard `CoordinationEvent` format. The recommended approach follows OpenClaw's proven hub-and-spoke architecture: a single Gateway owns all messaging channels and communicates with agents via the existing WebSocket/broadcast channel infrastructure from Phase 1.

**Primary recommendation:** Create `aof-gateway` crate with a hub-and-spoke control plane using platform-specific adapters (`slack-morphism` for Slack, `serenity` for Discord, `teloxide` for Telegram). Use NAT-transparent outbound WebSocket connections (Slack Socket Mode, Discord Gateway WebSocket) to eliminate ngrok dependency. Implement per-platform rate limiting with the `governor` crate (GCRA algorithm, async-ready). Normalize all platform messages to `CoordinationEvent`, then route to agent runtime via existing tokio::broadcast channel.

**Key insight from OpenClaw:** The hub-and-spoke model with a single Gateway owning all messaging surfaces (WhatsApp, Telegram, Slack, Discord, Signal, iMessage) provides a clean separation between messaging channels and agent execution, enabling multi-channel access while maintaining security boundaries and persistent sessions.

## Architecture Pattern: Hub-and-Spoke Gateway

### Overview

The hub-and-spoke pattern uses a central control plane (Gateway) with platform-specific adapters (spokes) that translate platform quirks into a standard message format. This pattern is proven in enterprise integration and recently validated by OpenClaw's architecture.

### ASCII Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AOF MESSAGING GATEWAY                          │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    GATEWAY HUB (Control Plane)               │   │
│  │  - Message routing                                           │   │
│  │  - Event translation (Platform → CoordinationEvent)          │   │
│  │  - Rate limiting (per-platform token buckets)                │   │
│  │  - Squad broadcast (one-to-many)                             │   │
│  │  - WebSocket connection to agent runtime                     │   │
│  └──────────┬──────────────┬──────────────┬──────────────┬──────┘   │
│             │              │              │              │           │
│  ┌──────────▼─────┐  ┌────▼────┐  ┌──────▼──────┐  ┌───▼──────┐   │
│  │ Slack Adapter  │  │ Discord │  │ Telegram    │  │ WhatsApp │   │
│  │ (morphism)     │  │ (serenity)│ │ (teloxide)  │  │ (whatsapp│   │
│  │                │  │          │  │             │  │ -rust)   │   │
│  │ - Socket Mode  │  │ - Gateway│  │ - Long poll │  │ - Web API│   │
│  │ - Threads      │  │ - Embeds │  │ - Inline KB │  │ - Media  │   │
│  │ - Blocks       │  │ - Threads│  │ - Markdown  │  │          │   │
│  └────────┬───────┘  └────┬─────┘  └──────┬──────┘  └────┬─────┘   │
│           │               │               │              │          │
└───────────┼───────────────┼───────────────┼──────────────┼──────────┘
            │               │               │              │
            ▼               ▼               ▼              ▼
    ┌───────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
    │  Slack    │   │ Discord  │   │ Telegram │   │ WhatsApp │
    │  API      │   │  API     │   │   API    │   │   Web    │
    └───────────┘   └──────────┘   └──────────┘   └──────────┘
            │               │               │              │
            ▼               ▼               ▼              ▼
    NAT-TRANSPARENT (outbound WebSocket/polling, no ngrok needed)

            ┌───────────────────────────────────────┐
            │  Agent Runtime (Phase 1 Infrastructure)│
            │  - tokio::broadcast event bus          │
            │  - AgentExecutor                       │
            │  - Memory backends                     │
            └───────────────────────────────────────┘
```

### Pattern Benefits

1. **Linear scaling:** Adding 51st platform requires only 1 new adapter, not 50 integrations
2. **Normalization point:** Platform quirks isolated in adapters, core logic platform-agnostic
3. **Bidirectional bridge:** Gateway translates both inbound (user → agent) and outbound (agent → user)
4. **NAT-transparent:** Outbound connections eliminate need for public endpoints or ngrok
5. **Decoupling:** Messaging changes don't affect agent runtime, vice versa

### References

- [Hub and Spoke Pattern - Enterprise Integration Patterns](https://www.enterpriseintegrationpatterns.com/ramblings/03_hubandspoke.html)
- [OpenClaw Architecture Explained](https://ppaolo.substack.com/p/openclaw-system-architecture-overview)
- [Gateway Architecture - OpenClaw](https://docs.openclaw.ai/concepts/architecture)
- [OpenClaw GitHub - Hub-and-Spoke Implementation](https://github.com/openclaw/openclaw)

## Channel Adapters: Platform-Specific Crates

### Comparison Table

| Platform | Rust Crate | Version | Connection Type | Threading Support | Rate Limit | Rich Format | Maturity |
|----------|------------|---------|-----------------|-------------------|------------|-------------|----------|
| **Slack** | `slack-morphism` | 2.0+ | Socket Mode (WSS) | ✅ thread_ts | 1 msg/sec (Tier 1) | Block Kit | Production-ready |
| **Discord** | `serenity` | 0.12+ | Gateway (WSS) | ✅ Threads API | 10 req/sec global | Embeds | Production-ready |
| **Telegram** | `teloxide` | 0.13+ | Long polling / Webhook | ❌ Reply-to only | 30 msg/sec | Inline keyboards, Markdown | Production-ready |
| **WhatsApp** | `whatsapp-rust` | 0.1+ | Web API (unofficial) | ❌ Limited | Unknown | Media, buttons | Experimental ⚠️ |

### Slack Adapter: `slack-morphism`

**Crate:** [slack-morphism](https://github.com/abdolence/slack-morphism-rust) v2.0+

**Why recommended:**
- Modern async client with Slack Web/Events API and Socket Mode support
- Handles HMAC-SHA256 signature verification automatically
- Block Kit builder for rich formatting
- Comprehensive documentation and active maintenance

**Connection approach:**
```rust
// Socket Mode - NAT-transparent (outbound WebSocket)
use slack_morphism::prelude::*;
use slack_morphism_hyper::*;

let client = SlackClient::new(SlackClientHyperConnector::new());
let socket_mode_client = SlackClientSocketModeConfig::new()
    .app_token(&app_token)
    .build();

// Subscribe to events (messages, reactions, slash commands)
socket_mode_client.listen_for_events(|event| {
    // Translate to CoordinationEvent
    gateway.route_to_agent(normalize_slack_event(event))
}).await?;
```

**Threading normalization:**
- Slack uses `thread_ts` (message timestamp as thread ID)
- Map to `CoordinationEvent.thread_id: Option<String>`
- Preserve parent message context in agent prompt

**Rate limiting:**
- Tier 1 apps: 1 request/sec (60 req/min)
- Tier 2 apps: Higher limits after review
- Implement token bucket with 1 req/sec refill rate

**Gotchas:**
- Socket Mode requires App-level token (starts with `xapp-`)
- Bot user ID must be detected to ignore own reactions (approval workflow)
- Stale message filtering needed (drop messages >5 min old from queue)

**References:**
- [slack-morphism Documentation](https://docs.rs/slack-morphism/latest/slack_morphism/)
- [Slack Rate Limits](https://api.slack.com/docs/rate-limits)
- [Slack Socket Mode](https://api.slack.com/apis/connections/socket)

### Discord Adapter: `serenity`

**Crate:** [serenity](https://github.com/serenity-rs/serenity) v0.12+

**Why recommended:**
- Mature Discord API wrapper with Gateway WebSocket support
- Transparent shard management (auto-scales for large bots)
- Built-in event handlers (message_create, interaction_create)
- Companion crates for slash commands (poise) and voice (songbird)

**Connection approach:**
```rust
// Gateway WebSocket - NAT-transparent (outbound connection)
use serenity::prelude::*;
use serenity::model::prelude::*;

let mut client = Client::builder(&token, GatewayIntents::GUILD_MESSAGES)
    .event_handler(Handler)
    .await?;

// Event handler translates Discord events to CoordinationEvent
struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Normalize to CoordinationEvent
        gateway.route_message(normalize_discord_message(msg)).await;
    }
}
```

**Threading normalization:**
- Discord threads are actual channels (separate channel_id)
- Thread creation emits `ThreadCreate` event
- Map to `CoordinationEvent.thread_id` with thread metadata

**Embed normalization:**
- Discord embeds have structured fields (title, description, fields, footer)
- Convert to markdown for agent consumption
- When responding, translate markdown back to embed structure

**Rate limiting:**
- Global: 50 requests/sec per bot
- Per-route: Varies (indicated by `X-RateLimit-Bucket` header)
- Discord returns 429 with `Retry-After` header
- Implement token bucket with route-specific buckets

**Gotchas:**
- Ed25519 signature verification required for interactions (not gateway events)
- Embed total character limit: 6,000 across all text fields
- Threads auto-archive after 3 days (free plan), 7 days (premium)

**References:**
- [Serenity Documentation](https://docs.rs/serenity/latest/serenity/)
- [Discord Rate Limits](https://docs.discord.com/developers/topics/rate-limits)
- [Building Rust Discord Bot with Serenity](https://blog.logrocket.com/building-rust-discord-bot-shuttle-serenity/)

### Telegram Adapter: `teloxide`

**Crate:** [teloxide](https://github.com/teloxide/teloxide) v0.13+

**Why recommended:**
- Elegant async bot framework with dptree functional pipeline
- Supports both long polling (NAT-friendly) and webhooks
- Inline keyboard, command parsing, conversation state management
- Comprehensive examples and active development

**Connection approach:**
```rust
// Long polling - NAT-transparent (outbound HTTP polling)
use teloxide::prelude::*;

let bot = Bot::from_env();

teloxide::repl(bot, |bot: Bot, msg: Message| async move {
    // Normalize to CoordinationEvent
    gateway.route_telegram_message(normalize_telegram(msg)).await;
    Ok(())
}).await;
```

**Threading normalization:**
- Telegram doesn't have native threads, uses `reply_to_message_id`
- Map reply chains to thread context (not as robust as Slack/Discord)
- Consider thread context limited to parent message only

**Rate limiting:**
- 30 messages/sec to the same chat
- 20 messages/min to different chats
- Implement per-chat token bucket (30 msg/sec refill)

**Gotchas:**
- Long polling blocks a connection, may need timeout tuning
- Markdown parsing strict (use `ParseMode::MarkdownV2`)
- File uploads require separate API calls (not inline)

**References:**
- [teloxide Documentation](https://github.com/teloxide/teloxide)
- [Telegram Bot API Rate Limits](https://core.telegram.org/bots/faq#my-bot-is-hitting-limits-how-do-i-avoid-this)

### WhatsApp Adapter: `whatsapp-rust` (Experimental)

**Crate:** [whatsapp-rust](https://github.com/jlucaso1/whatsapp-rust) v0.1+ (unofficial)

**Why experimental:**
- Unofficial implementation (violates Meta ToS, risk of account suspension)
- No official WhatsApp Bot API for Rust
- Official WhatsApp Business Cloud API exists but requires business account

**Recommendation:**
- **For production:** Use official WhatsApp Business Cloud API via HTTP client
- **For development/testing:** `whatsapp-rust` with clear ToS warnings
- **Alternative:** whatsapp-cloud-api crate for official API

**Connection approach (unofficial):**
```rust
// whatsapp-rust uses WhatsApp Web protocol (reverse-engineered)
use whatsapp_rust::Client;

let client = Client::new().await?;
client.authenticate_with_qr().await?;

client.on_message(|msg| {
    // Normalize to CoordinationEvent
    gateway.route_whatsapp_message(normalize_whatsapp(msg)).await;
});
```

**Official API approach:**
```toml
whatsapp-cloud-api = "0.1"
```

**Rate limiting:**
- Official API: 1000 messages per 24 hours (free tier)
- Unofficial: Unknown, likely subject to WhatsApp's anti-spam detection

**Gotchas:**
- Unofficial implementations may break without warning (protocol changes)
- QR code authentication expires, requires re-scan
- Official API requires business verification (slow process)

**Recommendation for Phase 3:** Defer WhatsApp support or use official Cloud API only (avoid ToS risk).

**References:**
- [whatsapp-rust GitHub](https://github.com/jlucaso1/whatsapp-rust)
- [WhatsApp Business Cloud API](https://developers.facebook.com/docs/whatsapp/cloud-api)
- [Rust at Scale: WhatsApp Security](https://engineering.fb.com/2026/01/27/security/rust-at-scale-security-whatsapp/)

## NAT-Transparent Implementation: Outbound WebSocket Pattern

### Why NAT-Transparent Matters

Traditional webhook-based bots require:
1. Public HTTP endpoint
2. Reverse proxy (ngrok, rathole) or port forwarding
3. SSL certificate management
4. Firewall configuration

**NAT-transparent approach:** Bots initiate outbound connections to platform APIs (WebSocket or long polling), eliminating need for public endpoints.

### Platform Support Matrix

| Platform | NAT-Transparent Method | Fallback (if needed) |
|----------|------------------------|----------------------|
| Slack | ✅ Socket Mode (outbound WSS) | Events API (webhook) |
| Discord | ✅ Gateway (outbound WSS) | None required |
| Telegram | ✅ Long polling (outbound HTTP) | Webhook (optional) |
| WhatsApp | ❌ Unofficial (Web protocol) | Business Cloud API webhook |

### Implementation Pattern (Slack Socket Mode Example)

```rust
use slack_morphism::prelude::*;

// Socket Mode client initiates outbound WebSocket connection
let socket_config = SlackClientSocketModeConfig::new()
    .app_token(&config.app_token) // xapp-1-...
    .build();

// Listen for events (connection is outbound, no public endpoint needed)
socket_config.listen_for_events(|event| async move {
    match event {
        SlackSocketModeEvent::EventsApi(events_api) => {
            // Translate to CoordinationEvent
            let coord_event = normalize_slack_event(events_api)?;
            gateway.broadcast(coord_event).await?;
        }
        SlackSocketModeEvent::SlashCommand(cmd) => {
            // Handle slash command
            let coord_event = normalize_slash_command(cmd)?;
            gateway.broadcast(coord_event).await?;
        }
        _ => {}
    }
    Ok(())
}).await?;
```

### Security Considerations

**Outbound WebSocket benefits:**
- No public attack surface (no inbound connections)
- Credential exposure limited to outbound TLS connections
- No firewall/NAT configuration required

**Credential management:**
- Store bot tokens in environment variables (12-factor)
- Use `aofctl serve` YAML config with `${ENV_VAR}` substitution
- Never commit tokens to version control

**Message interception risk:**
- TLS/WSS encrypts all platform communication
- HMAC signature verification for platforms that support it (Slack, Discord interactions)

### References

- [Connectivity to Slack without Ngrok](https://forum.rasa.com/t/connectivity-to-slack-without-using-ngrok/10346)
- [NAT Traversal Alternatives](https://github.com/anderspitman/awesome-tunneling)
- [Slack Socket Mode Documentation](https://api.slack.com/apis/connections/socket)

## Event Translation: Platform → CoordinationEvent Mapping

### Standard Message Schema

All platforms normalize to this structure before routing to agents:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    /// Unique message ID (platform-specific)
    pub message_id: String,
    /// Platform source (slack, discord, telegram, whatsapp)
    pub platform: Platform,
    /// Channel/chat ID
    pub channel_id: String,
    /// Thread ID (if threaded)
    pub thread_id: Option<String>,
    /// User who sent message
    pub user: MessageUser,
    /// Message content (normalized to markdown)
    pub content: String,
    /// Attachments (images, files)
    pub attachments: Vec<Attachment>,
    /// Platform-specific metadata (stored as JSON)
    pub metadata: serde_json::Value,
    /// When message was sent
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUser {
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Slack,
    Discord,
    Telegram,
    WhatsApp,
}
```

### Platform-Specific Translations

#### Slack → InboundMessage

```rust
fn normalize_slack_event(event: SlackEventMessage) -> InboundMessage {
    InboundMessage {
        message_id: event.ts.clone(),
        platform: Platform::Slack,
        channel_id: event.channel,
        thread_id: event.thread_ts, // Slack threading
        user: MessageUser {
            user_id: event.user,
            username: fetch_slack_username(&event.user), // API call or cache
            display_name: None,
        },
        content: slack_blocks_to_markdown(event.blocks), // Convert Block Kit
        attachments: extract_slack_files(event.files),
        metadata: json!({"workspace_id": event.team_id}),
        timestamp: parse_slack_timestamp(&event.ts),
    }
}
```

#### Discord → InboundMessage

```rust
fn normalize_discord_message(msg: serenity::model::channel::Message) -> InboundMessage {
    InboundMessage {
        message_id: msg.id.to_string(),
        platform: Platform::Discord,
        channel_id: msg.channel_id.to_string(),
        thread_id: if msg.is_thread() { Some(msg.channel_id.to_string()) } else { None },
        user: MessageUser {
            user_id: msg.author.id.to_string(),
            username: msg.author.name.clone(),
            display_name: msg.author.global_name.clone(),
        },
        content: msg.content.clone(), // Discord uses markdown natively
        attachments: extract_discord_attachments(msg.attachments),
        metadata: json!({"guild_id": msg.guild_id}),
        timestamp: msg.timestamp.into(),
    }
}
```

#### Telegram → InboundMessage

```rust
fn normalize_telegram_message(msg: teloxide::types::Message) -> InboundMessage {
    InboundMessage {
        message_id: msg.id.to_string(),
        platform: Platform::Telegram,
        channel_id: msg.chat.id.to_string(),
        thread_id: msg.reply_to_message().map(|m| m.id.to_string()), // Reply chain
        user: MessageUser {
            user_id: msg.from().map(|u| u.id.to_string()).unwrap_or_default(),
            username: msg.from().and_then(|u| u.username.clone()).unwrap_or_default(),
            display_name: msg.from().map(|u| format!("{} {}", u.first_name, u.last_name.unwrap_or_default())),
        },
        content: msg.text().unwrap_or("").to_string(),
        attachments: extract_telegram_media(msg),
        metadata: json!({"chat_type": msg.chat.kind}),
        timestamp: Utc.timestamp_opt(msg.date.unix_timestamp(), 0).unwrap(),
    }
}
```

### Rich Format Normalization

**Challenge:** Each platform has different rich formatting:
- Slack: Block Kit (JSON structure)
- Discord: Embeds (structured fields)
- Telegram: Markdown/HTML
- WhatsApp: Plain text + media

**Strategy:**
1. **Inbound (user → agent):** Normalize all to markdown for LLM consumption
2. **Outbound (agent → user):** Detect target platform, translate markdown to native format

**Markdown as Lingua Franca:**
```rust
// Inbound: Slack Block Kit → Markdown
fn slack_blocks_to_markdown(blocks: Vec<SlackBlock>) -> String {
    blocks.iter().map(|block| match block {
        SlackBlock::Section { text, .. } => text.as_markdown(),
        SlackBlock::Divider => "---",
        // ... handle all block types
    }).join("\n\n")
}

// Outbound: Markdown → Slack Block Kit
fn markdown_to_slack_blocks(markdown: &str) -> Vec<SlackBlock> {
    // Parse markdown, convert to Slack blocks
    // Headings → Section with bold text
    // Lists → Section with mrkdwn
    // Code blocks → Section with code formatting
}
```

### Bidirectional Bridge: Agent Responses → Platform API

```rust
// Agent emits response event
pub struct AgentResponse {
    pub agent_id: String,
    pub content: String, // Markdown
    pub target_platform: Platform,
    pub target_channel: String,
    pub thread_id: Option<String>,
}

// Gateway translates and sends
async fn send_agent_response(response: AgentResponse) {
    match response.target_platform {
        Platform::Slack => {
            let blocks = markdown_to_slack_blocks(&response.content);
            slack_client.post_message(PostMessageRequest {
                channel: response.target_channel,
                thread_ts: response.thread_id,
                blocks,
                ..Default::default()
            }).await?;
        }
        Platform::Discord => {
            let embed = markdown_to_discord_embed(&response.content);
            discord_client.send_message(response.target_channel, |m| {
                m.embed(|e| embed)
            }).await?;
        }
        // ... other platforms
    }
}
```

### References

- [Channel Adapter Pattern - Enterprise Integration Patterns](https://www.enterpriseintegrationpatterns.com/patterns/messaging/ChannelAdapter.html)
- [Message Translator Pattern](https://www.enterpriseintegrationpatterns.com/patterns/messaging/MessageTranslator.html)

## Rate Limiting: Token Bucket Implementation

### Why Token Bucket (GCRA)

Token bucket (specifically Generic Cell Rate Algorithm) is the gold standard for rate limiting:
- **Smooth rate limiting:** No thundering herd when limit resets
- **Burst allowance:** Can consume tokens up to bucket capacity, then refills at constant rate
- **Fairness:** Prevents single client from monopolizing quota
- **Async-ready:** Futures resolve when tokens available

**Alternatives considered:**
- Fixed window: Thundering herd at reset time, bursty traffic
- Sliding window: More complex, similar benefits to token bucket
- Leaky bucket: Requires background drip process, token bucket equivalent without overhead

### Recommended Crate: `governor`

**Crate:** [governor](https://github.com/boinkor-net/governor) v0.6+

**Why recommended:**
- Production-ready, used in high-throughput systems
- GCRA implementation (leaky bucket without background process)
- Async-first: `until_ready()` returns future that resolves when tokens available
- Thread-safe: 64-bit atomic compare-and-swap, no locks
- Jitter support: `until_ready_with_jitter()` reduces thundering herd

**Installation:**
```toml
governor = "0.6"
tokio = { version = "1", features = ["time", "rt"] }
```

### Per-Platform Rate Limiter Configuration

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Slack: 1 request/sec (Tier 1)
let slack_quota = Quota::per_second(NonZeroU32::new(1).unwrap());
let slack_limiter = RateLimiter::direct(slack_quota);

// Discord: 10 requests/sec (global)
let discord_quota = Quota::per_second(NonZeroU32::new(10).unwrap());
let discord_limiter = RateLimiter::direct(discord_quota);

// Telegram: 30 messages/sec (per chat)
let telegram_quota = Quota::per_second(NonZeroU32::new(30).unwrap());
let telegram_limiter = RateLimiter::keyed(telegram_quota); // Per-chat keying

// WhatsApp: 1000 messages/24hr (Cloud API)
let whatsapp_quota = Quota::per_day(NonZeroU32::new(1000).unwrap());
let whatsapp_limiter = RateLimiter::direct(whatsapp_quota);
```

### Async Usage in Gateway

```rust
use governor::clock::DefaultClock;

async fn send_slack_message(
    limiter: &RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    message: SlackMessage,
) -> Result<()> {
    // Wait until rate limiter allows (async, non-blocking)
    limiter.until_ready().await;

    // Now send message
    slack_client.post_message(message).await?;
    Ok(())
}
```

### Backoff Strategy for 429 Errors

When platform returns 429 (rate limit exceeded):

```rust
async fn send_with_retry(
    limiter: &RateLimiter,
    message: Message,
) -> Result<()> {
    loop {
        // Wait for token
        limiter.until_ready().await;

        match platform_client.send(message.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if e.status_code() == 429 => {
                // Extract Retry-After header (Discord, Slack return this)
                let retry_after = e.retry_after_seconds().unwrap_or(60);
                warn!("Rate limited, retrying after {}s", retry_after);
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

### Jitter for Thundering Herd Prevention

```rust
use governor::Jitter;

// Add jitter to reduce simultaneous retries
limiter.until_ready_with_jitter(Jitter::up_to(Duration::from_millis(100))).await;
```

### Per-Route Rate Limiting (Discord)

Discord has per-route rate limits (indicated by `X-RateLimit-Bucket` header). Use keyed rate limiters:

```rust
use governor::RateLimiter;
use std::sync::Arc;
use dashmap::DashMap;

// Map bucket ID → rate limiter
let route_limiters: Arc<DashMap<String, RateLimiter>> = Arc::new(DashMap::new());

async fn send_discord_request(
    route_limiters: &DashMap<String, RateLimiter>,
    bucket_id: &str,
    request: DiscordRequest,
) -> Result<()> {
    // Get or create rate limiter for this bucket
    let limiter = route_limiters.entry(bucket_id.to_string())
        .or_insert_with(|| {
            let quota = Quota::per_second(NonZeroU32::new(5).unwrap()); // Default
            RateLimiter::direct(quota)
        });

    limiter.until_ready().await;
    discord_client.send(request).await
}
```

### References

- [governor Crate Documentation](https://docs.rs/governor/latest/governor/)
- [GCRA Algorithm Explanation](https://github.com/boinkor-net/governor#algorithm)
- [Implementing API Rate Limiting in Rust](https://www.shuttle.dev/blog/2024/02/22/api-rate-limiting-rust)
- [How to Implement Rate Limiting in Rust Without External Services](https://oneuptime.com/blog/post/2026-01-07-rust-rate-limiting/view)

## Configuration Strategy: Gateway YAML

### Recommended Structure

```yaml
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: messaging-gateway
spec:
  # WebSocket connection to agent runtime (Phase 1 infrastructure)
  runtime:
    websocket_url: "ws://localhost:8080/ws"
    session_id: "${SESSION_ID}" # Generated or from env

  # Platform adapters
  adapters:
    - platform: slack
      enabled: true
      config:
        # Bot tokens from environment (never hardcoded)
        bot_token: "${SLACK_BOT_TOKEN}" # xoxb-...
        app_token: "${SLACK_APP_TOKEN}" # xapp-1-... (Socket Mode)
        signing_secret: "${SLACK_SIGNING_SECRET}"
        bot_user_id: "${SLACK_BOT_USER_ID}" # For reaction filtering

        # Optional: Channel filtering
        allowed_channels:
          - "C01234567" # #ops-team
          - "C89012345" # #incidents

        # Optional: Approval whitelist
        approval_allowed_users:
          - "U12345678" # @alice
          - "U87654321" # @bob

        # Rate limiting
        rate_limit:
          requests_per_second: 1
          burst_size: 5

    - platform: discord
      enabled: true
      config:
        bot_token: "${DISCORD_BOT_TOKEN}"
        application_id: "${DISCORD_APP_ID}"
        public_key: "${DISCORD_PUBLIC_KEY}" # For signature verification

        # Optional: Guild filtering
        guild_ids:
          - "123456789012345678"

        # Optional: Role-based access
        allowed_roles:
          - "987654321098765432" # @ops-team

        rate_limit:
          requests_per_second: 10
          per_route: true # Enable per-route bucketing

    - platform: telegram
      enabled: true
      config:
        bot_token: "${TELEGRAM_BOT_TOKEN}"

        # Connection mode
        connection_mode: long_polling # or webhook
        webhook_url: "https://example.com/telegram" # If webhook mode

        rate_limit:
          messages_per_second: 30
          per_chat: true # Separate limiter per chat

    - platform: whatsapp
      enabled: false # Defer to future phase
      config:
        # Official Cloud API
        access_token: "${WHATSAPP_ACCESS_TOKEN}"
        phone_number_id: "${WHATSAPP_PHONE_NUMBER_ID}"

        rate_limit:
          messages_per_day: 1000

  # Squad announcement routing
  squads:
    - name: ops-team
      description: "Operations team agents"
      agents:
        - "k8s-monitor"
        - "incident-responder"
        - "log-analyzer"

      # Platform mappings
      channels:
        slack: "C01234567" # #ops-team
        discord: "987654321098765432" # ops-team channel
        telegram: "-1001234567890" # ops-team group

    - name: dev-team
      description: "Development team agents"
      agents:
        - "code-reviewer"
        - "ci-cd-manager"
      channels:
        slack: "C98765432"
        discord: "123456789012345678"
```

### Secrets Management

**Environment variable substitution:**
```rust
use std::env;

fn resolve_env_vars(config_str: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Z_]+)\}").unwrap();
    re.replace_all(config_str, |caps: &regex::Captures| {
        let var_name = &caps[1];
        env::var(var_name).unwrap_or_else(|_| {
            warn!("Environment variable {} not set", var_name);
            String::new()
        })
    }).to_string()
}
```

**Reading from .env file (development):**
```toml
# Cargo.toml
dotenv = "0.15"
```

```rust
// In main()
dotenv::dotenv().ok(); // Load .env file
```

**Production deployment:**
- Use Kubernetes Secrets or Docker secrets
- Never commit `.env` to version control
- Use secret management (HashiCorp Vault, AWS Secrets Manager)

### Hot-Reload Capability (Future Enhancement)

**Current scope:** Daemon restart required for config changes

**Future enhancement (not Phase 3):**
- Watch config file with `notify` crate
- Reload adapters on file change without dropping connections
- Graceful shutdown of old adapters, start new ones

### Multi-Workspace Support

**Challenge:** Single organization may have multiple Slack workspaces, Discord servers, etc.

**Solution:** Array of adapter configs per platform
```yaml
adapters:
  - platform: slack
    name: workspace-main
    config:
      bot_token: "${SLACK_BOT_TOKEN_MAIN}"
      # ...

  - platform: slack
    name: workspace-staging
    config:
      bot_token: "${SLACK_BOT_TOKEN_STAGING}"
      # ...
```

Each adapter instance runs independently with separate rate limiters.

## Squad Announcements: Broadcast Pattern

### Use Cases

1. **All-hands broadcast:** "Deploy starting in 5 minutes" → all agents in all channels
2. **Team-specific:** "Incident SEV1 detected" → ops-team agents only
3. **Channel-specific:** Slack #incidents → only agents monitoring that channel

### Broadcast Event Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// Message content (markdown)
    pub content: String,
    /// Target audience
    pub target: BroadcastTarget,
    /// Priority (affects notification style)
    pub priority: Priority,
    /// Originating platform (optional, for reply-to)
    pub source_platform: Option<Platform>,
    pub source_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BroadcastTarget {
    /// All agents in all channels
    AllAgents,
    /// Specific squad (from config)
    Squad(String),
    /// Specific agents by ID
    Agents(Vec<String>),
    /// All agents in specific platform channel
    Channel { platform: Platform, channel_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}
```

### Implementation in Gateway

```rust
async fn broadcast_to_squad(
    gateway: &Gateway,
    message: BroadcastMessage,
) -> Result<()> {
    // Resolve target agents
    let agents = match message.target {
        BroadcastTarget::AllAgents => gateway.get_all_agents(),
        BroadcastTarget::Squad(name) => gateway.get_squad_agents(&name)?,
        BroadcastTarget::Agents(ids) => ids,
        BroadcastTarget::Channel { platform, channel_id } => {
            // Get agents subscribed to this channel
            gateway.get_agents_for_channel(platform, &channel_id)
        }
    };

    // Send to each platform channel
    for agent in agents {
        let channels = gateway.get_agent_channels(&agent)?;

        for (platform, channel_id) in channels {
            // Apply rate limiting per platform
            let limiter = gateway.get_rate_limiter(platform);
            limiter.until_ready().await;

            // Send message
            match platform {
                Platform::Slack => {
                    slack_client.post_message(channel_id, &message.content).await?;
                }
                Platform::Discord => {
                    discord_client.send_message(channel_id, &message.content).await?;
                }
                // ... other platforms
            }
        }
    }

    Ok(())
}
```

### Filtering and Acknowledgment

**Challenge:** How do agents know broadcast is for them?

**Pattern 1: Mention-based filtering**
- Broadcast includes @mentions: "@k8s-monitor @incident-responder"
- Agents filter based on their configured username/ID

**Pattern 2: Tag-based filtering**
- Message includes tags: `[ops-team] [sev1]`
- Agents subscribe to tags, filter in runtime

**Pattern 3: Event bus subscription**
- Agents subscribe to specific event types on event bus
- Gateway publishes broadcast as typed event

**Acknowledgment (future enhancement):**
- Agents respond with thumbs-up reaction
- Gateway tracks acks, escalates if not all agents respond within timeout

## Known Gotchas & Mitigations

### 1. Slack: Stale Message Filtering

**Problem:** Slack Events API may deliver messages out of order or with delay. Bot may respond to 5-minute-old message.

**Mitigation:**
```rust
const MAX_MESSAGE_AGE_SECS: i64 = 300; // 5 minutes

fn is_message_stale(slack_ts: &str) -> bool {
    let msg_time = parse_slack_timestamp(slack_ts);
    let age = Utc::now().signed_duration_since(msg_time);
    age.num_seconds() > MAX_MESSAGE_AGE_SECS
}

// In event handler
if is_message_stale(&event.ts) {
    warn!("Dropping stale message: {}", event.ts);
    return Ok(());
}
```

### 2. Discord: Embed Character Limits

**Problem:** Discord embeds have total 6,000 character limit across all fields. Agent response may exceed this.

**Mitigation:**
```rust
fn split_long_response(content: &str, max_len: usize) -> Vec<String> {
    // Split at sentence boundaries, not mid-word
    content.split(". ")
        .fold(Vec::new(), |mut chunks, sentence| {
            if let Some(last) = chunks.last_mut() {
                if last.len() + sentence.len() < max_len {
                    last.push_str(sentence);
                    last.push_str(". ");
                } else {
                    chunks.push(sentence.to_string());
                }
            } else {
                chunks.push(sentence.to_string());
            }
            chunks
        })
}

// Send multiple messages if needed
let chunks = split_long_response(&agent_response, 5500); // Leave buffer
for chunk in chunks {
    send_discord_message(channel_id, chunk).await?;
}
```

### 3. Telegram: Markdown Parsing Strictness

**Problem:** Telegram's MarkdownV2 is strict (requires escaping `_`, `*`, `[`, `]`, `(`, `)`, `~`, `` ` ``, `>`, `#`, `+`, `-`, `=`, `|`, `{`, `}`, `.`, `!`).

**Mitigation:**
```rust
fn escape_telegram_markdown(text: &str) -> String {
    let special_chars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#',
                         '+', '-', '=', '|', '{', '}', '.', '!'];
    let mut result = text.to_string();
    for c in special_chars {
        result = result.replace(c, &format!("\\{}", c));
    }
    result
}
```

**Alternative:** Use plain text mode (no formatting) to avoid parsing errors.

### 4. WhatsApp: ToS Violation Risk

**Problem:** Unofficial APIs violate Meta's Terms of Service, risk account suspension.

**Mitigation:**
- Use official WhatsApp Business Cloud API (requires business verification)
- Clearly document ToS risks if using unofficial API
- Defer WhatsApp support until official Rust SDK available

### 5. Rate Limiting: Token Exhaustion

**Problem:** High message volume exhausts rate limit tokens, messages queue up.

**Mitigation:**
- Implement backpressure: Return 429 to agents if gateway queue full
- Priority queuing: Urgent messages skip queue
- Adaptive rate limiting: Reduce agent activity when rate limit approached

```rust
if rate_limiter.check().is_err() {
    warn!("Rate limit exhausted, queuing message");
    message_queue.push(message);

    // Notify agent runtime to slow down
    gateway.emit_backpressure_event().await;
}
```

### 6. Threading Context Loss

**Problem:** Platforms differ in threading semantics. Telegram has weak threading, Slack/Discord strong.

**Mitigation:**
- Store thread context in agent memory (Phase 1 persistence)
- Include parent message summary in agent prompt
- For Telegram, use reply chains + manual context tracking

### 7. Bot Self-Reaction Loop

**Problem:** Bot reacts to approval message, then reacts to its own reaction (infinite loop).

**Mitigation:**
```rust
// In Slack reaction handler
if event.user == config.bot_user_id {
    debug!("Ignoring bot's own reaction");
    return Ok(());
}
```

Already implemented in existing `aof-triggers/platforms/slack.rs` (line 41 shows `bot_user_id` config).

## Recommended Reading

### Enterprise Integration Patterns
- [Channel Adapter Pattern](https://www.enterpriseintegrationpatterns.com/patterns/messaging/ChannelAdapter.html)
- [Message Translator](https://www.enterpriseintegrationpatterns.com/patterns/messaging/MessageTranslator.html)
- [Hub and Spoke](https://www.enterpriseintegrationpatterns.com/ramblings/03_hubandspoke.html)

### OpenClaw Architecture (Real-World Hub-and-Spoke)
- [OpenClaw Architecture Explained](https://ppaolo.substack.com/p/openclaw-system-architecture-overview)
- [OpenClaw Gateway Architecture](https://docs.openclaw.ai/concepts/architecture)
- [OpenClaw Deep Dive](https://rajvijayaraj.substack.com/p/openclaw-architecture-a-deep-dive)

### Platform-Specific Documentation
- [Slack API Rate Limits](https://api.slack.com/docs/rate-limits)
- [Slack Socket Mode](https://api.slack.com/apis/connections/socket)
- [Discord Rate Limits](https://docs.discord.com/developers/topics/rate-limits)
- [Discord Gateway WebSocket](https://discord.com/developers/docs/topics/gateway)
- [Telegram Bot API](https://core.telegram.org/bots/api)

### Rust Crates
- [slack-morphism Documentation](https://docs.rs/slack-morphism/latest/slack_morphism/)
- [serenity Documentation](https://docs.rs/serenity/latest/serenity/)
- [teloxide GitHub](https://github.com/teloxide/teloxide)
- [governor Rate Limiter](https://docs.rs/governor/latest/governor/)

### Rate Limiting & Performance
- [How to Implement Rate Limiting in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-rate-limiting/view)
- [Implementing API Rate Limiting with Shuttle](https://www.shuttle.dev/blog/2024/02/22/api-rate-limiting-rust)
- [GCRA Algorithm (governor)](https://github.com/boinkor-net/governor#algorithm)

### NAT Traversal
- [Awesome Tunneling (ngrok alternatives)](https://github.com/anderspitman/awesome-tunneling)
- [Connectivity to Slack without Ngrok](https://forum.rasa.com/t/connectivity-to-slack-without-using-ngrok/10346)

### Rust Message Queues & Broadcasting
- [RSQueue - High-Performance Rust Queue](https://rsqueue.com/)
- [How to Build Message Queue Consumers in Rust](https://oneuptime.com/blog/post/2026-02-01-rust-message-queue-consumers/view)
- [multiqueue - Broadcast Queue](https://docs.rs/multiqueue)

## RESEARCH COMPLETE

**Next Steps:**
1. Create `03-01-PLAN.md` - Core gateway hub with channel adapter trait
2. Create `03-02-PLAN.md` - Platform adapters implementation (Slack, Discord, Telegram)
3. Create `03-03-PLAN.md` - Rate limiting, squad broadcast, configuration

**Key Dependencies:**
- Phase 1 complete (WebSocket event infrastructure exists)
- Existing trigger platforms in `aof-triggers/platforms/` can be reference implementation
- `CoordinationEvent` type from `aof-core/coordination.rs` is the target event format

**Success Metrics:**
- Slack message → agent response in <2 seconds
- Discord integration works identically (adapter transparency)
- Rate limiting prevents 429 errors (0 rate limit violations in 7-day test)
- NAT-transparent operation (no ngrok/public endpoint required)
- Squad broadcast reaches all target agents (100% delivery rate)
