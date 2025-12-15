---
sidebar_position: 1
---

# aofctl CLI Reference

Complete command reference for the AOF (Agent Orchestration Framework) CLI tool with full Kubernetes-style compatibility.

## Overview

`aofctl` follows the **kubectl-style verb-noun pattern** for all commands:

```bash
aofctl <verb> <resource-type> [resource-name] [flags]
```

Examples:
- `aofctl get agents` - List all agents
- `aofctl run agent my-agent --input "query"` - Execute an agent
- `aofctl delete workflow production-workflow` - Delete a workflow

---

## Global Flags

Available for all commands:

```bash
-n, --namespace string      Namespace to operate in (default: "default")
--all-namespaces            Show resources across all namespaces
-o, --output string         Output format: json|yaml|wide|name (default: "wide")
--verbose                   Enable verbose logging
--help                      Show help for command
--version                   Show version information
```

---

## Core Commands

### `aofctl get` - List or retrieve resources

List or get detailed information about resources.

**Syntax:**
```bash
aofctl get <resource-type> [resource-name] [flags]
```

**Resource Types:**
- `agents` / `agent` / `ag` - AI agents
- `workflows` / `workflow` / `wf` - Multi-step workflows
- `tools` / `tool` / `tl` - MCP tools
- `jobs` / `job` - One-time job executions
- `cronjobs` / `cronjob` - Scheduled executions
- `configs` / `config` - Configuration objects
- `secrets` / `secret` - Sensitive data
- `deployments` / `deployment` / `deploy` - Managed deployments

**Examples:**

```bash
# List all agents in current namespace
aofctl get agents

# Get a specific agent
aofctl get agent my-agent

# List all resources across all namespaces
aofctl get agents --all-namespaces

# Output as JSON
aofctl get agents -o json

# Output as YAML
aofctl get agent my-agent -o yaml

# Output only resource names
aofctl get agents -o name

# Use short name
aofctl get ag my-agent
```

**Output Formats:**
- `wide` (default) - Table format with additional columns
- `json` - JSON format
- `yaml` - YAML format
- `name` - Resource name only

---

### `aofctl run` - Execute agents and workflows

Execute agents or workflows with interactive or single-query mode.

**Syntax:**
```bash
aofctl run <resource-type> <config-file> [flags]
```

#### Interactive Mode (Default)

When no `--input` flag is provided, the agent runs in interactive REPL mode with a beautiful CLI:

```bash
aofctl run agent my-agent.yaml
```

Output:
```
============================================================
  🤖 Interactive Agent Console - my-agent
  Type your query and press Enter. Type 'exit' or 'quit' to exit.
============================================================

💬 You: What is the status?

⏳ Processing...
✓  Agent Response:
────────────────────────────────────────────────────────────
[Agent response here]
────────────────────────────────────────────────────────────

💬 You:
```

#### Single Query Mode

Use `--input` for single-execution or automation:

```bash
# Run an agent with a specific query
aofctl run agent my-agent-config.yaml --input "What is the status?"

# Run a workflow
aofctl run workflow my-workflow.yaml --input "Process this data"

# Specify output format
aofctl run agent config.yaml --input "query" --output json

# Run with custom namespace
aofctl run agent config.yaml -n production --input "query"
```

**Flags:**
- `-i, --input string` - Input/query for the resource (enables single-query mode)
- `-o, --output string` - Output format (json|yaml|text) [default: text]
- `-n, --namespace string` - Namespace to run in

---

### `aofctl apply` - Create or update resources

Apply configuration files to create or update resources.

**Syntax:**
```bash
aofctl apply -f <file-or-directory> [flags]
```

**Examples:**

```bash
# Apply a single resource file
aofctl apply -f agent.yaml

# Apply multiple files
aofctl apply -f agent.yaml -f workflow.yaml

# Apply all files in a directory
aofctl apply -f ./resources/

# Apply with specific namespace
aofctl apply -f agent.yaml -n production

# Dry run to validate without applying
aofctl apply -f agent.yaml --dry-run
```

**Flags:**
- `-f, --file string` - Path to file or directory (required)
- `-n, --namespace string` - Target namespace
- `--dry-run` - Validate without applying

---

### `aofctl delete` - Remove resources

Delete specific resources.

**Syntax:**
```bash
aofctl delete <resource-type> <resource-name> [flags]
```

**Examples:**

```bash
# Delete a specific agent
aofctl delete agent my-agent

# Delete a workflow
aofctl delete workflow my-workflow

# Delete from specific namespace
aofctl delete agent my-agent -n production

# Force delete without graceful shutdown
aofctl delete agent my-agent --force

# Delete with grace period
aofctl delete job my-job --grace-period=30
```

**Flags:**
- `--force` - Skip graceful termination
- `--grace-period int` - Seconds to wait before force delete (default: 30)
- `-n, --namespace string` - Target namespace

---

### `aofctl describe` - Show detailed resource information

Display detailed information about a specific resource.

**Syntax:**
```bash
aofctl describe <resource-type> <resource-name> [flags]
```

**Examples:**

```bash
# Describe an agent
aofctl describe agent my-agent

# Describe a workflow
aofctl describe workflow my-workflow

# From specific namespace
aofctl describe agent my-agent -n production
```

---

### `aofctl logs` - View resource logs

Display logs from agents, jobs, or other resources.

**Syntax:**
```bash
aofctl logs <resource-type> <resource-name> [flags]
```

**Examples:**

```bash
# View logs from an agent
aofctl logs agent my-agent

# View logs from a job
aofctl logs job my-job

# Follow logs (streaming)
aofctl logs agent my-agent --follow

# Show last 100 lines
aofctl logs agent my-agent --tail 100

# Show logs from past 10 minutes
aofctl logs agent my-agent --since 10m
```

**Flags:**
- `-f, --follow` - Follow log output (streaming)
- `--tail int` - Number of lines to show (default: 10)
- `--since string` - Show logs since duration (e.g., "10m", "1h")

---

### `aofctl exec` - Execute commands in resources

Execute commands inside running agents or containers.

**Syntax:**
```bash
aofctl exec <resource-type> <resource-name> -- <command> [args]
```

**Examples:**

```bash
# Execute a command in an agent
aofctl exec agent my-agent -- python script.py

# Interactive shell
aofctl exec agent my-agent -- bash

# Pass multiple arguments
aofctl exec agent my-agent -- python -c "print('hello')"
```

---

### `aofctl api-resources` - List all available resource types

Display information about all available resource types in the system.

**Syntax:**
```bash
aofctl api-resources [flags]
```

**Examples:**

```bash
# List all available resources
aofctl api-resources

# Show with wide format (additional columns)
aofctl api-resources -o wide

# Filter by namespace capability
aofctl api-resources --namespaced=true

# Filter by API group
aofctl api-resources --api-group=aof.io
```

**Output:**
```
NAME             SHORTNAMES   APIVERSION         NAMESPACED   KIND
agents           ag           v1                 true         Agent
workflows        wf,workflow  v1                 true         Workflow
tools            tl           mcp/v1             false        Tool
configs          cfg          v1                 true         Config
...
```

---

### `aofctl version` - Show version information

Display aofctl version and build information.

**Syntax:**
```bash
aofctl version
```

---

## Resource Type Details

### Agent

AI agents that can execute tasks and workflows.

**API Version:** `v1`
**Kind:** `Agent`
**Namespaced:** Yes

**Example YAML:**
```yaml
apiVersion: v1
kind: Agent
metadata:
  name: my-agent
  namespace: default
  labels:
    app: my-agent
    version: "1.0"
spec:
  model: claude-sonnet-4-5-20250929
  instructions: |
    You are a helpful assistant that analyzes data.
  tools:
    - name: search
      source: google-search
  memory:
    type: short-term
    size: 5000
status:
  phase: Running
  conditions:
    - type: Ready
      status: "True"
```

---

### Workflow

Multi-step orchestration of agents and tools.

**API Version:** `v1`
**Kind:** `Workflow`
**Namespaced:** Yes

**Example YAML:**
```yaml
apiVersion: v1
kind: Workflow
metadata:
  name: analysis-pipeline
  namespace: default
spec:
  steps:
    - name: gather-data
      resource:
        kind: Agent
        name: data-collector
    - name: analyze
      resource:
        kind: Agent
        name: analyzer
      dependsOn: gather-data
    - name: report
      resource:
        kind: Agent
        name: reporter
      dependsOn: analyze
```

---

### Tool

MCP tools available in the system.

**API Version:** `mcp/v1`
**Kind:** `Tool`
**Namespaced:** No

---

### Job

One-time job execution.

**API Version:** `batch/v1`
**Kind:** `Job`
**Namespaced:** Yes

---

### CronJob

Scheduled job execution.

**API Version:** `batch/v1`
**Kind:** `CronJob`
**Namespaced:** Yes

---

## Common Use Cases

### Create and run an agent

```bash
# 1. Create agent configuration
cat > my-agent.yaml << EOF
apiVersion: v1
kind: Agent
metadata:
  name: my-agent
spec:
  model: claude-sonnet-4-5-20250929
  instructions: "You are a helpful assistant"
EOF

# 2. Apply the configuration
aofctl apply -f my-agent.yaml

# 3. Execute the agent
aofctl run agent my-agent --input "What is 2+2?"
```

### List and manage agents

```bash
# List all agents
aofctl get agents

# Get specific agent details
aofctl get agent my-agent -o yaml

# Describe agent in detail
aofctl describe agent my-agent

# Delete agent
aofctl delete agent my-agent
```

### Work with workflows

```bash
# List workflows
aofctl get workflows

# Get workflow details
aofctl describe workflow my-workflow

# Run workflow
aofctl run workflow my-workflow.yaml --input "data"
```

### Monitor resources

```bash
# Watch agent logs
aofctl logs agent my-agent --follow

# View job status
aofctl get job my-job -o wide

# Check resource details
aofctl describe job my-job
```

---

## Kubernetes Compatibility

`aofctl` follows Kubernetes CLI conventions:

- **Verb-noun pattern:** `aofctl verb noun name`
- **Namespace support:** Use `-n` or `--namespace` flag
- **Output formats:** Support for json, yaml, table, and name formats
- **Resource discovery:** `api-resources` command shows all available resources
- **Global flags:** Consistent flags across all commands

---

## Migration from Old Pattern

If you're using the old noun-verb pattern, here's a quick reference:

| Old Command | New Command |
|------------|-------------|
| `aofctl agent run config.yaml` | `aofctl run agent config.yaml` |
| `aofctl agent get` | `aofctl get agents` |
| `aofctl agent get name` | `aofctl get agent name` |
| `aofctl workflow run config.yaml` | `aofctl run workflow config.yaml` |
| `aofctl tools list` | `aofctl get tools` |

---

## Examples Directory

Comprehensive examples are available in the `examples/` directory:

- `examples/agents/` - Agent configurations
- `examples/workflows/` - Workflow definitions
- `examples/jobs/` - Job specifications
- `examples/configs/` - Configuration examples

---

## Troubleshooting

### Command not found

Ensure aofctl is installed and in your PATH:
```bash
which aofctl
aofctl version
```

### Permission denied

Check namespace permissions and ensure you have access to the resource:
```bash
aofctl get agents -n default
```

### Resource not found

Verify the resource exists and check the namespace:
```bash
aofctl get agents --all-namespaces
aofctl describe agent my-agent
```

---

## Further Reading

- [Agent Specification](./agent-spec.md)
- [Workflow Tutorial](../tutorials/first-agent.md)
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.34/) (inspiration)

