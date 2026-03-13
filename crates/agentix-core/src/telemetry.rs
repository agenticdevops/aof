//! Telemetry types for OpenAgentiX observability.
//!
//! Provides OpenTelemetry-compatible trace contexts, span records, structured
//! log entries, and supporting enums for the agent execution pipeline.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Trace context that propagates through agent execution.
///
/// Every agent run starts with a root `TraceContext` that carries a unique
/// `trace_id`. Child spans inherit the `trace_id` and reference their parent
/// via `parent_span_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Unique identifier for the entire trace (one per agent run).
    pub trace_id: String,
    /// Current span identifier.
    pub span_id: String,
    /// Parent span identifier (None for root spans).
    pub parent_span_id: Option<String>,
    /// Name of the agent that owns this trace.
    pub agent_name: String,
    /// Run identifier this trace belongs to.
    pub run_id: String,
}

impl TraceContext {
    /// Create a new root trace context for an agent run.
    ///
    /// Generates unique `trace_id` and `span_id` via UUID v4.
    pub fn new_root(agent_name: &str, run_id: &str) -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            agent_name: agent_name.to_string(),
            run_id: run_id.to_string(),
        }
    }

    /// Create a child span under this trace context.
    ///
    /// The child span inherits `trace_id` and sets `parent_span_id` to the
    /// current context's `span_id`.
    pub fn child_span(&self, name: &str, kind: SpanKind) -> SpanRecord {
        SpanRecord {
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            trace_id: self.trace_id.clone(),
            name: name.to_string(),
            kind,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::Ok,
            attributes: HashMap::new(),
        }
    }
}

/// A single span record representing one phase of agent execution.
///
/// Spans form a tree: each span has a `parent_span_id` (except the root).
/// The lifecycle is: create → add attributes → complete/complete_with_error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanRecord {
    /// Unique span identifier.
    pub span_id: String,
    /// Parent span identifier (None for root spans).
    pub parent_span_id: Option<String>,
    /// Trace identifier (shared across all spans in one run).
    pub trace_id: String,
    /// Human-readable span name (e.g., "llm_call", "tool_call_kubectl").
    pub name: String,
    /// What kind of execution phase this span represents.
    pub kind: SpanKind,
    /// When this span started.
    pub start_time: DateTime<Utc>,
    /// When this span ended (None if still in progress).
    pub end_time: Option<DateTime<Utc>>,
    /// Duration in milliseconds (computed on complete).
    pub duration_ms: Option<u64>,
    /// Span outcome status.
    pub status: SpanStatus,
    /// Key-value attributes attached to this span.
    pub attributes: HashMap<String, String>,
}

impl SpanRecord {
    /// Mark this span as successfully completed.
    ///
    /// Sets `end_time` to now and computes `duration_ms`.
    pub fn complete(&mut self) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.duration_ms = Some(
            (now - self.start_time)
                .num_milliseconds()
                .max(0) as u64,
        );
    }

    /// Mark this span as completed with an error.
    ///
    /// Sets `end_time`, computes `duration_ms`, and records the error message.
    pub fn complete_with_error(&mut self, error: &str) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.duration_ms = Some(
            (now - self.start_time)
                .num_milliseconds()
                .max(0) as u64,
        );
        self.status = SpanStatus::Error(error.to_string());
    }

    /// Add an attribute to this span (builder pattern).
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }
}

/// The kind of execution phase a span represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    /// Root span for an entire agent run.
    Run,
    /// One iteration of the ReAct loop.
    Iteration,
    /// A call to an LLM provider.
    LlmCall,
    /// Execution of a single tool.
    ToolCall,
    /// A research phase (multi-step information gathering).
    Research,
    /// A vector memory recall/search operation.
    MemoryRecall,
}

/// Span completion status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    /// Span completed successfully.
    Ok,
    /// Span completed with an error.
    Error(String),
}

/// A single structured log entry with correlation IDs.
///
/// Serializes to single-line JSON for log aggregation pipelines.
/// Correlation IDs (`trace_id`, `span_id`) link logs to trace spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    /// When this log entry was created.
    pub timestamp: DateTime<Utc>,
    /// Log severity level.
    pub level: LogLevel,
    /// Human-readable log message.
    pub message: String,
    /// Trace ID for correlation (links to TraceContext).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Span ID for correlation (links to SpanRecord).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// Agent name that produced this log entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Run ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Additional structured fields.
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log severity levels, ordered from least to most severe.
///
/// Supports `PartialOrd`/`Ord` — `Error > Warn > Info > Debug > Trace`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Finest-grained diagnostic information.
    Trace,
    /// Diagnostic information for developers.
    Debug,
    /// Normal operational messages.
    Info,
    /// Potential issues that deserve attention.
    Warn,
    /// Error conditions that should be investigated.
    Error,
}
