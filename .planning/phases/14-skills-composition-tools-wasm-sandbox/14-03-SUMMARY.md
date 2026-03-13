---
plan: 14-03
phase: 14
title: MCP Tool Executor Integration
status: completed
wave: 3
commit: 59857eb
---

# Plan 14-03 Summary: MCP Tool Executor Integration

## What Was Built

Integrated the existing `agentix-mcp` crate into the ReAct loop executor via McpToolExecutor and CompositeToolExecutor.

## Tasks Completed

### Task 1: Inspect MCP API and Write Failing Tests (TDD — RED)
- Read agentix-mcp client.rs: `McpClient` has `initialize()`, `call_tool(name, input)`, `shutdown()`
- Read agentix-core mcp.rs: `McpServerConfig` uses flat fields (transport enum, command, args, env, endpoint) — not nested config struct
- Created `crates/agentix-runtime/tests/mcp_tool_test.rs` with 5 tests
- Confirmed RED: compile error because McpToolExecutor/CompositeToolExecutor didn't exist

### Task 2: Implement McpToolExecutor and CompositeToolExecutor (TDD — GREEN)
- Created `crates/agentix-runtime/src/tools/mcp_executor.rs`
- `McpToolExecutor::new(Vec<McpServerConfig>)` — keyed HashMap for server lookup
- `build_mcp_client(config)` — bridges McpServerConfig to McpClientBuilder
- `CompositeToolExecutor<C, M>` — generic over C: ToolExecutor and M: ToolExecutor
- Routes Cli/Shell → C executor, Mcp → M executor
- All 5 tests pass (GREEN)

### Task 3: Wire CompositeToolExecutor into AgentManager
- Updated import to `use crate::tools::{CliToolExecutor, CompositeToolExecutor, McpToolExecutor}`
- Updated `execute_run()` to build CompositeToolExecutor from agent's mcp_servers
- Added `mcp_servers: Vec<McpServerConfig>` to `AgentDefinition` struct
- Added `mcp_servers: Vec<McpServerConfig>` to `AgentManifest` struct (with `#[serde(default)]`)
- Updated DirectoryLoader to populate `mcp_servers: manifest.mcp_servers`
- Updated FlatYamlLoader to populate `mcp_servers: vec![]` (flat YAML doesn't use MCP in directory format)
- Fixed `crates/agentix/src/commands/init.rs` to include `mcp_servers: vec![]` in manifest construction
- Workspace build: 0 errors

### Task 4: Write docs/guides/mcp.md and Verify Build
- Created `docs/guides/mcp.md` documenting: transport types, agent.yaml config, tool reference YAML, flat YAML format, connection lifecycle, error handling, popular MCP servers
- All 297 workspace lib tests pass
- mcp_tool_test: 5 tests pass
- cli_tool_test: 8 tests pass (still green)
- Release build: Finished (EXIT:0)

## Key Decisions
- MCP connections are per-run (no caching) to avoid holding stdio processes for idle agents
- CompositeToolExecutor is generic (C, M) to enable future extension without dynamic dispatch overhead
- SSE/HTTP transports return an error pointing users to enable the feature flag (not a runtime panic)
- McpServerConfig uses flat fields (not nested transport config) — matches agentix-core/src/mcp.rs existing schema

## Requirements Satisfied
- TOOL-02: Agents with mcp_servers configured can call MCP server tools in the ReAct loop ✓
