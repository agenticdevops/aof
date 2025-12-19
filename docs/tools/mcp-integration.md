# MCP Integration

The Model Context Protocol (MCP) enables agents to use tools from external servers. AOF supports MCP as a first-class tool source, allowing you to extend agent capabilities with any MCP-compatible server.

## Overview

MCP provides a standardized way for AI agents to interact with external tools and services. Key benefits:

- **Extensibility**: Add new tools without modifying AOF
- **Ecosystem**: Use existing MCP servers from the community
- **Isolation**: Tools run in separate processes
- **Flexibility**: Mix built-in and MCP tools

## Transport Types

AOF supports three MCP transport mechanisms:

### Stdio Transport

Communicates with MCP servers via standard input/output. Best for local servers.

```yaml
mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    env:
      DEBUG: "true"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique server identifier |
| `transport` | string | Yes | Must be `stdio` |
| `command` | string | Yes | Command to execute |
| `args` | array | No | Command arguments |
| `env` | object | No | Environment variables |
| `timeout_secs` | int | No | Connection timeout (default: 30) |
| `auto_reconnect` | bool | No | Auto-reconnect on failure (default: true) |

### SSE Transport

Server-Sent Events over HTTP. Best for remote servers with streaming.

```yaml
mcp_servers:
  - name: remote-tools
    transport: sse
    endpoint: https://mcp.example.com/events
    headers:
      Authorization: "Bearer ${MCP_TOKEN}"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique server identifier |
| `transport` | string | Yes | Must be `sse` |
| `endpoint` | string | Yes | SSE endpoint URL |
| `headers` | object | No | HTTP headers |
| `timeout_secs` | int | No | Connection timeout |

### HTTP Transport

Standard HTTP request/response. Best for REST-based MCP servers.

```yaml
mcp_servers:
  - name: api-tools
    transport: http
    endpoint: https://mcp.example.com/v1
    headers:
      X-API-Key: "${API_KEY}"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique server identifier |
| `transport` | string | Yes | Must be `http` |
| `endpoint` | string | Yes | HTTP endpoint URL |
| `headers` | object | No | HTTP headers |
| `timeout_secs` | int | No | Request timeout |

---

## Configuration

### Agent with MCP Servers

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: full-stack-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a full-stack assistant with access to filesystem,
    GitHub, and database tools.

  mcp_servers:
    # Local filesystem access
    - name: filesystem
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]

    # GitHub integration
    - name: github
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"

    # PostgreSQL access
    - name: postgres
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres"]
      env:
        DATABASE_URL: "${DATABASE_URL}"
```

### Fleet with MCP Servers

Fleet agents can have individual MCP configurations:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: analysis-fleet
spec:
  agents:
    - name: code-analyst
      spec:
        model: google:gemini-2.5-flash
        instructions: Analyze code quality and patterns.
        mcp_servers:
          - name: filesystem
            transport: stdio
            command: npx
            args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]

    - name: security-scanner
      spec:
        model: google:gemini-2.5-flash
        instructions: Scan for security vulnerabilities.
        mcp_servers:
          - name: filesystem
            transport: stdio
            command: npx
            args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
          - name: github
            transport: stdio
            command: npx
            args: ["-y", "@modelcontextprotocol/server-github"]
            env:
              GITHUB_TOKEN: "${GITHUB_TOKEN}"

  coordination:
    mode: peer
```

### AgentFlow with MCP Servers

Flow nodes can specify MCP servers for inline agent configurations:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: document-processor
spec:
  trigger:
    type: HTTP
    config:
      path: /process

  nodes:
    - id: analyzer
      type: Agent
      config:
        agent_config: |
          name: inline-analyzer
          model: google:gemini-2.5-flash
          instructions: Analyze the uploaded document.
          mcp_servers:
            - name: filesystem
              transport: stdio
              command: npx
              args: ["-y", "@modelcontextprotocol/server-filesystem", "/uploads"]

  connections:
    - from: trigger
      to: analyzer
```

---

## Tool Filtering

Limit which tools from an MCP server are available:

```yaml
mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    tools:
      - read_file
      - list_directory
    # write_file and other tools will NOT be available
```

---

## Initialization Options

Pass options to MCP servers during initialization:

```yaml
mcp_servers:
  - name: custom-server
    transport: stdio
    command: ./my-mcp-server
    init_options:
      debug: true
      max_connections: 10
      custom_setting: "value"
```

---

## Popular MCP Servers

### Official Anthropic Servers

| Server | Package | Purpose |
|--------|---------|---------|
| Filesystem | `@modelcontextprotocol/server-filesystem` | File operations |
| GitHub | `@modelcontextprotocol/server-github` | GitHub API |
| PostgreSQL | `@modelcontextprotocol/server-postgres` | Database queries |
| Slack | `@modelcontextprotocol/server-slack` | Slack integration |
| Google Drive | `@modelcontextprotocol/server-gdrive` | Google Drive access |
| Memory | `@modelcontextprotocol/server-memory` | Persistent memory |

### Community Servers

| Server | Package | Purpose |
|--------|---------|---------|
| Brave Search | `@anthropic-ai/mcp-server-brave-search` | Web search |
| Puppeteer | `@anthropic-ai/mcp-server-puppeteer` | Browser automation |
| SQLite | `@anthropic-ai/mcp-server-sqlite` | SQLite database |

---

## Examples

### Kubernetes + GitHub Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-github-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a DevOps assistant that can manage Kubernetes
    clusters and interact with GitHub repositories.

  tools:
    - kubectl
    - helm

  mcp_servers:
    - name: github
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

### Database Query Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: db-analyst
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a database analyst. Query the database to answer
    questions about the data. Always use safe read-only queries.

  mcp_servers:
    - name: postgres
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres"]
      env:
        DATABASE_URL: "postgresql://readonly:pass@localhost/analytics"
      tools:
        - query  # Only allow query, not execute
```

### Multi-Server Research Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: research-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a research assistant with access to web search,
    files, and note-taking capabilities.

  mcp_servers:
    - name: search
      transport: stdio
      command: npx
      args: ["-y", "@anthropic-ai/mcp-server-brave-search"]
      env:
        BRAVE_API_KEY: "${BRAVE_API_KEY}"

    - name: filesystem
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "./research"]

    - name: memory
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-memory"]
```

---

## Troubleshooting

### Server Won't Start

1. Check the command exists: `which npx`
2. Verify package is installed: `npx -y @modelcontextprotocol/server-filesystem --version`
3. Check environment variables are set
4. Look at AOF logs for connection errors

### Tools Not Appearing

1. Ensure `tools` filter includes the tool name
2. Check server logs for initialization errors
3. Verify MCP server version compatibility

### Connection Timeouts

1. Increase `timeout_secs` in configuration
2. Check network connectivity for remote servers
3. Enable `auto_reconnect: true` for unstable connections

### Environment Variables

Use `${VAR_NAME}` syntax in YAML. Variables are expanded at runtime:

```yaml
env:
  API_KEY: "${MY_API_KEY}"      # Expanded from environment
  STATIC: "literal-value"       # Used as-is
```

---

## See Also

- [Tools Overview](./index.md) - Introduction to tools
- [Built-in Tools](./builtin-tools.md) - Native tool reference
- [Agent Reference](../reference/agent-spec.md) - Agent configuration
- [Fleet Reference](../reference/fleet-spec.md) - Fleet configuration

