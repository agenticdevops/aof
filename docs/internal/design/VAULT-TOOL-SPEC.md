# HashiCorp Vault Tool Specification

## 1. Overview

The Vault Tool provides programmatic access to HashiCorp Vault's HTTP API for secrets management, dynamic credentials, and encryption operations. This tool enables AOF agents to securely retrieve secrets, manage credentials, and implement secret rotation workflows.

### 1.1 Purpose

- **Secret Retrieval**: Read secrets from KV, database, and other secret engines
- **Dynamic Credentials**: Generate short-lived credentials for databases and cloud providers
- **Secret Writing**: Store secrets securely (with appropriate permissions)
- **Token Management**: Validate and lookup token information
- **Encryption**: Encrypt/decrypt data using Vault's transit engine

### 1.2 Vault API Capabilities

Vault provides comprehensive secret management:
- **KV Secrets Engine**: Key-value secret storage (v1 and v2)
- **Database Secrets**: Dynamic database credentials
- **AWS/Azure/GCP Secrets**: Cloud provider credentials
- **Transit Engine**: Encryption as a service
- **PKI**: Certificate management

### 1.3 Feature Flag

```toml
[features]
security = ["reqwest", "serde_json", "base64"]
```

## 2. Tool Operations

### 2.1 vault_kv_get

Read a secret from KV secrets engine.

**Purpose**: Retrieve secrets for application configuration, database credentials, API keys.

**Parameters**:
- `endpoint` (required): Vault server URL (e.g., `https://vault.example.com`)
- `path` (required): Secret path (e.g., `secret/data/myapp/config`)
- `version` (optional): Secret version (KV v2 only), default: latest
- `token` (required): Vault token for authentication

**Response**:
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
  }
}
```

### 2.2 vault_kv_list

List secrets at a path.

**Purpose**: Discover available secrets in a path.

**Parameters**:
- `endpoint` (required): Vault server URL
- `path` (required): Path to list (e.g., `secret/metadata/myapp`)
- `token` (required): Vault token

**Response**:
```json
{
  "success": true,
  "keys": ["config", "database/", "api-keys/"]
}
```

### 2.3 vault_kv_put

Write a secret to KV secrets engine.

**Purpose**: Store or update secrets.

**Parameters**:
- `endpoint` (required): Vault server URL
- `path` (required): Secret path
- `data` (required): Secret data as JSON object
- `cas` (optional): Check-and-set version for optimistic locking
- `token` (required): Vault token

**Response**:
```json
{
  "success": true,
  "metadata": {
    "version": 4,
    "created_time": "2024-01-15T10:05:00Z"
  }
}
```

### 2.4 vault_token_lookup

Look up information about a token.

**Purpose**: Validate tokens, check permissions, get metadata.

**Parameters**:
- `endpoint` (required): Vault server URL
- `token` (required): Token to lookup (uses self-lookup if same as auth token)

**Response**:
```json
{
  "success": true,
  "data": {
    "accessor": "...",
    "creation_time": 1705312800,
    "display_name": "token-myapp",
    "expire_time": "2024-01-16T10:00:00Z",
    "policies": ["default", "myapp-read"]
  }
}
```

### 2.5 vault_transit_encrypt

Encrypt data using Transit secrets engine.

**Purpose**: Encrypt sensitive data for storage outside Vault.

**Parameters**:
- `endpoint` (required): Vault server URL
- `key_name` (required): Transit key name
- `plaintext` (required): Base64-encoded plaintext
- `token` (required): Vault token

**Response**:
```json
{
  "success": true,
  "ciphertext": "vault:v1:..."
}
```

### 2.6 vault_transit_decrypt

Decrypt data using Transit secrets engine.

**Purpose**: Decrypt data encrypted by Vault Transit.

**Parameters**:
- `endpoint` (required): Vault server URL
- `key_name` (required): Transit key name
- `ciphertext` (required): Vault ciphertext
- `token` (required): Vault token

**Response**:
```json
{
  "success": true,
  "plaintext": "base64-encoded-data"
}
```

## 3. Authentication Methods

The Vault tool supports multiple authentication methods:

1. **Token Authentication** (default): Direct token via `token` parameter
2. **AppRole**: Use `vault_approle_login` to get token
3. **Kubernetes**: Use `vault_k8s_login` for in-cluster auth

### 3.1 vault_approle_login

Authenticate using AppRole and get a token.

**Parameters**:
- `endpoint` (required): Vault server URL
- `role_id` (required): AppRole role ID
- `secret_id` (required): AppRole secret ID

**Response**:
```json
{
  "success": true,
  "auth": {
    "client_token": "...",
    "lease_duration": 3600,
    "policies": ["default", "myapp"]
  }
}
```

## 4. Error Handling

All operations return structured errors:

```json
{
  "success": false,
  "error": "permission denied",
  "error_code": 403,
  "details": "missing required capability: read"
}
```

## 5. Security Considerations

1. **Token Security**: Never log or expose tokens in output
2. **Least Privilege**: Use tokens with minimal required permissions
3. **Token Rotation**: Use short-lived tokens, renew as needed
4. **TLS Verification**: Always verify Vault's TLS certificate in production
5. **Audit Trail**: All Vault operations are logged in Vault's audit log

## 6. Example Agent Usage

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: secret-retriever
spec:
  model: google:gemini-2.5-flash
  tools:
    - vault_kv_get
    - vault_kv_list
    - vault_token_lookup

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You are a secrets management assistant.

    Use vault_kv_get to retrieve secrets when asked.
    Use vault_kv_list to discover available secrets.
    Never expose secret values directly - summarize what's available.
```
