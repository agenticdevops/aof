---
phase: 01-event-infrastructure
plan: 01
subsystem: coordination
tags: [foundation, events, coordination, persistence]
dependency_graph:
  requires: []
  provides:
    - CoordinationEvent (event envelope with routing metadata)
    - EventBroadcaster (tokio::broadcast wrapper for pub/sub)
    - SessionPersistence (FileBackend wrapper for state storage)
  affects:
    - aof-core (new coordination module)
    - workspace (new aof-coordination crate)
tech_stack:
  added:
    - tokio::sync::broadcast (event broadcasting)
    - aof-memory::FileBackend (session persistence)
  patterns:
    - pub/sub event distribution
    - session state snapshots for daemon restarts
key_files:
  created:
    - crates/aof-core/src/coordination.rs
    - crates/aof-coordination/Cargo.toml
    - crates/aof-coordination/src/lib.rs
    - crates/aof-coordination/src/events.rs
    - crates/aof-coordination/src/broadcaster.rs
    - crates/aof-coordination/src/persistence.rs
  modified:
    - crates/aof-core/src/lib.rs
    - crates/aof-core/Cargo.toml
    - Cargo.toml
decisions:
  - title: "Convenience constructors in aof-core not aof-coordination"
    rationale: "Cannot implement methods on types outside their defining crate. Added agent_started, agent_completed, tool_executing, thinking, error to CoordinationEvent in aof-core."
    alternatives: ["Extension trait in aof-coordination"]
    selected: "Direct implementation in aof-core"
  - title: "Use AofError::memory for serialization errors"
    rationale: "SessionPersistence errors are memory/storage related. AofError doesn't have ::internal, so used ::memory constructor for consistency."
    alternatives: ["AofError::config", "anyhow::Error"]
    selected: "AofError::memory"
  - title: "EventBroadcaster ignores send errors"
    rationale: "No active subscribers is valid state. Events are best-effort, not guaranteed delivery. Logs debug messages for monitoring."
    alternatives: ["Return Result and force caller to handle", "Buffer events for future subscribers"]
    selected: "Ignore errors, log debug"
metrics:
  duration_seconds: 485
  tasks_completed: 2
  files_created: 6
  files_modified: 3
  commits: 2
  tests_added: 20
  lines_of_code: 1006
completed_date: 2026-02-11
---

# Phase 01 Plan 01: Foundation Types and Coordination Crate Summary

**One-liner:** Created CoordinationEvent wrapper with routing metadata and aof-coordination crate providing EventBroadcaster (tokio::broadcast) and SessionPersistence (FileBackend) for multi-agent event streaming.

## Objective

Established foundation types and aof-coordination crate powering Phase 1's event streaming architecture. All subsequent plans depend on CoordinationEvent (event envelope), EventBroadcaster (pub/sub bus), and SessionPersistence (state survival across restarts).

## Tasks Completed

### Task 1: Add CoordinationEvent type to aof-core ✓
**Commit:** `76c4b11`

Created `crates/aof-core/src/coordination.rs` with:
- **CoordinationEvent** - wraps ActivityEvent with agent_id, session_id, event_id (UUID v4), timestamp
- **SessionState** - serializable session snapshot with agent_states, task_queue, timestamps
- **AgentState** - individual agent status (Idle, Running, Completed, Error, Disconnected)
- **AgentStatus** enum - agent state variants
- **TaskInfo** - task coordination with task_id, description, assigned_agent, status
- **TaskStatus** enum - task lifecycle (Pending, InProgress, Completed, Failed, Cancelled)
- Convenience constructors: agent_started(), agent_completed(), tool_executing(), thinking(), error()

All types implement Serialize + Deserialize for JSON persistence. Added 14 unit tests covering event creation, unique ID generation, serialization, status equality, and convenience constructors.

**Files:**
- Created: `crates/aof-core/src/coordination.rs` (343 lines)
- Modified: `crates/aof-core/src/lib.rs` (added module and re-exports)
- Modified: `crates/aof-core/Cargo.toml` (added uuid dependency)

### Task 2: Create aof-coordination crate ✓
**Commit:** `6a4b98e`

Created workspace crate `aof-coordination` with:

**EventBroadcaster** (`broadcaster.rs`):
- Wraps `tokio::sync::broadcast::Sender<CoordinationEvent>`
- `new(capacity: usize)` - creates broadcast channel (default 1000 events)
- `emit(&self, event: CoordinationEvent)` - sends to all subscribers, ignores errors if no subscribers
- `subscribe() -> Receiver<CoordinationEvent>` - returns new receiver
- `subscriber_count() -> usize` - for health checks
- Clone-able for multiple emitters

**SessionPersistence** (`persistence.rs`):
- Uses `aof_memory::SimpleMemory` with FileBackend
- `new(persist_dir: PathBuf)` - stores at `persist_dir/session-state.json`
- `save_session(&SessionState) -> Result<()>` - serializes to JSON, stores by session_id
- `restore_session(session_id) -> Result<Option<SessionState>>` - retrieves by session_id
- `list_sessions() -> Result<Vec<String>>` - list all session IDs
- `delete_session(session_id) -> Result<()>` - remove session
- `clear_all() -> Result<()>` - remove all sessions

**events.rs**: Re-exports CoordinationEvent convenience constructors from aof-core

**lib.rs**: Public API with re-exports and crate documentation

**Files:**
- Created: `crates/aof-coordination/Cargo.toml`
- Created: `crates/aof-coordination/src/lib.rs` (58 lines)
- Created: `crates/aof-coordination/src/events.rs` (9 lines)
- Created: `crates/aof-coordination/src/broadcaster.rs` (208 lines)
- Created: `crates/aof-coordination/src/persistence.rs` (242 lines)
- Modified: `Cargo.toml` (added crate to workspace members and dependencies)

## Verification Results

✅ **All verification criteria met:**

1. `cargo check --workspace` - PASSED (all crates compile)
2. `cargo test -p aof-core coordination` - PASSED (14 tests, 0 failures)
3. `cargo test -p aof-coordination` - PASSED (11 tests, 0 failures)
4. CoordinationEvent wraps ActivityEvent with agent_id, session_id, event_id - VERIFIED
5. EventBroadcaster supports multiple subscribers receiving same events - VERIFIED (test_single_producer_multiple_consumers)
6. SessionPersistence saves/restores SessionState across calls - VERIFIED (test_persistence_across_instances)

**Test coverage:**
- Coordination module: 14 tests (event creation, unique IDs, serialization, convenience constructors)
- Broadcaster: 6 tests (single/multiple consumers, no subscribers, subscriber count, clone)
- Persistence: 5 tests (save/restore, list, delete, clear, persistence across instances)

## Deviations from Plan

None - plan executed exactly as written. All must_haves delivered:

✅ CoordinationEvent wraps ActivityEvent with routing metadata
✅ EventBroadcaster emits to multiple subscribers via tokio::broadcast
✅ SessionPersistence saves/restores session state to/from FileBackend
✅ aof-coordination crate compiles and unit tests pass

## Key Decisions

### 1. Convenience Constructors Location
**Decision:** Implemented convenience constructors (agent_started, agent_completed, etc.) directly on CoordinationEvent in aof-core rather than extension trait in aof-coordination.

**Rationale:** Rust doesn't allow implementing methods on types outside their defining crate. Initially attempted to add impl block in aof-coordination/src/events.rs, which resulted in compiler error E0116. Moving to aof-core maintains all CoordinationEvent functionality in one place.

**Alternatives considered:**
- Extension trait in aof-coordination (more complex, less discoverable)
- Free functions in aof-coordination (less ergonomic)

### 2. Error Handling Strategy
**Decision:** Use `AofError::memory()` for serialization/deserialization errors in SessionPersistence.

**Rationale:** SessionPersistence operations are fundamentally memory/storage operations. AofError doesn't provide `::internal()` constructor. Using `::memory()` groups these errors with other storage-related failures (FileBackend, MemoryBackend).

**Alternatives considered:**
- `AofError::config()` - less semantically accurate
- Wrapping in `anyhow::Error` - breaks AofResult consistency across crate

### 3. EventBroadcaster Send Error Handling
**Decision:** EventBroadcaster::emit() ignores send errors when no subscribers are active.

**Rationale:** Zero active subscribers is a valid operational state (e.g., daemon running before any WebSocket clients connect). Events are best-effort notifications, not guaranteed delivery. Logs debug messages for observability without failing caller.

**Alternatives considered:**
- Return Result and force caller to handle - adds boilerplate everywhere
- Buffer events for future subscribers - unbounded memory growth risk

## Architecture Impact

### Dependencies Created
- **Downstream consumers** (future plans) can now:
  - Import `aof_coordination::{EventBroadcaster, SessionPersistence, CoordinationEvent}`
  - Emit coordination events with routing metadata
  - Subscribe to events via broadcast channel
  - Persist/restore session state across daemon restarts

### Type System
- CoordinationEvent is the **canonical event type** for multi-agent coordination
- ActivityEvent remains focused on single-agent TUI logging
- Clear separation: ActivityEvent (what happened) vs CoordinationEvent (what + who + when + session)

### Crate Structure
```
aof-core (0 deps added)
  └─ coordination.rs (foundation types)
       ↓
aof-coordination (new crate)
  ├─ broadcaster.rs (tokio::broadcast wrapper)
  ├─ persistence.rs (aof-memory FileBackend wrapper)
  └─ events.rs (re-exports)
```

## Technical Notes

### Event Broadcasting Pattern
EventBroadcaster uses `tokio::sync::broadcast`, which provides:
- **Clone semantics**: Each subscriber gets independent receiver
- **Lagging handling**: Receivers that can't keep up get RecvError::Lagged
- **Zero-copy**: Events are Arc-wrapped internally by tokio
- **Capacity**: Fixed at channel creation (1000 events default)

**Trade-offs:**
- ✅ Efficient multi-subscriber distribution
- ✅ No coordinator thread required
- ❌ Slow subscribers can lag and miss events (future: metrics/alerts)
- ❌ Bounded capacity (future: backpressure strategy)

### Persistence Strategy
SessionPersistence uses FileBackend with JSON serialization:
- **Immediate writes**: Each save_session() writes to disk (durability)
- **No buffering**: Simple, predictable behavior
- **Session-per-key**: Each session_id is independent JSON document

**Trade-offs:**
- ✅ Survives daemon crashes/restarts
- ✅ Human-readable JSON for debugging
- ✅ No external dependencies (no database)
- ❌ File I/O on every save (future: batching if performance issue)
- ❌ No ACID transactions across sessions (acceptable for current use case)

## Next Steps

This plan provides the atoms for Phase 1's event streaming architecture. Subsequent plans will:

1. **Plan 02** - Modify aof-runtime to emit CoordinationEvent during agent execution
2. **Plan 03** - Create WebSocket server in aofctl (`serve` command) that broadcasts events
3. **Plan 04** - Implement session lifecycle (create, restore, cleanup) using SessionPersistence

## Self-Check: PASSED

Verified all claimed artifacts exist:

```bash
# Files created
✓ crates/aof-core/src/coordination.rs
✓ crates/aof-coordination/Cargo.toml
✓ crates/aof-coordination/src/lib.rs
✓ crates/aof-coordination/src/events.rs
✓ crates/aof-coordination/src/broadcaster.rs
✓ crates/aof-coordination/src/persistence.rs

# Commits
✓ 76c4b11 feat(01-event-infrastructure): add CoordinationEvent types to aof-core
✓ 6a4b98e feat(01-event-infrastructure): create aof-coordination crate with EventBroadcaster and SessionPersistence
```

All files present. All commits in git log. All tests passing.
