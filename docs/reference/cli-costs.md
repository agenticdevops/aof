# CLI Reference: `agentix costs`

Cost tracking commands for viewing LLM usage and spend across agents.

## Synopsis

```
agentix costs [OPTIONS]
agentix costs agent <NAME> [OPTIONS]
```

## Commands

### `agentix costs`

List a cost summary table for all agents, sorted by total spend (highest first).

```bash
agentix costs
```

**Output columns:**

| Column | Description |
|--------|-------------|
| AGENT | Agent name |
| RUNS | Total number of completed runs |
| INPUT TOKENS | Total input tokens consumed across all runs |
| OUTPUT TOKENS | Total output tokens generated across all runs |
| COST (USD) | Total estimated cost in USD |
| LAST RUN | Timestamp of the most recent run |

**Example output:**

```
AGENT                     RUNS   INPUT TOKENS  OUTPUT TOKENS   COST (USD)  LAST RUN
----------------------------------------------------------------------------------------------------
my-analyst                  14         48,230         12,810     $0.0823  2026-03-13 04:12
data-pipeline                7         22,150          5,640     $0.0341  2026-03-12 22:05
quick-responder             52          6,180          3,920     $0.0012  2026-03-13 04:50
----------------------------------------------------------------------------------------------------
Total: $0.1176
```

### `agentix costs agent <NAME>`

Show a detailed cost summary and per-run breakdown for a specific agent.

```bash
agentix costs agent my-analyst
agentix costs agent my-analyst --limit 50
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--limit N` | 20 | Number of recent runs to include in the per-run table |

**Example output:**

```
Cost summary for: my-analyst
============================================================
  Total runs:          14
  Total input tokens:  48230
  Total output tokens: 12810
  Total cost (USD):    $0.0823
  Last run at:         2026-03-13 04:12:07

Recent runs (last 14):
------------------------------------------------------------------------------------------
RUN ID          MODEL                         INPUT TOK    OUTPUT TOK  COST (USD)
------------------------------------------------------------------------------------------
a1b2c3d4e5f6    gemini-2.0-flash                   3240           812   $0.0006
9f8e7d6c5b4a    gemini-2.0-flash                   3580           920   $0.0007
...
```

## Global Options

| Flag | Description |
|------|-------------|
| `--output json` | Output raw JSON instead of formatted tables |
| `--gateway-url URL` | Gateway base URL (default: `http://127.0.0.1:7777`) |

## JSON Output

Use `--output json` for machine-readable output:

```bash
agentix costs --output json
agentix costs agent my-analyst --output json
```

The `agent` subcommand returns:
```json
{
  "summary": { "agent": "...", "total_runs": 14, "total_cost_usd": 0.0823, ... },
  "runs": [ { "run_id": "...", "model": "...", "cost_usd": 0.0006, ... } ]
}
```

## Cost Sources

Costs shown in the table use the following priority:

1. **Actual cost** — if the LLM provider reports usage cost in the API response, that value is recorded and used
2. **Estimated cost** — if no actual cost is reported, cost is estimated using the built-in pricing table

The pricing table covers models from Anthropic, OpenAI, Google, and Groq. See `docs/concepts/cost-tracking.md` for the full table.

## REST API Equivalent

These CLI commands call the following REST endpoints on the gateway:

| CLI command | REST endpoint |
|-------------|---------------|
| `agentix costs` | `GET /api/v1/costs` |
| `agentix costs agent <name>` | `GET /api/v1/costs/agents/<name>` |
| `agentix costs agent <name> --limit N` | `GET /api/v1/costs/agents/<name>/runs?limit=N` |

## Configuring Budgets

To set spending limits on an agent, add a `budget` section to its YAML:

```yaml
budget:
  daily_limit_usd: 0.50    # Block new runs if today's spend >= $0.50
  max_tokens_per_run: 4000  # Stop the ReAct loop if token usage exceeds this
```

See `docs/guides/cost-budgets.md` for the full budget configuration guide.
