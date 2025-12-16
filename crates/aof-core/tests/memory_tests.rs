//! Unit tests for aof-core memory traits

use aof_core::{AofResult, MemoryBackend, MemoryEntry, MemoryQuery};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock memory backend for testing
struct MockMemory {
    data: Arc<Mutex<HashMap<String, MemoryEntry>>>,
}

impl MockMemory {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MemoryBackend for MockMemory {
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()> {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), entry);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(key).cloned())
    }

    async fn delete(&self, key: &str) -> AofResult<()> {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> AofResult<Vec<String>> {
        let data = self.data.lock().unwrap();
        let keys: Vec<String> = if let Some(pfx) = prefix {
            data.keys()
                .filter(|k| k.starts_with(pfx))
                .cloned()
                .collect()
        } else {
            data.keys().cloned().collect()
        };
        Ok(keys)
    }

    async fn clear(&self) -> AofResult<()> {
        let mut data = self.data.lock().unwrap();
        data.clear();
        Ok(())
    }
}

#[tokio::test]
async fn test_memory_store_and_retrieve() {
    let memory = MockMemory::new();
    let entry = MemoryEntry::new("test_key", serde_json::json!({"data": "test_value"}));

    // Store entry
    memory.store("test_key", entry.clone()).await.unwrap();

    // Retrieve entry
    let retrieved = memory.retrieve("test_key").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.key, "test_key");
    assert_eq!(retrieved_entry.value, serde_json::json!({"data": "test_value"}));
}

#[tokio::test]
async fn test_memory_retrieve_nonexistent() {
    let memory = MockMemory::new();
    let retrieved = memory.retrieve("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_memory_delete() {
    let memory = MockMemory::new();
    let entry = MemoryEntry::new("delete_me", serde_json::json!({}));

    memory.store("delete_me", entry).await.unwrap();

    // Verify it exists
    assert!(memory.retrieve("delete_me").await.unwrap().is_some());

    // Delete it
    memory.delete("delete_me").await.unwrap();

    // Verify it's gone
    assert!(memory.retrieve("delete_me").await.unwrap().is_none());
}

#[tokio::test]
async fn test_memory_clear() {
    let memory = MockMemory::new();

    // Store multiple entries
    for i in 0..5 {
        let entry = MemoryEntry::new(format!("key_{}", i), serde_json::json!({"index": i}));
        memory.store(&format!("key_{}", i), entry).await.unwrap();
    }

    // Clear all
    memory.clear().await.unwrap();

    // Verify all gone
    for i in 0..5 {
        assert!(memory.retrieve(&format!("key_{}", i)).await.unwrap().is_none());
    }
}

#[tokio::test]
async fn test_memory_list_keys() {
    let memory = MockMemory::new();

    // Store test entries
    for i in 0..3 {
        let entry = MemoryEntry::new(format!("query_key_{}", i), serde_json::json!({"value": i}));
        memory.store(&format!("query_key_{}", i), entry).await.unwrap();
    }

    let keys = memory.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 3);

    // Test prefix filtering
    let prefixed_keys = memory.list_keys(Some("query_")).await.unwrap();
    assert_eq!(prefixed_keys.len(), 3);
}

#[tokio::test]
async fn test_memory_entry_serialization() {
    let entry = MemoryEntry::new("test", serde_json::json!({"field": "value"}))
        .with_metadata("source", "test");

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: MemoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.key, "test");
    assert_eq!(deserialized.metadata.get("source"), Some(&"test".to_string()));
}
