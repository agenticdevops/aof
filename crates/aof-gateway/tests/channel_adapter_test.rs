//! Channel adapter trait ergonomics tests

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

use aof_core::AofError;
use aof_gateway::{ChannelAdapter, Platform, InboundMessage, AgentResponse, MessageUser};

struct MockSlackAdapter {
    id: String,
    started: bool,
    stopped: bool,
}

#[async_trait]
impl ChannelAdapter for MockSlackAdapter {
    fn adapter_id(&self) -> &str {
        &self.id
    }

    fn platform(&self) -> Platform {
        Platform::Slack
    }

    async fn start(&mut self) -> Result<(), AofError> {
        self.started = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        self.stopped = true;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        Ok(self.started && !self.stopped)
    }

    async fn receive_message(&mut self) -> Result<InboundMessage, AofError> {
        Ok(InboundMessage {
            message_id: "test-msg".to_string(),
            platform: Platform::Slack,
            channel_id: "C123".to_string(),
            thread_id: None,
            user: MessageUser {
                user_id: "U123".to_string(),
                username: "testuser".to_string(),
                display_name: None,
            },
            content: "test message".to_string(),
            attachments: vec![],
            metadata: json!({}),
            timestamp: Utc::now(),
        })
    }

    async fn send_message(&self, _response: &AgentResponse) -> Result<(), AofError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_mock_adapter_implements_trait() {
    let mut adapter = MockSlackAdapter {
        id: "test-slack".to_string(),
        started: false,
        stopped: false,
    };

    // Test lifecycle
    assert!(!adapter.started);
    adapter.start().await.unwrap();
    assert!(adapter.started);

    // Test health check
    assert!(adapter.health_check().await.unwrap());

    // Test receive message
    let msg = adapter.receive_message().await.unwrap();
    assert_eq!(msg.message_id, "test-msg");
    assert_eq!(msg.platform, Platform::Slack);

    // Test send message
    let response = AgentResponse {
        agent_id: "test-agent".to_string(),
        content: "response".to_string(),
        target_platform: Platform::Slack,
        target_channel: "C123".to_string(),
        thread_id: None,
    };
    assert!(adapter.send_message(&response).await.is_ok());

    // Test stop
    adapter.stop().await.unwrap();
    assert!(adapter.stopped);
}

#[test]
fn test_platform_enum_serialization() {
    // Test all platform variants serialize/deserialize
    let platforms = vec![
        Platform::Slack,
        Platform::Discord,
        Platform::Telegram,
        Platform::WhatsApp,
    ];

    for platform in platforms {
        let json = serde_json::to_string(&platform).unwrap();
        let deserialized: Platform = serde_json::from_str(&json).unwrap();
        assert_eq!(platform, deserialized);
    }
}
