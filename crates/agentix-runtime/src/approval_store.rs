//! SQLite-backed approval request store for OpenAgentiX.
//!
//! Persists approval requests so that paused agent runs can survive gateway
//! restarts. Uses `rusqlite` with `parking_lot::Mutex` for thread-safe access,
//! following the same pattern as `AuditStore`, `CostStore`, and `TraceStore`.

use std::sync::Arc;

use agentix_core::{AgentixError, ApprovalRequest, ApprovalStatus};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};

// ---------------------------------------------------------------------------
// ApprovalStore
// ---------------------------------------------------------------------------

/// SQLite-backed approval request store.
///
/// Opens/creates `approval.db` in the same data directory as other stores.
/// Uses WAL mode for concurrent reads.
pub struct ApprovalStore {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for ApprovalStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalStore").finish()
    }
}

impl ApprovalStore {
    /// Open or create the approval database at the given path.
    ///
    /// Use `":memory:"` for tests.
    pub fn open(path: &str) -> Result<Self, AgentixError> {
        let conn = Connection::open(path).map_err(|e| {
            AgentixError::runtime(format!("Failed to open approval DB at '{path}': {e}"))
        })?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| AgentixError::runtime(format!("Failed to set WAL mode: {e}")))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS approval_requests (
                id                  TEXT PRIMARY KEY,
                run_id              TEXT NOT NULL,
                agent_name          TEXT NOT NULL,
                action_description  TEXT NOT NULL,
                tool_name           TEXT,
                tool_input          TEXT,
                requested_at        TEXT NOT NULL,
                timeout_secs        INTEGER NOT NULL DEFAULT 300,
                status              TEXT NOT NULL DEFAULT 'pending',
                approver            TEXT,
                reason              TEXT,
                decided_at          TEXT,
                expired_at          TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_approval_agent  ON approval_requests(agent_name);
            CREATE INDEX IF NOT EXISTS idx_approval_run    ON approval_requests(run_id);
            CREATE INDEX IF NOT EXISTS idx_approval_status ON approval_requests(status);
            ",
        )
        .map_err(|e| {
            AgentixError::runtime(format!("Failed to create approval_requests table: {e}"))
        })?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a new approval request into the store.
    pub fn create_request(&self, request: &ApprovalRequest) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        let tool_input_json = request
            .tool_input
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()));

        conn.execute(
            "INSERT INTO approval_requests
             (id, run_id, agent_name, action_description, tool_name, tool_input, requested_at, timeout_secs, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                request.id,
                request.run_id,
                request.agent_name,
                request.action_description,
                request.tool_name,
                tool_input_json,
                request.requested_at.to_rfc3339(),
                request.timeout_secs,
                "pending",
            ],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to insert approval request: {e}")))?;
        Ok(())
    }

    /// Retrieve a single approval request by ID.
    pub fn get_request(&self, id: &str) -> Result<Option<ApprovalRequest>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, run_id, agent_name, action_description, tool_name, tool_input,
                        requested_at, timeout_secs, status, approver, reason, decided_at, expired_at
                 FROM approval_requests
                 WHERE id = ?1",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare get query: {e}")))?;

        let mut rows = stmt
            .query_map(params![id], |row| Ok(Self::row_to_request(row)))
            .map_err(|e| AgentixError::runtime(format!("Failed to query approval request: {e}")))?;

        match rows.next() {
            Some(Ok(req)) => Ok(Some(req)),
            Some(Err(e)) => Err(AgentixError::runtime(format!(
                "Failed to read approval row: {e}"
            ))),
            None => Ok(None),
        }
    }

    /// List all pending approval requests, most recent first.
    pub fn list_pending(&self, limit: u32) -> Result<Vec<ApprovalRequest>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, run_id, agent_name, action_description, tool_name, tool_input,
                        requested_at, timeout_secs, status, approver, reason, decided_at, expired_at
                 FROM approval_requests
                 WHERE status = 'pending'
                 ORDER BY requested_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to prepare list_pending query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![limit], |row| Ok(Self::row_to_request(row)))
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to query pending approvals: {e}"))
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let req = row.map_err(|e| {
                AgentixError::runtime(format!("Failed to read pending approval row: {e}"))
            })?;
            requests.push(req);
        }
        Ok(requests)
    }

    /// List approval requests for a specific agent, most recent first.
    pub fn list_by_agent(
        &self,
        agent_name: &str,
        limit: u32,
    ) -> Result<Vec<ApprovalRequest>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, run_id, agent_name, action_description, tool_name, tool_input,
                        requested_at, timeout_secs, status, approver, reason, decided_at, expired_at
                 FROM approval_requests
                 WHERE agent_name = ?1
                 ORDER BY requested_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to prepare list_by_agent query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![agent_name, limit], |row| {
                Ok(Self::row_to_request(row))
            })
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to query agent approvals: {e}"))
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let req = row.map_err(|e| {
                AgentixError::runtime(format!("Failed to read agent approval row: {e}"))
            })?;
            requests.push(req);
        }
        Ok(requests)
    }

    /// Approve a pending request by ID. Returns the updated request.
    pub fn approve(
        &self,
        id: &str,
        approver: &str,
        _reason: Option<&str>,
    ) -> Result<ApprovalRequest, AgentixError> {
        self.assert_pending(id)?;

        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE approval_requests
             SET status = 'approved', approver = ?1, decided_at = ?2
             WHERE id = ?3",
            params![approver, now, id],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to approve request: {e}")))?;

        drop(conn);
        self.get_request(id)?
            .ok_or_else(|| AgentixError::runtime("Request disappeared after update".to_string()))
    }

    /// Deny a pending request by ID. Returns the updated request.
    pub fn deny(
        &self,
        id: &str,
        approver: &str,
        reason: Option<&str>,
    ) -> Result<ApprovalRequest, AgentixError> {
        self.assert_pending(id)?;

        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE approval_requests
             SET status = 'denied', approver = ?1, reason = ?2, decided_at = ?3
             WHERE id = ?4",
            params![approver, reason, now, id],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to deny request: {e}")))?;

        drop(conn);
        self.get_request(id)?
            .ok_or_else(|| AgentixError::runtime("Request disappeared after update".to_string()))
    }

    /// Mark all expired pending requests as timed_out.
    /// Returns the number of requests that were expired.
    ///
    /// Uses Unix epoch arithmetic (via `strftime('%s', ...)`) to avoid
    /// SQLite datetime parsing issues with RFC3339 nanosecond-precision timestamps.
    pub fn expire_stale(&self) -> Result<usize, AgentixError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        let count = conn
            .execute(
                "UPDATE approval_requests
                 SET status = 'timed_out', expired_at = ?1
                 WHERE status = 'pending'
                   AND (CAST(strftime('%s', requested_at) AS INTEGER) + timeout_secs)
                       <= CAST(strftime('%s', ?2) AS INTEGER)",
                params![now, now],
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to expire stale approvals: {e}"))
            })?;
        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Assert that a request exists and is Pending. Returns error otherwise.
    fn assert_pending(&self, id: &str) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        let status: Result<String, _> = conn.query_row(
            "SELECT status FROM approval_requests WHERE id = ?1",
            params![id],
            |row| row.get(0),
        );
        match status {
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(AgentixError::runtime(format!(
                "Approval request '{}' not found",
                id
            ))),
            Err(e) => Err(AgentixError::runtime(format!(
                "Failed to query approval request: {e}"
            ))),
            Ok(s) if s != "pending" => Err(AgentixError::runtime(format!(
                "Approval request '{}' is not pending (current status: {})",
                id, s
            ))),
            Ok(_) => Ok(()),
        }
    }

    /// Convert a SQLite row to an `ApprovalRequest`.
    ///
    /// Column order:
    ///   0=id, 1=run_id, 2=agent_name, 3=action_description,
    ///   4=tool_name, 5=tool_input, 6=requested_at, 7=timeout_secs,
    ///   8=status, 9=approver, 10=reason, 11=decided_at, 12=expired_at
    fn row_to_request(row: &rusqlite::Row) -> ApprovalRequest {
        let id: String = row.get(0).unwrap_or_default();
        let run_id: String = row.get(1).unwrap_or_default();
        let agent_name: String = row.get(2).unwrap_or_default();
        let action_description: String = row.get(3).unwrap_or_default();
        let tool_name: Option<String> = row.get(4).unwrap_or(None);
        let tool_input_str: Option<String> = row.get(5).unwrap_or(None);
        let requested_at_str: String = row.get(6).unwrap_or_default();
        let timeout_secs: u32 = row.get::<_, i64>(7).unwrap_or(300) as u32;
        let status_str: String = row.get(8).unwrap_or_else(|_| "pending".to_string());
        let approver: Option<String> = row.get(9).unwrap_or(None);
        let reason: Option<String> = row.get(10).unwrap_or(None);
        let decided_at_str: Option<String> = row.get(11).unwrap_or(None);
        let expired_at_str: Option<String> = row.get(12).unwrap_or(None);

        let tool_input: Option<serde_json::Value> = tool_input_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        let requested_at = DateTime::parse_from_rfc3339(&requested_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let parse_dt = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        };

        let status = match status_str.as_str() {
            "approved" => ApprovalStatus::Approved {
                approver: approver.unwrap_or_default(),
                decided_at: decided_at_str
                    .as_deref()
                    .map(parse_dt)
                    .unwrap_or_else(Utc::now),
            },
            "denied" => ApprovalStatus::Denied {
                approver: approver.unwrap_or_default(),
                reason,
                decided_at: decided_at_str
                    .as_deref()
                    .map(parse_dt)
                    .unwrap_or_else(Utc::now),
            },
            "timed_out" => ApprovalStatus::TimedOut {
                expired_at: expired_at_str
                    .as_deref()
                    .map(parse_dt)
                    .unwrap_or_else(Utc::now),
            },
            _ => ApprovalStatus::Pending,
        };

        ApprovalRequest {
            id,
            run_id,
            agent_name,
            action_description,
            tool_name,
            tool_input,
            requested_at,
            timeout_secs,
            status,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> ApprovalStore {
        ApprovalStore::open(":memory:").expect("Failed to open in-memory store")
    }

    fn test_request(id: &str) -> ApprovalRequest {
        let mut req = ApprovalRequest::new(
            "run-001".to_string(),
            "test-agent".to_string(),
            "Delete pod nginx".to_string(),
            Some("kubectl".to_string()),
            Some(serde_json::json!({"args": "delete pod nginx"})),
            300,
        );
        req.id = id.to_string();
        req
    }

    #[test]
    fn test_create_and_get_request() {
        let store = test_store();
        let req = test_request("req-001");

        store.create_request(&req).unwrap();

        let retrieved = store.get_request("req-001").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "req-001");
        assert_eq!(retrieved.agent_name, "test-agent");
        assert_eq!(retrieved.run_id, "run-001");
        assert_eq!(retrieved.tool_name, Some("kubectl".to_string()));
        assert!(matches!(retrieved.status, ApprovalStatus::Pending));
    }

    #[test]
    fn test_get_nonexistent_request() {
        let store = test_store();
        let result = store.get_request("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_pending() {
        let store = test_store();

        store.create_request(&test_request("req-a")).unwrap();
        store.create_request(&test_request("req-b")).unwrap();
        store.create_request(&test_request("req-c")).unwrap();

        let pending = store.list_pending(10).unwrap();
        assert_eq!(pending.len(), 3);

        for req in &pending {
            assert!(matches!(req.status, ApprovalStatus::Pending));
        }
    }

    #[test]
    fn test_list_pending_excludes_decided() {
        let store = test_store();

        store.create_request(&test_request("req-1")).unwrap();
        store.create_request(&test_request("req-2")).unwrap();

        store.approve("req-1", "admin", None).unwrap();

        let pending = store.list_pending(10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "req-2");
    }

    #[test]
    fn test_approve_request() {
        let store = test_store();
        store.create_request(&test_request("req-approve")).unwrap();

        let updated = store
            .approve("req-approve", "admin@co.com", Some("LGTM"))
            .unwrap();

        match &updated.status {
            ApprovalStatus::Approved { approver, .. } => {
                assert_eq!(approver, "admin@co.com");
            }
            other => panic!("Expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn test_deny_request() {
        let store = test_store();
        store.create_request(&test_request("req-deny")).unwrap();

        let updated = store
            .deny("req-deny", "security-lead", Some("Too risky"))
            .unwrap();

        match &updated.status {
            ApprovalStatus::Denied { approver, reason, .. } => {
                assert_eq!(approver, "security-lead");
                assert_eq!(reason.as_deref(), Some("Too risky"));
            }
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    #[test]
    fn test_cannot_approve_already_decided() {
        let store = test_store();
        store.create_request(&test_request("req-double")).unwrap();

        store.approve("req-double", "admin", None).unwrap();

        let err = store.approve("req-double", "admin2", None).unwrap_err();
        assert!(err.to_string().contains("not pending"));
    }

    #[test]
    fn test_cannot_deny_already_decided() {
        let store = test_store();
        store.create_request(&test_request("req-dd")).unwrap();

        store.deny("req-dd", "admin", None).unwrap();

        let err = store.deny("req-dd", "admin2", None).unwrap_err();
        assert!(err.to_string().contains("not pending"));
    }

    #[test]
    fn test_decide_nonexistent() {
        let store = test_store();
        let err = store.approve("nonexistent", "admin", None).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_list_by_agent() {
        let store = test_store();

        let mut req1 = test_request("req-agent-1");
        req1.agent_name = "agent-a".to_string();
        let mut req2 = test_request("req-agent-2");
        req2.agent_name = "agent-b".to_string();
        let mut req3 = test_request("req-agent-3");
        req3.agent_name = "agent-a".to_string();

        store.create_request(&req1).unwrap();
        store.create_request(&req2).unwrap();
        store.create_request(&req3).unwrap();

        let agent_a_reqs = store.list_by_agent("agent-a", 10).unwrap();
        assert_eq!(agent_a_reqs.len(), 2);

        let agent_b_reqs = store.list_by_agent("agent-b", 10).unwrap();
        assert_eq!(agent_b_reqs.len(), 1);
    }

    #[test]
    fn test_expire_stale() {
        let store = test_store();

        // Create a request with 0 timeout (already expired)
        let mut req_zero = ApprovalRequest::new(
            "run".to_string(),
            "agent".to_string(),
            "desc".to_string(),
            None,
            None,
            0, // expires immediately
        );
        req_zero.id = "req-expire".to_string();
        store.create_request(&req_zero).unwrap();

        // Also create one that won't expire
        store.create_request(&test_request("req-keep")).unwrap();

        let expired_count = store.expire_stale().unwrap();
        assert_eq!(expired_count, 1);

        // Verify the expired one
        let expired = store.get_request("req-expire").unwrap().unwrap();
        assert!(matches!(expired.status, ApprovalStatus::TimedOut { .. }));

        // Verify the kept one
        let kept = store.get_request("req-keep").unwrap().unwrap();
        assert!(matches!(kept.status, ApprovalStatus::Pending));
    }

    #[test]
    fn test_list_pending_respects_limit() {
        let store = test_store();

        for i in 0..5 {
            store
                .create_request(&test_request(&format!("req-lim-{}", i)))
                .unwrap();
        }

        let limited = store.list_pending(3).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn test_tool_input_preserved() {
        let store = test_store();
        let mut req = test_request("req-input");
        req.tool_input = Some(serde_json::json!({
            "command": "kubectl delete pod nginx",
            "namespace": "production",
            "force": true
        }));

        store.create_request(&req).unwrap();

        let retrieved = store.get_request("req-input").unwrap().unwrap();
        let input = retrieved.tool_input.unwrap();
        assert_eq!(input["command"], "kubectl delete pod nginx");
        assert_eq!(input["namespace"], "production");
        assert_eq!(input["force"], true);
    }
}
