//! Unit tests for aof-memory backends

use aof_core::{MemoryBackend, MemoryEntry, MemoryQuery};
use aof_memory::InMemoryBackend;

#[tokio::test]
async fn test_in_memory_backend_store_retrieve() {
    let backend = InMemoryBackend::new();

    let entry = MemoryEntry::new("test_key", serde_json::json!({"data": "test_value"}));

    // Store
    backend.store("test_key", entry.clone()).await.unwrap();

    // Retrieve
    let retrieved = backend.retrieve("test_key").await.unwrap();
    assert!(retrieved.is_some());

    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.key, "test_key");
    assert_eq!(retrieved_entry.value, serde_json::json!({"data": "test_value"}));
}

#[tokio::test]
async fn test_in_memory_backend_retrieve_nonexistent() {
    let backend = InMemoryBackend::new();
    let result = backend.retrieve("nonexistent_key").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_in_memory_backend_delete() {
    let backend = InMemoryBackend::new();

    let entry = MemoryEntry::new("delete_me", serde_json::json!({"temp": true}));

    // Store
    backend.store("delete_me", entry).await.unwrap();

    // Verify it exists
    let before_delete = backend.retrieve("delete_me").await.unwrap();
    assert!(before_delete.is_some());

    // Delete
    backend.delete("delete_me").await.unwrap();

    // Verify it's gone
    let after_delete = backend.retrieve("delete_me").await.unwrap();
    assert!(after_delete.is_none());
}

#[tokio::test]
async fn test_in_memory_backend_list_keys() {
    let backend = InMemoryBackend::new();

    // Store multiple entries
    for i in 0..5 {
        let entry = MemoryEntry::new(format!("key_{}", i), serde_json::json!({"index": i}));
        backend.store(&format!("key_{}", i), entry).await.unwrap();
    }

    // List all keys
    let keys = backend.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 5);
    assert!(keys.contains(&"key_0".to_string()));
    assert!(keys.contains(&"key_4".to_string()));
}

#[tokio::test]
async fn test_in_memory_backend_search() {
    let backend = InMemoryBackend::new();

    // Store entries with metadata
    for i in 0..10 {
        let entry = MemoryEntry::new(
            format!("query_key_{}", i),
            serde_json::json!({"index": i})
        ).with_metadata("category", if i % 2 == 0 { "even" } else { "odd" });
        backend.store(&format!("query_key_{}", i), entry).await.unwrap();
    }

    let query = MemoryQuery {
        prefix: Some("query_key".to_string()),
        limit: Some(100),
        metadata: Default::default(),
        include_expired: false,
    };

    let results = backend.search(&query).await.unwrap();
    assert_eq!(results.len(), 10);
}

#[tokio::test]
async fn test_in_memory_backend_search_with_limit() {
    let backend = InMemoryBackend::new();

    // Store many entries
    for i in 0..20 {
        let entry = MemoryEntry::new(format!("limit_key_{}", i), serde_json::json!({"index": i}));
        backend.store(&format!("limit_key_{}", i), entry).await.unwrap();
    }

    let query = MemoryQuery {
        prefix: Some("limit_key".to_string()),
        limit: Some(5),
        metadata: Default::default(),
        include_expired: false,
    };

    let results = backend.search(&query).await.unwrap();
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_in_memory_backend_clear() {
    let backend = InMemoryBackend::new();

    // Store some entries
    for i in 0..5 {
        let entry = MemoryEntry::new(format!("clear_key_{}", i), serde_json::json!({"index": i}));
        backend.store(&format!("clear_key_{}", i), entry).await.unwrap();
    }

    // Verify entries exist
    let keys_before = backend.list_keys(None).await.unwrap();
    assert_eq!(keys_before.len(), 5);

    // Clear
    backend.clear().await.unwrap();

    // Verify all gone
    let keys_after = backend.list_keys(None).await.unwrap();
    assert_eq!(keys_after.len(), 0);
}

#[tokio::test]
async fn test_in_memory_backend_concurrent_access() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let backend = Arc::new(InMemoryBackend::new());
    let mut tasks = JoinSet::new();

    // Spawn concurrent writes
    for i in 0..10 {
        let backend_clone = backend.clone();
        tasks.spawn(async move {
            let entry = MemoryEntry::new(format!("concurrent_{}", i), serde_json::json!({"index": i}));
            backend_clone
                .store(&format!("concurrent_{}", i), entry)
                .await
                .unwrap();
        });
    }

    // Wait for all tasks
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }

    // Verify all entries stored
    let keys = backend.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 10);
}

#[tokio::test]
async fn test_in_memory_backend_update() {
    let backend = InMemoryBackend::new();

    let entry1 = MemoryEntry::new("update_key", serde_json::json!({"version": 1}));
    backend.store("update_key", entry1).await.unwrap();

    let entry2 = MemoryEntry::new("update_key", serde_json::json!({"version": 2}));
    backend.store("update_key", entry2).await.unwrap();

    let retrieved = backend.retrieve("update_key").await.unwrap().unwrap();
    assert_eq!(retrieved.value, serde_json::json!({"version": 2}));
}

#[tokio::test]
async fn test_memory_entry_ttl() {
    let entry_with_ttl = MemoryEntry::new("ttl_key", serde_json::json!({"data": "expires"}))
        .with_ttl(10);

    assert_eq!(entry_with_ttl.ttl, Some(10));
    assert!(!entry_with_ttl.is_expired());
}

#[tokio::test]
async fn test_memory_entry_metadata() {
    let entry = MemoryEntry::new("meta_key", serde_json::json!({"data": "test"}))
        .with_metadata("tag", "important")
        .with_metadata("category", "system");

    assert_eq!(entry.metadata.get("tag"), Some(&"important".to_string()));
    assert_eq!(entry.metadata.get("category"), Some(&"system".to_string()));
}
