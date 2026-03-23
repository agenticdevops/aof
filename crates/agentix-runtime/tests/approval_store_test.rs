//! Integration tests for ApprovalStore — SQLite-backed approval persistence.
//!
//! Tests follow TDD pattern for Plan 20-02. All tests use `:memory:` SQLite for isolation.

use agentix_core::{ApprovalRequest, ApprovalStatus};
use agentix_runtime::ApprovalStore;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_store() -> ApprovalStore {
    ApprovalStore::open(":memory:").expect("failed to open in-memory ApprovalStore")
}

fn make_request(id: &str, run_id: &str, agent_name: &str) -> ApprovalRequest {
    let mut req = ApprovalRequest::new(
        run_id.to_string(),
        agent_name.to_string(),
        "Delete pod nginx in production".to_string(),
        Some("kubectl".to_string()),
        Some(serde_json::json!({"args": "delete pod nginx"})),
        300,
    );
    req.id = id.to_string();
    req
}

fn make_expired_request(id: &str, run_id: &str, agent_name: &str) -> ApprovalRequest {
    let mut req = ApprovalRequest::new(
        run_id.to_string(),
        agent_name.to_string(),
        "Uninstall helm release".to_string(),
        Some("helm".to_string()),
        Some(serde_json::json!({"args": "uninstall my-release"})),
        0, // expires immediately
    );
    req.id = id.to_string();
    req
}

// ---------------------------------------------------------------------------
// Test 1: create_and_get_request
// ---------------------------------------------------------------------------

#[test]
fn create_and_get_request() {
    let store = make_store();
    let req = make_request("req-001", "run-aaa", "dba-agent");

    store.create_request(&req).unwrap();

    let retrieved = store.get_request("req-001").unwrap();
    assert!(retrieved.is_some(), "Expected Some but got None");
    let retrieved = retrieved.unwrap();

    assert_eq!(retrieved.id, "req-001");
    assert_eq!(retrieved.run_id, "run-aaa");
    assert_eq!(retrieved.agent_name, "dba-agent");
    assert_eq!(retrieved.tool_name, Some("kubectl".to_string()));
    assert!(matches!(retrieved.status, ApprovalStatus::Pending));
}

// ---------------------------------------------------------------------------
// Test 2: get_request_not_found
// ---------------------------------------------------------------------------

#[test]
fn get_request_not_found() {
    let store = make_store();
    let result = store.get_request("nonexistent-id").unwrap();
    assert!(result.is_none(), "Expected None for nonexistent ID");
}

// ---------------------------------------------------------------------------
// Test 3: list_pending_by_run (via list_pending)
// ---------------------------------------------------------------------------

#[test]
fn list_pending_with_multiple_runs() {
    let store = make_store();

    store.create_request(&make_request("req-a1", "run-A", "agent-x")).unwrap();
    store.create_request(&make_request("req-a2", "run-A", "agent-x")).unwrap();
    store.create_request(&make_request("req-b1", "run-B", "agent-x")).unwrap();

    let all_pending = store.list_pending(10).unwrap();
    assert_eq!(all_pending.len(), 3, "Expected 3 total Pending requests");
    for r in &all_pending {
        assert!(matches!(r.status, ApprovalStatus::Pending));
    }
}

// ---------------------------------------------------------------------------
// Test 4: list_by_agent
// ---------------------------------------------------------------------------

#[test]
fn get_pending_by_agent() {
    let store = make_store();

    store.create_request(&make_request("req-d1", "run-1", "dba")).unwrap();
    store.create_request(&make_request("req-d2", "run-2", "dba")).unwrap();
    store.create_request(&make_request("req-s1", "run-3", "security")).unwrap();

    let dba_pending = store.list_by_agent("dba", 10).unwrap();
    assert_eq!(dba_pending.len(), 2, "Expected exactly 2 requests for dba");
    for r in &dba_pending {
        assert_eq!(r.agent_name, "dba");
    }

    let security_pending = store.list_by_agent("security", 10).unwrap();
    assert_eq!(security_pending.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 5: list_pending with limit
// ---------------------------------------------------------------------------

#[test]
fn list_pending_with_limit() {
    let store = make_store();

    for i in 0..5 {
        store
            .create_request(&make_request(&format!("req-lim-{}", i), "run-X", "agent"))
            .unwrap();
    }

    let results = store.list_pending(3).unwrap();
    assert_eq!(results.len(), 3, "Expected limit=3 to return exactly 3 entries");
    for r in &results {
        assert!(matches!(r.status, ApprovalStatus::Pending));
    }
}

// ---------------------------------------------------------------------------
// Test 6: approve
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_approved() {
    let store = make_store();
    let req = make_request("req-approve", "run-Z", "dba");
    store.create_request(&req).unwrap();

    let approved = store.approve("req-approve", "admin@company.com", None).unwrap();

    match &approved.status {
        ApprovalStatus::Approved { approver, decided_at } => {
            assert_eq!(approver, "admin@company.com");
            let age = chrono::Utc::now().signed_duration_since(*decided_at).num_seconds().abs();
            assert!(age < 5, "decided_at should be recent");
        }
        other => panic!("Expected Approved status, got {:?}", other),
    }

    // Verify persisted
    let retrieved = store.get_request("req-approve").unwrap().unwrap();
    assert!(matches!(retrieved.status, ApprovalStatus::Approved { .. }));
}

// ---------------------------------------------------------------------------
// Test 7: deny
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_denied() {
    let store = make_store();
    store.create_request(&make_request("req-deny", "run-Z", "dba")).unwrap();

    let denied = store.deny("req-deny", "security-lead", Some("Too risky")).unwrap();

    match &denied.status {
        ApprovalStatus::Denied { approver, reason, .. } => {
            assert_eq!(approver, "security-lead");
            assert_eq!(reason.as_deref(), Some("Too risky"));
        }
        other => panic!("Expected Denied status, got {:?}", other),
    }

    // Verify persisted
    let retrieved = store.get_request("req-deny").unwrap().unwrap();
    match &retrieved.status {
        ApprovalStatus::Denied { reason, .. } => {
            assert_eq!(reason.as_deref(), Some("Too risky"));
        }
        other => panic!("Expected Denied when retrieved, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test 8: expire_stale (timed_out)
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_expired() {
    let store = make_store();
    store.create_request(&make_request("req-timeout", "run-Z", "dba")).unwrap();

    // expire_stale marks requests where timeout has elapsed
    // For a 300s timeout, nothing expires immediately — so we call expire_stale
    // and verify it returns 0 for fresh requests
    let expired = store.expire_stale().unwrap();
    assert_eq!(expired, 0, "No requests should have expired yet");

    // get_request should still show Pending
    let retrieved = store.get_request("req-timeout").unwrap().unwrap();
    assert!(matches!(retrieved.status, ApprovalStatus::Pending));
}

// ---------------------------------------------------------------------------
// Test 9: list_pending_excludes_decided
// ---------------------------------------------------------------------------

#[test]
fn list_pending_excludes_decided() {
    let store = make_store();

    store.create_request(&make_request("req-p1", "run-1", "agent")).unwrap();
    store.create_request(&make_request("req-p2", "run-2", "agent")).unwrap();
    store.create_request(&make_request("req-p3", "run-3", "agent")).unwrap();

    store.approve("req-p1", "admin", None).unwrap();
    store.deny("req-p2", "admin", Some("Nope")).unwrap();

    let pending = store.list_pending(10).unwrap();
    assert_eq!(pending.len(), 1, "Only 1 should remain Pending");
    assert_eq!(pending[0].id, "req-p3");
}

// ---------------------------------------------------------------------------
// Test 10: expire_timed_out
// ---------------------------------------------------------------------------

#[test]
fn expire_timed_out() {
    let store = make_store();

    // 2 requests with 0s timeout (immediately expired)
    store.create_request(&make_expired_request("req-exp1", "run-E", "dba")).unwrap();
    store.create_request(&make_expired_request("req-exp2", "run-E", "dba")).unwrap();

    // 1 request with long timeout (not expired)
    let mut long_req = ApprovalRequest::new(
        "run-E".to_string(),
        "dba".to_string(),
        "Keep me alive".to_string(),
        Some("kubectl".to_string()),
        None,
        3600,
    );
    long_req.id = "req-keep".to_string();
    store.create_request(&long_req).unwrap();

    let expired_count = store.expire_stale().unwrap();
    assert_eq!(expired_count, 2, "Expected 2 expired requests");

    // Verify expired requests have TimedOut status
    let exp1 = store.get_request("req-exp1").unwrap().unwrap();
    assert!(matches!(exp1.status, ApprovalStatus::TimedOut { .. }));

    let exp2 = store.get_request("req-exp2").unwrap().unwrap();
    assert!(matches!(exp2.status, ApprovalStatus::TimedOut { .. }));

    // The non-expired one remains Pending
    let kept = store.get_request("req-keep").unwrap().unwrap();
    assert!(matches!(kept.status, ApprovalStatus::Pending));
}

// ---------------------------------------------------------------------------
// Test 11: double approve fails
// ---------------------------------------------------------------------------

#[test]
fn double_approve_fails() {
    let store = make_store();
    store.create_request(&make_request("req-double", "run-Z", "dba")).unwrap();

    store.approve("req-double", "admin", None).unwrap();

    let err = store.approve("req-double", "admin2", None).unwrap_err();
    assert!(
        err.to_string().contains("not pending"),
        "Error should mention 'not pending': {}",
        err
    );
}

// ---------------------------------------------------------------------------
// Test 12: cannot deny after approval
// ---------------------------------------------------------------------------

#[test]
fn deny_after_approve_fails() {
    let store = make_store();
    store.create_request(&make_request("req-da", "run-Z", "dba")).unwrap();

    store.approve("req-da", "admin", None).unwrap();

    let err = store.deny("req-da", "admin", None).unwrap_err();
    assert!(err.to_string().contains("not pending"));
}

// ---------------------------------------------------------------------------
// Test 13: nonexistent request approve fails
// ---------------------------------------------------------------------------

#[test]
fn approve_nonexistent_fails() {
    let store = make_store();
    let err = store.approve("no-such-id", "admin", None).unwrap_err();
    assert!(err.to_string().contains("not found"));
}
