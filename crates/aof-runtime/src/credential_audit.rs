//! Credential access monitoring and audit logging
//!
//! This module provides the CredentialAccessInterceptor for monitoring credential
//! access patterns, logging accesses with tamper-proof sequencing, and detecting
//! anomalies.

use aof_core::credential::{
    AccessMode, CredentialAccessAnomaly, CredentialAccessEvent, CredentialType, RiskLevel,
    ToolContext,
};
use aof_core::error::{AofError, AofResult};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::credential_anomaly::AnomalyDetector;

/// Credential access interceptor for audit logging
pub struct CredentialAccessInterceptor {
    audit_log_path: PathBuf,
    sequence_counter: AtomicU64,
    anomaly_detector: Arc<AnomalyDetector>,
}

impl CredentialAccessInterceptor {
    /// Create a new credential access interceptor
    ///
    /// # Arguments
    /// * `audit_log_path` - Path to the audit log file (will be created if doesn't exist)
    /// * `anomaly_detector` - Shared anomaly detector for behavioral analysis
    pub fn new(audit_log_path: PathBuf, anomaly_detector: Arc<AnomalyDetector>) -> Self {
        Self {
            audit_log_path,
            sequence_counter: AtomicU64::new(1),
            anomaly_detector,
        }
    }

    /// Detect which credentials a tool invocation will access
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool being executed
    /// * `args` - Tool arguments
    ///
    /// # Returns
    /// Vector of credential types the tool is likely to access
    pub fn detect_credential_requirements(
        &self,
        tool_name: &str,
        _args: &[String],
    ) -> Vec<CredentialType> {
        match tool_name {
            "kubectl" | "k9s" => vec![CredentialType::Kubernetes],
            "aws" | "aws-cli" => vec![CredentialType::Aws],
            "gcloud" | "gsutil" => vec![CredentialType::Gcp],
            "az" => vec![CredentialType::Azure],
            "git" | "gh" => vec![CredentialType::Git],
            "psql" | "mysql" | "redis-cli" | "mongosh" => vec![CredentialType::Database],
            "vault" => vec![CredentialType::Vault],
            _ => vec![],
        }
    }

    /// Log a credential access event with tamper-proof sequencing
    ///
    /// # Arguments
    /// * `event` - The credential access event to log
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn log_access(&self, event: CredentialAccessEvent) -> AofResult<()> {
        // Serialize event to JSON
        let json_line = serde_json::to_string(&event)
            .map_err(|e| AofError::memory(format!("Failed to serialize audit event: {}", e)))?;

        // Append to audit log with newline
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)
            .await
            .map_err(|e| {
                AofError::memory(format!(
                    "Failed to open audit log {}: {}",
                    self.audit_log_path.display(),
                    e
                ))
            })?;

        file.write_all(format!("{}\n", json_line).as_bytes())
            .await
            .map_err(|e| AofError::memory(format!("Failed to write audit event: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| AofError::memory(format!("Failed to sync audit log: {}", e)))?;

        tracing::debug!(
            "Logged credential access: agent={}, type={:?}, seq={}",
            event.agent_id,
            event.credential_type,
            event.sequence_number
        );

        Ok(())
    }

    /// Check access against anomaly detector and return anomaly details
    ///
    /// # Arguments
    /// * `agent_id` - Agent requesting access
    /// * `credential_type` - Type of credential being accessed
    ///
    /// # Returns
    /// Anomaly details with score and recommended action
    pub async fn check_access(
        &self,
        agent_id: &str,
        credential_type: &CredentialType,
    ) -> CredentialAccessAnomaly {
        self.anomaly_detector
            .score_access(agent_id, credential_type)
            .await
    }

    /// Query audit log entries for a time range
    ///
    /// # Arguments
    /// * `since` - Start of time range
    /// * `until` - End of time range
    ///
    /// # Returns
    /// Vector of credential access events in the time range
    pub async fn query_log(
        &self,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> AofResult<Vec<CredentialAccessEvent>> {
        let content = tokio::fs::read_to_string(&self.audit_log_path)
            .await
            .map_err(|e| {
                AofError::memory(format!(
                    "Failed to read audit log {}: {}",
                    self.audit_log_path.display(),
                    e
                ))
            })?;

        let mut events = Vec::new();
        let mut prev_seq = 0u64;

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let event: CredentialAccessEvent = serde_json::from_str(line).map_err(|e| {
                AofError::memory(format!("Failed to parse audit log line {}: {}", line_num + 1, e))
            })?;

            // Check for sequence gaps (tamper detection)
            if prev_seq > 0 && event.sequence_number != prev_seq + 1 {
                tracing::warn!(
                    "Audit log sequence gap detected: expected {}, got {} (line {})",
                    prev_seq + 1,
                    event.sequence_number,
                    line_num + 1
                );
            }
            prev_seq = event.sequence_number;

            // Filter by time range
            if event.timestamp >= since && event.timestamp <= until {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Get the next sequence number for a new event
    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Create a credential access event
    pub fn create_event(
        &self,
        agent_id: String,
        credential_type: CredentialType,
        file_path: String,
        access_mode: AccessMode,
        tool_context: ToolContext,
        session_id: String,
    ) -> CredentialAccessEvent {
        let sequence = self.next_sequence();
        let event_id = format!("evt-{}-{}", Utc::now().timestamp_millis(), sequence);

        CredentialAccessEvent::new(
            event_id,
            agent_id,
            credential_type,
            file_path,
            access_mode,
            tool_context,
            sequence,
            session_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_interceptor() -> (CredentialAccessInterceptor, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!("aof-test-audit-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let audit_log = temp_dir.join("audit.log");

        let detector = Arc::new(AnomalyDetector::new());
        let interceptor = CredentialAccessInterceptor::new(audit_log.clone(), detector);

        (interceptor, temp_dir)
    }

    #[test]
    fn test_detect_credential_requirements_kubectl() {
        let (interceptor, _temp_dir) = setup_test_interceptor();
        let creds = interceptor.detect_credential_requirements("kubectl", &[]);
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0], CredentialType::Kubernetes);
    }

    #[test]
    fn test_detect_credential_requirements_aws() {
        let (interceptor, _temp_dir) = setup_test_interceptor();
        let creds = interceptor.detect_credential_requirements("aws", &[]);
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0], CredentialType::Aws);
    }

    #[test]
    fn test_detect_credential_requirements_unknown() {
        let (interceptor, _temp_dir) = setup_test_interceptor();
        let creds = interceptor.detect_credential_requirements("unknown-tool", &[]);
        assert_eq!(creds.len(), 0);
    }

    #[tokio::test]
    async fn test_log_access() {
        let (interceptor, _temp_dir) = setup_test_interceptor();

        let event = interceptor.create_event(
            "agent-1".to_string(),
            CredentialType::Kubernetes,
            "/home/.kube/config".to_string(),
            AccessMode::Read,
            ToolContext::new(
                "kubectl".to_string(),
                "get pods".to_string(),
                vec![],
                RiskLevel::Low,
            ),
            "session-1".to_string(),
        );

        let result = interceptor.log_access(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sequence_numbers_monotonic() {
        let (interceptor, _temp_dir) = setup_test_interceptor();

        let seq1 = interceptor.next_sequence();
        let seq2 = interceptor.next_sequence();
        let seq3 = interceptor.next_sequence();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }

    #[tokio::test]
    async fn test_query_log_time_range() {
        let (interceptor, _temp_dir) = setup_test_interceptor();

        let now = Utc::now();
        let event = interceptor.create_event(
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
            "session-1".to_string(),
        );

        interceptor.log_access(event).await.unwrap();

        let events = interceptor
            .query_log(now - chrono::Duration::minutes(1), now + chrono::Duration::minutes(1))
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
    }
}
