---
phase: 13
plan: 06
subsystem: agentix-runtime/gateway
tags: [gateway, http-api, axum, sse, agent-loading, react-loop]
dependency_graph:
  requires: [13-04, 13-05]
  provides: [gateway-http-api, agent-directory-loading, sse-streaming, run-management]
  affects: [agentix-runtime, agentix-core, CLI-gateway-start, CLI-apply]
tech_stack:
  added:
    - axum 0.7 (HTTP server)
    - tower-http 0.6 cors layer
    - tokio-stream 0.1 with sync feature (BroadcastStream)
    - http-body-util 0.1 (test response body reading)
    - tower 0.4 util feature (test ServiceExt::oneshot)
    - anyhow 1.0 (error context)
  patterns:
    - DashMap for lock-free concurrent agent/run state
    - tokio broadcast channel for SSE fan-out
    - oneshot cancel channel for run cancellation
    - block_in_place for async provider factory in sync context
    - ServiceExt::oneshot for in-process axum testing (no real server)
key_files:
  created:
    - crates/agentix-runtime/src/gateway/mod.rs
    - crates/agentix-runtime/src/gateway/agent_manager.rs
    - crates/agentix-runtime/src/gateway/router.rs
    - crates/agentix-runtime/src/gateway/api.rs
    - crates/agentix-runtime/tests/gateway_test.rs
    - docs/api/gateway-http-api.md
  modified:
    - crates/agentix-runtime/src/lib.rs (pub mod gateway)
    - crates/agentix-runtime/src/streaming.rs (encode_data, encode_event_name methods)
    - crates/agentix-runtime/Cargo.toml (axum, tower-http, tokio-stream, http-body-util, anyhow, tower dev-dep)
    - crates/agentix-core/src/agent.rs (FlatYamlLoader::load_from_str)
    - Cargo.toml (axum, tokio-stream sync feature, http-body-util, tower util feature)
decisions:
  - AgentManager uses DashMap for lock-free concurrent state across axum handlers
  - ShellToolExecutor uses block_in_place + block_on for ProviderFactory (sync vs async boundary)
  - AgentixError::Runtime used for gateway-specific errors (no Other variant in AgentixError)
  - FlatYamlLoader::load_from_str added to agentix-core so gateway can parse inline YAML from API body
  - SseEncoder::encode_data and encode_event_name added as separate methods for axum SSE event construction
  - Box<dyn Model> -> Box<dyn Model + Send + Sync> coercion is safe because Model trait requires Send + Sync
metrics:
  duration: 13 minutes
  completed: 2026-03-12
  tasks: 2
  files_created: 7
  files_modified: 5
---

# Phase 13 Plan 06: Gateway HTTP Server Summary

Gateway HTTP server implementation connecting ReAct loop + SSE streaming + agent directory loading into the core OpenAgentiX runtime service.

## What Was Built

### Task 1: AgentManager + Directory Loader Integration

The `AgentManager` is the shared state object for all gateway HTTP handlers. It uses `DashMap` for lock-free concurrent access and manages:

- **Agent loading** — `load_from_dir()` scans a directory, uses `AgentLoader` to handle both GitAgent directory format (subdirectories with `agent.yaml`) and flat YAML backward-compat files. Invalid agents log warnings without aborting.
- **Agent registration** — `register_from_yaml()` / `update_from_yaml()` parse inline YAML from API requests using the new `FlatYamlLoader::load_from_str()` method.
- **Run lifecycle** — `start_run()` spawns a tokio task that races the ReAct engine against a cancellation channel. Events fan-out via a broadcast channel.
- **ShellToolExecutor** — implements `ToolExecutor` for shell/cli tools with `{{var}}` template substitution. MCP tools return a stub error pointing to Phase 14.

`Gateway::start()` in router.rs is the entry point — creates the manager, scans agents_dir, prints the loaded agent list, binds the axum server.

### Task 2: HTTP API + Integration Tests

All REST endpoints are implemented in api.rs using axum's `Router`:

| Endpoint | Description |
|---------|-------------|
| `GET /healthz` | Version + status check |
| `GET /api/v1/agents` | List all agents |
| `POST /api/v1/agents` | Register from inline YAML |
| `GET /api/v1/agents/:name` | Get agent summary |
| `PUT /api/v1/agents/:name` | Update agent |
| `POST /api/v1/agents/:name/run` | Run agent (SSE stream) |
| `GET /api/v1/agents/:name/runs` | List runs |
| `GET /api/v1/agents/:name/runs/:run_id` | Get run details |
| `GET /api/v1/agents/:name/runs/:run_id/logs` | Get run logs |
| `DELETE /api/v1/agents/:name/runs/:run_id` | Stop run |

The run endpoint streams `ReActEvent`s as SSE using `BroadcastStream` + axum's `Sse::new()`. All error responses follow `{"error":"...","field":"..."}` format.

**11 integration tests pass** using `ServiceExt::oneshot` (in-process testing, no real server or LLM needed):
- Health endpoint returns 200 with `status:ok`
- Empty agent list returns `[]`
- Register + list agent
- Invalid YAML → 400
- Duplicate name → 409
- Update agent → 200
- Update nonexistent → 404
- Get nonexistent → 404
- Run nonexistent → 404
- Stop nonexistent run → 404
- Run registered agent returns `text/event-stream` content type

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AgentixError::Other does not exist**
- **Found during:** Task 1 compilation
- **Issue:** Plan spec used `AgentixError::Other(anyhow::anyhow!(...))` but `AgentixError` enum has no `Other` variant
- **Fix:** Used `AgentixError::Runtime(format!(...))` for gateway runtime errors, `AgentixError::Config(format!(...))` for configuration errors
- **Files modified:** `gateway/agent_manager.rs`, `gateway/router.rs`

**2. [Rule 2 - Missing] FlatYamlLoader::load_from_str not available**
- **Found during:** Task 1 — `register_from_yaml` API handler receives YAML as a string, not a file path
- **Fix:** Added `FlatYamlLoader::load_from_str(content: &str)` to `agentix-core/src/agent.rs` and refactored `load(path)` to call it
- **Files modified:** `agentix-core/src/agent.rs`

**3. [Rule 2 - Missing] SseEncoder missing encode_data and encode_event_name methods**
- **Found during:** Task 2 — `api.rs` referenced `SseEncoder::encode_data` and `SseEncoder::encode_event_name` but they didn't exist
- **Fix:** Added both methods to `streaming.rs`; `encode_data` returns JSON string payload, `encode_event_name` returns SSE event name
- **Files modified:** `agentix-runtime/src/streaming.rs`

**4. [Rule 3 - Blocking] tower::ServiceExt requires util feature for testing**
- **Found during:** Task 2 — `tower::ServiceExt` is behind the `util` feature gate in tower 0.4
- **Fix:** Added `features = ["util"]` to workspace tower dependency
- **Files modified:** `Cargo.toml`

**5. [Rule 3 - Blocking] axum/tokio-stream not in workspace dependencies**
- **Found during:** Task 1 compilation
- **Fix:** Added `axum = { version = "0.7", features = ["macros"] }`, `tokio-stream = { version = "0.1", features = ["sync"] }`, `http-body-util = "0.1"` to workspace `Cargo.toml`
- **Files modified:** `Cargo.toml`, `agentix-runtime/Cargo.toml`

## Self-Check: PASSED

**Files verified:**
- FOUND: `crates/agentix-runtime/src/gateway/mod.rs`
- FOUND: `crates/agentix-runtime/src/gateway/agent_manager.rs`
- FOUND: `crates/agentix-runtime/src/gateway/router.rs`
- FOUND: `crates/agentix-runtime/src/gateway/api.rs`
- FOUND: `crates/agentix-runtime/tests/gateway_test.rs`
- FOUND: `docs/api/gateway-http-api.md`

**Commits verified:**
- `aa5753d` — feat(13-06): gateway agent manager + directory loader
- `f259223` — test(13-06): gateway HTTP API integration tests
