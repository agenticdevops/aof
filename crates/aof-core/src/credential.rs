//! Credential access audit types
//!
//! This module provides types for credential access auditing and anomaly detection.
//! These types enable monitoring and alerting on credential access patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A credential access event recorded in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAccessEvent {
    /// Unique event ID
    pub event_id: String,
    /// Timestamp of the access
    pub timestamp: DateTime<Utc>,
    /// Agent performing the access
    pub agent_id: String,
    /// Type of credential accessed
    pub credential_type: CredentialType,
    /// Path to the credential file
    pub file_path: String,
    /// Access mode (read/write/execute)
    pub access_mode: AccessMode,
    /// Tool context for the access
    pub tool_context: ToolContext,
    /// Anomaly score (0.0-1.0)
    pub anomaly_score: f64,
    /// Sequence number for tamper detection
    pub sequence_number: u64,
    /// Session ID for grouping related accesses
    pub session_id: String,
}

impl CredentialAccessEvent {
    /// Create a new credential access event
    pub fn new(
        event_id: String,
        agent_id: String,
        credential_type: CredentialType,
        file_path: String,
        access_mode: AccessMode,
        tool_context: ToolContext,
        sequence_number: u64,
        session_id: String,
    ) -> Self {
        Self {
            event_id,
            timestamp: Utc::now(),
            agent_id,
            credential_type,
            file_path,
            access_mode,
            tool_context,
            anomaly_score: 0.0,
            sequence_number,
            session_id,
        }
    }

    /// Set the anomaly score
    pub fn with_anomaly_score(mut self, score: f64) -> Self {
        self.anomaly_score = score;
        self
    }
}

/// Type of credential being accessed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CredentialType {
    /// Kubernetes config (kubeconfig)
    Kubernetes,
    /// AWS credentials
    Aws,
    /// Google Cloud credentials
    Gcp,
    /// Azure credentials
    Azure,
    /// Git credentials
    Git,
    /// Database credentials
    Database,
    /// HashiCorp Vault token
    Vault,
    /// Custom credential type
    Custom(String),
}

impl CredentialType {
    /// Get the credential type name
    pub fn name(&self) -> &str {
        match self {
            Self::Kubernetes => "kubernetes",
            Self::Aws => "aws",
            Self::Gcp => "gcp",
            Self::Azure => "azure",
            Self::Git => "git",
            Self::Database => "database",
            Self::Vault => "vault",
            Self::Custom(name) => name,
        }
    }
}

/// Access mode for credential
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessMode {
    /// Read access
    Read,
    /// Write access (should never happen with read-only mounts)
    Write,
    /// Execute access
    Execute,
}

/// Tool context for credential access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// Name of the tool accessing the credential
    pub tool_name: String,
    /// Operation being performed
    pub operation: String,
    /// Tool arguments
    pub arguments: Vec<String>,
    /// Risk level of the operation
    pub risk_level: RiskLevel,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(
        tool_name: String,
        operation: String,
        arguments: Vec<String>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            tool_name,
            operation,
            arguments,
            risk_level,
        }
    }
}

/// Risk level for tool operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low risk (read-only operations)
    Low,
    /// Medium risk (non-destructive changes)
    Medium,
    /// High risk (destructive changes)
    High,
    /// Critical risk (production deletions, etc.)
    Critical,
}

/// Detected anomaly in credential access pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAccessAnomaly {
    /// Agent with anomalous behavior
    pub agent_id: String,
    /// Credential type accessed
    pub credential_type: CredentialType,
    /// Anomaly score (0.0-1.0)
    pub anomaly_score: f64,
    /// Reasons for anomaly detection
    pub reasons: Vec<String>,
    /// Recommended action
    pub recommended_action: AnomalyAction,
}

impl CredentialAccessAnomaly {
    /// Create a new anomaly
    pub fn new(
        agent_id: String,
        credential_type: CredentialType,
        anomaly_score: f64,
        reasons: Vec<String>,
    ) -> Self {
        let recommended_action = AnomalyAction::from_score(anomaly_score);
        Self {
            agent_id,
            credential_type,
            anomaly_score,
            reasons,
            recommended_action,
        }
    }
}

/// Action to take based on anomaly score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalyAction {
    /// Allow access (score < 0.5)
    Allow,
    /// Log the access (score 0.5 - 0.7)
    Log,
    /// Alert administrators (score 0.7 - 0.8)
    Alert,
    /// Require manual approval (score > 0.8)
    RequireApproval,
    /// Block access (score > 0.95)
    Block,
}

impl AnomalyAction {
    /// Determine action from anomaly score
    pub fn from_score(score: f64) -> Self {
        if score > 0.95 {
            Self::Block
        } else if score > 0.8 {
            Self::RequireApproval
        } else if score > 0.7 {
            Self::Alert
        } else if score > 0.5 {
            Self::Log
        } else {
            Self::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_access_event_serialization() {
        let event = CredentialAccessEvent::new(
            "evt-123".to_string(),
            "agent-1".to_string(),
            CredentialType::Kubernetes,
            "/home/.kube/config".to_string(),
            AccessMode::Read,
            ToolContext::new(
                "kubectl".to_string(),
                "get pods".to_string(),
                vec!["get".to_string(), "pods".to_string()],
                RiskLevel::Low,
            ),
            1,
            "session-1".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CredentialAccessEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.agent_id, deserialized.agent_id);
        assert_eq!(event.credential_type, deserialized.credential_type);
    }

    #[test]
    fn test_credential_types_distinct() {
        assert_ne!(CredentialType::Kubernetes, CredentialType::Aws);
        assert_ne!(CredentialType::Gcp, CredentialType::Azure);
        assert_eq!(
            CredentialType::Custom("test".to_string()),
            CredentialType::Custom("test".to_string())
        );
    }

    #[test]
    fn test_anomaly_action_from_score() {
        assert_eq!(AnomalyAction::from_score(0.3), AnomalyAction::Allow);
        assert_eq!(AnomalyAction::from_score(0.6), AnomalyAction::Log);
        assert_eq!(AnomalyAction::from_score(0.75), AnomalyAction::Alert);
        assert_eq!(AnomalyAction::from_score(0.85), AnomalyAction::RequireApproval);
        assert_eq!(AnomalyAction::from_score(0.98), AnomalyAction::Block);
    }

    #[test]
    fn test_credential_type_name() {
        assert_eq!(CredentialType::Kubernetes.name(), "kubernetes");
        assert_eq!(CredentialType::Aws.name(), "aws");
        assert_eq!(CredentialType::Custom("test".to_string()).name(), "test");
    }

    #[test]
    fn test_tool_context_creation() {
        let ctx = ToolContext::new(
            "kubectl".to_string(),
            "delete pod".to_string(),
            vec!["delete".to_string(), "pod".to_string()],
            RiskLevel::High,
        );

        assert_eq!(ctx.tool_name, "kubectl");
        assert_eq!(ctx.operation, "delete pod");
        assert_eq!(ctx.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_anomaly_with_score() {
        let event = CredentialAccessEvent::new(
            "evt-1".to_string(),
            "agent-1".to_string(),
            CredentialType::Kubernetes,
            "/path".to_string(),
            AccessMode::Read,
            ToolContext::new(
                "kubectl".to_string(),
                "get pods".to_string(),
                vec![],
                RiskLevel::Low,
            ),
            1,
            "session-1".to_string(),
        )
        .with_anomaly_score(0.85);

        assert_eq!(event.anomaly_score, 0.85);
    }
}
