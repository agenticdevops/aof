# FlowBinding Resource

## Overview

The `FlowBinding` resource connects triggers, contexts, and flows together. Bindings define which flows execute in which contexts when specific triggers fire, with optional pattern overrides for fine-grained control.

## API Reference

**apiVersion:** `aof.dev/v1`
**kind:** `FlowBinding`

## Specification

### FlowBinding Spec Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `trigger` | string | Yes | Name of the Trigger resource to bind |
| `context` | string | Yes | Name of the Context resource to use for execution |
| `flow` | string | Yes | Name of the Flow resource to execute |
| `match` | [MatchConfig](#matchconfig) | No | Optional pattern overrides to refine trigger matching |

### MatchConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `patterns` | []string | No | Additional regex patterns to match. Overrides trigger patterns if specified |
| `conditions` | map[string]string | No | Key-value conditions for conditional execution (e.g., environment, user roles) |

## How Bindings Work

Bindings create a many-to-many relationship between triggers, contexts, and flows:

1. **Trigger fires**: An event occurs (Slack message, webhook, schedule tick, PagerDuty alert)
2. **Binding matches**: AOF finds all bindings referencing the trigger
3. **Pattern check**: If binding has `match.patterns`, event must match those patterns
4. **Condition check**: If binding has `match.conditions`, event metadata must satisfy conditions
5. **Flow execution**: Matching flow executes in the specified context

### Binding Resolution Flow

```
Trigger Event → Find Bindings → Check Patterns → Check Conditions → Execute Flow in Context
```

### Multiple Bindings

A single trigger can have multiple bindings, allowing:
- Same trigger, different flows per environment (prod vs staging)
- Same trigger, different contexts per team (team-a vs team-b)
- Same trigger, multiple flows for complex workflows

## Examples

### Basic Binding

Connect Slack trigger to Kubernetes troubleshooting flow in production context:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-k8s-prod
spec:
  trigger: slack-oncall
  context: prod
  flow: k8s-troubleshoot
```

### Binding with Pattern Override

Refine trigger patterns for specific binding:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-incident-prod
spec:
  trigger: slack-oncall
  context: prod
  flow: incident-response
  match:
    patterns:
      - "^incident.*critical"
      - "P0.*help"
```

### Multi-Environment Bindings

Same trigger executes different flows based on environment:

```yaml
# Production binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack
spec:
  trigger: slack-oncall
  context: prod
  flow: k8s-troubleshoot
  match:
    patterns:
      - "@agent.*prod"
---
# Staging binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: staging-slack
spec:
  trigger: slack-oncall
  context: staging
  flow: k8s-troubleshoot
  match:
    patterns:
      - "@agent.*staging"
---
# Development binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: dev-telegram
spec:
  trigger: telegram-ops
  context: dev
  flow: k8s-troubleshoot
```

### Multi-Tenant Bindings

Different teams with isolated contexts:

```yaml
# Team A binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: team-a-support
spec:
  trigger: slack-support
  context: team-a-prod
  flow: customer-support
  match:
    conditions:
      team: team-a
---
# Team B binding
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: team-b-support
spec:
  trigger: slack-support
  context: team-b-prod
  flow: customer-support
  match:
    conditions:
      team: team-b
```

### Conditional Execution

Execute different flows based on event metadata:

```yaml
# High priority incidents → immediate response
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: high-priority-incident
spec:
  trigger: pagerduty-incidents
  context: prod
  flow: incident-response-urgent
  match:
    conditions:
      urgency: high
      service: production-api
---
# Low priority incidents → automated triage
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: low-priority-incident
spec:
  trigger: pagerduty-incidents
  context: prod
  flow: incident-triage-automated
  match:
    conditions:
      urgency: low
```

### Scheduled Maintenance

Different flows for different schedules:

```yaml
# Daily health check
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: daily-health-check
spec:
  trigger: daily-schedule
  context: prod
  flow: health-check
---
# Weekly security scan
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: weekly-security-scan
spec:
  trigger: weekly-schedule
  context: prod
  flow: security-audit
---
# Monthly cost report
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: monthly-cost-report
spec:
  trigger: monthly-schedule
  context: prod
  flow: cost-analysis
```

### Webhook to Multiple Flows

GitHub webhook triggers multiple flows:

```yaml
# CI/CD flow
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: github-cicd
spec:
  trigger: github-webhook
  context: prod
  flow: cicd-pipeline
  match:
    patterns:
      - "refs/heads/main"
---
# Security scan flow
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: github-security
spec:
  trigger: github-webhook
  context: prod
  flow: security-scan
  match:
    patterns:
      - ".*\\.(py|js|go|rs)$"
---
# Deployment notification flow
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: github-notify
spec:
  trigger: github-webhook
  context: prod
  flow: deployment-notify
```

## CLI Usage

### List Bindings

```bash
# List all bindings
aofctl get bindings

# Output:
# NAME              TRIGGER          CONTEXT    FLOW
# prod-slack        slack-oncall     prod       k8s-troubleshoot
# staging-slack     slack-oncall     staging    k8s-troubleshoot
# dev-telegram      telegram-ops     dev        k8s-troubleshoot
```

### Describe Binding

```bash
# View detailed binding configuration
aofctl describe binding prod-slack

# Output:
# Name:         prod-slack
#
# Trigger:      slack-oncall
#   Type:       Slack
#   Channels:   2
#
# Context:      prod
#   Namespace:  production
#   Approval:   Required
#   Audit:      Enabled
#
# Flow:         k8s-troubleshoot
#   Agent:      KubernetesTroubleshooter
#   Tools:      kubectl, logs, describe
#
# Match Patterns:
#   - @agent.*prod
#
# Status:
#   Active:     true
#   Last Run:   2025-12-18T10:45:23Z
#   Success:    98.5%
```

### Create Binding

```bash
# Create binding from YAML file
aofctl apply -f binding-prod-slack.yaml

# Create binding inline
aofctl create binding prod-slack \
  --trigger=slack-oncall \
  --context=prod \
  --flow=k8s-troubleshoot
```

### Update Binding

```bash
# Update binding from modified YAML
aofctl apply -f binding-prod-slack-updated.yaml

# Edit binding interactively
aofctl edit binding prod-slack
```

### Delete Binding

```bash
# Delete specific binding
aofctl delete binding prod-slack

# Delete multiple bindings
aofctl delete bindings prod-slack staging-slack
```

### Validate Binding

```bash
# Validate binding configuration
aofctl validate binding prod-slack

# Output:
# Validating binding 'prod-slack'...
# ✓ Trigger 'slack-oncall' exists
# ✓ Context 'prod' exists
# ✓ Flow 'k8s-troubleshoot' exists
# ✓ Match patterns are valid regex
# ✓ All conditions are supported
```

### Dry-Run Binding

```bash
# Test binding without executing flow
aofctl dry-run binding prod-slack \
  --message="@agent check pod status in prod"

# Output:
# Dry-run binding 'prod-slack'
#
# Trigger Match: ✓ (pattern: @agent.*)
# Context:       prod
# Flow:          k8s-troubleshoot
#
# Would execute:
#   Agent: KubernetesTroubleshooter
#   Context: prod (namespace: production)
#   Approval: Required (timeout: 600s)
#   Tools: kubectl, logs, describe
```

## Common Patterns

### Environment Promotion Pipeline

Gradual rollout across environments:

```yaml
# Dev environment - fast iteration, no approval
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: dev-deploy
spec:
  trigger: github-webhook
  context: dev
  flow: deploy-service
  match:
    patterns:
      - "refs/heads/dev"
---
# Staging environment - approval required
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: staging-deploy
spec:
  trigger: github-webhook
  context: staging
  flow: deploy-service
  match:
    patterns:
      - "refs/heads/staging"
---
# Production environment - strict approval + audit
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-deploy
spec:
  trigger: github-webhook
  context: prod
  flow: deploy-service
  match:
    patterns:
      - "refs/heads/main"
```

### On-Call Routing

Route incidents to appropriate teams:

```yaml
# Database incidents → DBA team
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: oncall-database
spec:
  trigger: pagerduty-incidents
  context: dba-team
  flow: database-incident
  match:
    conditions:
      service: database
---
# API incidents → Backend team
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: oncall-api
spec:
  trigger: pagerduty-incidents
  context: backend-team
  flow: api-incident
  match:
    conditions:
      service: api
---
# Frontend incidents → Frontend team
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: oncall-frontend
spec:
  trigger: pagerduty-incidents
  context: frontend-team
  flow: frontend-incident
  match:
    conditions:
      service: frontend
```

### Progressive Automation

Start with manual approval, gradually automate:

```yaml
# Phase 1: Manual approval for everything
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: manual-remediation
spec:
  trigger: slack-oncall
  context: prod-manual
  flow: incident-remediation
---
# Phase 2: Auto-approve low-risk operations
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: semi-auto-remediation
spec:
  trigger: slack-oncall
  context: prod-semi-auto
  flow: incident-remediation
  match:
    conditions:
      risk: low
---
# Phase 3: Fully automated for known issues
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: auto-remediation
spec:
  trigger: pagerduty-incidents
  context: prod-auto
  flow: incident-remediation
  match:
    conditions:
      known_issue: "true"
```

## Best Practices

1. **Explicit Naming**: Use descriptive binding names that indicate trigger, context, and purpose (e.g., `prod-slack-k8s-troubleshoot`)
2. **Pattern Specificity**: Use `match.patterns` to avoid unintended flow executions
3. **Condition Guards**: Use `match.conditions` for conditional logic rather than duplicating flows
4. **Environment Isolation**: Always use separate bindings for prod/staging/dev to prevent cross-environment issues
5. **Validation**: Run `aofctl validate binding` before deploying to production
6. **Dry-Run Testing**: Use `aofctl dry-run binding` to verify matching logic
7. **Monitoring**: Track binding execution success rates to identify problematic bindings
8. **Documentation**: Add comments in YAML explaining complex pattern or condition logic

## Troubleshooting

### Binding Not Triggering

```bash
# Check binding status
aofctl describe binding prod-slack

# Validate binding configuration
aofctl validate binding prod-slack

# Test pattern matching
aofctl dry-run binding prod-slack \
  --message="your test message"

# Check trigger status
aofctl describe trigger slack-oncall

# View binding execution logs
aofctl logs binding prod-slack
```

### Multiple Bindings Triggering

When multiple bindings match the same trigger event, AOF executes all matching bindings in parallel. To prevent this:

1. **Use exclusive patterns**: Ensure `match.patterns` don't overlap
2. **Use conditions**: Add `match.conditions` to differentiate bindings
3. **Priority-based execution**: Use labels to control execution order

### Pattern Not Matching

```bash
# Test pattern with actual message
aofctl dry-run binding prod-slack \
  --message="@agent check pod status"

# Output will show which patterns matched/failed
```

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Binding always triggers | No pattern override | Add `match.patterns` to refine matching |
| Binding never triggers | Pattern too specific | Simplify `match.patterns` or check trigger patterns |
| Wrong context used | Binding references wrong context | Verify `context` field references correct Context resource |
| Flow not found | Flow doesn't exist | Check `flow` field and verify Flow resource exists |
| Approval timeout | Context has strict timeout | Increase `approval.timeout_seconds` in Context |

## Advanced Use Cases

### Blue-Green Deployment

Route traffic between blue and green environments:

```yaml
# Blue environment
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: deploy-blue
spec:
  trigger: deployment-webhook
  context: prod-blue
  flow: deploy-service
  match:
    conditions:
      target: blue
---
# Green environment
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: deploy-green
spec:
  trigger: deployment-webhook
  context: prod-green
  flow: deploy-service
  match:
    conditions:
      target: green
```

### Compliance & Audit

Separate bindings for audit trail:

```yaml
# Normal operations
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-normal-ops
spec:
  trigger: slack-oncall
  context: prod
  flow: k8s-troubleshoot
---
# Compliance-sensitive operations
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-compliance-ops
spec:
  trigger: slack-oncall
  context: prod-audit
  flow: k8s-troubleshoot
  match:
    patterns:
      - "compliance.*"
      - "pci.*"
      - "hipaa.*"
```

### Cost Optimization

Different contexts for cost control:

```yaml
# Standard operations (normal limits)
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: standard-ops
spec:
  trigger: slack-oncall
  context: prod-standard
  flow: k8s-troubleshoot
---
# Budget-sensitive operations (strict limits)
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: budget-ops
spec:
  trigger: slack-oncall
  context: prod-budget
  flow: k8s-troubleshoot
  match:
    conditions:
      customer_tier: free
```

## Related Resources

- [Context Resource](context.md) - Define execution environments
- [Trigger Resource](trigger.md) - Configure event sources
- [Flow Resource](../concepts/flows.md) - Define agent workflows

## Kubernetes CRD Compatibility

FlowBinding resources are designed for **Kubernetes Operator** deployment, enabling composable multi-tenant architectures.

### CRD Metadata

| Field | Value |
|-------|-------|
| API Group | `aof.dev` |
| Version | `v1` |
| Kind | `FlowBinding` |
| Scope | `Namespaced` |
| Plural | `flowbindings` |
| Singular | `flowbinding` |
| Short Names | `fb`, `binding` |

### Status Fields (Operator-Managed)

When deployed via a Kubernetes Operator, FlowBinding resources track execution status:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack-k8s
  namespace: workflows
spec:
  # User-defined spec fields (see above)
status:
  active: true
  phase: Bound
  conditions:
    - type: Bound
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: AllReferencesValid
      message: "All referenced resources exist and are valid"
    - type: TriggerReady
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: TriggerConnected
      message: "Trigger 'slack-oncall' is connected"
    - type: ContextReady
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: ContextValid
      message: "Context 'prod' is ready"
  matchCount: 47
  executionCount: 45
  successCount: 44
  failureCount: 1
  lastTriggered:
    timestamp: "2025-12-18T10:45:23Z"
    eventId: "evt_abc123"
    successful: true
    duration: "2.3s"
  observedGeneration: 1
```

### Status Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `active` | boolean | Whether binding is active and processing events |
| `phase` | string | Binding phase: `Pending`, `Bound`, `Failed`, `Suspended` |
| `conditions` | []Condition | Detailed status conditions |
| `matchCount` | integer | Number of trigger events that matched this binding |
| `executionCount` | integer | Number of flow executions initiated |
| `successCount` | integer | Number of successful executions |
| `failureCount` | integer | Number of failed executions |
| `lastTriggered` | object | Information about most recent execution |
| `observedGeneration` | integer | Generation of resource last processed |

### Resource Resolution Status

Bindings report the status of all referenced resources:

```yaml
status:
  resourceStatus:
    trigger:
      name: slack-oncall
      namespace: team-production
      ready: true
      connected: true
    context:
      name: prod
      namespace: aof-system
      ready: true
      kubeconfigValid: true
    flow:
      name: k8s-troubleshoot
      namespace: workflows
      ready: true
      agentAvailable: true
```

### Cross-Namespace References

FlowBindings support cross-namespace resource references for flexible multi-tenant architectures:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: team-a-production
  namespace: team-a-workflows
spec:
  trigger:
    name: slack-prod           # In same namespace
  context:
    name: prod-cluster
    namespace: shared-contexts  # Cross-namespace
  flow:
    name: k8s-troubleshoot
    namespace: shared-flows     # Cross-namespace
```

### Execution Metrics

Bindings expose detailed execution metrics:

```yaml
status:
  metrics:
    averageExecutionTime: "2.1s"
    p95ExecutionTime: "4.8s"
    p99ExecutionTime: "7.2s"
    successRate: 0.978
    lastHourExecutions: 12
    lastDayExecutions: 287
```

### Pattern Matching Statistics

Track how often binding patterns match:

```yaml
status:
  patternMatching:
    totalEvents: 1000
    matchedEvents: 47
    matchRate: 0.047
    patterns:
      - pattern: "^@agent.*prod"
        matches: 35
      - pattern: "incident.*critical"
        matches: 12
```

### Namespace Support

FlowBindings are namespaced for tenant isolation:

```yaml
# Team A's bindings
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack
  namespace: team-a-workflows
spec:
  trigger: team-a-slack
  context: team-a-prod
  flow: k8s-troubleshoot

---
# Team B's bindings (isolated)
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack
  namespace: team-b-workflows
spec:
  trigger: team-b-slack
  context: team-b-prod
  flow: k8s-troubleshoot
```

### Kubernetes Deployment

Once the AOF Operator is available, bindings can be managed via kubectl:

```bash
# Apply binding
kubectl apply -f binding-prod-slack.yaml

# Get bindings
kubectl get flowbindings -n workflows

# Describe binding with status
kubectl describe flowbinding prod-slack-k8s -n workflows

# Check binding activity
kubectl get flowbinding prod-slack-k8s -n workflows -o jsonpath='{.status.active}'

# View execution count
kubectl get flowbinding prod-slack-k8s -n workflows -o jsonpath='{.status.executionCount}'

# View success rate
kubectl get flowbinding prod-slack-k8s -n workflows -o jsonpath='{.status.metrics.successRate}'

# List all bindings for a trigger
kubectl get flowbindings -n workflows -l trigger=slack-oncall

# Monitor binding executions
kubectl get flowbindings -n workflows --watch
```

### RBAC Considerations

Cross-namespace references require appropriate RBAC permissions:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: flowbinding-reader
  namespace: shared-contexts
rules:
  - apiGroups: ["aof.dev"]
    resources: ["contexts"]
    verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: team-a-flowbinding-reader
  namespace: shared-contexts
subjects:
  - kind: ServiceAccount
    name: aof-operator
    namespace: aof-system
roleRef:
  kind: Role
  name: flowbinding-reader
  apiGroup: rbac.authorization.k8s.io
```

### Webhook Configuration

For validating webhooks to ensure referenced resources exist:

```yaml
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingWebhookConfiguration
metadata:
  name: aof-flowbinding-validator
webhooks:
  - name: flowbinding.aof.dev
    rules:
      - apiGroups: ["aof.dev"]
        apiVersions: ["v1"]
        operations: ["CREATE", "UPDATE"]
        resources: ["flowbindings"]
    # Validates that trigger, context, and flow all exist
```

> **Note**: Kubernetes Operator support is planned for a future release. See [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) for complete CRD definitions.

## See Also

- [Architecture Overview](../concepts/architecture.md) - Understanding AOF's composable architecture
- [Configuration Guide](../guides/configuration.md) - Comprehensive configuration examples
- [Security & Approval](../concepts/security-approval.md) - Approval workflow details
- [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) - Complete CRD definitions and operator architecture
