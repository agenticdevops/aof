//! Memory backend implementations

pub mod memory;

use aof_core::{AofError, AofResult, Memory, MemoryBackend, MemoryEntry};
use async_trait::async_trait;
use std::sync::Arc;

/// Simple memory wrapper that implements the Memory trait
///
/// Provides a high-level interface over MemoryBackend for easier usage.
pub struct SimpleMemory {
    backend: Arc<dyn MemoryBackend>,
}

impl SimpleMemory {
    /// Create a new SimpleMemory with the given backend
    pub fn new(backend: Arc<dyn MemoryBackend>) -> Self {
        Self { backend }
    }

    /// Create a new SimpleMemory with InMemoryBackend
    pub fn in_memory() -> Self {
        Self::new(Arc::new(memory::InMemoryBackend::new()))
    }
}

#[async_trait]
impl Memory for SimpleMemory {
    async fn store(&self, key: &str, value: serde_json::Value) -> AofResult<()> {
        let entry = MemoryEntry::new(key, value);
        self.backend.store(key, entry).await
    }

    async fn retrieve<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<Option<T>> {
        match self.backend.retrieve(key).await? {
            Some(entry) => {
                if entry.is_expired() {
                    // Lazy cleanup: delete expired entry
                    let _ = self.backend.delete(key).await;
                    Ok(None)
                } else {
                    let value = serde_json::from_value(entry.value)
                        .map_err(|e| AofError::memory(format!("Failed to deserialize value: {}", e)))?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> AofResult<()> {
        self.backend.delete(key).await
    }

    async fn list_keys(&self) -> AofResult<Vec<String>> {
        self.backend.list_keys(None).await
    }

    async fn clear(&self) -> AofResult<()> {
        self.backend.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_memory_store_retrieve() {
        let memory = SimpleMemory::in_memory();

        // Store value
        memory.store("key1", json!({"name": "test"})).await.unwrap();

        // Retrieve value
        let result: Option<serde_json::Value> = memory.retrieve("key1").await.unwrap();
        assert_eq!(result, Some(json!({"name": "test"})));
    }

    #[tokio::test]
    async fn test_simple_memory_delete() {
        let memory = SimpleMemory::in_memory();

        memory.store("key1", json!({"name": "test"})).await.unwrap();
        memory.delete("key1").await.unwrap();

        let result: Option<serde_json::Value> = memory.retrieve("key1").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_simple_memory_list_keys() {
        let memory = SimpleMemory::in_memory();

        memory.store("key1", json!(1)).await.unwrap();
        memory.store("key2", json!(2)).await.unwrap();
        memory.store("key3", json!(3)).await.unwrap();

        let keys = memory.list_keys().await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_simple_memory_clear() {
        let memory = SimpleMemory::in_memory();

        memory.store("key1", json!(1)).await.unwrap();
        memory.store("key2", json!(2)).await.unwrap();

        memory.clear().await.unwrap();

        let keys = memory.list_keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }
}
