//! Event types for coordination protocols

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session message for agent-to-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    /// Unique message ID (UUID v4)
    pub id: String,
    /// Source agent ID
    pub from_agent: String,
    /// Destination agent ID
    pub to_agent: String,
    /// Message type
    pub message_type: MessageType,
    /// Message content
    pub content: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// When message was created
    pub timestamp: DateTime<Utc>,
    /// When message expires (timestamp + TTL)
    pub expires_at: DateTime<Utc>,
}

impl SessionMessage {
    /// Create a new session message
    pub fn new(
        from_agent: impl Into<String>,
        to_agent: impl Into<String>,
        message_type: MessageType,
        content: impl Into<String>,
        ttl: Duration,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + ttl;
        Self {
            id: Uuid::new_v4().to_string(),
            from_agent: from_agent.into(),
            to_agent: to_agent.into(),
            message_type,
            content: content.into(),
            metadata: HashMap::new(),
            timestamp: now,
            expires_at,
        }
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Message type enum for session messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Informational broadcast
    Announcement,
    /// Request for help/collaboration
    CollaborationRequest,
    /// Delegate work to another agent
    TaskAssignment,
    /// Route to human for intervention
    HumanEscalation,
    /// Health check ping
    HeartbeatRequest,
    /// Health check pong
    HeartbeatResponse,
    /// Daily standup trigger
    StandupRequest,
    /// Standup report
    StandupResponse,
    /// Extensible custom message type
    Custom(String),
}

/// Agent health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentHealthStatus {
    /// Agent is healthy and responsive
    Healthy,
    /// Agent is degraded (slow responses, partial functionality)
    Degraded { reason: String },
    /// Agent is unresponsive
    Unresponsive,
}

/// Standup report structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandupReport {
    /// What the agent accomplished recently
    pub what_i_did: String,
    /// What the agent is currently working on
    pub what_im_doing: String,
    /// Current blockers/issues
    pub blockers: Vec<String>,
}

/// Coordination mode - per-agent opt-in for coordination protocols
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationMode {
    /// Full coordination: heartbeat + standup + roundtables + messages
    Full,
    /// Standard: heartbeat + standup + messages (no roundtables)
    Standard,
    /// Reduced: heartbeat at lower frequency + messages
    Reduced,
    /// Heartbeat only: just health checks
    HeartbeatOnly,
    /// Disabled: no coordination protocols
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_message_creation() {
        let msg = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Hello",
            Duration::minutes(30),
        );

        assert!(!msg.id.is_empty());
        assert_eq!(msg.from_agent, "agent-a");
        assert_eq!(msg.to_agent, "agent-b");
        assert_eq!(msg.message_type, MessageType::Announcement);
        assert_eq!(msg.content, "Hello");
        assert!(msg.metadata.is_empty());
        assert!(msg.expires_at > msg.timestamp);
    }

    #[test]
    fn test_session_message_expiry() {
        let msg = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Test",
            Duration::milliseconds(0),
        );

        // Sleep briefly to ensure expiry
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(msg.is_expired());
    }

    #[test]
    fn test_session_message_not_expired() {
        let msg = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Test",
            Duration::minutes(30),
        );

        assert!(!msg.is_expired());
    }

    #[test]
    fn test_session_message_with_metadata() {
        let msg = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::TaskAssignment,
            "Do work",
            Duration::minutes(30),
        )
        .with_metadata("priority", serde_json::json!("high"))
        .with_metadata("task_id", serde_json::json!("task-123"));

        assert_eq!(msg.metadata.len(), 2);
        assert_eq!(msg.metadata.get("priority").unwrap(), &serde_json::json!("high"));
        assert_eq!(msg.metadata.get("task_id").unwrap(), &serde_json::json!("task-123"));
    }

    #[test]
    fn test_message_type_serialization() {
        let types = vec![
            MessageType::Announcement,
            MessageType::CollaborationRequest,
            MessageType::TaskAssignment,
            MessageType::HumanEscalation,
            MessageType::HeartbeatRequest,
            MessageType::HeartbeatResponse,
            MessageType::StandupRequest,
            MessageType::StandupResponse,
            MessageType::Custom("custom_type".to_string()),
        ];

        for msg_type in types {
            let json = serde_json::to_string(&msg_type).unwrap();
            let deserialized: MessageType = serde_json::from_str(&json).unwrap();
            assert_eq!(msg_type, deserialized);
        }
    }

    #[test]
    fn test_coordination_mode_serialization() {
        let modes = vec![
            CoordinationMode::Full,
            CoordinationMode::Standard,
            CoordinationMode::Reduced,
            CoordinationMode::HeartbeatOnly,
            CoordinationMode::Disabled,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: CoordinationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }
    }

    #[test]
    fn test_agent_health_status_variants() {
        let healthy = AgentHealthStatus::Healthy;
        let degraded = AgentHealthStatus::Degraded {
            reason: "High latency".to_string(),
        };
        let unresponsive = AgentHealthStatus::Unresponsive;

        // Serialize and deserialize
        let json_healthy = serde_json::to_string(&healthy).unwrap();
        let json_degraded = serde_json::to_string(&degraded).unwrap();
        let json_unresponsive = serde_json::to_string(&unresponsive).unwrap();

        let _: AgentHealthStatus = serde_json::from_str(&json_healthy).unwrap();
        let _: AgentHealthStatus = serde_json::from_str(&json_degraded).unwrap();
        let _: AgentHealthStatus = serde_json::from_str(&json_unresponsive).unwrap();
    }

    #[test]
    fn test_standup_report_creation() {
        let report = StandupReport {
            what_i_did: "Fixed bug in auth".to_string(),
            what_im_doing: "Working on API endpoint".to_string(),
            blockers: vec!["Need database access".to_string()],
        };

        assert_eq!(report.what_i_did, "Fixed bug in auth");
        assert_eq!(report.what_im_doing, "Working on API endpoint");
        assert_eq!(report.blockers.len(), 1);

        // Test serialization
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: StandupReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.what_i_did, "Fixed bug in auth");
    }
}
