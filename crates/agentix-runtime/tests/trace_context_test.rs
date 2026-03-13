use agentix_core::telemetry::{LogLevel, SpanKind, TraceContext};
use agentix_runtime::TraceCollector;
use std::collections::HashMap;

#[test]
fn test_trace_collector_new() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let collector = TraceCollector::new(ctx);

    assert!(collector.get_spans().is_empty());
    assert!(collector.get_logs().is_empty());
}

#[test]
fn test_trace_collector_record_span() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let collector = TraceCollector::new(ctx);

    let mut span = collector.start_span("test", SpanKind::LlmCall);
    span.complete();
    collector.record_span(span);

    let spans = collector.get_spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "test");
    assert_eq!(spans[0].kind, SpanKind::LlmCall);
}

#[test]
fn test_trace_collector_log() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let trace_id = ctx.trace_id.clone();
    let collector = TraceCollector::new(ctx);

    let mut fields = HashMap::new();
    fields.insert("key".to_string(), serde_json::json!("value"));
    collector.log(LogLevel::Info, "test msg", fields);

    let logs = collector.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "test msg");
    assert_eq!(logs[0].level, LogLevel::Info);
    assert_eq!(logs[0].trace_id, Some(trace_id));
}

#[test]
fn test_trace_collector_multiple_spans() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let trace_id = ctx.trace_id.clone();
    let collector = TraceCollector::new(ctx);

    for (name, kind) in [
        ("run", SpanKind::Run),
        ("llm", SpanKind::LlmCall),
        ("tool", SpanKind::ToolCall),
    ] {
        let mut span = collector.start_span(name, kind);
        span.complete();
        collector.record_span(span);
    }

    let spans = collector.get_spans();
    assert_eq!(spans.len(), 3);
    for span in &spans {
        assert_eq!(span.trace_id, trace_id);
    }
}

#[tokio::test]
async fn test_trace_collector_thread_safety() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let collector = TraceCollector::new(ctx);

    let mut handles = vec![];
    for i in 0..4 {
        let c = collector.clone();
        handles.push(tokio::spawn(async move {
            let mut span = c.start_span(&format!("task_{}", i), SpanKind::ToolCall);
            span.complete();
            c.record_span(span);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(collector.get_spans().len(), 4);
}
