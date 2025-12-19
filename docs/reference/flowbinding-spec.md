# FlowBinding Resource Reference

Complete reference for FlowBinding resource specifications. FlowBindings tie together Triggers, Contexts, and Flows to create complete routing configurations.

## Overview

A FlowBinding is the composition layer that connects:
- **Trigger** - Where messages come from (Slack, Telegram, etc.)
- **Context** - Execution environment (prod, staging, customer-X)
- **Flow/Agent/Fleet** - What handles the message

This enables:
- Decoupled architecture (define once, compose with bindings)
- Multi-tenant deployments (same flow, different contexts)
- Flexible routing (multiple bindings per trigger)

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value

spec:
  trigger: string           # Required: Reference to Trigger
  context: string           # Optional: Reference to Context
  flow: string              # Required*: Reference to AgentFlow
  agent: string             # Required*: Reference to Agent (simple)
  fleet: string             # Optional: Reference to Fleet
  match:                    # Optional: Additional routing rules
    patterns: [string]
    priority: int
  enabled: bool             # Optional: Enable/disable binding
```

*One of `flow`, `agent`, or `fleet` is required.

---

## Spec Fields

### Required References

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `trigger` | string | Yes | Name of Trigger resource |
| `flow` | string | Yes* | Name of AgentFlow resource |
| `agent` | string | Yes* | Name of Agent resource (simple flows) |
| `fleet` | string | No | Name of Fleet resource |
| `context` | string | No | Name of Context resource |

*One of `flow` or `agent` is required.

### Match Configuration

Additional routing rules applied after trigger matching.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `patterns` | array | [] | Message patterns (regex) |
| `channels` | array | [] | Override trigger channel filter |
| `users` | array | [] | Override trigger user filter |
| `events` | array | [] | Override trigger event filter |
| `priority` | int | 0 | Routing priority (higher = first) |
| `required_keywords` | array | [] | All must be present |
| `excluded_keywords` | array | [] | None may be present |

### Enable/Disable

```yaml
spec:
  enabled: false  # Disable this binding (useful for maintenance)
```

---

## Routing Logic

### How Bindings Are Matched

1. **Trigger filter** - Platform, channel, user, patterns
2. **Binding filter** - Additional patterns, keywords
3. **Score calculation** - Priority + specificity
4. **Best match wins** - Highest score routes the message

### Match Scoring

| Criterion | Score |
|-----------|-------|
| Binding priority | +priority value |
| Channel specificity | +100 |
| User specificity | +80 |
| Pattern match | +60 |
| Required keywords | +40 per keyword |
| Base binding score | +10 |

**Example:** A binding with `priority: 50`, channel filter, and 2 required keywords scores: `50 + 100 + 80 = 230`

---

## Pattern Matching

### Regex Patterns

```yaml
spec:
  match:
    patterns:
      - "^kubectl"           # Starts with kubectl
      - "(?i)deploy"         # Case-insensitive "deploy"
      - "pr/d+"             # PR followed by digits
```

### Keyword Filtering

```yaml
spec:
  match:
    required_keywords:
      - pod                  # Message must contain "pod"
      - production          # AND "production"
    excluded_keywords:
      - delete              # Message must NOT contain "delete"
      - force
```

---

## Complete Examples

### Simple Agent Binding

Route Slack messages directly to an agent:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-k8s-simple
spec:
  trigger: slack-prod
  context: prod
  agent: k8s-ops           # Direct to agent (no flow)
  match:
    patterns:
      - "kubectl"
      - "k8s"
```

### Flow-Based Binding

Route to a multi-step AgentFlow:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: pr-review-binding
spec:
  trigger: slack-engineering
  context: dev
  flow: pr-review-flow      # Multi-step workflow
  match:
    patterns:
      - "review PR"
      - "PR review"
    priority: 100
```

### Fleet Binding

Route to a team of agents:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: rca-fleet-binding
spec:
  trigger: pagerduty-incidents
  context: prod
  fleet: rca-fleet         # Multi-agent team
  flow: incident-triage-flow
```

### Multi-Tenant Bindings

Same agent, different contexts:

```yaml
# Customer A binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: customer-a-k8s
  labels:
    tenant: customer-a
spec:
  trigger: slack-customer-a
  context: customer-a-prod   # Customer A's environment
  agent: k8s-ops             # Same agent!
  match:
    patterns: ["kubectl", "k8s"]

---
# Customer B binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: customer-b-k8s
  labels:
    tenant: customer-b
spec:
  trigger: slack-customer-b
  context: customer-b-prod   # Customer B's environment
  agent: k8s-ops             # Same agent!
  match:
    patterns: ["kubectl", "k8s"]
```

### Priority-Based Routing

Multiple bindings with different priorities:

```yaml
# High-priority: Security-related commands
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: security-commands
spec:
  trigger: slack-prod
  context: prod
  agent: security-agent
  match:
    patterns:
      - "secret"
      - "credential"
      - "auth"
    priority: 200            # Highest priority

---
# Medium-priority: K8s commands
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: k8s-commands
spec:
  trigger: slack-prod
  context: prod
  agent: k8s-ops
  match:
    patterns:
      - "kubectl"
      - "k8s"
    priority: 100

---
# Low-priority: Catch-all
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: default-handler
spec:
  trigger: slack-prod
  context: prod
  agent: general-assistant
  match:
    priority: 0              # Lowest priority (catch-all)
```

### Read-Only Binding

For platforms like Telegram where you want safety:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: telegram-readonly
spec:
  trigger: telegram-oncall
  context: prod-readonly     # Context with read-only policies
  agent: k8s-ops
  match:
    excluded_keywords:       # Extra safety at binding level
      - delete
      - scale
      - restart
```

### Disabled Binding (Maintenance)

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-binding
  annotations:
    maintenance-reason: "Upgrading to new flow"
spec:
  trigger: slack-prod
  context: prod
  flow: k8s-flow
  enabled: false             # Temporarily disabled
```

---

## Configuration Overrides

Bindings can override values via `config`:

```yaml
spec:
  trigger: slack-prod
  flow: k8s-flow
  config:
    timeout_seconds: 600     # Override flow timeout
    max_retries: 5           # Override retry count
    custom_param: value      # Custom config passed to flow
```

---

## Validation

```bash
# Validate binding YAML
aofctl validate -f binding.yaml

# List loaded bindings
aofctl get bindings

# Describe specific binding
aofctl describe binding prod-k8s
```

### Validation Rules
- Name is required
- Trigger reference is required
- Must have at least one of: `flow`, `agent`, or `fleet`
- Referenced resources must exist (validated at runtime)
- Patterns must be valid regex

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ Incoming Message                                            │
│ Platform: slack, Channel: production, User: U123            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ BindingRouter                                               │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 1. Find triggers matching platform                      │ │
│ │ 2. Filter by trigger config (channels, users, patterns) │ │
│ │ 3. Find bindings referencing matched triggers           │ │
│ │ 4. Apply binding filters (match config)                 │ │
│ │ 5. Calculate scores, select highest                     │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ BindingMatch                                                │
│ ├─ Binding: prod-k8s-binding                               │
│ ├─ Trigger: slack-prod                                     │
│ ├─ Context: prod (injected at runtime)                     │
│ ├─ Flow: k8s-ops-flow                                      │
│ └─ Score: 210                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Execute Flow with Context                                   │
│ - Environment variables from Context                        │
│ - Kubeconfig from Context                                   │
│ - Approval rules from Context                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Best Practices

### 1. Use Descriptive Names

```yaml
# Good
metadata:
  name: prod-k8s-slack-binding

# Avoid
metadata:
  name: binding-1
```

### 2. Use Labels for Organization

```yaml
metadata:
  labels:
    environment: production
    team: platform
    tenant: acme
```

### 3. Set Appropriate Priorities

- **200+**: Security-sensitive commands
- **100-199**: Specific tools (kubectl, helm)
- **50-99**: General handlers
- **0-49**: Catch-all defaults

### 4. Use Required/Excluded Keywords for Safety

```yaml
spec:
  match:
    excluded_keywords:
      - "production"   # Prevent accidental prod commands
      - "delete"
      - "--force"
```

### 5. Document with Annotations

```yaml
metadata:
  annotations:
    description: "Routes K8s commands from production Slack"
    owner: platform-team@company.com
    created: "2024-01-15"
```

---

## See Also

- [Trigger Reference](./trigger-spec.md) - Message source configuration
- [Context Reference](./context-spec.md) - Execution environment configuration
- [AgentFlow Reference](./agentflow-spec.md) - Workflow orchestration
- [Fleet Reference](./fleet-spec.md) - Multi-agent teams
- [Resource Selection Guide](../concepts/resource-selection.md) - When to use what
