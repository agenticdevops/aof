# Approval Workflows

OpenAgentiX supports human-in-the-loop approval workflows, allowing organizations to maintain control over agent actions in production environments. Agents can operate in three autonomy modes, with configurable approval gates for specific tools or action patterns.

## Autonomy Modes

Every agent operates in one of three autonomy modes, controlled by the `mode` field in the agent configuration:

| Mode | Behavior |
|------|----------|
| `autonomous` | Agent executes all actions without human approval. **Default.** |
| `semi-autonomous` | Agent pauses only for actions matching `flagged_tools` or `flagged_patterns`. Other actions proceed automatically. |
| `manual` | Agent requests approval for **every** tool call. |

### Example: Configuring Autonomy Mode

```yaml
# agents/dba-optimizer/agent.yaml
name: dba-optimizer
model:
  preferred: anthropic/claude-sonnet-4-6
mode: semi-autonomous
approval:
  flagged_tools:
    - kubectl
    - aws
  flagged_patterns:
    - delete
    - drop
    - terminate
  approvers:
    - "slack:U015ADMIN"
    - "email:sre-lead@company.com"
  timeout_seconds: 300
```

## Approval Policy

The `approval` block in `agent.yaml` configures when the agent pauses for human review.

### `flagged_tools`

A list of tool names that always require approval in `semi-autonomous` mode. The tool name must match exactly (case-sensitive).

```yaml
approval:
  flagged_tools:
    - kubectl        # Matches tool named exactly "kubectl"
    - aws            # Matches tool named exactly "aws"
    - psql           # Matches tool named exactly "psql"
```

### `flagged_patterns`

Substrings (case-insensitive) matched against the serialized tool input JSON. Any match triggers an approval gate in `semi-autonomous` mode.

```yaml
approval:
  flagged_patterns:
    - delete         # Matches any input containing "delete" (e.g., "kubectl delete pod")
    - drop           # Matches "DROP TABLE users" (case-insensitive)
    - terminate      # Matches "terminate instances"
    - --force        # Matches inputs with --force flag
```

### `approvers`

A list of authorized approver identities. Supported formats:

- `"slack:UXXXXXXXXX"` — Slack user ID (notifications via Slack integration)
- `"email:user@company.com"` — Email address
- `"slack:#channel-name"` — Post to a Slack channel

```yaml
approval:
  approvers:
    - "slack:U015ADMIN"
    - "email:sre-lead@company.com"
    - "slack:#ops-approvals"
```

### `timeout_seconds`

How long (in seconds) before an unanswered approval request expires. Expired requests are treated as denied. Default: `300` (5 minutes).

```yaml
approval:
  timeout_seconds: 300
```

### Full Configuration Example

```yaml
approval:
  mode: semi-autonomous
  flagged_tools:
    - kubectl
    - aws
  flagged_patterns:
    - delete
    - drop
    - terminate
  approvers:
    - "slack:U015ADMIN"
    - "email:sre-lead@company.com"
  timeout_seconds: 300
```

## Approval Flow

The complete lifecycle of an approval request:

1. **Encounter** — During the ReAct loop, the agent is about to call a flagged tool or one whose input matches a flagged pattern.
2. **Pause** — The agent pauses execution and creates an `ApprovalRequest` in `Pending` state.
3. **Notify** — Configured approvers are notified (via Slack, email, or CLI polling).
4. **Review** — An approver reviews the action description and tool input.
5. **Decide** — The approver runs `agentix approve <run-id>` or `agentix deny <run-id> --reason "..."`.
6. **Resume** — On approval, the agent executes the tool and continues reasoning. On denial, the agent receives a denial observation and reasons about alternatives. On timeout, the request expires and the agent receives a timeout observation.
7. **Audit** — All decisions are logged in the audit trail.

### Status Transitions

```
Pending ──┬── Approved { approver, decided_at } ──> Tool executes
           ├── Denied { approver, reason, decided_at } ──> Error observation
           └── TimedOut { expired_at } ──> Timeout observation
```

## CLI Commands

The `agentix` CLI provides full approval workflow management:

```bash
# List all pending approval requests
agentix approvals

# List pending approvals for a specific agent
agentix approvals --agent dba-optimizer

# Approve a pending request
agentix approve <request-id>
agentix approve <request-id> --reason "Reviewed and approved"

# Deny a pending request
agentix deny <request-id>
agentix deny <request-id> --reason "Too risky for production"
```

## Audit Integration

All approval decisions are logged in the audit trail as `ApprovalDecision` events. Each entry includes:

| Field | Description |
|-------|-------------|
| `approver` | Identity of the approver (email, Slack ID, etc.) |
| `decision` | `approved` or `denied` |
| `reason` | Optional reason provided by the approver |
| `decided_at` | Timestamp of the decision |
| `action` | The tool call that was approved or denied |
| `agent_name` | The agent that requested approval |
| `run_id` | The agent run in which the request occurred |

Audit logs can be queried via the REST API or forwarded to external systems (OpenTelemetry, S3, etc.).

## Configuration Reference

### Agent-Level (`agent.yaml`)

```yaml
name: my-agent
model:
  preferred: anthropic/claude-sonnet-4-6
mode: semi-autonomous               # autonomous | semi-autonomous | manual

approval:
  flagged_tools:                    # Tool names requiring approval in semi-autonomous mode
    - kubectl
    - aws
  flagged_patterns:                 # Input substrings requiring approval (case-insensitive)
    - delete
    - drop
    - terminate
  approvers:                        # Who receives approval requests
    - "slack:U015ADMIN"
    - "email:sre-lead@company.com"
  timeout_seconds: 300              # Seconds before request expires (default: 300)
```

### Workspace-Level (`agentix.yaml`)

Workspace defaults apply to all agents that do not specify their own approval config:

```yaml
spec:
  defaults:
    mode: semi-autonomous
    approval:
      timeout_seconds: 300
      approvers:
        - "email:platform-team@company.com"
```

### REST API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/approvals` | GET | List pending approval requests (filter: `?agent=name`) |
| `/api/v1/approvals/:id` | GET | Get a specific approval request |
| `/api/v1/approvals/:id/approve` | POST | Approve a pending request |
| `/api/v1/approvals/:id/deny` | POST | Deny a pending request |

### WebSocket Events

When streaming agent output via WebSocket, two events indicate approval activity:

| Event | Payload | Description |
|-------|---------|-------------|
| `approval_waiting` | `{ id, agent_name, action }` | Agent paused, awaiting decision |
| `approval_decided` | `{ id, decision }` | Decision made, agent resuming |

## Quickstart Example

The `quickstart/semi-autonomous-agent/` directory contains a complete working example of a semi-autonomous infrastructure management agent:

```
quickstart/semi-autonomous-agent/
├── agent.yaml   # infra-guardian with approval gates for kubectl, aws, shell
└── SOUL.md      # Agent identity — explains destructive operation policy
```

To run the example:

```bash
# Start the gateway with the example agent
agentix gateway start --config quickstart/agentix.yaml

# In another terminal, run the agent
agentix run infra-guardian --input "List all pods in the default namespace"

# When the agent tries a flagged operation, approve or deny from a third terminal
agentix approvals
agentix approve <request-id>
```

## Related

- [Security concepts](security.md) — audit trail and SSRF protection
- [CLI reference: approvals](../reference/cli-approvals.md) — full CLI command reference
- [Telemetry](telemetry.md) — tracing approval decisions in distributed systems
