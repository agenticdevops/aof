# RuVector Integration for AOF Agentic RAG

## Overview

This document describes the integration of RuVector as the default vector database for AOF's Agentic RAG system. RuVector provides high-performance vector search with self-learning capabilities through its SONA (Self-Optimizing Neural Architecture) runtime adaptation system.

## Key Features

### 1. **HNSW Vector Search**
- **Latency**: ~61µs per query
- **Throughput**: 16,400 QPS
- **Accuracy**: 99.5% recall@10

### 2. **Self-Learning GNN**
- Automatic index optimization based on query patterns
- SONA runtime adaptation without retraining
- Continuous learning from user feedback
- Query pattern recognition and expansion

### 3. **Graph-Enhanced Retrieval**
- Neo4j-compatible Cypher queries
- Automatic relationship creation based on similarity
- Multi-hop traversal for related documents
- Graph centrality scoring

### 4. **Adaptive Compression**
- Hot data: f32 (full precision)
- Warm data: f16 (half precision)
- Cold data: PQ8 (product quantization 8-bit)
- Archive: PQ4 (product quantization 4-bit)
- **Memory Savings**: Up to 32x compression

### 5. **Distributed Architecture**
- Raft consensus for consistency
- Multi-master replication
- Automatic sharding
- Self-healing clusters

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     AOF Agentic System                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Agent 1    │    │   Agent 2    │    │   Agent N    │  │
│  │ (K8s Ops)    │    │ (Security)   │    │ (Monitoring) │  │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘  │
│         │                   │                   │           │
│         └───────────────────┴───────────────────┘           │
│                             │                                │
│                   ┌─────────▼─────────┐                     │
│                   │ Self-Learning RAG  │                     │
│                   │    Pipeline        │                     │
│                   └─────────┬─────────┘                     │
│                             │                                │
│         ┌───────────────────┼───────────────────┐           │
│         │                   │                   │           │
│    ┌────▼────┐         ┌───▼───┐          ┌───▼────┐      │
│    │ Query   │         │ Graph │          │ Learned│      │
│    │Expansion│         │Search │          │Ranker  │      │
│    └────┬────┘         └───┬───┘          └───┬────┘      │
│         │                  │                  │            │
│         └──────────────────┴──────────────────┘            │
│                            │                                │
├────────────────────────────┼────────────────────────────────┤
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │ RuVector Backend│                       │
│                   ├─────────────────┤                       │
│                   │ HNSW Index      │                       │
│                   │ Graph Database  │                       │
│                   │ SONA Optimizer  │                       │
│                   │ Feedback Loop   │                       │
│                   └────────┬────────┘                       │
│                            │                                │
│         ┌──────────────────┼──────────────────┐            │
│         │                  │                  │            │
│    ┌────▼────┐        ┌───▼───┐         ┌───▼────┐       │
│    │Vector   │        │Graph  │         │Adaptive│       │
│    │Store    │        │Store  │         │Compress│       │
│    └─────────┘        └───────┘         └────────┘       │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

## Configuration

### Memory CRD

Deploy RuVector as a memory backend using Kubernetes CRD:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Memory
metadata:
  name: ruvector-rag-memory
  namespace: agentic-ops
spec:
  backend:
    type: RuVector
    ruvector:
      mode: Cluster  # Embedded | Cluster | Distributed

      # Self-Learning Configuration
      gnn:
        enabled: true
        model: SONA-Adaptive
        learningRate: 0.001
        adaptationInterval: 100

      # Graph Configuration
      graph:
        enabled: true
        relationships:
          - type: SIMILAR_TO
            threshold: 0.85
            bidirectional: true
          - type: DEPENDS_ON
            threshold: 0.75
            bidirectional: false

      # Compression Strategy
      compression:
        strategy: Adaptive
        tiers:
          hot:
            encoding: f32
            maxAge: 7d
          warm:
            encoding: f16
            maxAge: 30d
          cold:
            encoding: PQ8
            maxAge: 90d
```

See [config/memory-ruvector.yaml](../config/memory-ruvector.yaml) for complete configuration.

## Usage

### 1. Initialize Backend

```rust
use aof_memory::backends::ruvector::{RuVectorBackend, RuVectorConfig};

let config = RuVectorConfig {
    mode: DeploymentMode::Cluster,
    vector: VectorConfig {
        dimensions: 1536,
        metric: Metric::Cosine,
        normalize: true,
    },
    gnn: GNNConfig {
        enabled: true,
        model: "SONA-Adaptive".to_string(),
        learning_rate: 0.001,
        adaptation_interval: 100,
        feedback_weight: 0.7,
        // ...
    },
    // ...
};

let backend = RuVectorBackend::new(config).await?;
```

### 2. Create Self-Learning RAG

```rust
use aof_memory::rag::self_learning::{SelfLearningRAG, RAGConfig};

let rag_config = RAGConfig {
    retrieval: RetrievalConfig {
        strategy: RetrievalStrategy::Hybrid,
        top_k: 10,
        reranking_enabled: true,
        // ...
    },
    learning: LearningConfig {
        enabled: true,
        feedback: FeedbackConfig {
            collect_user_feedback: true,
            implicit_feedback: true,
            // ...
        },
        // ...
    },
    // ...
};

let rag = SelfLearningRAG::new(Arc::new(backend), rag_config).await?;
```

### 3. Retrieve with Learning

```rust
// Retrieve documents
let results = rag.retrieve(
    "How to fix CrashLoopBackOff in Kubernetes?",
    query_vector,
    None,
).await?;

// Use results
for result in results {
    println!("Found: {} (score: {})", result.id, result.score);
}

// Record feedback for learning
rag.record_feedback(
    &query_id,
    &result_id,
    UserFeedback {
        relevant: true,
        rating: Some(5),
    },
).await?;
```

### 4. Graph Queries with Cypher

```rust
// Find related documents through graph relationships
let cypher_query = r#"
    MATCH (d:Document {id: $doc_id})
    MATCH (d)-[r:SIMILAR_TO|DEPENDS_ON*1..3]-(related:Document)
    WHERE r.score >= 0.7
    RETURN related
    ORDER BY r.score DESC
    LIMIT 10
"#;

let params = serde_json::json!({ "doc_id": "k8s-crashloop" });
let related_docs = backend.cypher_search(cypher_query, params).await?;
```

### 5. Hybrid Search (Vector + Graph)

```rust
// Combine vector similarity with graph relationships
let results = backend.hybrid_search(
    query_vector,
    Some("MATCH (d)-[:RELATES_TO]-(r) RETURN r"),
    0.7,  // vector_weight
    0.3,  // graph_weight
).await?;
```

## Self-Learning Features

### 1. Query Pattern Recognition

RuVector's SONA system automatically recognizes query patterns:

- **Successful queries**: Patterns leading to relevant results
- **Failed queries**: Patterns requiring expansion or reformulation
- **Temporal patterns**: Time-of-day, day-of-week patterns
- **User patterns**: Per-user retrieval preferences

### 2. Automatic Index Optimization

SONA continuously adapts the index:

```rust
// Adaptation happens automatically based on query patterns
// Manual trigger available:
let adaptations = optimizer.adapt(&patterns).await?;
for adaptation in adaptations {
    db.apply_adaptation(adaptation).await?;
}
```

### 3. Feedback Loop

Both explicit and implicit feedback:

```rust
// Explicit feedback
rag.record_feedback(&query_id, &result_id, UserFeedback {
    relevant: true,
    rating: Some(5),
}).await?;

// Implicit feedback from agent actions
rag.record_implicit_feedback(&query_id, &result_id, AgentAction::Used).await?;
```

### 4. Query Expansion

Automatically expand queries based on learned patterns:

```rust
let expanded = query_expander.expand(
    "pod crashing",
    &query_vector,
).await?;
// Returns: ["pod crashing", "crashloopbackoff", "container restart"]
```

## Graph-Enhanced RAG

### Relationship Types

```rust
// Automatic relationship creation
backend.insert_with_relationships(
    "doc-123",
    vector,
    metadata,
).await?;

// Creates relationships:
// - SIMILAR_TO: Based on vector similarity
// - DEPENDS_ON: From explicit dependencies
// - PART_OF: Hierarchical relationships
// - RELATES_TO: Semantic relationships
```

### Multi-Hop Traversal

```rust
let config = MultiHopConfig {
    enabled: true,
    max_hops: 3,
    min_relevance_score: 0.7,
};

// Automatically expands results through graph
let results = rag.retrieve(query, vector, None).await?;
// Includes: direct matches + 1-hop neighbors + 2-hop neighbors
```

### Cypher Query Examples

**Find similar issues:**
```cypher
MATCH (issue:Issue {id: $issue_id})
MATCH (issue)-[:SIMILAR_TO]-(similar:Issue)
WHERE similar.resolved = true
RETURN similar
ORDER BY similar.success_rate DESC
```

**Find dependency chain:**
```cypher
MATCH path = (service:Service {id: $service_id})-[:DEPENDS_ON*1..5]->(dep)
RETURN path, dep
ORDER BY length(path)
```

**Find root cause:**
```cypher
MATCH (symptom:Issue {id: $symptom_id})
MATCH path = (symptom)<-[:CAUSES*1..3]-(root:Issue)
WHERE NOT (root)<-[:CAUSES]-()
RETURN root, path
ORDER BY length(path)
```

## Performance Optimization

### 1. Compression Strategy

```yaml
compression:
  strategy: Adaptive
  tiers:
    hot:      # Frequently accessed
      encoding: f32
      minAccessCount: 100
      maxAge: 7d
    warm:     # Moderately accessed
      encoding: f16
      minAccessCount: 10
      maxAge: 30d
    cold:     # Rarely accessed
      encoding: PQ8
      minAccessCount: 1
      maxAge: 90d
    archive:  # Archival
      encoding: PQ4
      maxAge: 365d
```

**Memory savings:**
- f32 → f16: 2x reduction
- f32 → PQ8: 4x reduction
- f32 → PQ4: 8x reduction

### 2. HNSW Tuning

```yaml
hnsw:
  m: 16              # Links per node (4-64)
  efConstruction: 200 # Build quality (100-500)
  efSearch: 100       # Search quality (50-200)
  maxLayers: 6        # Graph depth
```

**Trade-offs:**
- Higher `m`: Better recall, more memory
- Higher `efConstruction`: Better index, slower build
- Higher `efSearch`: Better recall, slower queries

### 3. Query Parallelism

```yaml
performance:
  queryParallelism: 8      # Concurrent queries
  batchInsertSize: 1000    # Batch operations
  flushInterval: 5s        # Disk flush
```

## Distributed Deployment

### Cluster Mode

```yaml
distributed:
  replication:
    factor: 3           # 3 replicas
    strategy: Raft      # Consensus protocol
    minSyncReplicas: 2  # Write confirmation

  sharding:
    enabled: true
    strategy: Consistent  # Hash distribution
    shardCount: 8         # Number of shards
```

### High Availability

```yaml
operations:
  highAvailability:
    enabled: true
    minReplicas: 2
    maxReplicas: 5

  autoscaling:
    enabled: true
    metrics:
      - type: QueryLatency
        target: 50ms
      - type: MemoryUsage
        target: 70%
```

## Monitoring

### Metrics

```rust
// Get performance metrics
let metrics = backend.get_metrics().await?;
println!("QPS: {}", metrics.qps);
println!("P50 latency: {:?}", metrics.query_latency_p50);
println!("P99 latency: {:?}", metrics.query_latency_p99);
println!("Memory usage: {} GB", metrics.index_memory_usage / 1_000_000_000);

// Get learning metrics
let learning = backend.get_learning_metrics().await?;
println!("Total queries: {}", learning.total_queries);
println!("Adaptation count: {}", learning.adaptation_count);
println!("Accuracy: {:.2}%", learning.accuracy * 100.0);
```

### Prometheus Metrics

```yaml
monitoring:
  enabled: true
  metricsPort: 9090
  exportInterval: 10s

  alerts:
    queryLatencyP99: 100ms
    indexMemoryUsage: 80%
    replicationLag: 1000ms
    errorRate: 1%
```

## Complete Example: K8s Troubleshooting Agent

See [examples/k8s_troubleshooting_agent.rs](../examples/k8s_troubleshooting_agent.rs) for a production-ready example that demonstrates:

1. **Knowledge Base Initialization**: Load common K8s issues
2. **Issue Diagnosis**: Vector + graph retrieval
3. **Solution Application**: Execute remediation steps
4. **Feedback Learning**: Learn from success/failure
5. **Graph Relationships**: Track issue-solution patterns

### Running the Example

```bash
# Build the example
cargo build --example k8s_troubleshooting_agent

# Run with tracing
RUST_LOG=info cargo run --example k8s_troubleshooting_agent

# Output:
# === Diagnosis Results ===
# Solution 1: Pod CrashLoopBackOff
# Confidence: 95.00%
# Success Rate: 87.50%
# Steps:
#   1. Check pod logs: kubectl logs <pod-name>
#   2. Inspect container exit code and reason
#   3. Verify resource limits (CPU/memory)
#   ...
```

## Migration from Other Vector Stores

### From Pinecone

```rust
// Old: Pinecone
let index = pinecone.index("my-index");
let results = index.query(vector, top_k).await?;

// New: RuVector
let results = backend.search(vector, top_k).await?;
```

### From Qdrant

```rust
// Old: Qdrant
let results = qdrant.search(
    SearchRequest {
        vector,
        limit: top_k,
        with_payload: true,
        ..Default::default()
    }
).await?;

// New: RuVector
let results = backend.search_with_learning(
    vector,
    SearchOptions {
        top_k,
        ..Default::default()
    },
).await?;
```

### From Weaviate

```rust
// Old: Weaviate (GraphQL)
let query = r#"
{
  Get {
    Document(nearVector: {vector: $vector}) {
      content
      _additional { certainty }
    }
  }
}
"#;

// New: RuVector (Cypher)
let query = r#"
MATCH (d:Document)
CALL db.idx.vector.queryNodes(d, $vector) YIELD node, score
RETURN node
"#;
```

## Best Practices

### 1. Chunking Strategy

```rust
// Semantic chunking for better context
ChunkingConfig {
    strategy: ChunkingStrategy::Semantic,
    chunk_size: 512,
    chunk_overlap: 50,
}
```

### 2. Hybrid Retrieval

```rust
// Balance vector and graph weights
HybridConfig {
    vector_weight: 0.7,  // Precision
    graph_weight: 0.3,   // Context
}
```

### 3. Feedback Collection

```rust
// Collect both explicit and implicit feedback
FeedbackConfig {
    collect_user_feedback: true,
    implicit_feedback: true,  // Learn from agent actions
    feedback_store: "memory://feedback",
}
```

### 4. Multi-Hop Traversal

```rust
// Limit hops to avoid noise
MultiHopConfig {
    enabled: true,
    max_hops: 2,  // 1-2 hops usually sufficient
    min_relevance_score: 0.7,
}
```

### 5. Compression

```rust
// Let adaptive compression optimize automatically
CompressionConfig {
    strategy: CompressionStrategy::Adaptive,
    // Tiers configured in YAML
}
```

## Troubleshooting

### High Query Latency

1. Check HNSW parameters:
   ```yaml
   hnsw:
     efSearch: 50  # Lower for faster queries
   ```

2. Enable query cache:
   ```yaml
   graph:
     cypher:
       enableQueryCache: true
       cacheTTL: 1h
   ```

3. Increase query parallelism:
   ```yaml
   performance:
     queryParallelism: 16
   ```

### High Memory Usage

1. Enable compression:
   ```yaml
   compression:
     strategy: Adaptive
   ```

2. Reduce cache sizes:
   ```yaml
   performance:
     memory:
       cacheSize: 2Gi
       indexCacheSize: 1Gi
   ```

3. Trigger compaction:
   ```yaml
   performance:
     compactionInterval: 30m
   ```

### Learning Not Improving

1. Check feedback volume:
   ```rust
   let metrics = backend.get_learning_metrics().await?;
   assert!(metrics.total_queries >= 50);  // Minimum for adaptation
   ```

2. Verify feedback quality:
   ```yaml
   gnn:
     patterns:
       minPatternsForAdaptation: 50
   ```

3. Adjust learning rate:
   ```yaml
   gnn:
     learningRate: 0.01  # Higher for faster adaptation
   ```

## Future Enhancements

1. **Multi-Modal Embeddings**: Support for images, code, structured data
2. **Federated Learning**: Learn across multiple clusters
3. **Real-Time Adaptation**: Sub-second index updates
4. **AutoML Integration**: Automatic hyperparameter tuning
5. **Explainable Retrieval**: Why was this document retrieved?

## References

- RuVector Documentation: https://github.com/ruvnet/ruvector
- SONA Paper: [Self-Optimizing Neural Architecture]
- AOF Documentation: [Agentic Ops Framework]

## License

Apache 2.0
