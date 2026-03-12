# MCP (Model Context Protocol) Servers

MCP servers expose tool capabilities to your agent over a standard protocol.
They're typically third-party tools (filesystem access, GitHub, databases, etc.)
wrapped in the MCP protocol.

## Supported Transports

| Transport | Description | Use case |
|-----------|-------------|----------|
| `stdio` | Server runs as a subprocess | Local tools, CLI wrappers |
| `sse` | Server-Sent Events over HTTP | Remote MCP servers |
| `http` | HTTP requests | Stateless remote servers |

## Configuring MCP Servers

Add MCP servers to your agent's `agent.yaml` under `mcp_servers`:

```yaml
# agents/my-agent/agent.yaml
spec_version: v1
name: my-agent
model:
  preferred: anthropic/claude-sonnet-4-6
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
      GITHUB_PERSONAL_ACCESS_TOKEN: "${GITHUB_TOKEN}"
```

## Defining MCP Tool References

After configuring a server, define tool references in your agent's `tools/` directory:

```yaml
# agents/my-agent/tools/read-file.yaml
name: read_file
type: mcp
server: filesystem
description: Read a file from the workspace
```

The `server` field must match a name in your `mcp_servers` list.

## Flat YAML Format (Kubernetes-Style)

In the flat YAML format (`apiVersion: openagentix.dev/v1`), add MCP servers to the spec:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: file-agent
spec:
  model: anthropic/claude-sonnet-4-6
  mcp_servers:
    - name: filesystem
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
```

## Connection Lifecycle

MCP server connections are **per-run**:
1. When an agent run starts and calls an MCP tool, the connection is established
2. The server runs as a subprocess (for stdio) for the duration of the run
3. After the run completes, the server process is terminated

This keeps resource usage predictable — no server processes running for idle agents.

## Error Handling

If an MCP server fails to connect or a tool call returns an error, the error is fed
back to the agent as a tool observation. The agent can then decide to retry, try a
different approach, or report the failure to the user.

Example error observation:
```
MCP tool 'read_file' on server 'filesystem' failed: connection refused
```

## Popular MCP Servers

| Server | Package | Use case |
|--------|---------|----------|
| Filesystem | `@modelcontextprotocol/server-filesystem` | File read/write |
| GitHub | `@modelcontextprotocol/server-github` | PR, issues, code |
| PostgreSQL | `@modelcontextprotocol/server-postgres` | Database queries |
| Slack | `@modelcontextprotocol/server-slack` | Messages, channels |
| Fetch | `@modelcontextprotocol/server-fetch` | HTTP requests |

Find more: https://github.com/modelcontextprotocol/servers
