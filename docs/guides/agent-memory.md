# Agent Memory

## Overview

OpenAgentiX agents can optionally maintain a vector memory of past run results. Before each run, the agent recalls the most similar past contexts and prepends them to the system prompt. After each run, the final answer is stored for future recall.

This enables agents to improve over time — a database specialist that has analyzed PostgreSQL issues dozens of times will automatically retrieve relevant past analyses when similar queries arrive.

## How It Works

1. **Before the run**: the current input is converted to an embedding. The top_k most similar past entries are retrieved via cosine similarity and injected into the system prompt as a `## Relevant context from past runs` block.
2. **During the run**: the ReAct loop proceeds as normal, with the recalled context available in the system prompt.
3. **After the run**: the final answer is embedded and stored in the vector database with metadata (`trigger_source`, `run_id`).

## Enabling Memory in Agent YAML

Add a `vector_memory` block to the agent's `spec`:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-specialist
spec:
  model: anthropic/claude-3-5-haiku
  system_prompt: |
    You are a PostgreSQL specialist.

  vector_memory:
    enabled: true
    backend: sqlite
    db_path: ./memory/dba-specialist.db
    top_k: 5
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable vector memory for this agent |
| `backend` | string | `sqlite` | Storage backend (only `sqlite` is supported in v2.0) |
| `db_path` | string | `./memory/<agent-name>.db` | Path to the SQLite database file |
| `top_k` | integer | `5` | Number of similar past contexts to retrieve per run |
| `embed_model` | string | (agent model) | Optional embedding model override |

## What Gets Stored

After each run, the agent stores:
- **Text**: the final answer text
- **Embedding**: a 256-dim vector derived from the input text
- **Metadata**: `trigger_source` (how the run was triggered), `run_id`

## What Gets Recalled

Before each run, the agent retrieves the `top_k` entries from its own memory that are most similar to the current input. These are presented as:

```
## Relevant context from past runs

**Run run-abc123** (similarity: 0.94):
The replication lag on pg-primary was 2ms — well within normal bounds.
No tuning was required.

**Run run-def456** (similarity: 0.87):
pg-primary experienced a 45ms lag spike due to a long-running VACUUM.
Recommended: schedule VACUUM during off-peak hours.
```

## Agent Isolation

Each agent has its own isolated memory store. The `agent_id` field scopes all queries and stores — an agent named `dba-specialist` can never read entries written by `infra-specialist`, even if they share the same database file.

## Enabling the Research Phase

The research phase is an optional pre-loop step that asks the LLM to identify and summarise relevant facts before the main ReAct loop begins. Enable it with `research_phase`:

```yaml
spec:
  model: anthropic/claude-3-5-haiku
  system_prompt: |
    You are an operations specialist.

  research_phase:
    enabled: true
    max_iterations: 3
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable the research phase |
| `max_iterations` | integer | `3` | Maximum fact-gathering iterations |

## Research Phase Behavior

When `research_phase.enabled=true`, the runtime fires a single LLM call before the main loop:

1. The LLM is asked to identify 3-5 relevant facts it already knows about the query.
2. The LLM summarises them in a `## RESEARCH COMPLETE` block.
3. This block is injected into the system prompt as `## Research Context` before the main ReAct loop begins.

This is especially useful for agents that handle complex multi-step operational tasks where upfront context-gathering improves the quality of the plan.

## Combining Memory and Research Phase

Memory recall and the research phase compose naturally:

```
[Before run]
1. Vector recall: retrieve top_k similar past contexts
2. Research phase: pre-loop fact summary

[System prompt assembly]
## Relevant context from past runs
...recalled entries...

## Research Context
...pre-gathered facts...

[Original system prompt]
...
[ReAct instructions]
...

[After run]
3. Vector store: persist final answer
```

## Performance Notes

- **Memory recall** adds one SQLite query per run — typically < 1ms.
- **Memory store** adds one SQLite write per run — typically < 1ms.
- **Research phase** adds one LLM round-trip per run — latency depends on the model.
- Disable `vector_memory` and `research_phase` for latency-sensitive agents where speed matters more than context continuity.

## CLI and REST API

```bash
# List memory entries for an agent (Plan 16-05)
agentix memory list <agent-name>

# Clear all memory entries for an agent
agentix memory clear <agent-name>
```

REST:

```
GET    /api/v1/agents/:name/memory    — list memory entries
DELETE /api/v1/agents/:name/memory    — clear all memory entries
```

## Implementation Notes

- Embedding generation in v2.0 uses a deterministic hash-based fallback (no external embedding API required). Semantic similarity is approximate. A real embedding model integration is planned for v2.1.
- The `embed_model` field is reserved for future use when a dedicated embedding API is wired in.
- Memory is stored per-agent in a local SQLite file. For distributed deployments, point all instances at a shared network-accessible SQLite file or use a remote vector store backend (planned for v3.0).
