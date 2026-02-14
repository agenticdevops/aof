//! Test helpers for coordination protocol integration tests
//!
//! Provides MockAgent infrastructure for simulating agent behavior in tests.

use aof_coordination_protocols::{
    AgentHealthStatus, CoordinationConfig, CoordinationManager, CoordinationMode, HeartbeatConfig,
    MessageType, SessionMessage, StandupReport,
};
use aof_core::coordination::CoordinationEvent;
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Mock agent for integration testing
pub struct MockAgent {
    pub id: String,
    pub coordination_mode: CoordinationMode,
    event_rx: broadcast::Receiver<CoordinationEvent>,
    event_tx: broadcast::Sender<CoordinationEvent>,
    session_id: String,
    handle: Option<JoinHandle<()>>,
}

impl MockAgent {
    /// Create new mock agent
    pub fn new(
        id: impl Into<String>,
        coordination_mode: CoordinationMode,
        event_tx: broadcast::Sender<CoordinationEvent>,
        session_id: impl Into<String>,
    ) -> Self {
        let event_rx = event_tx.subscribe();
        Self {
            id: id.into(),
            coordination_mode,
            event_rx,
            event_tx,
            session_id: session_id.into(),
            handle: None,
        }
    }

    /// Respond to heartbeat requests with configurable delay
    pub async fn respond_to_heartbeat(&mut self, delay: std::time::Duration) {
        let id = self.id.clone();
        let tx = self.event_tx.clone();
        let session = self.session_id.clone();
        let mut rx = self.event_tx.subscribe();

        let handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // Check for heartbeat request
                if let Some(activity) = &event.coordination_activity {
                    if let aof_core::coordination::CoordinationActivity::HeartbeatRequest {
                        request_id,
                        ..
                    } = activity
                    {
                        // Simulate processing delay
                        tokio::time::sleep(delay).await;

                        // Send heartbeat response
                        let response = CoordinationEvent::heartbeat_response(
                            &session,
                            &id,
                            request_id.clone(),
                            AgentHealthStatus::Healthy,
                            Some(delay.as_millis() as u64),
                        );

                        let _ = tx.send(response);
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Respond to standup requests
    pub async fn respond_to_standup(&mut self) {
        let id = self.id.clone();
        let tx = self.event_tx.clone();
        let session = self.session_id.clone();
        let mut rx = self.event_tx.subscribe();

        let handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // Check for standup request
                if let Some(activity) = &event.coordination_activity {
                    if let aof_core::coordination::CoordinationActivity::StandupRequest {
                        request_id,
                        ..
                    } = activity
                    {
                        // Create mock standup report
                        let report = StandupReport {
                            what_i_did: format!("{}: Completed task X", id),
                            what_im_doing: format!("{}: Working on task Y", id),
                            blockers: None,
                        };

                        // Send standup response
                        let response = CoordinationEvent::standup_response(
                            &session,
                            &id,
                            request_id.clone(),
                            report,
                        );

                        let _ = tx.send(response);
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Run both heartbeat and standup responders
    pub async fn run(&mut self, heartbeat_delay: std::time::Duration) {
        let id = self.id.clone();
        let tx = self.event_tx.clone();
        let session = self.session_id.clone();
        let mut rx = self.event_tx.subscribe();

        let handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Some(activity) = &event.coordination_activity {
                    match activity {
                        aof_core::coordination::CoordinationActivity::HeartbeatRequest {
                            request_id,
                            ..
                        } => {
                            tokio::time::sleep(heartbeat_delay).await;
                            let response = CoordinationEvent::heartbeat_response(
                                &session,
                                &id,
                                request_id.clone(),
                                AgentHealthStatus::Healthy,
                                Some(heartbeat_delay.as_millis() as u64),
                            );
                            let _ = tx.send(response);
                        }
                        aof_core::coordination::CoordinationActivity::StandupRequest {
                            request_id,
                            ..
                        } => {
                            let report = StandupReport {
                                what_i_did: format!("{}: Completed tasks", id),
                                what_im_doing: format!("{}: Working on features", id),
                                blockers: None,
                            };
                            let response = CoordinationEvent::standup_response(
                                &session,
                                &id,
                                request_id.clone(),
                                report,
                            );
                            let _ = tx.send(response);
                        }
                        _ => {}
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Stop the agent (simulate crash)
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for MockAgent {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Test configuration with fast intervals for quick testing
#[derive(Debug, Clone)]
pub struct TestConfig;

impl TestConfig {
    /// Create coordination config optimized for fast testing
    pub fn coordination_config() -> CoordinationConfig {
        CoordinationConfig {
            enabled: true,
            mode: CoordinationMode::Full,
            heartbeat: HeartbeatConfig {
                frequency: std::time::Duration::from_millis(500),
                timeout: std::time::Duration::from_secs(1),
                enabled: true,
            },
            token_limits: Default::default(),
            standup: Default::default(),
        }
    }

    /// Create coordination manager for testing
    pub fn create_test_coordination_manager(
        session_id: impl Into<String>,
    ) -> (
        CoordinationManager,
        broadcast::Sender<CoordinationEvent>,
        broadcast::Receiver<CoordinationEvent>,
    ) {
        let config = Self::coordination_config();
        let (event_tx, event_rx) = broadcast::channel(1000);
        let manager = CoordinationManager::new(config, event_tx.clone(), session_id.into());

        (manager, event_tx, event_rx)
    }
}
