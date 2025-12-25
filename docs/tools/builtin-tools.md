# Built-in Tools Reference

AOF includes a comprehensive set of built-in tools for DevOps workflows. All built-in tools are implemented in Rust and execute locally.

## CLI Tools (Unified)

These tools accept a single `command` argument, allowing the LLM to construct any subcommand. This is the **recommended** approach for maximum flexibility.

### kubectl

Execute any Kubernetes command.

| Property | Value |
|----------|-------|
| **Name** | `kubectl` |
| **Binary** | kubectl |
| **Timeout** | 120 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full kubectl command arguments"
}
```

**Example Usage:**
```yaml
tools:
  - kubectl
```

**Agent will call:**
```json
{
  "name": "kubectl",
  "input": {
    "command": "get pods -n production -o wide"
  }
}
```

---

### git

Execute any Git command.

| Property | Value |
|----------|-------|
| **Name** | `git` |
| **Binary** | git |
| **Timeout** | 120 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full git command arguments"
}
```

**Example:**
```json
{
  "name": "git",
  "input": {
    "command": "log --oneline -10"
  }
}
```

---

### docker

Execute any Docker command.

| Property | Value |
|----------|-------|
| **Name** | `docker` |
| **Binary** | docker |
| **Timeout** | 300 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full docker command arguments"
}
```

---

### terraform

Execute any Terraform command.

| Property | Value |
|----------|-------|
| **Name** | `terraform` |
| **Binary** | terraform |
| **Timeout** | 600 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full terraform command arguments"
}
```

---

### aws

Execute any AWS CLI command.

| Property | Value |
|----------|-------|
| **Name** | `aws` |
| **Binary** | aws |
| **Timeout** | 120 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full AWS CLI command arguments"
}
```

---

### helm

Execute any Helm command.

| Property | Value |
|----------|-------|
| **Name** | `helm` |
| **Binary** | helm |
| **Timeout** | 300 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Full helm command arguments"
}
```

---

## File System Tools

Native Rust implementations for file operations. No external binaries required.

### read_file

Read contents of a file.

**Parameters:**
```json
{
  "path": "string (required) - Path to the file to read"
}
```

**Returns:**
```json
{
  "content": "string - File contents",
  "size": "number - File size in bytes"
}
```

---

### write_file

Write content to a file.

**Parameters:**
```json
{
  "path": "string (required) - Path to the file",
  "content": "string (required) - Content to write"
}
```

---

### list_directory

List contents of a directory.

**Parameters:**
```json
{
  "path": "string (required) - Directory path",
  "recursive": "boolean (optional) - List recursively, default false"
}
```

**Returns:**
```json
{
  "entries": [
    {
      "name": "string",
      "type": "file | directory",
      "size": "number"
    }
  ]
}
```

---

### search_files

Search for files matching a pattern.

**Parameters:**
```json
{
  "path": "string (required) - Starting directory",
  "pattern": "string (required) - Glob pattern (e.g., '*.yaml')",
  "recursive": "boolean (optional) - Search recursively, default true"
}
```

---

## Execution Tools

### shell

Execute shell commands.

| Property | Value |
|----------|-------|
| **Name** | `shell` |
| **Timeout** | 120 seconds |

**Parameters:**
```json
{
  "command": "string (required) - Shell command to execute"
}
```

**Returns:**
```json
{
  "stdout": "string",
  "stderr": "string",
  "exit_code": "number"
}
```

---

### http_request

Make HTTP requests.

| Property | Value |
|----------|-------|
| **Name** | `http_request` |
| **Timeout** | 60 seconds |

**Parameters:**
```json
{
  "url": "string (required) - Request URL",
  "method": "string (optional) - HTTP method, default GET",
  "headers": "object (optional) - Request headers",
  "body": "string (optional) - Request body"
}
```

**Returns:**
```json
{
  "status": "number",
  "headers": "object",
  "body": "string"
}
```

---

## Observability Tools

Native HTTP clients for querying observability systems.

### prometheus_query

Query Prometheus metrics using PromQL.

| Property | Value |
|----------|-------|
| **Name** | `prometheus_query` |
| **Timeout** | 60 seconds |

**Parameters:**
```json
{
  "endpoint": "string (required) - Prometheus server URL",
  "query": "string (required) - PromQL query",
  "time": "string (optional) - Evaluation timestamp",
  "timeout": "string (optional) - Query timeout"
}
```

**Example:**
```json
{
  "endpoint": "http://prometheus:9090",
  "query": "rate(http_requests_total[5m])"
}
```

---

### loki_query

Query Loki logs using LogQL.

| Property | Value |
|----------|-------|
| **Name** | `loki_query` |
| **Timeout** | 60 seconds |

**Parameters:**
```json
{
  "endpoint": "string (required) - Loki server URL",
  "query": "string (required) - LogQL query",
  "start": "string (optional) - Start timestamp",
  "end": "string (optional) - End timestamp",
  "limit": "number (optional) - Maximum entries"
}
```

**Example:**
```json
{
  "endpoint": "http://loki:3100",
  "query": "{namespace=\"production\"} |= \"error\""
}
```

---

### elasticsearch_query

Query Elasticsearch/OpenSearch.

| Property | Value |
|----------|-------|
| **Name** | `elasticsearch_query` |
| **Timeout** | 60 seconds |

**Parameters:**
```json
{
  "endpoint": "string (required) - Elasticsearch URL",
  "index": "string (required) - Index pattern",
  "query": "object (required) - Elasticsearch query DSL"
}
```

---

### victoriametrics_query

Query VictoriaMetrics using MetricsQL.

| Property | Value |
|----------|-------|
| **Name** | `victoriametrics_query` |
| **Timeout** | 60 seconds |

**Parameters:**
```json
{
  "endpoint": "string (required) - VictoriaMetrics URL",
  "query": "string (required) - MetricsQL query"
}
```

---

## Legacy Per-Operation Tools

These tools provide structured parameters for specific operations. They exist for backward compatibility but the unified CLI tools are recommended.

### Kubernetes (kubectl_*)

| Tool | Operation | Key Parameters |
|------|-----------|----------------|
| `kubectl_get` | Get resources | `resource`, `name`, `namespace`, `output` |
| `kubectl_apply` | Apply manifest | `manifest`, `filename`, `namespace` |
| `kubectl_delete` | Delete resources | `resource`, `name`, `namespace` |
| `kubectl_logs` | Get pod logs | `pod`, `namespace`, `container`, `tail` |
| `kubectl_exec` | Execute in pod | `pod`, `namespace`, `container`, `command` |
| `kubectl_describe` | Describe resource | `resource`, `name`, `namespace` |

### Docker (docker_*)

| Tool | Operation | Key Parameters |
|------|-----------|----------------|
| `docker_ps` | List containers | `all`, `filter` |
| `docker_build` | Build image | `context`, `tag`, `dockerfile` |
| `docker_run` | Run container | `image`, `name`, `ports`, `volumes` |
| `docker_logs` | Get logs | `container`, `tail`, `follow` |
| `docker_exec` | Execute command | `container`, `command` |
| `docker_images` | List images | `filter` |

### Git (git_*)

| Tool | Operation | Key Parameters |
|------|-----------|----------------|
| `git_status` | Show status | - |
| `git_diff` | Show diff | `path`, `staged` |
| `git_log` | Show history | `count`, `oneline` |
| `git_commit` | Create commit | `message`, `all` |
| `git_branch` | Manage branches | `name`, `delete` |
| `git_checkout` | Switch branches | `branch`, `create` |
| `git_pull` | Pull changes | `remote`, `branch` |
| `git_push` | Push changes | `remote`, `branch` |

### Terraform (terraform_*)

| Tool | Operation | Key Parameters |
|------|-----------|----------------|
| `terraform_init` | Initialize | `backend_config` |
| `terraform_plan` | Create plan | `out`, `var` |
| `terraform_apply` | Apply changes | `auto_approve`, `var` |
| `terraform_destroy` | Destroy resources | `auto_approve` |
| `terraform_output` | Get outputs | `name`, `json` |

---

## Feature Flags

Tools are organized by feature flags in the `aof-tools` crate:

| Feature | Tools Included |
|---------|----------------|
| `file` | `read_file`, `write_file`, `list_directory`, `search_files` |
| `shell` | `shell` |
| `kubectl` | `kubectl_*` legacy tools |
| `docker` | `docker_*` legacy tools |
| `git` | `git_*` legacy tools |
| `terraform` | `terraform_*` legacy tools |
| `http` | `http_request` |
| `observability` | `prometheus_query`, `loki_query`, `elasticsearch_query`, `victoriametrics_query`, `newrelic_*` |
| `siem` | `splunk_*` (Splunk SPL queries, alerts, HEC) |
| `itsm` | `servicenow_*` (ServiceNow incidents, CMDB, changes) |
| `all` | All tools |

The unified CLI tools (`kubectl`, `git`, `docker`, `terraform`, `aws`, `helm`) are always available.

---

## Platform-Specific Tools

For detailed documentation on platform-specific integrations:

- [Grafana Tools](./grafana.md) - Native Grafana integration for metrics and dashboards
- [Datadog Tools](./datadog.md) - Native Datadog integration for observability
- [New Relic Tools](./newrelic.md) - Native New Relic NerdGraph integration
- [Splunk Tools](./splunk.md) - Native Splunk SIEM and log analysis
- [ServiceNow Tools](./servicenow.md) - Native ServiceNow ITSM integration

---

## See Also

- [Tools Overview](./index.md) - Introduction to tools
- [MCP Integration](./mcp-integration.md) - External tool servers
- [Agent Reference](../reference/agent-spec.md) - Agent configuration

