# Context Resource

## Overview

The `Context` resource defines the execution environment and constraints for agent operations. It specifies Kubernetes cluster access, environment variables, approval workflows, audit settings, and operational limits.

## API Reference

**apiVersion:** `aof.dev/v1`
**kind:** `Context`

## Specification

### Context Spec Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kubeconfig` | string | No | Path to kubeconfig file. If not specified, uses default kubeconfig location or in-cluster config |
| `namespace` | string | No | Kubernetes namespace for operations. Defaults to `default` |
| `env` | map[string]string | No | Environment variables available to agents during execution |
| `approval` | [ApprovalConfig](#approvalconfig) | No | Approval workflow configuration for sensitive operations |
| `audit` | [AuditConfig](#auditconfig) | No | Audit logging configuration |
| `limits` | [LimitsConfig](#limitsconfig) | No | Resource and rate limiting configuration |

### ApprovalConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `required` | boolean | Yes | Whether approval is required for operations in this context |
| `allowed_users` | []string | No | List of user identifiers who can approve. If empty, any user can approve |
| `timeout_seconds` | integer | No | Timeout for approval requests in seconds. Defaults to 300 (5 minutes) |

### AuditConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | boolean | Yes | Whether audit logging is enabled |
| `sink` | string | No | Audit log destination. Options: `stdout`, `file`, `webhook`. Defaults to `stdout` |

### LimitsConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `max_requests_per_minute` | integer | No | Maximum API requests per minute. Prevents rate limit exhaustion |
| `max_tokens_per_day` | integer | No | Maximum LLM tokens consumed per day. Controls costs |

## Examples

### Production Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  kubeconfig: /etc/aof/kubeconfig-prod
  namespace: production
  env:
    ENVIRONMENT: production
    LOG_LEVEL: info
    DATADOG_API_KEY: ${DATADOG_API_KEY}
  approval:
    required: true
    allowed_users:
      - alice@example.com
      - bob@example.com
      - oncall@example.com
    timeout_seconds: 600
  audit:
    enabled: true
    sink: webhook
  limits:
    max_requests_per_minute: 100
    max_tokens_per_day: 1000000
```

### Staging Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging
spec:
  kubeconfig: /etc/aof/kubeconfig-staging
  namespace: staging
  env:
    ENVIRONMENT: staging
    LOG_LEVEL: debug
    SLACK_CHANNEL: "#staging-alerts"
  approval:
    required: true
    allowed_users:
      - dev-team@example.com
    timeout_seconds: 300
  audit:
    enabled: true
    sink: file
  limits:
    max_requests_per_minute: 200
    max_tokens_per_day: 2000000
```

### Development Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: dev
spec:
  kubeconfig: ~/.kube/config-dev
  namespace: development
  env:
    ENVIRONMENT: development
    LOG_LEVEL: trace
    DEBUG: "true"
  approval:
    required: false
  audit:
    enabled: false
  limits:
    max_requests_per_minute: 500
    max_tokens_per_day: 5000000
```

### Minimal Context

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: local
spec:
  namespace: default
  env:
    ENVIRONMENT: local
```

## CLI Usage

### Using --context Flag

The `--context` (or `-C`) flag specifies which context to use for any operation:

```bash
# Run an agent with production context
aofctl run agent my-agent.yaml --context prod

# Run a flow with staging context
aofctl run flow incident-response.yaml -C staging

# Use context via environment variable
export AOFCTL_CONTEXT=prod
aofctl run agent my-agent.yaml  # Uses 'prod' context

# Specify contexts directory (default: ./contexts)
aofctl run agent my-agent.yaml --context prod --contexts-dir /etc/aof/contexts
```

When a context is specified:
- Environment variables from the context are injected into the agent's runtime
- Approval requirements are enforced before execution
- Audit logs include context information
- Rate limits are applied

### List Contexts

```bash
# List all contexts
aofctl get contexts

# Output:
# NAME       NAMESPACE      APPROVAL    AUDIT
# prod       production     true        enabled
# staging    staging        true        enabled
# dev        development    false       disabled
```

### Describe Context

```bash
# View detailed context configuration
aofctl describe context prod

# Output:
# Name:         prod
# Namespace:    production
# Kubeconfig:   /etc/aof/kubeconfig-prod
#
# Environment Variables:
#   ENVIRONMENT: production
#   LOG_LEVEL: info
#
# Approval:
#   Required: true
#   Allowed Users:
#     - alice@example.com
#     - bob@example.com
#     - oncall@example.com
#   Timeout: 600s
#
# Audit:
#   Enabled: true
#   Sink: webhook
#
# Limits:
#   Max Requests/Min: 100
#   Max Tokens/Day: 1000000
```

### Create Context

```bash
# Create context from YAML file
aofctl apply -f context-prod.yaml

# Create context inline
aofctl create context prod \
  --namespace=production \
  --kubeconfig=/etc/aof/kubeconfig-prod \
  --approval-required=true \
  --audit-enabled=true
```

### Update Context

```bash
# Update context from modified YAML
aofctl apply -f context-prod-updated.yaml

# Edit context interactively
aofctl edit context prod
```

### Delete Context

```bash
# Delete specific context
aofctl delete context dev

# Delete multiple contexts
aofctl delete contexts dev staging
```

## Use Cases

### Multi-Environment Deployments

Use different contexts to manage agent operations across production, staging, and development environments with appropriate safeguards:

- **Production**: Strict approval requirements, comprehensive audit logging, conservative rate limits
- **Staging**: Moderate approval requirements, audit logging, higher rate limits for testing
- **Development**: No approval required, optional audit logging, generous rate limits

### Multi-Tenant Operations

Create separate contexts for different teams or customers, each with isolated namespaces and resource limits:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: customer-acme
spec:
  namespace: tenant-acme
  env:
    TENANT_ID: acme
    BILLING_CODE: acme-prod
  approval:
    required: true
    allowed_users:
      - acme-admin@example.com
  limits:
    max_requests_per_minute: 50
    max_tokens_per_day: 500000
```

### Cost Control

Use token limits to prevent runaway costs:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: cost-controlled
spec:
  limits:
    max_tokens_per_day: 100000  # ~$2-3/day at typical rates
    max_requests_per_minute: 30
```

## Best Practices

1. **Approval for Production**: Always require approval for production contexts to prevent accidental destructive operations
2. **Audit Everything**: Enable audit logging for all non-development environments for compliance and debugging
3. **Environment Variables**: Use `${VAR}` syntax for secrets - AOF resolves from environment at runtime
4. **Namespace Isolation**: Use dedicated namespaces per context to prevent cross-contamination
5. **Rate Limits**: Set conservative limits to prevent API exhaustion and cost overruns
6. **Allowed Users**: Explicitly list allowed approvers for production contexts rather than allowing any user
7. **Timeout Tuning**: Set approval timeouts based on your organization's response time SLAs

## Related Resources

- [Trigger Resource](trigger.md) - Configure event sources that initiate agent workflows
- [FlowBinding Resource](binding.md) - Connect triggers, contexts, and flows together
- [Flow Resource](../concepts/flows.md) - Define agent workflows

## Kubernetes CRD Compatibility

AOF is designed with **Kubernetes Operator** compatibility in mind. The Context resource follows Kubernetes CRD conventions for future operator deployment.

### CRD Metadata

| Field | Value |
|-------|-------|
| API Group | `aof.dev` |
| Version | `v1` |
| Kind | `Context` |
| Scope | `Namespaced` |
| Plural | `contexts` |
| Singular | `context` |
| Short Names | `ctx` |

### Status Fields (Operator-Managed)

When deployed via a Kubernetes Operator, Context resources will include status fields:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  namespace: aof-system
spec:
  # User-defined spec fields (see above)
status:
  ready: true
  conditions:
    - type: Ready
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: ConfigurationValid
      message: "Context is ready for use"
    - type: KubeconfigValid
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: KubeconfigAccessible
      message: "Kubeconfig file is accessible and valid"
  lastAppliedConfiguration:
    kubeconfig: /etc/aof/kubeconfig-prod
    namespace: production
  observedGeneration: 1
```

### Status Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `ready` | boolean | Overall readiness status of the context |
| `conditions` | []Condition | Detailed status conditions |
| `lastAppliedConfiguration` | object | Snapshot of last successfully applied configuration |
| `observedGeneration` | integer | Generation of the resource that was last processed |

### Namespace Support

Contexts can be deployed:

- **Namespaced**: For tenant isolation (recommended for multi-tenant deployments)
  ```yaml
  apiVersion: aof.dev/v1
  kind: Context
  metadata:
    name: prod
    namespace: tenant-acme
  spec:
    # Context configuration
  ```

- **Cluster-scoped**: For shared contexts across all namespaces
  ```yaml
  apiVersion: aof.dev/v1
  kind: ClusterContext
  metadata:
    name: shared-dev
  spec:
    # Context configuration
  ```

### Cross-Namespace References

When referencing contexts from other resources (FlowBindings):

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack
  namespace: team-a
spec:
  trigger: slack-oncall
  context:
    name: prod
    namespace: aof-system  # Cross-namespace reference
  flow: k8s-troubleshoot
```

### Kubernetes Deployment

Once the AOF Operator is available, contexts can be deployed via kubectl:

```bash
# Apply context
kubectl apply -f context-prod.yaml

# Get contexts
kubectl get contexts -n aof-system

# Describe context
kubectl describe context prod -n aof-system

# View status
kubectl get context prod -n aof-system -o jsonpath='{.status.ready}'
```

> **Note**: Kubernetes Operator support is planned for a future release. See [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) for complete CRD definitions.

## See Also

- [Security & Approval](../concepts/security-approval.md) - Detailed approval workflow documentation
- [Multi-Tenancy](../guides/multi-tenancy.md) - Guide for multi-tenant deployments
- [CLI Reference](../cli/aofctl.md) - Complete aofctl command reference
- [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) - Complete CRD definitions and operator architecture
