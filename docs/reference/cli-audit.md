# CLI Reference: `agentix audit`

View the audit trail for an agent, including tool calls, LLM calls, security events, and lifecycle events.

## Synopsis

```
agentix audit <AGENT> [OPTIONS]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<AGENT>` | Yes | Agent name to show audit trail for |

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--limit N` | 50 | Maximum number of entries to show |
| `--security` | false | Show security events only (violations, denials, blocks) |
| `--run <RUN_ID>` | - | Filter to a specific run ID |
| `--output json` | text | Output raw JSON instead of formatted table |
| `--gateway-url URL` | `http://127.0.0.1:7777` | Gateway base URL |

## Examples

### Show recent audit entries

```bash
# Last 50 entries (default)
agentix audit dba-optimizer

# Last 100 entries
agentix audit dba-optimizer --limit 100
```

### Filter to security events

```bash
# Security violations, denials, and blocks only
agentix audit dba-optimizer --security
```

### Filter to a specific run

```bash
agentix audit dba-optimizer --run run-abc123
agentix audit dba-optimizer --run run-abc123 --limit 100
```

### JSON output

```bash
agentix audit dba-optimizer --output json
agentix audit dba-optimizer --security --output json | jq '.[] | select(.outcome == "blocked")'
```

## Output Format

### Text output (default)

```
TIMESTAMP            EVENT TYPE          ACTION                                        OUTCOME
-------------------------------------------------------------------------------------------------
2026-03-13 12:00:05  tool_call           kubectl get pods -n production                success
2026-03-13 12:00:03  llm_call            anthropic/claude-sonnet-4-6                   success
2026-03-13 12:00:01  agent_start         Starting agent run run-789                    success
2026-03-13 11:59:55  security_violation  SSRF blocked: 169.254.169.254                 blocked

4 entries shown.
```

- **TIMESTAMP**: UTC timestamp (YYYY-MM-DD HH:MM:SS)
- **EVENT TYPE**: Category of event (see Event Types below)
- **ACTION**: Human-readable description of what happened
- **OUTCOME**: Color-coded result:
  - Green: `success`
  - Red: `failure(...)` or `blocked(...)`
  - Yellow: `denied(...)`

### JSON output

```json
[
  {
    "timestamp": "2026-03-13T12:00:05Z",
    "event_type": "tool_call",
    "agent_name": "dba-optimizer",
    "run_id": "run-789",
    "actor": "agent:dba-optimizer",
    "action": "kubectl get pods -n production",
    "outcome": "success",
    "details": {
      "tool_name": "kubectl",
      "duration_ms": 245
    }
  }
]
```

## Event Types

| Event Type | Description |
|-----------|-------------|
| `agent_start` | Agent run initiated |
| `agent_complete` | Agent run completed successfully |
| `agent_error` | Agent run failed |
| `tool_call` | Tool was executed |
| `llm_call` | LLM provider was called |
| `security_violation` | SSRF or capability violation |
| `secret_access` | Secret was accessed |
| `approval_decision` | Approval workflow decision |

## REST API Equivalent

| CLI command | REST endpoint |
|-------------|---------------|
| `agentix audit <agent>` | `GET /api/v1/agents/<agent>/audit?limit=50` |
| `agentix audit <agent> --security` | `GET /api/v1/audit/security?limit=50` |
| `agentix audit <agent> --limit N` | `GET /api/v1/agents/<agent>/audit?limit=N` |

## Related

- [Security concepts](../concepts/security.md) -- full security model documentation
- [Security setup tutorial](../tutorials/security-setup.md) -- step-by-step guide
- [Telemetry config](telemetry-config.md) -- tracing and metrics configuration
