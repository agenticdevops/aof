# Agentic Ops Framework (AOF) - Schema Documentation

This directory contains comprehensive YAML schema definitions for the Agentic Ops Framework, a Rust-based framework designed for DevOps/SRE engineers who prefer Kubernetes-style declarative configuration.

## 📋 Schema Overview

### Core Resource Types

| Resource | File | Description |
|----------|------|-------------|
| **Agent** | `01-agent.yaml` | Single AI agent with model config, instructions, and tools |
| **AgentFleet** | `02-agent-fleet.yaml` | Coordinated team of agents working collaboratively |
| **AgentWorkflow** | `03-agent-workflow.yaml` | Sequential/parallel multi-step pipelines (DAG-based) |
| **AgentFlow** | `04-agent-flow.yaml` | Complex flows with branching, loops, and state machines |
| **ToolServer** | `05-tool-server.yaml` | MCP tool server definitions (stdio/sse/http) |
| **ModelConfig** | `06-model-config.yaml` | LLM provider configurations and routing |
| **Memory** | `07-memory.yaml` | Memory backend configurations for persistence |

## 🎯 Design Principles

### 1. Kubernetes-Native
- Follows Kubernetes CRD conventions (`apiVersion`, `kind`, `metadata`, `spec`, `status`)
- Familiar to ops/DevOps engineers
- Declarative configuration
- GitOps-friendly

### 2. Provider Agnostic
Supports multiple LLM providers:
- **OpenAI**: GPT-4 Turbo, GPT-3.5 Turbo
- **Anthropic**: Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku
- **Google**: Gemini 2.0 Flash, Gemini 1.5 Pro
- **Ollama**: Self-hosted models (Llama 3, CodeLlama)
- **xAI**: Grok
- **Docker Model Runner**: Containerized local models

### 3. MCP-First Tooling
- Model Context Protocol (MCP) as primary tool interface
- Three transport modes: `stdio`, `sse`, `http`
- Standardized tool discovery and execution
- Extensible tool ecosystem

### 4. Production-Ready Features
- Resource limits and quotas
- RBAC and security policies
- Observability (metrics, tracing, logging)
- Health checks and auto-recovery
- Cost management and budgets

## 🚀 Quick Start Examples

### Simple Agent
```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: Agent
metadata:
  name: my-agent
spec:
  model: "anthropic:claude-3-5-sonnet-20241022"
  description: "Helpful DevOps assistant"
  instructions: |
    You are a DevOps engineer. Help with infrastructure tasks.
  tools:
    - type: Shell
    - type: File
      config:
        baseDir: "/workspace"
```

### Agent Fleet (Code Review Team)
```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: AgentFleet
metadata:
  name: code-review-team
spec:
  strategy:
    type: Collaborative
  agents:
    - name: architect-reviewer
      spec:
        model: "anthropic:claude-3-5-sonnet-20241022"
        # ... architecture review instructions
    - name: security-reviewer
      spec:
        model: "openai:gpt-4-turbo-preview"
        # ... security review instructions
```

### Workflow (CI/CD Pipeline)
```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: AgentWorkflow
metadata:
  name: ci-pipeline
spec:
  entrypoint: build
  steps:
    - name: build
      agent:
        agentRef:
          name: builder-agent
      next:
        - test

    - name: test
      agent:
        agentRef:
          name: test-agent
      next:
        - deploy
```

## 📚 Schema Details

### 1. Agent (Single Agent)

**Key Features:**
- Model configuration with fallbacks
- Custom instructions and personality
- Tool integration (MCP-based)
- Memory backend support
- Resource limits
- Security policies
- Observability

**Use Cases:**
- Dockerfile generator
- Log analyzer
- Cloud resource manager
- Incident responder

### 2. AgentFleet (Multi-Agent Teams)

**Key Features:**
- Coordination strategies (collaborative, hierarchical, consensus)
- Shared memory and resources
- Task distribution and load balancing
- Result aggregation
- Fleet-level metrics

**Use Cases:**
- Code review teams
- Security audit teams
- Performance optimization teams
- Multi-perspective analysis

### 3. AgentWorkflow (DAG Pipelines)

**Key Features:**
- Directed Acyclic Graph (DAG) execution
- Sequential and parallel steps
- Conditional branching
- Artifact passing between steps
- Workflow-level retries
- Event hooks

**Use Cases:**
- CI/CD pipelines
- Incident response workflows
- Data processing pipelines
- Infrastructure provisioning

### 4. AgentFlow (State Machines)

**Key Features:**
- State machine control flow
- Dynamic branching
- Conditional loops
- Event-driven triggers
- Human-in-the-loop steps
- Complex decision logic

**Use Cases:**
- Adaptive deployments (canary, progressive rollout)
- Customer support automation
- Security incident response
- Multi-stage approval processes

### 5. ToolServer (MCP Servers)

**Key Features:**
- Three transport modes: stdio, sse, http
- Tool capability discovery
- Authentication and authorization
- Health checks
- Resource limits

**Supported Tool Types:**
- **CLI Tools**: kubectl, helm, terraform, docker
- **Cloud SDKs**: aws-cli, gcloud, az
- **Databases**: psql, mysql, redis-cli
- **Custom Tools**: Any MCP-compatible server

### 6. ModelConfig (LLM Configuration)

**Key Features:**
- Multi-provider support
- Intelligent routing (cost, performance, latency)
- Fallback chains
- Rate limiting and quotas
- Cost tracking and budgets
- Model capability matching

**Routing Strategies:**
- Cost-optimized
- Performance-optimized
- Latency-optimized
- Compliance-based (data residency)

### 7. Memory (Persistence)

**Key Features:**
- Multiple backends (Redis, PostgreSQL, S3, Vector DB)
- Conversation history management
- Automatic summarization
- Semantic search (vector embeddings)
- Backup and replication
- Encryption at rest and in transit

**Backend Types:**
- **Redis**: Fast distributed cache
- **PostgreSQL**: Queryable relational storage
- **S3**: Long-term archival
- **Vector DB**: Semantic search (Chroma, Pinecone, Weaviate)

## 🔧 Validation Rules

All schemas include validation rules following Kubernetes conventions:

- **Required Fields**: Marked in schema definitions
- **Enums**: Limited to specific values for type safety
- **Patterns**: Regex patterns for string validation
- **Ranges**: Min/max for numeric values
- **Cross-Field Validation**: Conditional requirements

## 📖 Field Reference

### Common Metadata Fields

```yaml
metadata:
  name: string              # Required: Unique identifier
  namespace: string         # Optional: Namespace for isolation
  labels: map[string]string # Optional: Key-value labels
  annotations: map[string]string # Optional: Non-identifying metadata
```

### Common Spec Fields

```yaml
spec:
  resources:               # Kubernetes-style resource limits
    requests:
      memory: string       # e.g., "256Mi", "1Gi"
      cpu: string          # e.g., "100m", "1"
    limits:
      memory: string
      cpu: string

  observability:           # Observability configuration
    metrics:
      enabled: bool
    tracing:
      enabled: bool
    logging:
      level: string        # debug, info, warn, error
```

### Common Status Fields

```yaml
status:
  phase: string           # Current state: Pending, Running, Ready, Failed
  conditions:             # Detailed conditions
    - type: string
      status: bool
      lastTransitionTime: timestamp
      reason: string
      message: string
  observedGeneration: int # Last observed spec generation
```

## 🎓 Best Practices

### 1. Namespace Organization
```yaml
# Separate by team or environment
metadata:
  namespace: platform-team    # or: dev, staging, prod
```

### 2. Resource Limits
```yaml
# Always set resource limits for production
spec:
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "1Gi"
      cpu: "1000m"
```

### 3. Security
```yaml
# Use secrets for credentials
spec:
  tools:
    - type: GitHub
      config:
        tokenSecretRef:
          name: github-token
          key: token
```

### 4. Observability
```yaml
# Enable metrics and tracing in production
spec:
  observability:
    metrics:
      enabled: true
    tracing:
      enabled: true
    logging:
      level: info
      format: json
```

### 5. Cost Management
```yaml
# Set budgets and alerts
spec:
  costControl:
    maxCostPerDay: 100.00
    budgetAlertThreshold: 80
```

## 🔗 References

- **Model Context Protocol (MCP)**: https://modelcontextprotocol.io
- **Kubernetes CRD Specification**: https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/
- **OpenTelemetry**: https://opentelemetry.io
- **Prometheus Metrics**: https://prometheus.io

## 🤝 Contributing

When adding new schemas:
1. Follow Kubernetes CRD conventions
2. Include comprehensive comments
3. Provide realistic examples
4. Document all fields
5. Add validation rules

## 📄 License

These schemas are part of the Agentic Ops Framework and follow the same licensing terms.
