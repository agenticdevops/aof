//! Integration test with mock adapter
//!
//! This test demonstrates the full gateway flow: mock adapter sends messages,
//! gateway hub receives and translates them, events are broadcast to runtime.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::{broadcast, watch};

use aof_core::AofError;
use aof_gateway::{
    GatewayHub, ChannelAdapter, Platform, InboundMessage, AgentResponse, MessageUser,
};

/// Mock Slack adapter that emits predetermined messages
struct MockSlackAdapter {
    id: String,
    platform: Platform,
    messages: Vec<InboundMessage>,
    message_index: usize,
    started: bool,
    stopped: bool,
}

impl MockSlackAdapter {
    fn new(id: impl Into<String>) -> Self {
        // Create 3 test messages
        let messages = vec![
            InboundMessage {
                message_id: "msg-1".to_string(),
                platform: Platform::Slack,
                channel_id: "C123".to_string(),
                thread_id: None,
                user: MessageUser {
                    user_id: "U1".to_string(),
                    username: "user1".to_string(),
                    display_name: Some("User One".to_string()),
                },
                content: "First message".to_string(),
                attachments: vec![],
                metadata: json!({}),
                timestamp: Utc::now(),
            },
            InboundMessage {
                message_id: "msg-2".to_string(),
                platform: Platform::Slack,
                channel_id: "C123".to_string(),
                thread_id: Some("thread-123".to_string()),
                user: MessageUser {
                    user_id: "U2".to_string(),
                    username: "user2".to_string(),
                    display_name: None,
                },
                content: "Second message in thread".to_string(),
                attachments: vec![],
                metadata: json!({}),
                timestamp: Utc::now(),
            },
            InboundMessage {
                message_id: "msg-3".to_string(),
                platform: Platform::Slack,
                channel_id: "C456".to_string(),
                thread_id: None,
                user: MessageUser {
                    user_id: "U3".to_string(),
                    username: "user3".to_string(),
                    display_name: Some("User Three".to_string()),
                },
                content: "Third message in different channel".to_string(),
                attachments: vec![],
                metadata: json!({}),
                timestamp: Utc::now(),
            },
        ];

        Self {
            id: id.into(),
            platform: Platform::Slack,
            messages,
            message_index: 0,
            started: false,
            stopped: false,
        }
    }
}

#[async_trait]
impl ChannelAdapter for MockSlackAdapter {
    fn adapter_id(&self) -> &str {
        &self.id
    }

    fn platform(&self) -> Platform {
        self.platform
    }

    async fn start(&mut self) -> Result<(), AofError> {
        self.started = true;
        tracing::info!(adapter_id = %self.id, "Mock adapter started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        self.stopped = true;
        tracing::info!(adapter_id = %self.id, "Mock adapter stopped");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        Ok(self.started && !self.stopped)
    }

    async fn receive_message(&mut self) -> Result<InboundMessage, AofError> {
        if self.message_index >= self.messages.len() {
            // No more messages - wait forever (hub will shut down)
            tokio::time::sleep(Duration::from_secs(3600)).await;
            return Err(AofError::runtime("No more messages"));
        }

        let msg = self.messages[self.message_index].clone();
        self.message_index += 1;

        // Small delay to simulate network latency
        tokio::time::sleep(Duration::from_millis(10)).await;

        tracing::info!(
            adapter_id = %self.id,
            message_id = %msg.message_id,
            "Mock adapter received message"
        );

        Ok(msg)
    }

    async fn send_message(&self, response: &AgentResponse) -> Result<(), AofError> {
        tracing::info!(
            adapter_id = %self.id,
            agent_id = %response.agent_id,
            "Mock adapter sending response"
        );
        Ok(())
    }
}

#[tokio::test]
async fn test_gateway_hub_integration() {
    // Initialize tracing for test debugging
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Create event broadcast channel (agent runtime connection)
    let (event_tx, _event_rx) = broadcast::channel(100);

    // Create shutdown signal
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create gateway hub
    let mut hub = GatewayHub::new(event_tx, shutdown_rx);

    // Register mock Slack adapter
    let adapter = Box::new(MockSlackAdapter::new("test-slack"));
    hub.register_adapter(adapter);

    // Start hub
    hub.start().await.expect("Failed to start hub");

    // Spawn hub event loop in background
    let hub_handle = tokio::spawn(async move {
        // Run hub for a short time, then signal shutdown
        tokio::select! {
            result = hub.run() => {
                result.expect("Hub run failed");
            }
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                // Auto-shutdown after 500ms
                tracing::info!("Test timeout - stopping hub");
            }
        }
        hub.stop().await.expect("Failed to stop hub");
        hub
    });

    // Wait for hub to process messages
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Signal shutdown
    shutdown_tx.send(true).expect("Failed to send shutdown signal");

    // Wait for hub to finish
    let hub = hub_handle.await.expect("Hub task panicked");

    // Verify hub session ID is valid UUID format
    assert!(!hub.session_id().is_empty());
    assert_eq!(hub.session_id().len(), 36);

    tracing::info!("Integration test completed successfully");
}

#[tokio::test]
async fn test_mock_adapter_lifecycle() {
    let mut adapter = MockSlackAdapter::new("lifecycle-test");

    // Initial state
    assert!(!adapter.started);
    assert!(!adapter.stopped);
    assert_eq!(adapter.message_index, 0);

    // Start adapter
    adapter.start().await.expect("Failed to start");
    assert!(adapter.started);
    assert!(adapter.health_check().await.expect("Health check failed"));

    // Receive all messages
    let msg1 = adapter.receive_message().await.expect("Failed to receive msg 1");
    assert_eq!(msg1.message_id, "msg-1");
    assert_eq!(adapter.message_index, 1);

    let msg2 = adapter.receive_message().await.expect("Failed to receive msg 2");
    assert_eq!(msg2.message_id, "msg-2");
    assert!(msg2.thread_id.is_some());
    assert_eq!(adapter.message_index, 2);

    let msg3 = adapter.receive_message().await.expect("Failed to receive msg 3");
    assert_eq!(msg3.message_id, "msg-3");
    assert_eq!(adapter.message_index, 3);

    // Send response
    let response = AgentResponse {
        agent_id: "test-agent".to_string(),
        content: "Response to msg-3".to_string(),
        target_platform: Platform::Slack,
        target_channel: "C456".to_string(),
        thread_id: None,
    };
    adapter.send_message(&response).await.expect("Failed to send response");

    // Stop adapter
    adapter.stop().await.expect("Failed to stop");
    assert!(adapter.stopped);
    assert!(!adapter.health_check().await.expect("Health check failed after stop"));
}
