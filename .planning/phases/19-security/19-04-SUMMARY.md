# Plan 19-04 Summary: Security Runtime Wiring

## Status: Complete

## What was built

Wired all Phase 19 security components into the runtime execution path:

1. **ReActConfig security fields** - Added `ssrf_guard`, `secret_redactor`, `audit_store`, and `agent_name` to ReActConfig struct for per-run security context.

2. **SSRF guard in ReAct loop** - dispatch_tool() checks URLs in tool arguments against SsrfGuard before execution. Blocked requests return SSRF protection messages as observations (not fatal errors).

3. **Secret redaction in tool output** - Tool output passes through SecretRedactor before being fed back to the LLM conversation, replacing known secret values with `[REDACTED:<name>]` markers.

4. **Audit logging in ReAct loop** - ToolCall and SecurityViolation events are logged to AuditStore with timing, outcome, and details. Audit failures are logged as warnings and never interrupt execution.

5. **AgentManager security wiring** - Reads SecurityConfig from workspace spec, constructs SsrfGuard with allowed endpoints, and passes security components into ReActConfig for each run.

6. **WorkspaceSpec security field** - Added `security: Option<SecurityConfig>` to WorkspaceSpec for workspace-level security configuration.

7. **ToolEntry capabilities field** - Added `capabilities: Option<Vec<serde_json::Value>>` to ToolEntry for WASM capability enforcement, with YAML parsing via `as_sequence()`.

## Files changed

- `crates/agentix-runtime/src/executor/react_loop.rs` - Security fields in ReActConfig, SSRF/audit/redaction in dispatch_tool
- `crates/agentix-runtime/src/gateway/agent_manager.rs` - Audit store initialization, SecurityConfig wiring
- `crates/agentix-core/src/config.rs` - SecurityConfig in WorkspaceSpec
- `crates/agentix-core/src/agent.rs` - capabilities field in ToolEntry
- `crates/agentix-core/src/secret_store.rs` - Debug impl for SecretRedactor
- `crates/agentix-runtime/src/audit_store.rs` - Debug impl for AuditStore
- `crates/agentix-runtime/tests/security_integration_test.rs` - 9 integration tests

## Test results

9 security integration tests covering SSRF blocking, secret redaction, audit logging, WASM capability enforcement, and combined security scenarios. All 130 agentix-runtime tests pass.
