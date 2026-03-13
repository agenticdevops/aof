use agentix_core::telemetry::{
    LogLevel, SpanKind, SpanStatus, StructuredLogEntry, TraceContext,
};
use agentix_runtime::TraceStore;
use std::collections::HashMap;

fn make_store() -> TraceStore {
    TraceStore::open(":memory:").expect("in-memory TraceStore")
}

#[test]
fn test_insert_and_get_spans() {
    let store = make_store();
    let ctx = TraceContext::new_root("test-agent", "run-1");

    let mut root = ctx.child_span("agent_run", SpanKind::Run);
    root.complete();

    let mut child1 = ctx.child_span("iteration_1", SpanKind::Iteration);
    // Ensure ordering
    std::thread::sleep(std::time::Duration::from_millis(1));
    child1.complete();

    let mut child2 = ctx.child_span("llm_call", SpanKind::LlmCall);
    std::thread::sleep(std::time::Duration::from_millis(1));
    child2.complete();

    store
        .insert_spans(&[root, child1, child2], "test-agent", "run-1")
        .unwrap();

    let spans = store.get_run_trace("run-1").unwrap();
    assert_eq!(spans.len(), 3);

    // Check parent linking is preserved
    assert!(spans[0].parent_span_id.is_some());
    assert_eq!(spans[0].parent_span_id.as_deref(), Some(ctx.span_id.as_str()));
}

#[test]
fn test_insert_and_get_logs() {
    let store = make_store();

    let logs = vec![
        StructuredLogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            message: "Run started".to_string(),
            trace_id: Some("trace-1".to_string()),
            span_id: Some("span-1".to_string()),
            agent: Some("test-agent".to_string()),
            run_id: Some("run-1".to_string()),
            fields: HashMap::new(),
        },
        StructuredLogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Warn,
            message: "Budget low".to_string(),
            trace_id: Some("trace-1".to_string()),
            span_id: Some("span-2".to_string()),
            agent: Some("test-agent".to_string()),
            run_id: Some("run-1".to_string()),
            fields: HashMap::new(),
        },
    ];

    store.insert_logs(&logs).unwrap();
    let result = store.get_run_logs("run-1").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].message, "Run started");
    assert_eq!(result[0].level, LogLevel::Info);
    assert_eq!(result[1].message, "Budget low");
    assert_eq!(result[1].level, LogLevel::Warn);
}

#[test]
fn test_get_trace_by_trace_id() {
    let store = make_store();

    let ctx1 = TraceContext::new_root("agent-1", "run-1");
    let ctx2 = TraceContext::new_root("agent-2", "run-2");

    let mut span1 = ctx1.child_span("run", SpanKind::Run);
    span1.complete();
    let mut span2 = ctx2.child_span("run", SpanKind::Run);
    span2.complete();

    store.insert_spans(&[span1], "agent-1", "run-1").unwrap();
    store.insert_spans(&[span2], "agent-2", "run-2").unwrap();

    let result = store.get_trace(&ctx1.trace_id).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].trace_id, ctx1.trace_id);
}

#[test]
fn test_span_status_round_trip() {
    let store = make_store();
    let ctx = TraceContext::new_root("test-agent", "run-1");

    let mut span = ctx.child_span("failing", SpanKind::ToolCall);
    span.complete_with_error("timeout");

    store.insert_spans(&[span], "test-agent", "run-1").unwrap();

    let result = store.get_run_trace("run-1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, SpanStatus::Error("timeout".to_string()));
}

#[test]
fn test_span_attributes_round_trip() {
    let store = make_store();
    let ctx = TraceContext::new_root("test-agent", "run-1");

    let mut span = ctx
        .child_span("llm_call", SpanKind::LlmCall)
        .with_attribute("model", "anthropic/claude-sonnet-4-6")
        .with_attribute("input_tokens", "500");
    span.complete();

    store.insert_spans(&[span], "test-agent", "run-1").unwrap();

    let result = store.get_run_trace("run-1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].attributes.get("model").unwrap(),
        "anthropic/claude-sonnet-4-6"
    );
    assert_eq!(result[0].attributes.get("input_tokens").unwrap(), "500");
}
