#!/bin/bash
# kubectl-ai MCP Server Wrapper
# This script ensures kubectl-ai runs with the correct environment

# Set PATH to include common binary locations
export PATH="/usr/local/bin:/usr/bin:/bin:$PATH"

# Run kubectl-ai in MCP server mode
exec /usr/local/bin/kubectl-ai --mcp-server "$@"
