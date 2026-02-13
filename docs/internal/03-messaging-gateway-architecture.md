# Messaging Gateway Architecture (Phase 3)

**Status:** Phase 3 Plan 01 Complete
**Crate:** `aof-gateway`
**Dependencies:** Phase 1 (Event Infrastructure), aof-core
**Last Updated:** 2026-02-13

## Overview

The messaging gateway is a hub-and-spoke integration pattern that connects multiple messaging platforms (Slack, Discord, Telegram, WhatsApp) to the AOF agent runtime. It provides platform normalization, event translation, rate limiting, and bidirectional message routing.

### Key Design Principles

1. **NAT-transparent**: All connections are outbound (WebSocket/polling), eliminating the need for public endpoints or ngrok
2. **Platform-agnostic**: Unified `ChannelAdapter` trait abstracts platform differences
3. **Event normalization**: All platforms map to standard `CoordinationEvent` format
4. **Rate limiting**: Per-platform token bucket (GCRA) algorithm prevents API throttling
5. **Lifecycle management**: Start/stop adapters gracefully, health checks, error recovery

### Why Hub-and-Spoke?

Traditional point-to-point integrations create N×M complexity (N platforms × M agents). Hub-and-spoke reduces this to N+M:

- **Without hub**: Slack↔Agent, Discord↔Agent, Telegram↔Agent (3×3 = 9 integrations for 3 platforms and 3 agents)
- **With hub**: Platform→Hub→Agent (3+3 = 6 integrations)

The hub acts as a **translation layer and control plane**, not just a message router.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AOF MESSAGING GATEWAY                          │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    GATEWAY HUB (Control Plane)               │   │
│  │  - Message routing                                           │   │
│  │  - Event translation (Platform → CoordinationEvent)          │   │
│  │  - Rate limiting (per-platform token buckets)                │   │
│  │  - Adapter lifecycle management                              │   │
│  │  - Connection to agent runtime via broadcast channel         │   │
│  └──────────┬──────────────┬──────────────┬──────────────┬──────┘   │
│             │              │              │              │           │
│  ┌──────────▼─────┐  ┌────▼────┐  ┌──────▼──────┐  ┌───▼──────┐   │
│  │ Slack Adapter  │  │ Discord │  │ Telegram    │  │ WhatsApp │   │
│  │ (Socket Mode)  │  │ (Gateway)│ │ (Polling)   │  │ (Future) │   │
│  └────────┬───────┘  └────┬─────┘  └──────┬──────┘  └────┬─────┘   │
│           │               │               │              │          │
└───────────┼───────────────┼───────────────┼──────────────┼──────────┘
            │               │               │              │
            ▼               ▼               ▼              ▼
    NAT-TRANSPARENT (outbound WebSocket/polling)
            │               │               │              │
            ▼               ▼               ▼              ▼
    ┌───────────────────────────────────────────────────────┐
    │  Agent Runtime (Phase 1 Infrastructure)                │
    │  - tokio::broadcast event bus                         │
    │  - AgentExecutor                                      │
    │  - Memory backends                                    │
    └───────────────────────────────────────────────────────┘
```

## Core Components

### 1. GatewayHub (Control Plane)

**File:** `crates/aof-gateway/src/hub.rs`

The central orchestrator that manages adapters, routes messages, and coordinates with the agent runtime.

**Responsibilities:**
- **Adapter registry**: Store and manage channel adapters (HashMap by adapter_id)
- **Rate limiting**: Per-platform rate limiters (GCRA token bucket)
- **Event routing**: Translate InboundMessage → CoordinationEvent → broadcast to runtime
- **Lifecycle management**: Start all adapters, graceful shutdown
- **Session management**: Generate and maintain session UUID

**Key Methods:**
```rust
pub struct GatewayHub {
    session_id: String,
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,
    rate_limiters: HashMap<Platform, RateLimiter>,
    event_tx: broadcast::Sender<CoordinationEvent>,
    shutdown_rx: watch::Receiver<bool>,
}

impl GatewayHub {
    pub fn new(event_tx, shutdown_rx) -> Self;
    pub fn register_adapter(&mut self, adapter: Box<dyn ChannelAdapter>);
    pub async fn start(&mut self) -> Result<(), AofError>;
    pub async fn run(&mut self) -> Result<(), AofError>;  // Event loop
    pub async fn stop(&mut self) -> Result<(), AofError>;
}
```

**Event Loop (Future Implementation):**

The `run()` method will use `tokio::select!` to poll multiple adapters concurrently:

```rust
pub async fn run(&mut self) -> Result<(), AofError> {
    loop {
        tokio::select! {
            // Poll each adapter for messages
            msg = adapter1.receive_message() => {
                self.handle_message(msg?).await?;
            }
            msg = adapter2.receive_message() => {
                self.handle_message(msg?).await?;
            }
            // ... more adapters

            // Shutdown signal
            _ = self.shutdown_rx.changed() => {
                if *self.shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }
    Ok(())
}
```

### 2. ChannelAdapter Trait (Platform Interface)

**File:** `crates/aof-gateway/src/adapters/channel_adapter.rs`

Platform-agnostic trait that all messaging platform adapters must implement.

**Design Philosophy:**
- **Send + Sync**: Required for `tokio::spawn` and concurrent execution
- **Trait objects**: Use `Box<dyn ChannelAdapter>` for dynamic dispatch
- **Error normalization**: All errors return `AofError` (no platform-specific types leak)
- **Lifecycle hooks**: Start, stop, health_check for graceful management
- **Message normalization**: Platform quirks hidden behind `InboundMessage`

**Trait Definition:**
```rust
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn platform(&self) -> Platform;

    async fn start(&mut self) -> Result<(), AofError>;
    async fn stop(&mut self) -> Result<(), AofError>;
    async fn health_check(&self) -> Result<bool, AofError>;

    async fn receive_message(&mut self) -> Result<InboundMessage, AofError>;
    async fn send_message(&self, response: &AgentResponse) -> Result<(), AofError>;
}
```

**Platform Types:**
```rust
pub enum Platform {
    Slack,     // Slack Socket Mode (WebSocket)
    Discord,   // Discord Gateway (WebSocket)
    Telegram,  // Telegram Bot API (long polling)
    WhatsApp,  // WhatsApp Business API (webhooks)
}
```

### 3. Event Translation Layer

**File:** `crates/aof-gateway/src/translation.rs`

Normalizes platform-specific messages to `CoordinationEvent` format for agent runtime.

**Translation Flow:**

```
Platform Message (Slack, Discord, etc.)
    ↓
InboundMessage (normalized)
    ↓
CoordinationEvent (agent runtime format)
    ↓
Broadcast to agents via tokio::broadcast
```

**InboundMessage Structure:**
```rust
pub struct InboundMessage {
    message_id: String,         // Platform-specific ID
    platform: Platform,         // Source platform
    channel_id: String,         // Channel/chat/room ID
    thread_id: Option<String>,  // Thread ID (if platform supports threading)
    user: MessageUser,          // Normalized user identity
    content: String,            // Message content (normalized to markdown)
    attachments: Vec<Attachment>, // Files, images, videos
    metadata: serde_json::Value, // Platform-specific extras
    timestamp: DateTime<Utc>,   // UTC timestamp
}
```

**Translation Function:**
```rust
pub fn translate_to_coordination_event(
    message: &InboundMessage,
    session_id: &str,
) -> Result<CoordinationEvent, AofError> {
    let activity = ActivityEvent::new(
        ActivityType::Info,
        format!("Message from {:?} in {}", message.platform, message.channel_id)
    );

    // Add message metadata to activity details
    // ...

    let agent_id = format!("gateway-{:?}", message.platform).to_lowercase();
    Ok(CoordinationEvent::from_activity(activity, agent_id, session_id))
}
```

**Design Notes:**
- **Markdown as lingua franca**: All content normalized to markdown (LLM-friendly)
- **Metadata preservation**: Platform quirks stored in `metadata` JSON field
- **Thread handling**: Platforms without threading use `thread_id: None`
- **Attachment normalization**: Images, files, videos unified to enum variants

### 4. Rate Limiter (GCRA Token Bucket)

**File:** `crates/aof-gateway/src/rate_limiter.rs`

Rate limiting abstraction using the `governor` crate (Generic Cell Rate Algorithm).

**Why GCRA?**
- **Smooth rate limiting**: No thundering herd (tokens refill continuously, not in bursts)
- **Burst allowance**: Allows short bursts up to `burst_size` tokens
- **Async-ready**: `until_ready().await` integrates with tokio
- **No lock contention**: Lock-free implementation for high concurrency

**Per-Platform Defaults:**
```rust
Platform::Slack     => 1 req/sec,  burst 5   // Strict Slack limits
Platform::Discord   => 10 req/sec, burst 20  // Discord allows higher rate
Platform::Telegram  => 30 msg/sec, burst 50  // Telegram is permissive
Platform::WhatsApp  => 1 req/sec,  burst 10  // 1000 msg/day ≈ 1/sec
```

**Usage:**
```rust
let limiter = RateLimiter::new(Platform::Slack, config);

// Async blocking (waits until token available)
limiter.acquire().await?;

// Non-blocking check (returns Err if no tokens)
limiter.check()?;

// Monitoring
let stats = limiter.stats();
```

**Integration with Hub:**

The hub applies rate limiting before broadcasting events:

```rust
async fn handle_message(&self, message: InboundMessage) -> Result<(), AofError> {
    // Apply rate limit for platform
    if let Some(limiter) = self.rate_limiters.get(&message.platform) {
        limiter.acquire().await?;
    }

    // Translate and broadcast
    let event = translate_to_coordination_event(&message, &self.session_id)?;
    self.event_tx.send(event)?;

    Ok(())
}
```

### 5. Configuration Schema

**File:** `crates/aof-gateway/src/config.rs`

YAML-based gateway configuration following AOF resource pattern (`apiVersion`, `kind`, `metadata`, `spec`).

**Example Configuration:**
```yaml
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: messaging-gateway

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"
    session_id: "${SESSION_ID}"  # Auto-generated if not set

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${SLACK_BOT_TOKEN}"
        app_token: "${SLACK_APP_TOKEN}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

    - platform: discord
      enabled: true
      config:
        bot_token: "${DISCORD_BOT_TOKEN}"
      rate_limit:
        requests_per_second: 10
        burst_size: 20
```

**Environment Variable Substitution:**

The loader automatically resolves `${VAR}` placeholders:

```rust
fn resolve_env_vars(yaml: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();
    re.replace_all(yaml, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_else(|_| String::new())
    }).to_string()
}
```

**Validation:**

The loader validates `apiVersion` and `kind` fields using `serde_path_to_error` for precise error messages:

```rust
pub fn load_gateway_config(path: &str) -> Result<GatewayConfig, AofError> {
    let content = fs::read_to_string(path)?;
    let resolved = resolve_env_vars(&content);

    let deserializer = serde_yaml::Deserializer::from_str(&resolved);
    let config: GatewayConfig = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| AofError::config(format!("Field: {}\nError: {}", e.path(), e.inner())))?;

    validate_config(&config)?;
    Ok(config)
}
```

## Adding a New Platform Adapter

Follow these steps to implement a new messaging platform adapter (e.g., Slack, Discord, Telegram).

### Step 1: Create Adapter Crate (Optional)

For complex adapters, create a separate crate:

```bash
mkdir -p crates/aof-gateway-slack
cargo new --lib crates/aof-gateway-slack
```

Add to workspace `Cargo.toml`:
```toml
members = ["crates/aof-gateway-slack"]
```

### Step 2: Implement ChannelAdapter Trait

Create your adapter struct:

```rust
use async_trait::async_trait;
use aof_core::AofError;
use aof_gateway::{ChannelAdapter, Platform, InboundMessage, AgentResponse};

pub struct SlackAdapter {
    adapter_id: String,
    bot_token: String,
    client: SlackClient,  // Platform-specific client
}

#[async_trait]
impl ChannelAdapter for SlackAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn platform(&self) -> Platform {
        Platform::Slack
    }

    async fn start(&mut self) -> Result<(), AofError> {
        // Initialize WebSocket connection
        self.client.connect(&self.bot_token).await
            .map_err(|e| AofError::runtime(format!("Slack connect failed: {}", e)))?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        // Close WebSocket gracefully
        self.client.disconnect().await
            .map_err(|e| AofError::runtime(format!("Slack disconnect failed: {}", e)))?;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        // Check WebSocket connection status
        Ok(self.client.is_connected())
    }

    async fn receive_message(&mut self) -> Result<InboundMessage, AofError> {
        // Poll for next message from platform
        let slack_msg = self.client.next_message().await
            .map_err(|e| AofError::runtime(format!("Slack receive failed: {}", e)))?;

        // Normalize to InboundMessage
        Ok(InboundMessage {
            message_id: slack_msg.ts,
            platform: Platform::Slack,
            channel_id: slack_msg.channel,
            thread_id: slack_msg.thread_ts,
            user: MessageUser {
                user_id: slack_msg.user,
                username: slack_msg.username,
                display_name: None,
            },
            content: slack_msg.text,
            attachments: vec![],
            metadata: serde_json::to_value(&slack_msg).unwrap_or_default(),
            timestamp: Utc::now(),
        })
    }

    async fn send_message(&self, response: &AgentResponse) -> Result<(), AofError> {
        // Translate agent response to platform format
        self.client.post_message(
            &response.target_channel,
            &response.content,
            response.thread_id.as_deref(),
        ).await
            .map_err(|e| AofError::runtime(format!("Slack send failed: {}", e)))?;
        Ok(())
    }
}
```

### Step 3: Handle Platform Quirks

Each platform has unique characteristics to normalize:

**Slack:**
- Threading: `thread_ts` field
- Rich formatting: Slack's mrkdwn → markdown conversion
- Reactions: Store in `metadata`
- File uploads: Map to `Attachment::File`

**Discord:**
- Threading: Thread channels vs. main channels
- Embeds: Rich embeds → markdown conversion
- Voice channels: Ignore (text-only gateway)
- Roles/mentions: `<@123>` → normalized format

**Telegram:**
- No threading: Always `thread_id: None`
- Inline keyboards: Store in `metadata`
- Bot commands: `/start` → parse as message
- Media groups: Multiple `Attachment` entries

**WhatsApp:**
- Templates: Constrained message format
- Session messages: 24-hour window
- Media: Images, videos, documents

### Step 4: Test with Mock Adapter

Use the integration test harness:

```rust
#[tokio::test]
async fn test_slack_adapter_integration() {
    let (event_tx, _event_rx) = broadcast::channel(100);
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut hub = GatewayHub::new(event_tx, shutdown_rx);

    let adapter = Box::new(SlackAdapter::new("test-slack", "test-token"));
    hub.register_adapter(adapter);

    hub.start().await.unwrap();
    // ... test message flow
    hub.stop().await.unwrap();
}
```

### Step 5: Add to Gateway Configuration

Register in `gateway.yaml`:

```yaml
adapters:
  - platform: slack
    enabled: true
    config:
      bot_token: "${SLACK_BOT_TOKEN}"
      app_token: "${SLACK_APP_TOKEN}"
    rate_limit:
      requests_per_second: 1
      burst_size: 5
```

## Testing Strategy

### Unit Tests

**Location:** `crates/aof-gateway/src/` (inline `#[cfg(test)]` modules)

**Coverage:**
- Rate limiter timing tests (GCRA algorithm)
- Config loading and validation
- Event translation (InboundMessage → CoordinationEvent)
- Platform enum serialization

**Run:**
```bash
cargo test -p aof-gateway --lib
```

### Integration Tests

**Location:** `crates/aof-gateway/tests/`

**Test Files:**
- `channel_adapter_test.rs`: Mock adapter trait implementation
- `translation_test.rs`: Platform message translation
- `rate_limiter_test.rs`: Rate limiting behavior
- `config_test.rs`: Configuration loading
- `integration_test.rs`: Full gateway flow with mock adapter

**Run:**
```bash
cargo test -p aof-gateway
```

**Coverage:**
- Mock adapter lifecycle (start, stop, health_check)
- Message flow: adapter → hub → event broadcast
- Shutdown signal handling
- Rate limiting integration

### Manual Testing (Live APIs)

For testing with real Slack/Discord/Telegram APIs:

1. **Set up bot credentials:**
   ```bash
   export SLACK_BOT_TOKEN="xoxb-..."
   export SLACK_APP_TOKEN="xapp-..."
   ```

2. **Create gateway config:**
   ```bash
   cp examples/gateway.yaml /tmp/test-gateway.yaml
   # Edit /tmp/test-gateway.yaml with your tokens
   ```

3. **Run gateway:**
   ```bash
   cargo run -p aofctl -- serve --config /tmp/test-gateway.yaml
   ```

4. **Send test message in Slack:**
   - Message should appear in agent runtime logs
   - Agent response should appear in Slack thread

5. **Verify rate limiting:**
   - Send rapid-fire messages
   - Observe 429 errors if rate limit exceeded
   - Check logs for backpressure handling

## Configuration

### Multi-Workspace Support

The gateway supports multiple adapters per platform (e.g., multiple Slack workspaces):

```yaml
adapters:
  - platform: slack
    enabled: true
    config:
      adapter_id: "slack-workspace-1"
      bot_token: "${SLACK_WORKSPACE_1_TOKEN}"
    rate_limit:
      requests_per_second: 1
      burst_size: 5

  - platform: slack
    enabled: true
    config:
      adapter_id: "slack-workspace-2"
      bot_token: "${SLACK_WORKSPACE_2_TOKEN}"
    rate_limit:
      requests_per_second: 1
      burst_size: 5
```

### Disabled Adapters

Set `enabled: false` to disable an adapter without removing its configuration:

```yaml
adapters:
  - platform: telegram
    enabled: false  # Temporarily disabled
    config:
      bot_token: "${TELEGRAM_BOT_TOKEN}"
```

### Session ID

If not provided, the hub auto-generates a UUID session ID:

```yaml
spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"
    # session_id omitted - auto-generated
```

## Future Enhancements (Out of Scope for 03-01)

### Squad Broadcast (Plan 03-03)

Broadcast messages to all agents or specific teams:

```rust
pub async fn broadcast_to_squad(
    &self,
    message: &str,
    squad_ids: Vec<String>,
) -> Result<(), AofError> {
    // Fan-out message to multiple channels
}
```

### Hot-Reload Configuration

Watch `gateway.yaml` for changes and reload adapters:

```rust
pub async fn reload_config(&mut self, config: GatewayConfig) -> Result<(), AofError> {
    // Stop old adapters
    // Start new adapters from updated config
}
```

### Per-Route Rate Limiting (Discord Buckets)

Discord uses per-route rate limits (not just per-platform):

```rust
pub struct DiscordRateLimiter {
    global_limiter: RateLimiter,
    bucket_limiters: HashMap<String, RateLimiter>,  // Per route
}
```

### Message Persistence

Store messages beyond session memory for audit trails:

```rust
pub async fn persist_message(&self, message: &InboundMessage) -> Result<(), AofError> {
    // Write to persistent storage (SQLite, PostgreSQL)
}
```

### Adapter Health Monitoring

Continuous health checks with auto-restart on failure:

```rust
pub async fn monitor_adapter_health(&self) -> Result<(), AofError> {
    loop {
        for adapter in &self.adapters {
            if !adapter.health_check().await? {
                adapter.restart().await?;
            }
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
```

## Related Files

- **Hub:** `crates/aof-gateway/src/hub.rs`
- **ChannelAdapter trait:** `crates/aof-gateway/src/adapters/channel_adapter.rs`
- **Translation:** `crates/aof-gateway/src/translation.rs`
- **Rate limiter:** `crates/aof-gateway/src/rate_limiter.rs`
- **Config:** `crates/aof-gateway/src/config.rs`
- **Tests:** `crates/aof-gateway/tests/*.rs`
- **Integration test:** `crates/aof-gateway/tests/integration_test.rs`

## References

- **Phase 1 Event Infrastructure:** `docs/dev/event-infrastructure.md`
- **CoordinationEvent:** `crates/aof-core/src/coordination.rs`
- **ActivityEvent:** `crates/aof-core/src/activity.rs`
- **Governor crate:** https://docs.rs/governor (GCRA rate limiting)
- **Slack Socket Mode:** https://api.slack.com/apis/connections/socket
- **Discord Gateway:** https://discord.com/developers/docs/topics/gateway
- **Telegram Bot API:** https://core.telegram.org/bots/api

---

**Document Status:** Complete
**Author:** Phase 3 execution agent
**Last Review:** 2026-02-13
