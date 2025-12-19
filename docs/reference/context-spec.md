# Context Resource Reference

Complete reference for Context resource specifications. Contexts define execution environment boundaries with cluster configuration, approval requirements, and rate limits.

## Overview

A Context represents an execution environment boundary that gets injected at runtime. Contexts enable:
- Separating agent logic from deployment configuration
- Multi-tenant deployments with isolated environments
- Context-specific approval policies
- Audit logging and rate limiting

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value

spec:
  kubeconfig: string        # Optional: Path to kubeconfig
  namespace: string         # Optional: Kubernetes namespace
  cluster: string           # Optional: Cluster name
  env:                      # Optional: Environment variables
    KEY: value
  approval:                 # Optional: Approval configuration
    required: bool
    allowed_users: [string]
  audit:                    # Optional: Audit logging
    enabled: bool
  limits:                   # Optional: Rate limits
    max_requests_per_minute: int
```

---

## Spec Fields

### Cluster Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kubeconfig` | string | No | Path to kubeconfig file (supports `${VAR}`) |
| `namespace` | string | No | Default Kubernetes namespace |
| `cluster` | string | No | Cluster name for identification |
| `working_dir` | string | No | Working directory for tool execution |

**Example:**
```yaml
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  cluster: prod-us-east-1
  working_dir: /app
```

### Environment Variables

Environment variables are made available to agents running in this context.

```yaml
spec:
  env:
    AWS_PROFILE: production
    AWS_REGION: us-east-1
    LOG_LEVEL: info
    CUSTOM_VAR: ${EXTERNAL_VAR}  # Supports expansion
```

**Auto-injected variables:**
| Variable | Description |
|----------|-------------|
| `AOF_CONTEXT` | Context name |
| `AOF_NAMESPACE` | Kubernetes namespace (if set) |
| `AOF_CLUSTER` | Cluster name (if set) |

---

## Approval Configuration

Control when and who can approve destructive operations.

### `spec.approval`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `required` | bool | false | Enable approval for destructive ops |
| `allowed_users` | array | [] | Users who can approve (empty = anyone) |
| `timeout_seconds` | int | 300 | Approval timeout (5 minutes) |
| `require_for` | array | [] | Patterns requiring approval (regex) |
| `allow_self_approval` | bool | false | Allow requestor to approve own commands |
| `min_approvers` | int | 1 | Minimum approvers required |

### Basic Approval

```yaml
spec:
  approval:
    required: true
    allowed_users:
      - U12345678         # Slack user ID
      - slack:U87654321   # Platform-prefixed ID
      - telegram:123456   # Telegram user ID
    timeout_seconds: 300
```

### Pattern-Based Approval

Only require approval for specific commands:

```yaml
spec:
  approval:
    required: true
    require_for:
      - "kubectl delete"
      - "kubectl scale.*--replicas=0"
      - "helm uninstall"
      - "aws.*terminate"
    allowed_users:
      - U015SRELEAD
```

### Multi-Approver Workflow

```yaml
spec:
  approval:
    required: true
    min_approvers: 2          # Requires 2 people to approve
    allow_self_approval: false
    allowed_users:
      - U015ADMIN
      - U016SRELEAD
      - U017ONCALL
```

### User ID Formats

Contexts support multiple user ID formats:

| Format | Example | Description |
|--------|---------|-------------|
| Raw ID | `U12345678` | Direct platform ID |
| Slack prefixed | `slack:U12345678` | Explicit Slack ID |
| Telegram prefixed | `telegram:123456789` | Explicit Telegram ID |
| Discord prefixed | `discord:123456789012345678` | Explicit Discord ID |

---

## Audit Configuration

Log agent executions for compliance and debugging.

### `spec.audit`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | false | Enable audit logging |
| `sink` | string | - | Audit sink URL |
| `events` | array | [] | Event types to audit |
| `include_payload` | bool | false | Include full request/response |
| `retention` | string | - | Retention period (e.g., "90d") |

### Audit Events

| Event | Description |
|-------|-------------|
| `agent_start` | Agent execution started |
| `agent_complete` | Agent execution completed |
| `tool_call` | Tool invocation |
| `approval_requested` | Approval requested |
| `approval_granted` | Approval granted |
| `approval_denied` | Approval denied |
| `error` | Error occurred |
| `all` | All events |

### Example

```yaml
spec:
  audit:
    enabled: true
    sink: s3://company-audit-logs/aof/prod/
    events:
      - agent_start
      - agent_complete
      - tool_call
      - approval_granted
      - approval_denied
    include_payload: false
    retention: "90d"
```

### Sink Formats

| Format | Example | Description |
|--------|---------|-------------|
| S3 | `s3://bucket/prefix/` | AWS S3 bucket |
| File | `file:///var/log/aof/audit.log` | Local file |
| HTTP | `https://audit.company.com/ingest` | HTTP endpoint |
| Stdout | `stdout://` | Console output |

---

## Rate Limits

Protect resources and control costs.

### `spec.limits`

| Field | Type | Description |
|-------|------|-------------|
| `max_requests_per_minute` | int | Request rate limit |
| `max_tokens_per_day` | int | Daily token limit |
| `max_concurrent` | int | Max parallel executions |
| `max_execution_time_seconds` | int | Per-request timeout |
| `max_cost_per_day` | float | Daily cost limit (credits) |

### Example

```yaml
spec:
  limits:
    max_requests_per_minute: 60
    max_tokens_per_day: 1000000
    max_concurrent: 5
    max_execution_time_seconds: 300
    max_cost_per_day: 50.00
```

---

## Secret References

Reference external secrets for credentials.

```yaml
spec:
  secrets:
    - name: aws-credentials
      key: access-key-id
      env_var: AWS_ACCESS_KEY_ID
    - name: aws-credentials
      key: secret-access-key
      env_var: AWS_SECRET_ACCESS_KEY
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Secret name |
| `key` | string | No | Specific key in secret |
| `env_var` | string | No | Environment variable to set |

---

## Complete Examples

### Production Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  labels:
    environment: production
    team: platform
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  cluster: prod-us-east-1

  env:
    AWS_PROFILE: production
    AWS_REGION: us-east-1
    LOG_LEVEL: warn

  approval:
    required: true
    require_for:
      - "kubectl delete"
      - "kubectl scale"
      - "helm uninstall"
      - "aws.*terminate"
    allowed_users:
      - U015SRELEAD
      - U016ADMIN
    timeout_seconds: 300
    allow_self_approval: false

  audit:
    enabled: true
    sink: s3://company-audit/prod/
    events: [all]
    retention: "365d"

  limits:
    max_requests_per_minute: 30
    max_concurrent: 3
    max_execution_time_seconds: 600
```

### Staging Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging
  labels:
    environment: staging
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  namespace: staging
  cluster: staging-us-east-1

  env:
    AWS_PROFILE: staging
    LOG_LEVEL: debug

  # No approval required for staging
  approval:
    required: false

  limits:
    max_requests_per_minute: 100
    max_concurrent: 10
```

### Development Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: dev
spec:
  namespace: default

  env:
    LOG_LEVEL: debug
    DEV_MODE: "true"

  # No restrictions for development
```

### Multi-Tenant Customer Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: customer-acme
  labels:
    tenant: acme
    tier: enterprise
spec:
  namespace: tenant-acme

  env:
    TENANT_ID: acme
    TENANT_TIER: enterprise

  approval:
    required: true
    allowed_users:
      - acme-admin@acme.com

  limits:
    max_requests_per_minute: 100
    max_tokens_per_day: 5000000
    max_cost_per_day: 100.00

  audit:
    enabled: true
    sink: s3://acme-audit-logs/aof/
```

---

## Environment Variable Expansion

Context values support `${VAR_NAME}` expansion:

```yaml
spec:
  kubeconfig: ${KUBECONFIG_PROD}       # Expanded at runtime
  env:
    API_KEY: ${EXTERNAL_API_KEY}       # Expanded at runtime
    STATIC_VALUE: "not-expanded"       # Used as-is
```

**Expansion order:**
1. System environment variables
2. Context `env` values (can reference system vars)
3. Auto-injected AOF variables

---

## Validation

```bash
# Validate context YAML
aofctl validate -f context.yaml

# List loaded contexts
aofctl get contexts

# Describe specific context
aofctl describe context prod
```

### Validation Rules
- Name is required
- `min_approvers` must be >= 1
- `max_concurrent` must be > 0
- Approval patterns must be valid regex

---

## Usage with FlowBinding

Contexts are referenced in FlowBindings:

```yaml
# contexts/prod.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
  approval:
    required: true

---
# bindings/prod-k8s.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-k8s-binding
spec:
  trigger: slack-prod
  context: prod              # Reference to Context
  flow: k8s-ops-flow
```

---

## See Also

- [FlowBinding Reference](./flowbinding-spec.md) - Compose contexts with flows
- [Trigger Reference](./trigger-spec.md) - Message source configuration
- [DaemonConfig Reference](./daemon-config.md) - Server configuration
- [Resource Selection Guide](../concepts/resource-selection.md) - When to use what
