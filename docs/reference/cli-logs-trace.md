# CLI: agentix logs --trace

The `agentix logs` command displays agent execution logs and, with the `--trace` flag, renders a trace waterfall showing the execution hierarchy of an agent run.

## Usage

```
agentix logs <AGENT> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `AGENT` | Yes | Agent name |

### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--run <RUN_ID>` | | latest | Run ID to show (uses latest run if omitted) |
| `--follow` | `-f` | false | Follow log output in real time |
| `--trace` | | false | Show execution trace with span waterfall |
| `--output <FORMAT>` | | `text` | Output format: `text` (default) or `json` |

## Examples

### View latest run logs

```bash
agentix logs my-agent
```

Output:
```
[start] run abc-123-def-456
[think] I need to check the database status first.
[act] kubectl(command="get pods -n database")
[observe] NAME                    READY   STATUS    RESTARTS
[think] The database pods are running. Let me check metrics.
[act] kubectl(command="top pods -n database")
[observe] NAME                    CPU     MEMORY
output: All database pods are healthy with normal resource usage.
```

### View trace waterfall for latest run

```bash
agentix logs my-agent --trace
```

Output:
```
Trace: abc-123-def-456  Agent: my-agent  Run: run-789

agent_run                                          5.2s  OK
  iteration_1                                      2.1s  OK
    llm_call  anthropic/claude-sonnet-4-6 (1500 in / 350 out)  1.8s  OK
    tool_call_kubectl  kubectl                     0.3s  OK
  iteration_2                                      3.0s  OK
    llm_call  anthropic/claude-sonnet-4-6 (2100 in / 500 out)  2.8s  OK
```

The waterfall view shows:
- Span names in cyan with indentation showing parent-child hierarchy
- Key attributes (model name, token counts, tool name) inline
- Duration formatted as seconds or milliseconds
- Status in green (OK) or red (ERROR) with error messages

### View trace as JSON (for piping to jq)

```bash
agentix logs my-agent --trace --output json
```

Output:
```json
[
  {
    "span_id": "abc-123",
    "parent_span_id": null,
    "trace_id": "trace-456",
    "name": "agent_run",
    "kind": "run",
    "start_time": "2026-03-13T12:00:00Z",
    "end_time": "2026-03-13T12:00:05Z",
    "duration_ms": 5200,
    "status": "ok",
    "attributes": {
      "agent": "my-agent"
    }
  }
]
```

### View trace for a specific run

```bash
agentix logs my-agent --trace --run abc-123
```

### Follow live logs

```bash
agentix logs my-agent -f
```

Polls the gateway every 500ms for new log events. Stops when the run completes (Complete or Error event).

### Pipe trace JSON to jq for analysis

```bash
# List all LLM calls with their token counts
agentix logs my-agent --trace --output json | \
  jq '.[] | select(.kind == "llm_call") | {name, model: .attributes.model, tokens: (.attributes.input_tokens + "/" + .attributes.output_tokens)}'

# Find spans that took longer than 2 seconds
agentix logs my-agent --trace --output json | \
  jq '.[] | select(.duration_ms > 2000) | {name, duration_ms}'
```

## REST API

The trace data is also available via REST API:

```bash
# Get trace spans for a run
curl http://localhost:7777/api/v1/agents/my-agent/runs/abc-123/trace

# Get structured logs for a run
curl http://localhost:7777/api/v1/agents/my-agent/runs/abc-123/structured-logs
```

## See Also

- [Telemetry Configuration](telemetry-config.md) — workspace telemetry settings
- [OTel Integration Guide](../guides/otel-integration.md) — Jaeger, Grafana Tempo, Datadog setup
