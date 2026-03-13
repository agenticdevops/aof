use agentix_core::coordination::{AgentInbox, DelegationMessage, DelegationResult, DelegationStatus};
use serde_json::json;

#[tokio::test]
async fn test_agent_inbox_send_receive() {
    let inbox = AgentInbox::new(256);
    let msg = DelegationMessage {
        id: "del-001".to_string(),
        from_agent: "coordinator".to_string(),
        task: "analyze replication lag".to_string(),
        payload: json!({"host": "pg-primary"}),
        reply_to: Some("coordinator".to_string()),
    };
    inbox.send(msg.clone()).await.unwrap();
    let received = inbox.receive().await.unwrap();
    assert_eq!(received.id, "del-001");
    assert_eq!(received.from_agent, "coordinator");
    assert_eq!(received.task, "analyze replication lag");
}

#[test]
fn test_delegation_message_fields() {
    let msg = DelegationMessage {
        id: "del-002".to_string(),
        from_agent: "ops-coordinator".to_string(),
        task: "check disk usage".to_string(),
        payload: json!({}),
        reply_to: None,
    };
    assert_eq!(msg.id, "del-002");
    assert!(msg.reply_to.is_none());
}

#[test]
fn test_delegation_result_success() {
    let result = DelegationResult {
        delegation_id: "del-001".to_string(),
        from_agent: "dba-specialist".to_string(),
        output: "Replication lag is 2ms — healthy".to_string(),
        status: DelegationStatus::Success,
        completed_at: chrono::Utc::now(),
    };
    assert!(matches!(result.status, DelegationStatus::Success));
}

#[test]
fn test_delegation_status_variants() {
    let s = DelegationStatus::Success;
    let f = DelegationStatus::Failure;
    assert!(matches!(s, DelegationStatus::Success));
    assert!(matches!(f, DelegationStatus::Failure));
}
