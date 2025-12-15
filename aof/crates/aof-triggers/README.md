# AOF Triggers

Event-driven triggers and platform integrations for the Agentic Ops Framework (AOF).

## Overview

`aof-triggers` provides abstractions and implementations for triggering agent workflows from various messaging platforms. It handles:
- Message parsing and command extraction
- Response formatting (Markdown, Block Kit, Embeds)
- Webhook signature verification
- User access control and rate limiting
- Integration with AOF RuntimeOrchestrator

## Features

- **Platform Abstraction**: Common `TriggerPlatform` trait for all messaging platforms
- **Command Parsing**: Structured command parsing with arguments and parameters
- **Rich Responses**: Platform-specific formatting (Telegram Markdown, Slack Block Kit, Discord Embeds)
- **Security**: Webhook signature verification, user whitelisting, rate limiting
- **Async/Await**: Fully asynchronous API using tokio
- **HTTP Server**: Built-in axum webhook server

## Supported Platforms

| Platform | Adapter | Features |
|----------|---------|----------|
| Telegram | `TelegramPlatform` | Text, inline keyboards, callbacks, MarkdownV2 |
| Slack | `SlackPlatform` | Events API, Block Kit, interactive components |
| Discord | `DiscordPlatform` | Slash commands, embeds, buttons, Ed25519 |
| WhatsApp | `WhatsAppPlatform` | Cloud API, interactive buttons/lists |

## Quick Start

### Add Dependency

```toml
[dependencies]
aof-triggers = { path = "../aof-triggers" }
aof-runtime = { path = "../aof-runtime" }
tokio = { version = "1.35", features = ["full"] }
```

### Create Webhook Server

```rust
use std::sync::Arc;
use aof_triggers::{
    TriggerServer, TriggerHandler,
    TelegramPlatform, TelegramConfig,
    SlackPlatform, SlackConfig,
    DiscordPlatform, DiscordConfig,
    WhatsAppPlatform, WhatsAppConfig,
};
use aof_runtime::RuntimeOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize runtime
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Create handler
    let mut handler = TriggerHandler::new(Arc::clone(&orchestrator));

    // Register Telegram
    let telegram = TelegramPlatform::new(TelegramConfig {
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN")?,
        webhook_secret: Some(std::env::var("TELEGRAM_WEBHOOK_SECRET").ok()),
        bot_name: "aofbot".to_string(),
        allowed_users: None,
        allowed_groups: None,
        webhook_url: None,
    })?;
    handler.register_platform(Arc::new(telegram));

    // Register Slack
    let slack = SlackPlatform::new(SlackConfig {
        bot_token: std::env::var("SLACK_BOT_TOKEN")?,
        signing_secret: std::env::var("SLACK_SIGNING_SECRET")?,
        app_id: std::env::var("SLACK_APP_ID")?,
        bot_user_id: std::env::var("SLACK_BOT_USER_ID")?,
        bot_name: "aofbot".to_string(),
        allowed_workspaces: None,
        allowed_channels: None,
    })?;
    handler.register_platform(Arc::new(slack));

    // Start server on port 8080
    let server = TriggerServer::new(Arc::new(handler));
    server.serve().await?;

    Ok(())
}
```

## Command Reference

### Agent Commands

```
/run agent <name> <input>     Run an agent with input
/status agent <id>            Get agent status
/list agents                  List available agents
```

### Task Commands

```
/create task <description>    Create a new task
/status task <id>             Check task status
/cancel task <id>             Cancel running task
/list tasks                   List all tasks
```

### Fleet Commands

```
/create fleet <name>          Create agent fleet
/status fleet <name>          Check fleet status
/list fleets                  List all fleets
```

### Flow Commands

```
/run flow <name>              Execute workflow
/status flow <id>             Check flow status
/cancel flow <id>             Cancel workflow
```

## Platform Configuration

### Telegram

```rust
TelegramConfig {
    bot_token: "123456:ABC-DEF1234...".to_string(),
    webhook_url: Some("https://example.com/webhook/telegram".to_string()),
    webhook_secret: Some("your-secret-token".to_string()),
    bot_name: "aofbot".to_string(),
    allowed_users: Some(vec![123456789]),  // Optional whitelist
    allowed_groups: Some(vec![-100123456789]),
}
```

### Slack

```rust
SlackConfig {
    bot_token: "xoxb-...".to_string(),
    signing_secret: "abc123...".to_string(),
    app_id: "A0123456789".to_string(),
    bot_user_id: "U0123456789".to_string(),
    bot_name: "aofbot".to_string(),
    allowed_workspaces: None,
    allowed_channels: None,
}
```

### Discord

```rust
DiscordConfig {
    bot_token: "MTIzNDU2Nzg5...".to_string(),
    application_id: "123456789012345678".to_string(),
    public_key: "abc123def456...".to_string(),
    guild_ids: None,
    allowed_roles: None,
}
```

### WhatsApp

```rust
WhatsAppConfig {
    phone_number_id: "123456789012345".to_string(),
    access_token: "EAABcD...".to_string(),
    verify_token: "your-verify-token".to_string(),
    app_secret: "abc123...".to_string(),
    business_account_id: None,
    allowed_numbers: None,
    api_version: "v18.0".to_string(),
}
```

## Security

### Signature Verification

All platforms support webhook signature verification:

```rust
// Telegram: X-Telegram-Bot-Api-Secret-Token header
// Slack: X-Slack-Signature (HMAC-SHA256)
// Discord: X-Signature-Ed25519 + X-Signature-Timestamp
// WhatsApp: X-Hub-Signature-256 (HMAC-SHA256)
```

### User Whitelisting

```rust
// Telegram
allowed_users: Some(vec![123456789, 987654321]),
allowed_groups: Some(vec![-100123456789]),

// Slack
allowed_workspaces: Some(vec!["T0123456789".to_string()]),
allowed_channels: Some(vec!["C0123456789".to_string()]),

// Discord
allowed_roles: Some(vec!["admin".to_string()]),

// WhatsApp
allowed_numbers: Some(vec!["+1234567890".to_string()]),
```

## Architecture

### TriggerPlatform Trait

```rust
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError>;

    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError>;

    fn platform_name(&self) -> &'static str;

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool;

    fn bot_name(&self) -> &str;

    fn supports_threading(&self) -> bool;
    fn supports_interactive(&self) -> bool;
    fn supports_files(&self) -> bool;
}
```

### Message Flow

```
Webhook → TriggerServer → TriggerHandler → CommandParser → RuntimeOrchestrator
                                                ↓
                                        TriggerResponse ← Task Execution
```

## HTTP Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Service info |
| `/health` | GET | Health check |
| `/webhook/{platform}` | POST | Platform webhooks |
| `/platforms` | GET | List registered platforms |

## Testing

```bash
# Run all tests
cargo test -p aof-triggers

# Run with logging
RUST_LOG=debug cargo test -p aof-triggers -- --nocapture

# Run specific test
cargo test -p aof-triggers test_telegram_parse
```

## Documentation

- [Full Integration Guide](../../docs/triggers-integration-guide.md)
- [Architecture Design](../../docs/triggers-architecture.md)
- [API Research](../../docs/research/messaging-platforms-api-research.md)

## License

Licensed under MIT or Apache-2.0 (same as AOF framework)
