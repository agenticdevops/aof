// TDD RED phase — written before RunRecord / RunStore implementation.

use agentix_runtime::gateway::{RunRecord, RunStore};
use chrono::Utc;

#[test]
fn test_run_record_create() {
    let record = RunRecord {
        id: uuid::Uuid::new_v4().to_string(),
        agent_name: "my-agent".to_string(),
        trigger_source: "cron".to_string(),
        trigger_id: Some("my-agent-cron-0".to_string()),
        started_at: Utc::now(),
        ended_at: None,
        duration_ms: None,
        status: "running".to_string(),
        iterations: None,
        input_summary: Some("check health".to_string()),
        output_summary: None,
        error: None,
    };
    assert!(!record.id.is_empty());
    assert_eq!(record.status, "running");
    assert_eq!(record.trigger_source, "cron");
}

#[test]
fn test_run_store_insert_and_query() {
    let store = RunStore::open(":memory:").expect("open in-memory db");

    let now = Utc::now();
    let r1 = RunRecord {
        id: "run-001".to_string(),
        agent_name: "agent-a".to_string(),
        trigger_source: "webhook".to_string(),
        trigger_id: Some("agent-a-webhook-0".to_string()),
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: "running".to_string(),
        iterations: None,
        input_summary: None,
        output_summary: None,
        error: None,
    };
    let r2 = RunRecord {
        id: "run-002".to_string(),
        agent_name: "agent-b".to_string(),
        trigger_source: "cli".to_string(),
        trigger_id: None,
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: "running".to_string(),
        iterations: None,
        input_summary: None,
        output_summary: None,
        error: None,
    };

    store.insert(&r1).expect("insert r1");
    store.insert(&r2).expect("insert r2");

    let all = store.list(None, 100).expect("list all");
    assert_eq!(all.len(), 2);

    let agent_a = store.list(Some("agent-a"), 100).expect("list agent-a");
    assert_eq!(agent_a.len(), 1);
    assert_eq!(agent_a[0].agent_name, "agent-a");
}

#[test]
fn test_run_store_update_status() {
    let store = RunStore::open(":memory:").expect("open in-memory db");

    let id = "run-003".to_string();
    let record = RunRecord {
        id: id.clone(),
        agent_name: "agent-c".to_string(),
        trigger_source: "github".to_string(),
        trigger_id: Some("agent-c-github-0".to_string()),
        started_at: Utc::now(),
        ended_at: None,
        duration_ms: None,
        status: "running".to_string(),
        iterations: None,
        input_summary: None,
        output_summary: None,
        error: None,
    };
    store.insert(&record).expect("insert record");

    let ended = Utc::now();
    store
        .update_completed(&id, ended, Some("analysis complete"), Some(3))
        .expect("update completed");

    let queried = store.get(&id).expect("get").expect("should exist");
    assert_eq!(queried.status, "completed");
    assert!(queried.ended_at.is_some());
    assert_eq!(queried.output_summary.as_deref(), Some("analysis complete"));
    assert_eq!(queried.iterations, Some(3));
}

#[test]
fn test_run_store_limit() {
    let store = RunStore::open(":memory:").expect("open in-memory db");

    let now = Utc::now();
    for i in 0..25 {
        let record = RunRecord {
            id: format!("run-{:03}", i),
            agent_name: "agent-d".to_string(),
            trigger_source: "cron".to_string(),
            trigger_id: None,
            started_at: now,
            ended_at: None,
            duration_ms: None,
            status: "completed".to_string(),
            iterations: Some(1),
            input_summary: None,
            output_summary: None,
            error: None,
        };
        store.insert(&record).expect("insert");
    }

    let limited = store.list(None, 10).expect("list with limit");
    assert_eq!(limited.len(), 10);
}

#[test]
fn test_run_store_trigger_source() {
    let store = RunStore::open(":memory:").expect("open in-memory db");

    let record = RunRecord {
        id: "run-004".to_string(),
        agent_name: "agent-e".to_string(),
        trigger_source: "slack".to_string(),
        trigger_id: Some("agent-e-slack-0".to_string()),
        started_at: Utc::now(),
        ended_at: None,
        duration_ms: None,
        status: "running".to_string(),
        iterations: None,
        input_summary: Some("analyze this".to_string()),
        output_summary: None,
        error: None,
    };
    store.insert(&record).expect("insert");

    let queried = store.get("run-004").expect("get").expect("should exist");
    assert_eq!(queried.trigger_source, "slack");
    assert_eq!(queried.trigger_id.as_deref(), Some("agent-e-slack-0"));
}
