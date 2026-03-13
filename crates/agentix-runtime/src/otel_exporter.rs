//! OpenTelemetry OTLP exporter for agentix trace spans.
//!
//! Converts internal `SpanRecord`s to the OTLP JSON wire format and pushes
//! them to any OpenTelemetry-compatible collector via HTTP POST.

use agentix_core::telemetry::{SpanKind, SpanRecord, SpanStatus};
use agentix_core::{AgentixError, TelemetryConfig};
use serde_json::json;

/// Converts internal SpanRecords to OTLP JSON and exports them to
/// an OpenTelemetry collector via HTTP.
#[derive(Clone)]
pub struct OtelExporter {
    config: TelemetryConfig,
    client: reqwest::Client,
}

impl OtelExporter {
    /// Create a new exporter with the given telemetry config.
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Convert SpanRecords to OTLP v1 JSON format.
    ///
    /// Produces a `resourceSpans` structure compatible with OTLP HTTP JSON.
    pub fn export_spans(service_name: &str, spans: &[SpanRecord]) -> serde_json::Value {
        let otlp_spans: Vec<serde_json::Value> = spans
            .iter()
            .map(|span| {
                let trace_id = uuid_to_hex_trace_id(&span.trace_id);
                let span_id = uuid_to_hex_span_id(&span.span_id);
                let parent_span_id = span
                    .parent_span_id
                    .as_ref()
                    .map(|id| uuid_to_hex_span_id(id))
                    .unwrap_or_default();

                let start_nanos = span.start_time.timestamp_nanos_opt().unwrap_or(0);
                let end_nanos = span
                    .end_time
                    .and_then(|t| t.timestamp_nanos_opt())
                    .unwrap_or(start_nanos);

                let (status_code, status_message) = match &span.status {
                    SpanStatus::Ok => (1, None),
                    SpanStatus::Error(msg) => (2, Some(msg.clone())),
                };

                let attributes: Vec<serde_json::Value> = span
                    .attributes
                    .iter()
                    .map(|(k, v)| {
                        json!({
                            "key": k,
                            "value": {"stringValue": v}
                        })
                    })
                    .collect();

                let kind = match span.kind {
                    SpanKind::LlmCall | SpanKind::ToolCall => 3, // CLIENT
                    _ => 1,                                       // INTERNAL
                };

                let mut status = json!({"code": status_code});
                if let Some(msg) = status_message {
                    status["message"] = json!(msg);
                }

                json!({
                    "traceId": trace_id,
                    "spanId": span_id,
                    "parentSpanId": parent_span_id,
                    "name": span.name,
                    "kind": kind,
                    "startTimeUnixNano": start_nanos.to_string(),
                    "endTimeUnixNano": end_nanos.to_string(),
                    "status": status,
                    "attributes": attributes,
                })
            })
            .collect();

        json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        {"key": "service.name", "value": {"stringValue": service_name}}
                    ]
                },
                "scopeSpans": [{
                    "scope": {
                        "name": "agentix",
                        "version": agentix_core::VERSION,
                    },
                    "spans": otlp_spans,
                }]
            }]
        })
    }

    /// Push spans to the configured OTLP collector endpoint.
    ///
    /// Fire-and-forget — logs a warning on failure but never propagates errors.
    pub async fn push_to_collector(&self, spans: &[SpanRecord]) -> Result<(), AgentixError> {
        let endpoint = match &self.config.otlp_endpoint {
            Some(ep) => ep.clone(),
            None => return Ok(()), // No endpoint configured — no-op.
        };

        let body = Self::export_spans(&self.config.service_name, spans);

        self.client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AgentixError::Runtime(format!("OTel export failed: {}", e))
            })?;

        Ok(())
    }
}

/// Convert a UUID string to a 32-character hex trace ID (OTLP format).
fn uuid_to_hex_trace_id(uuid: &str) -> String {
    let hex: String = uuid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() >= 32 {
        hex[..32].to_string()
    } else {
        format!("{:0>32}", hex)
    }
}

/// Convert a UUID string to a 16-character hex span ID (OTLP format).
fn uuid_to_hex_span_id(uuid: &str) -> String {
    let hex: String = uuid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() >= 16 {
        hex[..16].to_string()
    } else {
        format!("{:0>16}", hex)
    }
}
