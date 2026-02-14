//! Coordination Manager - Orchestrates all coordination protocols
//!
//! The CoordinationManager coordinates all protocol schedulers (heartbeat, standup, metrics)
//! and provides a unified interface for agent registration, event routing, and health monitoring.
//!
//! # Architecture
//!
//! ```text
//! CoordinationManager
//!   ├── SessionTools (agent message queues)
//!   ├── HeartbeatScheduler (health monitoring)
//!   ├── StandupScheduler (daily status - Plan 03)
//!   └── TokenMetrics (overhead tracking - Plan 04)
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_coordination_protocols::manager::{CoordinationManager, CoordinationConfig};
//! use tokio::sync::broadcast;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = CoordinationConfig::default();
//!     let (event_tx, _) = broadcast::channel(1000);
//!     let session_id = "session-123".to_string();
//!
//!     let manager = CoordinationManager::new(config, event_tx, session_id);
//!
//!     // Register agents with coordination modes
//!     manager.register_agent("k8s-monitor", aof_coordination_protocols::CoordinationMode::Full).await?;
//!     manager.register_agent("log-analyzer", aof_coordination_protocols::CoordinationMode::Standard).await?;
//!
//!     // Start background tasks
//!     let handles = manager.start().await?;
//!
//!     // Get health snapshot for REST API
//!     let health = manager.health_snapshot().await;
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, info};

use aof_core::coordination::CoordinationEvent;

use crate::error::CoordinationProtocolError;
use crate::events::{CoordinationMode, SessionMessage};
use crate::heartbeat::{AgentHealthRecord, HeartbeatConfig, HeartbeatScheduler};
use crate::session_tools::SessionTools;

/// Coordination configuration
#[derive(Debug, Clone)]
pub struct CoordinationConfig {
    /// Whether coordination protocols are enabled globally
    pub enabled: bool,
    /// Global default coordination mode for agents
    pub mode: CoordinationMode,
    /// Heartbeat protocol configuration
    pub heartbeat: HeartbeatConfig,
    // TODO: Add standup config in Plan 03
    // TODO: Add metrics config in Plan 04
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: CoordinationMode::Full,
            heartbeat: HeartbeatConfig::default(),
        }
    }
}

/// Coordination Manager - Orchestrates all coordination protocols
///
/// Manages:
/// - SessionTools for agent-to-agent messaging
/// - HeartbeatScheduler for health monitoring
/// - Agent coordination modes (per-agent opt-in)
/// - Event routing to protocol handlers
///
/// The manager spawns background tokio tasks for each enabled protocol
/// and routes incoming coordination events to the appropriate handler.
pub struct CoordinationManager {
    config: CoordinationConfig,
    session_tools: Arc<SessionTools>,
    heartbeat: Option<Arc<HeartbeatScheduler>>,
    event_tx: broadcast::Sender<CoordinationEvent>,
    session_id: String,
    /// Track per-agent coordination modes
    agent_modes: Arc<RwLock<HashMap<String, CoordinationMode>>>,
}

impl CoordinationManager {
    /// Create a new coordination manager
    pub fn new(
        config: CoordinationConfig,
        event_tx: broadcast::Sender<CoordinationEvent>,
        session_id: impl Into<String>,
    ) -> Self {
        let session_id_str = session_id.into();

        // Create session tools with default capacity (100 messages, 30-minute TTL)
        let session_tools = Arc::new(SessionTools::new(
            100,
            std::time::Duration::from_secs(30 * 60),
        ));

        // Create heartbeat scheduler if enabled
        let heartbeat = if config.enabled && config.heartbeat.enabled {
            let scheduler = Arc::new(HeartbeatScheduler::new(
                config.heartbeat.clone(),
                event_tx.clone(),
                session_id_str.clone(),
            ));
            Some(scheduler)
        } else {
            None
        };

        Self {
            config,
            session_tools,
            heartbeat,
            event_tx,
            session_id: session_id_str,
            agent_modes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent with its coordination mode
    ///
    /// The agent's coordination mode determines which protocols it participates in:
    /// - Full: heartbeat + standup + roundtables + messages
    /// - Standard: heartbeat + standup + messages
    /// - Reduced: heartbeat (lower frequency) + messages
    /// - HeartbeatOnly: just health checks
    /// - Disabled: no coordination
    pub async fn register_agent(
        &self,
        agent_id: impl Into<String>,
        mode: CoordinationMode,
    ) -> Result<(), CoordinationProtocolError> {
        let agent_id_str = agent_id.into();
        debug!(
            "Registering agent {} with coordination mode: {:?}",
            agent_id_str, mode
        );

        // Store agent mode
        self.agent_modes
            .write()
            .await
            .insert(agent_id_str.clone(), mode);

        // Register in session tools (for message routing)
        // All agents get message queues, even if coordination is disabled
        self.session_tools.register_agent(&agent_id_str).await?;

        // Register in heartbeat scheduler if mode includes heartbeat
        if matches!(
            mode,
            CoordinationMode::Full
                | CoordinationMode::Standard
                | CoordinationMode::Reduced
                | CoordinationMode::HeartbeatOnly
        ) {
            if let Some(heartbeat) = &self.heartbeat {
                heartbeat.register_agent(&agent_id_str).await;
            }
        }

        Ok(())
    }

    /// Start all background coordination tasks
    ///
    /// Spawns tokio tasks for each enabled protocol:
    /// - Heartbeat scheduler (if enabled)
    /// - Standup scheduler (Plan 03)
    /// - Token metrics tracker (Plan 04)
    ///
    /// Returns JoinHandles so caller can await shutdown.
    pub async fn start(&self) -> Result<Vec<JoinHandle<()>>, CoordinationProtocolError> {
        if !self.config.enabled {
            info!("Coordination protocols disabled in config");
            return Ok(vec![]);
        }

        info!("Starting coordination manager");
        let mut handles = vec![];

        // Start heartbeat scheduler
        if let Some(heartbeat) = &self.heartbeat {
            let heartbeat_clone = Arc::clone(heartbeat);
            let handle = tokio::spawn(async move {
                if let Err(e) = heartbeat_clone.run().await {
                    tracing::error!("Heartbeat scheduler error: {}", e);
                }
            });
            handles.push(handle);
            info!("Heartbeat scheduler started");
        }

        // TODO: Start standup scheduler in Plan 03
        // TODO: Start token metrics tracker in Plan 04

        Ok(handles)
    }

    /// Handle incoming coordination event
    ///
    /// Routes events to appropriate protocol handlers:
    /// - HeartbeatResponse -> HeartbeatScheduler::handle_response()
    /// - StandupResponse -> StandupScheduler::handle_response()
    /// - SessionMessage -> SessionTools (already handled separately)
    pub async fn handle_event(&self, event: &CoordinationEvent) {
        if let Some(activity) = &event.coordination_activity {
            use aof_core::coordination::CoordinationActivity;

            match activity {
                CoordinationActivity::HeartbeatResponse {
                    request_id,
                    agent_id,
                    ..
                } => {
                    if let Some(heartbeat) = &self.heartbeat {
                        // TODO: Extract actual response time from event metadata
                        // For now, use a placeholder value
                        heartbeat.handle_response(request_id, agent_id, 1000).await;
                    }
                }
                CoordinationActivity::StandupResponse { .. } => {
                    // TODO: Route to StandupScheduler in Plan 03
                }
                CoordinationActivity::SessionMessage { .. } => {
                    // Session messages are handled directly via SessionTools.send_message()
                    // This event is just for broadcast visibility
                }
                _ => {
                    // Other coordination activities (requests, timeouts, summaries)
                    // are emitted by schedulers, not handled
                }
            }
        }
    }

    /// Get current health snapshot for all agents
    ///
    /// Returns agent health records for REST API consumption.
    pub async fn health_snapshot(&self) -> Vec<AgentHealthRecord> {
        if let Some(heartbeat) = &self.heartbeat {
            heartbeat.agent_health_snapshot().await
        } else {
            vec![]
        }
    }

    /// Get reference to session tools for direct message sending
    pub fn session_tools(&self) -> Arc<SessionTools> {
        Arc::clone(&self.session_tools)
    }

    /// Send a session message from one agent to another
    ///
    /// Convenience method that wraps SessionTools::send_message().
    pub async fn send_session_message(
        &self,
        message: SessionMessage,
    ) -> Result<(), CoordinationProtocolError> {
        self.session_tools.send_message(message).await?;
        Ok(())
    }

    /// Get the coordination mode for a specific agent
    pub async fn get_agent_mode(&self, agent_id: &str) -> Option<CoordinationMode> {
        self.agent_modes.read().await.get(agent_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation_default_config() {
        let config = CoordinationConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config.clone(), tx, "test-session");

        assert_eq!(manager.session_id, "test-session");
        assert!(manager.heartbeat.is_some());
        assert!(manager.config.enabled);
    }

    #[tokio::test]
    async fn test_manager_disabled_coordination() {
        let mut config = CoordinationConfig::default();
        config.enabled = false;

        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config, tx, "test-session");

        assert!(manager.heartbeat.is_none());
    }

    #[tokio::test]
    async fn test_register_agent_with_full_mode() {
        let config = CoordinationConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config, tx, "test-session");

        manager
            .register_agent("agent-1", CoordinationMode::Full)
            .await
            .unwrap();

        // Check agent mode stored
        let mode = manager.get_agent_mode("agent-1").await;
        assert_eq!(mode, Some(CoordinationMode::Full));

        // Check registered in heartbeat
        if let Some(heartbeat) = &manager.heartbeat {
            let health = heartbeat.agent_health_snapshot().await;
            assert_eq!(health.len(), 1);
            assert_eq!(health[0].agent_id, "agent-1");
        }
    }

    #[tokio::test]
    async fn test_register_agent_with_heartbeat_only() {
        let config = CoordinationConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config, tx, "test-session");

        manager
            .register_agent("agent-1", CoordinationMode::HeartbeatOnly)
            .await
            .unwrap();

        // Check agent mode stored
        let mode = manager.get_agent_mode("agent-1").await;
        assert_eq!(mode, Some(CoordinationMode::HeartbeatOnly));

        // Check registered in heartbeat
        if let Some(heartbeat) = &manager.heartbeat {
            let health = heartbeat.agent_health_snapshot().await;
            assert_eq!(health.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_register_agent_disabled() {
        let config = CoordinationConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config, tx, "test-session");

        manager
            .register_agent("agent-1", CoordinationMode::Disabled)
            .await
            .unwrap();

        // Check agent mode stored
        let mode = manager.get_agent_mode("agent-1").await;
        assert_eq!(mode, Some(CoordinationMode::Disabled));

        // Check NOT registered in heartbeat
        if let Some(heartbeat) = &manager.heartbeat {
            let health = heartbeat.agent_health_snapshot().await;
            assert_eq!(health.len(), 0); // Disabled mode = not in heartbeat
        }
    }

    #[tokio::test]
    async fn test_health_snapshot_delegated() {
        let config = CoordinationConfig::default();
        let (tx, _rx) = broadcast::channel(100);
        let manager = CoordinationManager::new(config, tx, "test-session");

        manager
            .register_agent("agent-1", CoordinationMode::Full)
            .await
            .unwrap();
        manager
            .register_agent("agent-2", CoordinationMode::Standard)
            .await
            .unwrap();

        let snapshot = manager.health_snapshot().await;
        assert_eq!(snapshot.len(), 2);

        let ids: Vec<String> = snapshot.iter().map(|r| r.agent_id.clone()).collect();
        assert!(ids.contains(&"agent-1".to_string()));
        assert!(ids.contains(&"agent-2".to_string()));
    }
}
