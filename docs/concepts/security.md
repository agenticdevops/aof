# Security Model

OpenAgentiX provides defense-in-depth security for agentic workloads through four pillars: WASM capability enforcement, secret encryption, audit trail, and SSRF protection.

## Overview

When agents execute tools autonomously, security boundaries are critical. OpenAgentiX enforces security at multiple layers:

| Pillar | What it Protects | Module |
|--------|-----------------|--------|
| SSRF Protection | Prevents tools from accessing internal networks | `agentix_core::security::SsrfGuard` |
| WASM Capability Enforcement | Sandboxes tool execution to declared capabilities | `agentix_core::security::CapabilityPolicy` |
| Secret Encryption | Encrypts secrets at rest, redacts from logs | `agentix_core::secret_store::SecretStore` |
| Audit Trail | Logs every action with actor, timestamp, outcome | `agentix_runtime::audit_store::AuditStore` |

## SSRF Protection

### What is SSRF?

Server-Side Request Forgery (SSRF) occurs when an agent's tool makes HTTP requests to internal services that should not be externally accessible. For example, an agent told to "check the API status" could be tricked into requesting `http://169.254.169.254/latest/meta-data/` to steal AWS credentials.

### How SsrfGuard Works

Every outbound HTTP request from a tool passes through `SsrfGuard::check_url()` before execution. The guard blocks requests to:

| Range | Description | Example |
|-------|------------|---------|
| `10.0.0.0/8` | RFC 1918 private | `http://10.0.0.1:8080/api` |
| `172.16.0.0/12` | RFC 1918 private | `http://172.16.0.1/` |
| `192.168.0.0/16` | RFC 1918 private | `http://192.168.1.1/` |
| `127.0.0.0/8` | Loopback | `http://127.0.0.1/` |
| `::1` | IPv6 loopback | `http://[::1]/` |
| `localhost` | Hostname loopback | `http://localhost:3000/` |
| `169.254.0.0/16` | Link-local / cloud metadata | `http://169.254.169.254/latest/meta-data/` |

### Allow-List for Internal Services

If your agents need to reach internal services (e.g., Prometheus, Grafana), configure the allow-list:

```yaml
security:
  ssrf_protection: true
  allowed_private_endpoints:
    - "10.0.0.5"     # Internal Prometheus
    - "172.16.1.10"   # Internal Grafana
```

Allow-listed IPs bypass SSRF checks while all other private IPs remain blocked.

### SSRF Violation Handling

When a tool attempts to access a blocked URL:
1. The request is **not executed**
2. An `SsrfViolation` error is returned to the agent
3. An `AuditEntry` with `event_type: SecurityViolation` and `outcome: Blocked` is logged
4. The agent receives an error message explaining why the request was blocked

## WASM Capability Enforcement

### How Capabilities Work

WASM tools declare their required capabilities in their tool manifest:

```yaml
name: kubectl-query
type: wasm
capabilities:
  - network          # Can make outbound network calls
  - http_endpoints:  # Specific endpoints allowed
      - "https://kubernetes.default.svc"
```

At runtime, the WASM sandbox only grants capabilities that are declared in the manifest. If a tool tries to use an undeclared capability (e.g., filesystem access when only `network` is declared), the operation is blocked.

### Capability Types

| Capability | Description |
|-----------|-------------|
| `network` | Outbound network access |
| `filesystem` | Read/write files on disk |
| `secrets` | Access to agent secrets |
| `http_endpoints` | Specific HTTP endpoints (allowlist) |
| `resource_limits` | CPU/memory/time limits |

### Violation Handling

When a capability is denied:
1. The WASM sandbox blocks the operation
2. A `SecurityViolation` audit entry is logged with `outcome: Denied`
3. The tool receives an error explaining which capability was missing

## Secret Encryption

### Encryption at Rest

All agent secrets are encrypted using AES-256-GCM before storage:

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key derivation**: SHA-256 of passphrase (from `AGENTIX_SECRET_KEY` env var or workspace config)
- **Nonce**: 12 bytes, randomly generated per encryption (ensures unique ciphertext)
- **Storage**: Ciphertext + nonce stored as base64-encoded JSON

### Secret Redaction

The `SecretRedactor` prevents secrets from appearing in logs, traces, or error output:

- Scans text output for known secret values
- Replaces matches with `[REDACTED:<secret_name>]`
- Works recursively on JSON objects
- Processes secrets longest-first to avoid partial matches

### Configuration

```bash
# Set the encryption key (required for secret encryption)
export AGENTIX_SECRET_KEY="your-secure-passphrase"
```

## Audit Trail

### What Gets Logged

Every significant action produces an `AuditEntry`:

| Event Type | When | Details |
|-----------|------|---------|
| `AgentStart` | Agent run begins | run_id, agent_name |
| `AgentComplete` | Agent run succeeds | iterations, tool_calls_count |
| `AgentError` | Agent run fails | error message |
| `ToolCall` | Tool is executed | tool_name, duration_ms, input summary |
| `LlmCall` | LLM is called | model, provider, token counts |
| `SecurityViolation` | SSRF or capability violation | blocked URL, denied capability |
| `SecretAccess` | Secret is accessed | secret name (not value) |
| `ApprovalDecision` | Approval workflow decision | approver, decision |

### Audit Entry Structure

Each entry includes:
- **timestamp**: When it happened (UTC)
- **event_type**: Category of event
- **agent_name**: Which agent
- **run_id**: Which run (if applicable)
- **actor**: Who/what initiated it (`system`, `agent:<name>`, `user:<id>`)
- **action**: Human-readable description
- **outcome**: `Success`, `Failure(reason)`, `Denied(reason)`, `Blocked(reason)`
- **details**: Structured key-value metadata
- **trace_id**: Link to distributed trace (if available)

### Querying Audit Data

**CLI:**
```bash
agentix audit dba-optimizer              # Last 50 entries
agentix audit dba-optimizer --limit 100  # Last 100 entries
agentix audit dba-optimizer --security   # Security events only
agentix audit dba-optimizer --run run-123 # Specific run
```

**REST API:**
```bash
# Agent audit trail
GET /api/v1/agents/dba-optimizer/audit?limit=100

# Security events
GET /api/v1/audit/security?limit=50
```

### Storage

Audit entries are stored in SQLite (`data/audit.db`) alongside other runtime databases (`runs.db`, `cost.db`, `trace.db`). The audit store is non-critical — a store failure never interrupts agent execution.

## Configuration Reference

Full `SecurityConfig` in workspace `agentix.yaml`:

```yaml
spec:
  security:
    # Enable SSRF protection for outbound HTTP tool calls (default: true)
    ssrf_protection: true

    # Enable AES-256-GCM encryption for secrets at rest (default: true)
    secret_encryption: true

    # Enable audit trail logging (default: true)
    audit_enabled: true

    # Private IPs allowed despite SSRF protection (default: empty)
    allowed_private_endpoints:
      - "10.0.0.5"      # Internal Prometheus
      - "172.16.1.10"    # Internal Grafana
```

All security features are **enabled by default** with sane defaults. To disable a feature, explicitly set it to `false`.
