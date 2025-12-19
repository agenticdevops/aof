# Context Injection Architecture

This document provides a deep dive into AOF's context injection system, which enables the same agent to operate safely across different environments, clusters, and security boundaries.

## What is a Context?

A **Context** is an environment boundary that defines the runtime execution environment for agents. Think of it as a namespace or deployment target in Kubernetes - it isolates resources and applies specific policies.

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
spec:
  # WHERE the agent connects (cluster, namespace)
  kubeconfig: /etc/kubernetes/prod-kubeconfig
  namespace: default
  cluster: prod-us-east-1

  # WHAT environment the agent sees
  env:
    ENVIRONMENT: production
    LOG_LEVEL: info
    REQUIRE_APPROVAL: "true"

  # WHO can approve destructive operations
  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD
      - U016PLATFORM_ADMIN

  # HOW MUCH the agent can do (rate limits)
  limits:
    max_requests_per_hour: 100
    max_tokens_per_request: 4096

  # AUDIT trail for compliance
  audit:
    enabled: true
    log_path: /var/log/aof/prod-audit.log
```

### Context vs Agent

| Aspect | Agent | Context |
|--------|-------|---------|
| **Definition** | What the agent can do (capabilities) | Where and how the agent operates (boundaries) |
| **Contents** | Model, instructions, tools | Kubeconfig, env vars, limits, approval rules |
| **Reusability** | Reused across environments | Specific to one environment |
| **Version Control** | Checked into main repo | May be in separate secure repo |
| **Changes** | Require review (change behavior) | Operational changes (change boundaries) |

---

## Why Context-Agnostic Resources?

### The Problem Without Contexts

Traditional approach: Agent definitions include environment details:

```yaml
# ❌ prod-k8s-agent.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: prod-k8s-agent
spec:
  model: anthropic:claude-sonnet-4
  instructions: |
    You manage the PRODUCTION Kubernetes cluster.
    Cluster: prod-us-east-1.example.com
    IMPORTANT: Always require approval for destructive commands.
  env:
    KUBECONFIG: /etc/kubernetes/prod-kubeconfig
    ENVIRONMENT: production
    REQUIRE_APPROVAL: "true"

# ❌ staging-k8s-agent.yaml (DUPLICATE with slight changes)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: staging-k8s-agent
spec:
  model: anthropic:claude-sonnet-4
  instructions: |
    You manage the STAGING Kubernetes cluster.
    Cluster: staging-us-east-1.example.com
    No approval required for destructive commands.
  env:
    KUBECONFIG: /etc/kubernetes/staging-kubeconfig
    ENVIRONMENT: staging
    REQUIRE_APPROVAL: "false"
```

**Problems:**
- ❌ Duplicate agent definitions (90% identical)
- ❌ Environment details leak into agent instructions
- ❌ Hard to maintain consistency across environments
- ❌ Changes require updating multiple files
- ❌ No separation of concerns (logic vs deployment)

### The Solution: Context-Agnostic Agents

With contexts, the agent is **pure logic** with **no environment details**:

```yaml
# ✅ agents/k8s-ops.yaml (SINGLE definition, works everywhere)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: |
    You are a Kubernetes operations assistant.
    Help users manage cluster resources using kubectl.
    Follow approval workflows when required.
  tools:
    - kubectl
    - shell

# ✅ contexts/prod-context.yaml (PRODUCTION boundaries)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
spec:
  kubeconfig: /etc/kubernetes/prod-kubeconfig
  namespace: default
  cluster: prod-us-east-1
  env:
    ENVIRONMENT: production
    REQUIRE_APPROVAL: "true"
  approval:
    required: true
    allowed_users: [U015SRE_LEAD]

# ✅ contexts/staging-context.yaml (STAGING boundaries)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging-context
spec:
  kubeconfig: /etc/kubernetes/staging-kubeconfig
  namespace: staging
  cluster: staging-us-east-1
  env:
    ENVIRONMENT: staging
    REQUIRE_APPROVAL: "false"
  approval:
    required: false
```

**Benefits:**
- ✅ One agent definition for all environments
- ✅ Clear separation: agent logic vs deployment context
- ✅ Easy to maintain and update
- ✅ Context changes don't require agent changes
- ✅ Security policies isolated in contexts

---

## How Context is Injected

Context can be injected at **two points in the execution lifecycle**:

```
┌──────────────────────────────────────────────────────────────┐
│                   INJECTION POINT 1                          │
│                   Via AgentFlow context field                │
│                                                              │
│  Slack message arrives                                       │
│        ↓                                                     │
│  Trigger routes to AgentFlow                                 │
│        ↓                                                     │
│  AgentFlow specifies context ref                             │
│        ↓                                                     │
│  Context variables injected                                  │
│        ↓                                                     │
│  Agent executes with context                                 │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                   INJECTION POINT 2                          │
│                   Via CLI --context flag                     │
│                                                              │
│  $ aofctl run agent k8s-ops --context prod-context          │
│        ↓                                                     │
│  Load Context from file                                      │
│        ↓                                                     │
│  Context variables injected                                  │
│        ↓                                                     │
│  Agent executes with context                                 │
└──────────────────────────────────────────────────────────────┘
```

### Injection Point 1: Via AgentFlow Context (Trigger-Based)

When using triggers (Slack, Telegram, webhooks), context is specified in the **AgentFlow**:

```yaml
# agents/k8s-ops.yaml (context-agnostic)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: google:gemini-2.5-flash
  instructions: "You are a Kubernetes ops assistant."
  tools: [kubectl]

---
# contexts/prod-context.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: default
  env:
    ENVIRONMENT: production

---
# flows/slack-k8s-flow.yaml (orchestration with context reference)
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-flow
spec:
  context:
    ref: prod-context    # Context injection happens here
  nodes:
    - id: agent-process
      type: Agent
      config:
        agent: k8s-ops
        input: ${MESSAGE_TEXT}
  connections:
    - from: start
      to: agent-process

---
# triggers/slack-prod.yaml (routes commands to flows)
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    channels: [production]
  commands:
    /kubectl:
      flow: slack-k8s-flow
  default_agent: devops
```

**Execution Flow:**

```
1. Slack message arrives in #production channel
   ↓
2. Trigger routes /kubectl command to slack-k8s-flow
   ↓
3. AgentFlow loads:
   - Context: prod-context (environment boundaries) ← INJECTED HERE
   - Nodes: Agent workflow
   ↓
4. Flow executes with prod-context:
   - Agent sees env var: ENVIRONMENT=production
   - Agent uses kubeconfig: ${KUBECONFIG_PROD}
   - kubectl commands target production cluster
   ↓
5. Response sent back to Slack
```

### Injection Point 2: Via CLI `--context` Flag (Direct Execution)

When running agents directly via CLI, context is specified with `--context`:

```bash
# Run with production context
$ aofctl run agent k8s-ops \
  --context prod-context \
  --input "List all pods in default namespace"

# Execution:
# 1. Load agent: k8s-ops
# 2. Load context: prod-context ← INJECTED HERE
# 3. Inject context variables into agent execution
# 4. Agent executes with production kubeconfig
```

```bash
# Same agent, different context (staging)
$ aofctl run agent k8s-ops \
  --context staging-context \
  --input "List all pods in default namespace"

# Execution:
# 1. Load agent: k8s-ops (SAME agent)
# 2. Load context: staging-context ← DIFFERENT context
# 3. Inject staging kubeconfig
# 4. Agent executes against STAGING cluster
```

```bash
# No context (uses defaults)
$ aofctl run agent k8s-ops \
  --input "List all pods"

# Execution:
# 1. Load agent: k8s-ops
# 2. No context specified
# 3. Uses default kubeconfig from environment (~/.kube/config)
# 4. Uses current context from kubeconfig
```

---

## Context Spec Reference

Full specification of the Context resource:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: string                     # Required: Context name
  namespace: string                # Optional: Namespace for isolation
  labels:                          # Optional: Labels for filtering
    environment: production
    team: platform
    region: us-east-1

spec:
  # ===================================================================
  # CLUSTER CONNECTION (where the agent operates)
  # ===================================================================
  kubeconfig: string               # Path to kubeconfig file
                                   # Supports env vars: ${KUBECONFIG_PROD}
                                   # Default: ~/.kube/config

  namespace: string                # Default Kubernetes namespace
                                   # Default: "default"

  cluster: string                  # Cluster name (for audit logging)
                                   # Optional, but recommended for multi-cluster

  # ===================================================================
  # ENVIRONMENT VARIABLES (injected into agent execution)
  # ===================================================================
  env:                             # map[string]string
    ENVIRONMENT: production        # Environment name
    LOG_LEVEL: info                # Logging verbosity
    REQUIRE_APPROVAL: "true"       # Approval flag
    REGION: us-east-1              # Cloud region
    # Any env vars needed by tools or agent logic

  # ===================================================================
  # WORKING DIRECTORY
  # ===================================================================
  working_dir: string              # Default working directory for commands
                                   # Default: current directory

  # ===================================================================
  # APPROVAL WORKFLOW (who can approve destructive operations)
  # ===================================================================
  approval:
    required: boolean              # Whether approval is required
                                   # Default: false

    allowed_users: []              # User IDs that can approve (Slack/platform IDs)
                                   # Example: ["U015SRE_LEAD", "U016ADMIN"]

    require_for: []                # Commands/patterns requiring approval
                                   # Example:
                                   #   - kubectl delete
                                   #   - kubectl scale --replicas=0
                                   #   - helm uninstall

    timeout_seconds: int           # Approval timeout
                                   # Default: 300 (5 minutes)

    auto_approve_readonly: boolean # Auto-approve read-only commands
                                   # Default: true

  # ===================================================================
  # AUDIT LOGGING (compliance and security)
  # ===================================================================
  audit:
    enabled: boolean               # Enable audit logging
                                   # Default: false

    log_path: string               # Path to audit log file
                                   # Default: /var/log/aof/audit.log

    log_commands: boolean          # Log executed commands
                                   # Default: true

    log_responses: boolean         # Log agent responses (may contain sensitive data)
                                   # Default: false

    log_format: string             # Log format: json, text
                                   # Default: json

  # ===================================================================
  # RATE LIMITS (prevent abuse and control costs)
  # ===================================================================
  limits:
    max_requests_per_hour: int     # Max requests per hour
                                   # Default: unlimited

    max_tokens_per_request: int    # Max tokens per LLM request
                                   # Default: model's context window

    max_concurrent_requests: int   # Max concurrent requests
                                   # Default: 5

    max_execution_time_secs: int   # Max execution time per request
                                   # Default: 300 (5 minutes)

  # ===================================================================
  # SECURITY (command filtering and restrictions)
  # ===================================================================
  security:
    allowed_namespaces: []         # Restrict kubectl to these namespaces
                                   # Empty = all namespaces allowed
                                   # Example: ["default", "production"]

    allowed_commands: []           # Whitelist commands (empty = all allowed)
                                   # Example:
                                   #   - kubectl get
                                   #   - kubectl describe
                                   #   - kubectl logs

    blocked_commands: []           # Blacklist commands
                                   # Example:
                                   #   - kubectl delete
                                   #   - rm -rf

    readonly_mode: boolean         # Force read-only operations
                                   # Default: false

  # ===================================================================
  # METADATA (additional context information)
  # ===================================================================
  metadata:
    owner: string                  # Team/person responsible
    contact: string                # Contact email/Slack
    description: string            # Context description
```

---

## Examples: Same Agent, Different Contexts

### Example 1: Production vs Staging

```yaml
# agents/k8s-ops.yaml (ONE agent definition)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant."
  tools: [kubectl, shell]

---
# contexts/prod-context.yaml (STRICT production boundaries)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-context
  labels:
    environment: production
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: default
  cluster: prod-us-east-1

  env:
    ENVIRONMENT: production
    LOG_LEVEL: info
    REQUIRE_APPROVAL: "true"

  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD       # Only SRE lead can approve
    require_for:
      - kubectl delete
      - kubectl scale --replicas=0
      - kubectl apply
    timeout_seconds: 300

  limits:
    max_requests_per_hour: 100
    max_tokens_per_request: 4096

  audit:
    enabled: true
    log_path: /var/log/aof/prod-audit.log
    log_commands: true
    log_responses: false

  security:
    readonly_mode: false
    blocked_commands:
      - kubectl delete namespace
      - kubectl delete pv

---
# contexts/staging-context.yaml (RELAXED staging boundaries)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging-context
  labels:
    environment: staging
spec:
  kubeconfig: ${KUBECONFIG_STAGING}
  namespace: staging
  cluster: staging-us-east-1

  env:
    ENVIRONMENT: staging
    LOG_LEVEL: debug
    REQUIRE_APPROVAL: "false"
    ALLOW_DELETE: "true"

  approval:
    required: false             # No approval needed in staging

  limits:
    max_requests_per_hour: 500  # Higher limit for testing
    max_tokens_per_request: 8192

  audit:
    enabled: true
    log_path: /var/log/aof/staging-audit.log

  security:
    readonly_mode: false
    # No command restrictions in staging

---
# CLI Usage
$ aofctl run agent k8s-ops --context prod-context
# → Targets production cluster
# → Requires approval for destructive commands
# → Strict rate limits

$ aofctl run agent k8s-ops --context staging-context
# → Targets staging cluster (SAME agent, DIFFERENT cluster!)
# → No approval required
# → Higher rate limits for testing
```

### Example 2: Multi-Region Production

```yaml
# agents/k8s-ops.yaml (ONE agent for all regions)
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: "You are a Kubernetes ops assistant."
  tools: [kubectl]

---
# contexts/prod-us-east.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-us-east
  labels:
    environment: production
    region: us-east-1
spec:
  kubeconfig: ${KUBECONFIG_PROD_US_EAST}
  cluster: prod-us-east-1
  env:
    REGION: us-east-1
    CLUSTER_NAME: prod-us-east-1

---
# contexts/prod-eu-west.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod-eu-west
  labels:
    environment: production
    region: eu-west-1
spec:
  kubeconfig: ${KUBECONFIG_PROD_EU_WEST}
  cluster: prod-eu-west-1
  env:
    REGION: eu-west-1
    CLUSTER_NAME: prod-eu-west-1

---
# flows/us-east-flow.yaml (references US East context)
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: us-east-k8s-flow
spec:
  context:
    ref: prod-us-east  # ← US region context
  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
  connections:
    - from: start
      to: agent

---
# flows/eu-west-flow.yaml (references EU West context)
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: eu-west-k8s-flow
spec:
  context:
    ref: prod-eu-west  # ← EU region context (SAME agent, DIFFERENT region!)
  nodes:
    - id: agent
      type: Agent
      config:
        agent: k8s-ops
  connections:
    - from: start
      to: agent

---
# triggers/slack-regional.yaml (routes channels to region-specific flows)
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-regional
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
  commands:
    /us-east:
      flow: us-east-k8s-flow
      description: "Manage US East cluster"
    /eu-west:
      flow: eu-west-k8s-flow
      description: "Manage EU West cluster"
  default_agent: devops
```

**Result**: One agent manages multiple production regions with different contexts!

### Example 3: Tiered Access (Readonly vs Full Access)

```yaml
# contexts/readonly-context.yaml (for junior engineers)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: readonly-context
  labels:
    access_level: readonly
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  cluster: prod-us-east-1

  env:
    ACCESS_LEVEL: readonly

  security:
    readonly_mode: true
    allowed_commands:
      - kubectl get
      - kubectl describe
      - kubectl logs
    blocked_commands:
      - kubectl delete
      - kubectl apply
      - kubectl scale

---
# contexts/admin-context.yaml (for senior SREs)
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: admin-context
  labels:
    access_level: admin
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  cluster: prod-us-east-1

  env:
    ACCESS_LEVEL: admin

  approval:
    required: true
    allowed_users:
      - U015SRE_LEAD
    require_for:
      - kubectl delete
      - kubectl scale

  security:
    readonly_mode: false
    # Full access with approval

---
# CLI Usage
$ aofctl run agent k8s-ops --context readonly-context
# Junior engineer: Read-only access, no destructive commands

$ aofctl run agent k8s-ops --context admin-context
# Senior SRE: Full access with approval workflow
```

---

## Security Implications

### 1. Separation of Concerns

Context injection enforces **clear security boundaries**:

```
┌─────────────────────────────────────────────────────────────┐
│ AGENT DEFINITION (business logic, version controlled)      │
│ ├─ Model configuration (which LLM)                         │
│ ├─ Instructions (what the agent does)                      │
│ └─ Tools (capabilities)                                    │
│                                                             │
│ ✅ Can be safely reviewed and changed by engineers         │
│ ✅ No secrets or environment details                       │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ CONTEXT DEFINITION (security boundaries)                   │
│ ├─ Kubeconfig secrets (cluster access)                     │
│ ├─ Approval rules (who can approve)                        │
│ ├─ Rate limits (prevent abuse)                             │
│ ├─ Audit logging (compliance)                              │
│ └─ Command restrictions (security policies)                │
│                                                             │
│ ✅ Managed by security/operations team                     │
│ ✅ Contains sensitive configuration                        │
│ ✅ May be in separate secure repo                          │
└─────────────────────────────────────────────────────────────┘
```

### 2. Principle of Least Privilege

Contexts enable **fine-grained access control**:

```yaml
# Least privilege for different roles
contexts:
  - name: developer-readonly
    security:
      readonly_mode: true
      allowed_commands: [kubectl get, kubectl describe, kubectl logs]

  - name: sre-operator
    security:
      readonly_mode: false
      allowed_commands: [kubectl get, kubectl describe, kubectl logs, kubectl apply]
      blocked_commands: [kubectl delete namespace]
    approval:
      required: true

  - name: platform-admin
    security:
      readonly_mode: false
      # Full access
    approval:
      required: true
      allowed_users: [U015PLATFORM_ADMIN]
```

### 3. Audit Trail

All context-injected operations are logged for **compliance**:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "context": "prod-context",
  "cluster": "prod-us-east-1",
  "namespace": "default",
  "agent": "k8s-ops",
  "user": "U015VBH1GTZ",
  "command": "kubectl delete pod nginx-abc123",
  "approval_required": true,
  "approved_by": "U015SRE_LEAD",
  "approval_timestamp": "2024-01-15T10:29:45Z",
  "execution_time_ms": 1250,
  "status": "success",
  "tokens_used": 512
}
```

### 4. Secrets Management

Contexts reference secrets via **environment variables**:

```yaml
# contexts/prod-context.yaml
spec:
  kubeconfig: ${KUBECONFIG_PROD}  # ← Loaded from environment
  env:
    AWS_ACCESS_KEY_ID: ${AWS_PROD_ACCESS_KEY}
    AWS_SECRET_ACCESS_KEY: ${AWS_PROD_SECRET_KEY}

# Secrets injected at runtime (never in YAML files)
$ export KUBECONFIG_PROD=/secure/path/prod-kubeconfig
$ export AWS_PROD_ACCESS_KEY=AKIA...
$ export AWS_PROD_SECRET_KEY=secret...
$ aofctl serve --flows-dir ./flows
```

---

## Best Practices

### 1. Keep Agents Context-Agnostic

```yaml
# ✅ Good: No environment-specific details
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  model: anthropic:claude-sonnet-4
  instructions: |
    You are a Kubernetes operations assistant.
    Help users manage cluster resources using kubectl.
  tools: [kubectl]

# ❌ Bad: Environment details in agent
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
spec:
  instructions: |
    You manage the PRODUCTION cluster at prod.example.com.
    IMPORTANT: Always require approval!
  env:
    KUBECONFIG: /path/to/prod-kubeconfig  # DON'T DO THIS
```

### 2. Use Contexts for All Environment Differences

```yaml
# Everything environment-specific goes in Context:
# - Kubeconfig path
# - Namespace
# - Approval rules
# - Rate limits
# - Audit settings
# - Security restrictions
```

### 3. Version Control Strategy

- **Agents/Flows**: Check into main repo (no secrets)
- **Contexts**: Separate secure repo or vault (contains secrets)
- **Bindings**: Main repo (references only, no secrets)

### 4. Naming Conventions

```yaml
# Contexts: {environment}-context or {environment}-{region}-context
prod-context
staging-context
prod-us-east-context
prod-eu-west-context

# Bindings: {platform}-{environment}-{purpose}
slack-prod-k8s-bot
whatsapp-staging-oncall
telegram-prod-incident-bot
```

---

## Summary

| Aspect | Without Contexts | With Contexts |
|--------|------------------|---------------|
| **Agent Definitions** | Duplicate per environment | Define once, reuse everywhere |
| **Environment Config** | Mixed with agent logic | Isolated in Context resources |
| **Secrets** | Risk of exposure in agent YAMLs | Injected at runtime via env vars |
| **Security** | Hard to enforce policies | Clear security boundaries |
| **Audit Trail** | Inconsistent | Comprehensive, context-aware |
| **Maintainability** | Update multiple files | Update one context |
| **Scale** | Doesn't scale beyond 5-10 envs | Scales to 100+ environments |

**Key Takeaway**: Context injection enables the same agent to operate safely across different environments, clusters, and security boundaries by separating business logic from deployment context.

---

## Next Steps

- [Composable Design Architecture](composable-design.md)
- [Multi-Tenant AgentFlow Architecture](multi-tenant-agentflows.md)
- [AgentFlow Spec Reference](../reference/agentflow-spec.md)
- [Context Spec Reference](../reference/context-spec.md)
