# WASM Tools (Sandboxed Execution)

OpenAgentiX can run WebAssembly (WASM) tools in an isolated sandbox with
capability-based permissions. Untrusted or third-party tools run in isolation —
they cannot access the network, filesystem, or secrets unless they explicitly
declare those capabilities.

## Why WASM?

- **Isolation**: WASM tools cannot access host resources without explicit permission
- **Safety**: Undeclared capability access is blocked at the sandbox boundary
- **Portability**: WASM modules run the same on any platform (Linux, macOS, Windows)
- **Third-party tools**: Safely run tools from untrusted sources

## Declaring a WASM Tool

Add a `type: wasm` tool definition to your agent's `tools/` directory:

```yaml
# agents/my-agent/tools/analyzer.yaml
name: analyze_text
type: wasm
wasm_path: tools/analyzer.wasm
description: Analyze text sentiment and extract key phrases
capabilities:
  - secrets          # Can read named secrets
```

The WASM module file is loaded from `wasm_path` relative to the agent directory.

## Capabilities

Every WASM tool starts with zero capabilities. Declare only what the tool needs:

| Capability | YAML | What it grants |
|------------|------|----------------|
| Network | `- network` | Outbound HTTP/TCP to any endpoint |
| Filesystem | `- filesystem` | Read/write local filesystem |
| Secrets | `- secrets` | Read named secrets from env |
| HTTP endpoints | `- http_endpoints: [url1, url2]` | Specific URLs only (stricter than network) |
| Resource limits | `- resource_limits: {memory_mb: 128, timeout_secs: 60}` | Override defaults |

### Default resource limits
- Memory: 64 MB
- Timeout: 30 seconds

## WASM Module Interface

Your WASM module must export:
- `memory` — a WebAssembly memory export
- `run(input_offset: i32, input_len: i32) -> i32` — entry point function

The runtime:
1. Serializes tool input to JSON and writes it to WASM memory at offset 1024
2. Calls `run(1024, input_len)` with the input offset and length
3. Reads the output string from memory at offset 0 with the returned length

Example in Rust (compiled to WASM):
```rust
#[no_mangle]
pub extern "C" fn run(input_offset: i32, input_len: i32) -> i32 {
    // Read input JSON from memory
    let input_slice = unsafe {
        std::slice::from_raw_parts(input_offset as *const u8, input_len as usize)
    };
    let input: serde_json::Value = serde_json::from_slice(input_slice).unwrap_or_default();

    // Do work...
    let output = format!("Processed: {}", input);
    let output_bytes = output.as_bytes();

    // Write output to memory at offset 0
    unsafe {
        std::ptr::copy_nonoverlapping(
            output_bytes.as_ptr(),
            0 as *mut u8,
            output_bytes.len(),
        );
    }
    output_bytes.len() as i32
}
```

Compile with: `cargo build --target wasm32-unknown-unknown --release`

## Capability Violation Behavior

When a WASM tool attempts to access an undeclared capability:
1. The sandbox intercepts the host function call
2. Returns a `WasmViolation` error as the tool observation
3. The ReAct loop receives the violation as an observation and decides how to proceed

Example observation on violation:
```
Network access blocked: WASM tool attempted to reach 'https://example.com'
without 'network' capability. Declare capabilities: [network] in tools/analyzer.yaml
```

## Enabling WASM Support

WASM support is opt-in (feature-gated to minimize compile time):

```bash
# Build with WASM support
cargo build --release --features agentix-runtime/wasm
```

Or in your workspace `Cargo.toml`:
```toml
[dependencies]
agentix-runtime = { path = "...", features = ["wasm"] }
```
