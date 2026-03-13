use agentix_runtime::{AuditStore, AuditEntry, AuditEventType, AuditOutcome};
use chrono::Utc;
use std::collections::HashMap;

fn make_entry(
    agent_name: &str,
    run_id: Option<&str>,
    event_type: AuditEventType,
    outcome: AuditOutcome,
) -> AuditEntry {
    AuditEntry {
        id: None,
        timestamp: Utc::now(),
        event_type,
        agent_name: agent_name.to_string(),
        run_id: run_id.map(|s| s.to_string()),
        actor: "system".to_string(),
        action: "test action".to_string(),
        outcome,
        details: HashMap::new(),
        trace_id: None,
    }
}

#[test]
fn test_log_and_get_agent_audit() {
    let store = AuditStore::open(":memory:").unwrap();

    // Log 3 entries for agent "dba-optimizer"
    let mut e1 = make_entry("dba-optimizer", Some("run-1"), AuditEventType::AgentStart, AuditOutcome::Success);
    e1.action = "Starting agent run".to_string();
    store.log_event(e1).unwrap();

    let mut e2 = make_entry("dba-optimizer", Some("run-1"), AuditEventType::ToolCall, AuditOutcome::Success);
    e2.action = "Tool call: kubectl".to_string();
    store.log_event(e2).unwrap();

    let mut e3 = make_entry("dba-optimizer", Some("run-1"), AuditEventType::AgentComplete, AuditOutcome::Success);
    e3.action = "Agent run complete".to_string();
    store.log_event(e3).unwrap();

    let entries = store.get_agent_audit("dba-optimizer", 10).unwrap();
    assert_eq!(entries.len(), 3);
    // Most recent first (DESC)
    assert_eq!(entries[0].event_type, AuditEventType::AgentComplete);
    assert_eq!(entries[2].event_type, AuditEventType::AgentStart);
}

#[test]
fn test_get_run_audit() {
    let store = AuditStore::open(":memory:").unwrap();

    store.log_event(make_entry("agent-a", Some("run-123"), AuditEventType::AgentStart, AuditOutcome::Success)).unwrap();
    store.log_event(make_entry("agent-a", Some("run-123"), AuditEventType::LlmCall, AuditOutcome::Success)).unwrap();
    store.log_event(make_entry("agent-a", Some("run-123"), AuditEventType::ToolCall, AuditOutcome::Success)).unwrap();
    store.log_event(make_entry("agent-a", Some("run-123"), AuditEventType::AgentComplete, AuditOutcome::Success)).unwrap();

    let entries = store.get_run_audit("run-123").unwrap();
    assert_eq!(entries.len(), 4);
    // Chronological order (ASC)
    assert_eq!(entries[0].event_type, AuditEventType::AgentStart);
    assert_eq!(entries[3].event_type, AuditEventType::AgentComplete);
}

#[test]
fn test_get_security_events() {
    let store = AuditStore::open(":memory:").unwrap();

    // 1 SecurityViolation
    store.log_event(make_entry("agent-a", Some("run-1"), AuditEventType::SecurityViolation, AuditOutcome::Blocked("SSRF: private IP".to_string()))).unwrap();
    // 1 ToolCall with Denied outcome
    store.log_event(make_entry("agent-a", Some("run-1"), AuditEventType::ToolCall, AuditOutcome::Denied("capability not granted".to_string()))).unwrap();
    // 1 normal ToolCall with Success
    store.log_event(make_entry("agent-a", Some("run-1"), AuditEventType::ToolCall, AuditOutcome::Success)).unwrap();

    let events = store.get_security_events(10).unwrap();
    assert_eq!(events.len(), 2, "Should return SecurityViolation + Denied, not Success");
}

#[test]
fn test_audit_outcome_round_trip() {
    let store = AuditStore::open(":memory:").unwrap();

    store.log_event(make_entry("agent-a", None, AuditEventType::ToolCall, AuditOutcome::Success)).unwrap();
    store.log_event(make_entry("agent-a", None, AuditEventType::AgentError, AuditOutcome::Failure("timeout".to_string()))).unwrap();
    store.log_event(make_entry("agent-a", None, AuditEventType::ToolCall, AuditOutcome::Denied("capability not granted".to_string()))).unwrap();
    store.log_event(make_entry("agent-a", None, AuditEventType::SecurityViolation, AuditOutcome::Blocked("SSRF: private IP".to_string()))).unwrap();

    let entries = store.get_agent_audit("agent-a", 10).unwrap();
    assert_eq!(entries.len(), 4);

    // Check each outcome type round-tripped correctly
    let blocked = entries.iter().find(|e| e.event_type == AuditEventType::SecurityViolation).unwrap();
    assert_eq!(blocked.outcome, AuditOutcome::Blocked("SSRF: private IP".to_string()));

    let denied = entries.iter().find(|e| matches!(&e.outcome, AuditOutcome::Denied(_))).unwrap();
    assert_eq!(denied.outcome, AuditOutcome::Denied("capability not granted".to_string()));

    let failure = entries.iter().find(|e| e.event_type == AuditEventType::AgentError).unwrap();
    assert_eq!(failure.outcome, AuditOutcome::Failure("timeout".to_string()));
}

#[test]
fn test_audit_details_round_trip() {
    let store = AuditStore::open(":memory:").unwrap();

    let mut entry = make_entry("agent-a", None, AuditEventType::ToolCall, AuditOutcome::Success);
    entry.details.insert("tool_name".to_string(), serde_json::json!("kubectl"));
    entry.details.insert("command".to_string(), serde_json::json!("get pods"));
    entry.details.insert("namespace".to_string(), serde_json::json!("production"));
    store.log_event(entry).unwrap();

    let entries = store.get_agent_audit("agent-a", 1).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].details["tool_name"], serde_json::json!("kubectl"));
    assert_eq!(entries[0].details["command"], serde_json::json!("get pods"));
    assert_eq!(entries[0].details["namespace"], serde_json::json!("production"));
}

#[test]
fn test_audit_limit() {
    let store = AuditStore::open(":memory:").unwrap();

    for i in 0..20 {
        let mut entry = make_entry("agent-a", None, AuditEventType::ToolCall, AuditOutcome::Success);
        entry.action = format!("Action {}", i);
        store.log_event(entry).unwrap();
    }

    let entries = store.get_agent_audit("agent-a", 5).unwrap();
    assert_eq!(entries.len(), 5);
}
