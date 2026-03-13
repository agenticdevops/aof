// Integration tests for memory-aware ReAct loop.
// These tests use the real SqliteVectorBackend to avoid mock complexity.

use agentix_core::vector_memory::VectorMemoryBackend;
use agentix_memory::SqliteVectorBackend;
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::test]
async fn test_memory_store_after_run() {
    // After a run completes with memory.enabled=true, the final answer
    // should be stored in the vector backend.
    //
    // This test validates the store_embedding round-trip with SqliteVectorBackend.
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("run-test.db");
    let backend = SqliteVectorBackend::new(&db_path).await.unwrap();

    // Simulate storing a run result
    let embedding = vec![0.5_f32; 256]; // 256-dim mock embedding
    let id = backend
        .store_embedding(
            "test-agent",
            "run-001",
            "pg replication lag was 2ms",
            embedding.clone(),
            {
                let mut m = HashMap::new();
                m.insert("trigger_source".to_string(), "cli".to_string());
                m
            },
        )
        .await
        .unwrap();
    assert!(!id.is_empty());

    // Retrieve it — identical embedding should give score ~1.0
    let results = backend
        .search_similar("test-agent", &embedding, 1)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].score > 0.99);
    assert_eq!(
        results[0].metadata.get("trigger_source").map(|s| s.as_str()),
        Some("cli")
    );
}

#[tokio::test]
async fn test_memory_context_injection_format() {
    // Validate that the memory context block has the expected format.
    // This tests the helper function that formats MemoryMatch items into a prompt block.
    use agentix_core::vector_memory::MemoryMatch;
    use agentix_runtime::memory::format_memory_context;

    let matches = vec![
        MemoryMatch {
            run_id: "run-001".to_string(),
            text: "pg replication lag was 2ms".to_string(),
            score: 0.97,
            metadata: HashMap::new(),
        },
        MemoryMatch {
            run_id: "run-002".to_string(),
            text: "pg replication lag was 5ms — check network".to_string(),
            score: 0.82,
            metadata: HashMap::new(),
        },
    ];

    let context = format_memory_context(&matches);
    assert!(context.contains("## Relevant context from past runs"));
    assert!(context.contains("run-001"));
    assert!(context.contains("pg replication lag was 2ms"));
}
