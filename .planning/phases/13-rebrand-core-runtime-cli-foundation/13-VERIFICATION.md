---
phase: 13-rebrand-core-runtime-cli-foundation
verified: 2026-03-13T19:30:00Z
status: gaps_found
score: 3/5 success criteria verified
gaps:
  - truth: "agentix run <file> and agentix start <file> commands function (CLI-01, CLI-02)"
    status: failed
    reason: "Neither 'run' nor 'start' subcommands exist in crates/agentix/src/cli.rs Commands enum. The CLI has Gateway/Agents/Runs/Logs/Stop/Apply/Validate/Version/Init/Onboard but no Run or Start."
    artifacts:
      - path: "crates/agentix/src/cli.rs"
        issue: "Commands enum missing Run and Start variants"
    missing:
      - "Add Run { file: String } variant to Commands enum in cli.rs"
      - "Add Start { file: String } variant to Commands enum in cli.rs"
      - "Implement run command handler that loads agent and executes ReActEngine directly (no gateway needed)"
      - "Implement start command handler (daemon/background execution)"
      - "Wire both to main.rs dispatch"

  - truth: "cargo test --workspace passes (all integration tests compile and pass)"
    status: failed
    reason: "Two agentix-triggers integration test files fail to compile: workflow_tests.rs imports agentix_runtime::RuntimeOrchestrator which does not exist in agentix-runtime (the stub lives in agentix_triggers::handler, not exported from agentix_runtime). jira_platform_test.rs has struct field mismatches (JiraConfig fields in tests don't match current struct definition — tests reference fields like api_url, email, api_token but struct may have changed)."
    artifacts:
      - path: "crates/agentix-triggers/tests/workflow_tests.rs"
        issue: "Line 19: 'use agentix_runtime::RuntimeOrchestrator' — RuntimeOrchestrator is not exported from agentix-runtime crate, only defined as a stub in agentix_triggers::handler::mod.rs"
      - path: "crates/agentix-triggers/tests/jira_platform_test.rs"
        issue: "Struct literal fields (api_url, email, api_token, webhook_secret, bot_name, enable_comments, enable_transitions) do not match JiraConfig definition (available fields are base_url, enable_updates per compiler error E0560/E0609)"
    missing:
      - "Either export RuntimeOrchestrator from agentix-runtime lib.rs, or update workflow_tests.rs to import from agentix_triggers::handler"
      - "Reconcile JiraConfig struct fields in jira_platform_test.rs with actual JiraConfig definition in agentix-triggers"
      - "Run 'cargo test --workspace' to confirm zero test compilation failures"
---

# Phase 13: Rebrand + Core Runtime + CLI Foundation Verification Report

**Phase Goal:** Users can define agents as GitAgent-compatible directories (agent.yaml + SOUL.md), run them via `agentix gateway start`, see streaming ReAct loop output, and manage everything with the `agentix` CLI. Existing AOF flat YAML agents still load.
**Verified:** 2026-03-13T19:30:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `agentix init --name my-agent` creates valid agent directory and `agentix validate agents/my-agent` passes | VERIFIED | init.rs exists with DirectoryLoader validation after scaffold; quickstart/agents/hello-world/ proves the pattern; cli.rs has Init command wired |
| 2 | `agentix gateway start` loads agent directories, exposes REST API, streams ReAct loop output via SSE | VERIFIED | gateway/router.rs::Gateway::start() does exactly this; api.rs has all 10 REST endpoints; SSE streaming wired via BroadcastStream in api.rs; 11 integration tests pass |
| 3 | An existing AOF flat YAML agent (apiVersion: openagentix.dev/v1) loads and executes without modification | VERIFIED | FlatYamlLoader in agentix-core/src/agent.rs; AgentLoader::load() auto-detects format; backward compat confirmed in 21 agent_types tests |
| 4 | `agentix agents`, `agentix runs`, `agentix logs`, `agentix stop`, `agentix apply`, `agentix validate`, `agentix init`, `agentix onboard` all function | VERIFIED | All 8 commands present in cli.rs Commands enum with handlers in commands/; each wired in main.rs |
| 5 | `agentix run <file>` and `agentix start <file>` function (CLI-01, CLI-02) | FAILED | Neither `run` nor `start` subcommand exists in cli.rs Commands enum. REQUIREMENTS.md marks CLI-01 and CLI-02 as Complete for Phase 13 but they are absent from the codebase. |

**Score:** 3/5 success criteria verified (criteria 1-4 pass, criterion 5 fails; the phase goal also requires cargo test --workspace clean — integration tests broken in agentix-triggers)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/agentix-core/` | Renamed from aof-core | VERIFIED | Directory exists, Cargo.toml name = "agentix-core" |
| `crates/agentix-llm/` | Renamed from aof-llm | VERIFIED | Directory exists |
| `crates/agentix-runtime/` | Renamed from aof-runtime | VERIFIED | Directory exists |
| `crates/agentix/` | Renamed from aofctl, binary = "agentix" | VERIFIED | [[bin]] name = "agentix" in Cargo.toml |
| `crates/agentix-core/src/agent.rs` | AgentManifest, AgentDefinition, DirectoryLoader, FlatYamlLoader | VERIFIED | All types present and substantive (69.7K file) |
| `crates/agentix-core/src/config.rs` | WorkspaceConfig, WorkspaceSpec, WorkspaceDefaults, GatewayConfig | VERIFIED | Full implementation, no stubs |
| `crates/agentix-core/tests/agent_types_test.rs` | 21 TDD integration tests | VERIFIED | 21 tests present, lib tests pass (293 total) |
| `crates/agentix-runtime/src/executor/react_loop.rs` | ReActEngine, ToolExecutor trait, ReActConfig, ReActStep, RunResult, ReActEvent | VERIFIED | Full implementation with plan-act-observe-reflect loop |
| `crates/agentix-runtime/src/streaming.rs` | SseEncoder, TextFormatter, JsonFormatter, EventSender/EventReceiver | VERIFIED | Full implementation, 14 streaming tests pass |
| `crates/agentix-runtime/src/gateway/` | mod.rs, agent_manager.rs, router.rs, api.rs | VERIFIED | All 4 files present and substantive |
| `crates/agentix-runtime/tests/gateway_test.rs` | 11 HTTP integration tests | VERIFIED | Present; tests pass via cargo test --lib |
| `crates/agentix/src/commands/gateway.rs` | gateway start/status commands | VERIFIED | Full implementation, calls Gateway::start() |
| `crates/agentix/src/commands/agents.rs` | agents list command | VERIFIED | Full implementation with table output |
| `crates/agentix/src/commands/runs.rs` | runs history command | VERIFIED | Full implementation with table output |
| `crates/agentix/src/commands/logs.rs` | logs command | VERIFIED | Full implementation |
| `crates/agentix/src/commands/stop.rs` | stop command | VERIFIED | Full implementation |
| `crates/agentix/src/commands/apply.rs` | apply command | VERIFIED | Full implementation with 409 retry |
| `crates/agentix/src/commands/validate.rs` | validate command | VERIFIED | Full implementation with auto-detection |
| `crates/agentix/src/commands/init.rs` | init command | VERIFIED | Full implementation, interactive + non-interactive |
| `crates/agentix/src/commands/onboard.rs` | onboard command | VERIFIED | Full implementation |
| `crates/agentix/src/commands/` | **run.rs** (CLI-01) | MISSING | No run.rs file, no Run variant in cli.rs |
| `crates/agentix/src/commands/` | **start.rs** (CLI-02) | MISSING | No start.rs file, no Start variant in cli.rs |
| `docs/spec/agent-directory-structure.md` | GitAgent-compatible directory spec | VERIFIED | 15.4K file, covers full layout |
| `docs/spec/agent-yaml-v1.md` | Minimal agent.yaml manifest spec | VERIFIED | 10.1K file, rewritten to minimal format |
| `docs/spec/workspace-config.md` | agentix.yaml workspace config spec | VERIFIED | 12.4K file |
| `docs/api/gateway-http-api.md` | Gateway HTTP API reference | VERIFIED | Present |
| `docs/guides/quickstart.md` | Quickstart guide | VERIFIED | Present |
| `quickstart/agents/hello-world/agent.yaml` | Reference agent manifest | VERIFIED | Present |
| `quickstart/agents/hello-world/SOUL.md` | Reference agent identity | VERIFIED | Present |
| `quickstart/agentix.yaml` | Reference workspace config | VERIFIED | Present |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `cli.rs Commands::Gateway` | `gateway/router.rs Gateway::start()` | `gateway.rs::start()` | WIRED | gateway.rs calls `Gateway::start(gateway_config, workspace_config, agents_dir)` |
| `Gateway::start()` | `AgentManager::load_from_dir()` | `agent_manager.rs` | WIRED | Router creates AgentManager, calls load_from_dir before binding server |
| `AgentManager::start_run()` | `ReActEngine::run()` | `agent_manager.rs` spawned task | WIRED | Spawns tokio task that calls `engine.run(&definition, &input)` |
| `ReActEngine::run()` | SSE stream | `broadcast::Sender<ReActEvent>` | WIRED | Events emitted via `event_tx.send(ReActEvent::Step(...))` |
| `api.rs POST .../run` | SSE response | `BroadcastStream + axum::Sse` | WIRED | Returns `text/event-stream` with BroadcastStream receiver |
| `AgentLoader::load()` | `DirectoryLoader` or `FlatYamlLoader` | `detect_format()` | WIRED | Auto-detects format and routes to correct loader |
| `Commands::Run` | `ReActEngine` | run.rs (missing) | NOT WIRED | run command does not exist |
| `Commands::Start` | background execution | start.rs (missing) | NOT WIRED | start command does not exist |
| `workflow_tests.rs` | `agentix_runtime::RuntimeOrchestrator` | import | BROKEN | RuntimeOrchestrator not exported from agentix-runtime; test fails to compile |
| `jira_platform_test.rs` | `JiraConfig { api_url, email, ... }` | struct literal | BROKEN | Struct fields mismatch current JiraConfig definition |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CORE-01 | 13-04 | ReAct loop (plan-act-observe-reflect) | SATISFIED | react_loop.rs implements full loop with 9 passing tests |
| CORE-02 | 13-04 | Agent can call tools and receive results | SATISFIED | ToolExecutor trait + dispatch_tool() in react_loop.rs |
| CORE-03 | 13-05 | Agent supports streaming responses (SSE) | SATISFIED | SseEncoder, EventSender/Receiver, axum SSE in api.rs |
| SPEC-01 | 13-02, 13-03 | YAML with apiVersion: openagentix.dev/v1, kind: Agent | SATISFIED | FlatYamlLoader handles this format; 21 tests verify |
| SPEC-02 | 13-02, 13-03 | Agent spec supports model, mode, skills, tools, etc. | SATISFIED | AgentDefinition struct has all fields; workspace defaults cover mode/max_iterations |
| SPEC-03 | 13-01 | Backward compat with existing AOF flat YAML | SATISFIED | FlatYamlLoader + AgentLoader::detect_format() |
| SPEC-04 | 13-02, 13-03 | Agent spec supports namespace | SATISFIED | AgentDefinition has namespace field |
| SPEC-05 | 13-02, 13-03 | Agent spec supports version | SATISFIED | AgentManifest has version field |
| CLI-01 | 13-08 | `agentix run <file>` runs agent one-shot | BLOCKED | No Run variant in cli.rs Commands enum; no run command handler |
| CLI-02 | 13-08 | `agentix start <file>` starts agent as daemon | BLOCKED | No Start variant in cli.rs Commands enum; no start command handler |
| CLI-03 | 13-07 | `agentix agents` lists agents | SATISFIED | agents.rs handler, Agents variant in cli.rs |
| CLI-04 | 13-07 | `agentix runs` shows run history | SATISFIED | runs.rs handler, Runs variant in cli.rs |
| CLI-05 | 13-07 | `agentix logs <agent>` shows logs | SATISFIED | logs.rs handler, Logs variant in cli.rs |
| CLI-06 | 13-07 | `agentix stop <agent>` stops agent | SATISFIED | stop.rs handler, Stop variant in cli.rs |
| CLI-07 | 13-07 | `agentix apply -f <file>` creates/updates agent | SATISFIED | apply.rs handler, Apply variant in cli.rs |
| CLI-10 | 13-07 | `agentix serve` starts the server | PARTIAL | `agentix gateway start` works but no `agentix serve` alias. REQUIREMENTS.md says "agentix serve" — the CLI uses `agentix gateway start` instead. Note: phase goal text says "agentix gateway start" which is what's implemented. |
| CLI-11 | 13-08 | `agentix init` scaffolds new agent | SATISFIED | init.rs handler, Init variant in cli.rs |
| CLI-12 | 13-07 | `agentix validate <file>` validates agent YAML | SATISFIED | validate.rs handler, Validate variant in cli.rs |

**Note on CLI-10:** REQUIREMENTS.md defines `agentix serve` but the implementation uses `agentix gateway start`. The phase goal text (ROADMAP.md) specifies `agentix gateway start` as the command. The REQUIREMENTS.md entry is inconsistent with the phase design. This should be reconciled — either add a `serve` alias or update REQUIREMENTS.md.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `crates/agentix-triggers/src/handler/mod.rs` | RuntimeOrchestrator, Runtime, TaskHandle — all `pub` stubs that return errors or no-ops | Warning | Does not prevent phase goal (gateway/CLI work), but triggers integration tests broken |
| `crates/agentix-triggers/tests/workflow_tests.rs` | Imports `agentix_runtime::RuntimeOrchestrator` which does not exist in that crate | Blocker | Prevents `cargo test --workspace` from compiling; 8 test compile errors |
| `crates/agentix-triggers/tests/jira_platform_test.rs` | JiraConfig struct literals use fields that don't match current struct definition | Blocker | Prevents `cargo test --workspace` from compiling; 5 test compile errors |

---

### Human Verification Required

#### 1. agentix gateway start — end-to-end SSE streaming

**Test:** Run `agentix onboard --non-interactive` then `agentix gateway start`, then in another terminal send a request to POST /api/v1/agents/hello-world/run with a prompt and observe the SSE stream.
**Expected:** Terminal shows ReAct loop steps (plan/act/observe) streaming in real time, ending with Complete event.
**Why human:** Live LLM call + SSE stream render cannot be verified by grep.

#### 2. agentix validate on flat YAML backward compat

**Test:** Create a flat YAML file with `apiVersion: openagentix.dev/v1 / kind: Agent` and run `agentix validate` on it.
**Expected:** Exits 0 with "valid" message.
**Why human:** Requires a real file on disk + binary execution.

---

### Gaps Summary

Two gaps block full phase goal achievement:

**Gap 1: CLI-01 and CLI-02 missing** — `agentix run <file>` and `agentix start <file>` are listed as Complete in REQUIREMENTS.md and were assigned to plan 13-08 (which marked them completed), but neither command exists in the compiled binary. The cli.rs Commands enum has no `Run` or `Start` variants, and no command handler files exist. The SUMMARY for 13-08 claims `requirements-completed: [CLI-01, CLI-02, CLI-11]` but only CLI-11 (init) was actually implemented.

**Gap 2: cargo test --workspace broken** — Two integration test files in agentix-triggers fail to compile due to: (a) `workflow_tests.rs` importing a type from the wrong crate (`agentix_runtime::RuntimeOrchestrator` vs `agentix_triggers::handler::RuntimeOrchestrator`), and (b) `jira_platform_test.rs` using struct fields that no longer match the JiraConfig definition. The summary for 13-09 claimed "293 lib tests pass" which is true for `--lib` only, but `--workspace` (which includes integration tests) fails. This was not caught because the 13-09 verification only ran `cargo test --workspace --lib`.

These two gaps are independent and can be fixed in a single targeted plan.

---

*Verified: 2026-03-13T19:30:00Z*
*Verifier: Claude (gsd-verifier)*
