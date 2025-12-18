# Bindings

Bindings **tie everything together**: trigger + flow + context = complete working bot.

## Available Bindings

- **`prod-slack-k8s.yaml`**
  - Trigger: Slack #production
  - Flow: K8s operations
  - Context: Production cluster
  - **Result**: Production K8s bot with strict approvals

- **`staging-slack-k8s.yaml`**
  - Trigger: Slack #staging
  - Flow: K8s operations (same as prod!)
  - Context: Staging cluster
  - **Result**: Staging K8s bot with relaxed approvals

- **`oncall-telegram-incident.yaml`**
  - Trigger: Telegram on-call group
  - Flow: Incident response
  - Context: Production cluster
  - **Result**: On-call incident bot with fast response

- **`pagerduty-incident.yaml`**
  - Trigger: PagerDuty webhooks
  - Flow: Incident response (same as Telegram!)
  - Context: Production cluster
  - **Result**: Automated incident response

## Binding Structure

\`\`\`yaml
apiVersion: aof.dev/v1
kind: Binding
metadata:
  name: prod-slack-k8s

spec:
  # Where messages come from
  trigger:
    ref: triggers/slack-prod.yaml

  # What workflow to run
  flow:
    ref: flows/k8s-ops-flow.yaml

  # What environment to use
  context:
    ref: contexts/prod.yaml

  # Optional: Override specific settings
  overrides:
    approval:
      timeout_seconds: 600
\`\`\`

## Multi-Tenant Architecture

The power of bindings is **reusability**:

\`\`\`
Same Flow, Different Environments:
┌──────────────┐
│ k8s-ops-flow │ (defined once)
└──────┬───────┘
       ├─→ prod-slack-k8s (strict)
       ├─→ staging-slack-k8s (relaxed)
       └─→ dev-slack-k8s (permissive)

Same Flow, Different Platforms:
┌────────────────┐
│ incident-flow  │ (defined once)
└────────┬───────┘
         ├─→ oncall-telegram (interactive)
         └─→ pagerduty-auto (automated)
\`\`\`

## Usage

Bindings are loaded in `daemon-config.yaml`:

\`\`\`yaml
spec:
  bindings:
    - bindings/prod-slack-k8s.yaml
    - bindings/staging-slack-k8s.yaml
    - bindings/oncall-telegram-incident.yaml
    - bindings/pagerduty-incident.yaml
\`\`\`

## Best Practices

1. **One binding per environment-platform combo**
2. **Use ref: syntax** - No inline definitions
3. **Override sparingly** - Prefer context configuration
4. **Name clearly** - `{env}-{platform}-{capability}`
5. **Document behavior** - Explain what the binding does
