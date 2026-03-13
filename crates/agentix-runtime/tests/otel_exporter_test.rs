use agentix_core::telemetry::{SpanKind, SpanStatus, TraceContext};
use agentix_runtime::OtelExporter;

fn make_span(
    ctx: &TraceContext,
    name: &str,
    kind: SpanKind,
) -> agentix_core::telemetry::SpanRecord {
    let mut span = ctx.child_span(name, kind);
    span.complete();
    span
}

#[test]
fn test_export_spans_format() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let root = make_span(&ctx, "agent_run", SpanKind::Run);
    let child = make_span(&ctx, "llm_call", SpanKind::LlmCall);

    let result = OtelExporter::export_spans("test-svc", &[root, child]);

    let resource_spans = result["resourceSpans"].as_array().unwrap();
    assert_eq!(resource_spans.len(), 1);

    // Check service name
    let resource = &resource_spans[0]["resource"];
    let attrs = resource["attributes"].as_array().unwrap();
    assert_eq!(attrs[0]["key"], "service.name");
    assert_eq!(attrs[0]["value"]["stringValue"], "test-svc");

    // Check spans count
    let scope_spans = &resource_spans[0]["scopeSpans"].as_array().unwrap();
    let spans = scope_spans[0]["spans"].as_array().unwrap();
    assert_eq!(spans.len(), 2);
}

#[test]
fn test_export_spans_attributes() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let span = ctx
        .child_span("llm_call", SpanKind::LlmCall)
        .with_attribute("model", "anthropic/claude-sonnet-4-6");
    let mut span = span;
    span.complete();

    let result = OtelExporter::export_spans("test-svc", &[span]);
    let spans = result["resourceSpans"][0]["scopeSpans"][0]["spans"]
        .as_array()
        .unwrap();
    let attrs = spans[0]["attributes"].as_array().unwrap();

    let model_attr = attrs.iter().find(|a| a["key"] == "model");
    assert!(
        model_attr.is_some(),
        "Should have model attribute"
    );
    assert_eq!(
        model_attr.unwrap()["value"]["stringValue"],
        "anthropic/claude-sonnet-4-6"
    );
}

#[test]
fn test_export_spans_status_mapping() {
    let ctx = TraceContext::new_root("test-agent", "run-123");

    let mut ok_span = ctx.child_span("ok_span", SpanKind::Run);
    ok_span.complete();

    let mut err_span = ctx.child_span("err_span", SpanKind::Run);
    err_span.complete_with_error("timeout");

    let result = OtelExporter::export_spans("test-svc", &[ok_span, err_span]);
    let spans = result["resourceSpans"][0]["scopeSpans"][0]["spans"]
        .as_array()
        .unwrap();

    assert_eq!(spans[0]["status"]["code"], 1); // OK
    assert_eq!(spans[1]["status"]["code"], 2); // ERROR
    assert_eq!(spans[1]["status"]["message"], "timeout");
}

#[test]
fn test_export_spans_empty_input() {
    let result = OtelExporter::export_spans("test-svc", &[]);
    let resource_spans = result["resourceSpans"].as_array().unwrap();
    let spans = resource_spans[0]["scopeSpans"][0]["spans"]
        .as_array()
        .unwrap();
    assert!(spans.is_empty());
}

#[test]
fn test_export_spans_parent_linking() {
    let ctx = TraceContext::new_root("test-agent", "run-123");
    let parent = make_span(&ctx, "parent", SpanKind::Run);
    let parent_span_id = parent.span_id.clone();

    // Create child with explicit parent
    let mut child = ctx.child_span("child", SpanKind::Iteration);
    child.parent_span_id = Some(parent_span_id.clone());
    child.complete();

    let result = OtelExporter::export_spans("test-svc", &[parent, child]);
    let spans = result["resourceSpans"][0]["scopeSpans"][0]["spans"]
        .as_array()
        .unwrap();

    // The parent's spanId should match the child's parentSpanId
    let parent_otlp_id = spans[0]["spanId"].as_str().unwrap();
    let child_parent_id = spans[1]["parentSpanId"].as_str().unwrap();

    // Both should be derived from the same UUID (first 16 hex chars)
    assert_eq!(parent_otlp_id, child_parent_id);
}
