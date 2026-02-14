//! Squad broadcast functionality
//!
//! This module implements one-to-many broadcast patterns for squad announcements.

use serde::{Deserialize, Serialize};

use crate::adapters::Platform;

/// Broadcast message (one-to-many announcement)
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

    /// Source channel ID (optional, for reply-to)
    pub source_channel: Option<String>,
}

/// Broadcast target (who receives the message)
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

/// Message priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Broadcast result
#[derive(Debug)]
pub struct BroadcastResult {
    /// Number of messages sent successfully
    pub sent_count: usize,

    /// Channels that failed to receive (platform, channel_id)
    pub failed_channels: Vec<(Platform, String)>,
}
