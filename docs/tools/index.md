# Tools Overview

AOF provides a comprehensive tools system that enables agents to interact with infrastructure, execute commands, and integrate with external services. Tools can be built-in (native Rust implementations) or provided via MCP (Model Context Protocol) servers.

## Tool Categories

| Category | Description | Examples |
|----------|-------------|----------|
| **CLI Tools** | Unified command-line wrappers | `kubectl`, `git`, `docker`, `terraform`, `aws`, `helm` |
| **File Tools** | File system operations | `read_file`, `write_file`, `list_directory`, `search_files` |
| **Execution Tools** | Shell and HTTP execution | `shell`, `http_request` |
| **Observability Tools** | Metrics and logs queries | `prometheus_query`, `loki_query`, `elasticsearch_query`, `grafana_*`, `datadog_*`, `newrelic_*` |
| **SIEM Tools** | Security information and event management | `splunk_*` |
| **ITSM Tools** | IT Service Management | `servicenow_*` |
| **CI/CD Tools** | Pipeline and deployment management | `github_*`, `gitlab_*`, `argocd_*`, `flux_*` |
| **MCP Tools** | External MCP server tools | Any tool from configured MCP servers |

## Quick Start

### Using Built-in Tools

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a DevOps assistant with access to Kubernetes and Git.
  tools:
    - kubectl
    - git
    - shell
```

### Using MCP Tools

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: filesystem-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a file management assistant.
  mcp_servers:
    - name: filesystem
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
```

## Tool Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Agent Execution                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Model Response with Tool Calls                                  │
│           │                                                       │
│           ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              Tool Executor Router                            │ │
│  │                                                               │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │ │
│  │  │  Built-in   │  │  Single MCP │  │    Multi-MCP        │  │ │
│  │  │  Executor   │  │  Executor   │  │    Executor         │  │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│           │                                                       │
│           ▼                                                       │
│  Tool Result → Added to conversation → Next model call           │
└─────────────────────────────────────────────────────────────────┘
```

## Tool Configuration Formats

### Simple Format (Recommended)

```yaml
tools:
  - kubectl
  - git
  - shell
```

### Qualified Format (Advanced)

```yaml
tools:
  - name: kubectl
    source: builtin
    timeout_secs: 120

  - name: custom_tool
    source: mcp
    server: my-mcp-server
```

## Next Steps

- [Built-in Tools Reference](./builtin-tools.md) - Complete list of built-in tools
- [CI/CD Tools](./cicd.md) - GitHub Actions, GitLab CI, ArgoCD, and Flux integration
- [Grafana Tools](./grafana.md) - Native Grafana integration for metrics and dashboards
- [Datadog Tools](./datadog.md) - Native Datadog integration for observability
- [New Relic Tools](./newrelic.md) - Native New Relic NerdGraph (GraphQL) integration
- [Splunk Tools](./splunk.md) - Native Splunk SIEM and log analysis integration
- [ServiceNow Tools](./servicenow.md) - Native ServiceNow ITSM integration
- [MCP Integration](./mcp-integration.md) - Using MCP servers with agents
- [Custom Tools](./custom-tools.md) - Creating custom tool implementations

