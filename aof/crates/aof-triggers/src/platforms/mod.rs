//! Platform abstraction for messaging platforms
//!
//! This module defines the core traits and types for integrating
//! different messaging platforms (Telegram, Slack, Discord, etc.)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::response::TriggerResponse;

/// Platform-specific errors
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unsupported message type")]
    UnsupportedMessageType,
}

/// User information from platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerUser {
    /// Platform-specific user ID
    pub id: String,

    /// Username or handle
    pub username: Option<String>,

    /// Display name
    pub display_name: Option<String>,

    /// Is bot/system user
    pub is_bot: bool,
}

/// Message from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMessage {
    /// Unique message ID
    pub id: String,

    /// Platform name (telegram, slack, discord, etc.)
    pub platform: String,

    /// Channel/chat ID where message was sent
    pub channel_id: String,

    /// User who sent the message
    pub user: TriggerUser,

    /// Message text content
    pub text: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Additional platform-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Thread ID (for threaded conversations)
    pub thread_id: Option<String>,

    /// Reply to message ID
    pub reply_to: Option<String>,
}

impl TriggerMessage {
    /// Create a new trigger message
    pub fn new(
        id: String,
        platform: String,
        channel_id: String,
        user: TriggerUser,
        text: String,
    ) -> Self {
        Self {
            id,
            platform,
            channel_id,
            user,
            text,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            thread_id: None,
            reply_to: None,
        }
    }

    /// Set thread ID for threaded conversations
    pub fn with_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    /// Set reply-to message ID
    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if message is a command (starts with /)
    pub fn is_command(&self) -> bool {
        self.text.trim().starts_with('/')
    }

    /// Check if message mentions bot (contains @botname)
    pub fn mentions_bot(&self, bot_name: &str) -> bool {
        self.text.contains(&format!("@{}", bot_name))
    }
}

/// Platform abstraction trait
///
/// Implement this trait to add support for new messaging platforms.
/// Each platform handles:
/// - Parsing incoming webhook payloads
/// - Verifying signatures/authenticity
/// - Sending responses back to the platform
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    /// Parse raw webhook payload into TriggerMessage
    ///
    /// # Arguments
    /// * `raw` - Raw HTTP request body bytes
    /// * `headers` - HTTP headers for signature verification
    ///
    /// # Returns
    /// Parsed TriggerMessage on success
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError>;

    /// Send a response back to the platform
    ///
    /// # Arguments
    /// * `channel` - Channel/chat ID to send to
    /// * `response` - Response to send
    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError>;

    /// Get platform name identifier
    fn platform_name(&self) -> &'static str;

    /// Verify webhook signature/authenticity
    ///
    /// # Arguments
    /// * `payload` - Raw payload bytes
    /// * `signature` - Signature from headers
    ///
    /// # Returns
    /// true if signature is valid
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool;

    /// Get bot name for mention detection
    fn bot_name(&self) -> &str;

    /// Check if platform supports threading
    fn supports_threading(&self) -> bool {
        false
    }

    /// Check if platform supports interactive elements
    fn supports_interactive(&self) -> bool {
        false
    }

    /// Check if platform supports file uploads
    fn supports_files(&self) -> bool {
        false
    }
}

// Platform-specific implementations
pub mod slack;
pub mod discord;
pub mod telegram;
pub mod whatsapp;

// Re-export platform types
pub use slack::{SlackConfig, SlackPlatform};
pub use discord::{DiscordConfig, DiscordPlatform};
pub use telegram::{TelegramConfig, TelegramPlatform};
pub use whatsapp::{WhatsAppConfig, WhatsAppPlatform};

// Type aliases for easier use
pub type Platform = Box<dyn TriggerPlatform>;

/// Platform configuration (general purpose)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PlatformConfig {
    /// Platform name identifier
    pub platform: String,

    /// API token/key
    #[serde(default)]
    pub api_token: Option<String>,

    /// Webhook secret for verification
    #[serde(default)]
    pub webhook_secret: Option<String>,

    /// Webhook URL (for setup)
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Typed platform configuration enum
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypedPlatformConfig {
    Slack(SlackConfig),
    Discord(DiscordConfig),
    Telegram(TelegramConfig),
    WhatsApp(WhatsAppConfig),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_message_creation() {
        let user = TriggerUser {
            id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            is_bot: false,
        };

        let msg = TriggerMessage::new(
            "msg123".to_string(),
            "telegram".to_string(),
            "chat456".to_string(),
            user,
            "/run agent-name task description".to_string(),
        );

        assert_eq!(msg.id, "msg123");
        assert_eq!(msg.platform, "telegram");
        assert!(msg.is_command());
    }

    #[test]
    fn test_command_detection() {
        let user = TriggerUser {
            id: "user123".to_string(),
            username: None,
            display_name: None,
            is_bot: false,
        };

        let cmd_msg = TriggerMessage::new(
            "1".to_string(),
            "test".to_string(),
            "ch1".to_string(),
            user.clone(),
            "/help".to_string(),
        );
        assert!(cmd_msg.is_command());

        let text_msg = TriggerMessage::new(
            "2".to_string(),
            "test".to_string(),
            "ch1".to_string(),
            user,
            "Hello world".to_string(),
        );
        assert!(!text_msg.is_command());
    }

    #[test]
    fn test_bot_mention() {
        let user = TriggerUser {
            id: "user123".to_string(),
            username: None,
            display_name: None,
            is_bot: false,
        };

        let msg = TriggerMessage::new(
            "1".to_string(),
            "test".to_string(),
            "ch1".to_string(),
            user,
            "Hey @aofbot can you help?".to_string(),
        );

        assert!(msg.mentions_bot("aofbot"));
        assert!(!msg.mentions_bot("otherbot"));
    }
}
