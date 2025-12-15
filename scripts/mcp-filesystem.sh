#!/bin/bash
# MCP Filesystem Server Wrapper
# This script ensures the correct environment is loaded for running npx

# Load NVM if available
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Run the MCP filesystem server
exec npx @modelcontextprotocol/server-filesystem "$@"
