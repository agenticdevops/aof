use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,                    // Unique ID
    pub timestamp: DateTime<Utc>,
    pub operator: String,              // User/agent who performed action
    pub operation: String,             // What was done (e.g., "delete_pod")
    pub category: String,              // Destructive/Risky/Safe
    pub resource: String,              // What was affected (e.g., "pod: nginx-123")
    pub approval_id: Option<String>,   // Link to approval request
    pub approval_decision: Option<String>, // Approved/Rejected/Auto-Approved
    pub result: OperationResult,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub approval_decision_by: Option<String>, // Who approved
    pub approval_decision_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,   // Additional context
    pub hash: String,                  // SHA256 hash for integrity
    pub previous_hash: String,         // Hash of previous entry (chain)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationResult {
    Success,
    Failed,
    Rejected,
    Timeout,
}

impl std::fmt::Display for OperationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Failed => write!(f, "Failed"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

/// Audit logger with file-based persistence
pub struct AuditLogger {
    log_path: PathBuf,
    cache: Arc<RwLock<Vec<AuditEvent>>>,
}

impl AuditLogger {
    pub fn new(log_path: PathBuf) -> std::io::Result<Self> {
        // Create log file if not exists
        if !log_path.exists() {
            File::create(&log_path)?;
        }

        // Load existing logs first before creating RwLock
        let mut cache = Vec::new();
        if log_path.exists() {
            let file = File::open(&log_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if !line.trim().is_empty() {
                    if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                        cache.push(event);
                    }
                }
            }
        }

        let logger = AuditLogger {
            log_path,
            cache: Arc::new(RwLock::new(cache)),
        };

        Ok(logger)
    }

    /// Log an operation
    pub async fn log_operation(
        &self,
        operator: &str,
        operation: &str,
        category: &str,
        resource: &str,
        result: OperationResult,
    ) -> std::io::Result<AuditEvent> {
        self.log_operation_full(
            AuditEventBuilder::new(operator, operation, category, resource)
                .result(result)
                .build(),
        )
        .await
    }

    /// Log with full details
    pub async fn log_operation_full(&self, mut event: AuditEvent) -> std::io::Result<AuditEvent> {
        // Calculate hash
        let previous_hash = {
            let cache = self.cache.read().await;
            cache
                .last()
                .map(|e| e.hash.clone())
                .unwrap_or_else(|| "genesis".to_string())
        };

        event.previous_hash = previous_hash;
        event.hash = self.calculate_event_hash(&event);

        // Append to file
        self.append_to_file(&event)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.push(event.clone());
        }

        Ok(event)
    }

    /// Get all audit logs
    pub async fn get_logs(&self) -> Vec<AuditEvent> {
        self.cache.read().await.clone()
    }

    /// Get logs filtered by operator
    pub async fn get_logs_by_operator(&self, operator: &str) -> Vec<AuditEvent> {
        self.cache
            .read()
            .await
            .iter()
            .filter(|e| e.operator == operator)
            .cloned()
            .collect()
    }

    /// Get logs filtered by time range
    pub async fn get_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditEvent> {
        self.cache
            .read()
            .await
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get logs filtered by operation
    pub async fn get_logs_by_operation(&self, operation: &str) -> Vec<AuditEvent> {
        self.cache
            .read()
            .await
            .iter()
            .filter(|e| e.operation.contains(operation))
            .cloned()
            .collect()
    }

    /// Verify audit trail integrity
    pub async fn verify_integrity(&self) -> bool {
        let logs = self.cache.read().await;

        let mut previous_hash = "genesis".to_string();
        for log in logs.iter() {
            if log.previous_hash != previous_hash {
                return false;
            }
            if log.hash != self.calculate_event_hash(log) {
                return false;
            }
            previous_hash = log.hash.clone();
        }

        true
    }

    // Private helpers
    fn calculate_event_hash(&self, event: &AuditEvent) -> String {
        // Create a copy with empty hash to calculate the hash
        let mut event_for_hashing = event.clone();
        event_for_hashing.hash = String::new();

        let serialized = serde_json::to_string(&event_for_hashing)
            .unwrap_or_else(|_| "".to_string());
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn append_to_file(&self, event: &AuditEvent) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string(event)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(writer, "{}", json)?;
        writer.flush()?;

        Ok(())
    }

}

/// Builder for constructing audit events
pub struct AuditEventBuilder {
    operator: String,
    operation: String,
    category: String,
    resource: String,
    approval_id: Option<String>,
    result: OperationResult,
    error_message: Option<String>,
    duration_ms: u64,
}

impl AuditEventBuilder {
    pub fn new(operator: &str, operation: &str, category: &str, resource: &str) -> Self {
        AuditEventBuilder {
            operator: operator.to_string(),
            operation: operation.to_string(),
            category: category.to_string(),
            resource: resource.to_string(),
            approval_id: None,
            result: OperationResult::Success,
            error_message: None,
            duration_ms: 0,
        }
    }

    pub fn result(mut self, result: OperationResult) -> Self {
        self.result = result;
        self
    }

    pub fn error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }

    pub fn approval_id(mut self, id: String) -> Self {
        self.approval_id = Some(id);
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operator: self.operator,
            operation: self.operation,
            category: self.category,
            resource: self.resource,
            approval_id: self.approval_id,
            approval_decision: None,
            result: self.result,
            error_message: self.error_message,
            duration_ms: self.duration_ms,
            approval_decision_by: None,
            approval_decision_at: None,
            metadata: serde_json::json!({}),
            hash: "".to_string(),     // Will be calculated
            previous_hash: "".to_string(), // Will be calculated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();

        let event = logger
            .log_operation("admin", "delete_pod", "Destructive", "pod: nginx-123", OperationResult::Success)
            .await
            .unwrap();

        assert!(!event.hash.is_empty());
        assert!(logger.verify_integrity().await);
    }

    #[tokio::test]
    async fn test_audit_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit_filter.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();

        // Log multiple events
        logger
            .log_operation("alice", "delete_pod", "Destructive", "pod: nginx-123", OperationResult::Success)
            .await
            .unwrap();

        logger
            .log_operation("bob", "get_pods", "Safe", "namespace: default", OperationResult::Success)
            .await
            .unwrap();

        logger
            .log_operation("alice", "update_config", "Risky", "configmap: app-config", OperationResult::Success)
            .await
            .unwrap();

        // Test filtering by operator
        let alice_logs = logger.get_logs_by_operator("alice").await;
        assert_eq!(alice_logs.len(), 2);

        // Test filtering by operation
        let delete_logs = logger.get_logs_by_operation("delete").await;
        assert_eq!(delete_logs.len(), 1);

        // Test integrity
        assert!(logger.verify_integrity().await);
    }

    #[tokio::test]
    async fn test_audit_hash_chain() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit_hash.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();

        let event1 = logger
            .log_operation("admin", "delete_pod", "Destructive", "pod: nginx-1", OperationResult::Success)
            .await
            .unwrap();

        let event2 = logger
            .log_operation("admin", "delete_pod", "Destructive", "pod: nginx-2", OperationResult::Success)
            .await
            .unwrap();

        // Event2's previous_hash should be event1's hash
        assert_eq!(event2.previous_hash, event1.hash);

        // Verify integrity
        assert!(logger.verify_integrity().await);
    }

    #[tokio::test]
    async fn test_audit_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit_persist.log");

        // Write some events
        {
            let logger = AuditLogger::new(log_path.clone()).unwrap();
            logger
                .log_operation("admin", "delete_pod", "Destructive", "pod: nginx-123", OperationResult::Success)
                .await
                .unwrap();
        }

        // Load same file again
        let logger = AuditLogger::new(log_path.clone()).unwrap();
        let logs = logger.get_logs().await;

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operator, "admin");
        assert_eq!(logs[0].operation, "delete_pod");
    }

    #[tokio::test]
    async fn test_operation_result_display() {
        assert_eq!(OperationResult::Success.to_string(), "Success");
        assert_eq!(OperationResult::Failed.to_string(), "Failed");
        assert_eq!(OperationResult::Rejected.to_string(), "Rejected");
        assert_eq!(OperationResult::Timeout.to_string(), "Timeout");
    }
}
