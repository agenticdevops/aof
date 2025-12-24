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

## Using MCP Servers with AOF

### Configuration

Add MCP servers to your agent or daemon configuration:

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

### Daemon-Level Configuration

For shared MCP servers across all agents:

```yaml
# daemon.yaml
mcp_servers:
  - name: postgres
    command: npx
    args: ["-y", "@modelcontextprotocol/server-postgres"]
    env:
      DATABASE_URL: ${DATABASE_URL}
```

## Catalog Overview

### Core Servers

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Filesystem](./filesystem.md) | Read/write files on the local filesystem | read_file, write_file, list_directory, search_files |
| [Fetch](./fetch.md) | Make HTTP requests and fetch web content | fetch (GET with auto markdown conversion) |
| [Puppeteer](./puppeteer.md) | Browser automation for scraping and testing | navigate, screenshot, click, fill, evaluate |

### Development

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [GitHub](./github.md) | GitHub repos, issues, PRs | create_issue, create_pull_request, get_file_contents, search_code |
| [GitLab](./gitlab.md) | GitLab projects, MRs, CI/CD | create_issue, create_merge_request, get_file_contents |

### Databases

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [PostgreSQL](./postgres.md) | Query PostgreSQL databases (read-only) | query |
| [SQLite](./sqlite.md) | Query and modify SQLite databases | read_query, write_query, create_table, list_tables |

### Communication

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Slack](./slack.md) | Send messages and interact with Slack | slack_post_message, slack_list_channels, slack_add_reaction |

### Search

| Server | Description | Key Tools |
|--------|-------------|-----------|
| [Brave Search](./brave-search.md) | Web search using Brave Search API | brave_web_search, brave_local_search |

## Installation

All official MCP servers can be installed via npx:

```bash
# No installation needed - npx downloads on first use
npx -y @modelcontextprotocol/server-filesystem /path

# Or install globally
npm install -g @modelcontextprotocol/server-filesystem
```

## Security Considerations

1. **Credential Management**: Use environment variables for secrets
2. **Scope Limitation**: Restrict filesystem access to specific directories
3. **Network Access**: Use firewalls to limit puppeteer/fetch targets
4. **Audit Logging**: AOF logs all MCP tool invocations

## Creating Custom MCP Servers

See the [MCP Integration Guide](../tools/mcp-integration.md) for building custom servers.

## Next Steps

- [Filesystem Server](./filesystem.md) - File operations
- [GitHub Server](./github.md) - Repository automation
- [PostgreSQL Server](./postgres.md) - Database queries
