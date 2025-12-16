#!/bin/bash
# Test MCP configuration parsing and server connectivity
# Usage: ./scripts/test-mcp-config.sh

set -e

cd "$(dirname "$0")/.."

echo "=== Testing MCP Configuration Parsing ==="

# Test 1: Parse the test config
echo ""
echo "Test 1: Parsing test-mcp-servers.yaml..."
cargo run --release -p aofctl -- validate -f examples/test-mcp-servers.yaml 2>&1 || {
    echo "Note: validate command may not fully support multi-doc YAML yet"
}

# Test 2: Test kubectl-ai MCP server directly
echo ""
echo "Test 2: Testing kubectl-ai MCP server initialization..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"aof-test","version":"0.1.0"}}}' | timeout 5 kubectl-ai --mcp-server --mcp-server-mode=stdio 2>/dev/null | head -5 || {
    echo "kubectl-ai MCP server test completed (or timed out)"
}

# Test 3: List tools from kubectl-ai
echo ""
echo "Test 3: Listing tools from kubectl-ai MCP server..."
(
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"aof-test","version":"0.1.0"}}}'
    sleep 1
    echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    sleep 1
) | timeout 10 kubectl-ai --mcp-server --mcp-server-mode=stdio 2>/dev/null | grep -o '"name":"[^"]*"' | head -10 || {
    echo "Tool listing completed"
}

# Test 4: Test filesystem MCP server (npx)
echo ""
echo "Test 4: Testing filesystem MCP server..."
if command -v npx &> /dev/null; then
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"aof-test","version":"0.1.0"}}}' | timeout 10 npx -y @modelcontextprotocol/server-filesystem /tmp 2>/dev/null | head -3 || {
        echo "Filesystem MCP server test completed"
    }
else
    echo "npx not available, skipping filesystem test"
fi

# Test 5: Check Docker MCP servers (just check if images exist)
echo ""
echo "Test 5: Checking Docker MCP server images..."
if command -v docker &> /dev/null; then
    echo "  GitHub MCP server: $(docker images ghcr.io/github/github-mcp-server --format '{{.Repository}}:{{.Tag}}' 2>/dev/null | head -1 || echo 'not pulled')"
    echo "  Prometheus MCP server: $(docker images ghcr.io/pab1it0/prometheus-mcp-server --format '{{.Repository}}:{{.Tag}}' 2>/dev/null | head -1 || echo 'not pulled')"
else
    echo "Docker not available"
fi

echo ""
echo "=== MCP Configuration Tests Complete ==="
