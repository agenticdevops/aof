# Run Persistence

OpenAgentiX persists every agent run to a local SQLite database so that `agentix runs` shows complete history across server restarts—including which trigger fired the run, when it started and ended, and a summary of the output.

## How it works

When any trigger fires an agent run (cron schedule, webhook, GitHub/Jira event, channel mention, agent-to-agent call, or CLI), the runtime:

1. Creates a `RunRecord` in the SQLite database before execution begins (`status: running`).
2. Updates the record to `status: completed` (or `failed`) once the run finishes, recording duration, iterations, and a truncated output summary.
3. Dispatches any configured notifications (webhook callbacks, log entries).

The database file is created automatically at `./agentix-runs.db` in the working directory when the server starts.

## RunRecord fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | UUID for this run |
| `agent_name` | string | Name of the agent that ran |
| `trigger_source` | string | What fired the run: `cron`, `webhook`, `github`, `jira`, `slack`, `discord`, `telegram`, `agent`, or `cli` |
| `trigger_id` | string? | ID of the specific trigger configuration |
| `started_at` | datetime | RFC 3339 timestamp when run began |
| `ended_at` | datetime? | RFC 3339 timestamp when run finished |
| `duration_ms` | integer? | Wall-clock duration in milliseconds |
| `status` | string | `running`, `completed`, or `failed` |
| `iterations` | integer? | Number of ReAct loop iterations |
| `input_summary` | string? | First 200 chars of the trigger payload |
| `output_summary` | string? | First 500 chars of the agent's final response |
| `error` | string? | Error message if status is `failed` |

## CLI: viewing run history

```bash
# Show the 20 most recent runs across all agents (default limit)
agentix runs

# Filter to a specific agent
agentix runs --agent my-agent

# Increase the limit
agentix runs --limit 50

# Machine-readable JSON output
agentix runs --output json
```

The text output includes a **TRIGGER** column showing which trigger source fired each run:

```
RUN ID        AGENT                 TRIGGER     STATUS        STARTED               ITERATIONS
a1b2c3d4e5f6  deploy-watcher        cron        completed     2026-03-13 05:12      3
98f7e6d5c4b3  incident-responder    webhook     completed     2026-03-13 05:10      5
77a6b5c4d3e2  doc-helper            slack       completed     2026-03-13 05:08      2
```

## REST API: querying runs

```http
GET /api/v1/runs
GET /api/v1/runs?agent=my-agent
GET /api/v1/runs?limit=50
GET /api/v1/runs?agent=my-agent&limit=10
```

Response is a JSON array of run objects matching the fields above.

## Notification routing

Agents can declare `on_run_complete` hooks in their YAML spec to receive a notification after each triggered run:

```yaml
name: my-agent
on_run_complete:
  - type: webhook
    url: https://hooks.example.com/run-complete
  - type: log
    level: info
```

### Notification types

| Type | Behavior |
|------|----------|
| `webhook` | HTTP POST to `url` with the full `RunRecord` as JSON body |
| `log` | Emits a structured log line at `info` level (default) |

## Database location

The database path defaults to `./agentix-runs.db` relative to the server's working directory. This can be inspected directly with any SQLite client:

```bash
sqlite3 ./agentix-runs.db "SELECT agent_name, trigger_source, status, started_at FROM runs ORDER BY started_at DESC LIMIT 20;"
```

## Storage considerations

- WAL (Write-Ahead Logging) mode is enabled for concurrent read performance.
- Input is truncated at 200 characters; output at 500 characters to keep the database compact.
- There is no automatic pruning — use `sqlite3` directly to delete old records if needed.
