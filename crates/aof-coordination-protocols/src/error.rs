//! Error types for coordination protocols

use thiserror::Error;

/// Errors that can occur during coordination protocol operations
#[derive(Debug, Error)]
pub enum CoordinationProtocolError {
    /// Queue is full, message cannot be sent
    #[error("Queue full: {from} -> {to} (capacity {capacity})")]
    QueueFull {
        from: String,
        to: String,
        capacity: usize,
    },

    /// Agent not found in session tools
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Agent has coordination disabled
    #[error("Agent coordination disabled: {0}")]
    CoordinationDisabled(String),

    /// Message expired (TTL exceeded)
    #[error("Message expired (TTL exceeded)")]
    MessageExpired,

    /// Invalid cron expression
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    /// Invalid timezone
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),

    /// Heartbeat timeout for agent
    #[error("Heartbeat timeout for agent: {0}")]
    HeartbeatTimeout(String),

    /// LLM error
    #[error("LLM error: {0}")]
    LlmError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for CoordinationProtocolError {
    fn from(err: anyhow::Error) -> Self {
        CoordinationProtocolError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_full_error() {
        let err = CoordinationProtocolError::QueueFull {
            from: "agent-a".to_string(),
            to: "agent-b".to_string(),
            capacity: 100,
        };
        assert!(err.to_string().contains("agent-a"));
        assert!(err.to_string().contains("agent-b"));
        assert!(err.to_string().contains("100"));
    }

    #[test]
    fn test_agent_not_found_error() {
        let err = CoordinationProtocolError::AgentNotFound("agent-x".to_string());
        assert!(err.to_string().contains("agent-x"));
    }

    #[test]
    fn test_coordination_disabled_error() {
        let err = CoordinationProtocolError::CoordinationDisabled("agent-y".to_string());
        assert!(err.to_string().contains("agent-y"));
    }

    #[test]
    fn test_message_expired_error() {
        let err = CoordinationProtocolError::MessageExpired;
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn test_invalid_cron_error() {
        let err = CoordinationProtocolError::InvalidCron("bad cron".to_string());
        assert!(err.to_string().contains("bad cron"));
    }

    #[test]
    fn test_heartbeat_timeout_error() {
        let err = CoordinationProtocolError::HeartbeatTimeout("agent-z".to_string());
        assert!(err.to_string().contains("agent-z"));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let err: CoordinationProtocolError = anyhow_err.into();
        match err {
            CoordinationProtocolError::Internal(msg) => {
                assert!(msg.contains("something went wrong"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
}
