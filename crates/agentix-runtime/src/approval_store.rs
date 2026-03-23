//! SQLite-backed approval request store for OpenAgentiX.
//!
//! Persists approval requests so that paused agent runs can survive gateway
//! restarts. Uses `rusqlite` with `parking_lot::Mutex` for thread-safe access,
//! following the same pattern as `AuditStore`, `CostStore`, and `TraceStore`.

use std::collections::HashMap;
use std::sync::Arc;

use agentix_core::{
    AgentixError, ApprovalAction, ApprovalDecision, ApprovalRequest,
    ApprovalStatus,
};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde_json;

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
                id           TEXT PRIMARY KEY,
                agent_name   TEXT NOT NULL,
                run_id       TEXT NOT NULL,
                tool_name    TEXT NOT NULL,
                tool_input   TEXT NOT NULL DEFAULT '{}',
                description  TEXT NOT NULL,
                status       TEXT NOT NULL DEFAULT 'pending',
                created_at   TEXT NOT NULL,
                expires_at   TEXT NOT NULL,
                approver     TEXT,
                action       TEXT,
                reason       TEXT,
                decided_at   TEXT,
                metadata     TEXT NOT NULL DEFAULT '{}'
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
        let tool_input_json =
            serde_json::to_string(&request.tool_input).unwrap_or_else(|_| "{}".to_string());
        let metadata_json =
            serde_json::to_string(&request.metadata).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO approval_requests
             (id, agent_name, run_id, tool_name, tool_input, description, status, created_at, expires_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                request.id,
                request.agent_name,
                request.run_id,
                request.tool_name,
                tool_input_json,
                request.description,
                request.status.to_string(),
                request.created_at.to_rfc3339(),
                request.expires_at.to_rfc3339(),
                metadata_json,
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
                "SELECT id, agent_name, run_id, tool_name, tool_input, description,
                        status, created_at, expires_at, approver, action, reason, decided_at, metadata
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
                "SELECT id, agent_name, run_id, tool_name, tool_input, description,
                        status, created_at, expires_at, approver, action, reason, decided_at, metadata
                 FROM approval_requests
                 WHERE status = 'pending'
                 ORDER BY created_at DESC
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
                "SELECT id, agent_name, run_id, tool_name, tool_input, description,
                        status, created_at, expires_at, approver, action, reason, decided_at, metadata
                 FROM approval_requests
                 WHERE agent_name = ?1
                 ORDER BY created_at DESC
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

    /// Return all Pending approval requests for a specific run.
    pub fn get_pending_by_run(&self, run_id: &str) -> Result<Vec<ApprovalRequest>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_name, run_id, tool_name, tool_input, description,
                        status, created_at, expires_at, approver, action, reason, decided_at, metadata
                 FROM approval_requests
                 WHERE run_id = ?1 AND status = 'pending'
                 ORDER BY created_at DESC",
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to prepare get_pending_by_run query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![run_id], |row| Ok(Self::row_to_request(row)))
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to query pending by run: {e}"))
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let req = row.map_err(|e| {
                AgentixError::runtime(format!("Failed to read pending by run row: {e}"))
            })?;
            requests.push(req);
        }
        Ok(requests)
    }

    /// Return all Pending approval requests for a specific agent.
    pub fn get_pending_by_agent(&self, agent_name: &str) -> Result<Vec<ApprovalRequest>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_name, run_id, tool_name, tool_input, description,
                        status, created_at, expires_at, approver, action, reason, decided_at, metadata
                 FROM approval_requests
                 WHERE agent_name = ?1 AND status = 'pending'
                 ORDER BY created_at DESC",
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to prepare get_pending_by_agent query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![agent_name], |row| Ok(Self::row_to_request(row)))
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to query pending by agent: {e}"))
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let req = row.map_err(|e| {
                AgentixError::runtime(format!("Failed to read pending by agent row: {e}"))
            })?;
            requests.push(req);
        }
        Ok(requests)
    }

    /// Update the status of an approval request.
    ///
    /// For Approved/Denied transitions, an `ApprovalDecision` must be provided.
    /// For Expired transitions, no decision is needed.
    pub fn update_status(
        &self,
        request_id: &str,
        new_status: &ApprovalStatus,
        decision: Option<ApprovalDecision>,
    ) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        let status_str = match new_status {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Denied => "denied",
            ApprovalStatus::Expired => "expired",
        };

        match decision {
            Some(dec) => {
                let action_str = match dec.action {
                    ApprovalAction::Approve => "approve",
                    ApprovalAction::Deny => "deny",
                };
                conn.execute(
                    "UPDATE approval_requests
                     SET status = ?1, approver = ?2, action = ?3, reason = ?4, decided_at = ?5
                     WHERE id = ?6",
                    params![
                        status_str,
                        dec.approver,
                        action_str,
                        dec.reason,
                        dec.decided_at.to_rfc3339(),
                        request_id,
                    ],
                )
                .map_err(|e| {
                    AgentixError::runtime(format!("Failed to update approval status: {e}"))
                })?;
            }
            None => {
                conn.execute(
                    "UPDATE approval_requests SET status = ?1 WHERE id = ?2",
                    params![status_str, request_id],
                )
                .map_err(|e| {
                    AgentixError::runtime(format!("Failed to update approval status: {e}"))
                })?;
            }
        }

        Ok(())
    }

    /// Mark all expired pending requests as Expired.
    ///
    /// A pending request is expired when `expires_at < NOW()`.
    /// Returns the number of requests that were marked expired.
    pub fn expire_timed_out(&self) -> Result<u32, AgentixError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        let count = conn
            .execute(
                "UPDATE approval_requests
                 SET status = 'expired'
                 WHERE status = 'pending' AND expires_at < ?1",
                params![now],
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to expire timed-out approvals: {e}"))
            })?;
        Ok(count as u32)
    }

    /// Approve a pending request. Returns the updated request.
    pub fn approve(
        &self,
        id: &str,
        approver: &str,
        reason: Option<&str>,
    ) -> Result<ApprovalRequest, AgentixError> {
        self.decide(id, approver, ApprovalAction::Approve, reason)
    }

    /// Deny a pending request. Returns the updated request.
    pub fn deny(
        &self,
        id: &str,
        approver: &str,
        reason: Option<&str>,
    ) -> Result<ApprovalRequest, AgentixError> {
        self.decide(id, approver, ApprovalAction::Deny, reason)
    }

    /// Apply a decision to a pending request.
    fn decide(
        &self,
        id: &str,
        approver: &str,
        action: ApprovalAction,
        reason: Option<&str>,
    ) -> Result<ApprovalRequest, AgentixError> {
        let conn = self.conn.lock();
        let now = Utc::now();
        let status = match action {
            ApprovalAction::Approve => "approved",
            ApprovalAction::Deny => "denied",
        };
        let action_str = match action {
            ApprovalAction::Approve => "approve",
            ApprovalAction::Deny => "deny",
        };

        // Verify the request exists and is pending
        let current_status: String = conn
            .query_row(
                "SELECT status FROM approval_requests WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AgentixError::runtime(format!("Approval request '{}' not found", id))
                }
                _ => AgentixError::runtime(format!("Failed to query approval request: {e}")),
            })?;

        if current_status != "pending" {
            return Err(AgentixError::runtime(format!(
                "Approval request '{}' is not pending (current status: {})",
                id, current_status
            )));
        }

        conn.execute(
            "UPDATE approval_requests
             SET status = ?1, approver = ?2, action = ?3, reason = ?4, decided_at = ?5
             WHERE id = ?6",
            params![
                status,
                approver,
                action_str,
                reason,
                now.to_rfc3339(),
                id,
            ],
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to update approval request: {e}")))?;

        drop(conn);
        self.get_request(id)?
            .ok_or_else(|| AgentixError::runtime("Request disappeared after update".to_string()))
    }

    /// Mark all expired pending requests as expired.
    /// Returns the number of requests that were expired.
    pub fn expire_stale(&self) -> Result<usize, AgentixError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        let count = conn
            .execute(
                "UPDATE approval_requests
                 SET status = 'expired'
                 WHERE status = 'pending' AND expires_at < ?1",
                params![now],
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to expire stale approvals: {e}"))
            })?;
        Ok(count)
    }

    /// Convert a SQLite row to an ApprovalRequest.
    fn row_to_request(row: &rusqlite::Row) -> ApprovalRequest {
        let id: String = row.get(0).unwrap_or_default();
        let agent_name: String = row.get(1).unwrap_or_default();
        let run_id: String = row.get(2).unwrap_or_default();
        let tool_name: String = row.get(3).unwrap_or_default();
        let tool_input_str: String = row.get(4).unwrap_or_else(|_| "{}".to_string());
        let description: String = row.get(5).unwrap_or_default();
        let status_str: String = row.get(6).unwrap_or_else(|_| "pending".to_string());
        let created_at_str: String = row.get(7).unwrap_or_default();
        let expires_at_str: String = row.get(8).unwrap_or_default();
        let approver: Option<String> = row.get(9).unwrap_or(None);
        let action_str: Option<String> = row.get(10).unwrap_or(None);
        let reason: Option<String> = row.get(11).unwrap_or(None);
        let decided_at_str: Option<String> = row.get(12).unwrap_or(None);
        let metadata_str: String = row.get(13).unwrap_or_else(|_| "{}".to_string());

        let tool_input: serde_json::Value =
            serde_json::from_str(&tool_input_str).unwrap_or(serde_json::Value::Null);
        let metadata: HashMap<String, serde_json::Value> =
            serde_json::from_str(&metadata_str).unwrap_or_default();

        let status = match status_str.as_str() {
            "approved" => ApprovalStatus::Approved,
            "denied" => ApprovalStatus::Denied,
            "expired" => ApprovalStatus::Expired,
            _ => ApprovalStatus::Pending,
        };

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let decision = match (approver, action_str, decided_at_str) {
            (Some(approver), Some(action), Some(decided_at)) => {
                let action = match action.as_str() {
                    "approve" => ApprovalAction::Approve,
                    "deny" => ApprovalAction::Deny,
                    _ => ApprovalAction::Deny,
                };
                let decided_at = DateTime::parse_from_rfc3339(&decided_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Some(ApprovalDecision {
                    approver,
                    action,
                    reason,
                    decided_at,
                })
            }
            _ => None,
        };

        ApprovalRequest {
            id,
            agent_name,
            run_id,
            tool_name,
            tool_input,
            description,
            status,
            created_at,
            expires_at,
            decision,
            metadata,
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
        ApprovalRequest::new(
            id.to_string(),
            "test-agent".to_string(),
            "run-001".to_string(),
            "kubectl".to_string(),
            serde_json::json!({"args": "delete pod nginx"}),
            "Delete pod nginx".to_string(),
            300,
        )
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
        assert_eq!(retrieved.tool_name, "kubectl");
        assert_eq!(retrieved.status, ApprovalStatus::Pending);
        assert!(retrieved.decision.is_none());
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

        // Create 3 requests
        store.create_request(&test_request("req-a")).unwrap();
        store.create_request(&test_request("req-b")).unwrap();
        store.create_request(&test_request("req-c")).unwrap();

        let pending = store.list_pending(10).unwrap();
        assert_eq!(pending.len(), 3);

        // All should be pending
        for req in &pending {
            assert_eq!(req.status, ApprovalStatus::Pending);
        }
    }

    #[test]
    fn test_list_pending_excludes_decided() {
        let store = test_store();

        store.create_request(&test_request("req-1")).unwrap();
        store.create_request(&test_request("req-2")).unwrap();

        // Approve req-1
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

        assert_eq!(updated.status, ApprovalStatus::Approved);
        assert!(updated.decision.is_some());
        let decision = updated.decision.unwrap();
        assert_eq!(decision.approver, "admin@co.com");
        assert_eq!(decision.action, ApprovalAction::Approve);
        assert_eq!(decision.reason, Some("LGTM".to_string()));
    }

    #[test]
    fn test_deny_request() {
        let store = test_store();
        store.create_request(&test_request("req-deny")).unwrap();

        let updated = store
            .deny("req-deny", "security-lead", Some("Too risky"))
            .unwrap();

        assert_eq!(updated.status, ApprovalStatus::Denied);
        assert!(updated.decision.is_some());
        let decision = updated.decision.unwrap();
        assert_eq!(decision.approver, "security-lead");
        assert_eq!(decision.action, ApprovalAction::Deny);
        assert_eq!(decision.reason, Some("Too risky".to_string()));
    }

    #[test]
    fn test_cannot_approve_already_decided() {
        let store = test_store();
        store.create_request(&test_request("req-double")).unwrap();

        // First approval succeeds
        store.approve("req-double", "admin", None).unwrap();

        // Second approval fails
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
        let req = ApprovalRequest::new(
            "req-expire".to_string(),
            "agent".to_string(),
            "run".to_string(),
            "tool".to_string(),
            serde_json::Value::Null,
            "desc".to_string(),
            0, // expires immediately
        );
        store.create_request(&req).unwrap();

        // Also create one that won't expire
        store.create_request(&test_request("req-keep")).unwrap();

        let expired_count = store.expire_stale().unwrap();
        assert_eq!(expired_count, 1);

        // Verify the expired one
        let expired = store.get_request("req-expire").unwrap().unwrap();
        assert_eq!(expired.status, ApprovalStatus::Expired);

        // Verify the kept one
        let kept = store.get_request("req-keep").unwrap().unwrap();
        assert_eq!(kept.status, ApprovalStatus::Pending);
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
        req.tool_input = serde_json::json!({
            "command": "kubectl delete pod nginx",
            "namespace": "production",
            "force": true
        });

        store.create_request(&req).unwrap();

        let retrieved = store.get_request("req-input").unwrap().unwrap();
        assert_eq!(
            retrieved.tool_input["command"],
            "kubectl delete pod nginx"
        );
        assert_eq!(retrieved.tool_input["namespace"], "production");
        assert_eq!(retrieved.tool_input["force"], true);
    }
}
