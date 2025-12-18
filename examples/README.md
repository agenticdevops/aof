# AOF Examples Library

Welcome to the AOF (Agentic Ops Framework) examples library! This collection demonstrates how to build production-ready AI agents for DevOps and SRE workflows using **composable, reusable Kubernetes-style YAML**.

## 🎯 Architecture Overview

AOF uses a **composable architecture** where resources are defined once and referenced everywhere:

```
agents/        → Define agents ONCE (k8s-ops, security, incident, etc.)
fleets/        → Compose agents into teams
flows/         → Define orchestration logic (context-agnostic)
contexts/      → Environment configurations (prod, staging, dev)
triggers/      → Platform connections (Slack, Telegram, PagerDuty)
bindings/      → Tie everything together

complete/      → Full working examples
  ├─ single-bot/    → Simplest setup (one agent, one channel)
  └─ enterprise/    → Multi-tenant production setup
```

**Key Principle**: Define once, reference everywhere using `ref:` syntax.

## 🚀 Quick Start

### Option 1: Simplest Setup (5 minutes)

**Single Slack bot for Kubernetes operations:**

```bash
cd examples/complete/single-bot

# Set environment variables
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_SIGNING_SECRET=your-secret
export GOOGLE_API_KEY=your-gemini-key

# Start the bot
aofctl serve --config daemon-config.yaml

# Test in Slack
# @k8s-bot get pods
```

See [complete/single-bot/README.md](complete/single-bot/README.md) for details.

### Option 2: Enterprise Multi-Tenant (Production)

**Multiple environments, platforms, and workflows:**

```bash
cd examples/complete/enterprise

# Configure environments
export KUBECONFIG_PROD=~/.kube/prod-config
export KUBECONFIG_STAGING=~/.kube/staging-config

# Start multi-tenant daemon
aofctl serve --config daemon-config.yaml
```

See [complete/enterprise/README.md](complete/enterprise/README.md) for details.

---

## 📦 Core Resources

### Agents (`agents/`)

Individual AI agents defined **once** and referenced everywhere:

| Agent | Purpose | Tools |
|-------|---------|-------|
| `k8s-ops.yaml` | Kubernetes operations | kubectl, helm |
| `security.yaml` | Security scanning | trivy, semgrep |
| `incident.yaml` | Incident response | kubectl, prometheus, loki |
| `devops.yaml` | Full-stack DevOps | kubectl, docker, terraform, git |
| `general-assistant.yaml` | General purpose helper | None (knowledge-based) |

**Usage**:
```bash
# Direct execution
aofctl run agent k8s-ops "check pod status"

# Reference in fleets
agents:
  - ref: agents/k8s-ops.yaml

# Reference in flows
agent: k8s-ops
```

📖 See [agents/README.md](agents/README.md) for details.

---

### Fleets (`fleets/`)

Teams of agents working together:

| Fleet | Agents | Purpose |
|-------|--------|---------|
| `code-review-team.yaml` | security + k8s-ops | Code and manifest reviews |
| `incident-team.yaml` | incident + k8s-ops + security | Incident response |

**Usage**:
```bash
# Direct execution
aofctl run fleet incident-team "investigate outage"

# Reference in flows
fleet: incident-team
```

📖 See [fleets/README.md](fleets/README.md) for details.

---

### Flows (`flows/`)

Context-agnostic orchestration logic:

| Flow | Triggers On | Features |
|------|-------------|----------|
| `k8s-ops-flow.yaml` | kubectl, k8s, pod, deploy | Approval for destructive ops |
| `incident-flow.yaml` | incident, alert, outage | Auto-investigation, RCA |
| `deploy-flow.yaml` | deploy, release, rollout | Security scan + validation |

**Key Design**: Flows don't know about Slack/Telegram or prod/staging. That's configured via bindings.

📖 See [flows/README.md](flows/README.md) for details.

---

### Contexts (`contexts/`)

Environment-specific configuration:

| Context | Approvals | Audit | Rate Limit |
|---------|-----------|-------|------------|
| `prod.yaml` | Strict | Full (S3) | 20/min |
| `staging.yaml` | Relaxed | Basic | 40/min |
| `dev.yaml` | None | None | 100/min |

**What contexts configure**:
- Kubernetes cluster (kubeconfig)
- Approval workflows
- Audit logging
- Resource limits
- Environment variables

📖 See [contexts/README.md](contexts/README.md) for details.

---

### Triggers (`triggers/`)

Platform-specific message sources:

| Trigger | Platform | Channels/Groups |
|---------|----------|-----------------|
| `slack-prod.yaml` | Slack | #production, #prod-alerts |
| `slack-staging.yaml` | Slack | #staging, #qa-testing |
| `telegram-oncall.yaml` | Telegram | SRE on-call group |
| `pagerduty.yaml` | Webhook | Incident alerts |

📖 See [triggers/README.md](triggers/README.md) for details.

---

### Bindings (`bindings/`)

**Tie everything together**: trigger + flow + context = working bot

| Binding | Platform | Environment | Purpose |
|---------|----------|-------------|---------|
| `prod-slack-k8s.yaml` | Slack #prod | Production | K8s ops with strict approvals |
| `staging-slack-k8s.yaml` | Slack #staging | Staging | K8s ops with relaxed approvals |
| `oncall-telegram-incident.yaml` | Telegram | Production | Fast incident response |
| `pagerduty-incident.yaml` | Webhook | Production | Auto incident response |

**Example binding**:
```yaml
spec:
  trigger: triggers/slack-prod.yaml      # Where messages come from
  flow: flows/k8s-ops-flow.yaml          # What workflow to run
  context: contexts/prod.yaml            # Environment configuration
```

📖 See [bindings/README.md](bindings/README.md) for details.

---

## 🏢 Multi-Tenant Architecture

The power of this architecture is **reusability**:

```
Same Flow, Different Environments:
┌──────────────┐
│ k8s-ops-flow │ (defined once)
└──────┬───────┘
       ├─→ prod-slack-k8s (Slack #prod, strict approvals)
       ├─→ staging-slack-k8s (Slack #staging, relaxed)
       └─→ dev-slack-k8s (Slack #dev, no approvals)

Same Flow, Different Platforms:
┌────────────────┐
│ incident-flow  │ (defined once)
└────────┬───────┘
         ├─→ oncall-telegram (Telegram, interactive)
         └─→ pagerduty-auto (Webhook, automated)
```

**Benefits**:
- ✅ No duplication
- ✅ Consistent behavior across environments
- ✅ Easy to add new environments/platforms
- ✅ Centralized configuration management

---

## 📋 Complete Examples

### Single Bot Example

**What**: Simplest possible setup - one bot, one channel
**Use for**: Learning, testing, local development

```bash
cd examples/complete/single-bot
aofctl serve --config daemon-config.yaml
```

📖 See [complete/single-bot/README.md](complete/single-bot/README.md)

---

### Enterprise Example

**What**: Production-ready multi-tenant setup
**Includes**:
- 3 environments (prod, staging, dev)
- 4 platforms (Slack, Telegram, PagerDuty, WhatsApp)
- 5 agents + 2 fleets + 3 flows
- Approval workflows
- Audit logging
- Multi-tenant routing

```bash
cd examples/complete/enterprise
aofctl serve --config daemon-config.yaml
```

📖 See [complete/enterprise/README.md](complete/enterprise/README.md)

---

## 🎯 Usage Patterns

### Pattern 1: Direct Agent Execution

```bash
# Run any agent directly
aofctl run agent k8s-ops "get pods in default namespace"
aofctl run agent security "scan image nginx:latest"
aofctl run agent incident "investigate API latency"
```

### Pattern 2: Fleet Collaboration

```bash
# Run multiple agents as a team
aofctl run fleet incident-team "prod outage investigation"
aofctl run fleet code-review-team "review PR #123"
```

### Pattern 3: Event-Driven Workflows

```bash
# Daemon processes events from platforms
aofctl serve --config daemon-config.yaml

# Slack message → k8s-ops-flow → Response
# PagerDuty alert → incident-flow → Auto-remediation
```

---

## 🛠️ Customization

### Adding a New Environment

1. Create context: `contexts/qa.yaml`
2. Create trigger: `triggers/slack-qa.yaml` (if needed)
3. Create binding: `bindings/qa-slack-k8s.yaml`
4. Add to `daemon-config.yaml`

### Adding a New Platform

1. Create trigger: `triggers/discord-prod.yaml`
2. Create bindings for each environment
3. Add to `daemon-config.yaml`

### Adding a New Capability

1. Create flow: `flows/database-ops-flow.yaml`
2. Create bindings for each environment
3. Add to `daemon-config.yaml`

---

## 📚 Documentation by Directory

- [agents/README.md](agents/README.md) - Agent definitions
- [fleets/README.md](fleets/README.md) - Fleet compositions
- [flows/README.md](flows/README.md) - Orchestration flows
- [contexts/README.md](contexts/README.md) - Environment contexts
- [triggers/README.md](triggers/README.md) - Platform triggers
- [bindings/README.md](bindings/README.md) - Resource bindings
- [complete/single-bot/README.md](complete/single-bot/README.md) - Simple setup
- [complete/enterprise/README.md](complete/enterprise/README.md) - Enterprise setup

---

## 🔧 Prerequisites

### Required

```bash
# AOF installation
curl -sSL https://docs.aof.sh/install.sh | bash

# LLM API key (Gemini 2.5 Flash recommended)
export GOOGLE_API_KEY=your-key-here
```

### Optional (depending on use case)

```bash
# Kubernetes
kubectl

# Docker
docker

# Security tools
trivy
semgrep

# Platform tokens
export SLACK_BOT_TOKEN=xoxb-xxxxx
export TELEGRAM_BOT_TOKEN=xxxxx
export PAGERDUTY_WEBHOOK_TOKEN=xxxxx
```

---

## 🤝 Contributing

Have a great example? Submit a PR!

**Guidelines:**
1. Follow the composable architecture pattern
2. Use `ref:` syntax instead of inline definitions
3. Add README.md in your directory
4. Include usage examples
5. Test before submitting

---

## 📄 License

All examples are provided under the Apache 2.0 License.

---

**Happy automating!** 🚀

For help: https://github.com/agenticdevops/aof/issues
