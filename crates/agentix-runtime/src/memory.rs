//! Memory integration helpers for the ReAct loop.
//!
//! This module bridges the vector memory backend (agentix-memory) with the
//! ReAct execution engine. It provides:
//! - `format_memory_context` — formats retrieved past-run matches into a system prompt block
//! - `open_agent_memory` — opens or creates a `SqliteVectorBackend` for a named agent
//! - `hash_embedding` — deterministic 256-dim unit vector from text (fallback when no embed API)

use agentix_core::vector_memory::MemoryMatch;
use agentix_core::AgentixResult;
use agentix_memory::SqliteVectorBackend;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

/// Format retrieved memory matches into a system prompt block.
///
/// Returns an empty string when `matches` is empty so the caller can safely
/// prepend it without injecting spurious whitespace.
pub fn format_memory_context(matches: &[MemoryMatch]) -> String {
    if matches.is_empty() {
        return String::new();
    }

    let mut block = String::from("## Relevant context from past runs\n\n");
    for m in matches {
        block.push_str(&format!(
            "**Run {}** (similarity: {:.2}):\n{}\n\n",
            m.run_id, m.score, m.text
        ));
    }
    block
}

// ---------------------------------------------------------------------------
// Backend lifecycle
// ---------------------------------------------------------------------------

/// Open or create a `SqliteVectorBackend` for a named agent.
///
/// If `db_path` is `None`, defaults to `./memory/<agent_name>.db`.
/// Creates the parent directory if it does not exist.
pub async fn open_agent_memory(
    agent_name: &str,
    db_path: Option<&str>,
) -> AgentixResult<Arc<SqliteVectorBackend>> {
    let path = match db_path {
        Some(p) => p.to_string(),
        None => format!("./memory/{}.db", agent_name),
    };

    // Ensure the parent directory exists.
    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let backend = SqliteVectorBackend::new(&path).await?;
    Ok(Arc::new(backend))
}

// ---------------------------------------------------------------------------
// Deterministic embedding (fallback)
// ---------------------------------------------------------------------------

/// Generate a deterministic 256-dim unit vector from arbitrary text.
///
/// This is a lightweight fallback for providers that do not have a dedicated
/// embedding endpoint. The vector is derived from a FNV-1a hash spread over
/// 256 dimensions, then L2-normalised.
///
/// Note: semantic similarity is NOT preserved by this method. Use a real
/// embedding model for production recall quality.
pub fn hash_embedding(text: &str) -> Vec<f32> {
    const DIMS: usize = 256;
    let mut raw = vec![0.0_f32; DIMS];

    // FNV-1a hash seeded per dimension to produce distinct components.
    for (i, byte) in text.bytes().enumerate() {
        let dim = i % DIMS;
        // Mix byte into the dimension bucket using FNV prime.
        let prev = raw[dim].to_bits();
        raw[dim] = f32::from_bits(prev ^ ((byte as u32).wrapping_mul(16_777_619)));
    }

    // L2-normalise so cosine similarity is well-defined.
    let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        raw.iter_mut().for_each(|x| *x /= norm);
    } else {
        // All-zero input: emit a unit vector in the first dimension.
        raw[0] = 1.0;
    }

    raw
}
