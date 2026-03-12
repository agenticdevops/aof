# Cron Triggers

Cron triggers fire an agent automatically on a recurring schedule defined by a cron expression. Use cron triggers for agents that perform periodic operational tasks: weekly reports, daily health checks, nightly cleanups, and similar scheduled work.

## Overview

When a cron trigger fires, the agent receives a `TriggerEvent` with:
- `source: "cron"` — identifies the trigger type
- `payload.expression` — the cron expression that was scheduled
- `payload.scheduled_at` — ISO 8601 timestamp of when the trigger was scheduled to fire

The agent runs as a normal autonomous agent with the TriggerEvent as its input.

## Agent YAML Configuration

Add a `triggers:` entry with `type: cron` and a `expression` field:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: weekly-reporter
  description: Generates a weekly operational summary every Monday at 9 AM UTC
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    You are a weekly operations reporter. When triggered by the cron scheduler,
    you receive a TriggerEvent as JSON in your input. The `payload.scheduled_at`
    field tells you when you were scheduled to run.

    Generate a concise operational summary covering:
    - System health status
    - Any recent incidents or alerts
    - Action items for the week

    Keep the summary under 500 words.
  triggers:
    - type: cron
      expression: "0 9 * * 1"
  notifications:
    - type: webhook
      url: ${SLACK_WEBHOOK_URL}
```

## Cron Expression Format

OpenAgentiX uses standard 5-field cron expressions: `min hour dom month dow`

| Field | Range | Description |
|-------|-------|-------------|
| `min` | 0-59 | Minute |
| `hour` | 0-23 | Hour (UTC) |
| `dom` | 1-31 | Day of month |
| `month` | 1-12 | Month |
| `dow` | 0-7 | Day of week (0 and 7 are Sunday) |

Common expression examples:

| Expression | Fires |
|------------|-------|
| `0 9 * * 1` | Every Monday at 9:00 AM UTC |
| `0 0 * * *` | Every day at midnight UTC |
| `0 */4 * * *` | Every 4 hours |
| `30 8 * * 1-5` | Weekdays at 8:30 AM UTC |
| `0 9 1 * *` | First day of every month at 9 AM UTC |
| `*/15 * * * *` | Every 15 minutes |

All times are UTC. Timezone support is planned for a future release.

## TriggerEvent Payload

When a cron trigger fires, the agent receives a TriggerEvent with this structure:

```json
{
  "source": "cron",
  "payload": {
    "expression": "0 9 * * 1",
    "scheduled_at": "2026-03-16T09:00:00Z"
  },
  "context": {},
  "fired_at": "2026-03-16T09:00:00.123Z",
  "trigger_id": "weekly-reporter-cron-0"
}
```

The `trigger_id` is auto-generated as `{agent_name}-{trigger_type}-{index}`.

## Timezone Handling

All cron expressions are evaluated in UTC. There is no per-trigger timezone configuration in v2.0.

To fire at a local time, convert to UTC manually. For example, "9 AM Eastern Standard Time (UTC-5)" becomes `0 14 * * 1` in UTC.

## Example: Weekly Standup Agent

This agent aggregates standups from a team and sends a summary every Monday morning:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: standup-aggregator
  description: Collects and summarizes team standups every Monday
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    You are a standup coordinator. When triggered on Monday mornings,
    use your tools to check the team standup channel for updates from
    the past week and generate a concise summary for leadership.

    Your input contains a TriggerEvent JSON — check payload.scheduled_at
    to confirm the date range you should cover (past 7 days).
  triggers:
    - type: cron
      expression: "0 9 * * 1"
  notifications:
    - type: webhook
      url: ${LEADERSHIP_WEBHOOK_URL}
  mode: autonomous
  max_iterations: 5
```

## Verification

After starting the gateway, verify a cron trigger is registered:

```bash
agentix agents get weekly-reporter
```

To see runs fired by cron triggers:

```bash
agentix runs --agent weekly-reporter
```

The `TRIGGER` column shows `cron` for scheduled runs.

To manually trigger a cron agent for testing (without waiting for the schedule):

```bash
agentix run weekly-reporter --input '{"source":"cron","payload":{"expression":"0 9 * * 1","scheduled_at":"2026-03-16T09:00:00Z"},"context":{},"fired_at":"2026-03-16T09:00:00Z","trigger_id":"test"}'
```

## See Also

- [Triggers Overview](triggers.md) — all trigger types and the TriggerEvent model
- [Run Persistence](run-persistence.md) — viewing run history
- [Triggers Quickstart](../guides/triggers-quickstart.md) — step-by-step setup guide
