# MCP (Model Context Protocol) Configuration Guide

**Last Updated**: December 16, 2025

AOF supports flexible MCP server configuration, allowing agents to connect to multiple MCP servers with different transports for accessing tools.

## Overview

MCP servers provide tools that agents can use to interact with external systems. AOF supports three transport types:

- **stdio**: Local process communication via stdin/stdout (default)
- **sse**: Server-Sent Events over HTTP
- **http**: Standard HTTP request/response

## Configuration Format

### Basic Configuration (Flat YAML)

```yaml
name: my-agent
model: openai:gpt-4o

instructions: |
  You are a helpful assistant with access to various tools.

mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/workspace"

max_iterations: 10
```

### Kubernetes-Style Configuration

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops-agent
  labels:
    environment: production

spec:
  model: openai:gpt-4o

  instructions: |
    You are a Kubernetes operations assistant.

  mcp_servers:
    - name: kubectl-ai
      transport: stdio
      command: kubectl-ai
      args:
        - "--mcp-server"
        - "--mcp-server-mode=stdio"
```

## Transport Types

### stdio Transport

Local process communication - the most common pattern.

```yaml
mcp_servers:
  - name: filesystem
    transport: stdio      # Can be omitted (default)
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/workspace"
    env:
      NODE_ENV: production
    timeout_secs: 30
    auto_reconnect: true
```

**Parameters:**
- `command` (required): The executable to run
- `args`: Command line arguments
- `env`: Environment variables to pass to the process
- `timeout_secs`: Connection timeout (default: 30)
- `auto_reconnect`: Reconnect on failure (default: true)

### SSE Transport

Server-Sent Events over HTTP for real-time tool communication.

```yaml
mcp_servers:
  - name: remote-tools
    transport: sse
    endpoint: http://localhost:3000/mcp/sse
    timeout_secs: 60
```

**Parameters:**
- `endpoint` (required): SSE endpoint URL
- `timeout_secs`: Connection timeout (default: 30)

### HTTP Transport

Standard HTTP JSON-RPC for simple request/response patterns.

```yaml
mcp_servers:
  - name: api-tools
    transport: http
    endpoint: http://localhost:8080/mcp/v1
    timeout_secs: 60
```

**Parameters:**
- `endpoint` (required): HTTP endpoint URL
- `timeout_secs`: Request timeout (default: 30)

## Real-World MCP Servers

### kubectl-ai

Kubernetes operations via natural language.

```yaml
mcp_servers:
  - name: kubectl-ai
    transport: stdio
    command: kubectl-ai
    args:
      - "--mcp-server"
      - "--mcp-server-mode=stdio"
    timeout_secs: 120
```

**Installation:**
```bash
brew install kubectl-ai
# or
go install github.com/GoogleCloudPlatform/kubectl-ai@latest
```

### GitHub MCP Server

GitHub operations (repos, issues, PRs).

```yaml
mcp_servers:
  - name: github
    transport: stdio
    command: docker
    args:
      - "run"
      - "-i"
      - "--rm"
      - "-e"
      - "GITHUB_PERSONAL_ACCESS_TOKEN"
      - "ghcr.io/github/github-mcp-server"
    env:
      GITHUB_PERSONAL_ACCESS_TOKEN: "${GITHUB_TOKEN}"
```

**Prerequisites:**
```bash
docker pull ghcr.io/github/github-mcp-server
export GITHUB_TOKEN="your-token"
```

### Prometheus MCP Server

Prometheus monitoring queries.

```yaml
mcp_servers:
  - name: prometheus
    transport: stdio
    command: docker
    args:
      - "run"
      - "-i"
      - "--rm"
      - "-e"
      - "PROMETHEUS_URL"
      - "ghcr.io/pab1it0/prometheus-mcp-server:latest"
    env:
      PROMETHEUS_URL: "http://host.docker.internal:9090"
```

### Official Filesystem Server

File system operations.

```yaml
mcp_servers:
  - name: filesystem
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/allowed/path"
```

### SQLite Server

Database operations.

```yaml
mcp_servers:
  - name: sqlite
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-sqlite"
      - "/path/to/database.db"
```

## Multi-Server Configuration

Agents can connect to multiple MCP servers simultaneously:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: sre-agent

spec:
  model: openai:gpt-4o

  instructions: |
    You are an SRE assistant with access to Kubernetes,
    monitoring, and filesystem tools.

  mcp_servers:
    # Kubernetes operations
    - name: kubectl-ai
      command: kubectl-ai
      args: ["--mcp-server", "--mcp-server-mode=stdio"]
      timeout_secs: 120

    # Prometheus monitoring
    - name: prometheus
      command: docker
      args:
        - "run"
        - "-i"
        - "--rm"
        - "-e"
        - "PROMETHEUS_URL"
        - "ghcr.io/pab1it0/prometheus-mcp-server:latest"
      env:
        PROMETHEUS_URL: "http://localhost:9090"

    # File system access
    - name: filesystem
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
```

## Advanced Configuration

### Tool Filtering

Limit which tools are exposed from a server:

```yaml
mcp_servers:
  - name: filesystem
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    tools:
      - read_file
      - list_directory
    # write_file and other tools will be filtered out
```

### Initialization Options

Pass custom options to the MCP server:

```yaml
mcp_servers:
  - name: custom-server
    command: ./my-mcp-server
    init_options:
      debug: true
      workspace: /home/user
      features:
        - advanced_search
        - cache_enabled
```

### Environment Variable Expansion

Use shell-style environment variables:

```yaml
mcp_servers:
  - name: github
    command: docker
    args: ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
    env:
      GITHUB_PERSONAL_ACCESS_TOKEN: "${GITHUB_TOKEN}"
      GITHUB_HOST: "${GITHUB_HOST:-https://github.com}"
```

## Backward Compatibility

The legacy `tools` field still works for simple cases:

```yaml
name: legacy-agent
model: openai:gpt-4o
tools:
  - shell
  - kubectl
  - read_file
```

When both `mcp_servers` and `tools` are specified, `mcp_servers` takes priority.

## CLI Usage

### Run an agent with MCP tools

```bash
aofctl run agent my-agent.yaml --input "list all pods"
```

### Validate configuration

```bash
aofctl validate -f my-agent.yaml
```

### Start daemon with agents directory

```bash
aofctl serve --config daemon-config.yaml --agents-dir ./agents/
```

## Troubleshooting

### Common Issues

1. **MCP server not starting**
   - Check if the command is in PATH
   - Verify arguments are correct
   - Check timeout settings

2. **Tools not appearing**
   - Verify server initialization completes
   - Check server logs for errors
   - Ensure tools/list response is valid JSON-RPC

3. **Docker-based servers failing**
   - Ensure Docker is running
   - Check if images are pulled
   - Verify environment variables are set

### Debug Tips

Enable verbose logging:

```bash
RUST_LOG=debug aofctl run agent config.yaml --input "test"
```

Test MCP server manually:

```bash
# Test kubectl-ai
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | kubectl-ai --mcp-server --mcp-server-mode=stdio
```

## See Also

- [Agent Configuration](./AGENT_CONFIGURATION.md)
- [CLI Reference](./CLI_REFERENCE.md)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
