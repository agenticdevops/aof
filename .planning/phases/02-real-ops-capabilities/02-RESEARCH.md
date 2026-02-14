# Phase 2: Real Ops Capabilities — Research

**Date:** 2026-02-13
**Status:** Complete
**Key Findings:**
- Agent Skills format is standardized with industry adoption (Anthropic, Microsoft, OpenAI, GitHub)
- LLM-based triage uses confidence thresholds (50-70%) for auto-routing vs human escalation
- Redis TTL locks provide simple, self-healing distributed coordination for Rust
- Decision logs benefit from hybrid event sourcing + structured search (semantic + SQL-like)
- Docker sandbox isolation requires defense-in-depth: user namespaces, resource limits, seccomp

---

## Sections

1. [Incident Response Patterns](#1-incident-response-patterns)
2. [Skills Platform Design](#2-skills-platform-design)
3. [Decision Logging Systems](#3-decision-logging-systems)
4. [Resource Collision Prevention](#4-resource-collision-prevention)
5. [Sandbox Isolation](#5-sandbox-isolation)

---

## 1. Incident Response Patterns

### Current Practice

**How do similar systems handle incident triage and specialist delegation?**

Industry systems use multi-agent coordination with confidence-based routing:

- **PagerDuty/Opsgenie:** Rule-based escalation chains with time-based triggers
- **Triangle (Microsoft Research 2025):** Multi-LLM agent system for incident triage with specialist coordination
- **CORTEX:** Collaborative LLM agents for high-stakes alert triage with context pulling
- **Forethought Triage LLM:** Auto-classifies with 50% confidence threshold (below = human escalation)

**Common patterns:**
1. **Triage classifies first** — LLM analyzes alert, assigns severity (SEV1-SEV4), confidence score
2. **Confidence-driven routing** — High confidence (>70%) → auto-route to specialist, Low (<50%) → human review
3. **Context pull model** — Specialists query shared context store (logs, metrics, events) rather than receiving full context upfront
4. **Escalation triggers** — Time-based (30min, 1hr), impact-based (revenue, user count), confidence-based

**LLM Classification Example:**
```json
{
  "alert": "Payment API 5xx rate > 10%",
  "classification": {
    "severity": "SEV2",
    "confidence": 0.85,
    "category": "api-degradation",
    "specialists_needed": ["log-analyzer", "metric-checker", "k8s-diagnostician"],
    "reasoning": "High error rate indicates service degradation, likely backend issue"
  }
}
```

**Specialist Coordination Patterns:**

From research, specialist agents work best with:
- **Dedicated scope** — Each specialist only fed data from its domain (logs, metrics, K8s state)
- **Independent investigation** — Specialists drive their own diagnosis flow
- **Shared context store** — Pull model where specialists query for what they need
- **Async coordination** — Specialists report findings independently, triage synthesizes

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Rule-based triage** | Deterministic, fast, no LLM cost | Brittle, requires maintenance, misses novel patterns |
| **LLM-based triage** | Handles novel alerts, contextual understanding | LLM cost, latency, requires confidence calibration |
| **Context push (full dump)** | Specialists have all data upfront | Overwhelming, high token cost, irrelevant data |
| **Context pull (query-based)** | Focused, efficient, specialist-driven | Requires query interface, may miss context |
| **Auto-escalation** | Fast response, no human bottleneck | False escalations, alert fatigue |
| **Human-in-loop** | Catches edge cases, high confidence | Slower, human availability dependency |

### Recommendation for Phase 2

**Adopt hybrid LLM-based triage with context pull:**

1. **Triage Agent:**
   - Use LLM to classify alerts (severity, confidence, category)
   - Confidence threshold: 70% for auto-routing, <70% escalate to human
   - Spawn only needed specialists (not all agents for every alert)
   - Log classification reasoning to decision log

2. **Specialist Coordination:**
   - Specialists pull context from shared memory (not pushed by triage)
   - Each specialist has dedicated scope (logs, metrics, K8s, network)
   - Specialists report findings via decision log (visible to all)
   - Triage synthesizes specialist findings into RCA

3. **Escalation Logic:**
   - Time-based: 30min → Team Lead, 1hr → Manager
   - Confidence-based: <50% → human review immediately
   - Impact-based: Revenue impact → executive notification
   - Severity auto-approve: SEV3/SEV4 can auto-escalate, SEV1/SEV2 require human

4. **Implementation Path:**
   - Leverage existing `aof-runtime::AgentExecutor` for specialist spawning
   - Use `aof-memory` for shared context store (query-based)
   - Emit all routing decisions to `CoordinationEvent` stream
   - Build 3-4 specialist agents: log-analyzer, metric-checker, k8s-diagnostician, network-debugger

### Implementation Notes

**Rust Patterns:**

- **LLM routing:** Use `aof-llm` with structured output schema for classification
- **Context store:** Extend `aof-memory` with query interface (key-based retrieval)
- **Specialist spawning:** Use existing `AgentExecutor::spawn()` pattern
- **Escalation chains:** Model as state machine in `workflow` module

**Confidence Threshold Tuning:**

Start conservative:
- **Auto-route threshold:** 75% (reduce false positives)
- **Human escalation:** <60% (catch ambiguous cases)
- **High-risk override:** SEV1 always human-approved, regardless of confidence

**Crates Needed:**
- `aof-llm` — LLM inference for classification
- `aof-runtime` — Agent execution and spawning
- `aof-memory` — Shared context store
- `aof-coordination` — Decision logging via events

**Sources:**
- [Forethought Triage LLM](https://support.forethought.ai/hc/en-us/articles/31216915973651-Triage-Large-Language-Model-LLM)
- [Triangle: Multi-LLM-Agents for Incident Triage](https://www.microsoft.com/en-us/research/wp-content/uploads/2025/02/TRIANGLE_FSE25.pdf)
- [4 Ways AI Agents Redefine Incident Command](https://thenewstack.io/4-ways-ai-agents-redefine-incident-command/)
- [Agentic Incident Management Guide](https://www.ilert.com/agentic-incident-management-guide)

---

## 2. Skills Platform Design

### Current Practice

**Agent Skills Standard (agentskills.io):**

Agent Skills is an **open standard** published by Anthropic (Dec 2025) for giving agents new capabilities. It's been adopted by:
- Anthropic (Claude)
- Microsoft (GitHub Copilot)
- OpenAI (Codex)
- Cursor, Atlassian, Figma

**Format Structure:**

Skills are directories with:
- **Minimum:** `SKILL.md` file (YAML frontmatter + Markdown instructions)
- **Optional:** `scripts/`, `references/`, `assets/` directories

**SKILL.md Example:**
```markdown
---
name: k8s-debug
description: "Kubernetes pod debugging and troubleshooting"
homepage: "https://docs.aof.sh/skills/k8s-debug"
metadata:
  emoji: "🐳"
  version: "1.0.0"
  requires:
    bins: ["kubectl"]
    env: []
    config: ["~/.kube/config"]
  tags: ["kubernetes", "debugging"]
---

# Kubernetes Debug Skill

Expert guidance for debugging Kubernetes workloads...

## When to Use This Skill
- Pod is in CrashLoopBackOff...
```

**Progressive Disclosure:**
When a user's request matches a skill's domain, the agent loads only the relevant skill information (not all skills at once).

**Skill Discovery Patterns:**

From research and existing implementations (Skillshub in Rust):
- **Filesystem scanning:** Auto-discover by scanning for `SKILL.md` files
- **Hot-reload:** Watch filesystem for changes, reload without restart
- **Version management:** Always use latest version (no pinning in v1)
- **Requirements gating:** Check binary, env var, config file existence before offering skill

**AOF Implementation (Existing):**

AOF already has `aof-skills` crate with:
- Frontmatter parsing (YAML + Markdown)
- Requirement checking (bins, env, config, OS)
- Workspace scanning (discovers skills from multiple sources)
- Prompt building (formats skills for LLM consumption)
- Hot-reload via file watching

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Filesystem-based skills** | Version-controlled, transparent, portable | No centralized discovery, manual distribution |
| **Registry-based (npm/pip style)** | Central discovery, versioning, dependency management | Complexity, hosting costs, approval process |
| **Always-latest versioning** | Simple, no version conflicts | Breaking changes impact all agents immediately |
| **Pinned versioning** | Stability, rollback capability | Version drift, compatibility matrix complexity |
| **Requirements gating** | Prevents errors, clear boundaries | Skill may not be offered when needed |
| **No requirements check** | All skills available | Runtime failures, confusing errors |

### Recommendation for Phase 2

**Use agentskills.io standard with filesystem-based discovery:**

1. **Skill Format:**
   - Strict adherence to agentskills.io spec (YAML frontmatter + Markdown)
   - Test compatibility with Claude/Codex (both should parse successfully)
   - Add optional `install` section for binary dependencies (brew, apt, etc.)

2. **Discovery & Loading:**
   - Filesystem scanning on startup (no database, files are source of truth)
   - Hot-reload via file watching (`notify` crate, already in `aof-skills::SkillWatcher`)
   - Progressive disclosure: Load skills only when matched by agent intent
   - Cache parsed skills in memory (invalidate on file change)

3. **Version Management:**
   - Always-latest approach for Phase 2 (defer pinning to Phase 8)
   - Document breaking changes in skill README
   - Skill authors responsible for backward compatibility
   - Future: Add versioning metadata to frontmatter for enterprise use

4. **Requirements Gating:**
   - Check binaries, env vars, config files before offering skill
   - Display clear error if skill unavailable ("kubectl not found, install with...")
   - Auto-suggest installation commands from `install` section
   - Graceful degradation: Offer partial skills if some requirements unmet

5. **Bundled Skills (10-20 ops skills):**
   - K8s debugging (kubectl)
   - Git operations
   - Prometheus queries
   - Loki log search
   - ArgoCD sync
   - Docker operations
   - Shell scripting
   - HTTP testing
   - Incident response procedures
   - Runbook execution

6. **Skill Gap Handling:**
   - Agent confidence scoring: >70% confident → attempt with raw tools
   - <70% confidence → create task for human to build skill
   - Log all attempts with reasoning and confidence level
   - Escalate repeated failures to human for approval

### Implementation Notes

**Rust Implementation (Use Existing aof-skills):**

AOF already has solid foundation:
- `aof_skills::SkillRegistry` — Load from workspace, bundle, enterprise paths
- `aof_skills::RequirementChecker` — Validates bins, env, config, OS
- `aof_skills::SkillWatcher` — Hot-reload via `notify` crate
- `aof_skills::build_skills_prompt()` — Formats for LLM consumption

**Enhancement Needed:**
```rust
// Add agentskills.io validation
impl SkillRegistry {
    pub async fn validate_agentskills_io_compat(&self) -> Result<ValidationReport> {
        // Test parsing with Claude/Codex formats
        // Verify required frontmatter fields
        // Check markdown structure
    }
}

// Add progressive disclosure
impl SkillRegistry {
    pub async fn match_skills(&self, intent: &str) -> Vec<Skill> {
        // Semantic matching of intent to skill tags/description
        // Only load matched skills (not all)
    }
}

// Add installation helpers
impl Skill {
    pub fn suggest_installation(&self) -> Option<InstallCommand> {
        // Parse `install` section, suggest OS-appropriate command
    }
}
```

**Filesystem Structure:**
```
skills/
├── k8s-debug/
│   ├── SKILL.md
│   └── scripts/
│       └── debug-pod.sh
├── prometheus-query/
│   ├── SKILL.md
│   └── references/
│       └── query-examples.txt
└── incident-diagnose/
    └── SKILL.md
```

**Crates:**
- `aof-skills` — Existing, enhance with agentskills.io validation
- `notify` — Already used for hot-reload
- `serde_yaml` — Frontmatter parsing
- `walkdir` — Filesystem scanning

**Sources:**
- [Agent Skills Specification](https://agentskills.io/specification)
- [Anthropic Agent Skills Standard](https://github.com/anthropics/skills/blob/main/spec/agent-skills-spec.md)
- [Agent Skills: Standard for Smarter AI](https://nayakpplaban.medium.com/agent-skills-standard-for-smarter-ai-bde76ea61c13)
- [Skillshub (Rust Implementation)](https://lib.rs/crates/skillshub)

---

## 3. Decision Logging Systems

### Current Practice

**How do systems implement decision transparency and searchability?**

Decision logging systems balance between **audit trails** and **operational context sharing**. Key patterns:

**Event Sourcing:**
- All state changes stored as sequence of events in append-only log
- Events capture the change itself (what happened)
- Can reconstruct past states by replaying events
- Strict correctness/completeness enforcement (business logic depends on it)

**Audit Logs:**
- Record of changes for compliance/security
- Events have no effect on application state
- May be incomplete (best-effort logging)
- Typically write-once, read-rarely

**Virtual Office Model (from OpenClaw/Phase 2 context):**
- Decision logs are **both** audit trail AND team communication
- Chat-like format (agent name, action, reasoning, timestamp)
- Visible to all fleet members + humans
- Searchable by semantic (natural language) + structured (SQL-like) queries

**Semantic Logging in Multi-Agent Systems:**
From research, semantic logging allows structured information logging where logs have relationships between events. This enables:
- Reconstruction of event order during a process
- Detailed execution trace and decision points
- Semantic interpretation according to defined relationships

**Search Architecture:**

Modern decision log systems combine:
1. **Semantic Search** — Vector embeddings + similarity search ("What happened with pod crashes?")
2. **Structured Search** — SQL-like queries (`agent=ops-bot AND action=restart AND confidence>80%`)
3. **Hybrid Approach** — Use both together (LLM + knowledge graph)

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Pure Event Sourcing** | Complete history, time travel, strong consistency | Complex, high storage cost, replay performance |
| **Persistent Log (append-only)** | Simple, fast writes, immutable | No state reconstruction, manual querying |
| **Database (CRUD)** | Easy queries, updates possible | Loses history, no audit trail |
| **File-based logs** | Simple, portable, version-controllable | No indexing, slow search, manual parsing |
| **Semantic-only search** | Natural language queries, context-aware | Slow, LLM cost, imprecise for structured data |
| **Structured-only search** | Fast, precise, efficient | Rigid schema, no natural language queries |
| **Hybrid search** | Best of both worlds | Complexity, dual indexing, sync overhead |

### Recommendation for Phase 2

**Use persistent decision log (append-only) with hybrid search:**

1. **Decision Log Architecture:**
   - Append-only event stream (via `CoordinationEvent`)
   - Stored in file-based log (JSON Lines format for portability)
   - Each decision contains: agent_id, action, reasoning, confidence, timestamp, tags, related_decision_ids
   - No updates (events are immutable, corrections are new events)

2. **Storage Format (JSON Lines):**
```jsonl
{"agent_id":"triage-bot","timestamp":"2024-12-20T14:30:00Z","action":"classify_alert","reasoning":"High 5xx rate indicates API degradation","confidence":0.85,"tags":["incident","api","sev2"],"related":[],"metadata":{"alert_id":"ALT-001","severity":"SEV2"}}
{"agent_id":"log-analyzer","timestamp":"2024-12-20T14:32:15Z","action":"search_logs","reasoning":"Checking for error patterns in last 15min","confidence":0.92,"tags":["investigation","logs"],"related":["event-001"],"metadata":{"query":"error AND payment-api","matches":147}}
```

3. **Virtual Office Interface:**
   - Chat-like display in Mission Control UI (Phase 4)
   - Real-time stream from broadcast channel
   - Thread support (related_decision_ids links decisions)
   - Reactions/comments from humans (future Phase 7)

4. **Search Implementation:**

**Semantic Search (Natural Language):**
- Use embeddings (OpenAI, Anthropic, local model)
- Vector similarity search in decision log corpus
- Query: "What happened with pod crashes?" → finds related decisions

**Structured Search (SQL-like):**
- Parse simple query syntax: `agent=ops-bot AND confidence>0.8`
- Filter JSON Lines by fields
- Fast, precise, no LLM cost

**Hybrid Approach:**
```rust
// User query: "Show high-confidence database restarts"
// 1. Semantic: Generate embedding, find similar decisions
// 2. Structured: Filter agent=* AND action=restart AND confidence>0.7 AND tags contains "database"
// 3. Combine: Intersection of results
```

5. **Access Patterns:**
   - All fleet members can read all decisions (transparency)
   - Humans can filter by agent, time range, severity
   - Agents query before acting (learn from similar past decisions)
   - Export for postmortems (generate timeline from logs)

### Implementation Notes

**Rust Implementation:**

```rust
// Decision log entry
#[derive(Serialize, Deserialize, Clone)]
pub struct DecisionLogEntry {
    pub event_id: String,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub reasoning: String,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub related: Vec<String>,
    pub metadata: serde_json::Value,
}

// Append-only logger
pub struct DecisionLogger {
    log_path: PathBuf,
    broadcaster: EventBroadcaster, // Real-time stream
}

impl DecisionLogger {
    pub async fn log(&self, entry: DecisionLogEntry) -> Result<()> {
        // 1. Append to JSON Lines file
        let json = serde_json::to_string(&entry)?;
        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await?
            .write_all(format!("{}\n", json).as_bytes())
            .await?;

        // 2. Broadcast to subscribers
        self.broadcaster.emit(CoordinationEvent::DecisionLogged(entry));

        Ok(())
    }
}

// Hybrid search
pub struct DecisionSearch {
    embeddings: Option<EmbeddingProvider>, // Semantic
}

impl DecisionSearch {
    pub async fn search(&self, query: &str) -> Result<Vec<DecisionLogEntry>> {
        // Parse query: detect if structured or semantic
        if is_structured_query(query) {
            self.structured_search(query).await
        } else {
            self.semantic_search(query).await
        }
    }
}
```

**Storage Backend:**
- **Phase 2:** File-based (JSON Lines)
- **Phase 8:** Optional SQLite for faster structured queries
- **Future:** Optional Redis/PostgreSQL for distributed deployment

**Indexing Strategy:**
- **Real-time:** No indexing (streaming from broadcast channel)
- **Historical:** File-based search (grep-like for structured, embeddings for semantic)
- **Future:** Full-text index (Tantivy, Meilisearch)

**Crates:**
- `serde_json` — JSON Lines serialization
- `chrono` — Timestamps
- `tokio::fs` — Async file I/O
- `tantivy` (optional) — Full-text search
- Future: `qdrant-client` or `meilisearch-sdk` for semantic search

**Sources:**
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Event Sourcing vs Audit Log](https://www.kurrent.io/blog/event-sourcing-audit)
- [Semantic Logging in Distributed Multi-Agent Systems](https://www.academia.edu/2163795/Semantic_logging_in_a_distributed_multi_agent_system)
- [Structured vs Semantic Search](https://neo4j.com/blog/developer/knowledge-graph-structured-semantic-search/)

---

## 4. Resource Collision Prevention

### Current Practice

**How do distributed systems prevent resource conflicts?**

Distributed locking is the standard approach for preventing concurrent operations on shared resources. Common implementations:

**Redis Locks (Redlock Pattern):**
- SET NX EX command (atomic set-if-not-exists with TTL)
- Lock acquisition: `SET lock_key unique_value NX EX 30`
- Lock release: Lua script to verify ownership before delete
- TTL auto-expiry prevents stuck locks (self-healing)
- Lock extension: Refresh TTL if operation takes longer

**etcd Locks:**
- Lease-based mechanism (token with TTL)
- Transaction-based acquisition (compare-and-swap on key)
- Watch-based waiting (notified when lock released)
- Stronger consistency than Redis (Raft consensus)
- Higher operational overhead

**File-based Locks:**
- POSIX file locks (flock, lockf)
- Simple for single-host scenarios
- No network dependency
- Limited to local filesystem

**Lock Scoping Patterns:**

From Phase 2 context:
- **Destructive ops only:** restart, delete, scale, terminate
- **Read ops parallel:** get logs, get status, inspect metrics
- **Per-resource granularity:** Pod A can lock while Pod B operates freely

**Conflict Resolution:**

- **Block-and-wait:** Agent B blocks until Agent A's lock released
- **Fail-fast:** Return error immediately if locked
- **Queue:** Order operations, process sequentially

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Redis locks** | Simple, fast, self-healing (TTL), good Rust support | No strong consistency, network dependency |
| **etcd locks** | Strong consistency, watch-based, robust | Complex, higher latency, operational overhead |
| **File-based locks** | Simple, no network, local state | Single-host only, no distributed support |
| **Block-and-wait** | Safe, serializes naturally | Latency, potential queue buildup |
| **Fail-fast** | Low latency, no blocking | Requires retry logic, user-visible errors |
| **Per-resource locks** | Fine-grained, high parallelism | More lock objects, complexity |
| **Coarse-grained locks** | Simple, fewer locks | Serializes unrelated operations, low parallelism |

### Recommendation for Phase 2

**Use Redis TTL locks with per-resource granularity:**

1. **Lock Mechanism:**
   - Redis SET NX EX for atomic lock acquisition
   - TTL-based expiry (default 30s, configurable per operation)
   - Ownership verification (store agent_id as lock value)
   - Lock extension via Lua script if operation takes >50% of TTL

2. **Lock Scope:**
   - **Destructive operations only:**
     - `kubectl delete pod`
     - `kubectl scale deployment`
     - `kubectl restart`
     - `argocd app delete`
   - **Read operations (no lock):**
     - `kubectl get pods`
     - `kubectl logs`
     - `prometheus query`
     - `loki search`

3. **Resource Identification:**
   - Lock key format: `aof:lock:{resource_type}:{resource_id}`
   - Examples:
     - `aof:lock:pod:production/payment-api-5f7c8`
     - `aof:lock:deployment:staging/web-frontend`
     - `aof:lock:namespace:production`

4. **Conflict Behavior:**
   - Block-and-wait (default)
   - Timeout after 60s (configurable)
   - Log all lock acquisitions/releases to decision log
   - Emit lock events via `CoordinationEvent`

5. **Self-Healing:**
   - TTL auto-releases locks (no manual cleanup)
   - Agent crash → lock expires after TTL
   - Stale locks detected via ownership check (agent still alive?)

### Implementation Notes

**Rust Implementation (using `redis` crate):**

```rust
use redis::{Client, Commands, Script};
use std::time::Duration;

pub struct ResourceLock {
    client: Client,
    resource_id: String,
    agent_id: String,
    ttl: Duration,
}

impl ResourceLock {
    pub async fn acquire(&self) -> Result<bool> {
        let key = format!("aof:lock:{}", self.resource_id);
        let value = self.agent_id.clone();
        let ttl_secs = self.ttl.as_secs() as usize;

        // SET key value NX EX ttl
        let mut conn = self.client.get_connection()?;
        let result: Option<String> = conn.set_options(
            &key,
            &value,
            redis::SetOptions::default()
                .with_expiration(redis::SetExpiry::EX(ttl_secs))
                .conditional_set(redis::ExistenceCheck::NX)
        )?;

        Ok(result.is_some())
    }

    pub async fn extend(&self) -> Result<bool> {
        // Lua script: extend TTL only if current owner
        let script = Script::new(r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#);

        let key = format!("aof:lock:{}", self.resource_id);
        let ttl_secs = self.ttl.as_secs() as i64;

        let mut conn = self.client.get_connection()?;
        let extended: i64 = script.key(&key)
            .arg(&self.agent_id)
            .arg(ttl_secs)
            .invoke(&mut conn)?;

        Ok(extended == 1)
    }

    pub async fn release(&self) -> Result<bool> {
        // Lua script: delete only if current owner
        let script = Script::new(r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#);

        let key = format!("aof:lock:{}", self.resource_id);

        let mut conn = self.client.get_connection()?;
        let deleted: i64 = script.key(&key)
            .arg(&self.agent_id)
            .invoke(&mut conn)?;

        Ok(deleted == 1)
    }

    pub async fn acquire_with_wait(&self, timeout: Duration) -> Result<bool> {
        let start = std::time::Instant::now();

        loop {
            if self.acquire().await? {
                return Ok(true);
            }

            if start.elapsed() > timeout {
                return Ok(false); // Timeout
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

// Helper: Determine if operation is destructive
pub fn is_destructive_op(tool: &str, args: &[String]) -> bool {
    match tool {
        "kubectl" => {
            args.get(0).map_or(false, |cmd| {
                matches!(cmd.as_str(), "delete" | "scale" | "patch" | "apply" | "create")
            })
        }
        "argocd" => {
            args.get(0).map_or(false, |cmd| {
                matches!(cmd.as_str(), "app delete" | "app sync" | "app rollback")
            })
        }
        _ => false,
    }
}
```

**Configuration:**

```yaml
# Context with locking config
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: production
spec:
  locking:
    enabled: true
    backend: redis
    redis:
      url: redis://localhost:6379
    ttl_seconds: 30
    timeout_seconds: 60
    scope:
      - pattern: "kubectl (delete|scale|patch)"
        ttl: 30
      - pattern: "argocd app delete"
        ttl: 60
```

**Fallback for Phase 2 (No Redis):**

If Redis not available, use **file-based locks** with same interface:
- Lock file: `/tmp/aof-locks/{resource_id}.lock`
- Content: `{agent_id}:{timestamp}`
- TTL emulated via timestamp check
- Works for single-host development/testing

**Crates:**
- `redis` — Redis client with async support
- `tokio::time` — Timeouts and delays
- `serde` — Lock metadata serialization

**Future Enhancements (Phase 8):**
- Distributed lock manager (DLM) crate abstraction
- etcd backend for stronger consistency
- Lock analytics (collision frequency, wait times)
- Deadlock detection (graph-based)

**Sources:**
- [Distributed Locks with Redis](https://redis.io/docs/latest/develop/clients/patterns/distributed-locks/)
- [How to Build Distributed Lock Service with Redis in Rust](https://oneuptime.com/blog/post/2026-01-25-distributed-lock-service-redis-rust/view)
- [Distributed Locking Best Practices](https://scalewithchintan.com/blog/distributed-locking-best-practices-redis-zookeeper-etcd)
- [Rust Redlock Implementation](https://github.com/badboy/redlock-rs)

---

## 5. Sandbox Isolation

### Current Practice

**How do production systems isolate AI agent tool execution?**

Sandbox isolation is critical for agent security. Industry approaches:

**Docker Container Isolation:**
- Agents run tools inside ephemeral containers
- Container-per-tool or container-per-session
- Resource limits (CPU, memory, network)
- File system restrictions
- Credential access control via volume mounts

**MicroVM Isolation (Firecracker, Kata Containers):**
- Stronger isolation than Docker (dedicated kernel per workload)
- Higher overhead (boot time, memory)
- Best for untrusted code execution
- Used by AWS Lambda, Fly.io

**gVisor (User-space Kernel):**
- Application kernel in userspace
- Intercepts syscalls before reaching host kernel
- Lower overhead than microVMs
- Used by Google Cloud Run

**Enhanced Container Isolation (Docker Desktop):**
- Linux user namespaces (map container root to unprivileged host user)
- Prevents container root = host root exploits
- File permission restrictions

**OpenClaw Patterns (from Phase 2 context):**
- Host-level access for trusted operations
- Sandbox per session type or risk level
- Docker-based tool execution for untrusted tools
- File permissions restrict credential access

**Common Vulnerabilities:**

Recent CVEs (2025-2026):
- **CVE-2025-9074:** Docker Desktop container escape via unauthorized Engine access
- **n8n sandbox escape:** Code execution breaking out of n8n's JavaScript sandbox
- **Kernel vulnerabilities:** Shared kernel = attack surface for all containers

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Docker containers** | Simple, fast, good Rust support | Shared kernel, escape risk, credential exposure |
| **MicroVMs** | Strongest isolation, dedicated kernel | Slow boot, high memory, complexity |
| **gVisor** | User-space kernel, syscall filtering | Performance overhead, compatibility issues |
| **User namespaces** | Unprivileged container root | Requires host kernel support, some tools break |
| **File permissions** | Simple, no runtime overhead | Relies on correct permissions, human error risk |
| **seccomp profiles** | Syscall filtering, limits attack surface | May break tools, requires tuning |
| **Network policies** | Limit egress, prevent data exfiltration | Complexity, may break legitimate tools |

### Recommendation for Phase 2

**Use Docker-based sandbox with defense-in-depth:**

1. **Execution Model (adopt OpenClaw pattern):**
   - **Trusted operations:** Run on host (kubectl with user's kubeconfig)
   - **Untrusted tools:** Run in ephemeral Docker containers
   - **Session isolation:** One container per agent session (reused for session lifetime)
   - **Risk-based:** Low-risk (read-only) → host, High-risk (destructive) → sandbox

2. **Docker Security Hardening:**

**User Namespaces:**
- Map container root (UID 0) to unprivileged host user (UID 100000+)
- Prevents container root from becoming host root on escape

**Resource Limits:**
```dockerfile
# Run container with limits
docker run \
  --memory=512m \
  --cpus=1.0 \
  --pids-limit=100 \
  --read-only \
  --tmpfs /tmp:size=100m \
  agent-sandbox:latest
```

**Seccomp Profile (restrict syscalls):**
```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "syscalls": [
    { "names": ["read", "write", "open", "close", "stat"], "action": "SCMP_ACT_ALLOW" },
    { "names": ["execve"], "action": "SCMP_ACT_ERRNO" }
  ]
}
```

**Network Restrictions:**
- Default deny egress
- Whitelist allowed destinations (K8s API, Prometheus, Loki)
- No internet access for high-risk operations

3. **Credential Access Control:**

**File-level permissions:**
- Credentials stored with 600 permissions (owner-only read)
- Mount credentials read-only into container
- Agent-specific credential directories

**Example:**
```bash
# Host: /var/aof/credentials/agent-001/
# Contains: kubeconfig, aws-creds, etc.
# Mounted to container: /credentials/ (read-only)

docker run \
  -v /var/aof/credentials/agent-001:/credentials:ro \
  --user 1000:1000 \
  agent-sandbox:latest
```

**Secret reference pattern (from existing `aof-core::context`):**
```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: production
spec:
  secrets:
    - name: kubeconfig
      path: /credentials/kubeconfig
      mode: "0400"  # Read-only for owner
    - name: aws-creds
      path: /credentials/aws
      mode: "0400"
```

4. **Escape Prevention:**

**Defense layers:**
1. **User namespaces** — Unprivileged container root
2. **Read-only root filesystem** — No binary modification
3. **Seccomp** — Syscall filtering (block dangerous calls)
4. **Resource limits** — Prevent DoS via resource exhaustion
5. **Network policies** — Egress filtering
6. **Audit logging** — Log all privileged operations

**Monitoring:**
- Log all container starts/stops
- Alert on unusual syscalls (via seccomp)
- Track credential access (audit logs)
- Monitor escape indicators (privilege escalation attempts)

5. **Session Trust Boundaries:**

From OpenClaw:
- **Session types:** dev (low trust) vs prod (high trust)
- **Risk levels:** read-only (low) vs write (medium) vs destructive (high)
- **Sandbox decision:**
  - Dev + destructive → always sandbox
  - Prod + read-only → host (faster)
  - Prod + destructive → sandbox + human approval

### Implementation Notes

**Rust Implementation (using `bollard` for Docker):**

```rust
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::models::HostConfig;

pub struct Sandbox {
    docker: Docker,
    image: String,
}

impl Sandbox {
    pub async fn execute_tool(
        &self,
        tool: &str,
        args: &[String],
        credentials_path: Option<&Path>,
    ) -> Result<String> {
        // Create ephemeral container
        let mut host_config = HostConfig {
            memory: Some(512 * 1024 * 1024), // 512MB
            nano_cpus: Some(1_000_000_000),  // 1 CPU
            pids_limit: Some(100),
            read_only_rootfs: Some(true),
            ..Default::default()
        };

        // Mount credentials if provided
        if let Some(creds) = credentials_path {
            host_config.binds = Some(vec![
                format!("{}:/credentials:ro", creds.display())
            ]);
        }

        let config = Config {
            image: Some(&self.image),
            cmd: Some(vec![tool].into_iter().chain(args.iter().map(|s| s.as_str())).collect()),
            host_config: Some(host_config),
            user: Some("1000:1000"), // Unprivileged user
            ..Default::default()
        };

        let container = self.docker.create_container(
            Some(CreateContainerOptions { name: format!("aof-sandbox-{}", uuid::Uuid::new_v4()) }),
            config,
        ).await?;

        // Start container
        self.docker.start_container(&container.id, None::<StartContainerOptions<String>>).await?;

        // Wait for completion and get output
        let output = self.docker.wait_container(&container.id, None::<WaitContainerOptions<String>>).await?;

        // Cleanup
        self.docker.remove_container(&container.id, None).await?;

        Ok(output)
    }

    pub fn should_sandbox(&self, context: &Context, tool: &str, args: &[String]) -> bool {
        // Risk-based sandboxing decision
        let is_destructive = is_destructive_op(tool, args);
        let is_prod = context.metadata.labels.get("env") == Some(&"production".to_string());

        match (is_prod, is_destructive) {
            (false, _) => true,          // Dev always sandboxed
            (true, false) => false,      // Prod read-only on host
            (true, true) => true,        // Prod destructive sandboxed
        }
    }
}
```

**Seccomp Profile (YAML):**
```yaml
# seccomp-profile.json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": ["read", "write", "open", "close", "stat", "fstat", "lstat"],
      "action": "SCMP_ACT_ALLOW"
    },
    {
      "names": ["execve", "execveat"],
      "action": "SCMP_ACT_ERRNO",
      "comment": "Prevent spawning new processes"
    }
  ]
}
```

**Crates:**
- `bollard` — Docker API client for Rust
- `tokio` — Async runtime
- `uuid` — Container naming
- `serde_json` — Seccomp profile parsing

**Future Enhancements (Phase 8):**
- gVisor integration for stronger isolation
- Device pairing (secure multi-client scenarios from OpenClaw)
- Credential rotation (auto-refresh credentials)
- Anomaly detection (unusual credential access patterns)

**Sources:**
- [How to Sandbox AI Agents in 2026](https://northflank.com/blog/how-to-sandbox-ai-agents)
- [Container Escape Vulnerabilities: AI Agent Security](https://blaxel.ai/blog/container-escape)
- [Docker Enhanced Container Isolation](https://docs.docker.com/enterprise/security/hardened-desktop/enhanced-container-isolation/)
- [Claude Code Sandbox Guide](https://claudefa.st/blog/guide/sandboxing-guide)

---

## RESEARCH COMPLETE

### Summary of Key Decisions for Planning

**Incident Response:**
- LLM-based triage with 70% confidence threshold
- Context pull model for specialist coordination
- Escalation: <60% → human, time-based chains, impact-based routing

**Skills Platform:**
- Strict agentskills.io standard (YAML frontmatter + Markdown)
- Filesystem-based discovery with hot-reload
- Always-latest versioning for Phase 2
- Progressive disclosure (load matched skills only)

**Decision Logging:**
- Append-only JSON Lines log (immutable events)
- Hybrid search (semantic + structured)
- Chat-like virtual office interface
- All fleet members read access

**Resource Collision:**
- Redis TTL locks (per-resource granularity)
- Destructive ops only (read ops parallel)
- Block-and-wait with 60s timeout
- Self-healing via TTL auto-expiry

**Sandbox Isolation:**
- Docker-based with defense-in-depth
- User namespaces + seccomp + resource limits + network policies
- Session-level trust boundaries (risk-based sandboxing)
- File-level credential access control

### Implementation Priority

1. **Week 1:** Decision logging + skills platform (foundational)
2. **Week 2:** Incident response triage + specialist coordination
3. **Week 3:** Resource locking + sandbox isolation

### Dependencies Confirmed

- Phase 1 event infrastructure ✓ (needed for decision logging)
- Existing `aof-skills` crate ✓ (enhance with agentskills.io validation)
- Existing `aof-coordination` crate ✓ (extend with decision events)
- New dependency: Redis (or file-based fallback for dev)

---

**Research Date:** 2026-02-13
**Next Step:** `/gsd:plan-phase 2` to create executable implementation plans
