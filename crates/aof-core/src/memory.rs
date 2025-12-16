use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::AofResult;

/// Memory backend trait - pluggable persistence for agent state
///
/// Implementations should optimize for read performance and minimize allocations.
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Store memory entry
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()>;

    /// Retrieve memory entry
    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>>;

    /// Delete memory entry
    async fn delete(&self, key: &str) -> AofResult<()>;

    /// List keys (with optional prefix filter)
    async fn list_keys(&self, prefix: Option<&str>) -> AofResult<Vec<String>>;

    /// Clear all entries
    async fn clear(&self) -> AofResult<()>;

    /// Search entries by metadata
    async fn search(&self, query: &MemoryQuery) -> AofResult<Vec<MemoryEntry>> {
        // Default implementation: filter in-memory
        let keys = self.list_keys(query.prefix.as_deref()).await?;
        let mut results = Vec::new();

        for key in keys {
            if let Some(entry) = self.retrieve(&key).await? {
                if query.matches(&entry) {
                    results.push(entry);
                }
            }
        }

        Ok(results)
    }
}

/// High-level memory interface for agents
#[async_trait]
pub trait Memory: Send + Sync {
    /// Store a value
    async fn store(&self, key: &str, value: serde_json::Value) -> AofResult<()>;

    /// Retrieve a value
    async fn retrieve<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<Option<T>>;

    /// Delete a value
    async fn delete(&self, key: &str) -> AofResult<()>;

    /// List all keys
    async fn list_keys(&self) -> AofResult<Vec<String>>;

    /// Clear all memory
    async fn clear(&self) -> AofResult<()>;
}

/// Memory entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Entry key
    pub key: String,

    /// Entry value (JSON)
    pub value: serde_json::Value,

    /// Timestamp (Unix epoch ms)
    pub timestamp: u64,

    /// Entry metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// TTL (seconds, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

impl MemoryEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            value,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::new(),
            ttl: None,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl = Some(ttl_secs);
        self
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let expiry = self.timestamp + (ttl * 1000);
            now > expiry
        } else {
            false
        }
    }
}

/// Memory query for searching
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Key prefix filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Metadata filters (key-value pairs)
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Limit results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Include expired entries
    #[serde(default)]
    pub include_expired: bool,
}

impl MemoryQuery {
    /// Check if entry matches query
    pub fn matches(&self, entry: &MemoryEntry) -> bool {
        // Check expiry
        if !self.include_expired && entry.is_expired() {
            return false;
        }

        // Check metadata filters
        for (key, value) in &self.metadata {
            if entry.metadata.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

/// Reference-counted memory backend
pub type MemoryBackendRef = Arc<dyn MemoryBackend>;

/// Reference-counted memory
pub type MemoryRef = Arc<dyn Memory>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_new() {
        let entry = MemoryEntry::new("test-key", serde_json::json!({"value": 42}));

        assert_eq!(entry.key, "test-key");
        assert_eq!(entry.value, serde_json::json!({"value": 42}));
        assert!(entry.timestamp > 0);
        assert!(entry.metadata.is_empty());
        assert!(entry.ttl.is_none());
    }

    #[test]
    fn test_memory_entry_with_metadata() {
        let entry = MemoryEntry::new("key", serde_json::json!(null))
            .with_metadata("type", "session")
            .with_metadata("agent", "test-agent");

        assert_eq!(entry.metadata.get("type"), Some(&"session".to_string()));
        assert_eq!(entry.metadata.get("agent"), Some(&"test-agent".to_string()));
    }

    #[test]
    fn test_memory_entry_with_ttl() {
        let entry = MemoryEntry::new("key", serde_json::json!(null))
            .with_ttl(3600);

        assert_eq!(entry.ttl, Some(3600));
    }

    #[test]
    fn test_memory_entry_is_expired() {
        // Entry without TTL should never expire
        let entry_no_ttl = MemoryEntry::new("key", serde_json::json!(null));
        assert!(!entry_no_ttl.is_expired());

        // Entry with very long TTL should not be expired
        let entry_long_ttl = MemoryEntry::new("key", serde_json::json!(null))
            .with_ttl(3600); // 1 hour
        assert!(!entry_long_ttl.is_expired());

        // Create an entry with timestamp in the past to test expiry
        let mut entry_expired = MemoryEntry::new("key", serde_json::json!(null))
            .with_ttl(1); // 1 second TTL
        // Set timestamp to 2 seconds ago (in milliseconds)
        entry_expired.timestamp -= 2000;
        assert!(entry_expired.is_expired());
    }

    #[test]
    fn test_memory_entry_serialization() {
        let entry = MemoryEntry::new("my-key", serde_json::json!({"data": "test"}))
            .with_metadata("source", "api")
            .with_ttl(60);

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: MemoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.key, "my-key");
        assert_eq!(deserialized.value, serde_json::json!({"data": "test"}));
        assert_eq!(deserialized.metadata.get("source"), Some(&"api".to_string()));
        assert_eq!(deserialized.ttl, Some(60));
    }

    #[test]
    fn test_memory_query_default() {
        let query = MemoryQuery::default();

        assert!(query.prefix.is_none());
        assert!(query.metadata.is_empty());
        assert!(query.limit.is_none());
        assert!(!query.include_expired);
    }

    #[test]
    fn test_memory_query_matches_basic() {
        let query = MemoryQuery::default();
        let entry = MemoryEntry::new("key", serde_json::json!(null));

        assert!(query.matches(&entry));
    }

    #[test]
    fn test_memory_query_matches_metadata() {
        let mut query = MemoryQuery::default();
        query.metadata.insert("type".to_string(), "session".to_string());

        // Entry without metadata shouldn't match
        let entry_no_meta = MemoryEntry::new("key", serde_json::json!(null));
        assert!(!query.matches(&entry_no_meta));

        // Entry with matching metadata should match
        let entry_match = MemoryEntry::new("key", serde_json::json!(null))
            .with_metadata("type", "session");
        assert!(query.matches(&entry_match));

        // Entry with wrong metadata value shouldn't match
        let entry_wrong = MemoryEntry::new("key", serde_json::json!(null))
            .with_metadata("type", "permanent");
        assert!(!query.matches(&entry_wrong));
    }

    #[test]
    fn test_memory_query_matches_expired() {
        // Create an entry with timestamp in the past to test expiry
        let mut entry_expired = MemoryEntry::new("key", serde_json::json!(null))
            .with_ttl(1); // 1 second TTL
        // Set timestamp to 2 seconds ago (in milliseconds) to ensure it's expired
        entry_expired.timestamp -= 2000;

        // Default query excludes expired entries
        let query_default = MemoryQuery::default();
        assert!(!query_default.matches(&entry_expired));

        // Query that includes expired entries
        let query_include = MemoryQuery {
            include_expired: true,
            ..Default::default()
        };
        assert!(query_include.matches(&entry_expired));
    }

    #[test]
    fn test_memory_query_serialization() {
        let mut query = MemoryQuery {
            prefix: Some("agent:".to_string()),
            metadata: HashMap::new(),
            limit: Some(100),
            include_expired: true,
        };
        query.metadata.insert("type".to_string(), "context".to_string());

        let json = serde_json::to_string(&query).unwrap();
        let deserialized: MemoryQuery = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.prefix, Some("agent:".to_string()));
        assert_eq!(deserialized.limit, Some(100));
        assert!(deserialized.include_expired);
    }
}
