//! TDD integration tests for approval workflow core types.
//!
//! Tests cover: ApprovalRequest lifecycle, ApprovalStatus transitions,
//! ApprovalDecision serde, ApprovalPolicy mode checks.
//!
//! Plan: 20-01 (Approval Core Types)

use agentix_core::{
    AgentMode, ApprovalDecision, ApprovalPolicy, ApprovalRequest, ApprovalStatus,
};
use chrono::Utc;
use serde_json::json;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_request() -> ApprovalRequest {
    ApprovalRequest::new(
        "run-1".to_string(),
        "dba-optimizer".to_string(),
        "Delete pod nginx".to_string(),
        Some("kubectl".to_string()),
        Some(json!({"command": "delete pod nginx"})),
        300,
    )
}

// ---------------------------------------------------------------------------
// ApprovalRequest::new
// ---------------------------------------------------------------------------

#[test]
fn test_approval_request_new_fields() {
    let req = make_request();

    // UUID id must be non-empty
    assert!(!req.id.is_empty(), "id should be a non-empty UUID");

    // Field values
    assert_eq!(req.run_id, "run-1");
    assert_eq!(req.agent_name, "dba-optimizer");
    assert_eq!(req.action_description, "Delete pod nginx");
    assert_eq!(req.tool_name, Some("kubectl".to_string()));
    assert_eq!(req.tool_input, Some(json!({"command": "delete pod nginx"})));
    assert_eq!(req.timeout_secs, 300);

    // Status must be Pending
    assert!(
        matches!(req.status, ApprovalStatus::Pending),
        "new request status should be Pending"
    );

    // requested_at should be recent (within 5 seconds)
    let now = Utc::now();
    let age = now.signed_duration_since(req.requested_at).num_seconds().abs();
    assert!(age < 5, "requested_at should be recent, age={}s", age);
}

// ---------------------------------------------------------------------------
// ApprovalRequest::approve
// ---------------------------------------------------------------------------

#[test]
fn test_approval_request_approve_transitions_to_approved() {
    let mut req = make_request();
    let result = req.approve("admin@company.com");

    assert!(result.is_ok(), "approve should succeed on Pending request");

    match &req.status {
        ApprovalStatus::Approved { approver, decided_at } => {
            assert_eq!(approver, "admin@company.com");
            let age = Utc::now().signed_duration_since(*decided_at).num_seconds().abs();
            assert!(age < 5, "decided_at should be recent");
        }
        other => panic!("Expected Approved status, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// ApprovalRequest::deny
// ---------------------------------------------------------------------------

#[test]
fn test_approval_request_deny_with_reason() {
    let mut req = make_request();
    let result = req.deny("sre-lead", Some("Too risky for production".to_string()));

    assert!(result.is_ok(), "deny should succeed on Pending request");

    match &req.status {
        ApprovalStatus::Denied { approver, reason, decided_at } => {
            assert_eq!(approver, "sre-lead");
            assert_eq!(reason.as_deref(), Some("Too risky for production"));
            let age = Utc::now().signed_duration_since(*decided_at).num_seconds().abs();
            assert!(age < 5, "decided_at should be recent");
        }
        other => panic!("Expected Denied status, got {:?}", other),
    }
}

#[test]
fn test_approval_request_deny_without_reason() {
    let mut req = make_request();
    let result = req.deny("sre-lead", None);

    assert!(result.is_ok(), "deny without reason should succeed");

    match &req.status {
        ApprovalStatus::Denied { reason, .. } => {
            assert!(reason.is_none(), "reason should be None");
        }
        other => panic!("Expected Denied status, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Single-decision semantics
// ---------------------------------------------------------------------------

#[test]
fn test_approval_request_double_approve_fails() {
    let mut req = make_request();

    // First approval succeeds
    req.approve("admin@company.com").expect("first approve should succeed");

    // Second approval must fail
    let second_result = req.approve("admin2@company.com");
    assert!(
        second_result.is_err(),
        "second approve on already-decided request should fail"
    );
    let err_msg = second_result.unwrap_err();
    assert!(
        err_msg.contains("already") || err_msg.contains("decided"),
        "error should mention already decided: {}",
        err_msg
    );
}

#[test]
fn test_approval_request_approve_after_deny_fails() {
    let mut req = make_request();

    req.deny("sre-lead", Some("Too risky".to_string())).expect("deny should succeed");

    let approve_result = req.approve("admin@company.com");
    assert!(
        approve_result.is_err(),
        "approve after deny should fail"
    );
}

// ---------------------------------------------------------------------------
// ApprovalRequest::is_expired
// ---------------------------------------------------------------------------

#[test]
fn test_approval_request_is_expired_with_zero_timeout() {
    let req = ApprovalRequest::new(
        "run-expire".to_string(),
        "agent".to_string(),
        "some action".to_string(),
        None,
        None,
        0, // expires immediately
    );

    // With 0-second timeout, the request should expire immediately
    assert!(
        req.is_expired(),
        "request with 0s timeout should be expired immediately"
    );
}

#[test]
fn test_approval_request_not_expired_with_long_timeout() {
    let req = ApprovalRequest::new(
        "run-long".to_string(),
        "agent".to_string(),
        "some action".to_string(),
        None,
        None,
        3600, // 1 hour
    );

    assert!(
        !req.is_expired(),
        "request with 1h timeout should not be expired"
    );
}

// ---------------------------------------------------------------------------
// ApprovalPolicy: Autonomous mode
// ---------------------------------------------------------------------------

#[test]
fn test_approval_policy_autonomous_never_requires_approval() {
    let policy = ApprovalPolicy {
        mode: AgentMode::Autonomous,
        flagged_tools: vec![],
        flagged_patterns: vec![],
        default_timeout_secs: 300,
        approvers: vec![],
    };

    assert!(
        !policy.requires_approval("kubectl", &json!({})),
        "Autonomous mode should never require approval"
    );
    assert!(
        !policy.requires_approval("any-tool", &json!({"anything": "here"})),
        "Autonomous mode should never require approval"
    );
}

// ---------------------------------------------------------------------------
// ApprovalPolicy: Manual mode
// ---------------------------------------------------------------------------

#[test]
fn test_approval_policy_manual_always_requires_approval() {
    let policy = ApprovalPolicy {
        mode: AgentMode::Manual,
        flagged_tools: vec![],
        flagged_patterns: vec![],
        default_timeout_secs: 300,
        approvers: vec![],
    };

    assert!(
        policy.requires_approval("kubectl", &json!({})),
        "Manual mode should always require approval"
    );
    assert!(
        policy.requires_approval("any-tool", &json!({})),
        "Manual mode should always require approval"
    );
}

// ---------------------------------------------------------------------------
// ApprovalPolicy: SemiAutonomous — flagged tools
// ---------------------------------------------------------------------------

#[test]
fn test_approval_policy_semi_autonomous_flagged_tools() {
    let policy = ApprovalPolicy {
        mode: AgentMode::SemiAutonomous,
        flagged_tools: vec!["kubectl".to_string(), "aws".to_string()],
        flagged_patterns: vec![],
        default_timeout_secs: 300,
        approvers: vec![],
    };

    assert!(
        policy.requires_approval("kubectl", &json!({})),
        "kubectl is flagged and should require approval"
    );
    assert!(
        policy.requires_approval("aws", &json!({})),
        "aws is flagged and should require approval"
    );
    assert!(
        !policy.requires_approval("git", &json!({})),
        "git is not flagged and should not require approval"
    );
}

// ---------------------------------------------------------------------------
// ApprovalPolicy: SemiAutonomous — flagged patterns
// ---------------------------------------------------------------------------

#[test]
fn test_approval_policy_semi_autonomous_flagged_patterns() {
    let policy = ApprovalPolicy {
        mode: AgentMode::SemiAutonomous,
        flagged_tools: vec![],
        flagged_patterns: vec!["delete".to_string(), "drop".to_string()],
        default_timeout_secs: 300,
        approvers: vec![],
    };

    // Input contains "delete" — should require approval
    assert!(
        policy.requires_approval("shell", &json!({"command": "kubectl delete pod"})),
        "Input containing 'delete' should require approval"
    );

    // Input contains "drop" — should require approval
    assert!(
        policy.requires_approval("shell", &json!({"command": "DROP TABLE users"})),
        "Input containing 'drop' should require approval (case-insensitive)"
    );

    // Input does not match any pattern — should not require approval
    assert!(
        !policy.requires_approval("shell", &json!({"command": "kubectl get pods"})),
        "Input without flagged patterns should not require approval"
    );
}

// ---------------------------------------------------------------------------
// ApprovalPolicy: default
// ---------------------------------------------------------------------------

#[test]
fn test_approval_policy_default() {
    let policy = ApprovalPolicy::default();

    assert!(
        matches!(policy.mode, AgentMode::Autonomous),
        "default mode should be Autonomous"
    );
    assert!(policy.flagged_tools.is_empty(), "default flagged_tools should be empty");
    assert!(
        policy.flagged_patterns.is_empty(),
        "default flagged_patterns should be empty"
    );
    assert_eq!(policy.default_timeout_secs, 300, "default timeout should be 300s");
    assert!(policy.approvers.is_empty(), "default approvers should be empty");
}

// ---------------------------------------------------------------------------
// ApprovalDecision serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_approval_decision_serde_round_trip() {
    let decision = ApprovalDecision {
        request_id: "req-abc".to_string(),
        approved: true,
        approver: "admin@company.com".to_string(),
        reason: Some("All good".to_string()),
        decided_at: Utc::now(),
    };

    let json_str = serde_json::to_string(&decision).expect("serialize should succeed");
    let back: ApprovalDecision = serde_json::from_str(&json_str).expect("deserialize should succeed");

    assert_eq!(back.request_id, decision.request_id);
    assert_eq!(back.approved, decision.approved);
    assert_eq!(back.approver, decision.approver);
    assert_eq!(back.reason, decision.reason);
    // decided_at comparison within 1 second tolerance
    let diff = back.decided_at.signed_duration_since(decision.decided_at).num_milliseconds().abs();
    assert!(diff < 1000, "decided_at should round-trip within 1s tolerance");
}

// ---------------------------------------------------------------------------
// ApprovalStatus serde round-trip for all variants
// ---------------------------------------------------------------------------

#[test]
fn test_approval_status_serde_round_trip_pending() {
    let status = ApprovalStatus::Pending;
    let json_str = serde_json::to_string(&status).expect("serialize Pending");
    let back: ApprovalStatus = serde_json::from_str(&json_str).expect("deserialize Pending");
    assert!(matches!(back, ApprovalStatus::Pending));
}

#[test]
fn test_approval_status_serde_round_trip_approved() {
    let status = ApprovalStatus::Approved {
        approver: "admin".to_string(),
        decided_at: Utc::now(),
    };
    let json_str = serde_json::to_string(&status).expect("serialize Approved");
    let back: ApprovalStatus = serde_json::from_str(&json_str).expect("deserialize Approved");
    match back {
        ApprovalStatus::Approved { approver, .. } => {
            assert_eq!(approver, "admin");
        }
        other => panic!("Expected Approved, got {:?}", other),
    }
}

#[test]
fn test_approval_status_serde_round_trip_denied() {
    let status = ApprovalStatus::Denied {
        approver: "sre-lead".to_string(),
        reason: Some("Too risky".to_string()),
        decided_at: Utc::now(),
    };
    let json_str = serde_json::to_string(&status).expect("serialize Denied");
    let back: ApprovalStatus = serde_json::from_str(&json_str).expect("deserialize Denied");
    match back {
        ApprovalStatus::Denied { approver, reason, .. } => {
            assert_eq!(approver, "sre-lead");
            assert_eq!(reason.as_deref(), Some("Too risky"));
        }
        other => panic!("Expected Denied, got {:?}", other),
    }
}

#[test]
fn test_approval_status_serde_round_trip_timed_out() {
    let status = ApprovalStatus::TimedOut {
        expired_at: Utc::now(),
    };
    let json_str = serde_json::to_string(&status).expect("serialize TimedOut");
    let back: ApprovalStatus = serde_json::from_str(&json_str).expect("deserialize TimedOut");
    assert!(
        matches!(back, ApprovalStatus::TimedOut { .. }),
        "Expected TimedOut variant"
    );
}
