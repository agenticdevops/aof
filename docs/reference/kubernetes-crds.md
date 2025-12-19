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

AOF defines **5 core resources** organized into two layers:

```
┌────────────────────────────────────────────────────────────┐
│                    AOF Resource Model                       │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │ Agent   │  │ Fleet   │  │ Flow    │  ← Execution       │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                             │
│  ┌─────────┐  ┌─────────┐                                 │
│  │ Context │  │ Trigger │              ← Configuration    │
│  └─────────┘  └─────────┘                                 │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Execution Resources

| Resource | Description | Kubernetes Analog |
|----------|-------------|-------------------|
| **Agent** | Single AI assistant | Pod |
| **Fleet** | Team of coordinated agents | Deployment |
| **Flow** | Multi-step workflow | Argo Workflow |

### Configuration Resources

| Resource | Description | Kubernetes Analog |
|----------|-------------|-------------------|
| **Context** | Environment configuration | ConfigMap |
| **Trigger** | Platform + command routing | Ingress with routing rules |

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

Defines event sources with command routing to agents, fleets, or flows.

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
                  enum: [Slack, Telegram, Discord, HTTP, Schedule, PagerDuty, GitHub, WhatsApp, Jira, Manual]
                  description: "Platform type"
                config:
                  type: object
                  x-kubernetes-preserve-unknown-fields: true
                  description: "Platform-specific configuration"
                commands:
                  type: object
                  additionalProperties:
                    type: object
                    properties:
                      agent:
                        type: string
                        description: "Route to agent"
                      fleet:
                        type: string
                        description: "Route to fleet"
                      flow:
                        type: string
                        description: "Route to flow"
                      description:
                        type: string
                        description: "Help text"
                  description: "Command bindings (e.g., /kubectl → k8s-agent)"
                default_agent:
                  type: string
                  description: "Fallback agent for natural language"
                enabled:
                  type: boolean
                  default: true
                  description: "Enable/disable trigger"
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
                commandStats:
                  type: object
                  additionalProperties:
                    type: integer
                  description: "Execution count per command"
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
        - name: Default
          type: string
          jsonPath: .spec.default_agent
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

### 3. Agent CRD

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

### 4. AgentFleet CRD

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
                      enum: [hierarchical, peer, tiered, pipeline, swarm, deep]
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

### 5. AgentFlow CRD

Multi-step workflow with nodes and connections. Flows are pure workflow logic - they are invoked via Trigger command bindings.

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
                - nodes
                - connections
              properties:
                description:
                  type: string
                  description: "Human-readable description"
                context:
                  type: object
                  properties:
                    ref:
                      type: string
                      description: "Reference to Context resource"
                    namespace:
                      type: string
                    env:
                      type: object
                      additionalProperties:
                        type: string
                  x-kubernetes-preserve-unknown-fields: true
                  description: "Execution context"
                nodes:
                  type: array
                  items:
                    type: object
                    required:
                      - id
                      - type
                    properties:
                      id:
                        type: string
                        description: "Unique node identifier"
                      type:
                        type: string
                        enum: [Agent, Fleet, HumanApproval, Conditional, Response, End]
                        description: "Node type"
                      config:
                        type: object
                        x-kubernetes-preserve-unknown-fields: true
                        description: "Type-specific configuration"
                  description: "Workflow nodes"
                connections:
                  type: array
                  items:
                    type: object
                    required:
                      - from
                      - to
                    properties:
                      from:
                        type: string
                        description: "Source node ID (or 'start')"
                      to:
                        type: string
                        description: "Target node ID"
                      condition:
                        type: string
                        description: "Conditional expression"
                  description: "Node connections (edges)"
                config:
                  type: object
                  properties:
                    default_timeout_seconds:
                      type: integer
                      description: "Default node timeout"
                    verbose:
                      type: boolean
                    retry:
                      type: object
                      properties:
                        max_attempts:
                          type: integer
                        initial_delay:
                          type: string
                        backoff_multiplier:
                          type: number
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
        - name: Ready
          type: boolean
          jsonPath: .status.ready
        - name: Nodes
          type: integer
          jsonPath: .spec.nodes[*]
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

# Trigger includes command routing
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-oncall
  namespace: aof-system
spec:
  type: Slack
  config:
    bot_token: \${SLACK_BOT_TOKEN}
    app_token: \${SLACK_APP_TOKEN}
    channels:
      - C01234567
  commands:
    /kubectl:
      agent: k8s-ops
    /diagnose:
      fleet: rca-fleet
    /deploy:
      flow: workflows/deploy-flow
  default_agent: devops
EOF
```

### Monitor Resources

```bash
# Watch all AOF resources
kubectl get contexts,triggers,agentflows -A --watch

# Check trigger status
kubectl describe trigger slack-oncall -n aof-system

# View trigger events
kubectl get trigger slack-oncall -n aof-system \
  -o jsonpath='{.status.eventCount}'

# Check context readiness
kubectl get context prod -n aof-system \
  -o jsonpath='{.status.ready}'

# View command execution stats
kubectl get trigger slack-oncall -n aof-system \
  -o jsonpath='{.status.commandStats}'
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
  commands:
    /kubectl:
      agent: k8s-ops
    /diagnose:
      fleet: rca-fleet
  default_agent: devops
EOF

# Team B gets isolated resources with their own Trigger
kubectl apply -f team-b-resources.yaml -n team-b
```

### Shared Flows with Team-Specific Triggers

```bash
# Shared flow (cluster-wide)
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
  namespace: shared-flows
spec:
  description: "Shared deployment workflow"
  nodes:
    - id: validate
      type: Agent
      config:
        agent: validator
    - id: deploy
      type: Agent
      config:
        agent: k8s-agent
  connections:
    - from: start
      to: validate
    - from: validate
      to: deploy
EOF

# Team A trigger references shared flow
kubectl apply -f - <<EOF
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: team-a-slack
  namespace: team-a
spec:
  type: Slack
  config:
    channels: [C_TEAM_A]
  commands:
    /deploy:
      flow: shared-flows/deploy-flow  # Cross-namespace reference
  default_agent: devops
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

# Command execution metrics
aof_command_executions_total{trigger="slack-oncall", command="/kubectl"} 287
aof_command_success_total{trigger="slack-oncall", command="/kubectl"} 281
aof_command_failure_total{trigger="slack-oncall", command="/kubectl"} 6
aof_command_execution_duration_seconds{trigger="slack-oncall", quantile="0.95"} 4.8
```

### Grafana Dashboard

Example PromQL queries:

```promql
# Success rate per command
sum(rate(aof_command_success_total[5m])) by (trigger, command) /
sum(rate(aof_command_executions_total[5m])) by (trigger, command)

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

- [Context Resource](./context-spec.md) - Context specification
- [Trigger Resource](./trigger-spec.md) - Trigger specification with command bindings
- [AgentFlow Resource](./agentflow-spec.md) - Multi-step workflow specification
- [Fleet Resource](./fleet-spec.md) - Fleet specification
- [Resource Selection Guide](../concepts/resource-selection.md) - When to use what
