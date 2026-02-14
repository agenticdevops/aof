//! Integration tests for heartbeat protocol
//!
//! Tests the full heartbeat lifecycle with mock agents:
//! - Multi-agent health monitoring
//! - Timeout detection
//! - Recovery after missed heartbeats
//! - Coordination mode enforcement

mod test_helpers;

use aof_coordination_protocols::{
    AgentHealthStatus, CoordinationManager, CoordinationMode, HeartbeatConfig,
};
use aof_core::coordination::{CoordinationActivity, CoordinationEvent};
use std::time::Duration;
use test_helpers::{MockAgent, TestConfig};
use tokio::time::sleep;

#[tokio::test]
async fn test_heartbeat_3_agents_all_respond() {
    // Create manager with test config (500ms heartbeat, 1s timeout)
    let (manager, event_tx, _event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register 3 agents
    manager
        .register_agent("agent-1", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("agent-2", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("agent-3", CoordinationMode::Full)
        .await
        .unwrap();

    // Create mock agents that respond to heartbeat
    let mut mock1 = MockAgent::new("agent-1", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock2 = MockAgent::new("agent-2", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock3 = MockAgent::new("agent-3", CoordinationMode::Full, event_tx.clone(), "test-session");

    // Start responders with 50ms delay
    mock1.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock2.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock3.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Wait for 3 heartbeat cycles (3 × 500ms = 1500ms)
    sleep(Duration::from_millis(1600)).await;

    // Get health snapshot
    let health = manager.health_snapshot().await;

    // Verify all agents are healthy
    assert_eq!(health.len(), 3);
    for agent in &health {
        assert_eq!(agent.status, AgentHealthStatus::Healthy);
        assert_eq!(agent.consecutive_misses, 0);
    }
}

#[tokio::test]
async fn test_heartbeat_1_agent_unresponsive() {
    let (manager, event_tx, _event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register 3 agents
    manager
        .register_agent("agent-1", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("agent-2", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("agent-3", CoordinationMode::Full)
        .await
        .unwrap();

    // Create mock agents
    let mut mock1 = MockAgent::new("agent-1", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock2 = MockAgent::new("agent-2", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock3 = MockAgent::new("agent-3", CoordinationMode::Full, event_tx.clone(), "test-session");

    // Start responders
    mock1.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock2.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock3.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Wait for first heartbeat cycle
    sleep(Duration::from_millis(600)).await;

    // Stop agent-2 (simulate crash)
    mock2.stop();

    // Wait for 3 more cycles (agent-2 will miss all)
    sleep(Duration::from_millis(1600)).await;

    // Get health snapshot
    let health = manager.health_snapshot().await;

    // Verify 2 healthy, 1 unresponsive
    assert_eq!(health.len(), 3);

    let agent1 = health.iter().find(|a| a.agent_id == "agent-1").unwrap();
    let agent2 = health.iter().find(|a| a.agent_id == "agent-2").unwrap();
    let agent3 = health.iter().find(|a| a.agent_id == "agent-3").unwrap();

    assert_eq!(agent1.status, AgentHealthStatus::Healthy);
    assert_eq!(agent3.status, AgentHealthStatus::Healthy);
    assert_eq!(agent2.status, AgentHealthStatus::Unresponsive);
    assert!(agent2.consecutive_misses > 0);
}

#[tokio::test]
async fn test_heartbeat_events_flow() {
    let (manager, event_tx, mut event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register 2 agents
    manager
        .register_agent("agent-1", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("agent-2", CoordinationMode::Full)
        .await
        .unwrap();

    // Create mock agents
    let mut mock1 = MockAgent::new("agent-1", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock2 = MockAgent::new("agent-2", CoordinationMode::Full, event_tx.clone(), "test-session");

    mock1.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock2.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Collect events for 2 heartbeat cycles
    let collect_handle = tokio::spawn(async move {
        let start = tokio::time::Instant::now();
        let mut req_count = 0;
        let mut resp_count = 0;

        while start.elapsed() < Duration::from_millis(1200) {
            if let Ok(event) = tokio::time::timeout(Duration::from_millis(100), event_rx.recv()).await {
                if let Ok(event) = event {
                    if let Some(activity) = &event.coordination_activity {
                        match activity {
                            CoordinationActivity::HeartbeatRequest { .. } => req_count += 1,
                            CoordinationActivity::HeartbeatResponse { .. } => resp_count += 1,
                            _ => {}
                        }
                    }
                }
            }
        }

        (req_count, resp_count)
    });

    let (heartbeat_requests, heartbeat_responses) = collect_handle.await.unwrap();

    // Verify events were received (2 cycles × 2 agents = 4 requests, 4 responses minimum)
    assert!(heartbeat_requests >= 2, "Expected at least 2 heartbeat requests, got {}", heartbeat_requests);
    assert!(heartbeat_responses >= 2, "Expected at least 2 heartbeat responses, got {}", heartbeat_responses);
}

#[tokio::test]
async fn test_heartbeat_timeout_alert() {
    let (manager, event_tx, mut event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register agent
    manager
        .register_agent("slow-agent", CoordinationMode::Full)
        .await
        .unwrap();

    // Create mock agent that NEVER responds
    let _mock = MockAgent::new("slow-agent", CoordinationMode::Full, event_tx.clone(), "test-session");
    // Don't start responder - agent will timeout

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Collect events for timeout duration + buffer (1s timeout + 500ms buffer)
    let collect_handle = tokio::spawn(async move {
        let start = tokio::time::Instant::now();
        let mut alert_count = 0;

        while start.elapsed() < Duration::from_millis(1600) {
            if let Ok(event) = tokio::time::timeout(Duration::from_millis(100), event_rx.recv()).await {
                if let Ok(event) = event {
                    if let Some(activity) = &event.coordination_activity {
                        if matches!(activity, CoordinationActivity::HeartbeatTimeout { .. }) {
                            alert_count += 1;
                        }
                    }
                }
            }
        }

        alert_count
    });

    let timeout_alerts = collect_handle.await.unwrap();

    // Verify at least one timeout alert emitted
    assert!(timeout_alerts >= 1, "Expected at least 1 timeout alert, got {}", timeout_alerts);
}

#[tokio::test]
async fn test_heartbeat_recovery() {
    let (manager, event_tx, _event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register agent
    manager
        .register_agent("flaky-agent", CoordinationMode::Full)
        .await
        .unwrap();

    // Create mock agent
    let mut mock = MockAgent::new("flaky-agent", CoordinationMode::Full, event_tx.clone(), "test-session");

    // Start responder
    mock.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Wait for first heartbeat
    sleep(Duration::from_millis(600)).await;

    // Stop agent (miss 2 heartbeats)
    mock.stop();
    sleep(Duration::from_millis(1200)).await;

    // Verify agent is unresponsive
    let health1 = manager.health_snapshot().await;
    let agent = health1.iter().find(|a| a.agent_id == "flaky-agent").unwrap();
    assert_eq!(agent.status, AgentHealthStatus::Unresponsive);
    let misses = agent.consecutive_misses;
    assert!(misses >= 2);

    // Restart responder (simulate recovery)
    mock.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Wait for 2 heartbeat cycles
    sleep(Duration::from_millis(1200)).await;

    // Verify agent recovered
    let health2 = manager.health_snapshot().await;
    let agent = health2.iter().find(|a| a.agent_id == "flaky-agent").unwrap();
    assert_eq!(agent.status, AgentHealthStatus::Healthy);
    assert_eq!(agent.consecutive_misses, 0);
}

#[tokio::test]
async fn test_heartbeat_respects_coordination_mode() {
    let (manager, event_tx, mut event_rx) =
        TestConfig::create_test_coordination_manager("test-session");

    // Register agents with different modes
    manager
        .register_agent("full-agent", CoordinationMode::Full)
        .await
        .unwrap();
    manager
        .register_agent("disabled-agent", CoordinationMode::Disabled)
        .await
        .unwrap();

    // Create mock agents
    let mut mock1 = MockAgent::new("full-agent", CoordinationMode::Full, event_tx.clone(), "test-session");
    let mut mock2 = MockAgent::new("disabled-agent", CoordinationMode::Disabled, event_tx.clone(), "test-session");

    mock1.respond_to_heartbeat(Duration::from_millis(50)).await;
    mock2.respond_to_heartbeat(Duration::from_millis(50)).await;

    // Start manager
    let _handles = manager.start().await.unwrap();

    // Collect heartbeat requests for 2 cycles
    let collect_handle = tokio::spawn(async move {
        let start = tokio::time::Instant::now();
        let mut full_count = 0;
        let mut disabled_count = 0;

        while start.elapsed() < Duration::from_millis(1200) {
            if let Ok(event) = tokio::time::timeout(Duration::from_millis(100), event_rx.recv()).await {
                if let Ok(event) = event {
                    if let Some(activity) = &event.coordination_activity {
                        if let CoordinationActivity::HeartbeatRequest { .. } = activity {
                            // Check agent_id in event
                            if event.agent_id == "full-agent" {
                                full_count += 1;
                            } else if event.agent_id == "disabled-agent" {
                                disabled_count += 1;
                            }
                        }
                    }
                }
            }
        }

        (full_count, disabled_count)
    });

    let (_full_agent_requests, _disabled_agent_requests) = collect_handle.await.unwrap();

    // Verify only Full agent received heartbeat requests
    // Note: HeartbeatRequest is broadcast to all, but disabled agents are not registered
    // in the heartbeat scheduler. We check health snapshot instead.
    let health = manager.health_snapshot().await;

    // Only full-agent should be in health tracking
    assert_eq!(health.len(), 1);
    assert_eq!(health[0].agent_id, "full-agent");
}
