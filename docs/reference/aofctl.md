# aofctl CLI Reference

Complete command reference for the AOF CLI tool.

## Command Structure

aofctl follows a verb-first pattern (like kubectl):

```bash
aofctl <verb> <resource_type> [name] [flags]
```

## Available Commands

| Command | Description | Status |
|---------|-------------|--------|
| `run` | Run an agent or workflow | ✅ Implemented |
| `get` | Get/list resources | ✅ Implemented |
| `apply` | Apply configuration from file | ✅ Implemented |
| `delete` | Delete resources | ✅ Implemented |
| `describe` | Describe resources in detail | ✅ Implemented |
| `logs` | Get logs from a resource | ✅ Implemented |
| `exec` | Execute a command in a resource | ✅ Implemented |
| `api-resources` | List available API resources | ✅ Implemented |
| `version` | Show version information | ✅ Implemented |
| `serve` | Start the trigger webhook server (daemon mode) | ✅ Implemented |

> **Note**: Fleet, Flow, Config, and Completion commands are planned for future releases.

---

## apply

Apply configuration from a YAML file.

```bash
aofctl apply -f <file> [flags]
```

**Flags:**
- `-f, --file string` - Path to YAML file (required)
- `-n, --namespace string` - Namespace for resources

**Examples:**
```bash
# Apply agent configuration
aofctl apply -f my-agent.yaml

# Apply with namespace
aofctl apply -f my-agent.yaml -n production
```

---

## get

List or retrieve resources.

```bash
aofctl get <resource_type> [name] [flags]
```

**Flags:**
- `-o, --output string` - Output format: json|yaml|wide|name (default: wide)
- `--all-namespaces` - Show resources across all namespaces

**Examples:**
```bash
# List all agents
aofctl get agent

# Get specific agent
aofctl get agent my-agent

# Get as YAML
aofctl get agent my-agent -o yaml

# All namespaces
aofctl get agent --all-namespaces
```

**Output:**
```
NAME              MODEL           STATUS    AGE
k8s-helper        openai:gpt-4    Running   5d
slack-bot         anthropic:claude-3-5-sonnet-20241022   Running   2d
incident-responder openai:gpt-4   Running   1d
```

---

## describe

Show detailed information about a resource.

```bash
aofctl describe <resource_type> <name> [flags]
```

**Flags:**
- `-n, --namespace string` - Namespace

**Examples:**
```bash
aofctl describe agent my-agent
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

## run

Run an agent or workflow.

```bash
aofctl run <resource_type> <name_or_file> [flags]
```

**Flags:**
- `-i, --input string` - Input/query for the agent
- `-o, --output string` - Output format: json|yaml|text (default: text)

**Examples:**
```bash
# Interactive mode
aofctl run agent my-agent.yaml

# With query
aofctl run agent my-agent.yaml -i "Show me all pods"

# Run workflow
aofctl run workflow incident-response.yaml
```

**Example Output:**
```bash
$ aofctl run agent k8s-helper.yaml --input "Show me all pods"

Agent: k8s-helper
Result: Here are the pods in the default namespace:

NAME                        READY   STATUS    RESTARTS   AGE
nginx-deployment-abc123     2/2     Running   0          5d
postgres-0                  1/1     Running   0          10d

All pods are healthy!
```

---

## exec

Execute a command in a resource.

```bash
aofctl exec <resource_type> <name> [command...] [flags]
```

**Examples:**
```bash
# Execute command
aofctl exec agent k8s-helper -- kubectl get pods
```

---

## delete

Delete a resource.

```bash
aofctl delete <resource_type> <name> [flags]
```

**Flags:**
- `-n, --namespace string` - Namespace

**Examples:**
```bash
# Delete agent
aofctl delete agent my-agent
```

---

## logs

View resource logs.

```bash
aofctl logs <resource_type> <name> [flags]
```

**Flags:**
- `-f, --follow` - Stream logs in real-time
- `--tail int` - Number of recent lines

**Examples:**
```bash
# View recent logs
aofctl logs agent my-agent

# Follow logs
aofctl logs agent my-agent -f

# Last 50 lines
aofctl logs agent my-agent --tail 50
```

---

## serve

Start the trigger webhook server (daemon mode) for running agents as a service.

```bash
aofctl serve [flags]
```

**Flags:**
- `-c, --config string` - Configuration file (YAML)
- `-p, --port int` - Port to listen on (overrides config)
- `--host string` - Host to bind to (default: 0.0.0.0)
- `--agents-dir string` - Directory containing agent YAML files

**Examples:**
```bash
# Start server with default settings
aofctl serve

# Serve with custom port and agents directory
aofctl serve --port 8080 --agents-dir ./agents

# Use a configuration file
aofctl serve -c daemon-config.yaml
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
aofctl version: 0.1.11
aof-core version: 0.1.11
MCP version: 2024-11-05
```

---

## Planned Features (Not Yet Implemented)

The following commands are planned for future releases:

### Fleet Commands (Coming Soon)

AgentFleet enables multi-agent coordination:

- `aofctl fleet create` - Create a new agent fleet
- `aofctl fleet apply` - Apply fleet configuration
- `aofctl fleet scale` - Scale fleet size
- `aofctl fleet exec` - Execute task with fleet
- `aofctl fleet status` - Get fleet status

### Flow Commands (Coming Soon)

AgentFlow enables workflow orchestration:

- `aofctl flow apply` - Apply flow configuration
- `aofctl flow run` - Execute a flow
- `aofctl flow status` - Get flow execution status
- `aofctl flow logs` - View flow execution logs
- `aofctl flow visualize` - Generate flow visualization
- `aofctl flow pause/resume/cancel` - Control flow execution

### Config Commands (Coming Soon)

- `aofctl config view` - Display current config
- `aofctl config set-context` - Set current context
- `aofctl config get-contexts` - List available contexts

### Completion (Coming Soon)

- `aofctl completion <shell>` - Generate shell completion scripts

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `GOOGLE_API_KEY` | Google Gemini API key | - |
| `GROQ_API_KEY` | Groq API key | - |
| `KUBECONFIG` | Kubernetes config path | `~/.kube/config` |

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
  model: google:gemini-2.5-flash
  instructions: "You are a K8s expert. Help users with kubectl commands."
  tools:
    - shell
EOF

# 2. Run the agent
aofctl run agent my-agent.yaml --input "Show me all pods"

# 3. Run with JSON output
aofctl run agent my-agent.yaml --input "Check deployment status" --output json

# 4. List available agents
aofctl get agent

# 5. Describe agent details
aofctl describe agent k8s-helper
```

### Start as Daemon Service

```bash
# Start the server with agents directory
aofctl serve --agents-dir ./agents --port 8080

# The server will expose agents via HTTP API
# Agents can be triggered via webhooks
```

---

## See Also

- [Agent Spec](agent-spec.md)
- [AgentFlow Spec](agentflow-spec.md) (specification for planned features)
- [Examples](../examples/)
