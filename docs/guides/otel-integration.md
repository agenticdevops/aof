# OpenTelemetry Integration Guide

OpenAgentiX provides built-in observability through OpenTelemetry-compatible tracing and Prometheus metrics. Every agent run automatically produces structured trace spans that can be exported to Jaeger, Grafana Tempo, Datadog, or any OTLP-compatible backend.

## Quick Start with Jaeger

Jaeger is the fastest way to visualize agent execution traces locally.

### 1. Start Jaeger with OTLP receiver

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

### 2. Configure telemetry in agentix.yaml

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
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
  defaults:
    model: anthropic/claude-sonnet-4-6
```

### 3. Start the gateway and run an agent

```bash
agentix gateway start
curl -X POST http://localhost:7777/run/hello-world \
  -H "Content-Type: application/json" \
  -d '{"input": "What is OpenTelemetry?"}'
```

### 4. View traces

Open [http://localhost:16686](http://localhost:16686) in your browser. Select the service name you configured (e.g., `my-project`) to see agent execution traces with full span hierarchy.

## Grafana Tempo Setup

Grafana Tempo is a distributed tracing backend that integrates with Grafana dashboards.

### Self-hosted Tempo

```bash
# Start Tempo with OTLP receiver
docker run -d --name tempo \
  -p 3200:3200 \
  -p 4318:4318 \
  grafana/tempo:latest \
  -config.file=/etc/tempo.yaml
```

Configure your workspace:

```yaml
spec:
  telemetry:
    enabled: true
    otlp_endpoint: "http://localhost:4318/v1/traces"
    service_name: "my-project"
```

### Grafana Cloud

For Grafana Cloud, use your Tempo endpoint with basic auth:

```yaml
spec:
  telemetry:
    enabled: true
    otlp_endpoint: "https://tempo-us-central1.grafana.net/tempo/api/push"
    service_name: "my-project"
```

Set `GRAFANA_CLOUD_TOKEN` in your environment and configure any required auth headers in your OTLP collector proxy.

## Prometheus Metrics

OpenAgentiX exposes Prometheus-compatible metrics at the `/metrics` endpoint on the gateway.

### Scraping metrics

```bash
curl http://localhost:7777/metrics
```

### Available metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `aof_agent_executions_total` | Counter | `agent_id`, `status` | Total agent runs (success, error, timeout) |
| `aof_agent_execution_duration_seconds` | Histogram | | Agent run duration with bucket boundaries |
| `aof_agents_active` | Gauge | | Currently executing agents |
| `aof_llm_requests_total` | Counter | `provider`, `model` | Total LLM API calls |
| `aof_llm_tokens_total` | Counter | `provider`, `type` | Token consumption (input/output) |
| `aof_llm_latency_seconds` | Histogram | | LLM API call latency |
| `aof_agent_restarts_total` | Counter | | Agent restarts from crashes |
| `aof_agent_failures_total` | Counter | | Agent failures after retry exhaustion |

### Prometheus scrape config

Add this job to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'agentix'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:7777']
```

### Grafana dashboard

Import the metrics into Grafana and create dashboards tracking:

- Agent run success rate: `rate(aof_agent_executions_total{status="success"}[5m])`
- Average LLM latency: `rate(aof_llm_latency_seconds_sum[5m]) / rate(aof_llm_latency_seconds_count[5m])`
- Token consumption rate: `rate(aof_llm_tokens_total[5m])`
- Active agents: `aof_agents_active`

## Trace Structure

Every agent run produces a structured trace with hierarchical spans:

```
agent_run (root span)
  |-- memory_recall          # Vector memory retrieval (if configured)
  |-- research_phase         # Research phase (if enabled)
  |-- iteration_1            # First ReAct loop iteration
  |   |-- llm_call           # LLM API call
  |   |-- tool_call_search   # Tool execution
  |-- iteration_2            # Second iteration
  |   |-- llm_call
  |   |-- tool_call_execute
  |-- iteration_3            # Final iteration
      |-- llm_call           # Completion (no tool call)
```

### Span attributes

Each span carries contextual attributes:

| Span | Attributes |
|------|-----------|
| `agent_run` | `agent` (agent name) |
| `iteration_N` | `iteration_number` |
| `llm_call` | `model`, `input_tokens`, `output_tokens` |
| `tool_call_<name>` | `tool_name`, `status` (success/error) |

### Querying traces via REST API

```bash
# Get trace spans for a specific run
curl http://localhost:7777/trace?run_id=<run-id>

# Get structured logs for a run
curl http://localhost:7777/structured-logs?run_id=<run-id>
```

## Configuration Reference

Full `TelemetryConfig` field reference:

```yaml
spec:
  telemetry:
    # Whether telemetry collection is enabled.
    # When false, no spans are collected (zero overhead).
    # Default: true
    enabled: true

    # OTLP HTTP endpoint for span export.
    # Spans are POST-ed as JSON to this URL after each agent run.
    # When not set, spans are stored locally but not exported.
    # Example: "http://localhost:4318/v1/traces"
    otlp_endpoint: "http://localhost:4318/v1/traces"

    # Service name in OTel resource attributes.
    # Appears as the service name in Jaeger, Tempo, Datadog, etc.
    # Default: "agentix"
    service_name: "my-workspace"

    # Whether to expose Prometheus metrics at GET /metrics.
    # Default: true
    export_metrics: true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable/disable trace collection |
| `otlp_endpoint` | string (optional) | none | OTLP HTTP endpoint URL |
| `service_name` | string | `"agentix"` | OTel service name attribute |
| `export_metrics` | bool | `true` | Expose `/metrics` endpoint |

## Datadog Integration

Datadog supports OTLP ingestion through the Datadog Agent. Configure the Agent as an OTLP collector, then point OpenAgentiX at it.

### 1. Configure the Datadog Agent

Add to your `datadog.yaml`:

```yaml
otlp_config:
  receiver:
    protocols:
      http:
        endpoint: "0.0.0.0:4318"
```

### 2. Point OpenAgentiX at the Datadog Agent

```yaml
spec:
  telemetry:
    enabled: true
    otlp_endpoint: "http://localhost:4318/v1/traces"
    service_name: "my-project"
```

Traces will appear in the Datadog APM section under the configured service name. Datadog automatically maps OTLP span attributes to Datadog tags.

### 3. Environment variables

Set `DD_API_KEY` and `DD_SITE` for the Datadog Agent:

```bash
export DD_API_KEY="your-datadog-api-key"
export DD_SITE="datadoghq.com"  # or datadoghq.eu, etc.
```
