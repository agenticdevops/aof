# CLI Reference: Approval Commands

Manage human-in-the-loop approval workflows for agent tool calls.

## `agentix approvals`

List approval requests from the gateway.

### Synopsis

```
agentix approvals [OPTIONS]
```

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--agent NAME` | - | Filter by agent name |
| `--all` | false | Show all statuses (not just pending) |
| `--output json` | text | Output raw JSON instead of formatted table |
| `--gateway-url URL` | `http://127.0.0.1:7777` | Gateway base URL |

### Examples

```bash
# List pending approvals
agentix approvals

# List all approvals (including approved, denied, expired)
agentix approvals --all

# Filter by agent
agentix approvals --agent dba-optimizer

# JSON output
agentix approvals --output json
agentix approvals --all --output json | jq '.[] | select(.status == "denied")'
```

### Output Format

```
REQUEST ID    AGENT                 TOOL                      STATUS      CREATED
----------------------------------------------------------------------------------
a1b2c3d4e5f6  dba-optimizer         kubectl                   pending     2026-03-13 14:30:05
f6e5d4c3b2a1  log-analyzer          shell_exec                approved    2026-03-13 14:28:12

2 request(s) shown.
```

---

## `agentix approve`

Approve a pending approval request, allowing the agent to proceed with the tool call.

### Synopsis

```
agentix approve <ID> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ID>` | Yes | Approval request ID |

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--reason TEXT` | - | Reason for approval (recorded in audit trail) |
| `--output json` | text | Output raw JSON instead of formatted text |
| `--gateway-url URL` | `http://127.0.0.1:7777` | Gateway base URL |

### Examples

```bash
# Approve a request
agentix approve a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Approve with reason
agentix approve a1b2c3d4-e5f6-7890-abcd-ef1234567890 --reason "Reviewed, safe for production"
```

---

## `agentix deny`

Deny a pending approval request, preventing the agent from executing the tool call.

### Synopsis

```
agentix deny <ID> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ID>` | Yes | Approval request ID |

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--reason TEXT` | - | Reason for denial (recorded in audit trail) |
| `--output json` | text | Output raw JSON instead of formatted text |
| `--gateway-url URL` | `http://127.0.0.1:7777` | Gateway base URL |

### Examples

```bash
# Deny a request
agentix deny a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Deny with reason
agentix deny a1b2c3d4-e5f6-7890-abcd-ef1234567890 --reason "Too risky for production"
```

## REST API Equivalents

| CLI command | REST endpoint |
|-------------|---------------|
| `agentix approvals` | `GET /api/v1/approvals?status=pending` |
| `agentix approvals --all` | `GET /api/v1/approvals` |
| `agentix approvals --agent X` | `GET /api/v1/approvals?agent=X&status=pending` |
| `agentix approve <id>` | `POST /api/v1/approvals/<id>/approve` |
| `agentix deny <id>` | `POST /api/v1/approvals/<id>/deny` |

## Related

- [Approval workflows concepts](../concepts/approval-workflows.md) — full workflow documentation
- [Security concepts](../concepts/security.md) — audit trail integration
- [Quickstart: approval workflows](../../quickstart/agentix-approvals.yaml) — example workspace config
