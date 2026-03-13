# Cost Tracking

OpenAgentiX tracks LLM token usage and cost at three granularities, giving you full visibility into what your agents spend.

## Overview

Every LLM call made by an agent produces a `CostRecord`. These records are aggregated into:

- **Per-call** (`CostRecord`): one record per API call — model, tokens, estimated cost, actual cost
- **Per-run** (`RunCostSummary`): all calls in one agent run summed together
- **Per-agent** (`CostSummary`): all runs for an agent summed together

Cost data is stored in `data/cost.db` (SQLite) in your workspace directory.

## Data Model

### CostRecord

One row per LLM API call:

```json
{
  "id": "a1b2c3d4-...",
  "agent_name": "ops-agent",
  "run_id": "run-xyz",
  "model": "claude-sonnet-4-6",
  "provider": "anthropic",
  "input_tokens": 1240,
  "output_tokens": 380,
  "cost_usd": 0.009420,
  "actual_cost_usd": 0.008100,
  "recorded_at": "2026-03-13T10:15:00Z"
}
```

| Field | Description |
|-------|-------------|
| `cost_usd` | Estimated cost from pricing table (always present) |
| `actual_cost_usd` | Cost from provider API response (when available) — see [Actual vs Estimated](#actual-vs-estimated-cost) |

### RunCostSummary

Aggregated across all LLM calls in one run:

```json
{
  "run_id": "run-xyz",
  "agent_name": "ops-agent",
  "model": "claude-sonnet-4-6",
  "input_tokens": 4800,
  "output_tokens": 1200,
  "cost_usd": 0.0324,
  "started_at": "2026-03-13T10:15:00Z"
}
```

### CostSummary

Aggregated across all runs for one agent:

```json
{
  "agent_name": "ops-agent",
  "total_runs": 12,
  "total_input_tokens": 58000,
  "total_output_tokens": 14400,
  "total_cost_usd": 0.3888,
  "last_run_at": "2026-03-13T10:15:00Z"
}
```

### ModelPricing

Static pricing table entry (USD per 1,000 tokens):

```json
{
  "provider": "anthropic",
  "model": "claude-sonnet-4-6",
  "input_cost_per_1k": 0.003,
  "output_cost_per_1k": 0.015
}
```

## Actual vs Estimated Cost

OpenAgentiX always computes an **estimated cost** from the token count × pricing table. However, some providers return the actual cost in their API response (e.g., Anthropic returns cached token counts that affect real billing).

When `actual_cost_usd` is present on a `CostRecord`, it takes precedence in all aggregations:

```sql
SUM(COALESCE(actual_cost_usd, cost_usd))
```

This means: if the provider told us the real cost, use that. Otherwise, use our estimate.

## Pricing Table

Default pricing (USD per 1,000 tokens) as of 2026-03:

| Provider | Model | Input /1k | Output /1k |
|----------|-------|-----------|------------|
| Anthropic | claude-sonnet-4-6 | $0.003 | $0.015 |
| Anthropic | claude-3-5-sonnet-20241022 | $0.003 | $0.015 |
| Anthropic | claude-3-5-haiku-20241022 | $0.0008 | $0.004 |
| Anthropic | claude-3-opus-20240229 | $0.015 | $0.075 |
| OpenAI | gpt-4o | $0.0025 | $0.01 |
| OpenAI | gpt-4o-mini | $0.00015 | $0.0006 |
| Google | gemini-2.0-flash | $0.000075 | $0.0003 |
| Google | gemini-2.5-pro | $0.00125 | $0.01 |
| Groq | llama-3.3-70b-versatile | $0.00059 | $0.00079 |

Sources: Anthropic, OpenAI, Google pricing pages (subject to change).

## CLI Usage

View cost summaries from the command line (implemented in Phase 17):

```bash
# Summary across all agents
agentix costs

# Drill into one agent by run
agentix costs agent ops-agent

# Limit run history shown
agentix costs agent ops-agent --limit 5
```

See [`agentix costs` CLI reference](../reference/cli-costs.md) for full output format details.

## REST API

Cost data is also accessible via the REST API for programmatic access and the Phase 22 Command Center:

```
GET /api/v1/costs                          → all agent summaries
GET /api/v1/costs/agents/:name             → single agent summary
GET /api/v1/costs/agents/:name/runs        → per-run breakdown
```
