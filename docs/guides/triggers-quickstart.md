# Triggers Quickstart

This guide walks you through setting up the most common trigger patterns in OpenAgentiX. By the end you will have working examples of cron scheduling, webhook integration, GitHub PR automation, and CLI-based agent invocation.

## Prerequisites

- OpenAgentiX installed (`agentix --version`)
- A running gateway (`agentix serve --config serve-config.yaml`)
- An Anthropic or other supported LLM API key in your environment

---

## 1. Cron trigger — weekly report (5 minutes)

A cron-triggered agent runs automatically on a schedule with no external request needed.

### Step 1: Create the agent YAML

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: weekly-reporter
  description: Sends a weekly operations summary every Monday at 9 AM UTC
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    Generate a concise weekly operational summary. Include system health,
    recent incidents, and action items for the week ahead.
  triggers:
    - type: cron
      expression: "0 9 * * 1"
  notifications:
    - type: webhook
      url: ${SLACK_WEBHOOK_URL}
  max_iterations: 3
```

### Step 2: Register and verify

```bash
agentix apply quickstart/agents/scheduled-reporter.yaml
agentix get agent weekly-reporter
```

The agent will fire automatically every Monday at 09:00 UTC. Check run history:

```bash
agentix runs --agent weekly-reporter
```

---

## 2. Webhook trigger (5 minutes)

A webhook-triggered agent fires when an HTTP POST arrives at its configured path.

### Step 1: Create the agent YAML

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: deploy-watcher
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    You receive deployment event notifications. Summarize what happened
    and flag any anomalies (unexpected services, unusual timing, errors).
  triggers:
    - type: webhook
      path: /webhooks/deploy-watcher
  max_iterations: 3
```

### Step 2: Register the agent

```bash
agentix apply deploy-watcher.yaml
```

### Step 3: Send a test webhook

```bash
curl -X POST http://localhost:7777/webhooks/deploy-watcher \
  -H "Content-Type: application/json" \
  -d '{"event": "deploy.completed", "service": "api-server", "environment": "production"}'
```

The agent receives the payload as its context and runs immediately.

### Optional: Add HMAC signature verification

```yaml
triggers:
  - type: webhook
    path: /webhooks/deploy-watcher
    secret: ${WEBHOOK_SECRET}
```

Callers must then include an `x-hub-signature-256` header with `sha256=<HMAC>`.

---

## 3. GitHub PR review trigger (10 minutes)

### Step 1: Create the agent

Use `quickstart/agents/github-pr-reviewer.yaml` as a starting point:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: github-pr-reviewer
spec:
  model: anthropic/claude-3-5-sonnet-20241022
  system_prompt: |
    Review the GitHub pull request payload and provide constructive feedback
    on the description, risks, and action items.
  triggers:
    - type: github
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events:
        - pull_request
  max_iterations: 5
```

### Step 2: Register the agent and note the trigger URL

```bash
agentix apply github-pr-reviewer.yaml
agentix get agent github-pr-reviewer
```

The webhook URL will be `/webhooks/github-pr-reviewer-github-0`.

### Step 3: Configure GitHub

1. Go to your GitHub repository: **Settings > Webhooks > Add webhook**
2. Set **Payload URL** to `https://your-domain/webhooks/github-pr-reviewer-github-0`
3. Set **Content type** to `application/json`
4. Set **Secret** to the value of `GITHUB_WEBHOOK_SECRET`
5. Select **Pull requests** event (or choose individual events)

### Step 4: Test with a real PR or ngrok

For local development, expose your gateway with ngrok:

```bash
ngrok http 7777
# Use the https://xxxx.ngrok.io URL as your GitHub webhook Payload URL
```

Open a pull request — the agent runs within seconds.

---

## 4. CLI trigger (2 minutes)

Fire any agent manually from the command line for one-off tasks:

```bash
# Basic invocation
agentix run my-agent --input "Summarize the last 24 hours of alerts"

# JSON output for scripting
agentix run my-agent --input "Check disk usage" --output json
```

The `agentix run` command calls `POST /api/v1/agents/:name/trigger` and returns the run ID immediately. The agent runs asynchronously.

---

## 5. Viewing trigger history

After any trigger fires, check the run log:

```bash
# All recent runs across all agents
agentix runs

# Filter to a specific agent
agentix runs --agent github-pr-reviewer

# Show more results
agentix runs --limit 50

# JSON for scripting
agentix runs --output json
```

The text output shows trigger source:

```
RUN ID        AGENT                 TRIGGER     STATUS        STARTED               ITERATIONS
a1b2c3d4e5f6  github-pr-reviewer    github      completed     2026-03-13 09:15      3
77a6b5c4d3e2  weekly-reporter       cron        completed     2026-03-13 09:00      2
```

---

## 6. Next steps

- [Cron trigger reference](../concepts/cron-triggers.md)
- [Webhook trigger reference](../concepts/webhook-triggers.md)
- [GitHub integration guide](./github-integration.md)
- [Jira integration guide](./jira-integration.md)
- [Slack mention trigger](./slack-mention-trigger.md)
- [Discord mention trigger](./discord-mention-trigger.md)
- [Telegram mention trigger](./telegram-mention-trigger.md)
- [Run persistence and history](../concepts/run-persistence.md)
- [Agent-to-agent triggers](./agent-to-agent-triggers.md)
