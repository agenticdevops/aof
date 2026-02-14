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
use aof_core::model::{MessageRole, Model, ModelRequest, RequestMessage};

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

    /// Handle a standup response from an agent
    ///
    /// Parses the structured response (DID/DOING/BLOCKERS format) and stores it
    /// in the collected_responses map for the given request_id.
    ///
    /// # Arguments
    ///
    /// * `request_id` - UUID of the standup request this is responding to
    /// * `agent_id` - ID of the agent submitting the response
    /// * `content` - Structured text response (DID/DOING/BLOCKERS format)
    /// * `token_count` - Number of tokens used by this response
    pub async fn handle_response(
        &self,
        request_id: &str,
        agent_id: &str,
        content: &str,
        token_count: u64,
    ) {
        debug!(
            "Received standup response from {} (request_id: {}, tokens: {})",
            agent_id, request_id, token_count
        );

        // Parse the structured response
        let record = parse_standup_response(agent_id, content, token_count);

        // Store in collected responses
        let mut responses = self.collected_responses.write().await;
        if let Some(response_list) = responses.get_mut(request_id) {
            response_list.push(record);
            debug!(
                "Stored response from {} ({} total responses for {})",
                agent_id,
                response_list.len(),
                request_id
            );
        } else {
            warn!(
                "Received response for unknown request_id: {} (from {})",
                request_id, agent_id
            );
        }
    }

    /// Get latest standup results
    ///
    /// Returns the most recent standup responses (if any).
    /// Used by REST API GET /api/coordination/standup/latest
    pub async fn latest_standup(&self) -> Vec<StandupResponseRecord> {
        // For now, return empty. In full implementation, we'd cache the last standup.
        // This will be enhanced in integration testing.
        vec![]
    }

    /// Generate a summary of standup responses using LLM
    ///
    /// Takes all collected responses for a standup and generates a human-readable
    /// prose summary using Sonnet (or provided model).
    ///
    /// This is feature-flagged via `config.summarize`. When disabled, responses
    /// are posted individually without aggregation.
    ///
    /// # Arguments
    ///
    /// * `responses` - All standup responses to summarize
    /// * `model` - Optional LLM model for summarization (Sonnet recommended)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(summary))` if summarization succeeds
    /// - `Ok(None)` if model is None or summarization disabled
    /// - `Err(...)` if LLM call fails
    pub async fn generate_summary(
        &self,
        responses: &[StandupResponseRecord],
        model: Option<&dyn Model>,
    ) -> Result<Option<String>, CoordinationProtocolError> {
        if !self.config.summarize {
            debug!("Summarization disabled in config");
            return Ok(None);
        }

        let Some(llm) = model else {
            warn!("Summarization enabled but no model provided");
            return Ok(None);
        };

        if responses.is_empty() {
            return Ok(Some("No standup responses received.".to_string()));
        }

        // Format responses into prompt
        let mut response_text = String::new();
        for record in responses {
            response_text.push_str(&format!(
                "{}: DID: {} | DOING: {} | BLOCKERS: {}\n",
                record.agent_id,
                record.what_i_did,
                record.what_im_doing,
                if record.blockers.is_empty() {
                    "none".to_string()
                } else {
                    record.blockers.join(", ")
                }
            ));
        }

        let prompt = format!(
            r#"Summarize this team standup in 2-3 paragraphs. Highlight key progress, active work, and any blockers that need attention.

Agent responses:
---
{}
---

Provide a concise, actionable summary focusing on:
1. What was accomplished (key wins)
2. What's in progress (current focus)
3. Blockers that need resolution"#,
            response_text
        );

        // Call LLM
        let request = ModelRequest {
            messages: vec![RequestMessage {
                role: MessageRole::User,
                content: prompt,
                tool_calls: None,
                tool_call_id: None,
            }],
            system: None,
            tools: Vec::new(),
            temperature: Some(0.7),
            max_tokens: Some(500),
            stream: false,
            extra: HashMap::new(),
        };

        let response = llm
            .generate(&request)
            .await
            .map_err(|e| CoordinationProtocolError::LlmError(e.to_string()))?;

        info!(
            "Generated standup summary ({} tokens: {} input + {} output)",
            response.usage.input_tokens + response.usage.output_tokens,
            response.usage.input_tokens,
            response.usage.output_tokens
        );

        Ok(Some(response.content))
    }
}

/// Parse a standup response from structured text
///
/// Extracts DID, DOING, and BLOCKERS fields from the response content.
/// Handles various formatting variations and missing fields gracefully.
fn parse_standup_response(
    agent_id: &str,
    content: &str,
    token_count: u64,
) -> StandupResponseRecord {
    let did = extract_field(content, "DID:");
    let doing = extract_field(content, "DOING:");
    let blockers_str = extract_field(content, "BLOCKERS:");

    // Parse blockers list
    let blockers = if blockers_str.to_lowercase().trim() == "none"
        || blockers_str.to_lowercase().trim() == "no response"
    {
        vec![]
    } else {
        blockers_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    StandupResponseRecord {
        agent_id: agent_id.to_string(),
        what_i_did: did,
        what_im_doing: doing,
        blockers,
        token_count,
        timestamp: Utc::now(),
    }
}

/// Extract a field value from structured text
///
/// Searches for lines starting with the given prefix (case-insensitive)
/// and returns the content after the prefix.
///
/// # Arguments
///
/// * `content` - The full response text
/// * `prefix` - The field prefix to search for (e.g., "DID:", "DOING:")
///
/// # Returns
///
/// The field value, or "No response" if not found
fn extract_field(content: &str, prefix: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.to_uppercase().starts_with(&prefix.to_uppercase()) {
            return trimmed[prefix.len()..].trim().to_string();
        }
    }
    "No response".to_string()
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

    #[test]
    fn test_parse_standup_response_clean() {
        let content = r#"
DID: Fixed authentication bug
DOING: Working on API endpoint
BLOCKERS: none
"#;
        let record = parse_standup_response("agent-1", content, 150);

        assert_eq!(record.agent_id, "agent-1");
        assert_eq!(record.what_i_did, "Fixed authentication bug");
        assert_eq!(record.what_im_doing, "Working on API endpoint");
        assert_eq!(record.blockers.len(), 0);
        assert_eq!(record.token_count, 150);
    }

    #[test]
    fn test_parse_standup_response_with_blockers() {
        let content = r#"
DID: Deployed to staging
DOING: Code review
BLOCKERS: Need database access, Waiting for PR approval
"#;
        let record = parse_standup_response("agent-2", content, 180);

        assert_eq!(record.agent_id, "agent-2");
        assert_eq!(record.what_i_did, "Deployed to staging");
        assert_eq!(record.what_im_doing, "Code review");
        assert_eq!(record.blockers.len(), 2);
        assert_eq!(record.blockers[0], "Need database access");
        assert_eq!(record.blockers[1], "Waiting for PR approval");
    }

    #[test]
    fn test_parse_standup_response_no_blockers() {
        let content = r#"
DID: Completed test suite
DOING: Documentation updates
BLOCKERS: No response
"#;
        let record = parse_standup_response("agent-3", content, 120);

        assert_eq!(record.blockers.len(), 0); // "No response" treated as empty
    }

    #[test]
    fn test_parse_standup_response_malformed() {
        let content = "Some random text without proper format";
        let record = parse_standup_response("agent-4", content, 50);

        assert_eq!(record.agent_id, "agent-4");
        assert_eq!(record.what_i_did, "No response");
        assert_eq!(record.what_im_doing, "No response");
        assert_eq!(record.blockers.len(), 0);
    }

    #[test]
    fn test_extract_field_case_insensitive() {
        let content = r#"
did: Task A
DOING: Task B
Blockers: none
"#;
        assert_eq!(extract_field(content, "DID:"), "Task A");
        assert_eq!(extract_field(content, "DOING:"), "Task B");
        assert_eq!(extract_field(content, "BLOCKERS:"), "none");
    }

    #[test]
    fn test_extract_field_missing() {
        let content = "DID: Something\nDOING: Something else";
        assert_eq!(extract_field(content, "BLOCKERS:"), "No response");
    }

    #[tokio::test]
    async fn test_handle_response_stores_record() {
        let config = StandupConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        // Initialize a standup request
        let request_id = "test-request-123";
        scheduler
            .collected_responses
            .write()
            .await
            .insert(request_id.to_string(), Vec::new());

        // Handle a response
        let content = "DID: Task 1\nDOING: Task 2\nBLOCKERS: none";
        scheduler
            .handle_response(request_id, "agent-1", content, 150)
            .await;

        // Verify stored
        let responses = scheduler.collected_responses.read().await;
        let response_list = responses.get(request_id).unwrap();
        assert_eq!(response_list.len(), 1);
        assert_eq!(response_list[0].agent_id, "agent-1");
        assert_eq!(response_list[0].what_i_did, "Task 1");
    }

    #[tokio::test]
    async fn test_handle_response_unknown_request() {
        let config = StandupConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        // Handle response for non-existent request (should log warning, not panic)
        let content = "DID: Task 1\nDOING: Task 2\nBLOCKERS: none";
        scheduler
            .handle_response("unknown-request", "agent-1", content, 150)
            .await;

        // Verify nothing stored
        let responses = scheduler.collected_responses.read().await;
        assert!(!responses.contains_key("unknown-request"));
    }

    // Mock model for testing summarization
    use aof_core::{AofResult, ModelProvider, ModelResponse, StopReason, Usage, StreamChunk};
    use async_trait::async_trait;
    use futures::Stream;
    use std::pin::Pin;

    struct MockModel {
        response: String,
    }

    #[async_trait]
    impl Model for MockModel {
        async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
            Ok(ModelResponse {
                content: self.response.clone(),
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 200,
                    output_tokens: 100,
                },
                tool_calls: Vec::new(),
                metadata: HashMap::new(),
            })
        }

        async fn generate_stream(
            &self,
            _request: &ModelRequest,
        ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
            unimplemented!()
        }

        fn config(&self) -> &aof_core::ModelConfig {
            unimplemented!()
        }

        fn provider(&self) -> ModelProvider {
            ModelProvider::Anthropic
        }
    }

    #[tokio::test]
    async fn test_generate_summary_disabled() {
        let config = StandupConfig {
            summarize: false,
            ..Default::default()
        };
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        let responses = vec![
            StandupResponseRecord {
                agent_id: "agent-1".to_string(),
                what_i_did: "Task A".to_string(),
                what_im_doing: "Task B".to_string(),
                blockers: vec![],
                token_count: 100,
                timestamp: Utc::now(),
            },
        ];

        let model = MockModel {
            response: "Summary text".to_string(),
        };

        let result = scheduler.generate_summary(&responses, Some(&model)).await.unwrap();
        assert!(result.is_none()); // Summarization disabled
    }

    #[tokio::test]
    async fn test_generate_summary_no_model() {
        let mut config = StandupConfig::default();
        config.summarize = true;

        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        let responses = vec![
            StandupResponseRecord {
                agent_id: "agent-1".to_string(),
                what_i_did: "Task A".to_string(),
                what_im_doing: "Task B".to_string(),
                blockers: vec![],
                token_count: 100,
                timestamp: Utc::now(),
            },
        ];

        let result = scheduler.generate_summary(&responses, None).await.unwrap();
        assert!(result.is_none()); // No model provided
    }

    #[tokio::test]
    async fn test_generate_summary_empty_responses() {
        let mut config = StandupConfig::default();
        config.summarize = true;

        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        let model = MockModel {
            response: "Summary text".to_string(),
        };

        let result = scheduler.generate_summary(&[], Some(&model)).await.unwrap();
        assert_eq!(result, Some("No standup responses received.".to_string()));
    }

    #[tokio::test]
    async fn test_generate_summary_success() {
        let mut config = StandupConfig::default();
        config.summarize = true;

        let (tx, _rx) = broadcast::channel(100);
        let scheduler = StandupScheduler::new(config, tx, "test-session");

        let responses = vec![
            StandupResponseRecord {
                agent_id: "agent-1".to_string(),
                what_i_did: "Fixed auth bug".to_string(),
                what_im_doing: "Working on API".to_string(),
                blockers: vec![],
                token_count: 150,
                timestamp: Utc::now(),
            },
            StandupResponseRecord {
                agent_id: "agent-2".to_string(),
                what_i_did: "Deployed to staging".to_string(),
                what_im_doing: "Code review".to_string(),
                blockers: vec!["Need DB access".to_string()],
                token_count: 180,
                timestamp: Utc::now(),
            },
        ];

        let model = MockModel {
            response: "Team made good progress. Agent-1 fixed auth bug and is working on API. Agent-2 deployed to staging but needs DB access.".to_string(),
        };

        let result = scheduler.generate_summary(&responses, Some(&model)).await.unwrap();
        assert!(result.is_some());
        let summary = result.unwrap();
        assert!(summary.contains("progress"));
        assert!(summary.contains("Agent-1"));
        assert!(summary.contains("Agent-2"));
    }
}
