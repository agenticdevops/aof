//! TDD tests for the streaming module.
//!
//! Tests cover:
//! - SseEncoder: correct SSE wire format with event:/data: lines and double-newline termination
//! - TextFormatter: labelled human-readable output with optional ANSI colour codes
//! - JsonFormatter: single-line valid JSON with `event_type` field
//! - EventSender/EventReceiver: broadcast channel ordering

use agentix_runtime::streaming::{
    EventReceiver, EventSender, JsonFormatter, ReActEvent, ReActStep, RunResult, SseEncoder,
    TextFormatter, ToolAction,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn plan_step(plan: &str) -> ReActEvent {
    ReActEvent::Step(ReActStep {
        plan: plan.to_string(),
        action: None,
        observation: String::new(),
        reflection: String::new(),
    })
}

fn act_step(tool_name: &str) -> ReActEvent {
    ReActEvent::Step(ReActStep {
        plan: "Calling a tool".to_string(),
        action: Some(ToolAction {
            tool_name: tool_name.to_string(),
            input: json!({}),
        }),
        observation: String::new(),
        reflection: String::new(),
    })
}

fn observe_step(observation: &str) -> ReActEvent {
    ReActEvent::Step(ReActStep {
        plan: "Analyzed result".to_string(),
        action: None,
        observation: observation.to_string(),
        reflection: String::new(),
    })
}

fn complete_event() -> ReActEvent {
    ReActEvent::Complete(RunResult {
        output: "Done".to_string(),
        iterations: 3,
        tool_calls: vec![],
        reached_max_iterations: false,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_usd: 0.0,
        stopped_reason: None,
    })
}

// ---------------------------------------------------------------------------
// SSE format tests
// ---------------------------------------------------------------------------

#[test]
fn test_sse_encode_plan_step() {
    let event = plan_step("Analyzing the query");
    let encoded = SseEncoder::encode(&event);
    assert!(
        encoded.contains("event: react_step\n"),
        "SSE must contain event: react_step line; got: {encoded}"
    );
    assert!(
        encoded.contains("data:"),
        "SSE must contain data: line; got: {encoded}"
    );
    assert!(
        encoded.contains(r#""phase":"plan""#),
        "SSE data must contain phase:plan; got: {encoded}"
    );
    assert!(
        encoded.contains("Analyzing the query"),
        "SSE data must contain content text; got: {encoded}"
    );
}

#[test]
fn test_sse_encode_act_step() {
    let event = act_step("kubectl");
    let encoded = SseEncoder::encode(&event);
    assert!(
        encoded.contains("event: react_step\n"),
        "SSE must contain react_step event; got: {encoded}"
    );
    assert!(
        encoded.contains(r#""phase":"act""#),
        "SSE data must contain phase:act; got: {encoded}"
    );
    assert!(
        encoded.contains("kubectl"),
        "SSE data must contain tool name; got: {encoded}"
    );
}

#[test]
fn test_sse_encode_observe_step() {
    let event = observe_step("Pods are running");
    let encoded = SseEncoder::encode(&event);
    assert!(
        encoded.contains(r#""phase":"observe""#),
        "SSE data must contain phase:observe; got: {encoded}"
    );
    assert!(
        encoded.contains("Pods are running"),
        "SSE data must contain observation text; got: {encoded}"
    );
}

#[test]
fn test_sse_encode_complete() {
    let event = complete_event();
    let encoded = SseEncoder::encode(&event);
    assert!(
        encoded.contains("event: complete\n"),
        "SSE must contain event: complete line; got: {encoded}"
    );
    assert!(
        encoded.contains(r#""output":"Done""#),
        "SSE complete data must contain output; got: {encoded}"
    );
    assert!(
        encoded.contains(r#""iterations":3"#),
        "SSE complete data must contain iterations; got: {encoded}"
    );
}

#[test]
fn test_sse_encode_error() {
    let event = ReActEvent::Error("timeout exceeded".to_string());
    let encoded = SseEncoder::encode(&event);
    assert!(
        encoded.contains("event: error\n"),
        "SSE must contain event: error line; got: {encoded}"
    );
    assert!(
        encoded.contains("timeout exceeded"),
        "SSE error data must contain message; got: {encoded}"
    );
    assert!(
        encoded.contains(r#""message""#),
        "SSE error data must have message field; got: {encoded}"
    );
}

#[test]
fn test_sse_format_double_newline() {
    // Every SSE message MUST end with \n\n per the SSE specification.
    let events = vec![
        plan_step("Test plan"),
        act_step("bash"),
        observe_step("result"),
        complete_event(),
        ReActEvent::Error("oops".to_string()),
    ];
    for event in &events {
        let encoded = SseEncoder::encode(event);
        assert!(
            encoded.ends_with("\n\n"),
            "SSE output must end with \\n\\n; got: {encoded:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Text format tests (CLI display)
// ---------------------------------------------------------------------------

#[test]
fn test_text_format_plan() {
    let event = plan_step("My plan");
    let fmt = TextFormatter { use_colors: false };
    let output = fmt.format(&event);
    assert!(
        output.contains("[Plan]"),
        "Text output must contain [Plan] label; got: {output}"
    );
    assert!(
        output.contains("My plan"),
        "Text output must contain plan text; got: {output}"
    );
}

#[test]
fn test_text_format_act_with_tool() {
    let event = act_step("kubectl");
    let fmt = TextFormatter { use_colors: false };
    let output = fmt.format(&event);
    assert!(
        output.contains("[Act]"),
        "Text output must contain [Act] label; got: {output}"
    );
    assert!(
        output.contains("kubectl"),
        "Text output must contain tool name; got: {output}"
    );
}

#[test]
fn test_text_format_observe() {
    let event = observe_step("Result text");
    let fmt = TextFormatter { use_colors: false };
    let output = fmt.format(&event);
    assert!(
        output.contains("[Observe]"),
        "Text output must contain [Observe] label; got: {output}"
    );
    assert!(
        output.contains("Result text"),
        "Text output must contain observation; got: {output}"
    );
}

#[test]
fn test_text_format_complete() {
    let event = complete_event();
    let fmt = TextFormatter { use_colors: false };
    let output = fmt.format(&event);
    // Must mention the final output text and some separator / heading
    assert!(
        output.contains("Done"),
        "Text output must contain the result text; got: {output}"
    );
    // Must have some visual separator (--- or Result or similar)
    let has_separator = output.contains("---") || output.contains("Result");
    assert!(
        has_separator,
        "Text output must contain a separator or Result heading; got: {output}"
    );
}

#[test]
fn test_text_format_no_color_flag() {
    let fmt = TextFormatter { use_colors: false };
    let events = vec![
        plan_step("step"),
        act_step("tool"),
        observe_step("obs"),
        complete_event(),
        ReActEvent::Error("err".to_string()),
    ];
    for event in &events {
        let output = fmt.format(event);
        assert!(
            !output.contains("\x1b["),
            "No ANSI escape codes when use_colors=false; got: {output:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// JSON format tests
// ---------------------------------------------------------------------------

#[test]
fn test_json_format_step_is_valid_json() {
    let event = plan_step("thinking...");
    let output = JsonFormatter::format(&event);
    // Must be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("JSON output must be parseable JSON");
    // Compact format: must not have embedded newlines (single-line NDJSON)
    assert!(
        !output.contains('\n'),
        "NDJSON output must be a single line; got: {output:?}"
    );
    let _ = parsed; // silence unused warning
}

#[test]
fn test_json_format_has_event_type() {
    let events = vec![
        plan_step("plan"),
        complete_event(),
        ReActEvent::Error("err".to_string()),
    ];
    for event in &events {
        let output = JsonFormatter::format(event);
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("JSON output must parse");
        assert!(
            parsed.get("event_type").is_some(),
            "JSON output must have event_type field; got: {output}"
        );
    }
}

// ---------------------------------------------------------------------------
// Channel streaming test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_event_channel_receives_in_order() {
    let (tx, mut rx): (EventSender, EventReceiver) =
        tokio::sync::broadcast::channel(16);

    let events = vec![
        plan_step("step 1"),
        act_step("kubectl"),
        observe_step("pods running"),
    ];

    // Send all events
    for event in &events {
        tx.send(event.clone()).expect("send must succeed");
    }

    // Receive and check order
    for expected in &events {
        let received = rx.recv().await.expect("must receive event");
        assert_eq!(
            &received, expected,
            "events must arrive in send order"
        );
    }
}
