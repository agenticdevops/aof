//! SQLite-backed audit trail store for OpenAgentiX.
//!
//! Logs every tool call, LLM call, agent lifecycle event, and security violation
//! with actor, timestamp, outcome, and structured details.
//!
//! Uses `rusqlite` with `parking_lot::Mutex` for thread-safe access, following
//! the same pattern as `CostStore` and `TraceStore`.

use std::collections::HashMap;
use std::sync::Arc;

use agentix_core::AgentixError;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Audit types
// ---------------------------------------------------------------------------

/// A single audit trail entry recording an agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Auto-generated ID (populated after insertion).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Category of event.
    pub event_type: AuditEventType,
    /// Which agent this event belongs to.
    pub agent_name: String,
    /// Run ID (if this event occurred during a run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Who/what initiated the action ("system", "agent:<name>", "user:<id>").
    pub actor: String,
    /// Human-readable description of what happened.
    pub action: String,
    /// Result of the action.
    pub outcome: AuditOutcome,
    /// Structured key-value metadata.
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
    /// Link to distributed trace (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Category of audit event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    ToolCall,
    LlmCall,
    ApprovalDecision,
    AgentStart,
    AgentComplete,
    AgentError,
    SecurityViolation,
    SecretAccess,
}

impl AuditEventType {
    fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::ToolCall => "tool_call",
            AuditEventType::LlmCall => "llm_call",
            AuditEventType::ApprovalDecision => "approval_decision",
            AuditEventType::AgentStart => "agent_start",
            AuditEventType::AgentComplete => "agent_complete",
            AuditEventType::AgentError => "agent_error",
            AuditEventType::SecurityViolation => "security_violation",
            AuditEventType::SecretAccess => "secret_access",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "tool_call" => AuditEventType::ToolCall,
            "llm_call" => AuditEventType::LlmCall,
            "approval_decision" => AuditEventType::ApprovalDecision,
            "agent_start" => AuditEventType::AgentStart,
            "agent_complete" => AuditEventType::AgentComplete,
            "agent_error" => AuditEventType::AgentError,
            "security_violation" => AuditEventType::SecurityViolation,
            "secret_access" => AuditEventType::SecretAccess,
            _ => AuditEventType::ToolCall, // fallback
        }
    }
}

/// Result of an audited action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure(String),
    Denied(String),
    Blocked(String),
}

impl AuditOutcome {
    fn kind(&self) -> &'static str {
        match self {
            AuditOutcome::Success => "success",
            AuditOutcome::Failure(_) => "failure",
            AuditOutcome::Denied(_) => "denied",
            AuditOutcome::Blocked(_) => "blocked",
        }
    }

    fn message(&self) -> Option<&str> {
        match self {
            AuditOutcome::Success => None,
            AuditOutcome::Failure(m) => Some(m),
            AuditOutcome::Denied(m) => Some(m),
            AuditOutcome::Blocked(m) => Some(m),
        }
    }

    fn from_parts(kind: &str, msg: Option<String>) -> Self {
        match kind {
            "failure" => AuditOutcome::Failure(msg.unwrap_or_default()),
            "denied" => AuditOutcome::Denied(msg.unwrap_or_default()),
            "blocked" => AuditOutcome::Blocked(msg.unwrap_or_default()),
            _ => AuditOutcome::Success,
        }
    }
}

// ---------------------------------------------------------------------------
// AuditStore
// ---------------------------------------------------------------------------

/// SQLite-backed audit trail store.
///
/// Opens/creates `audit.db` in the same data directory as `runs.db`, `cost.db`,
/// and `trace.db`. Uses WAL mode for concurrent reads.
pub struct AuditStore {
    conn: Arc<Mutex<Connection>>,
}

impl AuditStore {
    /// Open or create the audit database at the given path.
    ///
    /// Use `":memory:"` for tests.
    pub fn open(path: &str) -> Result<Self, AgentixError> {
        let conn = Connection::open(path).map_err(|e| {
            AgentixError::runtime(format!("Failed to open audit DB at '{path}': {e}"))
        })?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| AgentixError::runtime(format!("Failed to set WAL mode: {e}")))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS audit_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   TEXT NOT NULL,
                event_type  TEXT NOT NULL,
                agent_name  TEXT NOT NULL,
                run_id      TEXT,
                actor       TEXT NOT NULL,
                action      TEXT NOT NULL,
                outcome     TEXT NOT NULL,
                outcome_msg TEXT,
                details     TEXT NOT NULL DEFAULT '{}',
                trace_id    TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_audit_agent ON audit_log(agent_name);
            CREATE INDEX IF NOT EXISTS idx_audit_run   ON audit_log(run_id);
            CREATE INDEX IF NOT EXISTS idx_audit_type  ON audit_log(event_type);
            CREATE INDEX IF NOT EXISTS idx_audit_time  ON audit_log(timestamp);
            ",
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to create audit_log table: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Persist an audit entry to the database.
    pub fn log_event(&self, entry: AuditEntry) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        let details_json = serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO audit_log
             (timestamp, event_type, agent_name, run_id, actor, action, outcome, outcome_msg, details, trace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.event_type.as_str(),
                entry.agent_name,
                entry.run_id,
                entry.actor,
                entry.action,
                entry.outcome.kind(),
                entry.outcome.message(),
                details_json,
                entry.trace_id,
            ],
        )
        .map_err(|e| {
            AgentixError::runtime(format!("Failed to insert audit entry: {e}"))
        })?;
        Ok(())
    }

    /// Retrieve audit entries for a specific agent, most recent first.
    pub fn get_agent_audit(
        &self,
        agent_name: &str,
        limit: u32,
    ) -> Result<Vec<AuditEntry>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, event_type, agent_name, run_id, actor, action,
                        outcome, outcome_msg, details, trace_id
                 FROM audit_log
                 WHERE agent_name = ?1
                 ORDER BY timestamp DESC
                 LIMIT ?2",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare agent audit query: {e}")))?;

        let rows = stmt
            .query_map(params![agent_name, limit], |row| {
                Ok(Self::row_to_entry(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query agent audit: {e}")))?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = row.map_err(|e| AgentixError::runtime(format!("Failed to read audit row: {e}")))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Retrieve audit entries for a specific run, in chronological order.
    pub fn get_run_audit(
        &self,
        run_id: &str,
    ) -> Result<Vec<AuditEntry>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, event_type, agent_name, run_id, actor, action,
                        outcome, outcome_msg, details, trace_id
                 FROM audit_log
                 WHERE run_id = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare run audit query: {e}")))?;

        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(Self::row_to_entry(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query run audit: {e}")))?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = row.map_err(|e| AgentixError::runtime(format!("Failed to read audit row: {e}")))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Retrieve security-relevant events (violations, denials, blocks).
    pub fn get_security_events(
        &self,
        limit: u32,
    ) -> Result<Vec<AuditEntry>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, event_type, agent_name, run_id, actor, action,
                        outcome, outcome_msg, details, trace_id
                 FROM audit_log
                 WHERE event_type = 'security_violation'
                    OR outcome IN ('denied', 'blocked')
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare security audit query: {e}")))?;

        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(Self::row_to_entry(row))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to query security events: {e}")))?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = row.map_err(|e| AgentixError::runtime(format!("Failed to read audit row: {e}")))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Convert a SQLite row to an AuditEntry.
    fn row_to_entry(row: &rusqlite::Row) -> AuditEntry {
        let id: i64 = row.get(0).unwrap_or(0);
        let timestamp_str: String = row.get(1).unwrap_or_default();
        let event_type_str: String = row.get(2).unwrap_or_default();
        let agent_name: String = row.get(3).unwrap_or_default();
        let run_id: Option<String> = row.get(4).unwrap_or(None);
        let actor: String = row.get(5).unwrap_or_default();
        let action: String = row.get(6).unwrap_or_default();
        let outcome_str: String = row.get(7).unwrap_or_default();
        let outcome_msg: Option<String> = row.get(8).unwrap_or(None);
        let details_str: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
        let trace_id: Option<String> = row.get(10).unwrap_or(None);

        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let details: HashMap<String, serde_json::Value> =
            serde_json::from_str(&details_str).unwrap_or_default();

        AuditEntry {
            id: Some(id),
            timestamp,
            event_type: AuditEventType::from_str(&event_type_str),
            agent_name,
            run_id,
            actor,
            action,
            outcome: AuditOutcome::from_parts(&outcome_str, outcome_msg),
            details,
            trace_id,
        }
    }
}
