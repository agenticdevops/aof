//! Integration tests for ApprovalStore — SQLite-backed approval persistence.
//!
//! Tests follow TDD pattern for Plan 20-02. All tests use `:memory:` SQLite for isolation.

use agentix_core::{
    ApprovalAction, ApprovalDecision, ApprovalRequest, ApprovalStatus,
};
use agentix_runtime::ApprovalStore;
use chrono::Utc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_store() -> ApprovalStore {
    ApprovalStore::open(":memory:").expect("failed to open in-memory ApprovalStore")
}

fn make_request(id: &str, run_id: &str, agent_name: &str) -> ApprovalRequest {
    ApprovalRequest::new(
        id.to_string(),
        agent_name.to_string(),
        run_id.to_string(),
        "kubectl".to_string(),
        serde_json::json!({"args": "delete pod nginx"}),
        "Delete pod nginx in production".to_string(),
        300,
    )
}

fn make_expired_request(id: &str, run_id: &str, agent_name: &str) -> ApprovalRequest {
    ApprovalRequest::new(
        id.to_string(),
        agent_name.to_string(),
        run_id.to_string(),
        "helm".to_string(),
        serde_json::json!({"args": "uninstall my-release"}),
        "Uninstall helm release".to_string(),
        0, // expires immediately
    )
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
    assert_eq!(retrieved.tool_name, "kubectl");
    assert_eq!(retrieved.status, ApprovalStatus::Pending);
    assert!(retrieved.decision.is_none());
    // expires_at must be after created_at (300s timeout)
    assert!(retrieved.expires_at > retrieved.created_at);
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
// Test 3: get_pending_by_run
// ---------------------------------------------------------------------------

#[test]
fn get_pending_by_run() {
    let store = make_store();

    // 2 for run-A, 1 for run-B — all Pending
    store.create_request(&make_request("req-a1", "run-A", "agent-x")).unwrap();
    store.create_request(&make_request("req-a2", "run-A", "agent-x")).unwrap();
    store.create_request(&make_request("req-b1", "run-B", "agent-x")).unwrap();

    let run_a_pending = store.get_pending_by_run("run-A").unwrap();
    assert_eq!(run_a_pending.len(), 2, "Expected exactly 2 Pending requests for run-A");
    for r in &run_a_pending {
        assert_eq!(r.run_id, "run-A");
        assert_eq!(r.status, ApprovalStatus::Pending);
    }
}

// ---------------------------------------------------------------------------
// Test 4: get_pending_by_agent
// ---------------------------------------------------------------------------

#[test]
fn get_pending_by_agent() {
    let store = make_store();

    // 2 for "dba", 1 for "security"
    store.create_request(&make_request("req-d1", "run-1", "dba")).unwrap();
    store.create_request(&make_request("req-d2", "run-2", "dba")).unwrap();
    store.create_request(&make_request("req-s1", "run-3", "security")).unwrap();

    let dba_pending = store.get_pending_by_agent("dba").unwrap();
    assert_eq!(dba_pending.len(), 2, "Expected exactly 2 Pending requests for dba");
    for r in &dba_pending {
        assert_eq!(r.agent_name, "dba");
        assert_eq!(r.status, ApprovalStatus::Pending);
    }

    let security_pending = store.get_pending_by_agent("security").unwrap();
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
    // All must be Pending
    for r in &results {
        assert_eq!(r.status, ApprovalStatus::Pending);
    }
}

// ---------------------------------------------------------------------------
// Test 6: update_status_to_approved
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_approved() {
    let store = make_store();
    let req = make_request("req-approve", "run-Z", "dba");
    store.create_request(&req).unwrap();

    let decision = ApprovalDecision {
        approver: "admin@company.com".to_string(),
        action: ApprovalAction::Approve,
        reason: Some("Looks good".to_string()),
        decided_at: Utc::now(),
    };
    let approved_status = ApprovalStatus::Approved;
    store.update_status("req-approve", &approved_status, Some(decision.clone())).unwrap();

    let retrieved = store.get_request("req-approve").unwrap().unwrap();
    assert_eq!(retrieved.status, ApprovalStatus::Approved);
    assert!(retrieved.decision.is_some());
    let dec = retrieved.decision.unwrap();
    assert_eq!(dec.approver, "admin@company.com");
    assert!(dec.decided_at <= Utc::now());
}

// ---------------------------------------------------------------------------
// Test 7: update_status_to_denied
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_denied() {
    let store = make_store();
    store.create_request(&make_request("req-deny", "run-Z", "dba")).unwrap();

    let decision = ApprovalDecision {
        approver: "security-lead".to_string(),
        action: ApprovalAction::Deny,
        reason: Some("Too risky".to_string()),
        decided_at: Utc::now(),
    };
    store.update_status("req-deny", &ApprovalStatus::Denied, Some(decision)).unwrap();

    let retrieved = store.get_request("req-deny").unwrap().unwrap();
    assert_eq!(retrieved.status, ApprovalStatus::Denied);
    let dec = retrieved.decision.unwrap();
    assert_eq!(dec.approver, "security-lead");
    assert_eq!(dec.reason, Some("Too risky".to_string()));
    assert!(dec.decided_at <= Utc::now());
}

// ---------------------------------------------------------------------------
// Test 8: update_status_to_timed_out
// ---------------------------------------------------------------------------

#[test]
fn update_status_to_expired() {
    let store = make_store();
    store.create_request(&make_request("req-timeout", "run-Z", "dba")).unwrap();

    store.update_status("req-timeout", &ApprovalStatus::Expired, None).unwrap();

    let retrieved = store.get_request("req-timeout").unwrap().unwrap();
    assert_eq!(retrieved.status, ApprovalStatus::Expired);
    // No decision for expiry
    assert!(retrieved.decision.is_none());
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

    // Approve req-p1
    let approve_dec = ApprovalDecision {
        approver: "admin".to_string(),
        action: ApprovalAction::Approve,
        reason: None,
        decided_at: Utc::now(),
    };
    store.update_status("req-p1", &ApprovalStatus::Approved, Some(approve_dec)).unwrap();

    // Deny req-p2
    let deny_dec = ApprovalDecision {
        approver: "admin".to_string(),
        action: ApprovalAction::Deny,
        reason: Some("Nope".to_string()),
        decided_at: Utc::now(),
    };
    store.update_status("req-p2", &ApprovalStatus::Denied, Some(deny_dec)).unwrap();

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
    let long_req = ApprovalRequest::new(
        "req-keep".to_string(),
        "dba".to_string(),
        "run-E".to_string(),
        "kubectl".to_string(),
        serde_json::Value::Null,
        "Keep me alive".to_string(),
        3600,
    );
    store.create_request(&long_req).unwrap();

    let expired_count = store.expire_timed_out().unwrap();
    assert_eq!(expired_count, 2, "Expected 2 expired requests");

    // Verify expired requests have Expired status
    let exp1 = store.get_request("req-exp1").unwrap().unwrap();
    assert_eq!(exp1.status, ApprovalStatus::Expired);

    let exp2 = store.get_request("req-exp2").unwrap().unwrap();
    assert_eq!(exp2.status, ApprovalStatus::Expired);

    // The non-expired one remains Pending
    let kept = store.get_request("req-keep").unwrap().unwrap();
    assert_eq!(kept.status, ApprovalStatus::Pending);
}
