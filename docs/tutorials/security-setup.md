# Tutorial: Setting Up Security in OpenAgentiX

This tutorial walks through enabling all security features in an OpenAgentiX workspace: SSRF protection, secret encryption, audit trail, and WASM capability enforcement.

## Prerequisites

- OpenAgentiX installed (`cargo install agentix`)
- A workspace with `agentix.yaml` (run `agentix onboard` if needed)
- A running gateway (`agentix gateway start`)

## Step 1: Enable Security in Your Workspace

Add the `security` block to your `agentix.yaml`:

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-workspace
spec:
  agents_dir: ./agents
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 10
    timeout: 5m
  gateway:
    host: "127.0.0.1"
    port: 7777
  security:
    ssrf_protection: true
    secret_encryption: true
    audit_enabled: true
```

All three security features default to `true`, so simply adding `security: {}` is equivalent. Explicit values are shown for clarity.

## Step 2: Set Up Secret Encryption

Secret encryption requires a passphrase set via the `AGENTIX_SECRET_KEY` environment variable.

```bash
# Generate a strong passphrase
export AGENTIX_SECRET_KEY="$(openssl rand -base64 32)"

# Or use a memorable passphrase
export AGENTIX_SECRET_KEY="my-workspace-secret-key-2026"
```

Add it to your shell profile for persistence:

```bash
echo 'export AGENTIX_SECRET_KEY="your-passphrase"' >> ~/.bashrc
```

### How it works

- **Encryption**: AES-256-GCM with SHA-256 key derivation from the passphrase
- **Nonce**: 12 bytes, randomly generated per encryption (unique ciphertext every time)
- **Redaction**: The `SecretRedactor` automatically replaces secret values with `[REDACTED:<name>]` in all logs, traces, and error output

## Step 3: Configure SSRF Protection Allow-List

By default, SSRF protection blocks all requests to private/internal IP ranges:

| Range | Description |
|-------|-------------|
| `10.0.0.0/8` | RFC 1918 private |
| `172.16.0.0/12` | RFC 1918 private |
| `192.168.0.0/16` | RFC 1918 private |
| `127.0.0.0/8` | Loopback |
| `::1` | IPv6 loopback |
| `169.254.0.0/16` | Link-local / cloud metadata |

If your agents need to reach internal services, add them to the allow-list:

```yaml
security:
  ssrf_protection: true
  allowed_private_endpoints:
    - "10.0.0.5"      # Internal Prometheus
    - "172.16.1.10"    # Internal Grafana
    - "192.168.1.100"  # Internal database
```

Allow-listed IPs bypass SSRF checks. All other private IPs remain blocked. When a blocked request is detected:

1. The HTTP request is **not executed**
2. The agent receives an error explaining the block
3. A `SecurityViolation` audit entry with `outcome: Blocked` is logged

## Step 4: Add Capabilities to WASM Tools

If you use WASM tool modules, declare their required capabilities in the tool manifest:

```yaml
# agents/my-agent/tools/kubectl-query.yaml
name: kubectl-query
type: wasm
module: ./modules/kubectl-query.wasm
capabilities:
  - network
  - http_endpoints:
      - "https://kubernetes.default.svc"
```

### Available capabilities

| Capability | Description |
|-----------|-------------|
| `network` | Outbound network access |
| `filesystem` | Read/write files on disk |
| `secrets` | Access to agent secrets |
| `http_endpoints` | Specific HTTP endpoints (allowlist) |
| `resource_limits` | CPU/memory/time limits |

If a WASM tool attempts to use an undeclared capability, the operation is blocked and a `SecurityViolation` audit entry with `outcome: Denied` is logged.

## Step 5: View the Audit Trail

Start the gateway and run an agent:

```bash
# Start gateway
agentix gateway start

# Run an agent
agentix run my-agent --input "Check system health"

# View the audit trail
agentix audit my-agent
```

Example output:

```
TIMESTAMP            EVENT TYPE          ACTION                                        OUTCOME
-------------------------------------------------------------------------------------------------
2026-03-13 14:30:12  agent_complete      Completed in 3 iterations, 2 tool calls       success
2026-03-13 14:30:11  tool_call           kubectl get pods -n default                   success
2026-03-13 14:30:09  llm_call            anthropic/claude-sonnet-4-6                   success
2026-03-13 14:30:08  tool_call           curl http://10.0.0.5:9090/api/v1/query        success
2026-03-13 14:30:07  llm_call            anthropic/claude-sonnet-4-6                   success
2026-03-13 14:30:05  agent_start         Starting agent run run-abc123                 success

6 entries shown.
```

## Step 6: Monitor Security Events

Filter the audit trail to security-relevant events only:

```bash
# Security events only (SSRF blocks, capability denials)
agentix audit my-agent --security

# JSON output for programmatic analysis
agentix audit my-agent --security --output json

# Filter to a specific run
agentix audit my-agent --run run-abc123

# Increase the limit for longer histories
agentix audit my-agent --limit 200
```

### REST API access

You can also query audit data via the REST API:

```bash
# Agent audit trail
curl http://127.0.0.1:7777/api/v1/agents/my-agent/audit?limit=100

# Security events across all agents
curl http://127.0.0.1:7777/api/v1/audit/security?limit=50
```

## Complete Example

See `quickstart/agentix-secure.yaml` for a ready-to-use workspace config with all security features enabled.

```bash
# Copy the quickstart config
cp quickstart/agentix-secure.yaml agentix.yaml

# Set the encryption key
export AGENTIX_SECRET_KEY="my-secure-passphrase"
export ANTHROPIC_API_KEY="sk-ant-..."

# Start the gateway
agentix gateway start
```

## Summary

| Feature | Configuration | Default |
|---------|--------------|---------|
| SSRF Protection | `security.ssrf_protection` | `true` |
| Secret Encryption | `security.secret_encryption` + `AGENTIX_SECRET_KEY` env var | `true` |
| Audit Trail | `security.audit_enabled` | `true` |
| SSRF Allow-List | `security.allowed_private_endpoints` | empty |
| WASM Capabilities | `capabilities` in tool YAML | deny-all |

All security features are **enabled by default** with safe defaults. To disable a feature, explicitly set it to `false` in your workspace config.

## Related

- [Security concepts](../concepts/security.md) -- full security model documentation
- [CLI audit reference](../reference/cli-audit.md) -- `agentix audit` command reference
- [WASM tools guide](../guides/wasm-tools.md) -- WASM tool configuration
