//! In-memory backend using DashMap for lock-free concurrent access

use aof_core::{AofResult, MemoryBackend, MemoryEntry, MemoryQuery};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// High-performance in-memory backend using DashMap
///
/// Provides lock-free concurrent access to memory entries with automatic
/// TTL expiry on read (lazy cleanup).
#[derive(Clone)]
pub struct InMemoryBackend {
    /// DashMap for lock-free concurrent access
    store: Arc<DashMap<String, MemoryEntry>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Arc::new(DashMap::with_capacity(capacity)),
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryBackend for InMemoryBackend {
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()> {
        self.store.insert(key.to_string(), entry);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>> {
        match self.store.get(key) {
            Some(entry_ref) => {
                let entry = entry_ref.value().clone();
                drop(entry_ref); // Release read lock

                // Check TTL expiry (lazy cleanup)
                if entry.is_expired() {
                    self.store.remove(key);
                    Ok(None)
                } else {
                    Ok(Some(entry))
                }
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> AofResult<()> {
        self.store.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> AofResult<Vec<String>> {
        let keys: Vec<String> = match prefix {
            Some(p) => self
                .store
                .iter()
                .filter(|entry| entry.key().starts_with(p))
                .map(|entry| entry.key().clone())
                .collect(),
            None => self.store.iter().map(|entry| entry.key().clone()).collect(),
        };
        Ok(keys)
    }

    async fn clear(&self) -> AofResult<()> {
        self.store.clear();
        Ok(())
    }

    async fn search(&self, query: &MemoryQuery) -> AofResult<Vec<MemoryEntry>> {
        let mut results = Vec::new();

        // Filter entries based on prefix and metadata
        for entry_ref in self.store.iter() {
            let entry = entry_ref.value();

            // Check prefix filter
            if let Some(ref prefix) = query.prefix {
                if !entry.key.starts_with(prefix) {
                    continue;
                }
            }

            // Check if entry matches query
            if query.matches(entry) {
                results.push(entry.clone());

                // Check limit
                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let backend = InMemoryBackend::new();
        let entry = MemoryEntry::new("test_key", json!({"data": "test"}));

        backend.store("test_key", entry.clone()).await.unwrap();

        let retrieved = backend.retrieve("test_key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key, "test_key");
    }

    #[tokio::test]
    async fn test_delete() {
        let backend = InMemoryBackend::new();
        let entry = MemoryEntry::new("test_key", json!({"data": "test"}));

        backend.store("test_key", entry).await.unwrap();
        backend.delete("test_key").await.unwrap();

        let retrieved = backend.retrieve("test_key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_keys_all() {
        let backend = InMemoryBackend::new();

        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();
        backend
            .store("key2", MemoryEntry::new("key2", json!(2)))
            .await
            .unwrap();
        backend
            .store("prefix:key3", MemoryEntry::new("prefix:key3", json!(3)))
            .await
            .unwrap();

        let keys = backend.list_keys(None).await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"prefix:key3".to_string()));
    }

    #[tokio::test]
    async fn test_list_keys_with_prefix() {
        let backend = InMemoryBackend::new();

        backend
            .store("user:1", MemoryEntry::new("user:1", json!(1)))
            .await
            .unwrap();
        backend
            .store("user:2", MemoryEntry::new("user:2", json!(2)))
            .await
            .unwrap();
        backend
            .store("admin:1", MemoryEntry::new("admin:1", json!(3)))
            .await
            .unwrap();

        let keys = backend.list_keys(Some("user:")).await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"user:1".to_string()));
        assert!(keys.contains(&"user:2".to_string()));
        assert!(!keys.contains(&"admin:1".to_string()));
    }

    #[tokio::test]
    async fn test_clear() {
        let backend = InMemoryBackend::new();

        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();
        backend
            .store("key2", MemoryEntry::new("key2", json!(2)))
            .await
            .unwrap();

        backend.clear().await.unwrap();

        let keys = backend.list_keys(None).await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let backend = InMemoryBackend::new();
        let entry = MemoryEntry::new("ttl_key", json!({"data": "test"})).with_ttl(1); // 1 second TTL

        backend.store("ttl_key", entry).await.unwrap();

        // Should retrieve immediately
        let retrieved = backend.retrieve("ttl_key").await.unwrap();
        assert!(retrieved.is_some());

        // Wait for expiry
        sleep(Duration::from_millis(1100)).await;

        // Should be expired and removed (lazy cleanup)
        let retrieved = backend.retrieve("ttl_key").await.unwrap();
        assert!(retrieved.is_none());

        // Verify it's actually removed from storage
        assert_eq!(backend.len(), 0);
    }

    #[tokio::test]
    async fn test_search_with_metadata() {
        let backend = InMemoryBackend::new();

        let entry1 = MemoryEntry::new("key1", json!(1))
            .with_metadata("type", "user")
            .with_metadata("role", "admin");

        let entry2 = MemoryEntry::new("key2", json!(2))
            .with_metadata("type", "user")
            .with_metadata("role", "viewer");

        let entry3 = MemoryEntry::new("key3", json!(3)).with_metadata("type", "system");

        backend.store("key1", entry1).await.unwrap();
        backend.store("key2", entry2).await.unwrap();
        backend.store("key3", entry3).await.unwrap();

        // Search for users
        let mut query = MemoryQuery::default();
        query.metadata.insert("type".to_string(), "user".to_string());

        let results = backend.search(&query).await.unwrap();
        assert_eq!(results.len(), 2);

        // Search for admin users
        query
            .metadata
            .insert("role".to_string(), "admin".to_string());
        let results = backend.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "key1");
    }

    #[tokio::test]
    async fn test_search_with_prefix() {
        let backend = InMemoryBackend::new();

        backend
            .store("user:1", MemoryEntry::new("user:1", json!(1)))
            .await
            .unwrap();
        backend
            .store("user:2", MemoryEntry::new("user:2", json!(2)))
            .await
            .unwrap();
        backend
            .store("admin:1", MemoryEntry::new("admin:1", json!(3)))
            .await
            .unwrap();

        let query = MemoryQuery {
            prefix: Some("user:".to_string()),
            ..Default::default()
        };

        let results = backend.search(&query).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_search_with_limit() {
        let backend = InMemoryBackend::new();

        for i in 0..10 {
            backend
                .store(
                    &format!("key{}", i),
                    MemoryEntry::new(format!("key{}", i), json!(i)),
                )
                .await
                .unwrap();
        }

        let query = MemoryQuery {
            limit: Some(5),
            ..Default::default()
        };

        let results = backend.search(&query).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let backend = Arc::new(InMemoryBackend::new());
        let mut handles = vec![];

        // Spawn multiple concurrent tasks
        for i in 0..100 {
            let backend_clone = Arc::clone(&backend);
            let handle = tokio::spawn(async move {
                let key = format!("key{}", i);
                let entry = MemoryEntry::new(&key, json!(i));
                backend_clone.store(&key, entry).await.unwrap();

                let retrieved = backend_clone.retrieve(&key).await.unwrap();
                assert!(retrieved.is_some());
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(backend.len(), 100);
    }

    #[tokio::test]
    async fn test_with_capacity() {
        let backend = InMemoryBackend::with_capacity(100);
        assert_eq!(backend.len(), 0);
        assert!(backend.is_empty());

        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();

        assert_eq!(backend.len(), 1);
        assert!(!backend.is_empty());
    }
}
