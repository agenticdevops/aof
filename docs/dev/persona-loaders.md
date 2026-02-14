# Persona Loaders - Internal Developer Documentation

## Crate: `aof-personas`

### Purpose

Provides file loaders, validators, and caching for the agent persona system. This crate is the foundation that other persona-related functionality depends on (system prompt composition, introduction events, UI rendering).

### Module Structure

```
crates/aof-personas/
├── src/
│   ├── lib.rs          # Module declarations and re-exports
│   ├── types.rs        # Agent, Soul, SoulFrontmatter, AgentsFile structs
│   ├── loader.rs       # AgentLoader, SoulLoader, AgentCache
│   ├── validation.rs   # validate_agents, validate_souls, validate_personas, injection detection
│   └── watcher.rs      # PersonaWatcher with file change monitoring
├── tests/
│   └── loader_tests.rs # 17 integration tests
└── Cargo.toml
```

### Key Types

#### `Agent` (types.rs)
Represents an agent from AGENTS.md. All fields are `String` or `Vec<String>`.
- `id`: Unique lowercase-hyphenated identifier
- `name`, `role`, `avatar`: Display identity
- `personality_traits`, `can`, `cannot`, `skills`: Behavioral metadata
- `#[serde(default)]` on list fields makes them optional in YAML (defaults to empty vec)

#### `Soul` (types.rs)
Represents personality guidance from SOUL.md.
- Same structured fields as `SoulFrontmatter` plus `communication_guide` (prose)
- `From<SoulFrontmatter>` conversion sets `communication_guide` to empty string

#### `AgentCache` (loader.rs)
Thread-safe caching layer using `Arc<RwLock<>>` and SHA256 content hashing.
- `load_agents(path)` / `load_souls(path)` -- returns cached data if hash matches
- `invalidate()` -- forces re-read on next access

### Parsing Strategy

#### AGENTS.md
Pure YAML file. Parsed directly with `serde_yaml::Deserializer` + `serde_path_to_error`. The `AgentsFile` struct wraps `Vec<Agent>`.

#### SOUL.md
Markdown with embedded YAML. Parsing strategy:
1. Split content by `"\n## "` to get per-agent sections
2. For each section, find `` ```yaml `` and `` ``` `` boundaries
3. Extract YAML between boundaries, parse with serde_path_to_error
4. Everything after the closing `` ``` `` is the prose communication guide
5. Return `HashMap<String, Soul>` keyed by agent ID

**Edge cases handled:**
- First section before any `##` header is skipped (document title)
- Missing SOUL.md returns empty map (not an error)
- Sections without YAML blocks are skipped with a debug log
- `---` separators between sections are stripped

### Validation Logic

Three-level validation:
1. `validate_agents()` -- structural validation (ids, format, emoji, non-empty fields)
2. `validate_souls()` -- reference integrity + prompt injection detection
3. `validate_personas()` -- runs both in sequence

**Emoji validation** uses `unicode-segmentation` crate for grapheme cluster counting, then checks Unicode codepoint ranges for known emoji blocks.

**Prompt injection detection** uses regex patterns:
- `(?i)ignore\s+all\s+previous`
- `(?i)forget\s+(all\s+)?instructions`
- `(?i)disregard\s+.*prompt`
- `(?i)override\s+system`
- `(?i)you\s+are\s+now\s+(?:a|an)\s+(?:different|new)`
- `(?i)ignore\s+(?:the\s+)?above`

### File Watching

`PersonaWatcher::watch_for_changes(agents_path, souls_path)` returns a `mpsc::Receiver<PersonaUpdate>`.

Implementation details:
- Uses `notify` crate (`RecommendedWatcher`)
- Watches parent directories of both files (non-recursive)
- Debounces events with 100ms quiet period to coalesce rapid writes
- On change: reload -> validate -> emit PersonaUpdate
- Validation failure logs warning but does NOT emit update (preserves last valid state)
- Receiver drop cleanly stops the watcher task

### Dependencies

| Crate | Purpose |
|-------|---------|
| serde + serde_yaml | YAML parsing with derive macros |
| serde_path_to_error | Precise field-path error messages |
| anyhow | Error handling with context |
| regex | Prompt injection pattern matching |
| unicode-segmentation | Emoji grapheme cluster validation |
| sha2 | SHA256 content hashing for cache |
| notify | Filesystem event monitoring |
| tokio | Async file I/O and channels |
| tracing | Structured logging |
| chrono | Timestamps on PersonaUpdate events |

### Test Coverage

33 tests total:
- 14 unit tests (types, loader, validation modules)
- 17 integration tests (loader_tests.rs)
- 2 doc-tests

Coverage areas:
- Happy path: valid AGENTS.md, valid SOUL.md, 3 agents parsed
- Error cases: missing fields, duplicate IDs, invalid emoji, malformed YAML
- Security: 6 prompt injection variants detected
- Caching: cache hit/miss, content change detection
- Edge cases: empty files, nonexistent files, bytes loading
- Cross-reference: soul ID mismatch, missing soul permitted

### Usage from Other Crates

```rust
// In aof-runtime or aofctl
use aof_personas::{AgentLoader, SoulLoader, validate_personas, AgentCache, PersonaWatcher};

// One-shot loading
let agents = AgentLoader::load_from_file("workspace/AGENTS.md").await?;
let souls = SoulLoader::load_from_file("workspace/SOUL.md").await?;
validate_personas(&agents, &souls)?;

// Cached loading (daemon usage)
let cache = AgentCache::new();
let agents = cache.load_agents("workspace/AGENTS.md").await?;

// File watching (daemon usage)
let (watcher, mut rx) = PersonaWatcher::watch_for_changes(
    "workspace/AGENTS.md",
    "workspace/SOUL.md",
)?;
while let Some(update) = rx.recv().await {
    // Handle persona reload
}
```
