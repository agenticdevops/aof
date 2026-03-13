use agentix_core::telemetry::{
    LogLevel, SpanKind, SpanRecord, SpanStatus, StructuredLogEntry, TraceContext,
};
use std::collections::HashMap;

#[test]
fn test_trace_context_new_root() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    assert!(!ctx.trace_id.is_empty(), "trace_id should be non-empty");
    assert!(!ctx.span_id.is_empty(), "span_id should be non-empty");
    assert!(ctx.parent_span_id.is_none(), "parent_span_id should be None for root");
    assert_eq!(ctx.agent_name, "test-agent");
    assert_eq!(ctx.run_id, "run-123");
}

#[test]
fn test_trace_context_child_span() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let child = ctx.child_span("llm-call", SpanKind::LlmCall);

    assert_eq!(child.parent_span_id, Some(ctx.span_id.clone()));
    assert_eq!(child.trace_id, ctx.trace_id);
    assert_eq!(child.name, "llm-call");
    assert_eq!(child.kind, SpanKind::LlmCall);
    assert!(child.end_time.is_none());
    assert_eq!(child.status, SpanStatus::Ok);
}

#[test]
fn test_span_record_complete() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let mut span = ctx.child_span("test-span", SpanKind::Run);
    span.complete();

    assert!(span.end_time.is_some(), "end_time should be set after complete()");
    assert!(span.duration_ms.is_some(), "duration_ms should be set after complete()");
    assert_eq!(span.status, SpanStatus::Ok);
}

#[test]
fn test_span_record_complete_with_error() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let mut span = ctx.child_span("test-span", SpanKind::Run);
    span.complete_with_error("timeout");

    assert!(span.end_time.is_some());
    assert!(span.duration_ms.is_some());
    assert_eq!(span.status, SpanStatus::Error("timeout".to_string()));
}

#[test]
fn test_span_record_attributes() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let span = ctx
        .child_span("test-span", SpanKind::LlmCall)
        .with_attribute("model", "anthropic/claude-sonnet-4-6")
        .with_attribute("input_tokens", "1500");

    assert_eq!(span.attributes.get("model").unwrap(), "anthropic/claude-sonnet-4-6");
    assert_eq!(span.attributes.get("input_tokens").unwrap(), "1500");
}

#[test]
fn test_span_kind_all_variants() {
    let variants = vec![
        (SpanKind::Run, "run"),
        (SpanKind::Iteration, "iteration"),
        (SpanKind::LlmCall, "llm_call"),
        (SpanKind::ToolCall, "tool_call"),
        (SpanKind::Research, "research"),
        (SpanKind::MemoryRecall, "memory_recall"),
    ];

    for (kind, expected) in variants {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, format!("\"{}\"", expected), "SpanKind::{:?} should serialize to {}", kind, expected);
    }
}

#[test]
fn test_structured_log_entry_construction() {
    let mut fields = HashMap::new();
    fields.insert("iterations".to_string(), serde_json::json!(5));

    let entry = StructuredLogEntry {
        timestamp: chrono::Utc::now(),
        level: LogLevel::Info,
        message: "Agent run started".to_string(),
        trace_id: Some("abc".to_string()),
        span_id: Some("def".to_string()),
        agent: Some("test-agent".to_string()),
        run_id: Some("run-123".to_string()),
        fields: fields.clone(),
    };

    // Round-trip through JSON
    let json = serde_json::to_string(&entry).unwrap();
    let deser: StructuredLogEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.level, LogLevel::Info);
    assert_eq!(deser.message, "Agent run started");
    assert_eq!(deser.trace_id, Some("abc".to_string()));
    assert_eq!(deser.span_id, Some("def".to_string()));
    assert_eq!(deser.agent, Some("test-agent".to_string()));
    assert_eq!(deser.run_id, Some("run-123".to_string()));
    assert_eq!(deser.fields.get("iterations").unwrap(), &serde_json::json!(5));
}

#[test]
fn test_structured_log_entry_json_format() {
    let entry = StructuredLogEntry {
        timestamp: chrono::Utc::now(),
        level: LogLevel::Info,
        message: "Agent run started".to_string(),
        trace_id: Some("abc".to_string()),
        span_id: Some("def".to_string()),
        agent: Some("test-agent".to_string()),
        run_id: Some("run-123".to_string()),
        fields: HashMap::new(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    // Must be a single line
    assert!(!json.contains('\n'), "JSON output must be a single line");
    // Must contain required keys
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"level\""));
    assert!(json.contains("\"message\""));
    assert!(json.contains("\"trace_id\""));
    assert!(json.contains("\"span_id\""));
}

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Error > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Debug);
    assert!(LogLevel::Debug > LogLevel::Trace);
}
