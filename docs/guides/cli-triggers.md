# CLI Triggers

The `agentix run` command lets you fire an agent once from the command line
with a text input. This is the quickest way to test an agent or integrate
it into scripts and automation pipelines.

## Usage

```bash
agentix run <agent-name> --input "your task here"
```

### Examples

| Command | What it does |
|---------|--------------|
| `agentix run dba-optimizer --input "analyze slow queries"` | Triggers the dba-optimizer agent |
| `agentix run deploy-bot --input "deploy v1.2.3 to staging"` | Triggers a deployment agent |
| `agentix run incident-coordinator --input "high CPU on db-primary"` | Triggers incident triage |
| `agentix run weekly-reporter --input "generate weekly summary"` | Triggers report generation |

## How It Works

`agentix run` sends a `TriggerEvent{source: cli}` to the gateway:

```
POST /api/v1/agents/<agent>/trigger
Body: {"payload": {"input": "your task here"}, "caller": "cli"}
```

The gateway queues the agent run asynchronously and returns a run ID:

```
Agent:  dba-optimizer
Run ID: run_abc123def456
Status: queued

Run queued. Track progress with:
  agentix runs dba-optimizer
```

## Output Format

By default, `agentix run` prints human-readable text. Use `--output json`
for structured output:

```bash
agentix run dba-optimizer --input "analyze slow queries" --output json
```

```json
{
  "accepted": true,
  "run_id": "run_abc123def456",
  "agent": "dba-optimizer",
  "trigger_id": "dba-optimizer-agent-trigger"
}
```

## Tracking Runs

After firing an agent, use `agentix runs` to see its history:

```bash
# List all runs for this agent
agentix runs dba-optimizer

# Get detailed output of a specific run
agentix runs dba-optimizer
```

CLI-triggered runs appear in the history with `trigger_source: cli`.

## Exit Codes

| Exit code | Meaning |
|-----------|---------|
| `0` | Agent run accepted by gateway |
| `1` | Connection error, agent not found, or other error |

The exit code reflects whether the run was **accepted**, not whether the
agent succeeded. Use `agentix runs` to check completion status.

## Connecting to a Remote Gateway

By default, `agentix run` connects to `http://127.0.0.1:7777`. To connect
to a remote gateway:

```bash
# Via flag
agentix --gateway-url https://gateway.example.com run my-agent --input "task"

# Via environment variable
export AGENTIX_GATEWAY_URL=https://gateway.example.com
agentix run my-agent --input "task"
```

## TriggerEvent Structure

The agent receives this TriggerEvent as its input:

```json
{
  "source": "cli",
  "payload": {
    "input": "analyze slow queries"
  },
  "context": {
    "invocation": "cli"
  },
  "fired_at": "2026-03-13T12:00:00Z",
  "trigger_id": "dba-optimizer-agent-trigger"
}
```

The agent's system prompt should read `payload.input` to get the user's request.

## Script Integration

```bash
#!/bin/bash
# Run agent and wait for completion
RUN_ID=$(agentix run deploy-bot --input "deploy $VERSION to staging" --output json | jq -r '.run_id')
echo "Run ID: $RUN_ID"

# Poll for completion (simple polling, production use case)
# For real integration, use the gateway SSE endpoint
```

## See Also

- [Triggers Overview](../concepts/triggers.md)
- [Agent-to-Agent Triggers](agent-to-agent-triggers.md)
- [Cron Triggers](../concepts/cron-triggers.md)
