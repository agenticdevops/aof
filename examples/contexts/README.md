# Execution Contexts

**Context = Agent + Connection Parameters**

Contexts define what you're connected to when chatting with the bot. Each context bundles:
- **Agent** - The AI assistant with specific tools (k8s-readonly, aws-readonly, etc.)
- **Connection** - Where to connect (kubeconfig, AWS profile, etc.)
- **Environment Variables** - Runtime configuration

## Available Contexts

### Project/Cluster Contexts (New!)

Use these with the `/context` command:

- **`k8s-cluster-a.yaml`** - Kubernetes Cluster A (EKS)
  - Agent: k8s-readonly (kubectl, helm)
  - Connection: cluster-a-prod context
  - Region: us-east-1

- **`aws-dev.yaml`** - AWS Development Account
  - Agent: aws-readonly (aws, terraform)
  - Connection: development profile
  - Region: us-west-2

- **`database.yaml`** - PostgreSQL Database
  - Agent: db-readonly (psql)
  - Connection: production database
  - Read-only access

- **`prometheus.yaml`** - Monitoring Stack
  - Agent: prometheus-query (promql)
  - Connection: prometheus/grafana URLs

### Bot-per-Environment Contexts

For deploying separate bots per environment:

- **`telegram-prod.yaml`** - Production bot configuration
- **`telegram-dev.yaml`** - Development bot configuration
- **`telegram-personal.yaml`** - Personal testing bot

### Environment Contexts (Legacy)

For flow-level environment configuration:

- **`prod.yaml`** - Production environment policies
- **`staging.yaml`** - Staging environment policies
- **`dev.yaml`** - Development environment policies

## Context Resource Definition

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: cluster-a
  labels:
    type: kubernetes
    region: us-east-1

spec:
  # Display for /context command
  display:
    name: "Cluster A (EKS)"
    emoji: "🔷"
    description: "Production EKS cluster"

  # Connection parameters
  connection:
    kubeconfig: ~/.kube/config
    kubecontext: cluster-a-prod
    namespace: default

  # Environment variables
  env:
    AWS_PROFILE: production
    AWS_REGION: us-east-1

  # Agent to activate with this context
  agent:
    ref: agents/k8s-readonly.yaml

  # Available tools (informational)
  tools:
    - kubectl
    - helm
```

## Usage

### From Chat Platforms

```
/context              # List contexts with inline buttons
/context cluster-a    # Switch to cluster-a context
/context info         # Show current context details
```

### In Flows

Flows automatically receive context variables:

```yaml
nodes:
  - name: execute
    type: shell
    config:
      command: |
        kubectl --context=${context.cluster} \
                --namespace=${context.namespace} \
                get pods
```

### Bot-per-Environment Deployment

Deploy separate bots for each environment:

```bash
# Development bot (permissive)
TELEGRAM_BOT_TOKEN=$DEV_BOT_TOKEN aofctl serve \
  --contexts-dir=./contexts/dev/

# Production bot (strict, read-only)
TELEGRAM_BOT_TOKEN=$PROD_BOT_TOKEN aofctl serve \
  --contexts-dir=./contexts/prod/
```

## Multi-Tenant Pattern

Use different contexts within the same bot:

| Context | Agent | Connection | Use Case |
|---------|-------|------------|----------|
| cluster-a | k8s-readonly | EKS us-east-1 | K8s operations |
| cluster-b | k8s-readonly | GKE us-central1 | K8s operations |
| aws-dev | aws-readonly | AWS development | Cloud resources |
| database | db-readonly | PostgreSQL | Database queries |
| prometheus | prometheus-query | Prometheus | Monitoring |

## See Also

- [Context Switching Guide](../../docs/guides/context-switching.md)
- [Agent Reference](../agents/README.md)
- [Flow Reference](../flows/README.md)
