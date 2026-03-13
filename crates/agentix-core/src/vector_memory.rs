//! Vector memory abstraction for semantic recall of past agent run contexts.
//!
//! Agents store text + embedding pairs after each run. Before generating a response,
//! agents can query for semantically similar past contexts to reduce hallucinations
//! and improve consistency.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::AgentixResult;

/// A stored vector memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique ID for this entry.
    pub id: String,
    /// Agent that owns this memory (per-agent isolation via this field).
    pub agent_id: String,
    /// Run that produced this context.
    pub run_id: String,
    /// The original text that was embedded.
    pub text: String,
    /// The embedding vector (model-dimension floats).
    pub embedding: Vec<f32>,
    /// Optional metadata (e.g., trigger_source, task_type).
    pub metadata: HashMap<String, String>,
    /// When this entry was stored.
    pub stored_at: DateTime<Utc>,
}

/// A result from a similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMatch {
    /// The run that produced this context.
    pub run_id: String,
    /// The original text.
    pub text: String,
    /// Cosine similarity score (0.0–1.0, higher = more similar).
    pub score: f32,
    /// Metadata from the stored entry.
    pub metadata: HashMap<String, String>,
}

/// Pluggable vector memory backend.
///
/// Implementations must scope all operations to `agent_id` — an agent MUST NOT
/// be able to retrieve memories from another agent unless explicitly configured.
#[async_trait]
pub trait VectorMemoryBackend: Send + Sync {
    /// Store a text + embedding pair scoped to an agent.
    ///
    /// Returns the ID assigned to the stored entry.
    async fn store_embedding(
        &self,
        agent_id: &str,
        run_id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> AgentixResult<String>;

    /// Search for the top-K most similar memories for an agent.
    ///
    /// Results are ordered by cosine similarity (highest first).
    /// Only returns entries belonging to `agent_id`.
    async fn search_similar(
        &self,
        agent_id: &str,
        query_embedding: &[f32],
        top_k: usize,
    ) -> AgentixResult<Vec<MemoryMatch>>;

    /// Delete all stored memories for an agent.
    async fn clear_agent_memory(&self, agent_id: &str) -> AgentixResult<()>;

    /// List all stored entries for an agent (for CLI/REST display).
    /// Results are ordered by stored_at descending.
    async fn list_entries(&self, agent_id: &str) -> AgentixResult<Vec<VectorEntry>>;
}

/// Compute cosine similarity between two equal-length vectors.
/// Returns a value in [-1.0, 1.0]; higher = more similar.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "embedding dimension mismatch");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
