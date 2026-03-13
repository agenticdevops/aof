use agentix_core::vector_memory::{MemoryMatch, VectorEntry, VectorMemoryBackend};
use std::collections::HashMap;

struct MockVectorBackend;

#[async_trait::async_trait]
impl VectorMemoryBackend for MockVectorBackend {
    async fn store_embedding(
        &self,
        _agent_id: &str,
        _run_id: &str,
        _text: &str,
        _embedding: Vec<f32>,
        _metadata: HashMap<String, String>,
    ) -> agentix_core::AgentixResult<String> {
        Ok("mock-id".to_string())
    }

    async fn search_similar(
        &self,
        _agent_id: &str,
        _query_embedding: &[f32],
        _top_k: usize,
    ) -> agentix_core::AgentixResult<Vec<MemoryMatch>> {
        Ok(vec![])
    }

    async fn clear_agent_memory(&self, _agent_id: &str) -> agentix_core::AgentixResult<()> {
        Ok(())
    }

    async fn list_entries(&self, _agent_id: &str) -> agentix_core::AgentixResult<Vec<VectorEntry>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_mock_backend_implements_trait() {
    let backend = MockVectorBackend;
    let id = backend
        .store_embedding("agent-a", "run-1", "test text", vec![0.1, 0.2, 0.3], HashMap::new())
        .await
        .unwrap();
    assert_eq!(id, "mock-id");
}

#[test]
fn test_memory_match_fields() {
    let m = MemoryMatch {
        run_id: "run-1".to_string(),
        text: "test context".to_string(),
        score: 0.95,
        metadata: HashMap::new(),
    };
    assert_eq!(m.run_id, "run-1");
    assert!((m.score - 0.95).abs() < 0.001);
}

#[test]
fn test_cosine_similarity_identical() {
    use agentix_core::vector_memory::cosine_similarity;
    let v = vec![1.0_f32, 0.0, 0.0];
    let sim = cosine_similarity(&v, &v);
    assert!((sim - 1.0).abs() < 0.001);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    use agentix_core::vector_memory::cosine_similarity;
    let a = vec![1.0_f32, 0.0, 0.0];
    let b = vec![0.0_f32, 1.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!(sim.abs() < 0.001);
}

#[test]
fn test_cosine_similarity_zero_vector() {
    use agentix_core::vector_memory::cosine_similarity;
    let a = vec![0.0_f32, 0.0, 0.0];
    let b = vec![1.0_f32, 0.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert_eq!(sim, 0.0);
}
