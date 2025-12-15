//! AOF Triggers - Platform-agnostic messaging triggers for AOF agents
//!
//! This crate provides abstractions and implementations for triggering
//! AOF agent execution from various messaging platforms (Telegram, Slack,
//! Discord, WhatsApp, etc.) through webhooks and command parsing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod command;
pub mod handler;
pub mod platforms;
pub mod response;
pub mod server;

// Re-export main types from command module
pub use command::{CommandContext, CommandType, TriggerCommand, TriggerTarget};

// Re-export main types from handler module
pub use handler::{TriggerHandler, TriggerHandlerConfig};

// Re-export main types from platforms module
pub use platforms::{Platform, PlatformConfig};

// Re-export main types from response module
pub use response::{ResponseFormat, TriggerResponse, TriggerResponseBuilder};

// Re-export main types from server module
pub use server::{TriggerServer, TriggerServerBuilder, TriggerServerConfig};

// Re-export error types from aof-core
pub use aof_core::{AofError, AofResult};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export all platform types
pub use platforms::{
    SlackConfig, SlackPlatform,
    DiscordConfig, DiscordPlatform,
    TelegramConfig, TelegramPlatform,
    WhatsAppConfig, WhatsAppPlatform,
    TypedPlatformConfig,
};

/// Core trait for trigger platforms
///
/// Implementations provide platform-specific message parsing, response formatting,
/// and authentication/verification.
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    /// Parse incoming message from platform-specific format
    async fn parse_message(&self, payload: &[u8]) -> Result<TriggerMessage, TriggerError>;

    /// Send response back to the platform
    async fn send_response(
        &self,
        channel: &str,
        message: &ResponseMessage,
    ) -> Result<(), TriggerError>;

    /// Verify request authenticity (signature, token, etc.)
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> Result<bool, TriggerError>;

    /// Get platform name
    fn platform_name(&self) -> &str;

    /// Handle platform-specific events (optional)
    async fn handle_event(&self, event: &PlatformEvent) -> Result<Option<TriggerMessage>, TriggerError> {
        Ok(None)
    }
}

/// Unified message format from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMessage {
    /// Unique message ID
    pub message_id: String,

    /// User who sent the message
    pub user_id: String,

    /// Display name of user
    pub user_name: String,

    /// Channel/conversation ID
    pub channel_id: String,

    /// Message content/text
    pub text: String,

    /// Message thread/parent ID (for threaded replies)
    pub thread_id: Option<String>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Platform-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response message to send back
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Message text
    pub text: String,

    /// Thread to reply in (if any)
    pub thread_id: Option<String>,

    /// Rich formatting (platform-specific)
    pub blocks: Option<serde_json::Value>,

    /// Attachments
    pub attachments: Vec<MessageAttachment>,

    /// Ephemeral (visible only to user)
    pub ephemeral: bool,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub title: Option<String>,
    pub text: Option<String>,
    pub color: Option<String>,
    pub fields: Vec<AttachmentField>,
}

/// Attachment field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentField {
    pub title: String,
    pub value: String,
    pub short: bool,
}

/// Platform-specific event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlatformEvent {
    /// URL verification challenge (Slack)
    UrlVerification { challenge: String },

    /// App mentioned
    AppMention {
        user_id: String,
        channel_id: String,
        text: String,
    },

    /// Direct message
    DirectMessage {
        user_id: String,
        channel_id: String,
        text: String,
    },

    /// Slash command
    SlashCommand {
        user_id: String,
        channel_id: String,
        command: String,
        text: String,
    },

    /// Interactive component (button, select, etc.)
    InteractiveAction {
        user_id: String,
        channel_id: String,
        action_id: String,
        value: Option<String>,
    },

    /// Modal/view submission
    ViewSubmission {
        user_id: String,
        view_id: String,
        values: HashMap<String, serde_json::Value>,
    },

    /// Home tab opened
    AppHomeOpened { user_id: String },
}

/// Trigger error types
#[derive(Debug, thiserror::Error)]
pub enum TriggerError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Unsupported event type: {0}")]
    UnsupportedEvent(String),
}

pub type TriggerResult<T> = Result<T, TriggerError>;

impl ResponseMessage {
    /// Create simple text response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            thread_id: None,
            blocks: None,
            attachments: Vec::new(),
            ephemeral: false,
        }
    }

    /// Create threaded reply
    pub fn thread_reply(text: impl Into<String>, thread_id: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            thread_id: Some(thread_id.into()),
            blocks: None,
            attachments: Vec::new(),
            ephemeral: false,
        }
    }

    /// Create ephemeral message (visible only to user)
    pub fn ephemeral(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            thread_id: None,
            blocks: None,
            attachments: Vec::new(),
            ephemeral: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_message_text() {
        let msg = ResponseMessage::text("Hello, world!");
        assert_eq!(msg.text, "Hello, world!");
        assert!(!msg.ephemeral);
        assert!(msg.thread_id.is_none());
    }

    #[test]
    fn test_response_message_thread_reply() {
        let msg = ResponseMessage::thread_reply("Reply", "1234567890.123456");
        assert_eq!(msg.text, "Reply");
        assert_eq!(msg.thread_id, Some("1234567890.123456".to_string()));
    }

    #[test]
    fn test_response_message_ephemeral() {
        let msg = ResponseMessage::ephemeral("Private message");
        assert_eq!(msg.text, "Private message");
        assert!(msg.ephemeral);
    }
}
