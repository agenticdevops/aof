# Coordinator Agents

## Overview

A coordinator agent routes tasks to specialist agents and synthesizes their results. It does not execute domain-specific work itself — it delegates sub-tasks via the delegation API and assembles the final response.

This pattern separates concerns:
- **Coordinator**: understands the big picture, routes work, synthesizes results
- **Specialists**: focused domain agents that execute specific tasks

## How Delegation Works

A coordinator agent calls `POST /api/v1/agents/:specialist-name/delegate` with:

```json
{
  "from_agent": "ops-coordinator",
  "task": "Analyze replication lag on pg-primary and recommend tuning",
  "payload": {
    "host": "pg-primary",
    "threshold_ms": 10
  }
}
```

The gateway fires the specialist agent synchronously and returns the result:

```json
{
  "delegation_id": "uuid-...",
  "output": "Replication lag is 2ms — healthy. No tuning needed.",
  "status": "success",
  "from_agent": "dba-specialist",
  "completed_at": "2026-03-13T10:30:00Z"
}
```

## Coordinator Agent YAML

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: ops-coordinator
  description: "Coordinates operational tasks across specialist agents"
spec:
  model: anthropic/claude-3-5-haiku
  system_prompt: |
    You are an ops coordinator. When given an operational task, analyze it
    and delegate sub-tasks to the appropriate specialists using the delegate tools.

    After specialists complete their work, synthesize their results into a clear
    executive summary with specific findings and recommended actions.

    Available specialists:
    - dba-specialist: PostgreSQL analysis and tuning
    - infra-specialist: infrastructure health and capacity

  tools:
    - type: http
      name: delegate_to_dba
      description: "Delegate a database task to the dba-specialist agent"
      url: http://localhost:7777/api/v1/agents/dba-specialist/delegate
      method: POST
      schema:
        input:
          type: object
          required: [from_agent, task]
          properties:
            from_agent:
              type: string
            task:
              type: string
            payload:
              type: object

    - type: http
      name: delegate_to_infra
      description: "Delegate an infrastructure task to the infra-specialist agent"
      url: http://localhost:7777/api/v1/agents/infra-specialist/delegate
      method: POST
      schema:
        input:
          type: object
          required: [from_agent, task]
          properties:
            from_agent:
              type: string
            task:
              type: string
            payload:
              type: object
```

## Specialist Agent YAML

Specialist agents accept delegated tasks via the `agent` trigger type:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-specialist
  description: "PostgreSQL database specialist"
spec:
  model: anthropic/claude-3-5-haiku
  system_prompt: |
    You are a PostgreSQL specialist. Analyze the provided task and payload,
    then provide a concise technical assessment with specific recommendations.

  triggers:
    - type: agent
      id: agent-trigger-dba

  memory:
    enabled: true
    backend: sqlite
    db_path: ./memory/dba-specialist.db
    top_k: 5
```

## Parallel Fan-Out

A coordinator can delegate to multiple specialists simultaneously by calling both delegate tools in the same ReAct step. The LLM will issue parallel tool calls, and the gateway will execute them concurrently via `delegate_parallel()`.

Example coordinator reasoning:
```
I need to check both database and infrastructure health.
I'll delegate both tasks simultaneously:
- delegate_to_dba: check pg replication lag
- delegate_to_infra: check disk usage on all nodes
```

## Audit Trail

Every delegation creates a run record in the audit trail:

```bash
agentix runs
```

Output includes delegations with their `trigger_source: agent` column, and the `delegation_id` links child runs back to the coordinator's delegation.

The audit trail satisfies COORD-05: full parent/child context is preserved.

## Best Practices

1. **Keep specialists focused** — each specialist should handle one domain well
2. **Coordinator handles synthesis only** — do not add domain logic to the coordinator's system prompt
3. **Avoid circular delegation** — agent A should not delegate back to an agent that delegated to A
4. **Use memory on specialists** — specialists benefit from memory of past analyses; coordinators rarely do
5. **Set reasonable timeouts** — the delegation endpoint waits up to 60 seconds for the specialist to respond

## Error Handling

When a specialist fails (times out or returns an error), the delegation endpoint returns:

```json
{
  "delegation_id": "uuid-...",
  "output": "Agent 'dba-specialist' did not produce output",
  "status": "failure",
  "from_agent": "dba-specialist",
  "completed_at": "..."
}
```

The coordinator receives this as a tool result and can decide to retry, fall back, or report the failure to the user.

## REST API

```
POST /api/v1/agents/:name/delegate
  Body: { "from_agent": "...", "task": "...", "payload": {} }
  Response: { "delegation_id": "...", "output": "...", "status": "success|failure", ... }

  404: Agent not found
  503: Agent inbox full or unavailable
```
