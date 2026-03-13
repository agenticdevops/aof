---
plan: 14-04
phase: 14
title: WASM Sandbox with Capability Enforcement
status: completed
wave: 4
commit: 618f4b9
---

# Plan 14-04 Summary: WASM Sandbox with Capability Enforcement

## What Was Built

Implemented a WebAssembly sandbox with capability-based security using wasmtime, feature-gated for opt-in compilation.

## Tasks Completed

### Task 1: Add wasmtime dependency and write failing tests (TDD — RED)
- Added `[features] wasm = ["dep:wasmtime"]` and optional `wasmtime = "25"` to `crates/agentix-runtime/Cargo.toml`
- Added `wat = "1"` dev-dependency for WAT → WASM compilation in tests
- Created `crates/agentix-runtime/tests/wasm_sandbox_test.rs` with 7 tests (6 non-wasm, 1 wasm-feature)
- RED confirmed: compile error because `wasm_executor` module didn't exist

### Task 2: Implement WasmSandbox and capability enforcement (TDD — GREEN)
- Created `crates/agentix-runtime/src/tools/wasm_executor.rs`
- `WasmCapability` enum: Network, Filesystem, Secrets, HttpEndpoints(Vec<String>), ResourceLimits{memory_mb, timeout_secs}
- `WasmViolation` enum with descriptive `Display` messages
- `CapabilityManifest::check_capability()` returns `None` (allowed) or `Some(WasmViolation)` (blocked)
- `WasmSandbox::execute_bytes()` dispatches to wasmtime (with feature) or error (without)
- `WasmToolExecutor` implements `ToolExecutor` trait for `DirectoryToolType::Wasm`
- Added `DirectoryToolType::Wasm` variant to `agentix-core/src/agent.rs`
- Updated `cli_executor.rs` and `mcp_executor.rs` to handle new `Wasm` variant in match arms
- 7 tests pass with `--features wasm`; 6 non-wasm tests pass without feature

### Task 3: Wire WasmToolExecutor into CompositeToolExecutor
- Added `agents_dir: Option<PathBuf>` field to `AgentManager` for WASM file resolution
- Added `AgentManager::with_agents_dir()` constructor
- Updated `execute_run()` to create `WasmToolExecutor` from `agents_dir`
- Workspace build: 0 errors

### Task 4: Write docs/guides/wasm-tools.md and verify build
- Created `docs/guides/wasm-tools.md` covering: why WASM, declaring tools, capabilities table, module interface, violation behavior, enabling the feature
- All 297 workspace lib tests pass
- wasm_sandbox_test: 7 tests pass (with wasm feature)
- cli_tool_test: 8 tests pass
- mcp_tool_test: 5 tests pass
- Release build: EXIT:0

## Key Decisions
- Feature-gated: `cargo build` (no feature) compiles fast; opt-in with `--features agentix-runtime/wasm`
- `CapabilityManifest` uses `#[serde(default)]` for capabilities — parses cleanly from tool YAML
- `HttpEndpoints` does NOT grant general `Network` capability (stricter)
- `ResourceLimits` is always allowed as a checking operation (not a permission gate)
- wasmtime version 25 with `cranelift` feature for JIT compilation

## Requirements Satisfied
- TOOL-04: WASM tools run in isolation via wasmtime sandbox ✓
- TOOL-05: Capability manifest in tools/*.yaml enforces declared permissions ✓
