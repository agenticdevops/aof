//! Channel adapter trait and core types
//!
//! This module defines the platform-agnostic ChannelAdapter trait that all messaging
//! platform adapters must implement.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use aof_core::AofError;

/// Platform types supported by the gateway
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Slack messaging platform
    Slack,
    /// Discord messaging platform
    Discord,
    /// Telegram messaging platform
    Telegram,
    /// WhatsApp messaging platform
    WhatsApp,
}

/// Normalized inbound message from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    /// Unique message ID (platform-specific)
    pub message_id: String,
    /// Source platform
    pub platform: Platform,
    /// Channel/chat/room ID
    pub channel_id: String,
    /// Thread ID if threaded (Slack thread_ts, Discord thread channel_id)
    pub thread_id: Option<String>,
    /// User who sent message
    pub user: MessageUser,
    /// Message content (normalized to markdown)
    pub content: String,
    /// Attachments (images, files)
    pub attachments: Vec<Attachment>,
    /// Platform-specific metadata (JSON blob for future use)
    pub metadata: serde_json::Value,
    /// When message was sent
    pub timestamp: DateTime<Utc>,
}

/// Agent response before platform translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Agent ID
    pub agent_id: String,
    /// Response content (markdown String)
    pub content: String,
    /// Target platform
    pub target_platform: Platform,
    /// Target channel
    pub target_channel: String,
    /// Thread ID if replying in thread
    pub thread_id: Option<String>,
}

/// User identity across platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUser {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Display name (Option)
    pub display_name: Option<String>,
}

/// Attachment types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Attachment {
    /// Image attachment
    Image {
        /// URL to image
        url: String,
        /// Metadata (dimensions, size, etc.)
        metadata: serde_json::Value,
    },
    /// File attachment
    File {
        /// URL to file
        url: String,
        /// Metadata (filename, size, mime type, etc.)
        metadata: serde_json::Value,
    },
    /// Video attachment
    Video {
        /// URL to video
        url: String,
        /// Metadata (duration, size, codec, etc.)
        metadata: serde_json::Value,
    },
}

/// Platform-agnostic trait for messaging platform adapters
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// Unique adapter ID (e.g., "slack-main", "discord-prod")
    fn adapter_id(&self) -> &str;

    /// Platform type this adapter handles
    fn platform(&self) -> Platform;

    /// Start adapter (initiate outbound WebSocket/polling connection)
    async fn start(&mut self) -> Result<(), AofError>;

    /// Stop adapter gracefully (close connections, cleanup resources)
    async fn stop(&mut self) -> Result<(), AofError>;

    /// Health check (connection alive, authentication valid)
    async fn health_check(&self) -> Result<bool, AofError>;

    /// Receive next inbound message (blocks until message available)
    async fn receive_message(&mut self) -> Result<InboundMessage, AofError>;

    /// Send agent response to platform
    async fn send_message(&self, response: &AgentResponse) -> Result<(), AofError>;
}
