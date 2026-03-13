//! Streaming output module for the ReAct loop.
//!
//! Provides three formatters for emitting ReAct loop events to different consumers:
//! - [`SseEncoder`]: Converts events to `text/event-stream` (SSE) format for HTTP/WebSocket clients
//! - [`TextFormatter`]: Renders labeled `[Plan]` / `[Act]` / `[Observe]` / `[Reflect]` sections for CLI display
//! - [`JsonFormatter`]: Produces single-line NDJSON per event for programmatic consumption (`--json` flag)
//!
//! The [`EventSender`] / [`EventReceiver`] type aliases wrap a `tokio::sync::broadcast` channel
//! so the ReAct loop can fan-out events to multiple subscribers in real time.

// Re-export the canonical ReAct event types from react_loop so consumers can import from here.
pub use crate::executor::react_loop::{ReActEvent, ReActStep, RunResult, ToolAction};

// ---------------------------------------------------------------------------
// Broadcast channel type aliases
// ---------------------------------------------------------------------------

/// Sender side of the broadcast channel used to fan-out [`ReActEvent`]s.
///
/// Create with `tokio::sync::broadcast::channel::<ReActEvent>(capacity)`.
pub type EventSender = tokio::sync::broadcast::Sender<ReActEvent>;

/// Receiver side of the broadcast channel used to consume [`ReActEvent`]s.
pub type EventReceiver = tokio::sync::broadcast::Receiver<ReActEvent>;

// ---------------------------------------------------------------------------
// SseEncoder
// ---------------------------------------------------------------------------

/// Encodes [`ReActEvent`]s to the Server-Sent Events (`text/event-stream`) wire format.
///
/// Each call to [`SseEncoder::encode`] returns one or more SSE messages. Every message
/// is terminated by the mandatory double newline (`\n\n`) required by the SSE specification.
///
/// A `Step` event emits one SSE message per non-empty phase (plan, act, observe, reflect)
/// to allow clients to render incremental progress. `Complete` and `Error` events each
/// produce a single SSE message.
pub struct SseEncoder;

impl SseEncoder {
    /// Encode a [`ReActEvent`] to one or more SSE messages.
    ///
    /// # Format
    /// ```text
    /// event: react_step
    /// data: {"phase":"plan","content":"..."}
    ///
    /// ```
    pub fn encode(event: &ReActEvent) -> String {
        match event {
            ReActEvent::Step(step) => Self::encode_step(step),
            ReActEvent::Complete(result) => Self::encode_complete(result),
            ReActEvent::Error(msg) => Self::encode_error(msg),
            ReActEvent::ApprovalWaiting { request_id, tool_name, description } => {
                let data = serde_json::json!({
                    "request_id": request_id,
                    "tool_name": tool_name,
                    "description": description,
                });
                format!("event: approval_waiting\ndata: {}\n\n", data)
            }
            ReActEvent::ApprovalDecided { request_id, approved } => {
                let data = serde_json::json!({
                    "request_id": request_id,
                    "approved": approved,
                });
                format!("event: approval_decided\ndata: {}\n\n", data)
            }
        }
    }

    fn encode_step(step: &ReActStep) -> String {
        let mut parts = Vec::new();

        // Plan phase — always present
        parts.push(Self::encode_phase("plan", &step.plan, None));

        // Act phase — only when there is a tool call
        if let Some(action) = &step.action {
            let extra = serde_json::json!({
                "tool_name": action.tool_name,
                "input": action.input,
            });
            parts.push(Self::encode_phase(
                "act",
                &format!("Calling tool: {}", action.tool_name),
                Some(&extra),
            ));
        }

        // Observe phase — only when there is an observation
        if !step.observation.is_empty() {
            parts.push(Self::encode_phase("observe", &step.observation, None));
        }

        // Reflect phase — only when there is a reflection
        if !step.reflection.is_empty() {
            parts.push(Self::encode_phase("reflect", &step.reflection, None));
        }

        parts.join("")
    }

    fn encode_phase(phase: &str, content: &str, extra: Option<&serde_json::Value>) -> String {
        let mut data = serde_json::json!({ "phase": phase, "content": content });
        if let Some(extra) = extra {
            if let (Some(obj), Some(ext)) = (data.as_object_mut(), extra.as_object()) {
                obj.extend(ext.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        format!("event: react_step\ndata: {}\n\n", data)
    }

    fn encode_complete(result: &RunResult) -> String {
        let data = serde_json::json!({
            "output": result.output,
            "iterations": result.iterations,
            "tool_calls_count": result.tool_calls.len(),
            "reached_max_iterations": result.reached_max_iterations,
        });
        format!("event: complete\ndata: {}\n\n", data)
    }

    fn encode_error(msg: &str) -> String {
        let data = serde_json::json!({ "message": msg });
        format!("event: error\ndata: {}\n\n", data)
    }

    /// Return only the data payload (JSON string) for an event.
    ///
    /// Used by the gateway SSE handler to set the `data` field of each
    /// `axum::response::sse::Event`.
    pub fn encode_data(event: &ReActEvent) -> String {
        match event {
            ReActEvent::Step(step) => {
                // Return a single JSON object summarising the step
                let json = serde_json::json!({
                    "phase": "step",
                    "plan": step.plan,
                    "action": step.action.as_ref().map(|a| serde_json::json!({
                        "tool_name": a.tool_name,
                        "input": a.input,
                    })),
                    "observation": step.observation,
                });
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            }
            ReActEvent::Complete(result) => {
                let json = serde_json::json!({
                    "output": result.output,
                    "iterations": result.iterations,
                    "reached_max_iterations": result.reached_max_iterations,
                });
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            }
            ReActEvent::Error(msg) => {
                let json = serde_json::json!({ "message": msg });
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            }
            ReActEvent::ApprovalWaiting { request_id, tool_name, description } => {
                let json = serde_json::json!({
                    "request_id": request_id,
                    "tool_name": tool_name,
                    "description": description,
                });
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            }
            ReActEvent::ApprovalDecided { request_id, approved } => {
                let json = serde_json::json!({
                    "request_id": request_id,
                    "approved": approved,
                });
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            }
        }
    }

    /// Return the SSE event name for an event type.
    ///
    /// Used to set the `event:` field in the SSE stream so clients can
    /// distinguish between step, complete, and error events.
    pub fn encode_event_name(event: &ReActEvent) -> String {
        match event {
            ReActEvent::Step(_) => "react_step".to_string(),
            ReActEvent::Complete(_) => "complete".to_string(),
            ReActEvent::Error(_) => "error".to_string(),
            ReActEvent::ApprovalWaiting { .. } => "approval_waiting".to_string(),
            ReActEvent::ApprovalDecided { .. } => "approval_decided".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// TextFormatter
// ---------------------------------------------------------------------------

/// Formats [`ReActEvent`]s as human-readable text for CLI display.
///
/// Each ReAct phase is prefixed with a labelled bracket:
/// - `[Plan]` — in cyan when colours are enabled
/// - `[Act]` — in yellow when colours are enabled
/// - `[Observe]` — in green when colours are enabled
/// - `[Reflect]` — in blue when colours are enabled
///
/// Set `use_colors: false` in tests or when stdout is not a TTY to suppress
/// ANSI escape codes.
pub struct TextFormatter {
    /// Whether to emit ANSI colour codes in the output.
    pub use_colors: bool,
}

// ANSI colour constants
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

impl TextFormatter {
    /// Format a [`ReActEvent`] as labelled human-readable text.
    pub fn format(&self, event: &ReActEvent) -> String {
        match event {
            ReActEvent::Step(step) => self.format_step(step),
            ReActEvent::Complete(result) => self.format_complete(result),
            ReActEvent::Error(msg) => self.format_error(msg),
            ReActEvent::ApprovalWaiting { request_id, tool_name, description } => {
                if self.use_colors {
                    format!(
                        "{YELLOW}[Approval Required]{RESET} Tool '{}' needs approval (request: {})\n  {}\n",
                        tool_name, request_id, description
                    )
                } else {
                    format!(
                        "[Approval Required] Tool '{}' needs approval (request: {})\n  {}\n",
                        tool_name, request_id, description
                    )
                }
            }
            ReActEvent::ApprovalDecided { request_id, approved } => {
                let status = if *approved { "APPROVED" } else { "DENIED" };
                if self.use_colors {
                    let color = if *approved { GREEN } else { RED };
                    format!("{color}[Approval {status}]{RESET} Request: {request_id}\n")
                } else {
                    format!("[Approval {status}] Request: {request_id}\n")
                }
            }
        }
    }

    fn format_step(&self, step: &ReActStep) -> String {
        let mut output = String::new();

        // Plan phase — always present
        output.push_str(&self.label("Plan", CYAN));
        output.push_str(&step.plan);
        output.push('\n');

        // Act phase — only when there is a tool call
        if let Some(action) = &step.action {
            output.push_str(&self.label("Act", YELLOW));
            output.push_str(&format!(
                "Calling tool: {} with {}",
                action.tool_name, action.input
            ));
            output.push('\n');
        }

        // Observe phase — only when there is an observation
        if !step.observation.is_empty() {
            output.push_str(&self.label("Observe", GREEN));
            output.push_str(&step.observation);
            output.push('\n');
        }

        // Reflect phase — only when there is a reflection
        if !step.reflection.is_empty() {
            output.push_str(&self.label("Reflect", BLUE));
            output.push_str(&step.reflection);
            output.push('\n');
        }

        output
    }

    fn format_complete(&self, result: &RunResult) -> String {
        format!("\n--- Result ---\n{}\n", result.output)
    }

    fn format_error(&self, msg: &str) -> String {
        if self.use_colors {
            format!("{RED}[Error]{RESET} {msg}\n")
        } else {
            format!("[Error] {msg}\n")
        }
    }

    /// Render a bracketed label with optional ANSI colour.
    fn label(&self, name: &str, color_code: &str) -> String {
        if self.use_colors {
            format!("{color_code}[{name}]{RESET} ")
        } else {
            format!("[{name}] ")
        }
    }
}

// ---------------------------------------------------------------------------
// JsonFormatter
// ---------------------------------------------------------------------------

/// Formats [`ReActEvent`]s as compact single-line JSON (NDJSON) for programmatic consumers.
///
/// Suitable for the `--json` flag and CI/CD pipelines that parse agent output.
/// Every output line is guaranteed to be valid JSON with an `event_type` field.
pub struct JsonFormatter;

impl JsonFormatter {
    /// Format a [`ReActEvent`] as a single-line JSON object (NDJSON).
    pub fn format(event: &ReActEvent) -> String {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let json = match event {
            ReActEvent::Step(step) => serde_json::json!({
                "event_type": "react_step",
                "timestamp": timestamp,
                "plan": step.plan,
                "action": step.action.as_ref().map(|a| serde_json::json!({
                    "tool_name": a.tool_name,
                    "input": a.input,
                })),
                "observation": step.observation,
                "reflection": step.reflection,
            }),
            ReActEvent::Complete(result) => serde_json::json!({
                "event_type": "complete",
                "timestamp": timestamp,
                "output": result.output,
                "iterations": result.iterations,
                "reached_max_iterations": result.reached_max_iterations,
            }),
            ReActEvent::Error(msg) => serde_json::json!({
                "event_type": "error",
                "timestamp": timestamp,
                "message": msg,
            }),
            ReActEvent::ApprovalWaiting { request_id, tool_name, description } => serde_json::json!({
                "event_type": "approval_waiting",
                "timestamp": timestamp,
                "request_id": request_id,
                "tool_name": tool_name,
                "description": description,
            }),
            ReActEvent::ApprovalDecided { request_id, approved } => serde_json::json!({
                "event_type": "approval_decided",
                "timestamp": timestamp,
                "request_id": request_id,
                "approved": approved,
            }),
        };
        // serde_json compact serialization produces a single line with no embedded newlines
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }
}
