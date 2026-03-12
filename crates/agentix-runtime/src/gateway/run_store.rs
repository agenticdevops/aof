//! Persistent run history backed by SQLite.
//!
//! Every triggered agent run creates a `RunRecord` that is persisted to a
//! SQLite database. This enables `agentix runs` to show full history including
//! trigger source, duration, and output summaries across server restarts.

use std::sync::Arc;

use agentix_core::AgentixError;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// RunRecord
// ---------------------------------------------------------------------------

/// A persisted record of one agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub agent_name: String,
    /// Trigger source string: "cron", "webhook", "github", "jira", "slack",
    /// "discord", "telegram", "agent", or "cli".
    pub trigger_source: String,
    pub trigger_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    /// "running", "completed", "failed", or "stopped"
    pub status: String,
    pub iterations: Option<i32>,
    /// First 200 characters of the input
    pub input_summary: Option<String>,
    /// First 500 characters of the output
    pub output_summary: Option<String>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// RunStore
// ---------------------------------------------------------------------------

/// SQLite-backed run history store.
pub struct RunStore {
    conn: Arc<Mutex<Connection>>,
}

impl RunStore {
    /// Open or create the SQLite database at the given path.
    ///
    /// Use `":memory:"` for tests (no file on disk).
    pub fn open(path: &str) -> Result<Self, AgentixError> {
        let conn = Connection::open(path)
            .map_err(|e| AgentixError::runtime(format!("Failed to open SQLite at '{path}': {e}")))?;

        // Enable WAL mode for better concurrent write performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| AgentixError::runtime(format!("Failed to set WAL mode: {e}")))?;

        // Create schema
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS runs (
                id              TEXT PRIMARY KEY,
                agent_name      TEXT NOT NULL,
                trigger_source  TEXT NOT NULL DEFAULT 'cli',
                trigger_id      TEXT,
                started_at      TEXT NOT NULL,
                ended_at        TEXT,
                duration_ms     INTEGER,
                status          TEXT NOT NULL DEFAULT 'running',
                iterations      INTEGER,
                input_summary   TEXT,
                output_summary  TEXT,
                error           TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_runs_agent   ON runs(agent_name);
            CREATE INDEX IF NOT EXISTS idx_runs_started ON runs(started_at DESC);
            ",
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to create runs table: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a new RunRecord. Does nothing if a record with this id already exists.
    pub fn insert(&self, record: &RunRecord) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO runs
             (id, agent_name, trigger_source, trigger_id, started_at, ended_at,
              duration_ms, status, iterations, input_summary, output_summary, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id,
                record.agent_name,
                record.trigger_source,
                record.trigger_id,
                record.started_at.to_rfc3339(),
                record.ended_at.map(|d| d.to_rfc3339()),
                record.duration_ms,
                record.status,
                record.iterations,
                record.input_summary,
                record.output_summary,
                record.error,
            ],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to insert run '{}': {e}", record.id)))?;
        Ok(())
    }

    /// Mark a run as completed.
    pub fn update_completed(
        &self,
        id: &str,
        ended_at: DateTime<Utc>,
        output: Option<&str>,
        iterations: Option<i32>,
    ) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        let started_at: Option<String> = conn
            .query_row(
                "SELECT started_at FROM runs WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok();

        let duration_ms = started_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|started| (ended_at - started.with_timezone(&Utc)).num_milliseconds());

        let truncated_output = output.map(|s| {
            if s.len() > 500 { s[..500].to_string() } else { s.to_string() }
        });

        conn.execute(
            "UPDATE runs SET status='completed', ended_at=?1, duration_ms=?2,
             output_summary=?3, iterations=?4 WHERE id=?5",
            params![
                ended_at.to_rfc3339(),
                duration_ms,
                truncated_output,
                iterations,
                id,
            ],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to update run '{id}': {e}")))?;
        Ok(())
    }

    /// Mark a run as failed.
    pub fn update_failed(
        &self,
        id: &str,
        ended_at: DateTime<Utc>,
        error: &str,
    ) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE runs SET status='failed', ended_at=?1, error=?2 WHERE id=?3",
            params![ended_at.to_rfc3339(), error, id],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to update failed run '{id}': {e}")))?;
        Ok(())
    }

    /// List runs, optionally filtered by agent name.
    ///
    /// Returns the most recent `limit` runs (sorted by started_at DESC).
    pub fn list(
        &self,
        agent_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RunRecord>, AgentixError> {
        let conn = self.conn.lock();
        let limit_i64 = limit as i64;

        let records = if let Some(agent) = agent_filter {
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_name, trigger_source, trigger_id, started_at, ended_at,
                      duration_ms, status, iterations, input_summary, output_summary, error
                     FROM runs WHERE agent_name=?1 ORDER BY started_at DESC LIMIT ?2",
                )
                .map_err(|e| AgentixError::runtime(e.to_string()))?;
            collect_rows(&mut stmt, params![agent, limit_i64])?
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_name, trigger_source, trigger_id, started_at, ended_at,
                      duration_ms, status, iterations, input_summary, output_summary, error
                     FROM runs ORDER BY started_at DESC LIMIT ?1",
                )
                .map_err(|e| AgentixError::runtime(e.to_string()))?;
            collect_rows(&mut stmt, params![limit_i64])?
        };
        Ok(records)
    }

    /// Get a single run by its id.
    pub fn get(&self, run_id: &str) -> Result<Option<RunRecord>, AgentixError> {
        let conn = self.conn.lock();
        let result = conn
            .query_row(
                "SELECT id, agent_name, trigger_source, trigger_id, started_at, ended_at,
                  duration_ms, status, iterations, input_summary, output_summary, error
                 FROM runs WHERE id=?1",
                params![run_id],
                row_to_record,
            )
            .optional()
            .map_err(|e| AgentixError::runtime(format!("Failed to get run '{run_id}': {e}")))?;
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunRecord> {
    let started_str: String = row.get(4)?;
    let ended_str: Option<String> = row.get(5)?;

    let started_at = DateTime::parse_from_rfc3339(&started_str)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let ended_at = ended_str
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc));

    Ok(RunRecord {
        id: row.get(0)?,
        agent_name: row.get(1)?,
        trigger_source: row.get(2)?,
        trigger_id: row.get(3)?,
        started_at,
        ended_at,
        duration_ms: row.get(6)?,
        status: row.get(7)?,
        iterations: row.get(8)?,
        input_summary: row.get(9)?,
        output_summary: row.get(10)?,
        error: row.get(11)?,
    })
}

fn collect_rows(
    stmt: &mut rusqlite::Statement<'_>,
    params: impl rusqlite::Params,
) -> Result<Vec<RunRecord>, AgentixError> {
    let rows = stmt
        .query_map(params, row_to_record)
        .map_err(|e| AgentixError::runtime(e.to_string()))?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| AgentixError::runtime(e.to_string()))?);
    }
    Ok(records)
}
