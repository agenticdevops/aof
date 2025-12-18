# Execution Contexts

Contexts define **environment-specific configuration**: kubeconfig, approval rules, audit logging, resource limits.

## Available Contexts

- **`prod.yaml`** - Production environment
  - Strict approvals for destructive ops
  - Full audit logging to S3
  - Rate limiting: 20 req/min
  - Timeout: 5 minutes

- **`staging.yaml`** - Staging environment
  - Relaxed approvals
  - Basic audit logging
  - Rate limiting: 40 req/min
  - Timeout: 3 minutes

- **`dev.yaml`** - Development environment
  - No approvals
  - No audit logging
  - Rate limiting: 100 req/min
  - Timeout: 1 minute

## What Contexts Configure

\`\`\`yaml
spec:
  # K8s cluster access
  kubeconfig: /path/to/kubeconfig
  namespace: default
  cluster_name: prod-cluster

  # Approval workflow
  approval:
    enabled: true
    required_for: [delete, scale_down]
    allowed_users: [U012USER1, U012USER2]
    timeout_seconds: 300

  # Audit logging
  audit:
    enabled: true
    log_commands: true
    destination: s3://audit-logs/

  # Resource limits
  limits:
    max_concurrent_operations: 5
    rate_limit_per_minute: 20
    timeout_seconds: 300

  # Environment variables
  env:
    ENVIRONMENT: production
    LOG_LEVEL: info

  # Notifications
  notifications:
    slack_channel: "#prod-alerts"
\`\`\`

## Usage

Contexts are referenced in **bindings**:

\`\`\`yaml
spec:
  trigger: triggers/slack-prod.yaml
  flow: flows/k8s-ops-flow.yaml
  context: contexts/prod.yaml  # Environment config
\`\`\`

## Multi-Tenant Pattern

Use different contexts for same flow:

| Binding | Trigger | Flow | Context | Result |
|---------|---------|------|---------|--------|
| prod-slack-k8s | Slack #prod | k8s-ops | prod | Strict, audited |
| staging-slack-k8s | Slack #staging | k8s-ops | staging | Relaxed |
| dev-slack-k8s | Slack #dev | k8s-ops | dev | Permissive |
