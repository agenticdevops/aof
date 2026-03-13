---
phase: 18-telemetry-observability
plan: 05
status: complete
started: 2026-03-13
completed: 2026-03-13
---

# Summary: Plan 18-05 — CLI Trace Viewer + Quickstart + Docs + Changelog

## What Was Built

CLI `--trace` flag for `agentix logs` with waterfall renderer, quickstart telemetry examples, reference documentation, and CHANGELOG update for v2.0.0-alpha.6.

## Key Files

### Created
- `quickstart/agents/telemetry-example/agent.yaml` — example agent manifest
- `quickstart/agents/telemetry-example/SOUL.md` — example agent system prompt
- `quickstart/agentix-otel.yaml` — workspace config with OTLP endpoint
- `docs/reference/telemetry-config.md` — full TelemetryConfig field reference
- `docs/reference/cli-logs-trace.md` — CLI usage and examples

### Modified
- `crates/agentix/src/cli.rs` — added `--trace` flag to Logs command
- `crates/agentix/src/main.rs` — pass trace flag to logs handler
- `crates/agentix/src/client.rs` — added get_run_trace() and get_run_structured_logs() methods
- `crates/agentix/src/commands/logs.rs` — trace mode with render_trace_waterfall()
- `CHANGELOG.md` — added [2.0.0-alpha.6] section with all Phase 18 features

## Verification

- `cargo check --workspace` — clean (warnings only, no errors)
- All quickstart files exist and are valid
- CHANGELOG has [2.0.0-alpha.6] section

## Self-Check: PASSED

All must_haves verified:
- [x] agentix logs --trace flag exists in CLI definition
- [x] Waterfall view renders span hierarchy with indentation, durations, and status
- [x] --trace --output json outputs raw JSON trace data
- [x] GatewayClient has get_run_trace() and get_run_structured_logs() methods
- [x] quickstart/agents/telemetry-example/ is a valid GitAgent-compatible agent directory
- [x] quickstart/agentix-otel.yaml has telemetry config with otlp_endpoint
- [x] docs/reference/telemetry-config.md exists with full field reference
- [x] docs/reference/cli-logs-trace.md exists with CLI usage examples
- [x] CHANGELOG.md has [2.0.0-alpha.6] section listing all Phase 18 features
