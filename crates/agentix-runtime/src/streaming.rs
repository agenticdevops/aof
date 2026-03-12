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
// Formatter stubs (filled in during GREEN phase)
// ---------------------------------------------------------------------------

/// Encodes [`ReActEvent`]s to the Server-Sent Events (`text/event-stream`) wire format.
///
/// Each call to [`SseEncoder::encode`] returns one or more SSE messages terminated
/// by the mandatory double newline (`\n\n`) per the SSE specification.
pub struct SseEncoder;

/// Formats [`ReActEvent`]s as human-readable text for CLI display.
///
/// When `use_colors` is `true`, ANSI escape codes are added for terminal colour.
/// Set `use_colors: false` in tests or when stdout is not a TTY.
pub struct TextFormatter {
    /// Whether to emit ANSI colour codes in the output.
    pub use_colors: bool,
}

/// Formats [`ReActEvent`]s as compact single-line JSON (NDJSON) for programmatic consumers.
///
/// Suitable for the `--json` flag and CI/CD pipelines that parse agent output.
pub struct JsonFormatter;

impl SseEncoder {
    /// Encode a [`ReActEvent`] to one or more SSE messages.
    pub fn encode(_event: &ReActEvent) -> String {
        todo!("implement SseEncoder::encode")
    }
}

impl TextFormatter {
    /// Format a [`ReActEvent`] as labelled human-readable text.
    pub fn format(&self, _event: &ReActEvent) -> String {
        todo!("implement TextFormatter::format")
    }
}

impl JsonFormatter {
    /// Format a [`ReActEvent`] as a single-line JSON object (NDJSON).
    pub fn format(_event: &ReActEvent) -> String {
        todo!("implement JsonFormatter::format")
    }
}
