use agentix_core::vector_memory::VectorMemoryBackend;
use agentix_memory::SqliteVectorBackend;
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::test]
async fn test_sqlite_store_and_recall() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-memory.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    // Store two embeddings for agent-a
    let embed_a1 = vec![1.0_f32, 0.0, 0.0];
    let embed_a2 = vec![0.9_f32, 0.1, 0.0];
    backend
        .store_embedding("agent-a", "run-1", "pg replication lag was 2ms", embed_a1.clone(), HashMap::new())
        .await.unwrap();
    backend
        .store_embedding("agent-a", "run-2", "pg replication lag was 5ms — check network", embed_a2.clone(), HashMap::new())
        .await.unwrap();

    // Query with similar embedding — should return both, ordered by similarity
    let query = vec![1.0_f32, 0.0, 0.0];
    let results = backend.search_similar("agent-a", &query, 2).await.unwrap();
    assert_eq!(results.len(), 2);
    // First result should be highest similarity (exact match = run-1)
    assert_eq!(results[0].run_id, "run-1");
}

#[tokio::test]
async fn test_sqlite_agent_isolation() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-isolation.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    let embed = vec![1.0_f32, 0.0, 0.0];
    backend
        .store_embedding("agent-a", "run-1", "agent-a context", embed.clone(), HashMap::new())
        .await.unwrap();

    // Query from agent-b — should return 0 results
    let results = backend.search_similar("agent-b", &embed, 5).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_sqlite_clear_agent_memory() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-clear.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    let embed = vec![1.0_f32, 0.0, 0.0];
    backend
        .store_embedding("agent-a", "run-1", "some context", embed.clone(), HashMap::new())
        .await.unwrap();

    backend.clear_agent_memory("agent-a").await.unwrap();

    let results = backend.search_similar("agent-a", &embed, 5).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_sqlite_list_entries() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-list.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    let embed = vec![1.0_f32, 0.0, 0.0];
    backend
        .store_embedding("agent-a", "run-1", "first context", embed.clone(), HashMap::new())
        .await.unwrap();
    backend
        .store_embedding("agent-a", "run-2", "second context", embed.clone(), HashMap::new())
        .await.unwrap();

    let entries = backend.list_entries("agent-a").await.unwrap();
    assert_eq!(entries.len(), 2);

    // Entries from agent-b should be empty
    let empty = backend.list_entries("agent-b").await.unwrap();
    assert_eq!(empty.len(), 0);
}

#[tokio::test]
async fn test_sqlite_metadata_roundtrip() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-meta.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    let embed = vec![1.0_f32, 0.0, 0.0];
    let mut meta = HashMap::new();
    meta.insert("trigger_source".to_string(), "cli".to_string());
    meta.insert("task_type".to_string(), "analysis".to_string());

    backend
        .store_embedding("agent-a", "run-1", "text with metadata", embed.clone(), meta)
        .await.unwrap();

    let results = backend.search_similar("agent-a", &embed, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.get("trigger_source").map(|s| s.as_str()), Some("cli"));
    assert_eq!(results[0].metadata.get("task_type").map(|s| s.as_str()), Some("analysis"));
}
