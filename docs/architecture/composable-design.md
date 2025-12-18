# Composable Multi-Tenant Architecture

This document describes AOF's composable architecture that enables enterprise-scale deployments through resource reusability and context injection.

## Overview

AOF follows a **composable, DRY (Don't Repeat Yourself) architecture** inspired by Kubernetes' resource composition model. Just as Kubernetes composes Pods into Deployments and Services, AOF composes Agents into Fleets, Flows, and Triggers through reference-based composition.

### Why Composable?

In traditional agent frameworks, you duplicate agent definitions across deployments:

```yaml
# ❌ Traditional: Duplication everywhere
# prod-bot.yaml - Full agent definition
apiVersion: aof.dev/v1
kind: Agent
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant..."
  tools: [kubectl, shell]

# staging-bot.yaml - DUPLICATE agent definition
apiVersion: aof.dev/v1
kind: Agent
spec:
  model: anthropic:claude-sonnet-4  # Same agent
  instructions: "You are a Kubernetes ops assistant..."  # Same instructions
  tools: [kubectl, shell]  # Same tools
```

**Problems:**
- Duplicate definitions across environments
- Hard to maintain consistency
- Changes require updating multiple files
- No separation of concerns (agent logic vs deployment context)

### The Composable Solution

AOF separates **reusable resources** from **deployment context**:

```yaml
# ✅ Composable: Define once, reference everywhere

# 1. Define agent ONCE (context-agnostic)
# agents/k8s-ops.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant..."
  tools: [kubectl, shell]

# 2. Reference in production flow (injects prod context)
# flows/prod/slack-k8s-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-k8s
spec:
  trigger:
    type: Slack
    config:
      channels: [production]
  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: default
  agents:
    - name: k8s-ops  # ← Reference (no duplication!)

# 3. Reference in staging flow (injects staging context)
# flows/staging/slack-k8s-flow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-staging-k8s
spec:
  trigger:
    type: Slack
    config:
      channels: [staging]
  context:
    kubeconfig: ${KUBECONFIG_STAGING}
    namespace: staging
  agents:
    - name: k8s-ops  # ← Same agent, different context!
```

**Benefits:**
- ✅ Agent defined once, used everywhere
- ✅ Context injected at runtime
- ✅ Easy to maintain and update
- ✅ Clear separation of concerns
- ✅ Enterprise-scale deployments

---

## Architecture Layers

AOF's composable architecture is organized into **six conceptual layers**, from atomic resources to runtime bindings:

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 5: BINDINGS (Runtime Composition)                        │
│ FlowBinding ties everything together at runtime                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ FlowBinding: prod-slack-bot                                 │ │
│ │ ├─ Flow: slack-k8s-flow (orchestration logic)              │ │
│ │ ├─ Context: prod-context (env boundary)                    │ │
│ │ └─ Trigger: slack-bot-trigger (message source)             │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: TRIGGERS (Message Sources)                            │
│ Event sources that initiate flows                               │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│ │  Slack   │  │ WhatsApp │  │ Telegram │  │  HTTP    │        │
│ │ Trigger  │  │ Trigger  │  │ Trigger  │  │ Webhook  │        │
│ └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: FLOWS (Orchestration Logic)                           │
│ Event-driven workflows that coordinate agents                   │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ AgentFlow: slack-k8s-flow                                   │ │
│ │ ┌──────────────┐                                            │ │
│ │ │ Node: parse  │ → Agent: k8s-ops → Node: send-response    │ │
│ │ └──────────────┘                                            │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: FLEETS (Agent Composition)                            │
│ Compose multiple agents for collaboration                       │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ AgentFleet: code-review-team                                │ │
│ │ ├─ Agent: security-reviewer                                 │ │
│ │ ├─ Agent: performance-reviewer                              │ │
│ │ └─ Agent: quality-reviewer                                  │ │
│ │ Coordination: peer + consensus (majority)                   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: AGENTS (Atomic Units)                                 │
│ Individual AI agents (context-agnostic)                         │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│ │  k8s-ops │  │incident  │  │ security │  │terraform │        │
│ │  agent   │  │responder │  │ scanner  │  │ planner  │        │
│ └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 0: CONTEXTS (Environment Boundaries)                      │
│ Runtime environment definitions (injected at execution)         │
│ ┌────────────┐  ┌────────────┐  ┌────────────┐                │
│ │    prod    │  │  staging   │  │    dev     │                │
│ │  context   │  │  context   │  │  context   │                │
│ └────────────┘  └────────────┘  └────────────┘                │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Descriptions

| Layer | Resource Type | Purpose | Example |
|-------|--------------|---------|---------|
| **Layer 0** | Context | Environment boundaries (kubeconfig, namespace, env) | `prod-context`, `staging-context` |
| **Layer 1** | Agent | Atomic AI agents (context-agnostic) | `k8s-ops`, `incident-responder` |
| **Layer 2** | Fleet | Multi-agent collaboration | `code-review-team`, `rca-fleet` |
| **Layer 3** | Flow | Event-driven orchestration | `slack-k8s-flow`, `deploy-approval-flow` |
| **Layer 4** | Trigger | Message sources | `slack-trigger`, `webhook-trigger` |
| **Layer 5** | Binding | Runtime composition (ties layers 0-4 together) | `prod-slack-bot-binding` |

---

## How Composition Works

### The `ref:` Syntax

AOF uses the `ref:` syntax to reference resources without duplication:

```yaml
# Reference an agent
agents:
  - name: k8s-ops  # Simple reference (looks up Agent in registry)

# Reference in fleet
spec:
  agents:
    - name: security-reviewer
      spec:
        model: google:gemini-2.5-flash
        instructions: "Focus on security..."
    - name: k8s-ops  # Reference existing agent

# Reference fleet in workflow
steps:
  - name: code-review
    agent:
      agentRef:
        name: code-review-team
        kind: AgentFleet  # Explicit reference to Fleet
```

### Reference Resolution

When AOF loads resources, it resolves references in this order:

```
1. Load all resources from directory tree
   ├─ agents/*.yaml
   ├─ fleets/*.yaml
   ├─ flows/*.yaml
   └─ contexts/*.yaml

2. Build resource registry
   ├─ Register Contexts (Layer 0)
   ├─ Register Agents (Layer 1)
   ├─ Register Fleets (Layer 2)
   └─ Register Flows (Layer 3)

3. Resolve references
   ├─ Fleet references to Agents
   ├─ Flow references to Agents/Fleets
   └─ Binding references to Flow/Context/Trigger

4. Inject context at runtime
   └─ Context variables → Agent execution environment
```

---

## Directory Structure for Enterprise

Organize resources by layer and environment:

```
aof-deployment/
├── contexts/                  # Layer 0: Environment boundaries
│   ├── prod-context.yaml      # Production: kubeconfig, namespace, limits
│   ├── staging-context.yaml   # Staging: relaxed limits
│   └── dev-context.yaml       # Development: local setup
│
├── agents/                    # Layer 1: Atomic agents (context-agnostic)
│   ├── k8s-ops.yaml           # Kubernetes operations
│   ├── incident-responder.yaml
│   ├── security-scanner.yaml
│   └── terraform-planner.yaml
│
├── fleets/                    # Layer 2: Multi-agent teams
│   ├── code-review-team.yaml  # 3 specialists: security, perf, quality
│   ├── rca-team.yaml          # Tiered RCA: collectors → reasoners → coordinator
│   └── incident-response-team.yaml
│
├── flows/                     # Layer 3: Orchestration workflows
│   ├── slack-k8s-flow.yaml    # Slack → k8s-ops → response
│   ├── deploy-approval-flow.yaml
│   └── incident-auto-remediation-flow.yaml
│
├── triggers/                  # Layer 4: Event sources
│   ├── slack-bot.yaml         # Slack trigger config
│   ├── whatsapp-oncall.yaml   # WhatsApp trigger
│   └── pagerduty-webhook.yaml
│
└── bindings/                  # Layer 5: Runtime composition
    ├── prod/
    │   ├── slack-prod-bot.yaml      # Flow + prod-context + slack-trigger
    │   └── whatsapp-oncall-prod.yaml
    ├── staging/
    │   └── slack-staging-bot.yaml   # Same flow + staging-context
    └── dev/
        └── slack-dev-bot.yaml       # Same flow + dev-context
```

### Multi-Environment Deployment

Same resources, different contexts:

```yaml
# agents/k8s-ops.yaml - SHARED ACROSS ALL ENVIRONMENTS
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: |
    You are a Kubernetes operations assistant.
    Use kubectl to inspect and manage cluster resources.
  tools:
    - kubectl
    - shell

---
# contexts/prod-context.yaml - PRODUCTION CONTEXT
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: default
  cluster: prod-us-east-1
  env:
    ENVIRONMENT: production
    REQUIRE_APPROVAL: "true"
    KUBECTL_READONLY: "false"
  limits:
    max_tokens_per_request: 4096
    max_requests_per_hour: 100
  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD
      - U016PLATFORM_ADMIN

---
# contexts/staging-context.yaml - STAGING CONTEXT
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging-context
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  namespace: staging
  cluster: staging-us-east-1
  env:
    ENVIRONMENT: staging
    REQUIRE_APPROVAL: "false"
    KUBECTL_READONLY: "false"
    ALLOW_DELETE: "true"
  limits:
    max_tokens_per_request: 8192
    max_requests_per_hour: 500

---
# bindings/prod/slack-k8s-bot.yaml - PRODUCTION BINDING
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-prod-k8s-bot
spec:
  flow:
    name: slack-k8s-flow  # References flow
  context:
    name: prod-context    # Injects prod context
  trigger:
    name: slack-bot       # References trigger

---
# bindings/staging/slack-k8s-bot.yaml - STAGING BINDING
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-staging-k8s-bot
spec:
  flow:
    name: slack-k8s-flow  # SAME flow
  context:
    name: staging-context # DIFFERENT context
  trigger:
    name: slack-bot       # SAME trigger
```

**Result**: One agent definition, deployed to multiple environments with different contexts!

---

## Comparison with Kubernetes

AOF's composition model mirrors Kubernetes' declarative approach:

```
┌─────────────────────────────────────────────────────────────────┐
│                    KUBERNETES ANALOGY                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Container    →  Agent          (atomic unit)                  │
│     ↓                                                           │
│  Pod          →  Fleet          (co-located units)             │
│     ↓                                                           │
│  Deployment   →  Flow           (orchestration)                │
│     ↓                                                           │
│  Service      →  Trigger        (entry point)                  │
│     ↓                                                           │
│  Namespace    →  Context        (environment boundary)         │
│                                                                 │
│  kubectl      →  aofctl         (CLI for management)           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Kubernetes Example

```yaml
# Container → Pod → Deployment → Service
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: nginx
          image: nginx:1.21
---
apiVersion: v1
kind: Service
spec:
  selector:
    app: nginx
  ports:
    - port: 80
```

### AOF Equivalent

```yaml
# Agent → Fleet → Flow → Trigger
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
---
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-flow
spec:
  agents:
    - name: k8s-ops  # References agent (like Pod references Container)
  trigger:
    type: Slack      # Entry point (like Service)
```

---

## Context Injection Deep Dive

### What is a Context?

A Context defines the **runtime environment** for agent execution:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
spec:
  # Kubernetes cluster connection
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: default
  cluster: prod-us-east-1

  # Environment variables (injected into agent execution)
  env:
    ENVIRONMENT: production
    LOG_LEVEL: info
    REQUIRE_APPROVAL: "true"

  # Approval workflow
  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD
      - U016PLATFORM_ADMIN
    require_for:
      - kubectl delete
      - kubectl scale --replicas=0

  # Rate limits
  limits:
    max_requests_per_hour: 100
    max_tokens_per_request: 4096

  # Audit logging
  audit:
    enabled: true
    log_path: /var/log/aof/audit.log
    log_commands: true
    log_responses: false
```

### Context Injection Points

Context is injected at **two points**:

#### 1. Via FlowBinding (for triggers)

```yaml
# bindings/prod/slack-bot.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-prod-bot
spec:
  flow:
    name: slack-k8s-flow
  context:
    name: prod-context  # ← Context injected here
  trigger:
    name: slack-bot
```

When a Slack message arrives:
1. FlowRouter matches message to `slack-prod-bot` binding
2. FlowBinding loads `prod-context`
3. Context variables injected into agent execution environment
4. Agent executes with production kubeconfig, approval rules, etc.

#### 2. Via CLI `--context` flag (for direct execution)

```bash
# Run agent with production context
aofctl run agent k8s-ops --context prod-context

# Run agent with staging context (same agent, different environment!)
aofctl run agent k8s-ops --context staging-context

# Run without context (uses defaults)
aofctl run agent k8s-ops
```

---

## Context Spec Reference

Full Context specification:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: string                # Required: Context name
  namespace: string           # Optional: Namespace for isolation
  labels:                     # Optional: Labels for selection
    environment: production
    team: platform

spec:
  # Kubernetes cluster connection
  kubeconfig: string          # Path to kubeconfig (supports env vars)
  namespace: string           # Default K8s namespace
  cluster: string             # Cluster name (for auditing)

  # Environment variables
  env:                        # map[string]string
    KEY: value                # Injected into agent execution

  # Working directory
  working_dir: string         # Default: current directory

  # Approval workflow
  approval:
    required: boolean         # Require approval for destructive commands
    allowed_users: []         # User IDs that can approve
    require_for: []           # Patterns requiring approval
    timeout_seconds: int      # Approval timeout (default: 300)

  # Audit logging
  audit:
    enabled: boolean          # Enable audit logging
    log_path: string          # Audit log file path
    log_commands: boolean     # Log executed commands
    log_responses: boolean    # Log agent responses

  # Rate limits
  limits:
    max_requests_per_hour: int
    max_tokens_per_request: int
    max_concurrent_requests: int

  # Security
  security:
    allowed_namespaces: []    # Restrict kubectl to these namespaces
    allowed_commands: []      # Whitelist commands
    blocked_commands: []      # Blacklist commands
```

---

## Examples

### Example 1: Same Agent, Different Clusters

```yaml
# agents/k8s-ops.yaml (context-agnostic)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant."
  tools: [kubectl, shell]

---
# contexts/prod-us-east.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-us-east
spec:
  kubeconfig: ${KUBECONFIG_PROD_US_EAST}
  cluster: prod-us-east-1
  env:
    REGION: us-east-1

---
# contexts/prod-eu-west.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-eu-west
spec:
  kubeconfig: ${KUBECONFIG_PROD_EU_WEST}
  cluster: prod-eu-west-1
  env:
    REGION: eu-west-1

---
# bindings/prod/slack-us-bot.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-us-bot
spec:
  flow:
    name: slack-k8s-flow
  context:
    name: prod-us-east  # ← US cluster
  trigger:
    name: slack-bot
    config:
      channels: [prod-us-east]

---
# bindings/prod/slack-eu-bot.yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: slack-eu-bot
spec:
  flow:
    name: slack-k8s-flow
  context:
    name: prod-eu-west  # ← EU cluster (SAME agent, DIFFERENT cluster!)
  trigger:
    name: slack-bot
    config:
      channels: [prod-eu-west]
```

**Result**: One agent definition manages both US and EU clusters!

### Example 2: Same Agent, Different Approval Rules

```yaml
# contexts/prod-strict.yaml (requires approval)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-strict
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD
    require_for:
      - kubectl delete
      - kubectl scale
      - kubectl apply

---
# contexts/staging-permissive.yaml (no approval)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging-permissive
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  approval:
    required: false
  env:
    ALLOW_DELETE: "true"

---
# CLI usage
$ aofctl run agent k8s-ops --context prod-strict
# → Requires approval for destructive commands

$ aofctl run agent k8s-ops --context staging-permissive
# → No approval required (same agent, different rules!)
```

---

## Security Implications

### Separation of Concerns

Composable architecture enforces **clear security boundaries**:

```
┌─────────────────────────────────────────────────────────────┐
│ AGENT DEFINITION (checked into git, reviewed by team)      │
│ ├─ Model configuration (which LLM to use)                  │
│ ├─ Instructions (what the agent can do)                    │
│ └─ Tools (capabilities available)                          │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ CONTEXT DEFINITION (managed by security team)              │
│ ├─ Kubeconfig secrets (cluster access)                     │
│ ├─ Approval rules (who can approve)                        │
│ ├─ Rate limits (prevent abuse)                             │
│ └─ Audit logging (compliance)                              │
└─────────────────────────────────────────────────────────────┘
```

### Principle of Least Privilege

Contexts enable fine-grained access control:

```yaml
# contexts/readonly-context.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: readonly-context
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  security:
    allowed_commands:
      - kubectl get
      - kubectl describe
      - kubectl logs
    blocked_commands:
      - kubectl delete
      - kubectl apply
      - kubectl scale
  env:
    KUBECTL_READONLY: "true"
```

### Audit Trail

All context-injected operations are logged:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "context": "prod-context",
  "agent": "k8s-ops",
  "user": "U015VBH1GTZ",
  "command": "kubectl delete pod nginx-abc123",
  "approved_by": "U015SRE_LEAD",
  "cluster": "prod-us-east-1",
  "namespace": "default",
  "status": "success"
}
```

---

## Best Practices

### 1. Define Agents Context-Agnostic

```yaml
# ✅ Good: No environment-specific details
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant."
  tools: [kubectl]

# ❌ Bad: Environment details in agent
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You manage the PRODUCTION cluster at prod.example.com"
  # Now this agent is tied to production!
```

### 2. Use Contexts for Environment Differences

```yaml
# contexts/prod-context.yaml
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  approval:
    required: true
  limits:
    max_tokens_per_request: 4096

# contexts/staging-context.yaml
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  approval:
    required: false
  limits:
    max_tokens_per_request: 8192  # More generous for testing
```

### 3. Organize by Layer

```
deployment/
├── contexts/      # Layer 0
├── agents/        # Layer 1
├── fleets/        # Layer 2
├── flows/         # Layer 3
├── triggers/      # Layer 4
└── bindings/      # Layer 5
```

### 4. Version Control Strategy

- **Agents/Fleets/Flows**: Check into git, review changes
- **Contexts**: Separate repo (contains secrets), restrict access
- **Bindings**: Check into git (references only, no secrets)

---

## Summary

| Aspect | Traditional | Composable AOF |
|--------|------------|----------------|
| **Agent Definitions** | Duplicated per environment | Define once, reference everywhere |
| **Environment Config** | Mixed with agent logic | Separate Context resources |
| **Maintainability** | Update multiple files | Update one definition |
| **Reusability** | Low | High |
| **Scale** | Hard at 10+ environments | Easy at 100+ environments |
| **Security** | Mixed concerns | Clear separation |

**Key Takeaway**: AOF's composable architecture enables enterprise-scale deployments through separation of concerns, reference-based composition, and runtime context injection.

---

## Next Steps

- [Context Injection Deep Dive](context-injection.md)
- [Multi-Tenant AgentFlow Architecture](multi-tenant-agentflows.md)
- [AgentFlow Spec Reference](../reference/agentflow-spec.md)
- [Fleet Composition Guide](../concepts/fleets.md)
