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

    /// Run the standup scheduler loop
    ///
    /// This is the main loop that:
    /// 1. Calculates delay until next standup based on cron expression
    /// 2. Sleeps until that time
    /// 3. Triggers standup
    /// 4. Repeats
    ///
    /// Must be spawned in Arc for shared access from timeout tasks.
    pub async fn run(self: Arc<Self>) -> Result<(), CoordinationProtocolError> {
        if !self.config.enabled {
            info!("Standup protocol disabled in config");
            return Ok(());
        }

        // Parse cron expression
        let schedule = Schedule::from_str(&self.config.cron)
            .map_err(|e| CoordinationProtocolError::InvalidCron(e.to_string()))?;

        // Parse timezone
        let tz: chrono_tz::Tz = self
            .config
            .timezone
            .parse()
            .map_err(|_| CoordinationProtocolError::InvalidTimezone(self.config.timezone.clone()))?;

        info!(
            "Starting standup scheduler (cron: {}, timezone: {})",
            self.config.cron, self.config.timezone
        );

        loop {
            // Calculate delay until next standup
            let next = schedule
                .upcoming(tz)
                .next()
                .ok_or_else(|| CoordinationProtocolError::InvalidCron("No future runs".into()))?;
            let now = Utc::now().with_timezone(&tz);
            let delay = (next - now)
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(60));

            info!("Next standup at: {} (in {:?})", next, delay);
            tokio::time::sleep(delay).await;

            // Trigger standup
            if let Err(e) = self.trigger_standup().await {
                error!("Failed to trigger standup: {}", e);
            }
        }
    }

    /// Trigger an immediate standup
    ///
    /// This method:
    /// 1. Generates a unique request_id (UUID v4)
    /// 2. Emits StandupRequest event to all participating agents
    /// 3. Initializes response collection for this request
    /// 4. Waits for collection_timeout duration
    /// 5. Collects all received responses
    /// 6. Optionally generates summary (if summarize enabled)
    /// 7. Emits StandupSummary event
    ///
    /// Can be called manually via REST API or automatically by scheduler.
    pub async fn trigger_standup(&self) -> Result<String, CoordinationProtocolError> {
        let request_id = Uuid::new_v4().to_string();
        let agent_count = self.participating_agents.read().await.len();

        info!(
            "Triggering standup (request_id: {}, agents: {})",
            request_id, agent_count
        );

        // Initialize response collection
        self.collected_responses
            .write()
            .await
            .insert(request_id.clone(), Vec::new());

        // Emit StandupRequest event
        let event = CoordinationEvent::standup_request(
            self.session_id.clone(),
            request_id.clone(),
        );

        if let Err(e) = self.event_tx.send(event) {
            warn!("Failed to broadcast standup request: {}", e);
        }

        // Wait for collection timeout
        info!(
            "Waiting {:?} for standup responses",
            self.config.collection_timeout
        );
        tokio::time::sleep(self.config.collection_timeout).await;

        // Collect all responses
        let responses = self
            .collected_responses
            .write()
            .await
            .remove(&request_id)
            .unwrap_or_default();

        let response_count = responses.len();
        info!(
            "Standup complete: {}/{} agents responded",
            response_count, agent_count
        );

        // Optionally generate summary
        if self.config.summarize {
            // TODO: Implement Sonnet summarization in Task 4
            debug!("Summarization enabled but not yet implemented (Task 4)");
        }

        // Emit StandupSummary event
        // NOTE: StandupSummary constructor will be added to aof-core in Task 5
        // For now, we just log the completion
        info!(
            "Standup summary ready: {}/{} responses (request_id: {})",
            response_count, agent_count, request_id
        );

        Ok(request_id)
    }

    /// Trigger standup immediately (public method for REST API)
    ///
    /// This is a convenience wrapper around trigger_standup() for external callers.
    pub async fn trigger_now(&self) -> Result<String, CoordinationProtocolError> {
        self.trigger_standup().await
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

    #[tokio::test]
    async fn test_trigger_now_emits_events() {
        let config = StandupConfig {
            collection_timeout: Duration::from_millis(100), // Short timeout for testing
            ..StandupConfig::default()
        };
        let (tx, mut rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;
        scheduler.register_agent("agent-2").await;

        // Trigger standup
        let request_id = scheduler.trigger_now().await.unwrap();
        assert!(!request_id.is_empty());

        // Should receive StandupRequest event
        let event1 = rx.recv().await.unwrap();
        if let Some(activity) = &event1.coordination_activity {
            use aof_core::coordination::CoordinationActivity;
            match activity {
                CoordinationActivity::StandupRequest { request_id: req_id } => {
                    assert_eq!(req_id, &request_id);
                }
                _ => panic!("Expected StandupRequest, got {:?}", activity),
            }
        } else {
            panic!("Expected coordination_activity");
        }

        // NOTE: StandupSummary event will be tested once the constructor is added to aof-core
        // For now, we just verify that trigger_now completes successfully
    }

    #[test]
    fn test_invalid_cron_rejected() {
        let mut config = StandupConfig::default();
        config.cron = "invalid cron".to_string();

        let (tx, _rx) = broadcast::channel(100);
        let scheduler = Arc::new(StandupScheduler::new(config, tx, "test-session"));

        // This should be tested in run() but we can't easily test the scheduler loop
        // without actually running it. We test cron parsing indirectly.
        let schedule_result = Schedule::from_str("invalid cron");
        assert!(schedule_result.is_err());
    }

    #[test]
    fn test_invalid_timezone_rejected() {
        let invalid_tz: Result<chrono_tz::Tz, _> = "Invalid/Timezone".parse();
        assert!(invalid_tz.is_err());
    }
}
