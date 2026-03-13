# Approval Workflows

OpenAgentiX supports human-in-the-loop approval workflows that let operators control which agent actions proceed automatically and which require explicit approval.

## Autonomy Modes

Every agent operates in one of three modes:

| Mode | Behavior |
|------|----------|
| `autonomous` | All tool calls execute immediately. No approvals required. |
| `semi_autonomous` | Tool calls matching configured patterns pause for approval. Unmatched calls proceed automatically. |
| `manual` | Every tool call requires explicit approval before execution. |

The default mode is `autonomous` (backward-compatible).

## How It Works

1. Before each tool call in the ReAct loop, the **approval policy** is checked
2. If the policy requires approval for the tool, an **approval request** is created
3. The agent pauses and emits an `ApprovalWaiting` event (visible via SSE, CLI, and logs)
4. An operator reviews the request and issues `agentix approve <id>` or `agentix deny <id>`
5. On approval, the tool executes normally. On denial, the agent receives an error observation and continues reasoning

## Configuration

### Agent-Level Policy

Add an `approval` block to `agent.yaml`:

```yaml
# agents/dba-optimizer/agent.yaml
name: dba-optimizer
model:
  preferred: anthropic/claude-sonnet-4-6
approval:
  mode: semi_autonomous
  require_approval_for:
    - "kubectl.*delete"    # Any kubectl delete command
    - "helm.*uninstall"    # Helm uninstall operations
    - "psql.*DROP"         # Database DROP statements
  expires_after_secs: 300  # 5-minute expiry (default)
```

### Pattern Matching

The `require_approval_for` field accepts regex patterns matched against both tool names and their input descriptions:

- `"kubectl"` — matches any kubectl tool call
- `"kubectl.*delete"` — matches kubectl delete operations
- `".*"` — matches everything (equivalent to `manual` mode)
- Empty list with `semi_autonomous` mode — nothing requires approval (equivalent to `autonomous`)

### Expiry

Unapproved requests expire after `expires_after_secs` (default: 300 seconds / 5 minutes). Expired requests are treated as denied, and the agent receives a timeout observation.

## Approval Request Lifecycle

```
Created (Pending) ──┬── Approved ──> Tool executes
                    ├── Denied   ──> Error observation returned to agent
                    └── Expired  ──> Timeout observation returned to agent
```

Every approval decision is logged in the audit trail as an `ApprovalDecision` event.

## REST API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/approvals` | GET | List approval requests (filter by `?agent=`, `?status=`) |
| `/api/v1/approvals/:id` | GET | Get a single approval request |
| `/api/v1/approvals/:id/approve` | POST | Approve a pending request |
| `/api/v1/approvals/:id/deny` | POST | Deny a pending request |

### SSE Events

When streaming agent output, two additional event types appear:

- `approval_waiting` — agent is paused, waiting for a decision
- `approval_decided` — decision was made, agent resumes

## CLI Commands

```bash
# List pending approvals
agentix approvals

# List all approvals (including resolved)
agentix approvals --all

# Filter by agent
agentix approvals --agent dba-optimizer

# Approve a request
agentix approve <request-id>
agentix approve <request-id> --reason "Reviewed and safe"

# Deny a request
agentix deny <request-id>
agentix deny <request-id> --reason "Too risky for production"
```

## Related

- [Security concepts](security.md) — audit trail and SSRF protection
- [CLI reference: approvals](../reference/cli-approvals.md) — full command reference
- [Quickstart: approval workflows](../../quickstart/agentix-approvals.yaml) — example workspace config
