# Agent-to-Agent Triggers

One agent can delegate tasks to another agent by firing it via the HTTP trigger API.
This enables multi-agent coordination where a coordinator or orchestrator agent
breaks a task into subtasks and dispatches each subtask to a specialist agent.

## Overview

When a parent agent needs to delegate work, it calls:

```
POST /api/v1/agents/:target-agent/trigger
Body: {"payload": {"task": "..."}, "caller": "parent-agent-name"}
```

The gateway creates a `TriggerEvent{source: Agent}` and fires the target agent
asynchronously. In Phase 15, this is fire-and-forget — the parent agent does not
receive the child's output directly. Full result routing (returning output back to
the caller) is planned for Phase 16 (Multi-Agent Coordination).

## Agent YAML Configuration

The target (specialist) agent declares an `agent` trigger type:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-optimizer
  description: Database optimization specialist, callable by other agents
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    You are a database optimization specialist. When called by another agent,
    you receive a TriggerEvent in your input. Read payload.task to understand
    what optimization work is needed.

    Your input will be a TriggerEvent JSON:
    - source: "agent"
    - payload: {"task": "...", ...task-specific fields}
    - context.caller_agent: name of the agent that delegated this task
  triggers:
    - type: agent
```

## TriggerEvent Payload

The delegated agent receives:

```json
{
  "source": "agent",
  "payload": {
    "task": "analyze slow queries on the payments table for the past 24 hours",
    "database": "payments_prod",
    "time_range": "24h"
  },
  "context": {
    "caller_agent": "incident-coordinator"
  },
  "fired_at": "2026-03-13T11:30:00Z",
  "trigger_id": "dba-optimizer-agent-trigger"
}
```

## How a Parent Agent Fires Another Agent

A parent agent can call another agent by making an HTTP request to the gateway.
If the parent agent has access to an HTTP tool, it can use it directly:

```yaml
# In the parent agent's system prompt:
# To delegate database optimization work:
# POST /api/v1/agents/dba-optimizer/trigger
# Body: {"payload": {"task": "..."}, "caller": "incident-coordinator"}
```

Or via the CLI (for testing):

```bash
curl -X POST http://localhost:7777/api/v1/agents/dba-optimizer/trigger \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "task": "analyze slow queries on payments table",
      "database": "payments_prod"
    },
    "caller": "incident-coordinator"
  }'
```

Response:
```json
{
  "accepted": true,
  "run_id": "run_abc123",
  "agent": "dba-optimizer",
  "trigger_id": "dba-optimizer-agent-trigger"
}
```

## Example: Coordinator Pattern

An incident coordinator receives a Slack mention and delegates to specialists:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: incident-coordinator
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    You are an incident coordinator. When triggered, assess the incident
    and delegate to specialist agents:

    For database issues: POST /api/v1/agents/dba-optimizer/trigger
    For infrastructure issues: POST /api/v1/agents/infra-helper/trigger
    For security issues: POST /api/v1/agents/security-scanner/trigger

    Delegate by providing a structured payload with the task description
    and relevant context.
  triggers:
    - type: slack
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
  tools:
    - type: http
      name: trigger_agent
      url: http://localhost:7777/api/v1/agents/{agent}/trigger
```

## Limitations in Phase 15

- **Fire-and-forget**: The parent agent does not receive the child's output
- **No chaining**: The child's result is not automatically returned to the parent
- **No parallel dispatch**: Tools must be called sequentially

These limitations are addressed in Phase 16 (Multi-Agent Coordination) which adds:
- Result routing (child output returned to parent)
- Parallel agent fan-out
- Shared context between agents

## See Also

- [Triggers Overview](../concepts/triggers.md)
- [CLI Triggers](cli-triggers.md)
- [Slack Mention Trigger](slack-mention-trigger.md)
