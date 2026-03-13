# Telemetry and Observability

OpenAgentiX provides built-in observability through OpenTelemetry-compatible traces, metrics, and structured logs. Every agent run automatically produces a trace with spans for each loop iteration, LLM call, tool call, and research phase — giving operators full visibility into agent behavior without any additional instrumentation.

## Trace Model

Every agent execution generates a hierarchical trace. The trace is a tree of spans, where each span represents a distinct phase of the execution:

```
agent_run (SpanKind::Run)                    Root — one per agent run
  iteration_1 (SpanKind::Iteration)          One per ReAct loop cycle
    llm_call (SpanKind::LlmCall)             One per model.generate()
    tool_call_kubectl (SpanKind::ToolCall)    One per tool dispatch
  iteration_2 (SpanKind::Iteration)
    llm_call (SpanKind::LlmCall)
  research_phase (SpanKind::Research)         One per research phase (if enabled)
  memory_recall (SpanKind::MemoryRecall)      One per vector memory search (if enabled)
```

### Span Kinds

| Kind | Description | Parent |
|------|-------------|--------|
| `Run` | Root span for the entire agent run | None |
| `Iteration` | One cycle of the ReAct loop | Run |
| `LlmCall` | A call to an LLM provider (`model.generate()`) | Iteration |
| `ToolCall` | Execution of a single tool | Iteration |
| `Research` | A research phase (multi-step information gathering) | Run |
| `MemoryRecall` | A vector memory search/recall operation | Run |

## Correlation IDs

Every span carries two correlation identifiers:

- **`trace_id`** — Unique per agent run. All spans in one run share the same `trace_id`.
- **`span_id`** — Unique per span. Used to build the parent-child tree.

These IDs propagate into structured log entries, allowing logs to be correlated with traces in any log aggregation system (Loki, Elasticsearch, CloudWatch Logs).

### TraceContext

The `TraceContext` struct carries trace state through the execution pipeline:

```rust
let ctx = TraceContext::new_root("dba-optimizer", "run-abc-123");
// ctx.trace_id  = "550e8400-..."  (unique per run)
// ctx.span_id   = "6ba7b810-..."  (unique per span)
// ctx.agent_name = "dba-optimizer"
// ctx.run_id    = "run-abc-123"

let child_span = ctx.child_span("llm_call", SpanKind::LlmCall);
// child_span.parent_span_id = Some(ctx.span_id)
// child_span.trace_id = ctx.trace_id  (inherited)
```

## Structured Logs

All agent run logs use the `StructuredLogEntry` format — single-line JSON with correlation IDs:

```json
{
  "timestamp": "2026-03-13T12:00:00Z",
  "level": "info",
  "message": "Agent run started",
  "trace_id": "abc-123-def-456",
  "span_id": "789-012-345-678",
  "agent": "dba-optimizer",
  "run_id": "run-789",
  "fields": {"iterations": 5}
}
```

### Log Levels

Levels are ordered from least to most severe:

| Level | Use Case |
|-------|----------|
| `trace` | Finest-grained diagnostic info (token counts, timing) |
| `debug` | Developer diagnostics (prompt construction, tool selection) |
| `info` | Normal operations (run started, iteration complete) |
| `warn` | Potential issues (budget threshold approaching, tool retry) |
| `error` | Error conditions (LLM timeout, tool failure, budget exceeded) |

## SpanRecord Attributes

Each span kind carries domain-specific attributes:

### LLM Call Spans

| Attribute | Example | Description |
|-----------|---------|-------------|
| `model` | `anthropic/claude-sonnet-4-6` | Model identifier |
| `provider` | `anthropic` | Provider name |
| `input_tokens` | `1500` | Tokens sent to the model |
| `output_tokens` | `350` | Tokens received from the model |
| `cost_usd` | `0.0098` | Estimated cost of this call |

### Tool Call Spans

| Attribute | Example | Description |
|-----------|---------|-------------|
| `tool_name` | `kubectl` | Name of the tool executed |
| `tool_type` | `cli` | Tool type (cli/mcp/shell) |
| `status` | `ok` | Outcome (ok/error) |

### Run Spans

| Attribute | Example | Description |
|-----------|---------|-------------|
| `total_iterations` | `3` | Number of ReAct loop cycles |
| `total_tool_calls` | `5` | Total tool invocations |
| `final_status` | `completed` | Run outcome |

## CLI Access

View execution traces from the terminal:

```bash
# Waterfall view of the latest run
agentix logs my-agent --trace

# JSON output for piping to jq
agentix logs my-agent --trace --output json

# Trace for a specific run
agentix logs my-agent --trace --run abc-123
```

The waterfall view renders the span tree with indentation, durations, and key attributes:

```
Trace: abc-123-def-456  Agent: dba-optimizer  Run: run-789

agent_run                                          5.2s  OK
  iteration_1                                      2.1s  OK
    llm_call          anthropic/claude-sonnet-4-6   1.8s  OK  (1500 in / 350 out)
    tool_call_kubectl  kubectl                      0.3s  OK
  iteration_2                                      3.0s  OK
    llm_call          anthropic/claude-sonnet-4-6   2.8s  OK  (2100 in / 500 out)
```

See [CLI Logs Trace Reference](../reference/cli-logs-trace.md) for full usage details.
