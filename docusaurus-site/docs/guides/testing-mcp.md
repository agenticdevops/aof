# Smoke Test MCP Server

A minimal MCP (Model Context Protocol) implementation for testing and development without external dependencies.

## Overview

The smoke-test MCP server provides a lightweight, self-contained MCP implementation useful for:
- **Local Development**: Test MCP integration without API keys
- **CI/CD Smoke Testing**: Quick validation of MCP protocol compliance
- **Reference Implementation**: Example of MCP server implementation
- **Testing Agent Tools**: Validate tool execution flow

## Building

```bash
# Build the smoke-test MCP server
cargo build --release -p smoke-test-mcp

# Binary location: target/release/smoke-test-mcp
```

## Running Tests

```bash
# Run comprehensive smoke test suite
./scripts/test-smoke-mcp.sh
```

Output includes:
- ✅ MCP protocol initialization test
- ✅ Tool echo test (connectivity)
- ✅ Tool add test (parameter passing)
- ✅ Tool listing validation
- ✅ Integration configuration check

## Available Tools

### 1. Echo Tool
Tests basic connectivity and message passing.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {"message": "Hello MCP"}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "Hello MCP",
    "timestamp": "2024-12-12T17:00:00+00:00"
  }
}
```

### 2. Add Tool
Tests numeric parameter passing and calculation.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "add",
    "arguments": {"a": 5, "b": 3}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "result": 8,
    "inputs": {"a": 5, "b": 3}
  }
}
```

### 3. Get System Info Tool
Retrieves basic system information.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_system_info",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "os": "macos",
    "arch": "aarch64",
    "server": "smoke-test-mcp",
    "version": "0.1.0",
    "timestamp": "2024-12-12T17:00:00+00:00"
  }
}
```

## Using with AOF Runtime

### Agent Configuration

Create or use `testframework/smoke-test-agent.yaml`:

```yaml
name: smoke-test-agent
model: gemini-2.0-flash
provider: google
instructions: |
  You are a helpful assistant. You have access to tools to echo messages,
  do simple math, and get system information.

tools:
  - echo
  - add
  - get_system_info

max_iterations: 10
temperature: 0.7
max_tokens: 1000
```

### Using with Runtime

To use smoke-test-mcp instead of server-everything, modify the runtime's `create_tool_executor()`:

```rust
// In crates/aof-runtime/src/executor/runtime.rs
let mcp_client = McpClientBuilder::new()
    .stdio(
        "./target/release/smoke-test-mcp",  // Use local binary
        vec![],
    )
    .build()?;
```

## MCP Protocol Support

The smoke-test MCP server implements the MCP protocol with:

### Supported Methods
- `initialize` - Client initialization with protocol negotiation
- `tools/list` - List available tools with descriptions
- `tools/call` - Execute tools with parameters

### Protocol Version
- Supports MCP v2024-11-05

### Capabilities
- Tools execution
- Proper error handling
- JSON-RPC 2.0 compliance

## Use Cases

### 1. CI/CD Smoke Testing
```bash
# Quick validation without network/API calls
./scripts/test-smoke-mcp.sh
# Exits with code 0 on success
```

### 2. Local Development
```bash
# Test agent tool integration
cargo run --release -- run agent testframework/smoke-test-agent.yaml
```

### 3. Protocol Validation
```bash
# Manual protocol testing
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  ./target/release/smoke-test-mcp
```

## Performance

- **Startup**: &lt;100ms
- **Tool execution**: &lt;1ms per call
- **Memory usage**: ~2MB
- **No external dependencies**: All operations in-process

## Debugging

Enable debug logging:
```bash
RUST_LOG=debug ./target/release/smoke-test-mcp
```

Monitor logs from stderr (separate from stdout which contains MCP responses):
```bash
./target/release/smoke-test-mcp 2> debug.log
```

## Architecture

The server is implemented as a single-threaded event loop:
1. Read JSON-RPC requests from stdin
2. Route to appropriate handler
3. Execute tool or protocol operation
4. Send JSON-RPC response to stdout

This design ensures:
- Deterministic behavior for testing
- Easy to understand and extend
- Compatible with stdio-based transports

## Future Enhancements

Potential additions for more comprehensive testing:
- File operations (read, write, list)
- Time-based operations
- Error simulation
- Rate limiting simulation
- Concurrency testing
