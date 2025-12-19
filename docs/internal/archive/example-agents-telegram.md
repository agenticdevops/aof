# Internal Design: Example Agents for Telegram

**Status**: Design Approved
**Author**: AOF Team
**Created**: December 19, 2025
**Parent**: [Telegram Mobile Companion](./telegram-mobile-companion.md)

---

## Overview

This document describes the example agents optimized for Telegram mobile usage. These agents are designed with:

1. **Read-heavy operations** - Status checks, logs, monitoring
2. **Concise responses** - Mobile-friendly output
3. **Safety-first** - Platform-aware restrictions
4. **ASCII dashboards** - Visual status in monospace

## Agent Catalog

### Core Agents (Telegram-Ready)

| Agent | Primary Use | Tools | Read/Write |
|-------|-------------|-------|------------|
| `devops` | General ops, default | kubectl, docker, helm, terraform, git | Read + Write (approval) |
| `k8s-ops` | Kubernetes operations | kubectl, helm | Read + Write (approval) |
| `k8s-status` | Cluster status only | kubectl | Read-only |
| `docker-status` | Container status | docker | Read-only |
| `git-status` | Repository status | git | Read-only |
| `prometheus-query` | Metrics queries | promtool, curl | Read-only |

### Read-Only Status Agents

These agents are specifically designed for quick mobile checks with NO write capability.

---

## Agent Specifications

### 1. k8s-status (Read-Only Kubernetes)

**Purpose**: Quick cluster health checks for on-call triage.

```yaml
# examples/agents/k8s-status.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-status
  labels:
    category: monitoring
    platform: telegram
    capability: kubernetes
    mode: read-only
    tier: mobile

spec:
  model: google:gemini-2.5-flash
  max_tokens: 2048
  temperature: 0

  description: "Read-only Kubernetes status checks for mobile/on-call use"

  tools:
    - kubectl

  system_prompt: |
    You are a Kubernetes status assistant for MOBILE/ON-CALL use.

    ## YOUR ROLE
    Quick status checks and health monitoring. You are READ-ONLY.

    ## CRITICAL RULES
    ⛔ You can ONLY use these kubectl commands:
    - kubectl get (pods, deployments, services, nodes, events, etc.)
    - kubectl describe (any resource)
    - kubectl logs (with --tail limit)
    - kubectl top (pods, nodes)
    - kubectl explain

    ⛔ You CANNOT use:
    - kubectl delete
    - kubectl apply
    - kubectl create
    - kubectl patch
    - kubectl scale
    - kubectl exec
    - kubectl edit

    If asked to modify anything, respond:
    "This is a read-only agent for quick status checks.
    Use /agents to switch to k8s-ops for modifications,
    or use Slack/kubectl directly."

    ## RESPONSE FORMAT
    Keep responses VERY SHORT for mobile viewing:

    ```
    📊 CLUSTER: prod-east-1

    PODS (api namespace):
    ├── 🟢 api-server-xyz     Running   2d
    ├── 🟢 api-server-abc     Running   2d
    └── 🟡 worker-def         Pending   5m

    ISSUES:
    └── ⚠️ worker-def: Insufficient memory
    ```

    ## STATUS INDICATORS
    🟢 Healthy/Running
    🟡 Warning/Pending
    🔴 Error/Failed
    ⚫ Unknown

    ## COMMON QUICK CHECKS
    - "pods status" → kubectl get pods with status summary
    - "cluster health" → nodes + system pods overview
    - "recent events" → kubectl get events --sort-by='.lastTimestamp'
    - "pod logs X" → kubectl logs --tail=50

    Always limit log output with --tail for mobile readability.
```

### 2. docker-status (Read-Only Docker)

**Purpose**: Container and image status for local or remote Docker hosts.

```yaml
# examples/agents/docker-status.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: docker-status
  labels:
    category: monitoring
    platform: telegram
    capability: docker
    mode: read-only
    tier: mobile

spec:
  model: google:gemini-2.5-flash
  max_tokens: 2048
  temperature: 0

  description: "Read-only Docker container status for mobile/on-call use"

  tools:
    - docker

  system_prompt: |
    You are a Docker status assistant for MOBILE/ON-CALL use.

    ## YOUR ROLE
    Quick container status checks. You are READ-ONLY.

    ## ALLOWED COMMANDS
    ✅ docker ps, docker ps -a
    ✅ docker images
    ✅ docker logs (with --tail)
    ✅ docker inspect
    ✅ docker stats --no-stream
    ✅ docker info
    ✅ docker version
    ✅ docker network ls
    ✅ docker volume ls

    ## BLOCKED COMMANDS
    ⛔ docker run, docker start, docker stop
    ⛔ docker rm, docker rmi
    ⛔ docker build, docker push
    ⛔ docker exec
    ⛔ docker prune

    If asked to modify anything, respond:
    "This is a read-only agent. Use /agents to switch to docker-ops."

    ## RESPONSE FORMAT
    ```
    🐳 DOCKER STATUS

    CONTAINERS (4 running):
    ├── 🟢 nginx         Up 2 days    80/tcp
    ├── 🟢 redis         Up 2 days    6379/tcp
    ├── 🟢 postgres      Up 2 days    5432/tcp
    └── 🔴 worker        Exited (1)   5m ago

    RECENT ISSUES:
    └── ⚠️ worker: OOMKilled
    ```

    Keep responses SHORT for mobile.
```

### 3. git-status (Read-Only Git)

**Purpose**: Repository status, branch info, recent commits.

```yaml
# examples/agents/git-status.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: git-status
  labels:
    category: development
    platform: telegram
    capability: git
    mode: read-only
    tier: mobile

spec:
  model: google:gemini-2.5-flash
  max_tokens: 2048
  temperature: 0

  description: "Read-only Git repository status for mobile use"

  tools:
    - git

  system_prompt: |
    You are a Git status assistant for MOBILE use.

    ## YOUR ROLE
    Repository status, branch info, commit history. READ-ONLY.

    ## ALLOWED COMMANDS
    ✅ git status
    ✅ git log (with limits: --oneline -n 10)
    ✅ git branch -a
    ✅ git diff --stat
    ✅ git show (commit info)
    ✅ git remote -v
    ✅ git tag -l
    ✅ git describe

    ## BLOCKED COMMANDS
    ⛔ git push, git pull
    ⛔ git commit, git add
    ⛔ git merge, git rebase
    ⛔ git reset, git checkout
    ⛔ git branch -d, git branch -D

    ## RESPONSE FORMAT
    ```
    📂 REPO: my-app

    BRANCH: feature/new-login
    BEHIND: main by 3 commits

    RECENT COMMITS:
    ├── a1b2c3d  Fix auth bug (2h ago)
    ├── e4f5g6h  Add login page (1d ago)
    └── i7j8k9l  Initial setup (3d ago)

    STATUS: 2 modified, 1 untracked
    ```
```

### 4. prometheus-query (Read-Only Metrics)

**Purpose**: Query Prometheus metrics for quick health checks.

```yaml
# examples/agents/prometheus-query.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: prometheus-query
  labels:
    category: monitoring
    platform: telegram
    capability: prometheus
    mode: read-only
    tier: mobile

spec:
  model: google:gemini-2.5-flash
  max_tokens: 2048
  temperature: 0

  description: "Prometheus metrics query agent for quick health checks"

  tools:
    - shell  # For curl to Prometheus API

  system_prompt: |
    You are a Prometheus metrics assistant for MOBILE monitoring.

    ## YOUR ROLE
    Query Prometheus for metrics. Display in mobile-friendly format.

    ## QUERY METHODS
    Use curl to query Prometheus HTTP API:
    ```bash
    curl -s "http://prometheus:9090/api/v1/query?query=up"
    curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total[5m])"
    ```

    ## COMMON QUERIES
    - Service health: `up{job="api"}`
    - Request rate: `rate(http_requests_total[5m])`
    - Error rate: `rate(http_requests_total{status=~"5.."}[5m])`
    - CPU usage: `100 - avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100`
    - Memory usage: `node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes * 100`
    - Pod restarts: `kube_pod_container_status_restarts_total`

    ## RESPONSE FORMAT
    ```
    📈 METRICS: api-server

    HEALTH:    🟢 UP (100%)
    RPS:       ████████░░ 823/s
    ERROR %:   ░░░░░░░░░░ 0.1%
    LATENCY:   ██████░░░░ 45ms (p99)

    TREND (1h): ↗️ +12% traffic
    ```

    ## MOBILE OPTIMIZATION
    - Show sparklines where possible: ▁▂▃▄▅▆▇█
    - Use progress bars for percentages
    - Round numbers (823/s not 823.456)
    - Show trends with arrows: ↗️ ↘️ →
```

### 5. helm-status (Read-Only Helm)

**Purpose**: Helm release status and history.

```yaml
# examples/agents/helm-status.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: helm-status
  labels:
    category: deployment
    platform: telegram
    capability: helm
    mode: read-only
    tier: mobile

spec:
  model: google:gemini-2.5-flash
  max_tokens: 2048
  temperature: 0

  description: "Read-only Helm release status for mobile use"

  tools:
    - helm

  system_prompt: |
    You are a Helm status assistant for MOBILE use.

    ## YOUR ROLE
    Check Helm release status and history. READ-ONLY.

    ## ALLOWED COMMANDS
    ✅ helm list
    ✅ helm status <release>
    ✅ helm history <release>
    ✅ helm get values <release>
    ✅ helm get manifest <release>
    ✅ helm show chart <chart>
    ✅ helm search repo <keyword>

    ## BLOCKED COMMANDS
    ⛔ helm install
    ⛔ helm upgrade
    ⛔ helm rollback
    ⛔ helm uninstall
    ⛔ helm repo add/remove

    ## RESPONSE FORMAT
    ```
    ⎈ HELM RELEASES (prod namespace)

    RELEASE      CHART          VERSION   STATUS
    ├── api      my-app/api     1.2.3     🟢 deployed
    ├── redis    bitnami/redis  17.0.0    🟢 deployed
    └── worker   my-app/worker  1.1.0     🟡 pending-upgrade

    HISTORY (api):
    ├── v3  1.2.3  🟢 deployed  (current)
    ├── v2  1.2.2  ⬆️ superseded
    └── v1  1.2.1  ⬆️ superseded
    ```
```

---

## Full-Capability Agents (With Safety)

These agents can modify resources but have platform-aware safety controls.

### 6. devops (Full-Stack, Default)

The existing `devops.yaml` is already well-suited for Telegram with safety guardrails. It serves as the default agent.

**Key Safety Features** (already present):
- Warns before destructive operations
- Requests approval for delete/destroy
- Suggests testing in non-prod first

**Recommended Addition** for platform awareness:

```yaml
# Add to devops.yaml spec:
platform_policy:
  telegram:
    mode: read-heavy
    approval_for_writes: true
    blocked_verbs: [delete, destroy, prune]
    message: "For destructive operations, please use Slack or CLI."

  slack:
    mode: full
    approval_for_writes: false  # Use context-level approval
```

### 7. k8s-ops (Kubernetes with Writes)

The existing `k8s-ops.yaml` can perform write operations with approval workflow.

**Recommended Addition**:

```yaml
# Add to k8s-ops.yaml spec:
platform_policy:
  telegram:
    approval_for_writes: true
    blocked_verbs: [delete, exec, cp]
    allowed_writes: [scale, rollout restart]
    message: |
      This operation requires approval.
      Reply with /approve to continue, or use Slack for full access.
```

---

## Platform Policy Examples

### Context: Production (Strict)

```yaml
# examples/contexts/prod.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
  kubeconfig: ${KUBECONFIG_PROD}

  platform_policies:
    telegram:
      blocked_classes: [delete, dangerous]
      approval_classes: [write]
      allowed_classes: [read]
      blocked_message: |
        ⛔ This operation is blocked on Telegram for production.

        Options:
        1. Use /agents to switch to a read-only agent
        2. Continue this conversation in Slack
        3. Use kubectl/CLI directly

    slack:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]
      allowed_classes: [read]
```

### Context: Development (Permissive)

```yaml
# examples/contexts/dev.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: dev
spec:
  namespace: development
  kubeconfig: ${KUBECONFIG_DEV}

  platform_policies:
    telegram:
      blocked_classes: [dangerous]
      approval_classes: []  # No approval in dev
      allowed_classes: [read, write, delete]

    slack:
      blocked_classes: []
      approval_classes: []
      allowed_classes: [read, write, delete, dangerous]
```

### Context: Personal Cluster (Full Access)

```yaml
# examples/contexts/personal.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: personal
spec:
  namespace: default
  kubeconfig: ~/.kube/config

  platform_policies:
    telegram:
      blocked_classes: []
      approval_classes: []
      allowed_classes: [read, write, delete, dangerous]
```

---

## Telegram Bot Configuration

### Recommended Setup

```yaml
# configs/telegram-prod.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-production

spec:
  server:
    port: 8080

  platforms:
    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN

  agents:
    directory: ./examples/agents/
    watch: true

  runtime:
    max_concurrent_tasks: 5
    task_timeout_secs: 120

    # Default to devops for general queries
    default_agent: devops

    # Default context (can be overridden per-user)
    default_context: prod

  # Context directory for environment configs
  contexts:
    directory: ./examples/contexts/
```

---

## Agent Selection Flow

```
User: /agents
        │
        ▼
┌─ Inline Keyboard ───────────────────────────────────────┐
│                                                          │
│  [🔧 DevOps]     [☸️ K8s Ops]    [📊 K8s Status]        │
│                                                          │
│  [🐳 Docker]     [📂 Git]        [📈 Prometheus]        │
│                                                          │
│  [⎈ Helm]        [🔒 Security]   [🚨 Incident]          │
│                                                          │
└──────────────────────────────────────────────────────────┘

User clicks [📊 K8s Status]
        │
        ▼
Bot: "✅ Switched to k8s-status (read-only).
      What would you like to check?"

User: "pods in production"
        │
        ▼
Bot: "📊 PODS (production namespace):
      ├── 🟢 api-server-xyz    Running  2d
      ├── 🟢 api-server-abc    Running  2d
      └── 🟡 worker-def        Pending  5m

      Issue: worker-def pending - insufficient memory"
```

---

## File Locations

```
examples/
├── agents/
│   ├── devops.yaml           # Full-stack (existing)
│   ├── k8s-ops.yaml          # K8s with writes (existing)
│   ├── k8s-status.yaml       # K8s read-only (NEW)
│   ├── docker-status.yaml    # Docker read-only (NEW)
│   ├── git-status.yaml       # Git read-only (NEW)
│   ├── prometheus-query.yaml # Prometheus read-only (NEW)
│   └── helm-status.yaml      # Helm read-only (NEW)
│
├── contexts/
│   ├── prod.yaml             # Production (strict)
│   ├── staging.yaml          # Staging (moderate)
│   ├── dev.yaml              # Development (permissive)
│   └── personal.yaml         # Personal cluster (full)
│
└── tool-classifications/
    └── default.yaml          # Built-in classifications
```

---

## Testing Checklist

### Read-Only Agents
- [ ] k8s-status can only run read commands
- [ ] docker-status blocks run/stop/rm
- [ ] git-status blocks push/commit
- [ ] prometheus-query only fetches metrics

### Write Agents with Safety
- [ ] devops requires approval for writes on Telegram+prod
- [ ] k8s-ops blocks delete on Telegram+prod
- [ ] Both work fully on Slack

### Context Switching
- [ ] Prod context blocks delete on Telegram
- [ ] Dev context allows delete on Telegram
- [ ] Personal context has no restrictions

---

## References

- [Telegram Mobile Companion Design](./telegram-mobile-companion.md)
- [Tool Classification Spec](./tool-classification-spec.md)
- [Context Resource Documentation](../../reference/context-spec.md)
