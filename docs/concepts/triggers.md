# Triggers

Triggers are the mechanism that causes an OpenAgentiX agent to run. Instead of an agent being invoked manually every time, triggers let agents fire automatically in response to external events: a cron schedule, an incoming webhook, a GitHub pull request, a Jira issue update, a Slack mention, or a call from another agent.

## Overview

When a trigger fires, it produces a `TriggerEvent` — a structured envelope that is serialized to JSON and passed as the user input to the agent. The agent sees all the relevant context about what triggered it and what action to take.

Every triggered run is recorded in the run store with a `trigger_source` field, so you can see the full history of what fired which agent and when.

## TriggerEvent Envelope

Every trigger produces a `TriggerEvent` regardless of its type:

| Field | Type | Description |
|-------|------|-------------|
| `source` | `TriggerSource` | What fired this trigger (cron, webhook, github, etc.) |
| `payload` | `JSON` | Platform-specific data (PR object, webhook body, cron metadata) |
| `context` | `Map<String, String>` | Additional key/value metadata (repo, channel, caller agent) |
| `fired_at` | `DateTime<UTC>` | When the event was created |
| `trigger_id` | `String` | ID of the trigger instance in the `TriggerRunRegistry` |

The agent receives the `TriggerEvent` as a JSON string. For example, a GitHub trigger produces:

```json
{
  "source": "github",
  "payload": {
    "action": "opened",
    "number": 42,
    "title": "Fix authentication bug",
    "html_url": "https://github.com/org/repo/pull/42"
  },
  "context": {
    "repo": "org/repo"
  },
  "fired_at": "2026-03-13T09:00:00Z",
  "trigger_id": "github-pr-trigger"
}
```

## TriggerSource Variants

| Source | String | When it fires |
|--------|--------|---------------|
| `Cron` | `"cron"` | On a cron expression schedule (e.g., every Monday at 9 AM) |
| `Webhook` | `"webhook"` | HTTP POST to `/webhooks/:trigger_id` |
| `GitHub` | `"github"` | GitHub webhook events (pull_request, push, tag, etc.) |
| `Jira` | `"jira"` | Jira webhook events (issue_updated, comment_created, mention) |
| `Slack` | `"slack"` | @mention of the agent in a Slack channel |
| `Discord` | `"discord"` | @mention of the agent bot in Discord |
| `Telegram` | `"telegram"` | Message to the Telegram bot |
| `Agent` | `"agent"` | Another agent delegating work via the trigger API |
| `Cli` | `"cli"` | `agentix run <agent-name> --input "..."` from the command line |

## TriggerTrait

The `TriggerTrait` is the pluggable interface for all trigger implementations. To add a new trigger type, implement this trait:

```rust
use async_trait::async_trait;
use agentix_core::{AgentixError, TriggerEvent, TriggerSource, TriggerTrait};
use tokio::sync::mpsc;

pub struct MyCustomTrigger {
    id: String,
}

#[async_trait]
impl TriggerTrait for MyCustomTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        TriggerSource::Webhook  // or the appropriate source type
    }

    async fn start(
        &self,
        sender: mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        // For active triggers: spawn a background task that sends events via sender
        // For passive triggers: store sender for use when an HTTP request arrives
        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        // Clean up background tasks, clear stored sender, etc.
        Ok(())
    }
}
```

There are two patterns:

- **Active triggers** (Cron): `start()` spawns a tokio task that sleeps until the next scheduled time, sends a `TriggerEvent` via the mpsc sender, and repeats. `stop()` terminates the task via a shutdown signal.
- **Passive triggers** (Webhook, GitHub, Jira, Slack, Discord, Telegram): `start()` stores the sender internally. The gateway calls a `receive_payload()` method when an HTTP request arrives. `stop()` clears the stored sender.

## TriggerRunRegistry

The `TriggerRunRegistry` is owned by the gateway (`AgentManager`) and populated at agent load time. It holds all active `TriggerTrait` instances keyed by `trigger_id`.

```rust
use agentix_core::{TriggerRunRegistry, TriggerTrait};
use std::sync::Arc;

let mut registry = TriggerRunRegistry::new();

// Register a trigger instance
registry.register(Arc::new(my_trigger));

// Look up a trigger
if let Some(trigger) = registry.get("my-trigger-id") {
    trigger.start(sender).await?;
}

// All triggers (for starting at boot)
for trigger in registry.all() {
    trigger.start(sender.clone()).await?;
}
```

At gateway boot, the runtime:
1. Loads all agent YAML files
2. For each agent's `triggers:` field, constructs the appropriate trigger type
3. Registers it in `TriggerRunRegistry`
4. Calls `start()` on each trigger with a shared mpsc sender
5. A background dispatcher reads from the channel and calls `AgentManager::run_agent_with_trigger()`

## Agent YAML Trigger Configuration

Triggers are configured in the agent YAML file under the `triggers:` field. Each entry specifies a `type` and type-specific options:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: my-agent
  description: An agent with multiple trigger types
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    You are an automation agent. Read the trigger event from your input
    to understand what fired you and what to do.
  triggers:
    # Cron trigger: fires every Monday at 9 AM UTC
    - type: cron
      expression: "0 9 * * 1"

    # Webhook trigger: fires on HTTP POST /webhooks/my-webhook
    - type: webhook

    # GitHub trigger: fires on pull_request events
    - type: github
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events:
        - pull_request

    # Jira trigger: fires on issue updates
    - type: jira
      webhook_secret: ${JIRA_WEBHOOK_SECRET}
      events:
        - jira:issue_updated

    # Slack mention trigger
    - type: slack
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

    # Discord mention trigger
    - type: discord
      bot_token: ${DISCORD_BOT_TOKEN}
      application_id: ${DISCORD_APPLICATION_ID}

    # Telegram trigger
    - type: telegram
      bot_token: ${TELEGRAM_BOT_TOKEN}
```

## Agent Input from TriggerEvent

When a trigger fires, the `TriggerEvent` is serialized to JSON and passed as the user input to the agent's ReAct loop. The agent's system prompt should instruct it to read the trigger event from the input.

Example system prompt:

```
You are a PR review agent. When triggered, you receive a JSON TriggerEvent in
your input. Read the `payload` field to understand the GitHub event, then
review the pull request description and provide feedback.

For pull_request events with action="opened", respond with:
1. A brief summary of what the PR does
2. Any concerns or questions
3. An overall assessment (approved / needs changes / request info)
```

The agent receives input like:

```json
{
  "source": "github",
  "payload": {
    "action": "opened",
    "number": 42,
    "title": "Add OAuth2 support",
    "body": "This PR adds OAuth2 authentication...",
    "html_url": "https://github.com/org/repo/pull/42",
    "user": {"login": "developer"}
  },
  "context": {"repo": "org/repo"},
  "fired_at": "2026-03-13T09:00:00Z",
  "trigger_id": "github-pr-trigger"
}
```

## Notification Routing

After a triggered run completes, OpenAgentiX can send the result to a configured destination via `notifications:` in the agent YAML:

```yaml
spec:
  notifications:
    - type: webhook
      url: ${SLACK_WEBHOOK_URL}
```

Supported notification types in v2.0:
- `webhook` — HTTP POST with run summary to any URL (works with Slack incoming webhooks, etc.)
- `log` — Write result summary to tracing log (zero configuration)

Full platform-native notifications (Slack messages, Discord replies) are available in Phase 21 (Gateway).

## See Also

- [Cron Triggers](cron-triggers.md) — scheduling with cron expressions
- [Webhook Triggers](webhook-triggers.md) — generic HTTP webhooks
- [Run Persistence](run-persistence.md) — viewing run history with `agentix runs`
- [Triggers Quickstart](../guides/triggers-quickstart.md) — 5-minute setup guide
