# Cost Tracking and Budgets

OpenAgentiX tracks every LLM call your agents make — recording token usage and cost — and lets you set spending limits to prevent runaway costs.

## Overview

Every time an agent calls an LLM, the gateway records:
- Input and output token counts
- Estimated cost (from the built-in pricing table)
- Actual cost when the provider reports it in the API response

This data is stored in `data/cost.db` (SQLite) alongside the gateway's run history.

## Viewing Costs

Use the `agentix costs` command to inspect spending:

```bash
# Summary table for all agents
agentix costs

# Drill into one agent with per-run breakdown
agentix costs agent my-agent

# Show the 50 most recent runs
agentix costs agent my-agent --limit 50

# Machine-readable JSON
agentix costs --output json
```

**Example summary output:**
```
AGENT                     RUNS   INPUT TOKENS  OUTPUT TOKENS   COST (USD)  LAST RUN
----------------------------------------------------------------------------------------------------
my-analyst                  14         48,230         12,810     $0.0823  2026-03-13 04:12
data-pipeline                7         22,150          5,640     $0.0341  2026-03-12 22:05
quick-responder             52          6,180          3,920     $0.0012  2026-03-13 04:50
----------------------------------------------------------------------------------------------------
Total: $0.1176
```

## Setting Budgets

Add a `budget` section to your agent YAML to enforce spending limits:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: my-analyst
spec:
  model: google/gemini-2.0-flash
  system_prompt: |
    You are a helpful analyst...

  budget:
    daily_limit_usd: 0.50    # Stop new runs if today's spend >= $0.50
    max_tokens_per_run: 4000  # Stop the run if tokens exceed 4,000
```

Both fields are optional. You can set one or both.

### Daily Limit (`daily_limit_usd`)

Before starting a new run, the gateway checks how much this agent has spent today. If the current day's total equals or exceeds the limit, the run is rejected immediately with:

```
error: Daily budget exceeded for 'my-analyst': $0.52 spent, limit $0.50
```

The agent is not started and no tokens are consumed.

### Per-Run Token Limit (`max_tokens_per_run`)

During the ReAct loop, the gateway tracks cumulative token usage (input + output combined) across all LLM calls in the run. If the total exceeds the limit, the loop stops cleanly and reports:

```json
{
  "stopped_reason": {
    "reason": "token_limit_exceeded",
    "limit": 4000,
    "used": 4238
  }
}
```

The partial output up to that point is preserved and returned to the caller.

### Example Agent

See `quickstart/agents/budget-example.yaml` for a working agent with budget configuration.

## Smart Model Routing (CORE-05)

OpenAgentiX can automatically route requests to cheaper or more powerful models based on the complexity of the input message.

Messages are scored 0-100 using three factors:
- **Message length** (longer = more complex, up to 40 points)
- **Complexity keywords** (analyze, research, implement, compare, investigate, etc., up to 40 points)
- **Question depth** (presence of why/how questions, up to 20 points)

Score ranges map to model tiers:

| Score | Tier | Typical Use |
|-------|------|-------------|
| 0-30 | Flash | Simple Q&A, quick lookups |
| 31-70 | Standard | Moderate analysis, structured tasks |
| 71-100 | Pro | Complex research, multi-step reasoning |

This routing is applied automatically when the framework selects a model — it does not override the `model` field in your agent YAML. The tier provides a hint when multiple model options are configured for a provider.

## Cost Data Source

Cost values follow this priority order (COST-07):

1. **Actual cost** — if the LLM provider includes billing information in the API response, that value is stored as `actual_cost_usd` and used for all aggregations
2. **Estimated cost** — if no actual cost is reported, cost is estimated from the built-in pricing table using the recorded token counts

The pricing table covers models from Anthropic, Google, OpenAI, and Groq. See `docs/concepts/cost-tracking.md` for the full table.

The `COALESCE(actual_cost_usd, cost_usd)` SQL pattern ensures actual costs always take precedence when available.

## REST API

The same data is available via REST for programmatic access or dashboards:

```bash
# All agent summaries
GET /api/v1/costs

# One agent summary
GET /api/v1/costs/agents/my-analyst

# Per-run breakdown (paginated)
GET /api/v1/costs/agents/my-analyst/runs?limit=20
```

See `docs/reference/cli-costs.md` for full API details.

## Checking Current Daily Spend

To see how much an agent has spent today before starting a run:

```bash
agentix costs agent my-analyst
```

The summary section shows `last_run_at` but not today's spend directly. Use the REST API for precise real-time queries:

```bash
curl http://localhost:7777/api/v1/costs/agents/my-analyst | jq .total_cost_usd
```

Note: the `total_cost_usd` from the summary endpoint is all-time, not today only. The daily budget check happens automatically when you start a run.
