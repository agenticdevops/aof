//! Heartbeat protocol - Proactive health monitoring for agents
//!
//! The heartbeat scheduler sends periodic "alive?" checks to all registered agents
//! and tracks their health status. Unresponsive agents trigger timeout alerts.
//!
//! # Architecture
//!
//! - **Frequency:** 60 seconds (configurable via serve-config.yaml, default 60s)
//! - **Timeout:** 120 seconds (2x interval for LLM-based agents)
//! - **Model:** Claude Haiku for cheap health checks (~50 tokens per call)
//! - **Prompt:** Static "Are you alive?" (NO context loading - no AGENTS.md, SOUL.md, memories)
//! - **Cost:** ~$0.01/day for 10 agents @ 60s frequency
//!
//! # Token Efficiency
//!
//! Heartbeat is designed to be super-lightweight:
//! - No context loading (no agent config, no memories)
//! - Static prompt: "Are you alive?"
//! - Haiku model (cheapest Claude variant)
//! - Tokens tracked separately for visibility in metrics
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_coordination_protocols::heartbeat::{HeartbeatScheduler, HeartbeatConfig};
//! use tokio::sync::broadcast;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = HeartbeatConfig::default(); // 60s frequency, 120s timeout
//!     let (event_tx, _) = broadcast::channel(1000);
//!     let session_id = "session-123".to_string();
//!
//!     let scheduler = HeartbeatScheduler::new(config, event_tx, session_id);
//!     scheduler.register_agent("k8s-monitor").await;
//!
//!     // Run scheduler in background
//!     tokio::spawn(async move {
//!         scheduler.run().await
//!     });
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use aof_core::coordination::CoordinationEvent;

use crate::error::CoordinationProtocolError;
use crate::events::AgentHealthStatus;

/// Heartbeat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// How often to send heartbeat requests
    pub frequency: Duration,
    /// How long to wait before marking agent unresponsive
    pub timeout: Duration,
    /// Whether heartbeat protocol is enabled
    pub enabled: bool,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            frequency: Duration::from_secs(60), // 60 seconds (was 30s in earlier drafts)
            timeout: Duration::from_secs(120),  // 120 seconds (2x interval)
            enabled: true,
        }
    }
}

/// Pending heartbeat request tracking
#[derive(Debug, Clone)]
struct PendingHeartbeat {
    /// When request was sent (for debugging/metrics)
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
    /// Agents expected to respond
    expected_agents: HashSet<String>,
    /// Agents that have responded
    responded_agents: HashSet<String>,
}

/// Agent health record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthRecord {
    /// Agent identifier
    pub agent_id: String,
    /// Current health status
    pub status: AgentHealthStatus,
    /// Last successful heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Number of consecutive missed heartbeats
    pub consecutive_misses: u32,
    /// Last response time in milliseconds
    pub last_response_ms: Option<u64>,
}

/// Heartbeat scheduler - sends periodic health checks to agents
///
/// Uses tokio::time::interval for periodic ticks. On each tick:
/// 1. Generate request_id (UUID v4)
/// 2. Emit HeartbeatRequest event via broadcast
/// 3. Track pending request with expected agents
/// 4. Spawn timeout checker task
///
/// When responses arrive via handle_response():
/// - Update agent health record
/// - Reset consecutive_misses counter
/// - Mark agent as responded
///
/// When timeout expires via check_timeout():
/// - Identify unresponsive agents
/// - Increment consecutive_misses
/// - Emit HeartbeatTimeout alert
pub struct HeartbeatScheduler {
    config: HeartbeatConfig,
    event_tx: broadcast::Sender<CoordinationEvent>,
    session_id: String,
    /// Track pending requests: request_id -> PendingHeartbeat
    pending_requests: Arc<RwLock<HashMap<String, PendingHeartbeat>>>,
    /// Track agent health: agent_id -> AgentHealthRecord
    agent_health: Arc<RwLock<HashMap<String, AgentHealthRecord>>>,
    /// Registered agents (only agents with heartbeat-enabled coordination mode)
    registered_agents: Arc<RwLock<HashSet<String>>>,
}

impl HeartbeatScheduler {
    /// Create a new heartbeat scheduler
    pub fn new(
        config: HeartbeatConfig,
        event_tx: broadcast::Sender<CoordinationEvent>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            config,
            event_tx,
            session_id: session_id.into(),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            agent_health: Arc::new(RwLock::new(HashMap::new())),
            registered_agents: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register an agent for heartbeat monitoring
    ///
    /// Only agents with coordination_mode full, standard, reduced, or heartbeat_only
    /// should be registered. Disabled agents are excluded.
    pub async fn register_agent(&self, agent_id: impl Into<String>) {
        let agent_id_str = agent_id.into();
        debug!("Registering agent for heartbeat: {}", agent_id_str);

        // Add to registered set
        self.registered_agents.write().await.insert(agent_id_str.clone());

        // Initialize health record
        let record = AgentHealthRecord {
            agent_id: agent_id_str.clone(),
            status: AgentHealthStatus::Healthy,
            last_heartbeat: None,
            consecutive_misses: 0,
            last_response_ms: None,
        };
        self.agent_health.write().await.insert(agent_id_str, record);
    }

    /// Get current health status for all agents
    pub async fn agent_health_snapshot(&self) -> Vec<AgentHealthRecord> {
        self.agent_health.read().await.values().cloned().collect()
    }

    /// Handle a heartbeat response from an agent
    ///
    /// Updates agent health record, resets consecutive_misses, and marks
    /// agent as responded for the pending request.
    pub async fn handle_response(
        &self,
        request_id: impl Into<String>,
        agent_id: impl Into<String>,
        response_time_ms: u64,
    ) {
        let request_id_str = request_id.into();
        let agent_id_str = agent_id.into();

        debug!(
            "Heartbeat response from {} for request {} ({}ms)",
            agent_id_str, request_id_str, response_time_ms
        );

        // Update agent health
        if let Some(record) = self.agent_health.write().await.get_mut(&agent_id_str) {
            record.status = AgentHealthStatus::Healthy;
            record.last_heartbeat = Some(Utc::now());
            record.consecutive_misses = 0;
            record.last_response_ms = Some(response_time_ms);
        }

        // Mark as responded in pending request
        if let Some(pending) = self.pending_requests.write().await.get_mut(&request_id_str) {
            pending.responded_agents.insert(agent_id_str.clone());

            // If all agents responded, remove pending request
            if pending.responded_agents.len() == pending.expected_agents.len() {
                debug!(
                    "All agents responded for request {}, cleaning up",
                    request_id_str
                );
                self.pending_requests.write().await.remove(&request_id_str);
            }
        } else {
            warn!(
                "Received heartbeat response for unknown request_id: {}",
                request_id_str
            );
        }
    }

    /// Check for timeout and emit alerts for unresponsive agents
    ///
    /// Called after timeout duration expires. Identifies agents that did NOT
    /// respond to the pending request, increments consecutive_misses, and
    /// emits HeartbeatTimeout event.
    async fn check_timeout(&self, request_id: String) {
        debug!("Checking timeout for heartbeat request: {}", request_id);

        let mut pending_requests = self.pending_requests.write().await;
        if let Some(pending) = pending_requests.remove(&request_id) {
            let unresponsive: Vec<String> = pending
                .expected_agents
                .difference(&pending.responded_agents)
                .cloned()
                .collect();

            if !unresponsive.is_empty() {
                warn!(
                    "Heartbeat timeout: {} agents unresponsive for request {}",
                    unresponsive.len(),
                    request_id
                );

                // Update health records for unresponsive agents
                let mut agent_health = self.agent_health.write().await;
                for agent_id in &unresponsive {
                    if let Some(record) = agent_health.get_mut(agent_id) {
                        record.consecutive_misses += 1;
                        record.status = AgentHealthStatus::Unresponsive;
                        info!(
                            "Agent {} marked unresponsive (consecutive misses: {})",
                            agent_id, record.consecutive_misses
                        );
                    }
                }

                // Emit timeout event
                let event =
                    CoordinationEvent::heartbeat_timeout(&self.session_id, &request_id, unresponsive);
                if let Err(e) = self.event_tx.send(event) {
                    debug!("No subscribers for heartbeat timeout event: {}", e);
                }
            }
        } else {
            debug!(
                "Timeout check for request {} - already cleaned up (all agents responded)",
                request_id
            );
        }
    }

    /// Run the heartbeat scheduler
    ///
    /// This is the main loop that runs indefinitely. Uses tokio::time::interval
    /// for periodic ticks at config.frequency. On each tick:
    /// 1. Generate request_id
    /// 2. Record pending request with expected agents
    /// 3. Emit HeartbeatRequest event
    /// 4. Spawn timeout checker task
    ///
    /// This method should be spawned as a tokio task.
    pub async fn run(self: Arc<Self>) -> Result<(), CoordinationProtocolError> {
        if !self.config.enabled {
            info!("Heartbeat scheduler disabled in config");
            return Ok(());
        }

        info!(
            "Starting heartbeat scheduler (frequency: {:?}, timeout: {:?})",
            self.config.frequency, self.config.timeout
        );

        let mut interval = tokio::time::interval(self.config.frequency);

        loop {
            interval.tick().await;

            // Generate request ID
            let request_id = Uuid::new_v4().to_string();
            debug!("Heartbeat tick: request_id = {}", request_id);

            // Get registered agents
            let expected_agents: HashSet<String> =
                self.registered_agents.read().await.iter().cloned().collect();

            if expected_agents.is_empty() {
                debug!("No agents registered for heartbeat, skipping tick");
                continue;
            }

            // Record pending request
            let pending = PendingHeartbeat {
                timestamp: Utc::now(),
                expected_agents: expected_agents.clone(),
                responded_agents: HashSet::new(),
            };
            self.pending_requests
                .write()
                .await
                .insert(request_id.clone(), pending);

            // Emit HeartbeatRequest event
            let event = CoordinationEvent::heartbeat_request(&self.session_id, &request_id);
            if let Err(e) = self.event_tx.send(event) {
                debug!("No subscribers for heartbeat request event: {}", e);
            }

            // Spawn timeout checker
            let scheduler_clone = Arc::clone(&self);
            let timeout_duration = self.config.timeout;
            let request_id_clone = request_id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(timeout_duration).await;
                scheduler_clone.check_timeout(request_id_clone).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time;

    #[tokio::test]
    async fn test_heartbeat_config_default() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.frequency, Duration::from_secs(60));
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_heartbeat_scheduler_creation() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = HeartbeatScheduler::new(config.clone(), tx, "test-session");

        assert_eq!(scheduler.session_id, "test-session");
        assert_eq!(scheduler.config.frequency, config.frequency);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = HeartbeatScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;

        let agents = scheduler.registered_agents.read().await;
        assert!(agents.contains("agent-1"));

        let health = scheduler.agent_health.read().await;
        assert!(health.contains_key("agent-1"));
        assert_eq!(health.get("agent-1").unwrap().consecutive_misses, 0);
    }

    #[tokio::test]
    async fn test_handle_response_updates_health() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = HeartbeatScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;

        // Simulate pending request
        let request_id = "req-123";
        let mut expected = HashSet::new();
        expected.insert("agent-1".to_string());
        let pending = PendingHeartbeat {
            timestamp: Utc::now(),
            expected_agents: expected,
            responded_agents: HashSet::new(),
        };
        scheduler
            .pending_requests
            .write()
            .await
            .insert(request_id.to_string(), pending);

        // Handle response
        scheduler.handle_response(request_id, "agent-1", 1200).await;

        // Check health updated
        let health = scheduler.agent_health.read().await;
        let record = health.get("agent-1").unwrap();
        assert!(matches!(record.status, AgentHealthStatus::Healthy));
        assert!(record.last_heartbeat.is_some());
        assert_eq!(record.consecutive_misses, 0);
        assert_eq!(record.last_response_ms, Some(1200));

        // Pending request should be cleaned up (all agents responded)
        let pending = scheduler.pending_requests.read().await;
        assert!(!pending.contains_key(request_id));
    }

    #[tokio::test]
    async fn test_timeout_marks_unresponsive() {
        let config = HeartbeatConfig {
            frequency: Duration::from_millis(100),
            timeout: Duration::from_millis(200),
            enabled: true,
        };
        let (tx, mut rx) = broadcast::channel(100);
        let scheduler = Arc::new(HeartbeatScheduler::new(config, tx, "test-session"));

        scheduler.register_agent("agent-1").await;

        // Simulate pending request without response
        let request_id = "req-timeout";
        let mut expected = HashSet::new();
        expected.insert("agent-1".to_string());
        let pending = PendingHeartbeat {
            timestamp: Utc::now(),
            expected_agents: expected,
            responded_agents: HashSet::new(),
        };
        scheduler
            .pending_requests
            .write()
            .await
            .insert(request_id.to_string(), pending);

        // Trigger timeout check
        scheduler.check_timeout(request_id.to_string()).await;

        // Check agent marked unresponsive
        let health = scheduler.agent_health.read().await;
        let record = health.get("agent-1").unwrap();
        assert!(matches!(record.status, AgentHealthStatus::Unresponsive));
        assert_eq!(record.consecutive_misses, 1);

        // Check timeout event emitted
        let event = rx.try_recv().unwrap();
        assert!(event.coordination_activity.is_some());
    }

    #[tokio::test]
    async fn test_consecutive_misses_increment() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = Arc::new(HeartbeatScheduler::new(config, tx, "test-session"));

        scheduler.register_agent("agent-1").await;

        // Simulate 3 consecutive timeouts
        for i in 1..=3 {
            let request_id = format!("req-{}", i);
            let mut expected = HashSet::new();
            expected.insert("agent-1".to_string());
            let pending = PendingHeartbeat {
                timestamp: Utc::now(),
                expected_agents: expected,
                responded_agents: HashSet::new(),
            };
            scheduler
                .pending_requests
                .write()
                .await
                .insert(request_id.clone(), pending);

            scheduler.check_timeout(request_id).await;
        }

        // Check consecutive misses incremented
        let health = scheduler.agent_health.read().await;
        let record = health.get("agent-1").unwrap();
        assert_eq!(record.consecutive_misses, 3);
    }

    #[tokio::test]
    async fn test_response_resets_consecutive_misses() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = Arc::new(HeartbeatScheduler::new(config, tx, "test-session"));

        scheduler.register_agent("agent-1").await;

        // Miss 2 heartbeats
        for i in 1..=2 {
            let request_id = format!("req-miss-{}", i);
            let mut expected = HashSet::new();
            expected.insert("agent-1".to_string());
            let pending = PendingHeartbeat {
                timestamp: Utc::now(),
                expected_agents: expected,
                responded_agents: HashSet::new(),
            };
            scheduler
                .pending_requests
                .write()
                .await
                .insert(request_id.clone(), pending);
            scheduler.check_timeout(request_id).await;
        }

        // Verify misses
        {
            let health = scheduler.agent_health.read().await;
            assert_eq!(health.get("agent-1").unwrap().consecutive_misses, 2);
        }

        // Now respond to a heartbeat
        let request_id = "req-response";
        let mut expected = HashSet::new();
        expected.insert("agent-1".to_string());
        let pending = PendingHeartbeat {
            timestamp: Utc::now(),
            expected_agents: expected,
            responded_agents: HashSet::new(),
        };
        scheduler
            .pending_requests
            .write()
            .await
            .insert(request_id.to_string(), pending);
        scheduler.handle_response(request_id, "agent-1", 1000).await;

        // Check consecutive_misses reset
        let health = scheduler.agent_health.read().await;
        let record = health.get("agent-1").unwrap();
        assert_eq!(record.consecutive_misses, 0);
        assert!(matches!(record.status, AgentHealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_duplicate_response_ignored() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = HeartbeatScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;

        let request_id = "req-dup";
        let mut expected = HashSet::new();
        expected.insert("agent-1".to_string());
        let pending = PendingHeartbeat {
            timestamp: Utc::now(),
            expected_agents: expected,
            responded_agents: HashSet::new(),
        };
        scheduler
            .pending_requests
            .write()
            .await
            .insert(request_id.to_string(), pending);

        // Send response twice
        scheduler.handle_response(request_id, "agent-1", 1000).await;
        scheduler.handle_response(request_id, "agent-1", 1000).await; // Second response (request already cleaned up)

        // No panic, no error - second response is gracefully ignored
        let health = scheduler.agent_health.read().await;
        assert_eq!(health.get("agent-1").unwrap().consecutive_misses, 0);
    }

    #[tokio::test]
    async fn test_health_snapshot_returns_all_agents() {
        let config = HeartbeatConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let scheduler = HeartbeatScheduler::new(config, tx, "test-session");

        scheduler.register_agent("agent-1").await;
        scheduler.register_agent("agent-2").await;
        scheduler.register_agent("agent-3").await;

        let snapshot = scheduler.agent_health_snapshot().await;
        assert_eq!(snapshot.len(), 3);

        let ids: HashSet<String> = snapshot.iter().map(|r| r.agent_id.clone()).collect();
        assert!(ids.contains("agent-1"));
        assert!(ids.contains("agent-2"));
        assert!(ids.contains("agent-3"));
    }

    #[tokio::test]
    async fn test_heartbeat_emits_events() {
        time::pause(); // Pause tokio time for deterministic testing

        let config = HeartbeatConfig {
            frequency: Duration::from_millis(100),
            timeout: Duration::from_millis(200),
            enabled: true,
        };
        let (tx, mut rx) = broadcast::channel(100);
        let scheduler = Arc::new(HeartbeatScheduler::new(config, tx, "test-session"));

        scheduler.register_agent("agent-1").await;

        // Spawn scheduler
        let scheduler_clone = Arc::clone(&scheduler);
        let handle = tokio::spawn(async move {
            let _ = scheduler_clone.run().await;
        });

        // Wait for first tick and allow task to process
        time::advance(Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
        time::advance(Duration::from_millis(1)).await; // Small advance to trigger processing
        tokio::task::yield_now().await;

        // Should receive HeartbeatRequest event (use recv with timeout)
        let event = tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert!(event.coordination_activity.is_some());
        if let Some(aof_core::coordination::CoordinationActivity::HeartbeatRequest { request_id }) =
            event.coordination_activity
        {
            assert!(!request_id.is_empty());
        } else {
            panic!("Expected HeartbeatRequest event");
        }

        handle.abort();
    }

    #[tokio::test]
    async fn test_timeout_emits_alert() {
        let config = HeartbeatConfig {
            frequency: Duration::from_millis(100),
            timeout: Duration::from_millis(50),
            enabled: true,
        };
        let (tx, mut rx) = broadcast::channel(100);
        let scheduler = Arc::new(HeartbeatScheduler::new(config, tx, "test-session"));

        scheduler.register_agent("agent-1").await;

        // Create pending request without response
        let request_id = "req-alert";
        let mut expected = HashSet::new();
        expected.insert("agent-1".to_string());
        let pending = PendingHeartbeat {
            timestamp: Utc::now(),
            expected_agents: expected,
            responded_agents: HashSet::new(),
        };
        scheduler
            .pending_requests
            .write()
            .await
            .insert(request_id.to_string(), pending);

        // Trigger timeout
        scheduler.check_timeout(request_id.to_string()).await;

        // Should receive HeartbeatTimeout event
        let event = rx.try_recv().unwrap();
        assert!(event.coordination_activity.is_some());
        if let Some(aof_core::coordination::CoordinationActivity::HeartbeatTimeout {
            unresponsive_agents,
            ..
        }) = event.coordination_activity
        {
            assert_eq!(unresponsive_agents.len(), 1);
            assert!(unresponsive_agents.contains(&"agent-1".to_string()));
        } else {
            panic!("Expected HeartbeatTimeout event");
        }
    }
}
