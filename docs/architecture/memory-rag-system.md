# AOF Memory, Context, and Agentic RAG System Architecture

## Overview

The AOF Memory and RAG system provides persistent memory, contextual information injection, and retrieval-augmented generation capabilities for AI agents in DevOps/SRE workflows.

### Key Components

1. **Memory CRD** - Persistent storage backend configuration
2. **KnowledgeBase CRD** - RAG data source management
3. **Context System** - Static/dynamic/RAG context injection
4. **Vector Store Integration** - Semantic search and retrieval
5. **Embedding Providers** - Multi-provider embedding support

### Architecture Principles

- **Backend Agnostic**: Support Redis, PostgreSQL, SQLite, S3, Vector DBs
- **Provider Neutral**: OpenAI, Cohere, local embeddings
- **Kubernetes Native**: Full CRD integration
- **Performance Optimized**: Rust-based with async I/O
- **Security First**: Encrypted storage, access control

## System Diagram

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
│                                                              │
│         ┌────────────────────────────────┐                  │
│         │  Vector Stores                 │                  │
│         │  ┌──────┐ ┌────────┐ ┌──────┐ │                  │
│         │  │Qdrant│ │ Chroma │ │Pinecone│ │                  │
│         │  └──────┘ └────────┘ └──────┘ │                  │
│         │  ┌────────┐ ┌─────────┐       │                  │
│         │  │ Milvus │ │Weaviate │       │                  │
│         │  └────────┘ └─────────┘       │                  │
│         └────────────────────────────────┘                  │
│                                                              │
│         ┌────────────────────────────────┐                  │
│         │  Embedding Providers           │                  │
│         │  ┌──────┐ ┌────────┐ ┌──────┐ │                  │
│         │  │OpenAI│ │ Cohere │ │ Local│ │                  │
│         │  └──────┘ └────────┘ └──────┘ │                  │
│         └────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### Conversational Memory Flow
```
User Message → Agent → Store in Memory → Retrieve Context → LLM Response
                ↓                           ↑
            [Redis/SQL]              [Last N messages]
```

### RAG Flow
```
Query → Embed Query → Vector Search → Retrieve Top-K → Rerank → Context
         ↓              ↓                  ↓              ↓         ↓
    [Embedding]    [Vector DB]         [Chunks]      [Reranker] [Inject]
```

### Knowledge Ingestion Flow
```
Source → Fetch → Chunk → Embed → Store → Index
  ↓       ↓       ↓       ↓       ↓       ↓
[GitHub] [API]  [Split] [Model] [Vector] [Search]
```

## Performance Characteristics

### Memory Backends

| Backend    | Read Latency | Write Latency | Scalability | Use Case              |
|------------|--------------|---------------|-------------|-----------------------|
| Redis      | <1ms         | <1ms          | High        | Hot conversational    |
| PostgreSQL | 1-5ms        | 2-10ms        | High        | Structured memory     |
| SQLite     | <1ms         | 1-5ms         | Low         | Local/embedded        |
| S3         | 50-200ms     | 100-500ms     | Very High   | Cold storage/archive  |

### Vector Stores

| Store     | Index Speed | Search Speed | Scalability | Features              |
|-----------|-------------|--------------|-------------|-----------------------|
| Qdrant    | Fast        | <10ms        | High        | Filtering, payload    |
| Chroma    | Fast        | <20ms        | Medium      | Easy, embeddings      |
| Pinecone  | Fast        | <50ms        | Very High   | Managed, distributed  |
| Milvus    | Medium      | <30ms        | Very High   | GPU support           |
| Weaviate  | Fast        | <20ms        | High        | GraphQL, multi-modal  |

## Security Considerations

1. **Encryption at Rest**: All backends support encryption
2. **Encryption in Transit**: TLS for all network communications
3. **Access Control**: RBAC integration with K8s
4. **Secret Management**: Kubernetes secrets for API keys
5. **Data Isolation**: Namespace-based multi-tenancy
6. **Audit Logging**: All memory operations logged

## Integration Points

### With Agent CRD
- Automatic memory injection into agent context
- Conversation history management
- RAG-based knowledge retrieval

### With AgentFleet CRD
- Shared memory across fleet members
- Collaborative knowledge building
- Fleet-wide context propagation

### With AgentFlow CRD
- Context passing between workflow steps
- Flow-scoped memory isolation
- Cross-step knowledge transfer

## Future Enhancements

1. **Multi-modal Memory**: Support images, audio, video
2. **Graph Memory**: Knowledge graph integration
3. **Federated Learning**: Distributed embedding training
4. **Auto-chunking**: LLM-guided intelligent chunking
5. **Active Retrieval**: Predictive pre-fetching
6. **Memory Compression**: LLM-based summarization
7. **Temporal Memory**: Time-aware retrieval
