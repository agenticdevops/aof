---
sidebar_position: 2
---

# HashiCorp Vault Tools

AOF provides comprehensive integration with HashiCorp Vault for secrets management, encryption, and authentication.

## Available Tools

| Tool | Description |
|------|-------------|
| `vault_kv_get` | Read secret from KV secrets engine |
| `vault_kv_put` | Write secret to KV secrets engine |
| `vault_kv_list` | List secrets at a path |
| `vault_kv_delete` | Delete a secret |
| `vault_token_lookup` | Get information about a token |
| `vault_transit_encrypt` | Encrypt data using Transit engine |
| `vault_transit_decrypt` | Decrypt data using Transit engine |
| `vault_approle_login` | Authenticate using AppRole |

## Configuration

Set these environment variables:

```bash
export VAULT_ADDR="https://vault.example.com"
export VAULT_TOKEN="your-token"
```

## Tool Reference

### vault_kv_get

Read a secret from Vault's KV secrets engine.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `path` | string | Yes | Secret path (e.g., `secret/data/myapp`) |
| `version` | integer | No | Secret version (KV v2 only) |
| `token` | string | Yes | Vault authentication token |
| `namespace` | string | No | Vault namespace (Enterprise) |

**Example:**

```yaml
tools:
  - vault_kv_get

# Agent prompt usage:
# "Read the database credentials from secret/data/myapp/db"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "username": "admin",
    "password": "..."
  },
  "metadata": {
    "version": 3,
    "created_time": "2024-01-15T10:00:00Z"
  },
  "path": "secret/data/myapp/db"
}
```

### vault_kv_put

Write a secret to Vault's KV secrets engine.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `path` | string | Yes | Secret path |
| `data` | object | Yes | Secret data as key-value pairs |
| `cas` | integer | No | Check-and-set version for optimistic concurrency |
| `token` | string | Yes | Vault authentication token |

**Example:**

```yaml
# Write a secret with CAS for safe updates
# "Store API key in secret/data/myapp/api with cas=5"
```

### vault_kv_list

List secret keys at a path.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `path` | string | Yes | Path to list (e.g., `secret/metadata/myapp`) |
| `token` | string | Yes | Vault authentication token |

**Response:**

```json
{
  "success": true,
  "keys": ["db", "api", "cache/"],
  "path": "secret/metadata/myapp"
}
```

### vault_transit_encrypt

Encrypt data using Vault's Transit secrets engine.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `key_name` | string | Yes | Name of the encryption key |
| `plaintext` | string | Yes | Data to encrypt |
| `context` | string | No | Base64-encoded context for key derivation |
| `key_version` | integer | No | Specific key version to use |
| `token` | string | Yes | Vault authentication token |
| `mount` | string | No | Transit mount path (default: `transit`) |

**Response:**

```json
{
  "success": true,
  "ciphertext": "vault:v1:AbC123...",
  "key_version": 1
}
```

### vault_transit_decrypt

Decrypt ciphertext using Vault's Transit secrets engine.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `key_name` | string | Yes | Name of the encryption key |
| `ciphertext` | string | Yes | Vault-encrypted ciphertext |
| `context` | string | No | Base64-encoded context for key derivation |
| `token` | string | Yes | Vault authentication token |
| `mount` | string | No | Transit mount path (default: `transit`) |

### vault_approle_login

Authenticate to Vault using AppRole method.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Vault server URL |
| `role_id` | string | Yes | AppRole role ID |
| `secret_id` | string | Yes | AppRole secret ID |
| `mount` | string | No | AppRole mount path (default: `approle`) |

**Response:**

```json
{
  "success": true,
  "client_token": "s.abc123...",
  "accessor": "...",
  "policies": ["default", "myapp-policy"],
  "lease_duration": 3600,
  "renewable": true
}
```

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: secrets-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - vault_kv_get
    - vault_kv_put
    - vault_kv_list
    - vault_transit_encrypt
    - vault_transit_decrypt

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You are a secrets management assistant.

    - Read secrets when needed for configuration
    - Encrypt sensitive data before storing
    - Never expose secret values in your output
    - Track secret versions for auditing
```

## Best Practices

1. **Use AppRole for automation** - Don't use root tokens in production
2. **Enable versioning** - KV v2 provides version history
3. **Use Transit for encryption** - Let Vault manage encryption keys
4. **Set appropriate TTLs** - Use short-lived tokens when possible
5. **Audit access** - Enable Vault audit logging
