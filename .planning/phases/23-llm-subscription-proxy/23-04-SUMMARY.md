---
phase: 23-llm-subscription-proxy
plan: 04
subsystem: auth
tags: [oauth, pkce, auth-service, axum, gateway, subscription-proxy]

# Dependency graph
requires:
  - phase: 23-llm-subscription-proxy/23-01
    provides: AuthProfilesStore, PkceState, TokenSet, profile types
  - phase: 23-llm-subscription-proxy/23-02
    provides: AuthService (load/store/refresh/remove profile operations)
  - phase: 23-llm-subscription-proxy/23-03
    provides: ProviderMode::Subscription enum, ProviderFactory subscription routing
provides:
  - OAuth REST API endpoints: start, callback, status, disconnect, token
  - AuthService embedded in AgentManager (auth_service: Arc<AuthService>)
  - pending_pkce DashMap in AgentManager for in-flight UI OAuth flows
  - create_provider_from_definition async with subscription token resolution
  - Gateway docs updated with OAuth auth section
affects: [23-05, 23-06, 23-07, command-center-ui, agentix-cli]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-path OAuth design: UI popup flow via gateway callback, CLI flow via loopback servers (per CONTEXT.md)"
    - "PKCE state keyed by state param in DashMap, 10-min expiry check on callback"
    - "AgentManager owns Arc<AuthService> — shared across all axum handlers via State"
    - "create_provider_from_definition is now async; resolves OAuth token before ProviderFactory::create"

key-files:
  created:
    - crates/agentix-runtime/tests/auth_api_test.rs
  modified:
    - crates/agentix-runtime/src/gateway/api.rs
    - crates/agentix-runtime/src/gateway/agent_manager.rs
    - docs/api/gateway-http-api.md

key-decisions:
  - "PKCE state stored in AgentManager::pending_pkce DashMap (not separate state struct) to reuse existing State<Arc<AgentManager>> axum state"
  - "create_provider_from_definition converted from sync (block_in_place) to fully async — cleaner, no nested runtime concerns"
  - "Subscription error message includes `agentix auth start <provider>` command for discoverability"

patterns-established:
  - "Auth endpoints follow existing pattern: normalize_provider() → match provider → call AuthService"
  - "Callback returns HTML with window.close() + postMessage for popup flow coordination"

requirements-completed: [SUB-01, SUB-02]

# Metrics
duration: 16min
completed: 2026-03-22
---

# Phase 23 Plan 04: Gateway OAuth Endpoints + AgentManager Wiring Summary

**OAuth REST endpoints (start/callback/status/disconnect/token) + AgentManager subscription token resolution using AuthService**

## Performance

- **Duration:** 16 min
- **Started:** 2026-03-22T05:24:36Z
- **Completed:** 2026-03-22T05:41:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Five OAuth endpoints registered: GET start, GET callback, GET status, DELETE disconnect, POST token
- AuthService and pending_pkce DashMap added to AgentManager (all three constructors updated)
- `create_provider_from_definition` converted to async and resolves subscription tokens from AuthService
- `ProviderMode::Subscription` check triggers OAuth token lookup before ProviderFactory::create call
- Clear error message when subscription mode configured but no token stored
- 7 new tests in auth_api_test.rs: status, normalization, disconnect, start flow, token paste, subscription precondition
- 207 total tests pass, 0 compile errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add OAuth REST endpoints to gateway API** - `71188d2` (feat)
2. **Task 2: Integrate AuthService into AgentManager for subscription model creation** - `ddaaa6e` (feat)

## Files Created/Modified
- `crates/agentix-runtime/src/gateway/api.rs` - Added 5 OAuth handlers + route registrations + routing:delete import
- `crates/agentix-runtime/src/gateway/agent_manager.rs` - Added auth_service + pending_pkce fields; async create_provider_from_definition with subscription mode
- `crates/agentix-runtime/tests/auth_api_test.rs` - 7 tests covering status, normalization, token store/disconnect, start, Anthropic, subscription precondition
- `docs/api/gateway-http-api.md` - Added OAuth/Auth section with all 5 endpoints documented

## Decisions Made
- Stored PKCE state in `AgentManager::pending_pkce` DashMap rather than a separate state struct — reuses the existing `State<Arc<AgentManager>>` axum state without adding a new state layer
- Converted `create_provider_from_definition` from sync (using `block_in_place`) to fully async — the function is only called from async context (`execute_run`), making it cleaner
- Subscription error message includes `agentix auth start <provider>` command so users know exactly how to fix it
- Gateway callback serves UI popup flows only; CLI loopback servers remain separate per CONTEXT.md decision

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered
- Test for subscription mode via `run_agent` was rejected by compiler because `WorkspaceMetadata` field names differ from assumptions. Resolved by testing the precondition (AuthService returns None with no stored profile) rather than exercising the full private `execute_run` pathway.

## Next Phase Readiness
- OAuth gateway endpoints fully wired; Command Center can call /api/v1/auth/openai/start + /callback popup flow
- CLI auth commands (plan 05) can use the same AuthService and will target loopback servers per CONTEXT.md
- AgentManager subscription token resolution active; plan 06 can test end-to-end agent runs in subscription mode

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
