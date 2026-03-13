//! SQLite-backed vector memory store.
//!
//! Embeddings are serialized to BLOB (little-endian f32 arrays).
//! Cosine similarity search is performed in Rust after loading candidate rows.

use agentix_core::{
    vector_memory::{cosine_similarity, MemoryMatch, VectorEntry, VectorMemoryBackend},
    AgentixError, AgentixResult,
};
use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// SQLite-backed vector memory. Thread-safe via Mutex<Connection>.
pub struct SqliteVectorBackend {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteVectorBackend {
    /// Open (or create) a SQLite database at the given path.
    pub async fn new(path: impl AsRef<Path>) -> AgentixResult<Self> {
        let conn = Connection::open(path.as_ref())
            .map_err(|e| AgentixError::runtime(format!("sqlite open failed: {e}")))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS vector_memory (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                stored_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_agent_id ON vector_memory(agent_id);",
        )
        .map_err(|e| AgentixError::runtime(format!("sqlite schema init failed: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn encode_embedding(embedding: &[f32]) -> Vec<u8> {
        embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    fn decode_embedding(blob: &[u8]) -> Vec<f32> {
        blob.chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect()
    }
}

#[async_trait]
impl VectorMemoryBackend for SqliteVectorBackend {
    async fn store_embedding(
        &self,
        agent_id: &str,
        run_id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> AgentixResult<String> {
        let id = Uuid::new_v4().to_string();
        let blob = Self::encode_embedding(&embedding);
        let meta_json = serde_json::to_string(&metadata)
            .map_err(|e| AgentixError::runtime(format!("metadata serialize: {e}")))?;
        let stored_at = Utc::now().to_rfc3339();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO vector_memory (id, agent_id, run_id, text, embedding, metadata, stored_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, agent_id, run_id, text, blob, meta_json, stored_at],
        )
        .map_err(|e| AgentixError::runtime(format!("sqlite insert: {e}")))?;

        Ok(id)
    }

    async fn search_similar(
        &self,
        agent_id: &str,
        query_embedding: &[f32],
        top_k: usize,
    ) -> AgentixResult<Vec<MemoryMatch>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT run_id, text, embedding, metadata FROM vector_memory WHERE agent_id = ?1",
            )
            .map_err(|e| AgentixError::runtime(format!("sqlite prepare: {e}")))?;

        let rows: Vec<(String, String, Vec<u8>, String)> = stmt
            .query_map(params![agent_id], |row| {
                let run_id: String = row.get(0)?;
                let text: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                let meta_json: String = row.get(3)?;
                Ok((run_id, text, blob, meta_json))
            })
            .map_err(|e| AgentixError::runtime(format!("sqlite query: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        let mut scored: Vec<MemoryMatch> = rows
            .into_iter()
            .map(|(run_id, text, blob, meta_json)| {
                let embedding = Self::decode_embedding(&blob);
                let score = cosine_similarity(query_embedding, &embedding);
                let metadata: HashMap<String, String> =
                    serde_json::from_str(&meta_json).unwrap_or_default();
                MemoryMatch { run_id, text, score, metadata }
            })
            .collect();

        // Sort descending by score
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored)
    }

    async fn clear_agent_memory(&self, agent_id: &str) -> AgentixResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM vector_memory WHERE agent_id = ?1",
            params![agent_id],
        )
        .map_err(|e| AgentixError::runtime(format!("sqlite delete: {e}")))?;
        Ok(())
    }

    async fn list_entries(&self, agent_id: &str) -> AgentixResult<Vec<VectorEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_id, run_id, text, embedding, metadata, stored_at
                 FROM vector_memory WHERE agent_id = ?1 ORDER BY stored_at DESC",
            )
            .map_err(|e| AgentixError::runtime(format!("sqlite prepare list: {e}")))?;

        let entries: Vec<VectorEntry> = stmt
            .query_map(params![agent_id], |row| {
                let id: String = row.get(0)?;
                let a_id: String = row.get(1)?;
                let run_id: String = row.get(2)?;
                let text: String = row.get(3)?;
                let blob: Vec<u8> = row.get(4)?;
                let meta_json: String = row.get(5)?;
                let stored_at_str: String = row.get(6)?;
                Ok((id, a_id, run_id, text, blob, meta_json, stored_at_str))
            })
            .map_err(|e| AgentixError::runtime(format!("sqlite query list: {e}")))?
            .filter_map(|r| r.ok())
            .map(|(id, agent_id, run_id, text, blob, meta_json, stored_at_str)| {
                let embedding = Self::decode_embedding(&blob);
                let metadata: HashMap<String, String> =
                    serde_json::from_str(&meta_json).unwrap_or_default();
                let stored_at = chrono::DateTime::parse_from_rfc3339(&stored_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                VectorEntry { id, agent_id, run_id, text, embedding, metadata, stored_at }
            })
            .collect();

        Ok(entries)
    }
}
