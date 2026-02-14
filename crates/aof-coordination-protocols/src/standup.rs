//! Standup Protocol - Daily agent status reports
//!
//! The standup protocol implements daily team standups where agents report:
//! - **DID**: What they accomplished since last standup
//! - **DOING**: What they are working on today
//! - **BLOCKERS**: Current blockers/issues
//!
//! # Architecture
//!
//! ```text
//! StandupScheduler (cron: daily 9am)
//!     |
//!     v
//! StandupRequest event -> broadcast to participating agents
//!     |
//!     v
//! Agents respond with structured DID/DOING/BLOCKERS (Haiku, ~200 tokens)
//!     |
//!     v (5-minute collection window)
//! StandupScheduler::handle_response() collects records
//!     |
//!     v (optional)
//! Sonnet summarization -> StandupSummary event
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_coordination_protocols::standup::{StandupScheduler, StandupConfig};
//! use tokio::sync::broadcast;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = StandupConfig::default();
//!     let (event_tx, _) = broadcast::channel(1000);
//!     let session_id = "session-123".to_string();
//!
//!     let scheduler = StandupScheduler::new(config, event_tx, session_id);
//!
//!     // Register agents
//!     scheduler.register_agent("k8s-monitor").await;
//!     scheduler.register_agent("log-analyzer").await;
//!
//!     // Run in background
//!     tokio::spawn(async move {
//!         scheduler.run().await
//!     });
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use aof_core::coordination::CoordinationEvent;

use crate::error::CoordinationProtocolError;

/// Standup configuration
#[derive(Debug, Clone)]
pub struct StandupConfig {
    /// Cron expression (default: "0 0 9 * * *" for daily 9am)
    pub cron: String,
    /// IANA timezone (e.g., "America/New_York"), default: "UTC"
    pub timezone: String,
    /// Enable Sonnet summarization of all standup responses
    pub summarize: bool,
    /// Response collection timeout (how long to wait for agent responses)
    pub collection_timeout: Duration,
    /// Maximum tokens per agent standup response
    pub max_response_tokens: usize,
    /// Whether standup is enabled
    pub enabled: bool,
}

impl Default for StandupConfig {
    fn default() -> Self {
        Self {
            cron: "0 0 9 * * *".to_string(), // Daily at 9am
            timezone: "UTC".to_string(),
            summarize: false,
            collection_timeout: Duration::from_secs(5 * 60), // 5 minutes
            max_response_tokens: 200,
            enabled: true,
        }
    }
}

/// Standup response record from a single agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandupResponseRecord {
    /// Agent ID
    pub agent_id: String,
    /// What the agent did since last standup
    pub what_i_did: String,
    /// What the agent is doing today
    pub what_im_doing: String,
    /// Current blockers
    pub blockers: Vec<String>,
    /// Token count for this response
    pub token_count: u64,
    /// When the response was received
    pub timestamp: DateTime<Utc>,
}

/// Standup scheduler - manages daily standup protocol
///
/// The scheduler:
/// 1. Triggers standups at configured cron time (default: daily 9am)
/// 2. Broadcasts StandupRequest event to participating agents
/// 3. Collects StandupResponse events over collection window (default: 5 minutes)
/// 4. Optionally generates Sonnet summary of all responses
/// 5. Broadcasts StandupSummary event to squad chat
///
/// Token efficiency:
/// - Haiku for agent responses (~180 tokens each, cheap)
/// - Optional Sonnet for summary (~500 tokens, feature-flagged)
pub struct StandupScheduler {
    config: StandupConfig,
    event_tx: broadcast::Sender<CoordinationEvent>,
    session_id: String,
    /// Agents participating in standups
    participating_agents: Arc<RwLock<HashSet<String>>>,
    /// Collected responses for current standup: request_id -> responses
    collected_responses: Arc<RwLock<HashMap<String, Vec<StandupResponseRecord>>>>,
}

impl StandupScheduler {
    /// Create a new standup scheduler
    pub fn new(
        config: StandupConfig,
        event_tx: broadcast::Sender<CoordinationEvent>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            config,
            event_tx,
            session_id: session_id.into(),
            participating_agents: Arc::new(RwLock::new(HashSet::new())),
            collected_responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent to participate in standups
    pub async fn register_agent(&self, agent_id: impl Into<String>) {
        let agent_id_str = agent_id.into();
        debug!("Registering agent {} for standups", agent_id_str);
        self.participating_agents
            .write()
            .await
            .insert(agent_id_str);
    }

    /// Unregister an agent from standups
    pub async fn unregister_agent(&self, agent_id: &str) {
        debug!("Unregistering agent {} from standups", agent_id);
        self.participating_agents.write().await.remove(agent_id);
    }

    /// Get list of participating agents
    pub async fn participating_agents(&self) -> Vec<String> {
        self.participating_agents
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    /// Get the standup prompt template for agents
    ///
    /// Returns structured prompt that agents should respond to.
    /// Keeps responses concise and predictable for parsing.
    pub fn get_standup_prompt(agent_id: &str) -> String {
        format!(
            r#"You are {agent_id}. Answer these three questions concisely (max 50 words each):

1. What did you do since yesterday's standup?
2. What are you working on today?
3. Do you have any blockers?

Respond in this exact format:
DID: <your answer>
DOING: <your answer>
BLOCKERS: <your answer or "none">

Be brief and specific. Focus on results, not process."#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standup_config_defaults() {
        let config = StandupConfig::default();
        assert_eq!(config.cron, "0 0 9 * * *");
        assert_eq!(config.timezone, "UTC");
        assert!(!config.summarize);
        assert_eq!(config.collection_timeout, Duration::from_secs(5 * 60));
        assert_eq!(config.max_response_tokens, 200);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_standup_scheduler_creation() {
        let config = StandupConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config.clone(), tx, "test-session");

        assert_eq!(scheduler.session_id, "test-session");
        assert_eq!(scheduler.config.cron, "0 0 9 * * *");
        assert!(scheduler.participating_agents.read().await.is_empty());
        assert!(scheduler.collected_responses.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_and_list_agents() {
        let config = StandupConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;
        scheduler.register_agent("agent-2").await;
        scheduler.register_agent("agent-3").await;

        let agents = scheduler.participating_agents().await;
        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"agent-1".to_string()));
        assert!(agents.contains(&"agent-2".to_string()));
        assert!(agents.contains(&"agent-3".to_string()));
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let config = StandupConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;
        scheduler.register_agent("agent-2").await;

        scheduler.unregister_agent("agent-1").await;

        let agents = scheduler.participating_agents().await;
        assert_eq!(agents.len(), 1);
        assert!(!agents.contains(&"agent-1".to_string()));
        assert!(agents.contains(&"agent-2".to_string()));
    }

    #[test]
    fn test_standup_prompt_template() {
        let prompt = StandupScheduler::get_standup_prompt("test-agent");
        assert!(prompt.contains("You are test-agent"));
        assert!(prompt.contains("DID:"));
        assert!(prompt.contains("DOING:"));
        assert!(prompt.contains("BLOCKERS:"));
        assert!(prompt.contains("max 50 words each"));
    }
}
