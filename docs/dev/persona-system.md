# Persona System Architecture - Developer Guide

## Overview

The agent persona system gives each AOF agent a distinct personality, communication style, and visual identity. It transforms agents from anonymous scripts into recognizable team members with consistent behavior across all interaction channels (daemon, UI, messaging gateways).

The system is implemented in the `aof-personas` crate and integrates with:
- `aof-core` (event types, coordination events)
- `aofctl serve` (daemon startup, API endpoints)
- Mission Control UI (AgentCard rendering, introduction toasts)

## Architecture

The persona system consists of 5 interconnected components:

```
                     +--------------------------+
                     |   Workspace Files        |
                     |  AGENTS.md  |  SOUL.md   |
                     +------+------+-----+------+
                            |            |
                    +-------v----+ +-----v------+
                    | AgentLoader| | SoulLoader |
                    +-------+----+ +-----+------+
                            |            |
                    +-------v------------v------+
                    |     validate_personas()    |
                    +-------+-------------------+
                            |
          +-----------------+------------------+
          |                 |                  |
   +------v------+  +------v------+   +-------v--------+
   |PromptComposer|  |Introduction |   |ReliabilityCache|
   |  7 layers   |  |EventBuilder |   |  (metrics)     |
   +------+------+  +------+------+   +-------+--------+
          |                 |                  |
          v                 v                  v
   +------+------+  +------+------+   +-------+--------+
   |AgentExecutor|  |EventBroadcst|   |REST API        |
   |(system_prompt)  |(broadcast) |   |/api/agents/:id |
   +-------------+  +------+------+   +-------+--------+
                            |                  |
                     +------v------------------v--------+
                     |        Mission Control UI         |
                     | AgentCard | IntroToast | Metrics  |
                     +--------------------------------------+
```

### Data Flow

```
AGENTS.md ──> AgentLoader.load_from_file()
                │
                ├──> Vec<Agent> ──> PromptComposer ──> system_prompt ──> AgentExecutor
                │                                                        (LLM context)
                ├──> Vec<Agent> ──> build_introduction_event_batch()
                │                         │
                │                         └──> CoordinationEvent[] ──> broadcast channel
                │                                                       │
                │                                                       ├──> WebSocket ──> UI
                │                                                       └──> ReliabilityCache
                │
                └──> Vec<Agent> ──> GET /api/config/agents ──> AgentCard rendering

SOUL.md   ──> SoulLoader.load_from_file()
                │
                ├──> HashMap<String, Soul> ──> PromptComposer (personality layers)
                │
                └──> HashMap<String, Soul> ──> build_introduction_event (intro_message)
```

## Component Details

### 1. File Loaders (`loader.rs`)

**AgentLoader** parses AGENTS.md (YAML format) into `Vec<Agent>`. Uses `serde_path_to_error` for precise error messages that show exact field paths on parse failures.

```rust
// Async file loading
let agents = AgentLoader::load_from_file("workspace/AGENTS.md").await?;

// In-memory parsing (for testing)
let agents = AgentLoader::load_from_str(yaml_content)?;
```

**SoulLoader** parses SOUL.md (Markdown with YAML code blocks) into `HashMap<String, Soul>`. Each agent section starts with `## agent-id` and contains a `yaml` code block followed by prose guidance.

```rust
// Returns empty map if file not found (graceful degradation)
let souls = SoulLoader::load_from_file("workspace/SOUL.md").await?;
```

**AgentCache** provides SHA256-based caching for loaded data. Avoids re-parsing unchanged files on reload.

```rust
let cache = AgentCache::new();
let agents = cache.load_agents("workspace/AGENTS.md").await?;
// Second call returns cached data if file hash matches
let agents2 = cache.load_agents("workspace/AGENTS.md").await?;
```

### 2. System Prompt Composer (`composer.rs`)

Composes dynamic system prompts using 7-layer instruction layering:

| Layer | Source | Content | Priority |
|-------|--------|---------|----------|
| 1. Base Instructions | Fixed | "You are an AI agent..." | Highest |
| 2. Role Definition | AGENTS.md | Name, role, skills | Highest |
| 3. Personality & Values | SOUL.md | Summary, core values | Highest |
| 4. Communication Style | SOUL.md | Style, tone, guide prose | Medium |
| 5. Capabilities & Boundaries | AGENTS.md | CAN/CANNOT lists | Highest |
| 6. Tools Available | TOOLS.md | Tool descriptions linked from skills | Low |
| 7. Behavioral Rules | Fixed | "Always explain reasoning..." | Lowest |

**Truncation strategy:** When prompts exceed the token limit (default 8000 tokens, estimated as `len/4`), layers are removed in reverse priority order:
1. Behavioral rules removed first
2. Tool descriptions shortened to names only
3. Communication guide removed
4. Base, role, personality, and boundaries are never removed

```rust
let composer = PromptComposer::new(agents, souls, tools);

// Full prompt (no limit)
let prompt = composer.compose_system_prompt("k8s-monitor")?;

// With token limit
let prompt = composer.compose_system_prompt_with_limit("k8s-monitor", 8000)?;

// With caching (async)
let prompt = composer.compose_system_prompt_cached("k8s-monitor").await?;

// With validation (injection detection)
let prompt = composer.validate_and_compose("k8s-monitor")?;
```

**Caching:** Prompts are cached per agent with SHA256-based invalidation. The cache uses `Arc<RwLock<HashMap>>` for concurrent access and `AtomicU32` counters for hit/miss tracking.

**Injection detection:** 6 regex patterns detect common prompt injection attempts in composed prompts:
- `ignore all previous`
- `forget instructions`
- `disregard ... prompt`
- `override system`
- `you are now a different/new`
- `ignore the above`

### 3. Introduction Events (`events.rs`)

Builds `CoordinationEvent` instances with `AgentIntroduction` data. Events are emitted at daemon startup and when agents join squads.

```rust
// Single agent introduction
let event = build_introduction_event(&agent, Some(&soul), "session-id");

// Batch (all agents at once)
let events = build_introduction_event_batch(&agents, &souls, "session-id");
```

**AgentIntroduction** structure (from `aof-core`):
```rust
pub struct AgentIntroduction {
    pub agent_id: String,
    pub agent_name: String,
    pub role: String,
    pub avatar: String,           // Emoji
    pub intro_message: String,    // From SOUL.md default_intro
    pub personality_summary: String,
    pub skills: Vec<String>,
}
```

**Fallback behavior:** If no SOUL.md entry exists for an agent, the intro message defaults to `"I'm {name}, your {role}."` and personality_summary is empty.

### 4. Validation (`validation.rs`)

Three-tier validation:

1. **validate_agents()** - Structural validation of AGENTS.md data:
   - No duplicate IDs
   - IDs are lowercase-hyphenated (`^[a-z][a-z0-9-]*$`)
   - Avatar is a single emoji grapheme cluster
   - Required fields non-empty (traits, can, cannot, skills)

2. **validate_souls()** - Structural + security validation of SOUL.md:
   - All soul IDs match an agent ID (cross-reference integrity)
   - Required fields non-empty (values, boundaries, default_intro, style, tone)
   - Prompt injection detection on text fields

3. **validate_personas()** - Combined validation (agents + souls + cross-references)

**Emoji validation** uses `unicode-segmentation` for grapheme counting plus Unicode codepoint range checks for known emoji blocks (Emoticons, Misc Symbols, Transport, Flags, etc.).

### 5. Reliability Metrics (`metrics.rs`)

Computes uptime and success rate from `CoordinationEvent` history.

**Computation:**
- `uptime_percent` = (total - error_events) / total * 100
- `success_rate` = completed_events / total * 100
- Below 10 events (`MIN_EVENTS_FOR_METRICS`): returns `None` (insufficient data)

```rust
// Direct computation
let metrics = compute_agent_metrics("k8s-monitor", &events);

// With time window
let metrics = compute_metrics_with_window("k8s-monitor", &events, 24);
```

**ReliabilityCache:** Thread-safe cache with:
- `Arc<RwLock<HashMap>>` for concurrent metric reads
- `AtomicU64` version counter for UI cache invalidation
- FIFO eviction at 10,000 events to bound memory
- `update_with_event()`: append event, recompute affected agent
- `get_metrics()`: return cached or compute on miss
- `recompute_all()`: full cache refresh

### 6. File Watcher (`watcher.rs`)

Monitors AGENTS.md and SOUL.md for filesystem changes and triggers reload.

```rust
let (watcher, mut rx) = PersonaWatcher::watch_for_changes(
    "workspace/AGENTS.md",
    "workspace/SOUL.md",
)?;

// Listen for updates
while let Some(update) = rx.recv().await {
    // update.agents: Vec<Agent>
    // update.souls: HashMap<String, Soul>
    // Recompose prompts, re-emit introductions, etc.
}
```

**Debouncing:** Events are coalesced with a 100ms quiet period to avoid duplicate notifications from rapid successive writes (e.g., editor save).

## File Formats

### AGENTS.md

```yaml
agents:
  - id: k8s-monitor           # Required: lowercase-hyphenated
    name: Kubernetes Monitor   # Required: display name
    role: Infrastructure Spec  # Required: role description
    avatar: "\U0001F916"       # Required: single emoji
    personality_traits:        # Required: at least 1
      - methodical
      - detail-oriented
    can:                       # Required: at least 1
      - kubectl operations
      - pod debugging
    cannot:                    # Required: at least 1
      - modify cluster RBAC
    skills:                    # Required: at least 1 (links to TOOLS.md)
      - kubectl
      - pod-debugging
```

### SOUL.md

```markdown
# SOUL.md - Agent Personality Guide

## agent-id

\```yaml
id: agent-id
communication_style: formal-technical
tone: calm-professional
values:
  - system-stability
  - transparency
personality_summary: "One-line personality description."
boundaries:
  - "Never trade stability for speed"
default_intro: "I'm Agent Name, your role description."
\```

### Communication Style Guide

Free-form prose describing communication patterns, when to be proactive,
when to escalate, etc. This becomes part of the system prompt.
```

## Integration Points

### aofctl serve (daemon startup)

In `crates/aofctl/src/commands/serve.rs`:

1. Load AGENTS.md and SOUL.md from workspace directory
2. Validate with `validate_personas()`
3. Create `PromptComposer` with loaded data + tools
4. Build introduction events with `build_introduction_event_batch()`
5. Emit events via `EventBroadcaster`
6. Initialize `ReliabilityCache` and subscribe to event stream
7. Start PersonaWatcher for file change monitoring

### AgentExecutor (runtime)

In `crates/aof-runtime/src/executor/agent_executor.rs`:

The `AgentExecutor` accepts an optional `persona_prompt` via the builder pattern:
```rust
let executor = AgentExecutorBuilder::new()
    .with_persona_prompt(composed_prompt)
    .build()?;
```

If `config.system_prompt` is set (expert mode), it takes precedence over the persona prompt.

### REST API

- `GET /api/config/agents` - Returns agent roster with persona fields (avatar, traits, skills)
- `GET /api/agents/:id/metrics` - Returns `ReliabilityMetrics` JSON (uptime_percent, success_rate, event_count)

### Mission Control UI

- `AgentCard` component renders avatar, name, role, personality traits, CAN/CANNOT
- `MetricBadge` displays uptime/success with color coding (green >= 95%, yellow >= 80%, red < 80%)
- `IntroductionToast` shows agent introductions (max 3 with queue, 8s auto-dismiss)
- `useAgentMetrics` hook polls `/api/agents/:id/metrics` with exponential backoff

## Extension Points

### Adding a New Agent

1. Add entry to `workspace/AGENTS.md` with all required fields
2. Add section to `workspace/SOUL.md` with YAML block + prose guide
3. Reload daemon (or wait for PersonaWatcher to detect change)
4. Agent appears in UI, receives composed prompt, emits introduction

### Adding a New Validation Rule

Add to `validation.rs`:
```rust
// In validate_agents()
if agent.some_new_field.is_invalid() {
    bail!("{}.some_new_field: validation message", prefix);
}
```

### Adding a New Prompt Layer

Add in `composer.rs` between existing layers:
```rust
// Layer N: New section
sections.push(format!("[NEW SECTION]\n{}", new_content));
```

Update truncation strategy in `compose_system_prompt_with_limit()` accordingly.

### Adding New Metrics

In `metrics.rs`, extend `ReliabilityMetrics`:
```rust
pub struct ReliabilityMetrics {
    // existing fields...
    pub new_metric: Option<f32>,
}
```

Update `compute_agent_metrics()` to compute the new metric from events.

## Testing Strategy

### Unit Tests (in-module `#[cfg(test)]`)

Each source module has inline unit tests:
- `types.rs`: Construction, serialization, YAML parsing
- `loader.rs`: Valid/invalid YAML parsing, missing field errors
- `composer.rs`: Composition layers, truncation, caching, injection detection
- `validation.rs`: ID format, duplicate detection, emoji validation, injection
- `events.rs`: Event construction, batch, serialization, fallbacks
- `metrics.rs`: Computation, edge cases, cache operations

### Integration Tests (`tests/` directory)

- `loader_tests.rs`: File-based loading, SOUL.md section parsing
- `composer_tests.rs`: Cross-module composition with real fixtures
- `integration_composer_test.rs`: Full load-compose workflow
- `persona_events_test.rs`: Event emission patterns
- `metrics_computation_test.rs`: Metric edge cases
- `metrics_performance_test.rs`: Performance benchmarks
- `integration_e2e_test.rs`: Full pipeline validation (14 tests)

### Running Tests

```bash
# All persona tests (unit + integration + doc tests)
cargo test -p aof-personas

# Just unit tests
cargo test -p aof-personas --lib

# Just E2E integration test
cargo test -p aof-personas --test integration_e2e_test

# Specific test
cargo test -p aof-personas test_full_persona_workflow_integration
```

## Known Limitations

1. **Token counting approximation:** Uses `len/4` estimate (Claude standard). Actual token counts may differ by 10-20%. For production, consider tiktoken or a proper tokenizer.

2. **Emoji rendering inconsistency:** Some emoji render differently across browsers and terminals. Recommend using common emoji from the Emoticons block (U+1F600-1F64F).

3. **No behavioral fine-tuning:** Agents don't learn or adapt their personality from interactions. Personality is static, defined entirely by workspace files.

4. **Squad customization deferred:** Per-squad personality overrides via `squads.yaml` are supported but not fully integrated in the daemon startup flow.

5. **Large skill lists:** Agents with 50+ skills produce long tool sections. Truncation handles this, but consider splitting into multiple specialized agents.

6. **Prompt injection detection is heuristic:** 6 regex patterns catch common attacks but are not exhaustive. Review SOUL.md changes in code review for sensitive deployments.

## Patterns for Other AOF Features

The persona system establishes patterns reusable across AOF:

1. **Composable workspace files:** Define features via human-editable, version-controlled files (YAML + Markdown). Parse with `serde_path_to_error` for precise error messages.

2. **7-layer instruction composition:** Separate concerns into distinct layers with clear truncation priorities. Makes prompts debuggable and extensible.

3. **Event-based architecture:** Use `CoordinationEvent` broadcast channel for cross-component communication. Subscribers can be UI, gateways, or internal services.

4. **SHA256 cache invalidation:** Hash input data to detect changes. Works for both file-level caching (AgentCache) and prompt-level caching (PromptComposer).

5. **Graceful degradation:** Missing files log warnings but don't crash. Optional data uses sensible defaults.

6. **File watching with debounce:** Monitor config files for changes, coalesce rapid events, validate before applying.
