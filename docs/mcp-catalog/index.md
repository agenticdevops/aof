---
sidebar_position: 1
sidebar_label: Overview
---

# MCP Server Catalog

AOF supports the Model Context Protocol (MCP) for extending agent capabilities with external tools and data sources. This catalog documents tested and recommended MCP servers.

## What is MCP?

The [Model Context Protocol](https://modelcontextprotocol.io/) is an open standard for connecting AI models to external data sources and tools. MCP servers provide:

- **Tools**: Functions the agent can invoke (e.g., query database, fetch URL)
- **Resources**: Data the agent can read (e.g., files, database schemas)
- **Prompts**: Pre-defined prompt templates

## Quick Start

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: my-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: filesystem
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/files"]
    - name: github
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: ${GITHUB_TOKEN}
```

## Catalog by Category

### Infrastructure & Kubernetes

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Kubernetes](./kubernetes.md) | Query and manage K8s clusters | kubectl, get_pods, get_logs, describe_resource |
| [AWS](./aws.md) | EC2, S3, Lambda, CloudWatch | list_instances, get_metrics, invoke_function |

### Observability

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Prometheus](./prometheus.md) | PromQL queries and alerts | query, query_range, get_alerts |
| [Grafana](./grafana.md) | Dashboards and annotations | search_dashboards, query_data_source, create_annotation |
| [Datadog](./datadog.md) | Metrics, monitors, logs | query_metrics, get_monitors, search_logs |

### Development & Git

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [GitHub](./github.md) | Repos, issues, PRs | create_issue, create_pull_request, get_file_contents, search_code |
| [GitLab](./gitlab.md) | Projects, MRs, CI/CD | create_issue, create_merge_request, get_file_contents |
| [Filesystem](./filesystem.md) | Read/write local files | read_file, write_file, list_directory, search_files |

### Databases

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [PostgreSQL](./postgres.md) | Query PostgreSQL (read-only) | query |
| [SQLite](./sqlite.md) | Query and modify SQLite | read_query, write_query, create_table, list_tables |

### Communication

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Slack](./slack.md) | Send messages, interact | slack_post_message, slack_list_channels, slack_add_reaction |

### Web & Search

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Fetch](./fetch.md) | Make HTTP requests | fetch (GET with auto markdown conversion) |
| [Puppeteer](./puppeteer.md) | Browser automation | navigate, screenshot, click, fill, evaluate |
| [Brave Search](./brave-search.md) | Web search | brave_web_search, brave_local_search |

## Configuration Patterns

### Agent-Level Configuration

Add MCP servers to individual agents:

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: k8s-debugger
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: kubernetes
      command: npx
      args: ["-y", "@anthropic/mcp-server-kubernetes"]
    - name: prometheus
      command: npx
      args: ["-y", "@anthropic/mcp-server-prometheus"]
      env:
        PROMETHEUS_URL: ${PROMETHEUS_URL}
```

### Daemon-Level Configuration

Share MCP servers across all agents:

```yaml
# daemon.yaml
spec:
  mcp_servers:
    - name: postgres
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres"]
      env:
        DATABASE_URL: ${DATABASE_URL}
    - name: github
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: ${GITHUB_TOKEN}
```

### Environment Variables

Always use environment variables for secrets:

```yaml
mcp_servers:
  - name: aws
    command: npx
    args: ["-y", "@anthropic/mcp-server-aws"]
    env:
      AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID}
      AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY}
      AWS_REGION: us-east-1
```

## Installation

All MCP servers can be installed via npx (no pre-installation needed):

```bash
# npx downloads and runs on first use
npx -y @modelcontextprotocol/server-filesystem /path

# Or install globally for faster startup
npm install -g @modelcontextprotocol/server-filesystem
```

## Security Best Practices

### 1. Credential Management

- Use environment variables for all secrets
- Rotate API keys regularly
- Use service accounts where possible

### 2. Scope Limitation

- Restrict filesystem access to specific directories
- Use read-only database connections when possible
- Apply least-privilege IAM policies

### 3. Network Security

- Use firewalls to limit outbound connections
- Restrict puppeteer/fetch to allowed domains
- Use VPC endpoints for cloud services

### 4. Audit Logging

- AOF logs all MCP tool invocations
- Enable cloud provider audit logs (CloudTrail, etc.)
- Monitor for unusual access patterns

## Creating Custom MCP Servers

For custom integrations, see the [MCP Integration Guide](../guides/mcp-integration.md).

### Basic Server Template

```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

const server = new Server(
  { name: "my-server", version: "1.0.0" },
  { capabilities: { tools: {} } }
);

server.setRequestHandler("tools/list", async () => ({
  tools: [{
    name: "my_tool",
    description: "Does something useful",
    inputSchema: {
      type: "object",
      properties: {
        param: { type: "string", description: "Parameter" }
      },
      required: ["param"]
    }
  }]
}));

server.setRequestHandler("tools/call", async (request) => {
  if (request.params.name === "my_tool") {
    return { content: [{ type: "text", text: "Result" }] };
  }
  throw new Error("Unknown tool");
});

const transport = new StdioServerTransport();
await server.connect(transport);
```

## Troubleshooting

### MCP Server Not Starting

```bash
# Test server directly
npx -y @modelcontextprotocol/server-filesystem /tmp

# Check for errors
DEBUG=* npx -y @modelcontextprotocol/server-github
```

### Tool Calls Failing

1. Check environment variables are set
2. Verify credentials have required permissions
3. Check network connectivity
4. Review AOF logs for detailed errors

### Performance Issues

- Use daemon-level MCP config to share server instances
- Install servers globally to avoid npx download time
- Use connection pooling for database servers

## Next Steps

- [Kubernetes Server](./kubernetes.md) - K8s cluster management
- [GitHub Server](./github.md) - Repository automation
- [Prometheus Server](./prometheus.md) - Metrics queries
- [AWS Server](./aws.md) - Cloud infrastructure
