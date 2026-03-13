---
phase: 18-telemetry-observability
plan: 04
status: complete
started: 2026-03-13
completed: 2026-03-13
---

# Summary: Plan 18-04 — OpenTelemetry Export + Prometheus Metrics

## What Was Built

OtelExporter that converts internal SpanRecords to OTLP v1 JSON wire format for export to any OpenTelemetry-compatible collector. TelemetryConfig for workspace-level configuration. Prometheus /metrics endpoint. Integration guide.

## Key Files

### Created
- `crates/agentix-runtime/src/otel_exporter.rs` — OtelExporter with export_spans (OTLP JSON) and push_to_collector (async HTTP POST)
- `crates/agentix-runtime/tests/otel_exporter_test.rs` — 5 unit tests covering format, attributes, status mapping, empty input, parent linking
- `docs/guides/otel-integration.md` — Setup guides for Jaeger, Grafana Tempo, Prometheus, Datadog

### Modified
- `crates/agentix-core/src/config.rs` — Added TelemetryConfig struct (enabled, otlp_endpoint, service_name, export_metrics)
- `crates/agentix-core/src/lib.rs` — Re-exported TelemetryConfig
- `crates/agentix-runtime/src/lib.rs` — Added `pub mod otel_exporter` and re-export
- `crates/agentix-runtime/src/gateway/agent_manager.rs` — Added otel_exporter field, async push after run completion
- `crates/agentix-runtime/src/gateway/api.rs` — Added GET /metrics endpoint
- `crates/agentix/src/commands/onboard.rs` — Added `telemetry: None` to both WorkspaceSpec initializers

## Verification

- `cargo test --test otel_exporter_test -p agentix-runtime` — 5/5 tests pass
- `cargo check --workspace` — clean (warnings only, no errors)

## Self-Check: PASSED

All must_haves verified:
- [x] OtelExporter struct converts SpanRecords to OTLP JSON
- [x] export_spans produces valid resourceSpans structure
- [x] push_to_collector POSTs to OTLP endpoint with 5-second timeout
- [x] TelemetryConfig struct in config.rs with enabled, otlp_endpoint, service_name, export_metrics
- [x] WorkspaceSpec has telemetry: Option<TelemetryConfig> field
- [x] GET /metrics endpoint serves Prometheus text format
- [x] Spans pushed to OTel collector when otlp_endpoint is configured
- [x] Zero overhead when telemetry not configured
- [x] docs/guides/otel-integration.md with Jaeger, Tempo, Prometheus, Datadog guides
