# Building Custom Triggers and Platform Integrations

This guide covers how to extend AOF with custom platform triggers, event handlers, and integrations. AOF is designed as a **pluggable, extensible framework** where each component is a reusable building block.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Building a Custom Platform Trigger](#building-a-custom-platform-trigger)
3. [The TriggerPlatform Trait](#the-triggerplatform-trait)
4. [Registering Your Platform](#registering-your-platform)
5. [Webhook Handling](#webhook-handling)
6. [AgentFlow Integration](#agentflow-integration)
7. [Testing Your Trigger](#testing-your-trigger)
8. [Best Practices](#best-practices)
9. [Example: Building a PagerDuty Trigger](#example-building-a-pagerduty-trigger)

---

## Architecture Overview

AOF's trigger system follows a **plugin architecture** with these key components:

```
┌─────────────────────────────────────────────────────────────────┐
│                      AOF TRIGGER SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Webhook    │───▶│  Platform    │───▶│   Trigger    │       │
│  │   Server     │    │  Adapter     │    │   Handler    │       │
│  │  (Axum)      │    │              │    │              │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│        │                    │                   │                │
│        │                    │                   │                │
│        ▼                    ▼                   ▼                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Raw HTTP    │    │  Trigger     │    │   Agent/     │       │
│  │  Request     │    │  Message     │    │   Flow       │       │
│  │              │    │  (Unified)   │    │   Execution  │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Abstractions

| Component | Purpose |
|-----------|---------|
| `TriggerPlatform` trait | Defines how to parse/send messages for a platform |
| `TriggerMessage` | Unified message format across all platforms |
| `TriggerResponse` | Unified response format for all platforms |
| `TriggerHandler` | Routes messages to agents/flows |
| `PlatformRegistry` | Dynamic platform loading and creation |
| `FlowRouter` | Routes messages to matching AgentFlows |

---

## Building a Custom Platform Trigger

### Step 1: Create the Module

Create a new file in `crates/aof-triggers/src/platforms/`:

```rust
// crates/aof-triggers/src/platforms/myplatform.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

/// My Platform adapter
pub struct MyPlatform {
    config: MyPlatformConfig,
    client: reqwest::Client,
}

/// Configuration for My Platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPlatformConfig {
    /// API token for authentication
    pub api_token: String,

    /// Webhook secret for signature verification
    pub webhook_secret: String,

    /// Bot name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// API base URL (for self-hosted instances)
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Optional: Allowed users whitelist
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_api_url() -> String {
    "https://api.myplatform.com".to_string()
}
```

### Step 2: Implement Platform Methods

```rust
impl MyPlatform {
    /// Create a new platform instance
    pub fn new(config: MyPlatformConfig) -> Result<Self, PlatformError> {
        // Validate required fields
        if config.api_token.is_empty() {
            return Err(PlatformError::ParseError(
                "API token is required".to_string()
            ));
        }

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(
                format!("Failed to create HTTP client: {}", e)
            ))?;

        Ok(Self { config, client })
    }

    /// Verify webhook signature
    fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Implement platform-specific signature verification
        // Most platforms use HMAC-SHA256
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(
            self.config.webhook_secret.as_bytes()
        ) {
            Ok(m) => m,
            Err(_) => return false,
        };

        mac.update(payload);
        let computed = hex::encode(mac.finalize().into_bytes());

        // Compare signatures (timing-safe comparison recommended)
        computed == signature
    }

    /// Send a message via the platform's API
    pub async fn send_message(
        &self,
        channel: &str,
        text: &str,
    ) -> Result<String, PlatformError> {
        let url = format!("{}/messages", self.config.api_url);

        let payload = serde_json::json!({
            "channel": channel,
            "text": text,
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PlatformError::ApiError(
                format!("API returned {}", response.status())
            ));
        }

        // Parse response to get message ID
        let data: serde_json::Value = response.json().await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        Ok(data["id"].as_str().unwrap_or("unknown").to_string())
    }
}
```

### Step 3: Implement TriggerPlatform Trait

```rust
#[async_trait]
impl TriggerPlatform for MyPlatform {
    /// Parse incoming webhook payload into unified TriggerMessage
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // 1. Verify signature
        if let Some(signature) = headers.get("x-myplatform-signature") {
            if !self.verify_signature(raw, signature) {
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string()
                ));
            }
        }

        // 2. Parse webhook payload
        let payload: MyWebhookPayload = serde_json::from_slice(raw)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        // 3. Convert to unified TriggerMessage
        let user = TriggerUser {
            id: payload.user.id.clone(),
            username: Some(payload.user.name.clone()),
            display_name: payload.user.display_name.clone(),
            is_bot: payload.user.is_bot,
        };

        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(),
            serde_json::json!(payload.event_type));

        Ok(TriggerMessage {
            id: payload.message_id,
            platform: "myplatform".to_string(),
            channel_id: payload.channel_id,
            user,
            text: payload.text,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: payload.thread_id,
            reply_to: None,
        })
    }

    /// Send response back to the platform
    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Format response for platform
        let text = self.format_response(&response);
        self.send_message(channel, &text).await?;
        Ok(())
    }

    /// Platform identifier (lowercase)
    fn platform_name(&self) -> &'static str {
        "myplatform"
    }

    /// Verify webhook authenticity
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_signature(payload, signature)
    }

    /// Bot name for mention detection
    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    /// Platform capabilities
    fn supports_threading(&self) -> bool {
        true // Set based on platform capabilities
    }

    fn supports_interactive(&self) -> bool {
        true // Buttons, menus, etc.
    }

    fn supports_files(&self) -> bool {
        false // File uploads
    }

    /// For downcasting to platform-specific features
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

---

## The TriggerPlatform Trait

The `TriggerPlatform` trait is the core abstraction for all platforms:

```rust
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    // REQUIRED: Parse incoming webhook
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError>;

    // REQUIRED: Send response
    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError>;

    // REQUIRED: Platform identifier
    fn platform_name(&self) -> &'static str;

    // REQUIRED: Signature verification
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool;

    // REQUIRED: Bot name
    fn bot_name(&self) -> &str;

    // OPTIONAL: Capability flags (default: false)
    fn supports_threading(&self) -> bool { false }
    fn supports_interactive(&self) -> bool { false }
    fn supports_files(&self) -> bool { false }

    // REQUIRED: For downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}
```

---

## Registering Your Platform

### Option 1: Static Registration (Built-in)

Add to `platforms/mod.rs`:

```rust
// Add module
pub mod myplatform;

// Re-export
pub use myplatform::{MyPlatformConfig, MyPlatform};

// Add to TypedPlatformConfig enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypedPlatformConfig {
    Slack(SlackConfig),
    Discord(DiscordConfig),
    Telegram(TelegramConfig),
    WhatsApp(WhatsAppConfig),
    GitHub(GitHubConfig),
    MyPlatform(MyPlatformConfig), // Add here
}

// Register in PlatformRegistry::register_defaults()
self.register("myplatform", Box::new(|config| {
    let cfg: MyPlatformConfig = serde_json::from_value(config)
        .map_err(|e| PlatformError::ParseError(
            format!("Invalid MyPlatform config: {}", e)
        ))?;
    Ok(Box::new(MyPlatform::new(cfg)?))
}));
```

### Option 2: Dynamic Registration (Runtime)

```rust
use aof_triggers::{PlatformRegistry, PlatformFactory};

fn main() {
    let mut registry = PlatformRegistry::with_defaults();

    // Register custom platform at runtime
    registry.register("myplatform", Box::new(|config| {
        let cfg: MyPlatformConfig = serde_json::from_value(config)?;
        Ok(Box::new(MyPlatform::new(cfg)?))
    }));

    // Create platform instance
    let config = serde_json::json!({
        "api_token": "xxx",
        "webhook_secret": "yyy"
    });

    let platform = registry.create("myplatform", config)?;
}
```

---

## Webhook Handling

The webhook server automatically routes requests to your platform:

```
POST /webhook/myplatform
```

### Headers Passed to parse_message()

Common webhook headers (lowercase keys):

| Header | Description |
|--------|-------------|
| `content-type` | Request content type |
| `x-myplatform-signature` | Signature for verification |
| `x-request-id` | Unique request ID |

### Signature Verification Patterns

**HMAC-SHA256 (Slack, GitHub, WhatsApp):**
```rust
fn verify_hmac_sha256(secret: &str, payload: &[u8], signature: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);

    let computed = hex::encode(mac.finalize().into_bytes());
    computed == signature
}
```

**Secret Token (Telegram):**
```rust
fn verify_token(expected: &str, provided: &str) -> bool {
    expected == provided
}
```

**Ed25519 (Discord):**
```rust
fn verify_ed25519(public_key: &str, signature: &str, payload: &[u8]) -> bool {
    use ed25519_dalek::{Signature, VerifyingKey, Verifier};
    // ... implementation
}
```

---

## AgentFlow Integration

Your platform integrates automatically with AgentFlow:

```yaml
# examples/flows/myplatform-bot.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: myplatform-bot
  labels:
    platform: myplatform

spec:
  trigger:
    type: MyPlatform
    config:
      events:
        - message
        - button_click
      api_token: ${MYPLATFORM_TOKEN}
      webhook_secret: ${MYPLATFORM_SECRET}
      # Platform-specific options
      allowed_users:
        - user123
        - user456

  nodes:
    - id: process
      type: Agent
      config:
        agent: assistant
        input: ${event.text}

  connections:
    - from: trigger
      to: process
```

### Event Data Available in Flows

The `TriggerMessage` metadata is available as `${event.*}`:

```yaml
# Access message data
${event.text}           # Message text
${event.user_id}        # User ID
${event.channel_id}     # Channel/conversation ID
${event.thread_id}      # Thread ID (if supported)
${event.metadata.*}     # Platform-specific data
```

---

## Testing Your Trigger

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> MyPlatformConfig {
        MyPlatformConfig {
            api_token: "test-token".to_string(),
            webhook_secret: "test-secret".to_string(),
            bot_name: "testbot".to_string(),
            api_url: "https://test.api.com".to_string(),
            allowed_users: None,
        }
    }

    #[test]
    fn test_platform_creation() {
        let config = create_test_config();
        let platform = MyPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let config = MyPlatformConfig {
            api_token: "".to_string(), // Invalid
            ..create_test_config()
        };
        let platform = MyPlatform::new(config);
        assert!(platform.is_err());
    }

    #[tokio::test]
    async fn test_parse_message() {
        let config = create_test_config();
        let platform = MyPlatform::new(config).unwrap();

        let payload = r#"{
            "message_id": "123",
            "channel_id": "ch456",
            "text": "/run agent test hello",
            "user": {
                "id": "u789",
                "name": "testuser",
                "is_bot": false
            },
            "event_type": "message"
        }"#;

        let mut headers = HashMap::new();
        // Add signature header if needed

        let result = platform.parse_message(
            payload.as_bytes(),
            &headers
        ).await;

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.platform, "myplatform");
        assert_eq!(msg.text, "/run agent test hello");
    }

    #[test]
    fn test_signature_verification() {
        let config = create_test_config();
        let platform = MyPlatform::new(config).unwrap();

        let payload = b"test payload";

        // Generate valid signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        let mut mac = Hmac::<Sha256>::new_from_slice(b"test-secret").unwrap();
        mac.update(payload);
        let valid_sig = hex::encode(mac.finalize().into_bytes());

        assert!(platform.verify_signature(payload, &valid_sig));
        assert!(!platform.verify_signature(payload, "invalid"));
    }
}
```

### Integration Tests

```rust
// tests/integration_myplatform.rs
use aof_triggers::{TriggerHandler, TriggerHandlerConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_full_webhook_flow() {
    // Setup
    let handler = TriggerHandler::new(Arc::new(RuntimeOrchestrator::new()));

    // Register platform
    let config = MyPlatformConfig { ... };
    let platform = Arc::new(MyPlatform::new(config).unwrap());
    handler.register_platform(platform);

    // Simulate webhook
    let message = TriggerMessage {
        id: "test-123".to_string(),
        platform: "myplatform".to_string(),
        channel_id: "ch456".to_string(),
        user: TriggerUser {
            id: "u789".to_string(),
            username: Some("testuser".to_string()),
            display_name: None,
            is_bot: false,
        },
        text: "/run agent assistant hello".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
        thread_id: None,
        reply_to: None,
    };

    // Handle message
    let result = handler.handle_message("myplatform", message).await;
    assert!(result.is_ok());
}
```

---

## Best Practices

### 1. Security

- **Always verify signatures** - Never process unverified webhooks
- **Use environment variables** - Don't hardcode secrets
- **Validate input** - Sanitize user-provided data
- **Rate limiting** - Implement or rely on platform rate limits

### 2. Error Handling

```rust
// Use specific error types
#[derive(Debug, thiserror::Error)]
pub enum MyPlatformError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("User not authorized: {0}")]
    Unauthorized(String),
}

// Convert to PlatformError
impl From<MyPlatformError> for PlatformError {
    fn from(e: MyPlatformError) -> Self {
        match e {
            MyPlatformError::InvalidSignature(s) =>
                PlatformError::InvalidSignature(s),
            MyPlatformError::ApiError(s) =>
                PlatformError::ApiError(s),
            MyPlatformError::RateLimited =>
                PlatformError::RateLimitExceeded,
            MyPlatformError::Unauthorized(s) =>
                PlatformError::InvalidSignature(s),
        }
    }
}
```

### 3. Logging

```rust
use tracing::{debug, info, warn, error};

async fn parse_message(...) -> Result<TriggerMessage, PlatformError> {
    debug!("Parsing webhook payload ({} bytes)", raw.len());

    // Log signature verification
    if signature_valid {
        debug!("Signature verified successfully");
    } else {
        warn!("Invalid signature from IP: {}", client_ip);
        return Err(PlatformError::InvalidSignature(...));
    }

    info!("Received message {} from user {}", msg.id, msg.user.id);

    Ok(msg)
}
```

### 4. Configuration

```rust
/// Use environment variables for secrets
impl MyPlatformConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            api_token: std::env::var("MYPLATFORM_TOKEN")?,
            webhook_secret: std::env::var("MYPLATFORM_SECRET")?,
            bot_name: std::env::var("MYPLATFORM_BOT_NAME")
                .unwrap_or_else(|_| "aofbot".to_string()),
            api_url: std::env::var("MYPLATFORM_API_URL")
                .unwrap_or_else(|_| "https://api.myplatform.com".to_string()),
            allowed_users: None,
        })
    }
}
```

### 5. Response Formatting

```rust
impl MyPlatform {
    /// Format response for platform-specific rendering
    fn format_response(&self, response: &TriggerResponse) -> String {
        let emoji = match response.status {
            ResponseStatus::Success => "✅",
            ResponseStatus::Error => "❌",
            ResponseStatus::Warning => "⚠️",
            ResponseStatus::Info => "ℹ️",
        };

        // Platform-specific formatting
        format!("{} {}", emoji, response.text)
    }
}
```

---

## Example: Building a PagerDuty Trigger

Here's a complete example of implementing a PagerDuty trigger:

```rust
// crates/aof-triggers/src/platforms/pagerduty.rs

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

type HmacSha256 = Hmac<Sha256>;

/// PagerDuty platform adapter for incident-driven workflows
pub struct PagerDutyPlatform {
    config: PagerDutyConfig,
    client: reqwest::Client,
}

/// PagerDuty configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// API key for PagerDuty API
    pub api_key: String,

    /// Webhook secret for signature verification
    pub webhook_secret: String,

    /// Service ID to scope events (optional)
    #[serde(default)]
    pub service_ids: Option<Vec<String>>,

    /// Event types to handle
    #[serde(default)]
    pub event_types: Option<Vec<String>>,
}

/// PagerDuty webhook payload
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyWebhook {
    messages: Vec<PagerDutyMessage>,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyMessage {
    id: String,
    event: String,
    created_on: String,
    incident: Option<PagerDutyIncident>,
    log_entries: Option<Vec<PagerDutyLogEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyIncident {
    id: String,
    title: String,
    status: String,
    urgency: String,
    service: PagerDutyService,
    assigned_to: Vec<PagerDutyAssignment>,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyService {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyAssignment {
    at: String,
    assignee: PagerDutyUser,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyUser {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyLogEntry {
    id: String,
    #[serde(rename = "type")]
    entry_type: String,
    summary: String,
}

impl PagerDutyPlatform {
    pub fn new(config: PagerDutyConfig) -> Result<Self, PlatformError> {
        if config.api_key.is_empty() || config.webhook_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "API key and webhook secret are required".to_string()
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(e.to_string()))?;

        Ok(Self { config, client })
    }

    fn verify_pagerduty_signature(
        &self,
        payload: &[u8],
        signatures: &str
    ) -> bool {
        // PagerDuty sends multiple signatures, check if any match
        for sig in signatures.split(',') {
            let sig = sig.trim();
            if !sig.starts_with("v1=") {
                continue;
            }

            let provided = &sig[3..];

            let mut mac = match HmacSha256::new_from_slice(
                self.config.webhook_secret.as_bytes()
            ) {
                Ok(m) => m,
                Err(_) => continue,
            };

            mac.update(payload);
            let computed = hex::encode(mac.finalize().into_bytes());

            if computed == provided {
                return true;
            }
        }
        false
    }

    fn is_service_allowed(&self, service_id: &str) -> bool {
        match &self.config.service_ids {
            Some(ids) => ids.contains(&service_id.to_string()),
            None => true,
        }
    }

    fn is_event_allowed(&self, event: &str) -> bool {
        match &self.config.event_types {
            Some(types) => types.iter().any(|t| event.starts_with(t)),
            None => true,
        }
    }

    /// Add a note to an incident
    pub async fn add_note(
        &self,
        incident_id: &str,
        note: &str,
        email: &str,
    ) -> Result<(), PlatformError> {
        let url = format!(
            "https://api.pagerduty.com/incidents/{}/notes",
            incident_id
        );

        let payload = serde_json::json!({
            "note": {
                "content": note
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Token token={}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("From", email)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PlatformError::ApiError(
                format!("Failed to add note: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Update incident status
    pub async fn update_incident(
        &self,
        incident_id: &str,
        status: &str,
        email: &str,
    ) -> Result<(), PlatformError> {
        let url = format!(
            "https://api.pagerduty.com/incidents/{}",
            incident_id
        );

        let payload = serde_json::json!({
            "incident": {
                "type": "incident_reference",
                "status": status
            }
        });

        let response = self.client
            .put(&url)
            .header("Authorization", format!("Token token={}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("From", email)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PlatformError::ApiError(
                format!("Failed to update incident: {}", response.status())
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl TriggerPlatform for PagerDutyPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify signature
        if let Some(signatures) = headers.get("x-pagerduty-signature") {
            if !self.verify_pagerduty_signature(raw, signatures) {
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string()
                ));
            }
        }

        // Parse webhook
        let webhook: PagerDutyWebhook = serde_json::from_slice(raw)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        // Get first message
        let msg = webhook.messages.first()
            .ok_or_else(|| PlatformError::ParseError(
                "No messages in webhook".to_string()
            ))?;

        // Check event type filter
        if !self.is_event_allowed(&msg.event) {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Get incident details
        let incident = msg.incident.as_ref()
            .ok_or_else(|| PlatformError::ParseError(
                "No incident in message".to_string()
            ))?;

        // Check service filter
        if !self.is_service_allowed(&incident.service.id) {
            return Err(PlatformError::InvalidSignature(
                "Service not allowed".to_string()
            ));
        }

        // Build user from first assignee
        let user = incident.assigned_to.first()
            .map(|a| TriggerUser {
                id: a.assignee.id.clone(),
                username: Some(a.assignee.email.clone()),
                display_name: Some(a.assignee.name.clone()),
                is_bot: false,
            })
            .unwrap_or_else(|| TriggerUser {
                id: "pagerduty".to_string(),
                username: None,
                display_name: Some("PagerDuty".to_string()),
                is_bot: true,
            });

        // Build text from event
        let text = format!(
            "incident:{}:{} {} - {}",
            msg.event,
            incident.status,
            incident.id,
            incident.title
        );

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("event".to_string(), serde_json::json!(msg.event));
        metadata.insert("incident_id".to_string(), serde_json::json!(incident.id));
        metadata.insert("incident_title".to_string(), serde_json::json!(incident.title));
        metadata.insert("incident_status".to_string(), serde_json::json!(incident.status));
        metadata.insert("urgency".to_string(), serde_json::json!(incident.urgency));
        metadata.insert("service_id".to_string(), serde_json::json!(incident.service.id));
        metadata.insert("service_name".to_string(), serde_json::json!(incident.service.name));

        Ok(TriggerMessage {
            id: msg.id.clone(),
            platform: "pagerduty".to_string(),
            channel_id: incident.service.id.clone(),
            user,
            text,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: Some(incident.id.clone()),
            reply_to: None,
        })
    }

    async fn send_response(
        &self,
        channel: &str, // incident_id
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Add note to incident
        self.add_note(
            channel,
            &response.text,
            "aofbot@company.com"
        ).await
    }

    fn platform_name(&self) -> &'static str {
        "pagerduty"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_pagerduty_signature(payload, signature)
    }

    fn bot_name(&self) -> &str {
        "aofbot"
    }

    fn supports_threading(&self) -> bool {
        true // Incidents have notes/timeline
    }

    fn supports_interactive(&self) -> bool {
        false // No interactive elements
    }

    fn supports_files(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

---

## Summary

Building a custom platform trigger involves:

1. **Create the module** - Define config and platform structs
2. **Implement TriggerPlatform trait** - Parse messages, send responses, verify signatures
3. **Register the platform** - Add to PlatformRegistry
4. **Write tests** - Unit tests for parsing, signature verification
5. **Create AgentFlow examples** - Show users how to use your trigger

The pluggable architecture ensures your trigger integrates seamlessly with:
- Webhook server routing
- AgentFlow workflows
- Command parsing
- Conversation memory
- Approval workflows

For more examples, see the existing platform implementations in `crates/aof-triggers/src/platforms/`.
