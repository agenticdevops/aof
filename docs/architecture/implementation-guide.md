# AOF Memory & RAG System - Implementation Guide

## Implementation Phases

### Phase 1: Core Memory Backend (Week 1-2)
**Goal:** Basic memory storage and retrieval

**Tasks:**
1. Implement `MemoryBackend` trait
2. Redis backend implementation
3. PostgreSQL backend implementation
4. SQLite backend implementation
5. Unit tests for all backends
6. Memory CRD controller basics

**Deliverables:**
- `memory-backend/` crate with trait definitions
- `memory-redis/` implementation
- `memory-postgres/` implementation
- `memory-sqlite/` implementation
- Basic Memory CRD reconciliation

**Code Structure:**
```
src/
├── memory/
│   ├── mod.rs                 # Core traits
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── redis.rs           # Redis implementation
│   │   ├── postgres.rs        # PostgreSQL implementation
│   │   ├── sqlite.rs          # SQLite implementation
│   │   └── s3.rs              # S3 implementation (Phase 2)
│   ├── conversational.rs      # Conversational memory
│   └── manager.rs             # Memory manager
```

### Phase 2: Vector Store Integration (Week 3-4)
**Goal:** Vector storage and semantic search

**Tasks:**
1. Implement `VectorStore` trait
2. Qdrant integration
3. Chroma integration
4. Pinecone integration (optional)
5. Embedding provider abstraction
6. OpenAI embedding provider
7. Cohere embedding provider
8. Unit and integration tests

**Deliverables:**
- `vector-store/` crate with trait definitions
- `vector-qdrant/` implementation
- `vector-chroma/` implementation
- `embedding-providers/` crate
- Vector search benchmarks

**Code Structure:**
```
src/
├── vector/
│   ├── mod.rs                 # Core traits
│   ├── store/
│   │   ├── mod.rs
│   │   ├── qdrant.rs          # Qdrant implementation
│   │   ├── chroma.rs          # Chroma implementation
│   │   ├── pinecone.rs        # Pinecone implementation
│   │   ├── milvus.rs          # Milvus implementation
│   │   └── weaviate.rs        # Weaviate implementation
│   ├── embedding/
│   │   ├── mod.rs
│   │   ├── openai.rs          # OpenAI embeddings
│   │   ├── cohere.rs          # Cohere embeddings
│   │   ├── huggingface.rs     # HuggingFace embeddings
│   │   └── local.rs           # Local model embeddings
│   └── index.rs               # Vector indexing
```

### Phase 3: Chunking & Processing (Week 5)
**Goal:** Text chunking and document processing

**Tasks:**
1. Implement `ChunkingStrategy` trait
2. Fixed-size chunking
3. Semantic chunking
4. Markdown-aware chunking
5. Recursive chunking
6. Document processors (GitHub, Confluence, etc.)
7. Unit tests

**Deliverables:**
- `chunking/` crate with multiple strategies
- `processors/` crate for data sources
- Chunking benchmarks

**Code Structure:**
```
src/
├── chunking/
│   ├── mod.rs                 # Core traits
│   ├── fixed.rs               # Fixed-size chunking
│   ├── semantic.rs            # Semantic chunking
│   ├── markdown.rs            # Markdown-aware chunking
│   └── recursive.rs           # Recursive chunking
├── processors/
│   ├── mod.rs
│   ├── github.rs              # GitHub source processor
│   ├── gitlab.rs              # GitLab source processor
│   ├── confluence.rs          # Confluence processor
│   ├── notion.rs              # Notion processor
│   ├── local_files.rs         # Local files processor
│   └── url.rs                 # URL/web scraping processor
```

### Phase 4: RAG Pipeline (Week 6)
**Goal:** Complete RAG implementation

**Tasks:**
1. Implement RAG pipeline
2. Hybrid search (semantic + keyword)
3. Reranking support
4. Context builder
5. Query enhancement (HyDE, decomposition)
6. Multi-query retrieval
7. Integration tests

**Deliverables:**
- `rag/` crate with complete pipeline
- Reranking implementations
- RAG performance benchmarks

**Code Structure:**
```
src/
├── rag/
│   ├── mod.rs                 # Core pipeline
│   ├── retrieval.rs           # Retrieval strategies
│   ├── reranking.rs           # Reranking implementations
│   ├── query_enhancement.rs   # Query optimization
│   ├── context_builder.rs     # Context building
│   └── fusion.rs              # Result fusion strategies
```

### Phase 5: Knowledge Base Management (Week 7)
**Goal:** KnowledgeBase CRD and ingestion

**Tasks:**
1. KnowledgeBase CRD controller
2. Source connectors (GitHub, Confluence, etc.)
3. Ingestion pipeline
4. Scheduling and webhooks
5. Sync management
6. Status reporting

**Deliverables:**
- KnowledgeBase CRD controller
- Source connector implementations
- Ingestion monitoring

**Code Structure:**
```
src/
├── knowledge/
│   ├── mod.rs                 # Core types
│   ├── controller.rs          # K8s controller
│   ├── ingestion/
│   │   ├── mod.rs
│   │   ├── pipeline.rs        # Ingestion pipeline
│   │   ├── scheduler.rs       # Sync scheduling
│   │   └── webhook.rs         # Webhook handlers
│   └── sources/
│       ├── mod.rs
│       ├── github.rs          # GitHub connector
│       ├── confluence.rs      # Confluence connector
│       └── ...
```

### Phase 6: Context Management (Week 8)
**Goal:** Full context system

**Tasks:**
1. Context manager implementation
2. Static context loading
3. Dynamic context fetching
4. K8s API context source
5. HTTP context source
6. Context composition
7. Token optimization

**Deliverables:**
- `context/` crate
- Context source implementations
- Agent integration

**Code Structure:**
```
src/
├── context/
│   ├── mod.rs                 # Core manager
│   ├── manager.rs             # Context manager
│   ├── sources/
│   │   ├── mod.rs
│   │   ├── static_source.rs   # ConfigMap/inline
│   │   ├── k8s_api.rs         # K8s API queries
│   │   ├── http.rs            # HTTP endpoints
│   │   └── flow_context.rs    # AgentFlow context
│   └── builder.rs             # Context builder
```

### Phase 7: Agent Integration (Week 9)
**Goal:** Integrate memory/RAG with Agent CRD

**Tasks:**
1. Update Agent CRD schema
2. Memory injection in agent runtime
3. RAG context injection
4. Conversational memory management
5. Agent-level configuration
6. Testing with real agents

**Deliverables:**
- Updated Agent controller with memory support
- Integration tests
- Example agents

### Phase 8: AgentFleet & AgentFlow Integration (Week 10)
**Goal:** Multi-agent memory sharing

**Tasks:**
1. Shared memory for AgentFleet
2. Fleet-wide context propagation
3. AgentFlow context passing
4. Cross-step memory
5. Integration tests

**Deliverables:**
- AgentFleet shared memory
- AgentFlow context system
- Examples

### Phase 9: Observability & Monitoring (Week 11)
**Goal:** Production-ready monitoring

**Tasks:**
1. Prometheus metrics
2. OpenTelemetry tracing
3. Structured logging
4. Performance dashboards
5. Alert rules
6. Health checks

**Deliverables:**
- Metrics exporters
- Grafana dashboards
- Alert rules
- Runbooks

### Phase 10: Production Hardening (Week 12)
**Goal:** Production readiness

**Tasks:**
1. Error handling and retries
2. Rate limiting
3. Circuit breakers
4. Resource limits
5. Security review
6. Performance optimization
7. Documentation
8. Load testing

**Deliverables:**
- Production-ready release
- Complete documentation
- Load test results
- Security audit report

## Technical Decisions

### Decision 1: Vector Database Selection

**Options:**
1. **Qdrant** (Recommended for self-hosted)
   - Pros: Fast, open-source, good filtering, Rust-native
   - Cons: Relatively new, smaller community
   - Use when: Self-hosted deployment, need filtering

2. **Pinecone** (Recommended for cloud)
   - Pros: Managed, scalable, reliable
   - Cons: Cost, vendor lock-in
   - Use when: Cloud deployment, need scalability

3. **Chroma** (Recommended for development)
   - Pros: Simple, embedded option, good DX
   - Cons: Less mature for production
   - Use when: Development, simple use cases

**Decision: Support all three, default to Qdrant**

### Decision 2: Embedding Provider

**Options:**
1. **OpenAI** (text-embedding-3-small)
   - Dimensions: 1536
   - Cost: $0.02 / 1M tokens
   - Quality: High
   - Use when: Best quality needed

2. **Cohere** (embed-english-v3.0)
   - Dimensions: 1024
   - Cost: $0.10 / 1M tokens
   - Quality: High
   - Use when: Multilingual support

3. **Local Models** (all-MiniLM-L6-v2)
   - Dimensions: 384
   - Cost: Free (compute only)
   - Quality: Good
   - Use when: Cost-sensitive, privacy-first

**Decision: Support multiple providers, default to OpenAI**

### Decision 3: Memory Backend

**For conversational memory:**
- **Redis** (default): Fast, ephemeral conversations
- **PostgreSQL**: Persistent, queryable history
- **SQLite**: Embedded, development

**For semantic memory:**
- Vector stores (Qdrant, Pinecone, Chroma)

**Decision: Redis + Qdrant as default stack**

### Decision 4: Chunking Strategy

**Default strategy by content type:**
- **Markdown/Docs**: Markdown-aware chunking
- **Code**: Semantic chunking with syntax awareness
- **Plain text**: Recursive chunking
- **General**: Fixed-size with overlap

**Decision: Auto-detect content type, apply appropriate strategy**

## API Design

### Rust API

```rust
// High-level API for agents
let memory = MemoryManager::new()
    .with_backend(RedisBackend::new("redis://localhost")?)
    .with_vector_store(QdrantStore::new("http://qdrant:6333")?)
    .with_embedding(OpenAIEmbedding::new(api_key)?)
    .build()?;

// Store conversation
memory.conversational()
    .add_message("conv-1", ChatMessage {
        role: MessageRole::User,
        content: "Hello".to_string(),
        ..Default::default()
    })
    .await?;

// RAG retrieval
let rag = RAGPipeline::new(vector_store, embedding_provider, config);
let context = rag.retrieve("How to debug pods?", None).await?;

// Context management
let context_manager = ContextManager::new(memory)
    .with_rag(rag)
    .add_static_context("policies", policies_text);

let agent_context = context_manager
    .build_context("conv-1", "user query", config)
    .await?;
```

### HTTP API (for external integrations)

```
POST /api/v1/memory/{name}/conversations/{id}/messages
GET  /api/v1/memory/{name}/conversations/{id}/messages
DELETE /api/v1/memory/{name}/conversations/{id}

POST /api/v1/memory/{name}/search
{
  "query": "How to debug pods?",
  "top_k": 5,
  "filters": {
    "metadata.type": "runbook"
  }
}

POST /api/v1/knowledgebases/{name}/sync
GET  /api/v1/knowledgebases/{name}/status
GET  /api/v1/knowledgebases/{name}/documents

POST /api/v1/agents/{name}/query
{
  "message": "How do I debug this?",
  "conversation_id": "incident-123",
  "context": {
    "include_rag": true,
    "rag_top_k": 5
  }
}
```

## Testing Strategy

### Unit Tests
- Each backend implementation
- Chunking strategies
- Embedding providers
- Vector store operations

### Integration Tests
- End-to-end RAG pipeline
- Knowledge base ingestion
- Agent with memory
- Fleet shared memory

### Performance Tests
- Vector search latency
- Embedding throughput
- Memory backend performance
- RAG retrieval performance

### Load Tests
- 1000 concurrent conversations
- 10,000 vector searches/sec
- Large document ingestion

## Security Considerations

1. **Secret Management:**
   - Use Kubernetes secrets for API keys
   - Encrypt secrets at rest
   - Rotate credentials regularly

2. **Access Control:**
   - RBAC for memory access
   - Namespace isolation
   - Service account permissions

3. **Data Privacy:**
   - Encryption at rest for sensitive data
   - TLS for all network communication
   - PII detection and masking

4. **Vector Store Security:**
   - API key authentication
   - Network policies
   - Audit logging

## Performance Optimization

### Memory Backend
- Connection pooling (Redis, PostgreSQL)
- Batch operations for bulk inserts
- Compression for large messages
- TTL-based expiration

### Vector Store
- HNSW indexing for fast search
- Quantization for reduced memory
- Caching for frequent queries
- Batch embedding generation

### RAG Pipeline
- Query caching
- Result caching with TTL
- Parallel retrieval
- Lazy context building

## Migration Path

### From No Memory (Current)
1. Deploy Memory CRD and controller
2. Create Memory resources for existing agents
3. Update Agent manifests to reference Memory
4. Enable conversational memory first
5. Add RAG gradually

### From Custom Memory Solutions
1. Create migration scripts
2. Export existing conversations
3. Import to new Memory backend
4. Re-index documents in vector store
5. Validate and switch

## Operations

### Backup & Recovery
```bash
# Backup Redis conversations
redis-cli --rdb /backup/redis-dump.rdb

# Backup vector store
qdrant-backup --collection k8s-runbooks --output /backup/

# Restore
redis-cli --rdb /backup/redis-dump.rdb
qdrant-restore --collection k8s-runbooks --input /backup/
```

### Scaling
```bash
# Scale Redis cluster
kubectl scale statefulset redis --replicas=3

# Scale Qdrant
kubectl scale deployment qdrant --replicas=3

# Scale knowledge base workers
kubectl scale deployment kb-ingestion-worker --replicas=5
```

### Monitoring
```bash
# Check memory usage
kubectl top pods -l app=redis
kubectl top pods -l app=qdrant

# View metrics
kubectl port-forward svc/prometheus 9090:9090
# Open http://localhost:9090

# Check logs
kubectl logs -f deployment/memory-controller
kubectl logs -f deployment/knowledge-controller
```

## Cost Estimation

### Embeddings (OpenAI text-embedding-3-small)
- Cost: $0.02 per 1M tokens
- Average doc: 1000 tokens
- 10,000 docs: $0.20
- Monthly updates (1000 docs/day): $6/month

### Vector Storage (Qdrant self-hosted)
- 10,000 documents x 1536 dims x 4 bytes = ~60 MB
- 1M documents: ~6 GB
- Storage cost: ~$0.10/GB/month = $0.60/month

### Redis (Conversational Memory)
- 1000 active conversations x 50 messages x 1 KB = ~50 MB
- Memory cost: ~$0.02/MB/month = $1/month

### Total Monthly Cost (10K docs, 1K conversations)
- Embeddings: $6
- Vector storage: $0.60
- Redis: $1
- **Total: ~$8/month**

### Enterprise Scale (1M docs, 100K conversations)
- Embeddings: $600
- Vector storage: $60
- Redis: $100
- **Total: ~$760/month**

## Future Enhancements

### Phase 11+: Advanced Features

1. **Multi-modal Memory:**
   - Image embeddings (CLIP)
   - Audio transcription + embedding
   - Video frame analysis

2. **Graph Memory:**
   - Knowledge graph integration
   - Entity relationships
   - Graph traversal for context

3. **Active Learning:**
   - Feedback loop for relevance
   - Model fine-tuning from usage
   - Adaptive retrieval

4. **Federated RAG:**
   - Cross-cluster knowledge sharing
   - Privacy-preserving retrieval
   - Distributed vector search

5. **Agentic RAG:**
   - Agents decide when to retrieve
   - Multi-hop reasoning
   - Tool-augmented retrieval

6. **Memory Compression:**
   - LLM-based summarization
   - Hierarchical memory structure
   - Long-term vs short-term memory
