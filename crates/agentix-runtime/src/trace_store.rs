//! SQLite-backed trace storage for spans and structured log entries.
//!
//! Persists `SpanRecord`s and `StructuredLogEntry`s for completed agent runs.
//! Uses the same `data/` directory pattern as `CostStore` and `RunStore`.

use std::collections::HashMap;
use std::sync::Arc;

use agentix_core::telemetry::{
    LogLevel, SpanKind, SpanRecord, SpanStatus, StructuredLogEntry,
};
use agentix_core::AgentixError;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{Connection, params};

/// SQLite-backed store for trace spans and structured log entries.
///
/// One database per workspace, storing all agent runs.
/// Opened at `data/trace.db` alongside `runs.db` and `cost.db`.
pub struct TraceStore {
    conn: Arc<Mutex<Connection>>,
}

impl TraceStore {
    /// Open or create the SQLite database at the given path.
    ///
    /// Use `":memory:"` for tests (no file on disk).
    pub fn open(path: &str) -> Result<Self, AgentixError> {
        let conn = Connection::open(path).map_err(|e| {
            AgentixError::runtime(format!("Failed to open trace DB at '{path}': {e}"))
        })?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| AgentixError::runtime(format!("Failed to set WAL mode: {e}")))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS spans (
                span_id         TEXT PRIMARY KEY,
                parent_span_id  TEXT,
                trace_id        TEXT NOT NULL,
                run_id          TEXT NOT NULL,
                agent_name      TEXT NOT NULL,
                name            TEXT NOT NULL,
                kind            TEXT NOT NULL,
                start_time      TEXT NOT NULL,
                end_time        TEXT,
                duration_ms     INTEGER,
                status          TEXT NOT NULL DEFAULT 'ok',
                status_message  TEXT,
                attributes      TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_spans_trace ON spans(trace_id);
            CREATE INDEX IF NOT EXISTS idx_spans_run   ON spans(run_id);

            CREATE TABLE IF NOT EXISTS structured_logs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   TEXT NOT NULL,
                level       TEXT NOT NULL,
                message     TEXT NOT NULL,
                trace_id    TEXT,
                span_id     TEXT,
                agent       TEXT,
                run_id      TEXT,
                fields      TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_logs_run    ON structured_logs(run_id);
            CREATE INDEX IF NOT EXISTS idx_logs_trace  ON structured_logs(trace_id);
            ",
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to create trace tables: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert spans for a completed agent run.
    pub fn insert_spans(
        &self,
        spans: &[SpanRecord],
        agent_name: &str,
        run_id: &str,
    ) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        for span in spans {
            let (status_str, status_message) = match &span.status {
                SpanStatus::Ok => ("ok".to_string(), None),
                SpanStatus::Error(msg) => ("error".to_string(), Some(msg.clone())),
            };
            let kind_str =
                serde_json::to_string(&span.kind).unwrap_or_else(|_| "\"run\"".to_string());
            // serde serializes as quoted string, strip quotes
            let kind_str = kind_str.trim_matches('"');
            let attrs_json = serde_json::to_string(&span.attributes).unwrap_or_default();

            conn.execute(
                "INSERT OR REPLACE INTO spans
                 (span_id, parent_span_id, trace_id, run_id, agent_name, name, kind,
                  start_time, end_time, duration_ms, status, status_message, attributes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    span.span_id,
                    span.parent_span_id,
                    span.trace_id,
                    run_id,
                    agent_name,
                    span.name,
                    kind_str,
                    span.start_time.to_rfc3339(),
                    span.end_time.map(|t| t.to_rfc3339()),
                    span.duration_ms.map(|d| d as i64),
                    status_str,
                    status_message,
                    attrs_json,
                ],
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to insert span '{}': {e}", span.span_id))
            })?;
        }
        Ok(())
    }

    /// Insert structured log entries for a completed agent run.
    pub fn insert_logs(&self, logs: &[StructuredLogEntry]) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        for log in logs {
            let level_str =
                serde_json::to_string(&log.level).unwrap_or_else(|_| "\"info\"".to_string());
            let level_str = level_str.trim_matches('"');
            let fields_json = serde_json::to_string(&log.fields).unwrap_or_default();

            conn.execute(
                "INSERT INTO structured_logs
                 (timestamp, level, message, trace_id, span_id, agent, run_id, fields)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    log.timestamp.to_rfc3339(),
                    level_str,
                    log.message,
                    log.trace_id,
                    log.span_id,
                    log.agent,
                    log.run_id,
                    fields_json,
                ],
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to insert structured log: {e}"))
            })?;
        }
        Ok(())
    }

    /// Retrieve all spans for a given trace ID.
    pub fn get_trace(&self, trace_id: &str) -> Result<Vec<SpanRecord>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT span_id, parent_span_id, trace_id, name, kind,
                        start_time, end_time, duration_ms, status, status_message, attributes
                 FROM spans WHERE trace_id = ?1 ORDER BY start_time ASC",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare trace query: {e}")))?;

        let spans = stmt
            .query_map(params![trace_id], |row| {
                Ok(row_to_span_record(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query traces: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(spans)
    }

    /// Retrieve all spans for a given run ID, ordered by start_time.
    pub fn get_run_trace(&self, run_id: &str) -> Result<Vec<SpanRecord>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT span_id, parent_span_id, trace_id, name, kind,
                        start_time, end_time, duration_ms, status, status_message, attributes
                 FROM spans WHERE run_id = ?1 ORDER BY start_time ASC",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare run trace query: {e}")))?;

        let spans = stmt
            .query_map(params![run_id], |row| {
                Ok(row_to_span_record(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query run traces: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(spans)
    }

    /// Retrieve structured log entries for a given run ID, ordered by timestamp.
    pub fn get_run_logs(&self, run_id: &str) -> Result<Vec<StructuredLogEntry>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT timestamp, level, message, trace_id, span_id, agent, run_id, fields
                 FROM structured_logs WHERE run_id = ?1 ORDER BY timestamp ASC",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare log query: {e}")))?;

        let logs = stmt
            .query_map(params![run_id], |row| {
                Ok(row_to_log_entry(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query logs: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(logs)
    }
}

/// Convert a database row to a SpanRecord.
fn row_to_span_record(row: &rusqlite::Row) -> SpanRecord {
    let span_id: String = row.get(0).unwrap_or_default();
    let parent_span_id: Option<String> = row.get(1).unwrap_or(None);
    let trace_id: String = row.get(2).unwrap_or_default();
    let name: String = row.get(3).unwrap_or_default();
    let kind_str: String = row.get(4).unwrap_or_else(|_| "run".to_string());
    let start_time_str: String = row.get(5).unwrap_or_default();
    let end_time_str: Option<String> = row.get(6).unwrap_or(None);
    let duration_ms: Option<i64> = row.get(7).unwrap_or(None);
    let status_str: String = row.get(8).unwrap_or_else(|_| "ok".to_string());
    let status_message: Option<String> = row.get(9).unwrap_or(None);
    let attrs_json: String = row.get(10).unwrap_or_else(|_| "{}".to_string());

    let kind: SpanKind = serde_json::from_str(&format!("\"{}\"", kind_str))
        .unwrap_or(SpanKind::Run);

    let start_time: DateTime<Utc> = DateTime::parse_from_rfc3339(&start_time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let end_time = end_time_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    let status = if status_str == "error" {
        SpanStatus::Error(status_message.unwrap_or_default())
    } else {
        SpanStatus::Ok
    };

    let attributes: HashMap<String, String> =
        serde_json::from_str(&attrs_json).unwrap_or_default();

    SpanRecord {
        span_id,
        parent_span_id,
        trace_id,
        name,
        kind,
        start_time,
        end_time,
        duration_ms: duration_ms.map(|d| d as u64),
        status,
        attributes,
    }
}

/// Convert a database row to a StructuredLogEntry.
fn row_to_log_entry(row: &rusqlite::Row) -> StructuredLogEntry {
    let timestamp_str: String = row.get(0).unwrap_or_default();
    let level_str: String = row.get(1).unwrap_or_else(|_| "info".to_string());
    let message: String = row.get(2).unwrap_or_default();
    let trace_id: Option<String> = row.get(3).unwrap_or(None);
    let span_id: Option<String> = row.get(4).unwrap_or(None);
    let agent: Option<String> = row.get(5).unwrap_or(None);
    let run_id: Option<String> = row.get(6).unwrap_or(None);
    let fields_json: String = row.get(7).unwrap_or_else(|_| "{}".to_string());

    let timestamp: DateTime<Utc> = DateTime::parse_from_rfc3339(&timestamp_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let level: LogLevel = serde_json::from_str(&format!("\"{}\"", level_str))
        .unwrap_or(LogLevel::Info);

    let fields: HashMap<String, serde_json::Value> =
        serde_json::from_str(&fields_json).unwrap_or_default();

    StructuredLogEntry {
        timestamp,
        level,
        message,
        trace_id,
        span_id,
        agent,
        run_id,
        fields,
    }
}
