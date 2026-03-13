# Telemetry Configuration Reference

OpenAgentiX supports workspace-level telemetry configuration for OpenTelemetry trace export and Prometheus metrics.

## Configuration

Telemetry is configured in the `spec.telemetry` section of `agentix.yaml`:

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-project
spec:
  telemetry:
    enabled: true
    otlp_endpoint: "http://localhost:4318/v1/traces"
    service_name: "my-project"
    export_metrics: true
```

## Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `telemetry.enabled` | `bool` | `true` | Master switch for telemetry collection. When `false`, no trace spans or structured logs are collected. Zero overhead when disabled. |
| `telemetry.otlp_endpoint` | `string` | *(none)* | OTLP HTTP endpoint URL for span export. Spans are POST-ed as JSON to this URL after each agent run. When not set, spans are stored locally in `trace.db` but not exported. Example: `http://localhost:4318/v1/traces` |
| `telemetry.service_name` | `string` | `"agentix"` | Service name reported in OpenTelemetry resource attributes. Appears as the service identifier in Jaeger, Grafana Tempo, Datadog APM, etc. |
| `telemetry.export_metrics` | `bool` | `true` | Whether to expose Prometheus-compatible metrics at the `GET /metrics` endpoint on the gateway. |

## Behavior

### Trace Collection

When `enabled` is `true` (the default), every agent run produces trace spans:

- **agent_run** (root span) — the entire run lifecycle
- **memory_recall** — vector memory retrieval (if configured)
- **research_phase** — research phase (if enabled)
- **iteration_N** — each ReAct loop iteration
- **llm_call** — each LLM API call (with model, token counts)
- **tool_call_NAME** — each tool execution (with tool name, status)

Spans carry attributes such as `model`, `input_tokens`, `output_tokens`, `tool_name`, `iteration_number`, and `status`.

### Trace Persistence

All trace spans and structured logs are persisted to `data/trace.db` (SQLite) regardless of whether OTLP export is configured. This enables CLI trace viewing and REST API access without an external trace backend.

### OTLP Export

When `otlp_endpoint` is set, spans are converted to the OTLP v1 JSON wire format and POST-ed to the endpoint after each agent run completes. The export is fire-and-forget (async) and never blocks the run response. Failures are logged as warnings.

Compatible backends:
- Jaeger (with OTLP receiver enabled)
- Grafana Tempo
- Datadog Agent (with OTLP config)
- Any OTLP-compatible collector

### Prometheus Metrics

When `export_metrics` is `true`, the gateway exposes a `GET /metrics` endpoint serving Prometheus text format metrics:

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `aof_agent_executions_total` | Counter | `agent_id`, `status` | Total agent runs |
| `aof_agent_execution_duration_seconds` | Histogram | | Run duration |
| `aof_agents_active` | Gauge | | Currently executing agents |
| `aof_llm_requests_total` | Counter | `provider`, `model` | Total LLM API calls |
| `aof_llm_tokens_total` | Counter | `provider`, `type` | Token consumption (input/output) |
| `aof_llm_latency_seconds` | Histogram | | LLM API call latency |

## Examples

### Minimal (local tracing only)

```yaml
spec:
  telemetry:
    enabled: true
```

Traces stored locally, viewable via `agentix logs --trace` and REST API.

### Full export to Jaeger

```yaml
spec:
  telemetry:
    enabled: true
    otlp_endpoint: "http://localhost:4318/v1/traces"
    service_name: "production-agents"
    export_metrics: true
```

### Disabled (zero overhead)

```yaml
spec:
  telemetry:
    enabled: false
```

No traces collected, no metrics endpoint.

## See Also

- [OTel Integration Guide](../guides/otel-integration.md) — setup for Jaeger, Grafana Tempo, Datadog
- [CLI Trace Viewer](cli-logs-trace.md) — `agentix logs --trace` usage
