// Multi-channel gateway types for OpenAgentiX (Phase 21)
//
// Provides channel configuration, routing, notification, and the ChannelGateway trait
// for bi-directional communication with Slack, Telegram, and Discord.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::error::AgentixError;

/// Supported channel platforms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelPlatformType {
    Slack,
    Telegram,
    Discord,
}

impl fmt::Display for ChannelPlatformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelPlatformType::Slack => write!(f, "slack"),
            ChannelPlatformType::Telegram => write!(f, "telegram"),
            ChannelPlatformType::Discord => write!(f, "discord"),
        }
    }
}

/// Platform-specific credentials for channel authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", rename_all = "lowercase")]
pub enum ChannelCredentials {
    Slack {
        bot_token: String,
        signing_secret: String,
        app_id: String,
    },
    Telegram {
        bot_token: String,
        webhook_secret: Option<String>,
    },
    Discord {
        bot_token: String,
        application_id: String,
        public_key: String,
    },
}

/// Direction of communication for a channel route
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

impl Default for ChannelDirection {
    fn default() -> Self {
        Self::Bidirectional
    }
}

/// Maps a channel to an agent with a specific communication direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRoute {
    pub channel_id: String,
    pub agent_name: String,
    #[serde(default)]
    pub direction: ChannelDirection,
    pub description: Option<String>,
}

impl ChannelRoute {
    /// Returns true when the given agent_name matches this route AND the direction
    /// allows inbound messages (Inbound or Bidirectional).
    pub fn matches_agent(&self, agent_name: &str) -> bool {
        self.agent_name == agent_name
            && matches!(
                self.direction,
                ChannelDirection::Inbound | ChannelDirection::Bidirectional
            )
    }

    /// Returns true when the given agent_name matches this route AND the direction
    /// allows outbound messages (Outbound or Bidirectional).
    pub fn allows_outbound(&self, agent_name: &str) -> bool {
        self.agent_name == agent_name
            && matches!(
                self.direction,
                ChannelDirection::Outbound | ChannelDirection::Bidirectional
            )
    }
}

fn default_enabled() -> bool {
    true
}

/// Configuration for a single channel platform connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub platform: ChannelPlatformType,
    pub credentials: ChannelCredentials,
    #[serde(default)]
    pub routes: Vec<ChannelRoute>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Target for an outbound notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTarget {
    pub platform: ChannelPlatformType,
    pub channel_id: String,
    pub thread_id: Option<String>,
}

/// Severity level for notifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for NotificationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationSeverity::Info => write!(f, "info"),
            NotificationSeverity::Warning => write!(f, "warning"),
            NotificationSeverity::Error => write!(f, "error"),
            NotificationSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Payload for outbound agent notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub agent_name: String,
    pub title: String,
    pub body: String,
    pub severity: NotificationSeverity,
    pub run_id: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A message received from a channel platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub platform: ChannelPlatformType,
    pub channel_id: String,
    pub user_id: String,
    pub user_name: String,
    pub text: String,
    pub thread_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub raw_payload: serde_json::Value,
}

/// Trait for channel gateway adapters (Slack, Telegram, Discord)
#[async_trait]
pub trait ChannelGateway: Send + Sync {
    /// Send a text message to a channel, optionally in a thread
    async fn send_message(
        &self,
        channel_id: &str,
        text: &str,
        thread_id: Option<&str>,
    ) -> Result<(), AgentixError>;

    /// Send a structured notification to a target channel
    async fn send_notification(
        &self,
        target: &NotificationTarget,
        payload: &NotificationPayload,
    ) -> Result<(), AgentixError>;

    /// Returns the platform type this gateway handles
    fn platform_type(&self) -> ChannelPlatformType;
}
