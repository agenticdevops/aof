# Vector Memory

## Overview

Vector memory allows agents to persist context from past runs and recall semantically similar context before generating a new response. This reduces hallucinations (the agent remembers what it found before) and improves consistency across runs on similar tasks.

The workflow is:
1. At the end of a run, the agent's final answer is embedded and stored in the vector backend.
2. At the start of the next run, the agent embeds its current input and retrieves the top-K most similar past contexts.
3. Retrieved contexts are prepended to the system prompt as `## Relevant context from past runs`.
4. The ReAct loop runs with this enriched context.

## How It Works

```
Run N:
  input → embed → search_similar → [past contexts] → prepend to system prompt
  ReAct loop runs with enriched prompt
  final answer → embed → store_embedding

Run N+1:
  input → embed → search_similar → [Run N context, ...] → prepend
  ...
```

## VectorMemoryBackend Trait

Defined in `agentix-core::vector_memory`:

```rust
#[async_trait]
pub trait VectorMemoryBackend: Send + Sync {
    async fn store_embedding(
        &self,
        agent_id: &str,
        run_id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> AgentixResult<String>;

    async fn search_similar(
        &self,
        agent_id: &str,
        query_embedding: &[f32],
        top_k: usize,
    ) -> AgentixResult<Vec<MemoryMatch>>;

    async fn clear_agent_memory(&self, agent_id: &str) -> AgentixResult<()>;

    async fn list_entries(&self, agent_id: &str) -> AgentixResult<Vec<VectorEntry>>;
}
```

| Method | Description |
|--------|-------------|
| `store_embedding` | Persist text + embedding for a run. Returns the assigned entry ID. |
| `search_similar` | Return top-K entries ordered by cosine similarity. |
| `clear_agent_memory` | Delete all entries for an agent. |
| `list_entries` | Return all entries for an agent (for CLI/REST display). |

## SQLite Backend

The default backend stores embeddings in a SQLite database file.

- Embeddings are stored as BLOB (little-endian `f32` array).
- Cosine similarity search loads all candidate rows for the agent and ranks them in Rust — no SQL vector extension required.
- The database is created automatically on first use.
- The `bundled` feature compiles SQLite into the binary — no external dependency.

File location: configured via `spec.memory.db_path` in the agent YAML. Default: `./memory/<agent-name>.db`.

## Agent Isolation

All queries are scoped by `agent_id`. An agent can only read its own memories:

```
search_similar("agent-a", ...)  → returns only agent-a entries
search_similar("agent-b", ...)  → returns only agent-b entries
```

This satisfies MEM-04. Sharing memory between agents is not supported in v2.0 and requires explicit configuration in v2.1.

## Configuring Memory in Agent YAML

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-specialist
spec:
  model: anthropic/claude-3-5-haiku
  memory:
    enabled: true
    backend: sqlite
    db_path: ./memory/dba-specialist.db
    top_k: 5
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable/disable vector memory |
| `backend` | `sqlite` | Backend type (only `sqlite` in v2.0) |
| `db_path` | `./memory/<name>.db` | Path to the SQLite database file |
| `top_k` | `5` | Number of similar past contexts to retrieve per run |
| `embed_model` | agent's model | Override the embedding model |

## Embedding Model

Embeddings are generated using the LLM provider configured for the agent. If the provider does not support a dedicated embedding endpoint, a deterministic hash-based embedding is used as a fallback (normalized 256-dim unit vector derived from the text's SHA-256 hash).

Full embedding model support (e.g., `text-embedding-3-small`) is a v2.1 enhancement.

## Cosine Similarity

The similarity function is:

```rust
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    dot / (norm_a * norm_b)
}
```

Scores range from -1.0 to 1.0 (higher = more similar). Results are sorted descending before truncation to `top_k`.

## Pluggable Backends

Implement `VectorMemoryBackend` to use a different store:

```rust
struct PgVectorBackend { /* ... */ }

#[async_trait]
impl VectorMemoryBackend for PgVectorBackend {
    // implement all 4 methods
}
```

Planned backends for v2.1: `pgvector` (PostgreSQL), `qdrant`, `weaviate`.

## CLI Commands

```bash
# List all memory entries for an agent
agentix memory list dba-specialist

# Clear all memory entries for an agent
agentix memory clear dba-specialist

# With confirmation skip
agentix memory clear dba-specialist --yes
```

## REST API

```
GET    /api/v1/agents/:name/memory   — list all entries
DELETE /api/v1/agents/:name/memory   — clear all entries
```
