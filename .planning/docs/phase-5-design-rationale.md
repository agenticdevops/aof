# Phase 5: Agent Personas - Design Rationale

## Purpose

This document explains WHY the persona system was designed the way it was. It captures the reasoning behind each decision, alternatives considered, and tradeoffs accepted. Future maintainers should read this to understand design intent, not just mechanism.

## Decision 1: File-based Configuration vs Database

**Chosen:** Plain-text Markdown workspace files (AGENTS.md, SOUL.md)

**Alternatives considered:**
| Approach | Pros | Cons |
|----------|------|------|
| **Workspace files** (chosen) | Version-controlled, human-editable, mergeable, inspectable, no infrastructure | No query capability, limited to local filesystem |
| **SQLite database** | Queryable, transactional, fast reads | Operational complexity, not version-controlled, binary file in repo |
| **Remote API/service** | Multi-tenant, centralized | Network dependency, infrastructure cost, latency |
| **TOML configuration** | Strongly typed, familiar to Rust devs | Less readable for personality prose, harder to hand-edit |

**Why files won:** Personas are inherently version-controlled artifacts. Teams review persona changes in PRs, track history, and merge branches. Files satisfy this perfectly. A database would require migration tooling, backup strategy, and operational overhead -- all unnecessary for a config-level concern. The OpenClaw pattern (workspace files for agent config) is proven at scale.

**Tradeoff accepted:** No runtime query capability (e.g., "find all agents with trait X"). For the current scale (3-50 agents), linear scanning of loaded data is sufficient.

## Decision 2: AGENTS.md + SOUL.md Split

**Chosen:** Two files with different formats for different concerns

**Why two files instead of one:**

| Aspect | AGENTS.md | SOUL.md |
|--------|-----------|---------|
| **Content type** | Structured identity data | Personality prose + metadata |
| **Format** | Pure YAML | Markdown with YAML frontmatter |
| **Change frequency** | Rarely (when adding/removing agents) | Often (tuning personality) |
| **Author** | Platform engineer | Agent designer / domain expert |
| **Review focus** | Capability boundaries, skills | Communication style, tone |

Splitting allows each to evolve independently. An agent designer can tune the SOUL.md communication guide without touching the structured AGENTS.md roster. Different reviewers can focus on their area of expertise.

**Alternative considered:** Single `personas.yaml` with all data. Rejected because personality prose (communication guides) doesn't fit well in YAML -- it becomes awkward multiline strings. Markdown is the natural format for writing guidance text.

## Decision 3: 7-Layer Instruction Composition

**Chosen:** Compose system prompts from 7 distinct sections with clear headers

```
[BASE INSTRUCTIONS]       -- Fixed foundation
[ROLE DEFINITION]         -- From AGENTS.md
[PERSONALITY & VALUES]    -- From SOUL.md
[COMMUNICATION STYLE]     -- From SOUL.md
[CAPABILITIES & BOUNDS]   -- From AGENTS.md
[TOOLS]                   -- From TOOLS.md via skills
[BEHAVIORAL RULES]        -- Fixed guidelines
```

**Alternatives considered:**
| Approach | Pros | Cons |
|----------|------|------|
| **7-layer composition** (chosen) | Debuggable, modular, clear truncation priority | Slightly longer prompts |
| **Single monolithic template** | Shorter, simpler | Hard to debug, hard to truncate intelligently |
| **PromptForge templating** | Variable substitution, Mustache-style | Extra dependency, added complexity for limited benefit |
| **Separate system/user messages** | Personality in system, skills in user context | Not all LLM providers support this pattern |

**Why layering won:** When an agent behaves unexpectedly, the first question is "what's in the prompt?" With labeled sections, a developer can quickly identify which layer contributes which behavior. The `[SECTION HEADERS]` make logs instantly readable. The truncation strategy is natural: remove sections in reverse priority order, knowing exactly what's being dropped.

**PromptForge rejected because:** The variable substitution pattern is more complex than needed. Our data structures map directly to prompt sections -- simple string formatting is sufficient. PromptForge would add a crate dependency for minimal benefit.

## Decision 4: Token Limits with Graceful Truncation

**Chosen:** Estimate tokens as `len/4`, default limit 8000, graceful truncation by priority

**Token counting approach:**
- `len/4` is the Claude standard approximation (1 token ~ 4 characters)
- Conservative but sufficient for budget management
- Actual token counts may differ by 10-20%

**Truncation priority (lowest removed first):**
1. Behavioral rules (generic, least personalized)
2. Tool descriptions (shortened to tool names only)
3. Communication guide (prose reduced)
4. Base instructions, role, personality, boundaries (NEVER removed)

**Why this order:** The core identity (who the agent is, what it can/cannot do) must survive truncation. Generic behavioral rules ("always explain reasoning") add the least persona-specific value. Tool descriptions can be abbreviated to names without losing functionality -- the LLM already knows what kubectl does.

**Alternative considered:** Hard-fail when prompt exceeds limit. Rejected because it would prevent agents with many skills from functioning at all. Graceful degradation is more user-friendly.

## Decision 5: Reliability Metrics from Events (Not Stored)

**Chosen:** Compute uptime and success rate from CoordinationEvent history

**Alternatives:**
| Approach | Pros | Cons |
|----------|------|------|
| **Computed from events** (chosen) | Always current, survives restarts, no separate store | Requires event history in memory |
| **Stored metrics** (PostgreSQL/Redis) | Queryable, persistent, aggregate | Infrastructure dependency, staleness, sync issues |
| **Learned models** (ML-based) | Adaptive, predictive | Complexity, training data needs, opacity |
| **External monitoring** (Prometheus/Datadog) | Professional grade | External dependency, config overhead |

**Why computed won:** Phase 1 already established the CoordinationEvent broadcast channel. Events contain all information needed for metrics (agent_id, activity_type, timestamp). Computing from the event stream adds zero infrastructure: no database, no external service. The ReliabilityCache bounds memory with FIFO eviction at 10,000 events.

**MIN_EVENTS_FOR_METRICS = 10:** Below 10 events, percentages are statistically meaningless. An agent with 2 successes shows "100%" which creates false confidence. Returning `null` (displayed as "--" in UI) is more honest.

## Decision 6: Introduction as Broadcast Event

**Chosen:** Introduction events are CoordinationEvent types emitted on the Phase 1 broadcast channel

**Why not a separate mechanism:**
- Reuses existing infrastructure (broadcast channel, WebSocket, subscriber pattern)
- Introduction events are visible in the same event stream as activity events
- No additional transport layer needed
- WebSocket clients automatically receive introductions alongside activity events

**Emission timing:**
- **Daemon startup:** All configured agents introduce themselves
- **Not on restart:** Would spam messages (introduction is a first-time event)
- **On squad assignment:** When new agents are added via config change

The `AgentIntroduction` struct is attached as `Option<AgentIntroduction>` on `CoordinationEvent` (via `skip_serializing_if`). This keeps backward compatibility -- existing events don't include the introduction field in JSON.

## Decision 7: Emoji Avatars (MVP)

**Chosen:** Single emoji character stored in AGENTS.md

**Why emoji for MVP:**
- Single character, trivially parsed
- Universally supported across all platforms (terminal, web, Slack, Discord)
- No CDN, no image hosting, no binary files in repo
- Human-readable in config files
- Version-controlled alongside other agent data

**Validation:** Unicode grapheme clustering (`unicode-segmentation` crate) plus codepoint range checks for known emoji blocks. This catches multi-character "emoji" (like text + variation selector) and non-emoji characters.

**Future extension (Phase 5.2):** Optional `avatar_url` or `avatar_svg` fields for custom images. The emoji field remains as fallback.

## Decision 8: Caching Strategy

**Chosen:** SHA256-based cache invalidation with `Arc<RwLock>` storage

### Prompt Cache (PromptComposer)

- Stores composed prompts per agent: `agent_id -> (prompt, timestamp)`
- Invalidation: SHA256 hash of all input data (agents + souls + tools)
- If hash unchanged, return cached prompt (hit)
- If hash changed, recompose and update cache (miss)
- Hit/miss counters via `AtomicU32` for monitoring

### File Cache (AgentCache)

- Stores parsed data per file: `path -> (data, content_hash)`
- SHA256 of file content for invalidation
- Avoids re-parsing unchanged files on reload

### Metrics Cache (ReliabilityCache)

- Stores computed metrics per agent: `agent_id -> ReliabilityMetrics`
- Updated on every event (recompute affected agent only)
- `AtomicU64` version counter for UI cache invalidation (X-Metrics-Version header)
- FIFO eviction at 10,000 events to bound memory

**Why SHA256:** Deterministic, collision-resistant, fast. The same pattern was already used in `config.rs` for `X-Config-Version` headers. Consistency across the codebase.

## Decision 9: Prompt Injection Detection

**Chosen:** 6 regex patterns detect common injection attempts

```
ignore all previous
forget (all) instructions
disregard ... prompt
override system
you are now a different/new
ignore (the) above
```

**Why heuristic (not ML-based):**
- Fast: regex matches in microseconds
- Transparent: clear what's being checked
- Zero infrastructure: no model loading, no API calls
- Sufficient for MVP: catches the most common injection patterns
- False positives are preferable to missed attacks

**Limitations acknowledged:**
- Not exhaustive (sophisticated attacks could bypass)
- No semantic analysis (can't detect encoded or paraphrased injection)
- Recommending code review of SOUL.md changes as the primary defense

**Applied at two levels:**
1. Validation (`validation.rs`): checks raw SOUL.md text fields
2. Composition (`composer.rs`): checks the final composed prompt

## Decision 10: Separate `aof-personas` Crate

**Chosen:** New crate rather than extending `aof-core`

**Why separate:**
- Distinct concern boundary: persona parsing/composition vs core agent types
- Independent testing: can test persona logic without compiling the full runtime
- Smaller compilation units: changes to persona code don't rebuild `aof-core`
- Clear dependency direction: `aof-personas` depends on `aof-core`, not vice versa

**What stays in aof-core:**
- `CoordinationEvent` struct (used by all crates)
- `AgentIntroduction` struct (event payload type)
- `ActivityEvent` and `ActivityType` (event classification)

**What goes in aof-personas:**
- File loaders (AGENTS.md, SOUL.md parsing)
- Prompt composition (7-layer engine)
- Validation (structural + injection detection)
- Reliability metrics (computation + cache)
- File watching (change detection + reload)

## Decision Summary Table

| Decision | Chosen | Why | Alternatives Rejected |
|----------|--------|-----|----------------------|
| Config storage | Files (AGENTS.md + SOUL.md) | Version-controlled, editable, no infra | Database, remote API |
| File split | Two files | Different concerns, change frequencies | Single file |
| Prompt pattern | 7-layer composition | Debuggable, truncation-friendly | Monolithic, PromptForge |
| Token handling | len/4 estimate, graceful truncation | Conservative, no failures | Hard fail, tiktoken |
| Reliability source | Computed from events | No infra, always current | Stored, ML-based |
| Introduction delivery | Broadcast events | Reuses Phase 1 infra | Separate transport |
| Avatar format | Emoji | Universal, zero-config | Image URL, SVG |
| Cache pattern | SHA256 invalidation | Deterministic, fast | Timestamp-based, TTL |
| Security | 6 regex patterns | Fast, transparent | ML-based detection |
| Crate structure | Separate aof-personas | Clean boundaries | Extend aof-core |

## Patterns Applicable to Other AOF Features

1. **Workspace file pattern:** Human-editable config -> loader -> validator -> cache -> consumer. Applicable to trigger definitions, gateway configs, coordination protocols.

2. **Instruction layering:** Compose structured prompts from multiple data sources with clear priorities. Applicable to conversational interface (Phase 6) and coordination protocols (Phase 7).

3. **Event-based integration:** Use CoordinationEvent broadcast for cross-component communication. No tight coupling between components.

4. **SHA256 cache invalidation:** Hash input data to detect changes. Fast, deterministic, no TTL tuning needed.

5. **Graceful degradation:** Missing config files log warnings but don't crash. Optional data uses sensible defaults. The system always starts.

6. **File watching with debounce:** Monitor config for changes, validate before applying. Applicable to any config-driven feature.
