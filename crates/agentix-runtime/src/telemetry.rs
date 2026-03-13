//! Runtime telemetry collector for OpenAgentiX agent runs.
//!
//! The `TraceCollector` aggregates trace spans and structured log entries
//! produced during a single agent run. It is thread-safe via `Arc<Mutex<_>>`
//! so it can be shared across async tasks within one run.

use std::collections::HashMap;
use std::sync::Arc;

use agentix_core::telemetry::{
    LogLevel, SpanKind, SpanRecord, SpanStatus, StructuredLogEntry, TraceContext,
};
use chrono::Utc;
use parking_lot::Mutex;

/// Collects trace spans and structured log entries during an agent run.
///
/// Thread-safe via `Arc<Mutex<_>>` so it can be shared with async tasks.
/// The collector holds all spans and logs for one agent run, identified by
/// a single TraceContext (one trace_id).
#[derive(Debug, Clone)]
pub struct TraceCollector {
    inner: Arc<Mutex<TraceCollectorInner>>,
}

#[derive(Debug)]
struct TraceCollectorInner {
    trace_context: TraceContext,
    spans: Vec<SpanRecord>,
    logs: Vec<StructuredLogEntry>,
}

impl TraceCollector {
    /// Create a new collector for the given trace context.
    pub fn new(trace_context: TraceContext) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TraceCollectorInner {
                trace_context,
                spans: Vec::new(),
                logs: Vec::new(),
            })),
        }
    }

    /// Record a completed span.
    pub fn record_span(&self, span: SpanRecord) {
        self.inner.lock().spans.push(span);
    }

    /// Log a structured entry correlated to the current trace context.
    pub fn log(
        &self,
        level: LogLevel,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) {
        let inner = self.inner.lock();
        let entry = StructuredLogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            trace_id: Some(inner.trace_context.trace_id.clone()),
            span_id: Some(inner.trace_context.span_id.clone()),
            agent: Some(inner.trace_context.agent_name.clone()),
            run_id: Some(inner.trace_context.run_id.clone()),
            fields,
        };
        drop(inner);
        self.inner.lock().logs.push(entry);
    }

    /// Return all recorded spans (cloned).
    pub fn get_spans(&self) -> Vec<SpanRecord> {
        self.inner.lock().spans.clone()
    }

    /// Return all recorded log entries (cloned).
    pub fn get_logs(&self) -> Vec<StructuredLogEntry> {
        self.inner.lock().logs.clone()
    }

    /// Return a clone of the trace context.
    pub fn trace_context(&self) -> TraceContext {
        self.inner.lock().trace_context.clone()
    }

    /// Start a new child span under the trace context.
    pub fn start_span(&self, name: &str, kind: SpanKind) -> SpanRecord {
        self.inner.lock().trace_context.child_span(name, kind)
    }
}
