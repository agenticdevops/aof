# Local Testing Guide - Rapid Feedback Loop

Avoid the slow release-download-install cycle. Test directly from built binaries.

## Quick Build & Test (2-3 minutes instead of 10+)

```bash
# 1. Build locally
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)"

# 2. Test directly from binary (NO install needed)
./target/release/aofctl run agent testframework/smoke-test-agent.yaml <<< "quit"

# If it works: ✅
# If it fails: Fix code → goto step 1
```

## Full Workflow for Rapid Iteration

```bash
# Make your code change
nano crates/aof-runtime/src/executor/runtime.rs

# Build (incremental, usually <30s for small changes)
cargo build --release

# Test locally
./target/release/aofctl run agent testframework/smoke-test-agent.yaml <<< "quit"

# Repeat above 3 steps until working
```

## Testing Specific Components

### Test MCP Server Initialization
```bash
# Direct test with proper init params
npx -y @modelcontextprotocol/server-everything stdio --roots /tmp --roots / <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}
EOF
```

### Test MCP Tool Listing
```bash
npx -y @modelcontextprotocol/server-everything stdio --roots /tmp --roots / <<'EOF'
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
EOF
```

### Test Smoke Test MCP Server
```bash
# Build it first
cargo build --release -p smoke-test-mcp

# Test directly
./scripts/test-smoke-mcp.sh
```

## Binary Locations After Build

All built binaries are in `target/release/`:

```
target/release/
├── aofctl                    # Main CLI binary
├── smoke-test-mcp           # Local MCP server for testing
└── [other build artifacts]
```

No installation needed - use these binaries directly with full paths.

## Quick Config for Testing

Use `testframework/smoke-test-agent.yaml`:

```yaml
name: smoke-test-agent
model: gemini-2.0-flash
provider: google
system_prompt: "Your instructions"
tools:
  - echo
  - add
  - get_system_info
max_iterations: 10
temperature: 0.7
max_tokens: 1000
```

## Common Test Scenarios

### Scenario 1: Testing MCP Initialization
```bash
cargo build --release && \
./target/release/aofctl run agent testframework/smoke-test-agent.yaml <<< "quit" 2>&1 | head -20
```

### Scenario 2: Testing Tool Execution
```bash
# With API key set
OPENAI_API_KEY=sk-... ./target/release/aofctl run agent testframework/smoke-test-agent.yaml
```

### Scenario 3: Testing Error Handling
```bash
# Test with invalid config
./target/release/aofctl run agent /nonexistent/config.yaml 2>&1
```

## Performance Comparison

| Operation | Old Way (Release) | New Way (Local Test) |
|-----------|---|---|
| Edit code | 1 min | 1 min |
| Build | 3 min | 2-3 min |
| Install | 2 min | **0 min** |
| Test | 1 min | 1 min |
| **Total** | **~7 min** | **~3-4 min** |

### Even Faster: Small Changes
For small targeted fixes:
```bash
# Incremental rebuild is usually <30 seconds
cargo build --release -p aofctl
./target/release/aofctl run agent testframework/smoke-test-agent.yaml
```

## When to Do a Release

Only commit and release when:
1. ✅ All local tests pass
2. ✅ Feature is complete
3. ✅ Ready to document

## Tips

1. **Keep a terminal window open** with your build ready to iterate
2. **Use incremental builds** - only modified crates recompile
3. **Test early and often** - don't wait until "done"
4. **Use local configs** - keep test configs in testframework/
5. **Check logs** - Add `RUST_LOG=debug` for more output

```bash
RUST_LOG=debug ./target/release/aofctl run agent testframework/smoke-test-agent.yaml
```

## Troubleshooting Local Tests

### Binary not found
```bash
ls -la target/release/aofctl  # Check it exists
cargo build --release         # Build if missing
```

### Config file not found
```bash
pwd                           # Check current directory
ls testframework/             # Should see smoke-test-agent.yaml
```

### Ports in use
```bash
# Some tests might use ports, check for conflicts
lsof -i :8000  # Example port
kill -9 <PID>  # Kill if needed
```

## Next Steps After Local Testing

Once working locally:

```bash
# Commit changes
git add -A
git commit -m "feat: Fix issue description"

# Push to dev branch
git push origin dev

# Create release (when ready)
git tag v0.X.X
git push origin v0.X.X
gh release create v0.X.X --title "Version 0.X.X"
```

This workflow eliminates the install step entirely, saving you 2+ minutes per test cycle!
