# Context Switching Guide

Switch between different projects, clusters, and cloud accounts seamlessly within your chat platform.

## Overview

AOF uses **Contexts** to define what you're connected to:
- **Agent** - The AI assistant with specific tools (k8s-readonly, aws-readonly, etc.)
- **Connection** - Where to connect (kubeconfig, AWS profile, etc.)
- **Environment Variables** - Runtime configuration

**Key Principle**: Context = Agent + Connection Parameters

## Architecture

### Bot-per-Environment Pattern

Deploy separate bots for each environment (dev, staging, prod):

```
┌─────────────────────────────────────────────────────────────────────┐
│  SEPARATE BOTS (One per Environment - Deployment/Config concern)    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  @company-dev-bot         @company-staging-bot    @company-prod-bot │
│  (in #dev channel)        (in #staging channel)   (in #sre channel) │
│                                                                      │
│  Policy: permissive       Policy: moderate        Policy: strict    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Context-per-Project Pattern

Within each bot, switch between different projects/clusters:

```
┌─────────────────────────────────────────────────────────────────────┐
│  CONTEXTS (Switchable within each bot via /context)                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  /context → Inline Keyboard:                                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  [🔷 Cluster A (EKS)]   [🔷 Cluster B (GKE)]                    ││
│  │  [☁️ AWS Dev]           [☁️ GCP Dev]                             ││
│  │  [🗄️ Database]          [📊 Prometheus]                         ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  Each context has:                                                   │
│  • Different tools (kubectl vs aws vs psql)                          │
│  • Different agent (k8s-readonly vs aws-readonly vs db-readonly)     │
│  • Different connection params (kubeconfig, aws_profile, etc.)       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### From Telegram/Slack

```
/context              # List available contexts with inline buttons
/context cluster-a    # Switch to cluster-a context
/context info         # Show current context details
```

### User Experience

1. **Select Context** - Tap a button or type `/context <name>`
2. **See Confirmation** - Shows agent being activated and connection details
3. **Start Working** - All messages now use that agent/connection

```
User: /context

Bot: **Select Context**
     Current: 🔷 Cluster A (k8s-readonly)

     [🔷 Cluster A ✓] [🔷 Cluster B]
     [☁️ AWS Dev]      [🗄️ Database]

User: *taps AWS Dev*

Bot: ✅ Switched to ☁️ *AWS Dev*

     🔄 Switching agent: aws-readonly

     **Connection:**
     • AWS Account: 123456789
     • Region: us-west-2

     **Tools Available:**
     • aws, terraform

     You can now ask questions about AWS resources.

User: list ec2 instances in us-west-2

Bot: ☁️ AWS Dev | aws-readonly
     ────────────────────────────
     [AWS CLI output...]
```

## Context Resource Definition

### Example: Kubernetes Cluster

```yaml
# examples/contexts/k8s-cluster-a.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: cluster-a
  labels:
    type: kubernetes
    region: us-east-1

spec:
  display:
    name: "Cluster A (EKS)"
    emoji: "🔷"
    description: "Production EKS cluster in us-east-1"

  # Connection parameters
  connection:
    kubeconfig: ~/.kube/config
    kubecontext: cluster-a-prod
    namespace: default

  # Environment variables set when this context is active
  env:
    KUBECONFIG: ~/.kube/config
    KUBECTL_CONTEXT: cluster-a-prod
    AWS_PROFILE: production
    AWS_REGION: us-east-1

  # Agent to use for this context
  agent:
    ref: agents/k8s-readonly.yaml

  # Tools available in this context
  tools:
    - kubectl
    - helm
    - docker
```

### Example: AWS Account

```yaml
# examples/contexts/aws-dev.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: aws-dev
  labels:
    type: cloud
    provider: aws

spec:
  display:
    name: "AWS Dev Account"
    emoji: "☁️"
    description: "AWS development account (123456789)"

  connection:
    aws_profile: development
    aws_region: us-west-2
    aws_account_id: "123456789012"

  env:
    AWS_PROFILE: development
    AWS_REGION: us-west-2
    AWS_DEFAULT_REGION: us-west-2

  agent:
    ref: agents/aws-readonly.yaml

  tools:
    - aws
    - terraform
```

### Example: Database

```yaml
# examples/contexts/database.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: database
  labels:
    type: database
    engine: postgresql

spec:
  display:
    name: "PostgreSQL"
    emoji: "🗄️"
    description: "Production PostgreSQL database (read-only)"

  connection:
    database_url: ${DATABASE_URL}
    database_host: db.example.com
    database_name: production

  env:
    PGHOST: db.example.com
    PGDATABASE: production
    PGUSER: readonly

  agent:
    ref: agents/db-readonly.yaml

  tools:
    - psql
```

## Setting Up Contexts

### 1. Create Context Directory

```bash
mkdir -p ~/.aof/contexts/

# Copy examples
cp examples/contexts/*.yaml ~/.aof/contexts/
```

### 2. Customize Connections

Edit each context file with your actual credentials:

```yaml
# ~/.aof/contexts/k8s-cluster-a.yaml
spec:
  connection:
    kubecontext: your-actual-context-name
```

### 3. Create Agents for Each Context

Each context references an agent with appropriate tools:

```yaml
# agents/k8s-readonly.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-readonly
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Kubernetes operations assistant.
    Only use read-only kubectl commands.
  tools:
    - kubectl
    - helm
```

### 4. Start the Server with Contexts

```bash
aofctl serve \
  --agents-dir ./agents \
  --contexts-dir ./contexts \
  --flows-dir ./flows
```

## Bot-per-Environment Deployment

For enterprise deployments, run separate bots per environment:

```bash
# Development Bot (permissive)
TELEGRAM_BOT_TOKEN=$DEV_BOT_TOKEN aofctl serve \
  --contexts-dir=./contexts/dev/ \
  --policy=permissive

# Staging Bot (moderate)
TELEGRAM_BOT_TOKEN=$STAGING_BOT_TOKEN aofctl serve \
  --contexts-dir=./contexts/staging/ \
  --policy=moderate

# Production Bot (strict, read-only)
TELEGRAM_BOT_TOKEN=$PROD_BOT_TOKEN aofctl serve \
  --contexts-dir=./contexts/prod/ \
  --policy=strict
```

**Benefits:**
- **Safety Isolation** - Different bots = different risk levels
- **Team Access Control** - Invite users to appropriate bot only
- **Clear Visual Distinction** - Different bot name/avatar per env
- **No Code Changes** - Same code, different config

## Flows with Context Injection

Flows automatically receive the current context:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: kubectl-apply-approval
spec:
  # Flow receives these variables from current context:
  # ${context.name}      - "cluster-a"
  # ${context.cluster}   - "cluster-a-prod"
  # ${context.namespace} - "default"
  # ${context.aws_profile} - "production"

  nodes:
    - name: validate
      type: shell
      config:
        command: |
          kubectl --context=${context.cluster} \
                  --namespace=${context.namespace} \
                  ${command} --dry-run=client
```

## Context Commands Reference

| Command | Description |
|---------|-------------|
| `/context` | Show inline keyboard with all contexts |
| `/context <name>` | Switch directly to named context |
| `/context info` | Show detailed current context info |

## Best Practices

### 1. Use Descriptive Display Names

```yaml
spec:
  display:
    name: "Prod US-East (EKS)"  # Clear and specific
    emoji: "🔴"                   # Visual indicator
    description: "Production Kubernetes cluster in us-east-1"
```

### 2. Group Contexts by Type

```yaml
metadata:
  labels:
    type: kubernetes    # kubernetes, cloud, database, monitoring
    provider: aws       # aws, gcp, azure
    region: us-east-1   # For regional distinction
```

### 3. One Agent per Context Type

Create specialized read-only agents:

- `k8s-readonly` - kubectl, helm only
- `aws-readonly` - aws CLI only
- `db-readonly` - psql with SELECT only
- `prometheus-query` - PromQL only

### 4. Document Context Purpose

```yaml
spec:
  display:
    description: |
      Production EKS cluster in us-east-1.
      Contains customer-facing services.
      Read-only access from mobile platforms.
```

## Troubleshooting

### "Context not found"

```bash
# Verify contexts are loaded
aofctl get contexts

# Check contexts directory
ls ~/.aof/contexts/
```

### "Agent not found for context"

Ensure the agent referenced exists:

```bash
aofctl get agents

# Check agent reference in context
spec:
  agent:
    ref: agents/k8s-readonly.yaml  # This file must exist
```

### "Connection failed"

Test the connection outside AOF:

```bash
# For Kubernetes
kubectl --context=cluster-a-prod get nodes

# For AWS
aws --profile development sts get-caller-identity
```

### Wrong context used

Check current context:

```
/context info
```

## Migration from /env and /agents

If you were using the separate `/env` and `/agents` commands:

| Old Command | New Command |
|-------------|-------------|
| `/env` | `/context` (contexts include agent selection) |
| `/env prod` | `/context cluster-a` (named context) |
| `/agents` | `/context` (each context has an agent) |
| `/agents k8s-readonly` | `/context cluster-a` (context bundles the agent) |

**Key Change:** Context = Agent + Connection. You no longer select agents separately.

## Related Guides

- [Platform Policies Reference](../reference/platform-policies.md)
- [Safety Layer Guide](safety-layer.md)
- [AgentFlow Routing Guide](agentflow-routing.md)
- [Bot Deployment Guide](bot-deployment.md)
