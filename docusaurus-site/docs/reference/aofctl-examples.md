# aofctl Usage Examples - kubectl-Style Commands

This guide demonstrates the new kubectl-compatible command structure for `aofctl`.

## Quick Reference

```bash
# List all available resource types
aofctl api-resources

# Get resources
aofctl get agents
aofctl get agent my-agent
aofctl get workflows --all-namespaces

# Run resources
aofctl run agent config.yaml --input "query"
aofctl run workflow workflow.yaml

# Apply configurations
aofctl apply -f agent-config.yaml
aofctl apply -f workflow.yaml -n production

# Delete resources
aofctl delete agent my-agent
aofctl delete workflow my-workflow -n staging

# Describe resources
aofctl describe agent my-agent
aofctl describe workflow my-workflow

# Get logs
aofctl logs agent my-agent
aofctl logs agent my-agent --follow
aofctl logs job batch-job --tail 100

# Execute commands
aofctl exec agent my-agent -- ps aux
aofctl exec workflow my-workflow -- /bin/bash
```

## Resource Types

### Core Resources

#### Agents
```bash
# List all agents
aofctl get agents

# Get specific agent
aofctl get agent my-agent

# Get agents in all namespaces
aofctl get agents --all-namespaces

# Get agent as JSON
aofctl get agent my-agent -o json

# Run an agent
aofctl run agent examples/simple-agent.yaml --input "What is AI?"

# Delete an agent
aofctl delete agent my-agent
```

#### Workflows
```bash
# List workflows
aofctl get workflows
aofctl get wf  # using short name

# Get specific workflow
aofctl get workflow my-workflow

# Run a workflow
aofctl run workflow examples/workflow.yaml

# Apply workflow configuration
aofctl apply -f workflow-config.yaml
```

#### Tools
```bash
# List all tools
aofctl get tools
aofctl get t  # using short name

# Get specific tool
aofctl get tool claude-flow
```

#### Configs
```bash
# List configs (cluster-wide)
aofctl get configs
aofctl get cfg  # using short name

# Apply a config
aofctl apply -f cluster-config.yaml
```

### Runtime Resources

#### Deployments
```bash
# List deployments
aofctl get deployments
aofctl get deploy  # using short name

# Get specific deployment
aofctl get deployment my-deployment

# Apply deployment
aofctl apply -f deployment.yaml -n production
```

#### Templates
```bash
# List templates
aofctl get templates
aofctl get tmpl  # using short name

# Apply a template
aofctl apply -f agent-template.yaml
```

### MCP Resources

#### MCP Servers
```bash
# List MCP servers (cluster-wide)
aofctl get mcpservers

# Get specific MCP server
aofctl get mcpserver claude-flow

# Legacy command (still works)
aofctl tools --server npx --args "claude-flow@alpha mcp start"
```

#### MCP Tools
```bash
# List MCP tools
aofctl get mcptools

# Get specific tool
aofctl get mcptool swarm_init
```

### Execution Resources

#### Jobs
```bash
# List jobs
aofctl get jobs
aofctl get j  # using short name

# Get specific job
aofctl get job batch-processing

# Run a job
aofctl run job batch-job.yaml

# Get job logs
aofctl logs job batch-processing
aofctl logs job batch-processing --tail 50
```

#### Tasks
```bash
# List tasks
aofctl get tasks
aofctl get tsk  # using short name

# Get specific task
aofctl get task data-processing-1
```

### Storage Resources

#### Memory
```bash
# List memory stores
aofctl get memories
aofctl get mem  # using short name

# Get specific memory
aofctl get memory agent-context
```

#### State
```bash
# List states
aofctl get states
aofctl get st  # using short name

# Get specific state
aofctl get state workflow-state-1
```

## Output Formats

### Wide Format (Default)
```bash
aofctl get agents
# Output:
# NAME              STATUS    MODEL              AGE
# my-agent          Running   claude-sonnet-4    5m
```

### JSON Format
```bash
aofctl get agent my-agent -o json
# Output:
# {
#   "kind": "Agent",
#   "apiVersion": "v1",
#   "items": []
# }
```

### YAML Format
```bash
aofctl get agent my-agent -o yaml
# Output:
# kind: Agent
# apiVersion: v1
# items: []
```

### Name Only
```bash
aofctl get agents -o name
# Output:
# agent/my-agent
```

## Namespace Operations

### Default Namespace
```bash
# Uses 'default' namespace
aofctl get agents
aofctl delete agent my-agent
```

### Specific Namespace
```bash
# Target specific namespace
aofctl get agents -n production
aofctl apply -f agent.yaml -n staging
aofctl delete agent my-agent -n development
```

### All Namespaces
```bash
# View resources across all namespaces
aofctl get agents --all-namespaces

# Output includes NAMESPACE column:
# NAMESPACE    NAME              STATUS    MODEL              AGE
# default      agent-1           Running   claude-sonnet-4    5m
# production   agent-2           Running   claude-opus-3      2h
```

## Advanced Examples

### Running Agents with Different Configurations

```bash
# Run agent with custom input
aofctl run agent examples/simple-agent.yaml \
  --input "Analyze this data" \
  --output json

# Run agent with text output (default)
aofctl run agent examples/simple-agent.yaml \
  --input "Generate code" \
  --output text

# Run agent with YAML output
aofctl run agent examples/simple-agent.yaml \
  --input "Create report" \
  --output yaml
```

### Applying Multiple Configurations

```bash
# Apply agent configuration
aofctl apply -f configs/agent-config.yaml

# Apply to specific namespace
aofctl apply -f configs/production-agent.yaml -n production

# Apply workflow
aofctl apply -f configs/data-pipeline.yaml
```

### Viewing Logs

```bash
# View logs from an agent
aofctl logs agent my-agent

# Follow logs in real-time
aofctl logs agent my-agent --follow

# View last 100 lines
aofctl logs agent my-agent --tail 100

# View job logs
aofctl logs job batch-processing --tail 50
```

### Executing Commands

```bash
# Execute a command in an agent
aofctl exec agent my-agent -- ps aux

# Open a shell in an agent
aofctl exec agent my-agent -- /bin/bash

# Run a script
aofctl exec workflow my-workflow -- python script.py
```

### Resource Discovery

```bash
# List all resource types
aofctl api-resources

# Filter resource types (future feature)
# aofctl api-resources --namespaced=true
# aofctl api-resources --api-group=apps
```

## Common Workflows

### 1. Deploy a New Agent

```bash
# 1. Create agent configuration
cat > my-agent.yaml <<EOF
name: my-agent
model: claude-sonnet-4
max_iterations: 10
temperature: 0.7
system_prompt: "You are a helpful AI assistant"
tools:
  - calculator
  - web_search
EOF

# 2. Apply the configuration
aofctl apply -f my-agent.yaml

# 3. Verify it was created
aofctl get agent my-agent

# 4. Run the agent
aofctl run agent my-agent.yaml --input "What is 2+2?"
```

### 2. Monitor Agent Execution

```bash
# Check agent status
aofctl get agent my-agent

# Describe agent details
aofctl describe agent my-agent

# View logs
aofctl logs agent my-agent --follow
```

### 3. Update Agent Configuration

```bash
# Edit the configuration file
vim my-agent.yaml

# Apply the updated configuration
aofctl apply -f my-agent.yaml

# Verify the update
aofctl get agent my-agent
```

### 4. Clean Up Resources

```bash
# Delete specific agent
aofctl delete agent my-agent

# Delete agent in specific namespace
aofctl delete agent my-agent -n staging

# View remaining agents
aofctl get agents --all-namespaces
```

## Migration from Old Commands

### Before (Noun-Verb Pattern)
```bash
# Old commands (still work but deprecated)
aofctl agent run --config agent.yaml --input "query"
aofctl agent get my-agent
aofctl tools --server npx --args "claude-flow@alpha mcp start"
```

### After (Verb-Noun Pattern)
```bash
# New kubectl-style commands
aofctl run agent agent.yaml --input "query"
aofctl get agent my-agent
aofctl get mcptools
```

## Version Information

```bash
# Show version
aofctl version

# Output:
# aofctl version: 0.1.0
# aof-core version: 0.1.0
# MCP version: 1.0.0
```

## Getting Help

```bash
# General help
aofctl --help

# Command-specific help
aofctl get --help
aofctl run --help
aofctl apply --help

# Subcommand help
aofctl get agent --help
```

## Best Practices

1. **Use short names for common operations:**
   ```bash
   aofctl get ag        # instead of 'agents'
   aofctl get wf        # instead of 'workflows'
   ```

2. **Always specify namespace in production:**
   ```bash
   aofctl apply -f config.yaml -n production
   ```

3. **Use output formats for automation:**
   ```bash
   aofctl get agents -o json | jq '.items[].metadata.name'
   ```

4. **Follow logs for debugging:**
   ```bash
   aofctl logs agent my-agent --follow
   ```

5. **Use describe for detailed information:**
   ```bash
   aofctl describe agent my-agent
   ```

## Troubleshooting

### Unknown Resource Type
```bash
$ aofctl get unknown-type
Error: Unknown resource type: unknown-type

# Solution: Check available types
$ aofctl api-resources
```

### Resource Not Found
```bash
$ aofctl get agent non-existent
# Currently shows placeholder data
# Future: Will show "Error: agent 'non-existent' not found"
```

### Invalid Configuration
```bash
$ aofctl apply -f invalid.yaml
Error: Failed to parse configuration file: ...

# Solution: Validate your YAML syntax and structure
```

## Future Enhancements

The following features are planned:

1. **Watch mode:** `aofctl get agents --watch`
2. **Label selectors:** `aofctl get agents -l env=production`
3. **Field selectors:** `aofctl get agents --field-selector status=Running`
4. **Dry run:** `aofctl apply -f config.yaml --dry-run`
5. **Edit command:** `aofctl edit agent my-agent`
6. **Patch command:** `aofctl patch agent my-agent --type merge -p '{"spec":{"replicas":3}}'`
7. **Shell completion:** `aofctl completion bash`

## Contributing

To add new resource types:

1. Update `/aof/crates/aofctl/src/resources.rs`
2. Add the resource to `ResourceType` enum
3. Implement name, plural, short names, API version
4. Add handling in relevant commands

Example:
```rust
ResourceType::NewResource => {
    // Define properties
}
```

---

For more information, see the AOF framework documentation:
- [Getting Started](/docs/getting-started) - Quick start guide
- [Core Concepts](/docs/concepts) - Key architecture concepts
