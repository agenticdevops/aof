# AOF Memory, Context, and Agentic RAG System - Architecture Documentation

## Overview

This directory contains the complete architecture design for AOF's Memory, Context, and Retrieval-Augmented Generation (RAG) system. The system enables AI agents to maintain persistent memory, access organizational knowledge, and provide contextually-aware responses.

## Quick Start

1. **Start here:** [Architecture Summary](./architecture-summary.md) - High-level overview and key concepts
2. **See it in action:** [K8s Runbook Agent Example](./example-k8s-runbook-agent.yaml) - Complete working example
3. **Learn usage:** [Usage Examples](./usage-examples.md) - Common patterns and use cases
4. **Implement:** [Implementation Guide](./implementation-guide.md) - 12-week implementation roadmap

## Documents

### 1. [Architecture Summary](./architecture-summary.md)
**Purpose:** Executive overview and system introduction

**Contents:**
- Core components overview
- Key features and benefits
- Performance characteristics
- Cost analysis
- Success metrics
- Migration path

**Read this if:** You're new to the system or need a high-level understanding

---

### 2. [Memory System Overview](./memory-rag-system.md)
**Purpose:** Detailed system architecture and design principles

**Contents:**
- System diagrams and data flows
- Architecture principles
- Backend comparison tables
- Vector store evaluation
- Security considerations
- Integration points
- Future enhancements

**Read this if:** You need deep technical understanding of the architecture

---

### 3. [Memory CRD](./memory-crd.yaml)
**Purpose:** Complete Memory CRD specification

**Contents:**
- Full OpenAPI schema
- Backend configurations (Redis, PostgreSQL, SQLite, S3, Vector)
- Conversational memory settings
- Semantic memory with vector stores
- Knowledge base integration
- Lifecycle management
- Access control and RBAC
- Observability configuration

**Read this if:** You need to configure Memory resources

---

### 4. [KnowledgeBase CRD](./knowledgebase-crd.yaml)
**Purpose:** Complete KnowledgeBase CRD specification

**Contents:**
- Data source configurations
- Chunking strategies
- Processing pipelines
- Indexing configuration
- Sync policies
- Status reporting

**Supported Sources:**
- GitHub/GitLab repositories
- Confluence pages
- Notion databases
- Local files
- URLs and web scraping
- S3 buckets
- Generic Git repos
- HTTP APIs
- Databases

**Read this if:** You need to ingest knowledge into vector stores

---

### 5. [Agent Memory Integration](./agent-memory-integration.yaml)
**Purpose:** Agent, AgentFleet, and AgentFlow memory integration examples

**Contents:**
- Simple conversational memory
- RAG-enabled agents
- Inline memory configuration
- Context injection (static, dynamic, RAG)
- AgentFleet shared memory
- AgentFlow context passing
- Advanced RAG configurations
- Memory lifecycle management

**Read this if:** You need to integrate memory with agents

---

### 6. [Rust Implementation](./rust-implementation.md)
**Purpose:** Complete Rust trait definitions and implementation structures

**Contents:**
- `MemoryBackend` trait
- `VectorStore` trait
- `EmbeddingProvider` trait
- `ChunkingStrategy` trait
- `Reranker` trait
- RAG pipeline implementation
- Context manager implementation
- Complete code examples

**Read this if:** You're implementing the system in Rust

---

### 7. [K8s Runbook Agent Example](./example-k8s-runbook-agent.yaml)
**Purpose:** Complete, production-ready example implementation

**Contents:**
- Memory CRD with Redis + Qdrant
- KnowledgeBase CRDs for 3 sources (GitHub, Confluence)
- Agent with RAG and context management
- ServiceAccount and RBAC
- ConfigMaps and Secrets
- Full deployment manifests

**Use case:** Kubernetes expert agent with access to:
- Official K8s documentation
- Internal runbooks
- Past incident reports
- Real-time cluster state
- Active PagerDuty incidents

**Read this if:** You want a complete working example to deploy

---

### 8. [Usage Examples](./usage-examples.md)
**Purpose:** Practical usage patterns and examples

**Contents:**
- Query examples with sample responses
- Conversational memory examples
- Knowledge base ingestion
- Multi-agent fleet coordination
- RAG with filtering
- Context from multiple sources
- Workflow context passing
- Observability and debugging
- Programmatic API usage
- Performance benchmarks
- Best practices

**Read this if:** You need practical examples and patterns

---

### 9. [Implementation Guide](./implementation-guide.md)
**Purpose:** Step-by-step implementation roadmap

**Contents:**
- 12-week implementation plan
- Phase-by-phase deliverables
- Code structure and organization
- Technical decision rationale
- API design
- Testing strategy
- Security considerations
- Performance optimization
- Operations guide
- Cost estimation

**Read this if:** You're planning implementation or need project management info

---

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Agent Runtime                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ Conversational│  │   Semantic   │  │  Knowledge   │       │
│  │    Memory    │  │    Memory    │  │     Base     │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                  │               │
│         └─────────────────┴──────────────────┘               │
│                           │                                  │
│                    ┌──────▼──────┐                          │
│                    │   Memory    │                          │
│                    │   Manager   │                          │
│                    └──────┬──────┘                          │
├───────────────────────────┼──────────────────────────────────┤
│                           │                                  │
│    ┌──────────────────────┼─────────────────┐               │
│    │                      │                 │               │
│ ┌──▼────┐  ┌────────┐  ┌─▼──────┐  ┌──────▼────┐          │
│ │ Redis │  │Postgres│  │ SQLite │  │   Vector  │          │
│ │       │  │   SQL  │  │        │  │   Store   │          │
│ └───────┘  └────────┘  └────────┘  └───────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Memory Types

1. **Conversational Memory:**
   - Chat history between user and agent
   - Stored in Redis (fast) or PostgreSQL (persistent)
   - TTL-based expiration
   - Automatic summarization for long conversations

2. **Semantic Memory:**
   - Vector-based knowledge storage
   - Enables similarity search and RAG
   - Stored in specialized vector databases (Qdrant, Pinecone, etc.)
   - Indexed with embeddings from OpenAI, Cohere, or local models

3. **Knowledge Memory:**
   - External knowledge sources
   - Automatically ingested and indexed
   - Synced on schedule or via webhooks
   - Supports GitHub, Confluence, Notion, and more

### Context Types

1. **Static Context:**
   - Always included in agent prompts
   - Policies, guidelines, reference information
   - Loaded from ConfigMaps or inline YAML

2. **Dynamic Context:**
   - Fetched at runtime
   - Current system state (K8s API, monitoring)
   - External data (PagerDuty incidents, deployments)
   - Configurable refresh intervals

3. **RAG Context:**
   - Semantically retrieved based on user query
   - Most relevant documents from knowledge base
   - Filtered by metadata
   - Reranked for optimal relevance

### RAG Pipeline

```
Query → Embed → Search Vector Store → Retrieve Top-K → Rerank → Context
  ↓       ↓            ↓                    ↓            ↓         ↓
[Text]  [Model]    [Semantic]           [Results]    [Model]  [Inject]
```

## Quick Reference

### Embedding Models

| Model | Provider | Dimensions | Cost/1M tokens | Best For |
|-------|----------|------------|----------------|----------|
| text-embedding-3-small | OpenAI | 1536 | $0.02 | General use |
| text-embedding-3-large | OpenAI | 3072 | $0.13 | High quality |
| embed-english-v3.0 | Cohere | 1024 | $0.10 | Multilingual |
| all-MiniLM-L6-v2 | Local | 384 | Free | Cost-sensitive |

### Vector Stores

| Store | Type | Best For | Latency | Scalability |
|-------|------|----------|---------|-------------|
| Qdrant | Self-hosted | Production, filtering | <10ms | High |
| Chroma | Self-hosted | Development, simple | <20ms | Medium |
| Pinecone | Cloud | Managed, scale | <50ms | Very High |
| Milvus | Self-hosted | Large scale, GPU | <30ms | Very High |
| Weaviate | Self-hosted | Multi-modal | <20ms | High |

### Memory Backends

| Backend | Best For | Read Latency | Write Latency |
|---------|----------|--------------|---------------|
| Redis | Hot conversations | <1ms | <1ms |
| PostgreSQL | Persistent history | 1-5ms | 2-10ms |
| SQLite | Local, embedded | <1ms | 1-5ms |
| S3 | Archival, backup | 50-200ms | 100-500ms |

## Getting Started

### 1. Deploy Example Agent

```bash
# Deploy the K8s runbook agent example
kubectl apply -f docs/architecture/example-k8s-runbook-agent.yaml

# Check status
kubectl get memory,knowledgebase,agent -n sre-platform

# Query the agent
aof agent query k8s-expert \
  --message "How do I debug a CrashLoopBackOff pod?" \
  --conversation-id "test-1"
```

### 2. Create Custom Memory

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: Memory
metadata:
  name: my-memory
spec:
  backend:
    type: Redis
    redis:
      url: redis://redis:6379
  conversational:
    enabled: true
    maxMessages: 50
  semantic:
    enabled: true
    embedding:
      provider: OpenAI
      model: text-embedding-3-small
    vectorStore:
      type: Qdrant
      qdrant:
        url: http://qdrant:6333
```

### 3. Create Knowledge Base

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: KnowledgeBase
metadata:
  name: my-docs
spec:
  sources:
    - name: github-docs
      type: GitHub
      github:
        owner: myorg
        repo: docs
        branch: main
        paths: ["docs/**/*.md"]
  chunking:
    strategy: markdown
  indexing:
    memoryRef: my-memory
```

### 4. Use in Agent

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: Agent
metadata:
  name: my-agent
spec:
  provider: anthropic
  model: claude-3-5-sonnet-20241022
  memory:
    ref: my-memory
    conversational:
      enabled: true
    rag:
      enabled: true
      topK: 5
```

## Performance Benchmarks

### RAG Retrieval
- **Top-K=5:** 52ms avg, 89ms p95
- **With Reranking:** 210ms avg, 289ms p95

### Memory Operations
- **Redis Read:** 0.6ms avg
- **Redis Write:** 0.8ms avg
- **Vector Search (Qdrant):** 25ms avg

### Knowledge Ingestion
- **500 files:** ~4.5 minutes
- **10 changed files:** ~30 seconds

## Cost Estimate

### Small Scale (10K docs, 1K conversations)
- **Embeddings:** $6/month
- **Storage:** $1.60/month
- **Total:** ~$8/month

### Enterprise (1M docs, 100K conversations)
- **Embeddings:** $600/month
- **Storage:** $160/month
- **Total:** ~$760/month

## Support and Resources

### Documentation
- Full specs in this directory
- Inline comments in YAML files
- Code examples in Rust implementation

### Examples
- K8s runbook agent (complete example)
- Usage patterns and snippets
- Integration examples

### Implementation
- 12-week roadmap
- Phase-by-phase deliverables
- Testing strategies

## Next Steps

1. **Understand the System:**
   - Read [Architecture Summary](./architecture-summary.md)
   - Review [System Overview](./memory-rag-system.md)

2. **See it in Action:**
   - Deploy [K8s Runbook Agent](./example-k8s-runbook-agent.yaml)
   - Explore [Usage Examples](./usage-examples.md)

3. **Implement:**
   - Follow [Implementation Guide](./implementation-guide.md)
   - Review [Rust Implementation](./rust-implementation.md)

4. **Customize:**
   - Modify [Memory CRD](./memory-crd.yaml)
   - Configure [KnowledgeBase CRD](./knowledgebase-crd.yaml)
   - Integrate with [Agents](./agent-memory-integration.yaml)

## Contributing

When adding to this architecture:
1. Update relevant documents
2. Add examples to usage-examples.md
3. Update this README
4. Keep code in sync with specs

## Questions?

- Architecture questions: Review system overview and summary
- Implementation questions: Check implementation guide
- Usage questions: See usage examples
- Integration questions: Review agent integration examples
