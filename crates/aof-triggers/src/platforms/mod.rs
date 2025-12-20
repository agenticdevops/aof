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

    /// Get as Any for downcasting (for platform-specific features)
    fn as_any(&self) -> &dyn std::any::Any;
}

// Platform-specific implementations
pub mod slack;
pub mod discord;
pub mod telegram;
pub mod whatsapp;
pub mod github;
pub mod gitlab;
pub mod bitbucket;
pub mod jira;

// Re-export platform types
pub use slack::{SlackConfig, SlackPlatform};
pub use discord::{DiscordConfig, DiscordPlatform};
pub use telegram::{TelegramConfig, TelegramPlatform};
pub use whatsapp::{WhatsAppConfig, WhatsAppPlatform};
pub use github::{GitHubConfig, GitHubPlatform};
pub use gitlab::{GitLabConfig, GitLabPlatform};
pub use bitbucket::{BitbucketConfig, BitbucketPlatform};
pub use jira::{JiraConfig, JiraPlatform};

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
///
/// This enum provides strongly-typed configuration for each supported platform.
/// To add a new platform:
/// 1. Create the platform module (e.g., `platforms/myplatform.rs`)
/// 2. Add it to this enum
/// 3. Register it in the PlatformRegistry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypedPlatformConfig {
    Slack(SlackConfig),
    Discord(DiscordConfig),
    Telegram(TelegramConfig),
    WhatsApp(WhatsAppConfig),
    GitHub(GitHubConfig),
    GitLab(GitLabConfig),
    Bitbucket(BitbucketConfig),
    Jira(JiraConfig),
}

// ============================================================================
// PLATFORM REGISTRY - Plugin System for Dynamic Platform Loading
// ============================================================================

/// Platform factory function type
///
/// Used by the registry to create platform instances from configuration.
pub type PlatformFactory = Box<dyn Fn(serde_json::Value) -> Result<Platform, PlatformError> + Send + Sync>;

/// Platform Registry for dynamic platform loading
///
/// The registry provides a pluggable architecture where new platforms can be
/// registered at runtime. This is the foundation for the extensibility system.
///
/// # Example
///
/// ```rust,ignore
/// use aof_triggers::platforms::{PlatformRegistry, TriggerPlatform};
///
/// let mut registry = PlatformRegistry::new();
///
/// // Built-in platforms are auto-registered
/// registry.register_defaults();
///
/// // Create platform from typed config
/// let platform = registry.create("slack", config_json)?;
///
/// // Register a custom platform
/// registry.register("myplatform", Box::new(|config| {
///     let cfg: MyConfig = serde_json::from_value(config)?;
///     Ok(Box::new(MyPlatform::new(cfg)?))
/// }));
/// ```
#[derive(Default)]
pub struct PlatformRegistry {
    factories: HashMap<String, PlatformFactory>,
}

impl PlatformRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Create a registry with all built-in platforms registered
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register all built-in platforms
    pub fn register_defaults(&mut self) {
        // Slack
        self.register("slack", Box::new(|config| {
            let cfg: SlackConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid Slack config: {}", e)))?;
            Ok(Box::new(SlackPlatform::new(cfg)?))
        }));

        // Telegram
        self.register("telegram", Box::new(|config| {
            let cfg: TelegramConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid Telegram config: {}", e)))?;
            Ok(Box::new(TelegramPlatform::new(cfg)?))
        }));

        // WhatsApp
        self.register("whatsapp", Box::new(|config| {
            let cfg: WhatsAppConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid WhatsApp config: {}", e)))?;
            Ok(Box::new(WhatsAppPlatform::new(cfg)?))
        }));

        // GitHub
        self.register("github", Box::new(|config| {
            let cfg: GitHubConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid GitHub config: {}", e)))?;
            Ok(Box::new(GitHubPlatform::new(cfg)?))
        }));

        // GitLab
        self.register("gitlab", Box::new(|config| {
            let cfg: GitLabConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid GitLab config: {}", e)))?;
            Ok(Box::new(GitLabPlatform::new(cfg)?))
        }));

        // Bitbucket
        self.register("bitbucket", Box::new(|config| {
            let cfg: BitbucketConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid Bitbucket config: {}", e)))?;
            Ok(Box::new(BitbucketPlatform::new(cfg)?))
        }));

        // Discord
        self.register("discord", Box::new(|config| {
            let cfg: DiscordConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid Discord config: {}", e)))?;
            Ok(Box::new(DiscordPlatform::from_discord_config(cfg)?))
        }));

        // Jira
        self.register("jira", Box::new(|config| {
            let cfg: JiraConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid Jira config: {}", e)))?;
            Ok(Box::new(JiraPlatform::new(cfg)?))
        }));
    }

    /// Register a new platform factory
    ///
    /// # Arguments
    /// * `name` - Platform identifier (lowercase, e.g., "slack", "github")
    /// * `factory` - Function that creates the platform from JSON config
    pub fn register(&mut self, name: &str, factory: PlatformFactory) {
        self.factories.insert(name.to_lowercase(), factory);
    }

    /// Create a platform instance from configuration
    ///
    /// # Arguments
    /// * `name` - Platform identifier
    /// * `config` - JSON configuration for the platform
    ///
    /// # Returns
    /// The created platform instance
    pub fn create(&self, name: &str, config: serde_json::Value) -> Result<Platform, PlatformError> {
        let factory = self.factories
            .get(&name.to_lowercase())
            .ok_or_else(|| PlatformError::ParseError(format!(
                "Unknown platform: {}. Available: {:?}",
                name,
                self.list_platforms()
            )))?;

        factory(config)
    }

    /// Check if a platform is registered
    pub fn has_platform(&self, name: &str) -> bool {
        self.factories.contains_key(&name.to_lowercase())
    }

    /// List all registered platform names
    pub fn list_platforms(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Unregister a platform
    pub fn unregister(&mut self, name: &str) -> bool {
        self.factories.remove(&name.to_lowercase()).is_some()
    }
}

/// Platform capability flags
///
/// These flags indicate what features a platform supports.
/// Used for capability-based routing and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformCapabilities {
    /// Supports threaded conversations
    pub threading: bool,
    /// Supports interactive elements (buttons, menus)
    pub interactive: bool,
    /// Supports file uploads/attachments
    pub files: bool,
    /// Supports reactions/emojis
    pub reactions: bool,
    /// Supports rich text/markdown
    pub rich_text: bool,
    /// Supports approval workflows
    pub approvals: bool,
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self {
            threading: false,
            interactive: false,
            files: false,
            reactions: false,
            rich_text: true,
            approvals: false,
        }
    }
}

/// Get capabilities for a known platform
pub fn get_platform_capabilities(platform: &str) -> PlatformCapabilities {
    match platform.to_lowercase().as_str() {
        "slack" => PlatformCapabilities {
            threading: true,
            interactive: true,
            files: true,
            reactions: true,
            rich_text: true,
            approvals: true,
        },
        "telegram" => PlatformCapabilities {
            threading: true, // reply chains
            interactive: true, // inline keyboards
            files: true,
            reactions: false,
            rich_text: true,
            approvals: true,
        },
        "whatsapp" => PlatformCapabilities {
            threading: false,
            interactive: true, // buttons, lists
            files: true,
            reactions: false,
            rich_text: false, // limited formatting
            approvals: true,
        },
        "github" => PlatformCapabilities {
            threading: true, // conversation threads
            interactive: true, // workflows, reactions
            files: true, // artifacts
            reactions: true,
            rich_text: true, // markdown
            approvals: true, // PR reviews
        },
        "gitlab" => PlatformCapabilities {
            threading: true, // conversation threads on MRs/issues
            interactive: true, // CI/CD pipelines, reactions
            files: true, // artifacts, releases
            reactions: true, // emojis on notes
            rich_text: true, // markdown
            approvals: true, // MR approvals
        },
        "bitbucket" => PlatformCapabilities {
            threading: true, // PR conversations
            interactive: false, // no interactive elements in comments
            files: true, // attachments
            reactions: false,
            rich_text: true, // markdown
            approvals: true, // PR approvals
        },
        "discord" => PlatformCapabilities {
            threading: true,
            interactive: true, // buttons, selects
            files: true,
            reactions: true,
            rich_text: true,
            approvals: false,
        },
        "jira" => PlatformCapabilities {
            threading: true, // conversation threads via comments
            interactive: true, // transitions, workflows
            files: true, // attachments
            reactions: false,
            rich_text: true, // Jira text format
            approvals: true, // issue workflows
        },
        _ => PlatformCapabilities::default(),
    }
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
