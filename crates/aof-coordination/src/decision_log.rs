//! Decision logging with append-only JSON Lines storage and hybrid search
//!
//! Provides DecisionLogger for recording agent decisions to persistent storage
//! and DecisionSearch for querying decisions via structured and semantic queries.

use aof_core::DecisionLogEntry;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, warn};

use crate::broadcaster::EventBroadcaster;
use aof_core::CoordinationEvent;

/// Result type for decision logging operations
pub type DecisionLogResult<T> = std::result::Result<T, DecisionLogError>;

/// Error type for decision logging
#[derive(Debug, Clone)]
pub enum DecisionLogError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    Utf8Error(String),
}

impl std::fmt::Display for DecisionLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionLogError::IoError(e) => write!(f, "IO error: {}", e),
            DecisionLogError::ParseError(e) => write!(f, "Parse error: {}", e),
            DecisionLogError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            DecisionLogError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for DecisionLogError {}

impl From<std::io::Error> for DecisionLogError {
    fn from(e: std::io::Error) -> Self {
        DecisionLogError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for DecisionLogError {
    fn from(e: serde_json::Error) -> Self {
        DecisionLogError::SerializeError(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for DecisionLogError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        DecisionLogError::Utf8Error(e.to_string())
    }
}

/// Append-only decision logger with JSON Lines storage
///
/// Logs decisions to a file in JSON Lines format (one JSON object per line)
/// and emits them to subscribers via EventBroadcaster.
#[derive(Clone)]
pub struct DecisionLogger {
    log_path: PathBuf,
    broadcaster: Arc<EventBroadcaster>,
}

impl DecisionLogger {
    /// Create a new decision logger
    ///
    /// # Arguments
    /// * `log_path` - Path to the JSON Lines log file
    /// * `broadcaster` - EventBroadcaster for real-time event streaming
    pub fn new(log_path: PathBuf, broadcaster: Arc<EventBroadcaster>) -> Self {
        Self {
            log_path,
            broadcaster,
        }
    }

    /// Log a decision entry
    ///
    /// Appends the entry as a JSON line to the log file and emits a CoordinationEvent.
    /// File I/O errors are returned, but broadcast errors are logged and ignored (best-effort).
    pub async fn log(&self, entry: DecisionLogEntry) -> DecisionLogResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.log_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Serialize entry to JSON
        let json = serde_json::to_string(&entry)?;

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await?;

        file.write_all(format!("{}\n", json).as_bytes()).await?;
        file.sync_all().await?;

        debug!(
            "Decision logged: agent={}, action={}, confidence={}",
            entry.agent_id, entry.action, entry.confidence
        );

        // Broadcast the decision event
        let event = CoordinationEvent::from_activity(
            aof_core::activity::ActivityEvent::new(
                aof_core::activity::ActivityType::Thinking,
                format!("Decision: {}", entry.action),
            ),
            entry.agent_id.clone(),
            "decision-log",
        );
        self.broadcaster.emit(event);

        Ok(())
    }

    /// Load recent decision entries from the log
    ///
    /// Reads the last N lines from the JSON Lines file in chronological order.
    /// Malformed lines are skipped with a warning.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return
    pub async fn load_recent(&self, limit: usize) -> DecisionLogResult<Vec<DecisionLogEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = tokio::fs::File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut entries = Vec::new();

        while let Some(line) = lines.next_line().await? {
            match serde_json::from_str::<DecisionLogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => warn!("Skipping malformed decision log line: {}", e),
            }
        }

        // Return last `limit` entries in chronological order
        if entries.len() > limit {
            Ok(entries[entries.len() - limit..].to_vec())
        } else {
            Ok(entries)
        }
    }
}

impl std::fmt::Debug for DecisionLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecisionLogger")
            .field("log_path", &self.log_path)
            .finish()
    }
}

/// Decision search with structured and semantic query support
///
/// Supports both structured queries (e.g., `agent=ops-bot AND confidence>0.8`)
/// and semantic queries (e.g., "what happened with pod crashes?").
#[derive(Clone)]
pub struct DecisionSearch {
    log_path: PathBuf,
}

impl DecisionSearch {
    /// Create a new decision search instance
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    /// Search for decisions
    ///
    /// Automatically detects query type (structured vs semantic) and routes to appropriate handler.
    /// For Phase 2, semantic queries fall back to tag-based matching.
    pub async fn search(&self, query: &str) -> DecisionLogResult<Vec<DecisionLogEntry>> {
        if Self::is_structured_query(query) {
            self.structured_search(query).await
        } else {
            self.semantic_search(query).await
        }
    }

    /// Structured search with SQL-like query syntax
    ///
    /// Supports queries like: `agent=ops-bot AND confidence>0.8 AND tags:incident`
    async fn structured_search(&self, query: &str) -> DecisionLogResult<Vec<DecisionLogEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = tokio::fs::File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut results = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if let Ok(entry) = serde_json::from_str::<DecisionLogEntry>(&line) {
                if Self::matches_query(&entry, query) {
                    results.push(entry);
                }
            }
        }

        Ok(results)
    }

    /// Semantic search using tag-based matching (Phase 2 fallback)
    ///
    /// For Phase 2, this uses simple keyword matching against tags and action.
    /// Future: Replace with embeddings-based semantic search.
    async fn semantic_search(&self, query: &str) -> DecisionLogResult<Vec<DecisionLogEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = tokio::fs::File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if let Ok(entry) = serde_json::from_str::<DecisionLogEntry>(&line) {
                // Simple tag matching: check if any tag contains query keywords
                let matches_tags = entry
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower));

                let matches_action =
                    entry.action.to_lowercase().contains(&query_lower) ||
                    entry.reasoning.to_lowercase().contains(&query_lower);

                if matches_tags || matches_action {
                    results.push(entry);
                }
            }
        }

        Ok(results)
    }

    /// Detect if query is structured or semantic
    fn is_structured_query(query: &str) -> bool {
        query.contains('=') || query.contains('>') || query.contains('<') || query.contains("AND")
    }

    /// Check if entry matches structured query predicates
    fn matches_query(entry: &DecisionLogEntry, query: &str) -> bool {
        // Simple predicate parsing: split by AND, evaluate each predicate
        for predicate in query.split("AND") {
            let predicate = predicate.trim();

            if predicate.contains('=') {
                let parts: Vec<&str> = predicate.split('=').collect();
                if parts.len() == 2 {
                    let (field, value) = (parts[0].trim(), parts[1].trim());
                    let value = value.trim_matches('\'').trim_matches('"');

                    match field {
                        "agent" => {
                            if !entry.agent_id.contains(value) {
                                return false;
                            }
                        }
                        "action" => {
                            if !entry.action.contains(value) {
                                return false;
                            }
                        }
                        "tags" => {
                            if !entry.tags.iter().any(|t| t.contains(value)) {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }
            } else if predicate.contains('>') {
                let parts: Vec<&str> = predicate.split('>').collect();
                if parts.len() == 2 {
                    let (field, value) = (parts[0].trim(), parts[1].trim());
                    if field == "confidence" {
                        if let Ok(threshold) = value.parse::<f64>() {
                            if entry.confidence <= threshold {
                                return false;
                            }
                        }
                    }
                }
            } else if predicate.contains('<') {
                let parts: Vec<&str> = predicate.split('<').collect();
                if parts.len() == 2 {
                    let (field, value) = (parts[0].trim(), parts[1].trim());
                    if field == "confidence" {
                        if let Ok(threshold) = value.parse::<f64>() {
                            if entry.confidence >= threshold {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }
}

impl std::fmt::Debug for DecisionSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecisionSearch")
            .field("log_path", &self.log_path)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_decision_logger_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster);

        let entry = DecisionLogEntry::new("agent-1", "test_action", "test reasoning", 0.9);
        assert!(logger.log(entry).await.is_ok());

        assert!(log_path.exists());
    }

    #[tokio::test]
    async fn test_decision_logger_append_mode() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster);

        let entry1 = DecisionLogEntry::new("agent-1", "action1", "reasoning1", 0.8);
        let entry2 = DecisionLogEntry::new("agent-2", "action2", "reasoning2", 0.9);

        assert!(logger.log(entry1).await.is_ok());
        assert!(logger.log(entry2).await.is_ok());

        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        let line_count = content.lines().count();
        assert_eq!(line_count, 2);
    }

    #[tokio::test]
    async fn test_decision_logger_load_recent() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster);

        for i in 1..=5 {
            let entry = DecisionLogEntry::new(
                format!("agent-{}", i),
                format!("action-{}", i),
                format!("reasoning-{}", i),
                0.5 + (i as f64) * 0.1,
            );
            assert!(logger.log(entry).await.is_ok());
        }

        let recent = logger.load_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn test_structured_search_by_agent() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster.clone());

        let entry1 = DecisionLogEntry::new("agent-1", "restart", "pod crash", 0.9);
        let entry2 = DecisionLogEntry::new("agent-2", "scale", "load increase", 0.8);

        assert!(logger.log(entry1).await.is_ok());
        assert!(logger.log(entry2).await.is_ok());

        let search = DecisionSearch::new(log_path);
        let results = search.search("agent=agent-1").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id, "agent-1");
    }

    #[tokio::test]
    async fn test_structured_search_by_confidence() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster);

        let entry1 = DecisionLogEntry::new("agent-1", "action1", "reasoning1", 0.9);
        let entry2 = DecisionLogEntry::new("agent-2", "action2", "reasoning2", 0.6);
        let entry3 = DecisionLogEntry::new("agent-3", "action3", "reasoning3", 0.95);

        assert!(logger.log(entry1).await.is_ok());
        assert!(logger.log(entry2).await.is_ok());
        assert!(logger.log(entry3).await.is_ok());

        let search = DecisionSearch::new(log_path);
        let results = search.search("confidence>0.8").await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_semantic_search_by_tags() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("decisions.jsonl");

        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let logger = DecisionLogger::new(log_path.clone(), broadcaster);

        let entry1 = DecisionLogEntry::new("agent-1", "action1", "reasoning1", 0.9)
            .with_tags(vec!["incident".to_string(), "critical".to_string()]);

        let entry2 = DecisionLogEntry::new("agent-2", "action2", "reasoning2", 0.8)
            .with_tags(vec!["routine".to_string()]);

        assert!(logger.log(entry1).await.is_ok());
        assert!(logger.log(entry2).await.is_ok());

        let search = DecisionSearch::new(log_path);
        let results = search.search("incident").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id, "agent-1");
    }

    #[tokio::test]
    async fn test_query_type_detection() {
        assert!(DecisionSearch::is_structured_query("agent=ops-bot"));
        assert!(DecisionSearch::is_structured_query("confidence>0.8"));
        assert!(DecisionSearch::is_structured_query("agent=x AND confidence>0.7"));
        assert!(!DecisionSearch::is_structured_query("what happened with pods?"));
    }
}
