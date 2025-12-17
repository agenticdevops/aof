# AOF Implementation Plan v0.2

**Date**: December 2024
**Goal**: Bridge the gap between documented features and implementation

## Executive Summary

After evaluating documentation vs. implementation, we identified the following gaps:

| Feature | Documentation | Implementation | Status |
|---------|---------------|----------------|--------|
| Agent | Complete | Complete | Ready |
| AgentFleet | Complete | Core types + basic runtime | Needs CLI |
| AgentFlow | Complete | Core types + executor skeleton | Needs runtime |
| CLI (run/get/apply) | Complete | Complete | Ready |
| CLI (fleet/flow) | Documented | Not implemented | Needs work |
| MCP Support | Complete | Complete | Ready |
| Triggers | Complete | Complete | Ready |
| Memory | Complete | Complete | Ready |

## Current Architecture

```
crates/
├── aof-core/           # Core types (Agent, Fleet, Workflow, Error, Tools)
│   ├── agent.rs        # ✅ Complete
│   ├── fleet.rs        # ✅ Complete types
│   ├── workflow.rs     # ✅ Complete types
│   ├── mcp.rs          # ✅ Complete
│   ├── memory.rs       # ✅ Complete
│   ├── model.rs        # ✅ Complete
│   └── tool.rs         # ✅ Complete
├── aof-llm/            # LLM providers (OpenAI, Anthropic, Google, Ollama)
│   └── ✅ Complete
├── aof-mcp/            # MCP client
│   └── ✅ Complete
├── aof-memory/         # Memory backends
│   └── ✅ Complete
├── aof-runtime/        # Execution engine
│   ├── executor/       # ✅ AgentExecutor, WorkflowExecutor
│   ├── fleet/          # ✅ FleetCoordinator (needs CLI)
│   └── orchestrator/   # ✅ RuntimeOrchestrator
├── aof-tools/          # Built-in tools
│   └── ✅ Complete
├── aof-triggers/       # Event triggers
│   └── ✅ Complete
└── aofctl/             # CLI
    ├── ✅ run, get, apply, delete, describe, logs, exec, serve, version
    └── ❌ fleet, flow, config, completion
```

---

## Phase 1: Fleet & Flow CLI Commands (Week 1-2)

**Goal**: Connect existing runtime implementations to CLI

### 1.1 Fleet Commands

The `FleetCoordinator` in `aof-runtime/src/fleet/mod.rs` already implements:
- Fleet loading from YAML
- Agent instance management
- Coordination modes (hierarchical, peer, swarm, pipeline)
- Task distribution strategies

**Add to aofctl:**

```bash
# Fleet management
aofctl fleet apply -f fleet.yaml         # Load fleet config
aofctl fleet get [name]                  # List/get fleets
aofctl fleet describe <name>             # Show fleet details
aofctl fleet delete <name>               # Remove fleet
aofctl fleet status <name>               # Show runtime status

# Fleet execution
aofctl fleet run <name> -i "query"       # Execute task on fleet
aofctl fleet scale <name> --replicas 5   # Scale agent replicas
aofctl fleet logs <name>                 # View fleet logs
```

**Implementation tasks:**

- [ ] Create `crates/aofctl/src/commands/fleet.rs`
- [ ] Add `Fleet` subcommand to CLI parser
- [ ] Integrate `FleetCoordinator::from_file()`
- [ ] Implement `fleet run` with task submission
- [ ] Implement `fleet status` with state display
- [ ] Add fleet resource type to `api-resources`

### 1.2 Flow Commands

The `WorkflowExecutor` in `aof-runtime/src/executor/workflow_executor.rs` already implements:
- Workflow loading from YAML
- Step execution with routing
- State management
- Event streaming

**Add to aofctl:**

```bash
# Flow management
aofctl flow apply -f flow.yaml           # Load flow config
aofctl flow get [name]                   # List/get flows
aofctl flow describe <name>              # Show flow details
aofctl flow delete <name>                # Remove flow

# Flow execution
aofctl flow run <name> [--input json]    # Execute flow
aofctl flow status <run-id>              # Get execution status
aofctl flow logs <run-id>                # View execution logs
aofctl flow pause <run-id>               # Pause execution
aofctl flow resume <run-id>              # Resume execution
aofctl flow cancel <run-id>              # Cancel execution

# Flow visualization
aofctl flow visualize <name>             # ASCII/DOT graph output
```

**Implementation tasks:**

- [ ] Create `crates/aofctl/src/commands/flow.rs`
- [ ] Add `Flow` subcommand to CLI parser
- [ ] Integrate `WorkflowExecutor::from_file()`
- [ ] Implement `flow run` with execution
- [ ] Implement `flow status` with state display
- [ ] Add approval prompt handling for interactive mode
- [ ] Add flow resource type to `api-resources`

### 1.3 Config Commands

```bash
aofctl config view                       # Display current config
aofctl config set-context <name>         # Set active context
aofctl config get-contexts               # List contexts
aofctl config set <key> <value>          # Set config value
```

**Implementation tasks:**

- [ ] Create `crates/aofctl/src/commands/config.rs`
- [ ] Define config file format (`~/.aof/config.yaml`)
- [ ] Implement context management
- [ ] Implement config get/set

### 1.4 Shell Completion

```bash
aofctl completion bash > /etc/bash_completion.d/aofctl
aofctl completion zsh > ~/.zsh/completion/_aofctl
aofctl completion fish > ~/.config/fish/completions/aofctl.fish
```

**Implementation tasks:**

- [ ] Add clap `derive` completion generation
- [ ] Create `crates/aofctl/src/commands/completion.rs`

---

## Phase 2: Enhanced Fleet Runtime (Week 3-4)

**Goal**: Production-ready multi-agent coordination

### 2.1 Shared Memory Integration

Currently fleets have shared memory config but no implementation.

**Tasks:**

- [ ] Implement `SharedMemoryManager` in aof-memory
- [ ] Add Redis backend for distributed fleets
- [ ] Add in-memory shared state for single-process fleets
- [ ] Wire shared memory into `FleetCoordinator`

### 2.2 Enhanced Consensus

The consensus implementation is basic (takes first result).

**Tasks:**

- [ ] Implement proper majority voting
- [ ] Add weighted voting based on agent roles
- [ ] Implement unanimous consensus mode
- [ ] Add timeout handling for consensus
- [ ] Track and report consensus metrics

### 2.3 Fleet Monitoring

**Tasks:**

- [ ] Add Prometheus metrics endpoint
- [ ] Implement real-time fleet dashboard data
- [ ] Add health checks for agent instances
- [ ] Implement auto-restart on agent failure

### 2.4 Dynamic Scaling

**Tasks:**

- [ ] Implement agent replica scaling
- [ ] Add auto-scaling based on queue depth
- [ ] Add cooldown and rate limiting
- [ ] Implement graceful scale-down

---

## Phase 3: Enhanced Flow Runtime (Week 5-6)

**Goal**: Production-ready workflow orchestration

### 3.1 Approval System

The executor has approval events but no UI integration.

**Tasks:**

- [ ] Implement Slack approval integration
- [ ] Add email approval notifications
- [ ] Build CLI interactive approval flow
- [ ] Add approval audit logging
- [ ] Implement approval delegation

### 3.2 Checkpointing & Recovery

Types exist but implementation is incomplete.

**Tasks:**

- [ ] Implement file-based checkpoint storage
- [ ] Add Redis checkpoint backend
- [ ] Implement workflow resume from checkpoint
- [ ] Add checkpoint history management
- [ ] Test crash recovery scenarios

### 3.3 Conditional Routing Engine

Condition parsing needs real implementation.

**Tasks:**

- [ ] Implement expression evaluator for conditions
- [ ] Add state variable interpolation
- [ ] Support comparison operators
- [ ] Add function calls in conditions
- [ ] Test complex routing scenarios

### 3.4 Parallel Execution

Parallel branches exist but join logic needs work.

**Tasks:**

- [ ] Implement proper fork/join semantics
- [ ] Add timeout handling for parallel branches
- [ ] Implement partial failure handling
- [ ] Add result aggregation strategies

---

## Phase 4: Production Hardening (Week 7-8)

**Goal**: Enterprise-ready stability

### 4.1 Error Handling

- [ ] Standardize error codes across crates
- [ ] Add error recovery suggestions
- [ ] Implement circuit breaker for LLM calls
- [ ] Add rate limiting per provider

### 4.2 Observability

- [ ] Add OpenTelemetry tracing
- [ ] Implement structured logging
- [ ] Add metrics for all operations
- [ ] Create Grafana dashboard templates

### 4.3 Security

- [ ] Add RBAC for fleet/flow operations
- [ ] Implement secret management
- [ ] Add audit logging
- [ ] Implement API key rotation

### 4.4 Testing

- [ ] Add integration tests for fleet commands
- [ ] Add integration tests for flow commands
- [ ] Add end-to-end workflow tests
- [ ] Add performance benchmarks

---

## Implementation Priority

Based on user impact and dependency order:

### High Priority (Week 1-2)
1. **Fleet CLI commands** - Exposes existing FleetCoordinator
2. **Flow CLI commands** - Exposes existing WorkflowExecutor
3. **Shell completion** - Developer productivity

### Medium Priority (Week 3-4)
4. **Config commands** - Multi-environment support
5. **Shared memory** - Fleet coordination
6. **Enhanced consensus** - Multi-agent reliability

### Lower Priority (Week 5-6)
7. **Approval system** - Human-in-the-loop
8. **Checkpointing** - Long-running workflows
9. **Conditional routing** - Complex logic

### Future (Week 7+)
10. **Production hardening** - Enterprise features

---

## File Structure for New Code

```
crates/aofctl/src/commands/
├── mod.rs              # Add fleet, flow, config, completion
├── fleet.rs            # NEW: Fleet commands
├── flow.rs             # NEW: Flow commands
├── config.rs           # NEW: Config commands
└── completion.rs       # NEW: Shell completion

crates/aof-memory/src/
├── shared.rs           # NEW: SharedMemoryManager
└── backends/
    ├── redis.rs        # NEW: Redis shared memory
    └── in_memory.rs    # Enhance for sharing

crates/aof-runtime/src/
├── fleet/
│   ├── consensus.rs    # NEW: Enhanced consensus
│   └── scaling.rs      # NEW: Auto-scaling
└── executor/
    ├── checkpoint.rs   # NEW: Checkpoint impl
    ├── condition.rs    # NEW: Condition evaluator
    └── approval.rs     # NEW: Approval handler
```

---

## Success Metrics

1. **CLI Coverage**: All documented commands implemented
2. **Test Coverage**: >80% for new code
3. **Documentation**: All features have examples
4. **Performance**: Fleet executes 10 agents <5s startup
5. **Reliability**: Workflow resume works after crash

---

## Next Steps

1. Start with Phase 1.1 (Fleet CLI)
2. Create feature branch `feat/fleet-cli`
3. Implement commands incrementally
4. Add tests for each command
5. Update documentation as we go

---

## Phase 5: Memory & RAG System (Week 9-12)

**Goal**: Advanced memory, context, and RAG system per architecture docs.

> Reference: `docs/architecture/` contains comprehensive design for Memory CRD, KnowledgeBase CRD, and RAG pipeline.

### 5.1 Memory Backend Expansion (Week 9)

Currently `aof-memory` has basic file-based memory. Architecture calls for:

**Tasks:**
- [ ] Implement Redis backend (`MemoryBackend` trait)
- [ ] Implement PostgreSQL backend
- [ ] Add S3 archival backend
- [ ] Implement conversational memory with TTL
- [ ] Add message summarization for long conversations

### 5.2 Vector Store Integration (Week 10)

**Tasks:**
- [ ] Implement `VectorStore` trait from architecture
- [ ] Add Qdrant integration (primary recommended store)
- [ ] Add Chroma integration (development)
- [ ] Implement embedding providers (OpenAI, Cohere, local)
- [ ] Add vector search benchmarks

### 5.3 RAG Pipeline (Week 11)

**Tasks:**
- [ ] Implement `RAGPipeline` from `rust-implementation.md`
- [ ] Add chunking strategies (fixed, semantic, markdown)
- [ ] Implement hybrid search (semantic + keyword)
- [ ] Add reranking support (Cohere reranker)
- [ ] Build context manager with token optimization

### 5.4 KnowledgeBase Integration (Week 12)

**Tasks:**
- [ ] Add GitHub source connector
- [ ] Add Confluence source connector
- [ ] Implement ingestion pipeline
- [ ] Add sync scheduling
- [ ] Implement status reporting

### Architecture Reference

See detailed designs in:
- `docs/architecture/memory-rag-system.md` - System overview
- `docs/architecture/architecture-summary.md` - Executive summary
- `docs/architecture/rust-implementation.md` - Rust traits & code
- `docs/architecture/implementation-guide.md` - 12-week roadmap

### Performance Targets (from architecture)

| Operation | Target |
|-----------|--------|
| Redis read/write | <1ms |
| Vector search (top-5) | <50ms |
| RAG with reranking | <250ms |
| Full context building | <400ms |

### Cost Estimates (from architecture)

| Scale | Monthly Cost |
|-------|--------------|
| 10K docs, 1K conversations | ~$8/month |
| 1M docs, 100K conversations | ~$760/month |

---

## Summary: Full Implementation Roadmap

| Phase | Focus | Timeline |
|-------|-------|----------|
| **Phase 1** | Fleet & Flow CLI | Week 1-2 |
| **Phase 2** | Enhanced Fleet Runtime | Week 3-4 |
| **Phase 3** | Enhanced Flow Runtime | Week 5-6 |
| **Phase 4** | Production Hardening | Week 7-8 |
| **Phase 5** | Memory & RAG System | Week 9-12 |

### Quick Wins (Can Do Now)

1. **Fleet CLI** - Wire existing `FleetCoordinator` to CLI
2. **Flow CLI** - Wire existing `WorkflowExecutor` to CLI
3. **Shell completion** - 1-2 hours with clap derive

### Medium Effort

4. **Config commands** - New code, ~1 day
5. **Shared memory** - Extends existing memory crate
6. **Enhanced consensus** - Algorithmic improvements

### Larger Projects

7. **Memory/RAG system** - Full architecture from docs/architecture/
8. **Approval system** - Needs Slack/email integrations
9. **Observability** - OpenTelemetry integration

---

## Related Documentation

- [Agent Spec Reference](../reference/agent-spec.md)
- [AgentFlow Spec Reference](../reference/agentflow-spec.md)
- [CLI Reference](../reference/aofctl.md)
- [Core Concepts](../concepts.md)
- [Memory/RAG Architecture](../architecture/README.md)
- [Rust Implementation Guide](../architecture/rust-implementation.md)
