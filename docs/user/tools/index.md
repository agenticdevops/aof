# AOF Built-in Tools

AOF provides a modular tool system that allows agents to interact with external systems, execute commands, and perform operations. Tools can be used directly as built-in Rust implementations or through MCP (Model Context Protocol) servers.

## Quick Start: Unified CLI Tools (Recommended)

For DevOps workflows, use the **unified CLI tools** that take a single `command` argument. These let the LLM construct the appropriate command based on context:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-assistant
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a helpful DevOps assistant with access to Kubernetes,
    Git, Docker, and Terraform. Help users manage their infrastructure.
  tools:
    - kubectl    # Any kubectl command
    - git        # Any git command
    - docker     # Any docker command
    - terraform  # Any terraform command
    - aws        # Any AWS CLI command
    - helm       # Any helm command
    - shell      # General shell commands
```

This approach is:
- **Simpler** - fewer tools to configure
- **More flexible** - supports any subcommand
- **LLM-native** - leverages the model's command construction abilities

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
│    │  │ Unified CLI:  │ │                                        │
│    │  │  - kubectl    │ │                                        │
│    │  │  - git        │ │                                        │
│    │  │  - docker     │ │                                        │
│    │  │  - terraform  │ │                                        │
│    │  │  - aws        │ │                                        │
│    │  │  - helm       │ │                                        │
│    │  │ Core:         │ │                                        │
│    │  │  - shell      │ │                                        │
│    │  │  - FileTools  │ │                                        │
│    │  │  - HttpTool   │ │                                        │
│    │  └───────────────┘ │                                        │
│    └────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────┘
```

## Tool Categories

### Recommended: Unified CLI Tools

| Tool | Description | Example Commands |
|------|-------------|------------------|
| `kubectl` | Kubernetes operations | `get pods -n prod`, `apply -f deploy.yaml`, `logs my-pod` |
| `git` | Git operations | `status`, `commit -m "msg"`, `push origin main` |
| `docker` | Docker operations | `ps -a`, `build -t app .`, `logs container` |
| `terraform` | Terraform IaC | `init`, `plan`, `apply -auto-approve` |
| `aws` | AWS CLI operations | `s3 ls`, `ec2 describe-instances` |
| `helm` | Helm package manager | `list -A`, `install app ./chart` |

### Core Tools

| Tool | Feature Flag | Description |
|------|--------------|-------------|
| `shell` | `shell` | Execute shell commands |
| `read_file` | `file` | Read file contents |
| `write_file` | `file` | Write to files |
| `list_directory` | `file` | List directory contents |
| `search_files` | `file` | Search for files |
| `http` | `http` | HTTP requests |

### Observability Tools

| Tool | Description |
|------|-------------|
| `prometheus_query` | Query Prometheus metrics |
| `loki_query` | Query Loki logs |
| `elasticsearch_query` | Query Elasticsearch |
| `victoriametrics_query` | Query VictoriaMetrics |

---

## Unified CLI Tools Reference

### kubectl

Execute any kubectl command for Kubernetes operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | kubectl command (without 'kubectl' prefix) |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 120) |

**Examples:**
```json
{"command": "get pods -n production"}
{"command": "apply -f deployment.yaml"}
{"command": "logs my-pod --tail=100"}
{"command": "exec -it my-pod -- /bin/sh"}
{"command": "describe deployment nginx"}
```

### git

Execute any git command for version control.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | git command (without 'git' prefix) |
| working_dir | string | No | Repository path |
| timeout_secs | integer | No | Timeout (default: 120) |

**Examples:**
```json
{"command": "status"}
{"command": "commit -m \"Add new feature\""}
{"command": "push origin main"}
{"command": "log --oneline -10"}
{"command": "checkout -b feature/new"}
```

### docker

Execute any docker command for container operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | docker command (without 'docker' prefix) |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 300) |

**Examples:**
```json
{"command": "ps -a"}
{"command": "build -t myapp:latest ."}
{"command": "run --rm -p 8080:80 nginx"}
{"command": "logs my-container --tail 50"}
{"command": "compose up -d"}
```

### terraform

Execute any terraform command for Infrastructure as Code.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | terraform command (without 'terraform' prefix) |
| working_dir | string | No | Terraform config directory |
| timeout_secs | integer | No | Timeout (default: 600) |

**Examples:**
```json
{"command": "init"}
{"command": "plan -out=tfplan"}
{"command": "apply -auto-approve"}
{"command": "output -json"}
{"command": "state list"}
```

### aws

Execute any AWS CLI command.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | aws command (without 'aws' prefix) |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 120) |

**Examples:**
```json
{"command": "s3 ls"}
{"command": "ec2 describe-instances"}
{"command": "logs filter-log-events --log-group-name /aws/lambda/func"}
{"command": "lambda invoke --function-name myfunc out.json"}
{"command": "ecs list-clusters"}
```

### helm

Execute any helm command for Kubernetes package management.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | helm command (without 'helm' prefix) |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 300) |

**Examples:**
```json
{"command": "list -A"}
{"command": "install myapp ./chart"}
{"command": "upgrade myapp ./chart --set replicas=3"}
{"command": "repo add bitnami https://charts.bitnami.com/bitnami"}
{"command": "template myapp ./chart"}
```

---

## Configuration Examples

### Basic DevOps Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-assistant
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a DevOps assistant. Help users with Kubernetes, Git, and Docker tasks.
  tools:
    - kubectl
    - git
    - docker
    - shell
```

### Infrastructure Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: infra-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are an infrastructure automation specialist.
    Help manage cloud resources and Terraform configurations.
  tools:
    - terraform
    - aws
    - shell
    - read_file
    - write_file
```

### Multi-Agent Fleet

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: devops-team
spec:
  agents:
    - name: k8s-specialist
      role: worker
      spec:
        model: google:gemini-2.5-flash
        instructions: Handle Kubernetes operations
        tools:
          - kubectl
          - helm

    - name: git-specialist
      role: worker
      spec:
        model: google:gemini-2.5-flash
        instructions: Handle Git operations
        tools:
          - git

    - name: docker-specialist
      role: worker
      spec:
        model: google:gemini-2.5-flash
        instructions: Handle container operations
        tools:
          - docker

  coordination:
    mode: peer
    distribution: round-robin
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
    # Unified CLI tools
    - kubectl
    - git
    - docker

    # MCP tool
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

---

## Shell Tool

Execute shell commands with safety controls.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| command | string | Yes | Command to execute |
| working_dir | string | No | Working directory |
| timeout_secs | integer | No | Timeout (default: 60) |

**Security Features:**
- Blocked dangerous commands (rm -rf /, mkfs, etc.)
- Timeout protection
- Optional command whitelist

**Example:**
```json
{
  "command": "ls -la /var/log",
  "timeout_secs": 30
}
```

---

## File Tools

### read_file

Read contents of a file.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| path | string | Yes | Path to file |
| encoding | string | No | File encoding (default: utf-8) |
| max_bytes | integer | No | Max bytes to read (default: 1MB) |

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

### search_files

Search for files matching a pattern.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| pattern | string | Yes | Glob pattern (e.g., `**/*.rs`) |
| path | string | No | Base path (default: `.`) |
| max_results | integer | No | Max results (default: 100) |

---

## Observability Tools

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

**Example:**
```json
{
  "endpoint": "http://prometheus:9090",
  "query": "sum(rate(http_requests_total[5m])) by (service)"
}
```

### loki_query

Query Loki logs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | Loki URL |
| query | string | Yes | LogQL query |
| limit | integer | No | Max entries (default: 100) |

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

### victoriametrics_query

Query VictoriaMetrics.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| endpoint | string | Yes | VictoriaMetrics URL |
| query | string | Yes | MetricsQL query |

---

## HTTP Tool

Make HTTP requests.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| url | string | Yes | Request URL |
| method | string | No | HTTP method (default: GET) |
| headers | object | No | Request headers |
| body | string | No | Request body |
| json | object | No | JSON body |
| timeout_secs | integer | No | Timeout (default: 30) |

**Example:**
```json
{
  "url": "https://api.example.com/data",
  "method": "POST",
  "json": {"key": "value"}
}
```

---

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

## Legacy Per-Operation Tools (Backward Compatibility)

For backward compatibility, AOF still supports per-operation tools. These are more verbose but provide structured parameters:

| Legacy Tool | Unified Equivalent |
|-------------|-------------------|
| `kubectl_get`, `kubectl_apply`, `kubectl_delete`, etc. | `kubectl` |
| `git_status`, `git_commit`, `git_push`, etc. | `git` |
| `docker_ps`, `docker_build`, `docker_run`, etc. | `docker` |
| `terraform_init`, `terraform_plan`, `terraform_apply`, etc. | `terraform` |
| `aws_s3`, `aws_ec2`, `aws_logs`, etc. | `aws` |

**Legacy Example:**
```yaml
tools:
  - kubectl_get
  - kubectl_apply
  - kubectl_logs
  - git_status
  - docker_ps
```

**Recommended Equivalent:**
```yaml
tools:
  - kubectl
  - git
  - docker
```

The unified approach is simpler and more flexible. Use legacy tools only if you need the structured parameter schemas.

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
use aof_tools::{ToolRegistry, KubectlTool, GitTool};

let mut registry = ToolRegistry::new();
registry.register(KubectlTool::new());
registry.register(GitTool::new());
registry.register(MyCustomTool::new());
let executor = registry.into_executor();
```

---

## Feature Flags

Enable only the tools you need in `Cargo.toml`:

```toml
[dependencies]
aof-tools = { version = "0.1", features = ["file", "shell"] }

# Or enable all
aof-tools = { version = "0.1", features = ["all"] }
```

| Feature | Description |
|---------|-------------|
| `file` | File system tools |
| `shell` | Shell execution |
| `kubectl` | Legacy kubectl tools |
| `docker` | Legacy docker tools |
| `git` | Legacy git tools |
| `terraform` | Legacy terraform tools |
| `http` | HTTP client |
| `observability` | Prometheus, Loki, ELK, VictoriaMetrics |
| `all` | All tools |

**Note:** The unified CLI tools (kubectl, git, docker, terraform, aws, helm) are always available and don't require feature flags.
