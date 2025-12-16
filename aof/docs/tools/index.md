# AOF Built-in Tools

AOF provides a modular tool system that allows agents to interact with external systems, execute commands, and perform operations. Tools can be used directly as built-in Rust implementations or through MCP (Model Context Protocol) servers.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Agent                                     │
│                          │                                       │
│                    ToolExecutor                                  │
│                    /         \                                   │
│         BuiltinToolExecutor   McpToolExecutor                   │
│              │                     │                             │
│    ┌─────────┴─────────┐    ┌─────┴─────┐                       │
│    │    Tool Registry   │    │ MCP Server │                      │
│    │  ┌───────────────┐ │    └───────────┘                       │
│    │  │ FileTools     │ │                                        │
│    │  │ ShellTool     │ │                                        │
│    │  │ KubectlTools  │ │                                        │
│    │  │ DockerTools   │ │                                        │
│    │  │ GitTools      │ │                                        │
│    │  │ TerraformTools│ │                                        │
│    │  │ Observability │ │                                        │
│    │  │ AwsTools      │ │                                        │
│    │  └───────────────┘ │                                        │
│    └────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────┘
```

## Tool Categories

| Category | Feature Flag | Tools | Description |
|----------|--------------|-------|-------------|
| [File](#file-tools) | `file` | read_file, write_file, list_directory, search_files | File system operations |
| [Shell](#shell-tool) | `shell` | shell | Execute shell commands |
| [Kubectl](#kubectl-tools) | `kubectl` | kubectl_get, kubectl_apply, kubectl_delete, kubectl_logs, kubectl_exec, kubectl_describe | Kubernetes operations |
| [Docker](#docker-tools) | `docker` | docker_ps, docker_build, docker_run, docker_logs, docker_exec, docker_images | Container operations |
| [Git](#git-tools) | `git` | git_status, git_diff, git_log, git_commit, git_branch, git_checkout, git_pull, git_push | Repository operations |
| [Terraform](#terraform-tools) | `terraform` | terraform_init, terraform_plan, terraform_apply, terraform_destroy, terraform_output | IaC operations |
| [HTTP](#http-tool) | `http` | http_request | HTTP requests |
| [Observability](#observability-tools) | `observability` | prometheus_query, loki_query, elasticsearch_query, victoriametrics_query | Metrics & logs |
| [AWS](#aws-tools) | `aws` | aws_s3, aws_ec2, aws_logs, aws_iam, aws_lambda, aws_ecs | AWS CLI operations |

## Choosing Built-in vs MCP Tools

### Use Built-in Tools When:
- You need **low latency** (no subprocess overhead)
- You want **fewer dependencies** (no Node.js/Python required)
- You need **tighter integration** with the agent runtime
- You're building **production deployments**

### Use MCP Tools When:
- You need **community-maintained** tool implementations
- You want to use **existing MCP servers** from the ecosystem
- You need **language-specific** tooling (npm packages, Python libraries)
- You're **prototyping** and want quick integrations

---

## Tool Specification (ToolSpec)

AOF uses a unified `ToolSpec` format that supports both built-in and MCP tools with optional configuration.

### Spec Formats

Tools can be specified in three formats:

#### 1. Simple String (Backward Compatible)
```yaml
tools:
  - shell
  - kubectl_get
  - git_status
```
Simple strings default to **built-in** tools.

#### 2. Qualified Built-in Tool with Config
```yaml
tools:
  - name: shell
    source: builtin
    config:
      blocked_commands:
        - "rm -rf /"
        - "mkfs"
      timeout_secs: 60
    timeout_secs: 120  # Override default timeout
```

#### 3. Qualified MCP Tool
```yaml
tools:
  - name: read_file
    source: mcp
    server: filesystem   # References an MCP server by name
    config:
      allowed_paths:
        - /workspace
        - /home/user/projects
```

### ToolSpec Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | string | Yes | Tool name |
| `source` | string | No | `builtin` (default) or `mcp` |
| `server` | string | If MCP | MCP server name (must match an entry in `mcp_servers`) |
| `config` | object | No | Tool-specific configuration |
| `enabled` | boolean | No | Enable/disable tool (default: true) |
| `timeout_secs` | integer | No | Override default timeout |

---

## Configuration Examples

### Using Built-in Tools Only

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-agent
spec:
  model: openai:gpt-4o
  instructions: |
    You are a DevOps engineer assistant.
  tools:
    - shell
    - kubectl_get
    - kubectl_logs
    - docker_ps
    - git_status
```

### Using MCP Tools Only

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-agent-mcp
spec:
  model: openai:gpt-4o
  instructions: |
    You are a DevOps engineer assistant.
  mcp_servers:
    - name: kubectl-ai
      transport: stdio
      command: kubectl-ai
      args: ["--mcp-server"]
    - name: filesystem
      transport: stdio
      command: npx
      args: ["@modelcontextprotocol/server-filesystem", "/workspace"]
```

### Hybrid: Built-in + MCP Tools

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: hybrid-agent
spec:
  model: openai:gpt-4o
  instructions: |
    You are a DevOps engineer assistant.
  tools:
    # Simple built-in tools
    - shell
    - git_status

    # Built-in with custom config
    - name: kubectl_get
      source: builtin
      timeout_secs: 120

    # MCP tool from a server
    - name: github_search
      source: mcp
      server: github

  mcp_servers:
    - name: github
      transport: stdio
      command: npx
      args: ["@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

### Built-in Tools with Security Config

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: secure-agent
spec:
  model: openai:gpt-4o
  instructions: |
    You are a secure operations assistant.
  tools:
    - name: shell
      source: builtin
      config:
        blocked_commands:
          - "rm -rf"
          - "mkfs"
          - "dd"
          - "> /dev"
        allowed_commands:
          - "ls"
          - "cat"
          - "grep"
          - "find"
        timeout_secs: 30

    - name: read_file
      source: builtin
      config:
        max_bytes: 1048576  # 1MB limit
        allowed_paths:
          - /workspace
          - /tmp
```

### MCP Tools with Arguments

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: mcp-configured-agent
spec:
  model: openai:gpt-4o
  instructions: |
    You are a code assistant.
  tools:
    - name: search_code
      source: mcp
      server: sourcegraph
      config:
        default_repo: "github.com/myorg/myrepo"
        max_results: 50

  mcp_servers:
    - name: sourcegraph
      transport: http
      endpoint: http://sourcegraph.internal/mcp
      init_options:
        token: "${SOURCEGRAPH_TOKEN}"
        default_context: "myorg"
```

---

## File Tools

File system operations for reading, writing, and searching files.

### read_file

Read contents of a file.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | Yes | Path to file |
| encoding | string | No | File encoding (default: utf-8) |
| max_bytes | integer | No | Max bytes to read (default: 1MB) |

**Example:**
```json
{
  "path": "/app/config.yaml",
  "max_bytes": 10000
}
```

### write_file

Write content to a file.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | Yes | Path to file |
| content | string | Yes | Content to write |
| append | boolean | No | Append mode (default: false) |
| create_dirs | boolean | No | Create parent dirs (default: true) |

### list_directory

List contents of a directory.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | Yes | Directory path |
| recursive | boolean | No | List recursively (default: false) |
| include_hidden | boolean | No | Include hidden files (default: false) |
| max_depth | integer | No | Max recursion depth (default: 3) |

### search_files

Search for files matching a pattern.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| pattern | string | Yes | Glob pattern (e.g., `**/*.rs`) |
| path | string | No | Base path (default: `.`) |
| max_results | integer | No | Max results (default: 100) |

**MCP Alternative:** `@modelcontextprotocol/server-filesystem`

---

## Shell Tool

Execute shell commands with safety controls.

### shell

Execute a shell command.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | Command to execute |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 60) |
| env | object | No | Environment variables |

**Security Features:**
- Blocked dangerous commands (rm -rf /, mkfs, etc.)
- Timeout protection
- Optional command whitelist

**Example:**
```json
{
  "command": "kubectl get pods -n production",
  "timeout_secs": 30
}
```

---

## Kubectl Tools

Kubernetes operations via kubectl CLI.

### kubectl_get

Get Kubernetes resources.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| resource | string | Yes | Resource type (pods, deployments, etc.) |
| name | string | No | Resource name |
| namespace | string | No | Namespace |
| all_namespaces | boolean | No | All namespaces (-A) |
| output | string | No | Format: json, yaml, wide, name |
| selector | string | No | Label selector |
| field_selector | string | No | Field selector |

**Example:**
```json
{
  "resource": "pods",
  "namespace": "production",
  "selector": "app=nginx",
  "output": "json"
}
```

### kubectl_apply

Apply Kubernetes manifests.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| manifest | string | No | YAML manifest content |
| file | string | No | Path to manifest file |
| namespace | string | No | Namespace |
| dry_run | string | No | none, client, server |
| force | boolean | No | Force apply |

### kubectl_delete

Delete Kubernetes resources.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| resource | string | Yes | Resource type |
| name | string | No | Resource name |
| namespace | string | No | Namespace |
| selector | string | No | Label selector |
| force | boolean | No | Force delete |
| grace_period | integer | No | Grace period (default: 30) |

### kubectl_logs

Get pod logs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| pod | string | Yes | Pod name |
| namespace | string | No | Namespace |
| container | string | No | Container name |
| tail | integer | No | Lines from end (default: 100) |
| since | string | No | Duration (e.g., '5m', '1h') |
| previous | boolean | No | Previous instance |
| timestamps | boolean | No | Show timestamps |

### kubectl_exec

Execute commands in containers.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| pod | string | Yes | Pod name |
| command | string | Yes | Command to execute |
| namespace | string | No | Namespace |
| container | string | No | Container name |

### kubectl_describe

Describe resources.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| resource | string | Yes | Resource type |
| name | string | Yes | Resource name |
| namespace | string | No | Namespace |

**MCP Alternative:** `kubectl-ai --mcp-server`

---

## Docker Tools

Docker container operations.

### docker_ps

List containers.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| all | boolean | No | Show all containers |
| filter | string | No | Filter (e.g., 'status=running') |
| format | string | No | Output format |

### docker_build

Build images.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Build context (default: .) |
| tag | string | Yes | Image tag |
| dockerfile | string | No | Dockerfile path |
| build_args | object | No | Build arguments |
| no_cache | boolean | No | Don't use cache |
| target | string | No | Target stage |
| platform | string | No | Target platform |

### docker_run

Run containers.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| image | string | Yes | Image name |
| name | string | No | Container name |
| command | string | No | Command |
| detach | boolean | No | Run in background |
| rm | boolean | No | Remove after exit (default: true) |
| ports | array | No | Port mappings |
| volumes | array | No | Volume mounts |
| env | object | No | Environment variables |
| network | string | No | Network |

### docker_logs

Get container logs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| container | string | Yes | Container name/ID |
| tail | integer | No | Lines (default: 100) |
| since | string | No | Duration |
| timestamps | boolean | No | Show timestamps |

### docker_exec

Execute in containers.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| container | string | Yes | Container name/ID |
| command | string | Yes | Command |
| user | string | No | User |
| workdir | string | No | Working directory |

### docker_images

List images.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| all | boolean | No | Show all images |
| filter | string | No | Filter |

---

## Git Tools

Git repository operations.

### git_status

Show working tree status.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Repository path (default: .) |
| short | boolean | No | Short format |
| porcelain | boolean | No | Machine-readable (default: true) |

### git_diff

Show changes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Repository path |
| file | string | No | Specific file |
| staged | boolean | No | Show staged changes |
| commit | string | No | Compare with commit |
| stat | boolean | No | Show diffstat |

### git_log

Show commit history.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Repository path |
| count | integer | No | Number of commits (default: 10) |
| oneline | boolean | No | One line per commit |
| author | string | No | Filter by author |
| since | string | No | Since date |
| file | string | No | File history |

### git_commit

Create commits.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| message | string | Yes | Commit message |
| path | string | No | Repository path |
| all | boolean | No | Stage all (-a) |
| files | array | No | Files to commit |

### git_branch

List/create branches.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Repository path |
| name | string | No | Branch to create |
| delete | string | No | Branch to delete |
| all | boolean | No | List all branches |

### git_checkout

Switch branches.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| branch | string | No | Branch name |
| path | string | No | Repository path |
| create | boolean | No | Create new branch |
| file | string | No | Checkout file |

### git_pull / git_push

Pull/push changes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Repository path |
| remote | string | No | Remote name (default: origin) |
| branch | string | No | Branch |
| rebase | boolean | No | Rebase (pull only) |
| set_upstream | boolean | No | Set upstream (push only) |
| tags | boolean | No | Push tags |

---

## Terraform Tools

Terraform IaC operations.

### terraform_init

Initialize Terraform.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Configuration path |
| upgrade | boolean | No | Upgrade providers |
| reconfigure | boolean | No | Reconfigure backend |
| backend_config | object | No | Backend config values |

### terraform_plan

Create execution plan.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Configuration path |
| out | string | No | Save plan to file |
| var | object | No | Variables |
| var_file | string | No | Variable file |
| target | array | No | Target resources |
| destroy | boolean | No | Create destroy plan |

### terraform_apply

Apply changes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Configuration path |
| auto_approve | boolean | No | Skip approval (default: true) |
| var | object | No | Variables |
| var_file | string | No | Variable file |
| target | array | No | Target resources |
| plan_file | string | No | Apply saved plan |

### terraform_destroy

Destroy infrastructure.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Configuration path |
| auto_approve | boolean | No | Skip approval (REQUIRED for safety) |
| var | object | No | Variables |
| target | array | No | Target resources |

### terraform_output

Get outputs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | No | Configuration path |
| name | string | No | Output name |
| json | boolean | No | JSON format (default: true) |

---

## HTTP Tool

Make HTTP requests.

### http_request

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| url | string | Yes | Request URL |
| method | string | No | HTTP method (default: GET) |
| headers | object | No | Request headers |
| body | string | No | Request body |
| json | object | No | JSON body |
| timeout_secs | integer | No | Timeout (default: 30) |
| follow_redirects | boolean | No | Follow redirects (default: true) |

**MCP Alternative:** `@anthropic/mcp-server-fetch`

---

## Observability Tools

Query observability platforms for metrics and logs.

### prometheus_query

Query Prometheus metrics.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | Prometheus URL |
| query | string | Yes | PromQL query |
| time | string | No | Evaluation time |
| start | string | No | Range start |
| end | string | No | Range end |
| step | string | No | Step (default: 15s) |
| timeout | string | No | Timeout (default: 30s) |

**Example:**
```json
{
  "endpoint": "http://prometheus:9090",
  "query": "sum(rate(http_requests_total[5m])) by (service)",
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-01T01:00:00Z",
  "step": "1m"
}
```

### loki_query

Query Loki logs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | Loki URL |
| query | string | Yes | LogQL query |
| start | string | No | Start time |
| end | string | No | End time |
| limit | integer | No | Max entries (default: 100) |
| direction | string | No | forward or backward |

**Example:**
```json
{
  "endpoint": "http://loki:3100",
  "query": "{namespace=\"production\"} |= \"error\"",
  "limit": 50
}
```

### elasticsearch_query

Query Elasticsearch/OpenSearch.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | Elasticsearch URL |
| index | string | Yes | Index pattern |
| query | object | No | Query DSL |
| size | integer | No | Results (default: 10) |
| from | integer | No | Offset (default: 0) |
| sort | array | No | Sort spec |
| source | array | No | Fields to include |
| auth | object | No | {username, password} |

**Example:**
```json
{
  "endpoint": "http://elasticsearch:9200",
  "index": "logs-*",
  "query": {
    "bool": {
      "must": [
        {"match": {"level": "error"}},
        {"range": {"@timestamp": {"gte": "now-1h"}}}
      ]
    }
  },
  "size": 20
}
```

### victoriametrics_query

Query VictoriaMetrics.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | VictoriaMetrics URL |
| query | string | Yes | MetricsQL query |
| time | string | No | Evaluation time |
| start | string | No | Range start |
| end | string | No | Range end |
| step | string | No | Step (default: 15s) |

---

## AWS Tools

AWS CLI operations via the `aws` command-line tool.

### aws_s3

S3 operations (ls, cp, sync, rm).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | Operation: ls, cp, sync, rm, mb, rb |
| source | string | Depends | Source path (for cp, sync) |
| destination | string | Depends | Destination path (for cp, sync) |
| bucket | string | Depends | Bucket name (for ls, mb, rb) |
| prefix | string | No | Key prefix for listing |
| recursive | boolean | No | Recursive operation |
| delete | boolean | No | Delete during sync |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

**Example:**
```json
{
  "operation": "ls",
  "bucket": "my-bucket",
  "prefix": "logs/",
  "recursive": true
}
```

### aws_ec2

EC2 instance operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | describe-instances, start-instances, stop-instances, etc. |
| instance_ids | array | Depends | Instance IDs |
| filters | array | No | Filter expressions |
| query | string | No | JMESPath query |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

**Example:**
```json
{
  "operation": "describe-instances",
  "filters": [
    {"Name": "instance-state-name", "Values": ["running"]}
  ],
  "query": "Reservations[*].Instances[*].[InstanceId,InstanceType,State.Name]"
}
```

### aws_logs

CloudWatch Logs operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | filter-log-events, describe-log-groups, tail-logs |
| log_group | string | Yes | Log group name |
| log_stream | string | No | Log stream name |
| filter_pattern | string | No | Filter pattern |
| start_time | string | No | Start time (ISO8601 or epoch ms) |
| end_time | string | No | End time |
| limit | integer | No | Max events (default: 100) |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

**Example:**
```json
{
  "operation": "filter-log-events",
  "log_group": "/aws/lambda/my-function",
  "filter_pattern": "ERROR",
  "start_time": "2024-01-01T00:00:00Z",
  "limit": 50
}
```

### aws_iam

IAM operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | list-users, list-roles, get-user, get-role, etc. |
| user_name | string | Depends | User name |
| role_name | string | Depends | Role name |
| policy_arn | string | Depends | Policy ARN |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

### aws_lambda

Lambda operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | list-functions, get-function, invoke |
| function_name | string | Depends | Function name |
| payload | object | No | Invocation payload (for invoke) |
| qualifier | string | No | Version or alias |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

**Example:**
```json
{
  "operation": "invoke",
  "function_name": "my-function",
  "payload": {"key": "value"}
}
```

### aws_ecs

ECS operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| operation | string | Yes | list-clusters, list-services, describe-services, etc. |
| cluster | string | Depends | Cluster name/ARN |
| service | string | Depends | Service name |
| task | string | Depends | Task ID |
| profile | string | No | AWS profile |
| region | string | No | AWS region |

**Example:**
```json
{
  "operation": "describe-services",
  "cluster": "production",
  "service": "web-api"
}
```

**MCP Alternative:** `@aws/mcp-server-aws`

---

## Building Custom Tools

Platform engineers can extend AOF with custom tools:

```rust
use aof_core::{Tool, ToolConfig, ToolInput, ToolResult, AofResult};
use async_trait::async_trait;

pub struct MyCustomTool {
    config: ToolConfig,
}

impl MyCustomTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig {
                name: "my_tool".to_string(),
                description: "My custom tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    },
                    "required": ["input"]
                }),
                tool_type: aof_core::ToolType::Custom,
                timeout_secs: 30,
                extra: Default::default(),
            },
        }
    }
}

#[async_trait]
impl Tool for MyCustomTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let my_input: String = input.get_arg("input")?;

        // Your logic here

        Ok(ToolResult::success(serde_json::json!({
            "result": "success"
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

Register your tool:

```rust
use aof_tools::{ToolRegistry, BuiltinToolExecutor};

let mut registry = ToolRegistry::new();
registry.register(MyCustomTool::new());
let executor = registry.into_executor();
```

---

## Feature Flags

Enable only the tools you need in `Cargo.toml`:

```toml
[dependencies]
aof-tools = { version = "0.1", features = ["file", "shell", "kubectl"] }

# Or enable all
aof-tools = { version = "0.1", features = ["all"] }
```

| Feature | Description |
|---------|-------------|
| `file` | File system tools |
| `shell` | Shell execution |
| `kubectl` | Kubernetes tools |
| `docker` | Docker tools |
| `git` | Git tools |
| `terraform` | Terraform tools |
| `http` | HTTP client |
| `observability` | Prometheus, Loki, ELK, VictoriaMetrics |
| `aws` | AWS CLI tools (S3, EC2, Lambda, etc.) |
| `all` | All tools |
