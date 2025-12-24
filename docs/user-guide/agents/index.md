---
id: index
title: Pre-Built Agent Library
sidebar_label: Overview
format: md
---

# Pre-Built Agent Library

AOF includes a curated library of pre-built, production-ready agents for common operational tasks. These agents are designed to work out-of-the-box or serve as starting points for customization.

## What Are Pre-Built Agents?

Pre-built agents are ready-to-use YAML configurations that encapsulate best practices for specific operational domains. Each agent comes with:

- **Optimized system prompts** - Carefully crafted instructions for consistent behavior
- **Tool configurations** - Pre-configured access to observability and orchestration tools
- **Memory management** - Built-in context persistence for learning and continuity
- **Integration patterns** - Ready to connect with PagerDuty, Slack, Kubernetes, and more

## Agent Categories

### 🚨 Incident Management
Agents for incident response, alert correlation, root cause analysis, and postmortem generation.

- [Incident Management Agents](./incident-management)

### 📊 Observability (Coming in v0.4.0)
Agents for metrics analysis, log investigation, and performance troubleshooting.

### 🔄 CI/CD (Coming in v0.4.0)
Agents for deployment automation, test execution, and release management.

### 🔒 Security (Coming in v0.4.0)
Agents for vulnerability scanning, compliance checks, and security incident response.

### ☁️ Cloud Operations (Coming in v0.4.0)
Agents for cloud cost optimization, infrastructure provisioning, and resource management.

## How to Use Pre-Built Agents

### 1. Direct Execution

Run an agent directly from the library with your custom input:

```bash
# Run the incident responder agent
aofctl run agent library/incident/incident-responder "Triage the API outage in production"

# Run the alert analyzer agent
aofctl run agent library/incident/alert-analyzer "Analyze alerts from the last hour"
```

### 2. Reference in Triggers

Reference library agents in your trigger configurations:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  source:
    type: webhook
    config:
      path: /webhooks/pagerduty
      port: 8080

  actions:
    - type: agent
      # Reference the library agent
      ref: library/incident/incident-responder.yaml
      input: "{{ .payload.incident.summary }}"
```

### 3. Reference in Fleets

Coordinate multiple library agents in a fleet:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: incident-response-team
spec:
  agents:
    - name: triager
      ref: library/incident/incident-responder.yaml

    - name: investigator
      ref: library/incident/rca-investigator.yaml

    - name: documenter
      ref: library/incident/postmortem-writer.yaml

  workflow:
    - step: triage
      agent: triager
      input: "{{ .trigger.data.incident }}"

    - step: investigate
      agent: investigator
      input: "Deep dive into: {{ .steps.triage.output }}"
      condition: "{{ .steps.triage.severity | eq 'P0' or eq 'P1' }}"

    - step: document
      agent: documenter
      input: "{{ .steps.investigate.output }}"
```

### 4. Copy and Customize

Copy a library agent to your local configuration and customize it:

```bash
# Copy the agent to your config directory
cp examples/agents/library/incident/incident-responder.yaml \
   ~/.aof/agents/my-incident-responder.yaml

# Edit to customize system prompt, tools, or environment variables
vim ~/.aof/agents/my-incident-responder.yaml

# Run your customized version
aofctl run agent ~/.aof/agents/my-incident-responder.yaml "Custom input"
```

## Customization Guide

### Model Selection

Override the default model based on your requirements:

```yaml
spec:
  # Faster, cheaper model for simple tasks
  model: google:gemini-2.5-flash

  # Or use more capable model for complex reasoning
  model: anthropic:claude-3-7-sonnet-20250219
```

### Tool Configuration

Add or remove tools based on your infrastructure:

```yaml
spec:
  tools:
    # Standard tools
    - kubectl
    - prometheus_query

    # Add custom tools
    - datadog_metric_query
    - custom_runbook_search
```

### System Prompt Tuning

Customize the system prompt for your environment:

```yaml
spec:
  system_prompt: |
    You are an incident responder for Acme Corp.

    ## Custom Guidelines
    - Always check our custom dashboard: https://grafana.acme.com/incident
    - Escalate P0/P1 to #incidents-critical Slack channel
    - Follow our runbook library: https://runbooks.acme.com

    {{ include "library/incident/incident-responder.yaml" }}
```

### Environment Variables

Configure integration endpoints:

```yaml
spec:
  env:
    PROMETHEUS_URL: "https://prometheus.acme.com"
    LOKI_URL: "https://loki.acme.com"
    GRAFANA_URL: "https://grafana.acme.com"
    SLACK_WEBHOOK: "${SLACK_INCIDENT_WEBHOOK}"
    PAGERDUTY_API_KEY: "${PAGERDUTY_API_KEY}"
```

### Memory Configuration

Adjust memory settings for context persistence:

```yaml
spec:
  # Increase memory for long-running investigations
  memory: "File:./incident-memory.json:500"
  max_context_messages: 100

  # Or use SQLite for advanced querying
  memory: "SQLite:./incident-memory.db"
```

## Best Practices

### 1. Start with Defaults

Use library agents as-is initially to understand their behavior before customizing.

### 2. Version Control Your Customizations

Store your customized agents in version control:

```bash
# Your repository structure
.aof/
├── agents/
│   ├── incident/
│   │   ├── custom-responder.yaml
│   │   └── custom-rca.yaml
│   └── observability/
│       └── custom-log-analyzer.yaml
├── triggers/
│   └── pagerduty.yaml
└── fleets/
    └── incident-response.yaml
```

### 3. Use Environment Variables for Secrets

Never hardcode secrets in agent configurations:

```yaml
spec:
  env:
    # ✅ Good: Use environment variables
    API_KEY: "${PAGERDUTY_API_KEY}"

    # ❌ Bad: Hardcoded secrets
    # API_KEY: "sk-1234567890"
```

### 4. Test Before Production

Test customized agents in a non-production environment:

```bash
# Test with a sample incident
aofctl run agent ~/.aof/agents/custom-responder.yaml \
  "Test incident: API returning 500 errors" \
  --env-file .env.staging
```

### 5. Monitor Agent Performance

Track agent execution metrics:

```bash
# View agent execution history
aofctl get executions --agent incident-responder --limit 10

# Analyze token usage
aofctl analyze usage --agent incident-responder --timeframe 7d
```

## Integration Patterns

### PagerDuty Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  source:
    type: webhook
    config:
      path: /webhooks/pagerduty
      port: 8080

      # Verify PagerDuty signatures
      signature_header: X-PagerDuty-Signature
      signature_secret: "${PAGERDUTY_WEBHOOK_SECRET}"

  filter:
    # Only trigger on new incidents
    expression: .payload.event == "incident.triggered"

  actions:
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: |
        Incident from PagerDuty:
        - ID: {{ .payload.incident.id }}
        - Summary: {{ .payload.incident.summary }}
        - Service: {{ .payload.incident.service.name }}
        - Urgency: {{ .payload.incident.urgency }}
```

### Slack Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: slack-commands
spec:
  source:
    type: slack_command
    config:
      command: /triage
      signing_secret: "${SLACK_SIGNING_SECRET}"

  actions:
    - type: agent
      ref: library/incident/incident-responder.yaml
      input: "{{ .command.text }}"

      # Post response back to Slack
      output:
        type: slack_message
        channel: "{{ .command.channel_id }}"
```

### Scheduled Execution

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: alert-analysis-cron
spec:
  source:
    type: schedule
    config:
      # Run every 5 minutes
      cron: "*/5 * * * *"

  actions:
    - type: agent
      ref: library/incident/alert-analyzer.yaml
      input: "Analyze alerts from the last 5 minutes"
```

## Next Steps

- [Incident Management Agents](./incident-management) - Deep dive into the incident management agent library
- [Core Concepts](/docs/concepts) - Learn fundamental AOF concepts
- [Fleet Orchestration](/docs/concepts/fleets) - Coordinate multiple agents
- [Agent Specification](/docs/reference/agent-spec) - Agent configuration reference

## Community Contributions

The agent library grows through community contributions. To contribute a new pre-built agent:

1. Create your agent in `examples/agents/library/<category>/`
2. Submit a pull request with documentation

See the [GitHub Repository](https://github.com/agenticdevops/aof) for contribution guidelines.
