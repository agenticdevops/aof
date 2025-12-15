# aofctl CLI Reference

Complete command reference for the AOF CLI tool.

## Global Flags

Available for all commands:

```bash
--config string       Config file (default: $HOME/.aof/config.yaml)
--context string      Kubernetes-style context to use
--namespace string    Namespace to operate in (default: "default")
--output string       Output format: json|yaml|table (default: "table")
--verbose            Enable verbose logging
--help               Show help for command
```

---

## Agent Commands

Manage AI agents.

### `aofctl agent apply`

Create or update an agent from a YAML file.

```bash
aofctl agent apply -f agent.yaml [flags]
```

**Flags:**
- `-f, --file string` - Path to agent YAML file
- `--dry-run` - Validate without applying
- `--force` - Force update even if agent is running

**Examples:**
```bash
# Apply agent configuration
aofctl agent apply -f my-agent.yaml

# Dry run to validate
aofctl agent apply -f my-agent.yaml --dry-run

# Apply multiple files
aofctl agent apply -f agent1.yaml -f agent2.yaml

# Apply from directory
aofctl agent apply -f ./agents/
```

---

### `aofctl agent get`

List or retrieve agents.

```bash
aofctl agent get [name] [flags]
```

**Flags:**
- `-o, --output string` - Output format: json|yaml|table
- `-l, --selector string` - Label selector (e.g., "env=prod")
- `--all-namespaces` - Show agents across all namespaces

**Examples:**
```bash
# List all agents
aofctl agent get

# Get specific agent
aofctl agent get my-agent

# Get as YAML
aofctl agent get my-agent -o yaml

# Filter by label
aofctl agent get -l env=production

# All namespaces
aofctl agent get --all-namespaces
```

**Output:**
```
NAME              MODEL           STATUS    AGE
k8s-helper        openai:gpt-4    Running   5d
slack-bot         anthropic:claude-3-5-sonnet-20241022   Running   2d
incident-responder openai:gpt-4   Running   1d
```

---

### `aofctl agent describe`

Show detailed information about an agent.

```bash
aofctl agent describe <name> [flags]
```

**Examples:**
```bash
aofctl agent describe my-agent
```

**Output:**
```
Name:         my-agent
Namespace:    default
Labels:       env=production
              team=platform
Annotations:  owner=platform@company.com
Created:      2024-01-15 10:30:00

Spec:
  Model:              openai:gpt-4
  Temperature:        0.3
  Max Tokens:         2000

Tools:
  - Type: Shell
    Commands: kubectl, helm
  - Type: MCP
    Server: kubectl-mcp

Memory:
  Type: SQLite
  Path: ./agent-memory.db
  Messages: 150/1000

Status:
  State:              Running
  Last Activity:      2024-01-20 14:45:00
  Total Executions:   234
  Success Rate:       98.7%
  Avg Response Time:  2.3s

Recent Conversations:
  2024-01-20 14:45:00  User: Show me failing pods
  2024-01-20 14:30:12  User: Scale nginx to 5 replicas
  2024-01-20 14:15:00  User: What's the status of the cluster?
```

---

### `aofctl agent run`

Run an agent interactively.

```bash
aofctl agent run <file-or-name> [flags]
```

**Flags:**
- `-f, --file string` - Agent YAML file
- `--input string` - Single query (non-interactive)
- `--context string` - Additional context
- `--timeout int` - Response timeout in seconds

**Examples:**
```bash
# Interactive mode
aofctl agent run my-agent.yaml

# From applied agent
aofctl agent run my-agent

# Single query
aofctl agent run my-agent --input "Show me all pods"

# With context
aofctl agent run my-agent --context "namespace=production"
```

**Interactive Session:**
```bash
$ aofctl agent run k8s-helper

Agent 'k8s-helper' is ready. Type your message (or 'exit' to quit):

> Show me all pods in the default namespace

Fetching pods in default namespace...

NAME                        READY   STATUS    RESTARTS   AGE
nginx-deployment-abc123     2/2     Running   0          5d
postgres-0                  1/1     Running   0          10d

All pods are healthy! ✅

> exit

Session ended. Summary:
  Queries: 1
  Duration: 1m 23s
  Tokens used: 450
```

---

### `aofctl agent chat`

Chat with an applied agent.

```bash
aofctl agent chat <name> [message] [flags]
```

**Examples:**
```bash
# Interactive chat
aofctl agent chat my-agent

# Single message
aofctl agent chat my-agent "What's the cluster status?"
```

---

### `aofctl agent exec`

Execute a single command with an agent.

```bash
aofctl agent exec <name> <message> [flags]
```

**Flags:**
- `--timeout int` - Timeout in seconds (default: 120)
- `--context string` - Additional context

**Examples:**
```bash
# Single execution
aofctl agent exec k8s-helper "Show failing pods"

# With timeout
aofctl agent exec my-agent "Long running task" --timeout 300
```

---

### `aofctl agent delete`

Delete an agent.

```bash
aofctl agent delete <name> [flags]
```

**Flags:**
- `--force` - Skip confirmation
- `--cascade` - Delete associated resources

**Examples:**
```bash
# Delete with confirmation
aofctl agent delete my-agent

# Force delete
aofctl agent delete my-agent --force
```

---

### `aofctl agent logs`

View agent execution logs.

```bash
aofctl agent logs <name> [flags]
```

**Flags:**
- `-f, --follow` - Stream logs in real-time
- `--tail int` - Number of recent lines (default: 100)
- `--since string` - Show logs since timestamp (e.g., "1h", "2024-01-20")
- `--filter string` - Filter logs by pattern

**Examples:**
```bash
# View recent logs
aofctl agent logs my-agent

# Follow logs
aofctl agent logs my-agent -f

# Last 50 lines
aofctl agent logs my-agent --tail 50

# Since 1 hour ago
aofctl agent logs my-agent --since 1h

# Filter for errors
aofctl agent logs my-agent --filter error
```

---

### `aofctl agent validate`

Validate agent YAML without applying.

```bash
aofctl agent validate -f <file> [flags]
```

**Examples:**
```bash
aofctl agent validate -f my-agent.yaml
```

**Output:**
```
✓ YAML syntax valid
✓ API version supported
✓ Required fields present
✓ Model format correct (openai:gpt-4)
✓ Tools configuration valid
✓ Memory configuration valid

Warnings:
  - Temperature 1.5 is high, consider 0.3-0.7 for operations

Agent is valid and ready to apply.
```

---

## Fleet Commands

Manage agent fleets (teams of agents).

### `aofctl fleet create`

Create a new agent fleet.

```bash
aofctl fleet create <name> [flags]
```

**Flags:**
- `-f, --file string` - Fleet YAML file
- `--agents strings` - Agent names to include

**Examples:**
```bash
# From file
aofctl fleet create -f review-team.yaml

# Ad-hoc fleet
aofctl fleet create code-reviewers --agents security-agent,style-agent,perf-agent
```

---

### `aofctl fleet apply`

Apply fleet configuration.

```bash
aofctl fleet apply -f <file>
```

---

### `aofctl fleet scale`

Scale fleet size.

```bash
aofctl fleet scale <name> --replicas <count>
```

**Examples:**
```bash
aofctl fleet scale review-team --replicas 5
```

---

### `aofctl fleet exec`

Execute task with fleet.

```bash
aofctl fleet exec <name> <message> [flags]
```

**Flags:**
- `--aggregation string` - all|consensus|summary|first (default: all)

**Examples:**
```bash
# Get all responses
aofctl fleet exec review-team "Review this code" --aggregation all

# Majority vote
aofctl fleet exec review-team "Is this secure?" --aggregation consensus
```

---

### `aofctl fleet status`

Get fleet status.

```bash
aofctl fleet status <name>
```

**Output:**
```
Fleet: code-review-team
Agents: 3/3 ready

Agents:
  security-reviewer    openai:gpt-4        Running   2d
  performance-reviewer anthropic:claude-3-5-sonnet-20241022   Running   2d
  style-reviewer       ollama:llama3       Running   2d

Recent Tasks:
  2024-01-20 15:00:00  Code review PR#123   Completed  3.2s
  2024-01-20 14:30:00  Security audit       Completed  5.1s
```

---

## Flow Commands

Manage AgentFlow workflows.

### `aofctl flow apply`

Apply flow configuration.

```bash
aofctl flow apply -f <file> [flags]
```

**Examples:**
```bash
aofctl flow apply -f incident-response.yaml
```

---

### `aofctl flow run`

Execute a flow.

```bash
aofctl flow run <name> [flags]
```

**Flags:**
- `--daemon` - Run in background
- `--input string` - Trigger input data (JSON)
- `--var strings` - Set variables (key=value)

**Examples:**
```bash
# Run once
aofctl flow run my-flow

# Background daemon
aofctl flow run webhook-handler --daemon

# With input
aofctl flow run my-flow --input '{"data": "value"}'

# With variables
aofctl flow run my-flow --var NAMESPACE=production --var CLUSTER=us-east-1
```

---

### `aofctl flow status`

Get flow execution status.

```bash
aofctl flow status <name> [flags]
```

**Flags:**
- `--execution-id string` - Specific execution

**Examples:**
```bash
# Current status
aofctl flow status my-flow

# Specific execution
aofctl flow status my-flow --execution-id abc123
```

**Output:**
```
Flow: incident-response
Status: Running
Started: 2024-01-20 15:30:00
Duration: 2m 15s

Nodes:
  ✓ parse-alert        Transform   Completed  0.1s
  ✓ diagnose          Agent       Completed  45.2s
  ⟳ remediate         Agent       Running    1m 30s
  ⋯ verify            Agent       Pending    -
  ⋯ notify            Slack       Pending    -

Progress: 2/5 nodes completed (40%)
```

---

### `aofctl flow logs`

View flow execution logs.

```bash
aofctl flow logs <name> [flags]
```

**Flags:**
- `-f, --follow` - Stream logs
- `--node string` - Filter by node ID
- `--execution-id string` - Specific execution

**Examples:**
```bash
# All logs
aofctl flow logs my-flow

# Follow
aofctl flow logs my-flow -f

# Specific node
aofctl flow logs my-flow --node remediate

# Specific execution
aofctl flow logs my-flow --execution-id abc123
```

---

### `aofctl flow visualize`

Generate flow visualization.

```bash
aofctl flow visualize <name> [flags]
```

**Flags:**
- `-o, --output string` - Output file (default: stdout)
- `--format string` - dot|mermaid|svg (default: dot)

**Examples:**
```bash
# Generate DOT format
aofctl flow visualize my-flow > flow.dot

# Convert to PNG
aofctl flow visualize my-flow | dot -Tpng > flow.png

# Mermaid format
aofctl flow visualize my-flow --format mermaid
```

---

### `aofctl flow pause`

Pause running flow.

```bash
aofctl flow pause <name>
```

---

### `aofctl flow resume`

Resume paused flow.

```bash
aofctl flow resume <name>
```

---

### `aofctl flow cancel`

Cancel running flow.

```bash
aofctl flow cancel <name> [--execution-id string]
```

---

## Config Commands

Manage aofctl configuration.

### `aofctl config view`

Display current config.

```bash
aofctl config view
```

**Output:**
```yaml
current-context: production
contexts:
  - name: production
    server: https://aof-prod.company.com
    namespace: default
  - name: staging
    server: https://aof-staging.company.com
    namespace: default
```

---

### `aofctl config set-context`

Set current context.

```bash
aofctl config set-context <name> [flags]
```

**Flags:**
- `--server string` - Server URL
- `--namespace string` - Default namespace

**Examples:**
```bash
# Switch context
aofctl config set-context production

# Create new context
aofctl config set-context staging --server https://aof-staging.company.com
```

---

### `aofctl config get-contexts`

List available contexts.

```bash
aofctl config get-contexts
```

**Output:**
```
CURRENT   NAME         SERVER                              NAMESPACE
*         production   https://aof-prod.company.com        default
          staging      https://aof-staging.company.com     default
          local        http://localhost:8080               default
```

---

## Utility Commands

### `aofctl version`

Show version information.

```bash
aofctl version
```

**Output:**
```
aofctl version: v1.0.0
Rust version: 1.75.0
Server version: v1.0.0
```

---

### `aofctl completion`

Generate shell completion scripts.

```bash
aofctl completion <shell>
```

**Supported Shells:**
- bash
- zsh
- fish
- powershell

**Examples:**
```bash
# Bash
aofctl completion bash > /etc/bash_completion.d/aofctl

# Zsh
aofctl completion zsh > /usr/local/share/zsh/site-functions/_aofctl

# Fish
aofctl completion fish > ~/.config/fish/completions/aofctl.fish
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AOF_CONFIG` | Config file path | `~/.aof/config.yaml` |
| `AOF_CONTEXT` | Current context | `default` |
| `AOF_NAMESPACE` | Default namespace | `default` |
| `AOF_SERVER` | Server URL | `http://localhost:8080` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `GROQ_API_KEY` | Groq API key | - |

---

## Examples

### Complete Workflow

```bash
# 1. Create agent
cat > my-agent.yaml <<EOF
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: openai:gpt-4
  instructions: "You are a K8s expert"
  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl]
EOF

# 2. Validate
aofctl agent validate -f my-agent.yaml

# 3. Apply
aofctl agent apply -f my-agent.yaml

# 4. Test
aofctl agent chat k8s-helper "Show me all pods"

# 5. View logs
aofctl agent logs k8s-helper --tail 20

# 6. Get status
aofctl agent describe k8s-helper
```

---

## See Also

- [Agent Spec](agent-spec.md)
- [AgentFlow Spec](agentflow-spec.md)
- [Examples](../examples/)
