# AOF CLI Reference (aofctl)

**Last Updated**: December 16, 2025

`aofctl` is the command-line interface for AOF, providing kubectl-style commands for agent orchestration.

## Installation

```bash
# Build from source
cargo build --release -p aofctl

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Quick Start

```bash
# Run an agent
aofctl run agent my-agent.yaml --input "Hello, world!"

# List resources
aofctl get agents

# Start the trigger server
aofctl serve --config daemon-config.yaml
```

## Commands

### run

Execute an agent with a configuration file.

```bash
aofctl run agent <config-file> [options]
```

**Options:**
- `-i, --input <input>`: Input/query for the agent
- `-o, --output <format>`: Output format (json, yaml, text) [default: text]

**Examples:**
```bash
# Run with inline input
aofctl run agent k8s-agent.yaml --input "list all pods"

# Run with JSON output
aofctl run agent agent.yaml -i "summarize logs" -o json

# Run workflow (when implemented)
aofctl run workflow incident-response.yaml
```

### get

List resources in the system.

```bash
aofctl get <resource-type> [name] [options]
```

**Resource Types:**
- `agents` / `agent`: List configured agents
- `workflows` / `workflow`: List workflows
- `tools` / `tool`: List available tools
- `mcpservers` / `mcpserver`: List MCP servers
- `jobs` / `job`: List running jobs

**Options:**
- `-o, --output <format>`: Output format (json, yaml, wide, name) [default: wide]
- `--all-namespaces`: Show resources in all namespaces

**Examples:**
```bash
# List all agents
aofctl get agents

# Get specific agent details
aofctl get agent my-agent -o yaml

# List all MCP tools
aofctl get tools -o json
```

### apply

Apply configuration from a file.

```bash
aofctl apply -f <file> [options]
```

**Options:**
- `-f, --file <file>`: Configuration file (YAML)
- `-n, --namespace <namespace>`: Namespace for the resources

**Examples:**
```bash
# Apply an agent configuration
aofctl apply -f agent.yaml

# Apply to specific namespace
aofctl apply -f agent.yaml -n production
```

### delete

Delete resources by type and name.

```bash
aofctl delete <resource-type> <name> [options]
```

**Options:**
- `-n, --namespace <namespace>`: Namespace

**Examples:**
```bash
# Delete an agent
aofctl delete agent my-agent

# Delete from namespace
aofctl delete workflow my-workflow -n staging
```

### describe

Show detailed information about a resource.

```bash
aofctl describe <resource-type> <name> [options]
```

**Examples:**
```bash
# Describe an agent
aofctl describe agent my-agent

# Describe a workflow
aofctl describe workflow incident-response
```

### logs

View logs from a resource.

```bash
aofctl logs <resource-type> <name> [options]
```

**Options:**
- `-f, --follow`: Follow log output (stream)
- `--tail <lines>`: Number of lines to show from the end

**Examples:**
```bash
# View agent logs
aofctl logs agent my-agent

# Follow logs in real-time
aofctl logs job task-123 -f

# Show last 50 lines
aofctl logs agent my-agent --tail 50
```

### exec

Execute a command in a resource context.

```bash
aofctl exec <resource-type> <name> -- <command...>
```

**Examples:**
```bash
# Execute in agent context
aofctl exec agent my-agent -- "What is the weather?"

# Run workflow step
aofctl exec workflow my-workflow -- step-3
```

### serve

Start the trigger webhook server (daemon mode).

```bash
aofctl serve [options]
```

**Options:**
- `-c, --config <file>`: Configuration file (YAML)
- `-p, --port <port>`: Port to listen on [default: 8080]
- `--host <host>`: Host to bind to [default: 0.0.0.0]
- `--agents-dir <dir>`: Directory containing agent YAML files

**Examples:**
```bash
# Start with default settings
aofctl serve

# Start with configuration file
aofctl serve --config daemon-config.yaml

# Start on specific port
aofctl serve --port 9090 --agents-dir ./agents/
```

**Configuration File Format:**
```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: production

spec:
  server:
    port: 8080
    host: 0.0.0.0
    cors: true
    timeout_secs: 30

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN

  agents:
    directory: ./agents/
    watch: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

### validate

Validate an agent configuration file.

```bash
aofctl validate -f <file>
```

**Examples:**
```bash
# Validate single file
aofctl validate -f my-agent.yaml

# Validate with verbose output
RUST_LOG=debug aofctl validate -f my-agent.yaml
```

### api-resources

List all available API resources.

```bash
aofctl api-resources
```

**Output:**
```
NAME          SHORTNAMES   APIVERSION   NAMESPACED   KIND
agents        ag           aof.dev/v1   true         Agent
workflows     wf           aof.dev/v1   true         Workflow
tools                      aof.dev/v1   false        Tool
mcpservers    mcpsrv       mcp/v1       false        McpServer
jobs                       aof.dev/v1   true         Job
```

### version

Display version information.

```bash
aofctl version
```

## Configuration Files

### Agent Configuration

```yaml
# Flat format
name: my-agent
model: openai:gpt-4o
instructions: You are a helpful assistant.
mcp_servers:
  - name: tools
    command: my-mcp-server
max_iterations: 10

# Kubernetes-style format
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: my-agent
  labels:
    environment: production
spec:
  model: openai:gpt-4o
  instructions: You are a helpful assistant.
  mcp_servers:
    - name: tools
      command: my-mcp-server
```

See [MCP Configuration Guide](./MCP_CONFIGURATION.md) for detailed MCP server configuration.

### Daemon Configuration

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: production

spec:
  server:
    port: 8080
    host: 0.0.0.0

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `GOOGLE_API_KEY` | Google AI API key |
| `AWS_REGION` | AWS region for Bedrock |
| `RUST_LOG` | Log level (error, warn, info, debug, trace) |

## Output Formats

### text (default)
Human-readable text output.

### json
JSON formatted output for scripting.

### yaml
YAML formatted output.

### wide
Extended table output with additional columns.

### name
Just resource names, one per line.

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Resource not found |
| 4 | Timeout |

## Examples

### Complete Workflow

```bash
# 1. Create agent configuration
cat > k8s-agent.yaml << 'EOF'
name: k8s-agent
model: openai:gpt-4o
instructions: You help with Kubernetes operations.
mcp_servers:
  - name: kubectl-ai
    command: kubectl-ai
    args: ["--mcp-server", "--mcp-server-mode=stdio"]
EOF

# 2. Validate configuration
aofctl validate -f k8s-agent.yaml

# 3. Run the agent
aofctl run agent k8s-agent.yaml --input "list all pods in kube-system namespace"

# 4. Check logs (if using daemon mode)
aofctl logs agent k8s-agent --tail 100
```

### Daemon Mode

```bash
# 1. Create daemon configuration
cat > daemon.yaml << 'EOF'
apiVersion: aof.dev/v1
kind: DaemonConfig
spec:
  server:
    port: 8080
  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET
EOF

# 2. Set environment variables
export SLACK_BOT_TOKEN="xoxb-your-token"
export SLACK_SIGNING_SECRET="your-secret"

# 3. Start the server
aofctl serve --config daemon.yaml
```

## See Also

- [MCP Configuration Guide](./MCP_CONFIGURATION.md)
- [Agent Configuration](./AGENT_CONFIGURATION.md)
- [Project Status](../PROJECT_STATUS.md)
