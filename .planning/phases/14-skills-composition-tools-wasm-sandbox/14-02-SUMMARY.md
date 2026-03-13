---
plan: 14-02
phase: 14
title: CLI Tool Executor
status: completed
wave: 2
commit: 19e3305
---

# Plan 14-02 Summary: CLI Tool Executor

## What Was Built

Replaced the blocking `ShellToolExecutor` stub with a proper async `CliToolExecutor` with timeout enforcement, moved into its own `crates/agentix-runtime/src/tools/` module.

## Tasks Completed

### Task 1: Write Failing Tests for CliToolExecutor (TDD — RED)
- Created `crates/agentix-runtime/tests/cli_tool_test.rs` with 8 tests
- Confirmed RED: compile error because `agentix_runtime::tools::CliToolExecutor` didn't exist

### Task 2: Implement CliToolExecutor (TDD — GREEN)
- Created `crates/agentix-runtime/src/tools/mod.rs` and `cli_executor.rs`
- `CliToolExecutor::new()` — 30s default timeout
- `CliToolExecutor::with_timeout(secs)` — custom timeout
- Async subprocess execution via `tokio::process::Command`
- Timeout enforcement via `tokio::time::timeout`
- `{{variable}}` template substitution in command string and args
- MCP tools return descriptive error pointing to plan 14-03
- All 8 tests pass (GREEN)

### Task 3: Wire CliToolExecutor into AgentManager
- Removed `ShellToolExecutor` struct, its `ToolExecutor` impl, and `substitute_templates` from `agent_manager.rs`
- Added `use crate::tools::CliToolExecutor;` import
- Updated `execute_run()` to use `CliToolExecutor::new()` instead of `ShellToolExecutor`
- Workspace build: 0 errors

### Task 4: Create docs/guides/tools.md and Verify Build
- Created `docs/guides/tools.md` covering: tool types, defining tools, CLI/shell tool examples, template substitution, built-in tools, MCP servers, timeout, error handling
- All 297 workspace lib tests pass
- cli_tool_test: 8 tests pass
- Release build: Finished (0 errors)
- ShellToolExecutor completely removed from codebase
- CliToolExecutor confirmed in agent_manager.rs

## Key Decisions
- All commands executed via `sh -c` to support pipes, redirects, env vars
- Tool failures return `Err(String)` — fed back as ReAct loop observations, never abort the loop
- timeout test uses `sleep 10` killed after 1s — confirms process is actually terminated
- Template substitution: unknown variables left as-is (not substituted, not errored)

## Requirements Satisfied
- TOOL-01: Agents can call CLI tools (kubectl, aws, psql, etc.) during ReAct loop ✓
- TOOL-03: User-defined shell command tools with {{var}} substitution work ✓
