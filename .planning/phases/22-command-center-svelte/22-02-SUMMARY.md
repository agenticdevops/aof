---
phase: 22-command-center-svelte
plan: 02
subsystem: api
tags: [websocket, axum, tokio, broadcast, real-time, gateway]

requires:
  - phase: 22-command-center-svelte
    plan: 01
    provides: Phase 22 setup and Svelte command center scaffolding
  - phase: 20-approval-management
    provides: approval_store and approval REST handlers
  - phase: 17-gateway-foundation
    provides: AgentManager, create_router, REST gateway

provides:
  - /ws WebSocket endpoint on the agentix gateway for real-time event broadcasting
  - GatewayEvent enum covering agent_status, run_started, run_completed, approval_requested, approval_decided, cost_update
  - EventBroadcaster struct using tokio::sync::broadcast for fan-out to all WS clients
  - GatewayRouter builder with with_broadcaster() method
  - REST handlers (run_agent, approve_request, deny_request) now broadcast events

affects:
  - 22-03 (SvelteKit command center frontend — consumes /ws endpoint)
  - 22-04 (agent dashboard — relies on WS for live status)
  - future gateway plans (EventBroadcaster injected at Gateway::start)

tech-stack:
  added:
    - tokio-tungstenite 0.24 (dev-dep, WebSocket test client)
    - axum ws feature (WebSocket upgrade support in handlers)
  patterns:
    - GatewayRouter builder pattern: create_router() -> GatewayRouter, .with_broadcaster() -> Router
    - EventBroadcaster via axum Extension layer for optional use in REST handlers
    - TDD: RED test file committed before implementation, GREEN on second commit

key-files:
  created:
    - crates/agentix-runtime/src/gateway/websocket.rs
    - crates/agentix-runtime/tests/websocket_test.rs
  modified:
    - crates/agentix-runtime/src/gateway/api.rs
    - crates/agentix-runtime/src/gateway/mod.rs
    - crates/agentix-runtime/src/gateway/router.rs
    - crates/agentix-runtime/tests/gateway_test.rs
    - crates/agentix-runtime/tests/triggers_test.rs
    - docs/api/gateway-http-api.md
    - Cargo.toml (workspace axum ws feature)
    - crates/agentix-runtime/Cargo.toml (tokio-tungstenite dev-dep)

key-decisions:
  - "GatewayRouter builder pattern: create_router() returns GatewayRouter (not Router) to enable with_broadcaster() chain"
  - "EventBroadcaster added as axum Extension layer so REST handlers can optionally broadcast without changing state type"
  - "ws_handler uses State<Arc<EventBroadcaster>> via merged sub-router with its own state"
  - "AgentManager::new() already returns Arc<Self> — tests that wrapped in Arc::new() had double-Arc bug, fixed"

patterns-established:
  - "Pattern 1: Builder pattern for Router extension — create_router().with_broadcaster() returns fully-configured Router"
  - "Pattern 2: Optional broadcaster in handlers — Option<Extension<Arc<EventBroadcaster>>> allows REST handlers to broadcast without failing when broadcaster absent"
  - "Pattern 3: Fan-out WebSocket via tokio broadcast — EventBroadcaster.subscribe() per client, send() from producers"

requirements-completed: [CMD-09]

duration: 13min
completed: 2026-03-13
---

# Phase 22 Plan 02: WebSocket Backend Summary

**Real-time /ws endpoint on agentix gateway with EventBroadcaster fan-out — broadcasts RunStarted, ApprovalDecided events from REST handlers to all WebSocket clients**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-13T07:02:42Z
- **Completed:** 2026-03-13T07:16:00Z
- **Tasks:** 2 (TDD: RED + GREEN)
- **Files modified:** 8 files + 2 created

## Accomplishments
- WebSocket endpoint at `/ws` accepts upgrade, sends `Connected` message, fans-out all GatewayEvents
- `EventBroadcaster` wraps `tokio::sync::broadcast` channel (capacity 256), thread-safe, clone-safe via Arc
- `GatewayEvent` enum with `#[serde(tag = "type", rename_all = "snake_case")]` for typed JSON frames
- `run_agent`, `approve_request`, `deny_request` handlers broadcast events when broadcaster is present
- 4 new WebSocket integration tests using real TCP server + tokio-tungstenite client

## Task Commits

Each task was committed atomically:

1. **Task 1: Write failing tests for WebSocket event broadcasting** - `fdf236e` (test)
2. **Task 2: Implement WebSocket handler with event broadcasting** - `1d94b27` (feat)

## Files Created/Modified
- `crates/agentix-runtime/src/gateway/websocket.rs` (213 lines) — GatewayEvent, EventBroadcaster, ws_handler, handle_socket
- `crates/agentix-runtime/tests/websocket_test.rs` (212 lines) — 4 integration tests: connection, agent_status, run_event, fan-out
- `crates/agentix-runtime/src/gateway/api.rs` — Added GatewayRouter builder, /ws route, broadcaster Extension, events in approve/deny/run handlers
- `crates/agentix-runtime/src/gateway/mod.rs` — Export websocket module, EventBroadcaster, GatewayEvent
- `crates/agentix-runtime/src/gateway/router.rs` — Create broadcaster on startup, wire with_broadcaster()
- `crates/agentix-runtime/tests/gateway_test.rs` — Updated create_router() callers to use .into()
- `crates/agentix-runtime/tests/triggers_test.rs` — Fixed Arc::new(AgentManager::new()) double-Arc bug
- `docs/api/gateway-http-api.md` — Added /ws WebSocket section with event reference table and JS client example
- `Cargo.toml` — Added ws feature to axum workspace dependency
- `crates/agentix-runtime/Cargo.toml` — Added tokio-tungstenite dev-dependency

## Decisions Made
- Used `GatewayRouter` builder pattern (not `impl IntoMakeService`) to enable `with_broadcaster()` chaining without changing existing test signatures
- EventBroadcaster attached as `axum::Extension` so ALL route handlers (regardless of state type) can optionally emit events
- WebSocket sub-router uses its own `State<Arc<EventBroadcaster>>` separate from main `State<Arc<AgentManager>>` state — merged via `Router::merge()`
- Selected `tokio::sync::broadcast` (not `tokio::sync::watch`) because fan-out is needed: each subscriber gets every event

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed double-Arc bug in existing tests**
- **Found during:** Task 2 (GREEN phase - running gateway_test.rs and triggers_test.rs)
- **Issue:** `AgentManager::new(None)` already returns `Arc<AgentManager>`, but test code had `Arc::new(AgentManager::new(None))` creating `Arc<Arc<AgentManager>>`
- **Fix:** Removed the outer `Arc::new()` wrapping in gateway_test.rs and triggers_test.rs
- **Files modified:** tests/gateway_test.rs, tests/triggers_test.rs
- **Verification:** All 197 tests pass
- **Committed in:** 1d94b27 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed doctest in api.rs with same double-Arc bug**
- **Found during:** Task 2 (final test run — doctest failure)
- **Issue:** Doctest example used `Arc::new(AgentManager::new(None))`, causing compile error
- **Fix:** Removed outer `Arc::new()` in doctest
- **Files modified:** src/gateway/api.rs
- **Verification:** All doctests pass (197 total)
- **Committed in:** 1d94b27 (Task 2 commit)

**3. [Rule 1 - Bug] SinkExt import unused (tungstenite API difference)**
- **Found during:** Task 1 (RED verification - compile error)
- **Issue:** Test used `use futures::{SinkExt, StreamExt}` and `ws_stream.close(None)` but tokio-tungstenite's close takes a `CloseFrame` argument not compatible with `SinkExt::close()`
- **Fix:** Removed `SinkExt` import, replaced `.close(None)` with `drop()` for graceful teardown
- **Files modified:** tests/websocket_test.rs
- **Verification:** WebSocket tests compile and pass
- **Committed in:** 1d94b27 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 bugs)
**Impact on plan:** All fixes were correctness issues in pre-existing tests or test code. No scope creep. No architectural changes needed.

## Issues Encountered
- `GatewayRouter` with `Deref<Target=Router>` couldn't satisfy tower's `ServiceExt::oneshot` (takes `self` by value, can't move out of Deref) — resolved by having tests explicitly call `.into()` for Router conversion
- Two versions of tokio-tungstenite (0.21 and 0.24) in the workspace — cargo resolved to 0.24 which is correct

## Next Phase Readiness
- `/ws` WebSocket endpoint is live at the gateway server
- `EventBroadcaster` is exported from `agentix_runtime::gateway` and ready for frontend consumption
- Phase 22-03 (SvelteKit command center) can connect to `ws://host:port/ws` immediately
- Future gateway plans can call `broadcaster.send(GatewayEvent::...)` for any new event types

---
*Phase: 22-command-center-svelte*
*Completed: 2026-03-13*
