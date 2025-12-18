# Kubernetes Custom Resource Definitions (CRDs)

This document provides complete CRD definitions for all AOF resources, designed for future Kubernetes Operator deployment.

## Overview

AOF resources follow Kubernetes CRD conventions, enabling native Kubernetes deployment via an operator. This design provides:

- **Declarative Configuration**: GitOps-ready YAML specifications
- **Native kubectl Integration**: Standard kubectl commands work seamlessly
- **Multi-Tenancy**: Namespace-based isolation for teams and environments
- **Status Tracking**: Operator-managed runtime status fields
- **Cross-Namespace References**: Flexible resource composition
- **RBAC Integration**: Standard Kubernetes authorization

## Resource Model

AOF defines **6 core resources** organized into two layers:

```
┌────────────────────────────────────────────────────────────┐
│                    AOF Resource Model                       │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │ Agent   │  │ Fleet   │  │ Flow    │  ← Execution       │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │ Context │  │ Trigger │  │FlowBindng│  ← Configuration  │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Execution Resources

| Resource | Description | Kubernetes Analog |
|----------|-------------|-------------------|
| **Agent** | Single AI assistant | Pod |
| **Fleet** | Team of coordinated agents | Deployment |
| **Flow** | Event-driven workflow | Argo Workflow |

### Configuration Resources

| Resource | Description | Kubernetes Analog |
|----------|-------------|-------------------|
| **Context** | Environment configuration | ConfigMap |
| **Trigger** | Event source | CronJob trigger |
| **FlowBinding** | Connects Trigger → Context → Flow | Binding |

## Complete CRD Definitions

### 1. Context CRD

Defines execution environment and constraints for agent operations.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: contexts.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                kubeconfig:
                  type: string
                  description: "Path to kubeconfig file"
                namespace:
                  type: string
                  description: "Kubernetes namespace for operations"
                  default: default
                env:
                  type: object
                  additionalProperties:
                    type: string
                  description: "Environment variables for agent execution"
                approval:
                  type: object
                  properties:
                    required:
                      type: boolean
                      description: "Whether approval is required"
                    allowed_users:
                      type: array
                      items:
                        type: string
                      description: "Users who can approve operations"
                    timeout_seconds:
                      type: integer
                      default: 300
                      description: "Approval timeout in seconds"
                audit:
                  type: object
                  properties:
                    enabled:
                      type: boolean
                      description: "Enable audit logging"
                    sink:
                      type: string
                      enum: [stdout, file, webhook]
                      default: stdout
                      description: "Audit log destination"
                limits:
                  type: object
                  properties:
                    max_requests_per_minute:
                      type: integer
                      description: "Maximum API requests per minute"
                    max_tokens_per_day:
                      type: integer
                      description: "Maximum LLM tokens per day"
            status:
              type: object
              properties:
                ready:
                  type: boolean
                  description: "Overall readiness status"
                conditions:
                  type: array
                  items:
                    type: object
                    properties:
                      type:
                        type: string
                      status:
                        type: string
                        enum: ["True", "False", "Unknown"]
                      lastTransitionTime:
                        type: string
                        format: date-time
                      reason:
                        type: string
                      message:
                        type: string
                lastAppliedConfiguration:
                  type: object
                  x-kubernetes-preserve-unknown-fields: true
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Ready
          type: boolean
          jsonPath: .status.ready
        - name: Namespace
          type: string
          jsonPath: .spec.namespace
        - name: Approval
          type: string
          jsonPath: .spec.approval.required
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: contexts
    singular: context
    kind: Context
    shortNames:
      - ctx
```

### 2. Trigger CRD

Defines event sources that initiate agent workflows.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: triggers.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - type
                - config
              properties:
                type:
                  type: string
                  enum: [Slack, Telegram, Discord, HTTP, Schedule, PagerDuty, Jira]
                  description: "Trigger type"
                config:
                  type: object
                  x-kubernetes-preserve-unknown-fields: true
                  description: "Type-specific configuration"
            status:
              type: object
              properties:
                connected:
                  type: boolean
                  description: "Connection status to platform"
                phase:
                  type: string
                  enum: [Pending, Active, Failed, Disconnected]
                  description: "Trigger phase"
                conditions:
                  type: array
                  items:
                    type: object
                    properties:
                      type:
                        type: string
                      status:
                        type: string
                        enum: ["True", "False", "Unknown"]
                      lastTransitionTime:
                        type: string
                        format: date-time
                      reason:
                        type: string
                      message:
                        type: string
                lastEvent:
                  type: object
                  properties:
                    timestamp:
                      type: string
                      format: date-time
                    type:
                      type: string
                    source:
                      type: string
                    user:
                      type: string
                eventCount:
                  type: integer
                  description: "Total events processed"
                errorCount:
                  type: integer
                  description: "Total errors encountered"
                webhookEndpoint:
                  type: string
                  description: "Operator-assigned webhook URL (HTTP triggers)"
                platformStatus:
                  type: object
                  x-kubernetes-preserve-unknown-fields: true
                  description: "Platform-specific status"
                metrics:
                  type: object
                  properties:
                    eventsReceived:
                      type: integer
                    eventsProcessed:
                      type: integer
                    eventsDropped:
                      type: integer
                    averageProcessingTime:
                      type: string
                    errorRate:
                      type: number
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Type
          type: string
          jsonPath: .spec.type
        - name: Connected
          type: boolean
          jsonPath: .status.connected
        - name: Phase
          type: string
          jsonPath: .status.phase
        - name: Events
          type: integer
          jsonPath: .status.eventCount
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: triggers
    singular: trigger
    kind: Trigger
    shortNames:
      - trg
```

### 3. FlowBinding CRD

Connects triggers, contexts, and flows together.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: flowbindings.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - trigger
                - context
                - flow
              properties:
                trigger:
                  type: object
                  properties:
                    name:
                      type: string
                    namespace:
                      type: string
                  description: "Reference to Trigger resource"
                context:
                  type: object
                  properties:
                    name:
                      type: string
                    namespace:
                      type: string
                  description: "Reference to Context resource"
                flow:
                  type: object
                  properties:
                    name:
                      type: string
                    namespace:
                      type: string
                  description: "Reference to Flow resource"
                match:
                  type: object
                  properties:
                    patterns:
                      type: array
                      items:
                        type: string
                      description: "Regex patterns to match"
                    conditions:
                      type: object
                      additionalProperties:
                        type: string
                      description: "Key-value conditions"
            status:
              type: object
              properties:
                active:
                  type: boolean
                  description: "Whether binding is active"
                phase:
                  type: string
                  enum: [Pending, Bound, Failed, Suspended]
                  description: "Binding phase"
                conditions:
                  type: array
                  items:
                    type: object
                    properties:
                      type:
                        type: string
                      status:
                        type: string
                        enum: ["True", "False", "Unknown"]
                      lastTransitionTime:
                        type: string
                        format: date-time
                      reason:
                        type: string
                      message:
                        type: string
                matchCount:
                  type: integer
                  description: "Events matched"
                executionCount:
                  type: integer
                  description: "Flows executed"
                successCount:
                  type: integer
                  description: "Successful executions"
                failureCount:
                  type: integer
                  description: "Failed executions"
                lastTriggered:
                  type: object
                  properties:
                    timestamp:
                      type: string
                      format: date-time
                    eventId:
                      type: string
                    successful:
                      type: boolean
                    duration:
                      type: string
                resourceStatus:
                  type: object
                  properties:
                    trigger:
                      type: object
                      x-kubernetes-preserve-unknown-fields: true
                    context:
                      type: object
                      x-kubernetes-preserve-unknown-fields: true
                    flow:
                      type: object
                      x-kubernetes-preserve-unknown-fields: true
                metrics:
                  type: object
                  properties:
                    averageExecutionTime:
                      type: string
                    p95ExecutionTime:
                      type: string
                    p99ExecutionTime:
                      type: string
                    successRate:
                      type: number
                    lastHourExecutions:
                      type: integer
                    lastDayExecutions:
                      type: integer
                patternMatching:
                  type: object
                  properties:
                    totalEvents:
                      type: integer
                    matchedEvents:
                      type: integer
                    matchRate:
                      type: number
                    patterns:
                      type: array
                      items:
                        type: object
                        properties:
                          pattern:
                            type: string
                          matches:
                            type: integer
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Trigger
          type: string
          jsonPath: .spec.trigger.name
        - name: Context
          type: string
          jsonPath: .spec.context.name
        - name: Flow
          type: string
          jsonPath: .spec.flow.name
        - name: Active
          type: boolean
          jsonPath: .status.active
        - name: Executions
          type: integer
          jsonPath: .status.executionCount
        - name: Success Rate
          type: string
          jsonPath: .status.metrics.successRate
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: flowbindings
    singular: flowbinding
    kind: FlowBinding
    shortNames:
      - fb
      - binding
```

### 4. Agent CRD

Single AI assistant with specific instructions and tools.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: agents.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - model
                - instructions
              properties:
                model:
                  type: string
                  description: "LLM provider and model (e.g., google:gemini-2.5-flash)"
                instructions:
                  type: string
                  description: "System prompt / agent personality"
                tools:
                  type: array
                  items:
                    type: object
                    x-kubernetes-preserve-unknown-fields: true
                  description: "Tools available to agent"
                memory:
                  type: object
                  properties:
                    type:
                      type: string
                      enum: [InMemory, File, SQLite, PostgreSQL]
                    config:
                      type: object
                      x-kubernetes-preserve-unknown-fields: true
                temperature:
                  type: number
                  minimum: 0
                  maximum: 1
                  description: "Response creativity (0-1)"
                max_tokens:
                  type: integer
                  description: "Maximum response length"
            status:
              type: object
              properties:
                ready:
                  type: boolean
                phase:
                  type: string
                  enum: [Pending, Ready, Failed]
                conditions:
                  type: array
                  items:
                    type: object
                lastRun:
                  type: string
                  format: date-time
                totalRuns:
                  type: integer
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Model
          type: string
          jsonPath: .spec.model
        - name: Ready
          type: boolean
          jsonPath: .status.ready
        - name: Total Runs
          type: integer
          jsonPath: .status.totalRuns
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: agents
    singular: agent
    kind: Agent
```

### 5. AgentFleet CRD

Team of coordinated agents for parallel processing.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: agentfleets.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - agents
              properties:
                agents:
                  type: array
                  items:
                    type: object
                    properties:
                      name:
                        type: string
                      role:
                        type: string
                        enum: [manager, worker]
                      replicas:
                        type: integer
                        minimum: 1
                        default: 1
                      spec:
                        type: object
                        x-kubernetes-preserve-unknown-fields: true
                coordination:
                  type: object
                  properties:
                    mode:
                      type: string
                      enum: [hierarchical, peer, swarm]
                    distribution:
                      type: string
                      enum: [parallel, sequential, adaptive]
                    consensus:
                      type: string
                      enum: [majority, unanimous, first]
            status:
              type: object
              properties:
                ready:
                  type: boolean
                phase:
                  type: string
                  enum: [Pending, Ready, Failed]
                conditions:
                  type: array
                  items:
                    type: object
                agentStatus:
                  type: array
                  items:
                    type: object
                    properties:
                      name:
                        type: string
                      ready:
                        type: boolean
                      replicas:
                        type: integer
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Ready
          type: boolean
          jsonPath: .status.ready
        - name: Agents
          type: integer
          jsonPath: .spec.agents[*].replicas
        - name: Mode
          type: string
          jsonPath: .spec.coordination.mode
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: agentfleets
    singular: agentfleet
    kind: AgentFleet
    shortNames:
      - fleet
```

### 6. AgentFlow CRD

Event-driven workflow with triggers and nodes.

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: agentflows.aof.dev
spec:
  group: aof.dev
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - trigger
                - nodes
                - connections
              properties:
                trigger:
                  type: object
                  properties:
                    type:
                      type: string
                    config:
                      type: object
                      x-kubernetes-preserve-unknown-fields: true
                context:
                  type: object
                  x-kubernetes-preserve-unknown-fields: true
                nodes:
                  type: array
                  items:
                    type: object
                    properties:
                      id:
                        type: string
                      type:
                        type: string
                      config:
                        type: object
                        x-kubernetes-preserve-unknown-fields: true
                connections:
                  type: array
                  items:
                    type: object
                    properties:
                      from:
                        type: string
                      to:
                        type: string
            status:
              type: object
              properties:
                ready:
                  type: boolean
                phase:
                  type: string
                  enum: [Pending, Running, Succeeded, Failed]
                conditions:
                  type: array
                  items:
                    type: object
                lastExecution:
                  type: string
                  format: date-time
                totalExecutions:
                  type: integer
                observedGeneration:
                  type: integer
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Trigger
          type: string
          jsonPath: .spec.trigger.type
        - name: Ready
          type: boolean
          jsonPath: .status.ready
        - name: Executions
          type: integer
          jsonPath: .status.totalExecutions
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: agentflows
    singular: agentflow
    kind: AgentFlow
    shortNames:
      - flow
```

## AOF Operator Architecture

The future AOF Operator will manage these CRDs in a Kubernetes cluster.

### Operator Components

```
┌─────────────────────────────────────────────────────────────┐
│                      AOF Operator                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐           │
│  │ CRD Controller   │        │ Webhook Server   │           │
│  │ - Reconcile loop │        │ - Validation     │           │
│  │ - Status updates │        │ - Mutation       │           │
│  └──────────────────┘        └──────────────────┘           │
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐           │
│  │ Trigger Manager  │        │ Flow Executor    │           │
│  │ - Platform conns │        │ - Workflow engine│           │
│  │ - Event routing  │        │ - Agent runtime  │           │
│  └──────────────────┘        └──────────────────┘           │
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐           │
│  │ Metrics Server   │        │ Audit Logger     │           │
│  │ - Prometheus     │        │ - Event logs     │           │
│  │ - Custom metrics │        │ - Compliance     │           │
│  └──────────────────┘        └──────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Installation (Future)

Once the operator is available, installation will be via Helm:

```bash
# Add AOF Helm repository
helm repo add aof https://charts.aof.dev
helm repo update

# Install operator with CRDs
helm install aof-operator aof/aof-operator \
  --namespace aof-system \
  --create-namespace \
  --set operator.image.tag=v1.0.0

# Verify installation
kubectl get crds | grep aof.dev
kubectl get pods -n aof-system
```

### RBAC Requirements

The operator requires cluster-level permissions:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: aof-operator
rules:
  # CRD management
  - apiGroups: ["aof.dev"]
    resources: ["*"]
    verbs: ["*"]
  # Status updates
  - apiGroups: ["aof.dev"]
    resources: ["*/status"]
    verbs: ["get", "update", "patch"]
  # Event recording
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["create", "patch"]
  # Secrets for credentials
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list", "watch"]
  # ConfigMaps for configuration
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list", "watch"]
```

## Usage Examples

### Deploy Resources via kubectl

```bash
# Apply all resources in a directory
kubectl apply -f ./aof-resources/

# Create context
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  namespace: aof-system
spec:
  kubeconfig: /etc/aof/kubeconfig-prod
  namespace: production
  approval:
    required: true
    allowed_users:
      - alice@example.com
  audit:
    enabled: true
    sink: webhook
EOF

# Create trigger
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-oncall
  namespace: aof-system
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    app_token: ${SLACK_APP_TOKEN}
    channels:
      - C01234567
EOF

# Create binding
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-slack
  namespace: workflows
spec:
  trigger:
    name: slack-oncall
    namespace: aof-system
  context:
    name: prod
    namespace: aof-system
  flow:
    name: k8s-troubleshoot
    namespace: workflows
EOF
```

### Monitor Resources

```bash
# Watch all AOF resources
kubectl get contexts,triggers,flowbindings -A --watch

# Check binding status
kubectl describe flowbinding prod-slack -n workflows

# View trigger events
kubectl get trigger slack-oncall -n aof-system \
  -o jsonpath='{.status.eventCount}'

# Check context readiness
kubectl get context prod -n aof-system \
  -o jsonpath='{.status.ready}'

# View metrics
kubectl get flowbinding prod-slack -n workflows \
  -o jsonpath='{.status.metrics.successRate}'
```

### GitOps Integration

Store AOF resources in Git and use ArgoCD/Flux:

```yaml
# argocd-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: aof-production
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/example/aof-configs
    targetRevision: main
    path: production
  destination:
    server: https://kubernetes.default.svc
    namespace: aof-system
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

## Multi-Tenancy Patterns

### Namespace Per Team

```bash
# Create team namespaces
kubectl create namespace team-a
kubectl create namespace team-b

# Team A resources
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  namespace: team-a
spec:
  namespace: team-a-prod
  approval:
    required: true
    allowed_users:
      - team-a-admin@example.com
---
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack
  namespace: team-a
spec:
  type: Slack
  config:
    channels: [C_TEAM_A]
---
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-k8s
  namespace: team-a
spec:
  trigger: { name: slack }
  context: { name: prod }
  flow: { name: k8s-troubleshoot }
EOF

# Team B gets isolated resources
kubectl apply -f team-b-resources.yaml -n team-b
```

### Shared Resources with Cross-Namespace References

```bash
# Shared contexts (cluster-wide)
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: shared-dev
  namespace: shared-contexts
spec:
  namespace: development
  approval:
    required: false
EOF

# Teams reference shared context
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: dev-slack
  namespace: team-a
spec:
  trigger: { name: slack }
  context:
    name: shared-dev
    namespace: shared-contexts  # Cross-namespace
  flow: { name: k8s-troubleshoot }
EOF
```

## Monitoring and Observability

### Prometheus Metrics

The operator exposes metrics:

```
# Context metrics
aof_context_ready{name="prod", namespace="aof-system"} 1
aof_context_approval_required{name="prod"} 1

# Trigger metrics
aof_trigger_connected{name="slack-oncall", type="Slack"} 1
aof_trigger_events_total{name="slack-oncall"} 1420
aof_trigger_errors_total{name="slack-oncall"} 2

# Binding metrics
aof_binding_executions_total{name="prod-slack"} 287
aof_binding_success_total{name="prod-slack"} 281
aof_binding_failure_total{name="prod-slack"} 6
aof_binding_success_rate{name="prod-slack"} 0.979
aof_binding_execution_duration_seconds{name="prod-slack", quantile="0.95"} 4.8
```

### Grafana Dashboard

Example PromQL queries:

```promql
# Success rate per binding
sum(rate(aof_binding_success_total[5m])) by (name) /
sum(rate(aof_binding_executions_total[5m])) by (name)

# Average execution time
rate(aof_binding_execution_duration_seconds_sum[5m]) /
rate(aof_binding_execution_duration_seconds_count[5m])

# Trigger event rate
rate(aof_trigger_events_total[5m])
```

## Migration Path

### Current (CLI) → Future (Operator)

Current CLI deployments can migrate to Kubernetes:

```bash
# 1. Export existing configurations to YAML
aofctl get contexts -o yaml > contexts.yaml
aofctl get triggers -o yaml > triggers.yaml

# 2. Install operator
helm install aof-operator aof/aof-operator -n aof-system

# 3. Apply exported configurations
kubectl apply -f contexts.yaml
kubectl apply -f triggers.yaml

# 4. Verify migration
kubectl get contexts,triggers -A
```

## See Also

- [Context Resource](../resources/context.md) - Context specification
- [Trigger Resource](../resources/trigger.md) - Trigger specification
- [FlowBinding Resource](../resources/binding.md) - Binding specification
- [Architecture Overview](../concepts/architecture.md) - AOF architecture
- [Multi-Tenancy Guide](../guides/multi-tenancy.md) - Multi-tenant patterns
