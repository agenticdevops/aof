# Tutorial: Build Your First Agent

In this tutorial, you'll build a production-ready Kubernetes helper agent from scratch.

**What you'll learn:**
- Define Agent specifications
- Configure models and providers
- Add tools (Shell, MCP)
- Manage memory and context
- Deploy and interact with agents

**Time:** 15 minutes

## Prerequisites

- `aofctl` installed ([Getting Started](../getting-started.md))
- `kubectl` installed (for K8s tools)
- API key for OpenAI, Anthropic, or Ollama

## Step 1: Basic Agent Definition

Create `k8s-helper.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable
```

Test it:
```bash
aofctl run agent k8s-helper.yaml --input "How do I check if my deployment is healthy?"
```

The agent will explain but can't actually run kubectl yet. Let's add that.

## Step 2: Add Shell Tool

Update `k8s-helper.yaml` to add kubectl access:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
          - k9s
        working_directory: /tmp
        timeout_seconds: 30
```

Test it again:
```bash
aofctl run agent k8s-helper.yaml --input "Show me all pods in the default namespace"
```

The agent will run `kubectl get pods -n default` and explain the output.

## Step 3: Add MCP Server (kubectl-mcp)

For more structured Kubernetes access, add the kubectl MCP server:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        working_directory: /tmp
        timeout_seconds: 30

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"
```

The MCP server provides structured tools for:
- Listing resources
- Describing resources
- Getting logs
- Executing commands in pods

## Step 4: Add Memory for Context

Let's make the agent remember conversation context:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        working_directory: /tmp
        timeout_seconds: 30

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"

  memory:
    type: File
    config:
      path: ./k8s-helper-memory.json
      max_messages: 50
```

Now the agent will remember your previous questions in the same session.

## Step 5: Configure Model Parameters

Fine-tune the model behavior:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.5-flash

  model_config:
    temperature: 0.3        # Lower = more deterministic
    max_tokens: 2000        # Response length limit
    top_p: 0.9             # Nucleus sampling

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        working_directory: /tmp
        timeout_seconds: 30

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"

  memory:
    type: File
    config:
      path: ./k8s-helper-memory.json
      max_messages: 50
```

## Step 6: Run the Agent

Run the agent with different queries:

```bash
# Run with a query
aofctl run agent k8s-helper.yaml --input "Show me failing pods"

# Run with output format
aofctl run agent k8s-helper.yaml --input "Scale the nginx deployment to 3 replicas" --output json
```

## Step 7: View Agent Configuration

```bash
# List agents from a directory
aofctl get agent

# Describe agent details
aofctl describe agent k8s-helper

# View logs
aofctl logs agent k8s-helper
```

## Step 8: Monitor and Debug

```bash
# View agent logs (if using the daemon serve mode)
aofctl logs agent k8s-helper

# Follow logs in real-time
aofctl logs agent k8s-helper -f

# Check memory file (if using file-based memory)
ls -lh k8s-helper-memory.json
```

## Advanced: Add HTTP Tool

Let's make the agent able to check service endpoints:

```yaml
spec:
  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, helm]

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]

    - type: HTTP
      config:
        base_url: http://localhost
        timeout_seconds: 10
        allowed_methods: [GET, POST]
```

Now run:
```bash
aofctl run agent k8s-helper.yaml --input "Check if the nginx service on port 8080 is responding"
```

## Complete Final Agent

Here's the full production-ready configuration:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform
    env: production
  annotations:
    description: "Kubernetes operations assistant"
    owner: "platform-team@company.com"

spec:
  model: google:gemini-2.5-flash

  model_config:
    temperature: 0.3
    max_tokens: 2000

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    Guidelines:
    - Ask for namespace if not specified
    - Suggest --dry-run for destructive operations
    - Provide YAML examples when helpful
    - Keep responses practical and actionable
    - Check service health when troubleshooting

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        working_directory: /tmp
        timeout_seconds: 30

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"

    - type: HTTP
      config:
        base_url: http://localhost
        timeout_seconds: 10
        allowed_methods: [GET, POST]

  memory:
    type: File
    config:
      path: ./k8s-helper-memory.json
      max_messages: 50
```

## Testing Your Agent

Create a test script `test-agent.sh`:

```bash
#!/bin/bash

echo "Test 1: List pods"
aofctl run agent k8s-helper.yaml --input "Show all pods in kube-system"

echo -e "\nTest 2: Check deployment"
aofctl run agent k8s-helper.yaml --input "Is the coredns deployment healthy?"

echo -e "\nTest 3: Troubleshoot"
aofctl run agent k8s-helper.yaml --input "Find any pods that are not running"

echo -e "\nTest 4: Explain"
aofctl run agent k8s-helper.yaml --input "What's the difference between a Service and an Ingress?"
```

Run it:
```bash
chmod +x test-agent.sh
./test-agent.sh
```

## Next Steps

You now have a production-ready Kubernetes assistant! Here's what to try next:

1. **Build a Slack bot**: [Slack Bot Tutorial](slack-bot.md)
2. **Create an incident responder**: [Incident Response Tutorial](incident-response.md)
3. **Add more tools**: Check the [Agent Spec Reference](../reference/agent-spec.md)
4. **Team it up**: Learn about [AgentFleets](../concepts.md#agentfleet)

## Troubleshooting

### Agent can't run kubectl
```bash
# Check kubectl is in PATH
which kubectl

# Check KUBECONFIG
echo $KUBECONFIG

# Verify cluster access
kubectl cluster-info
```

### Memory file errors
```bash
# Check file permissions
ls -l k8s-helper-memory.json

# Reset memory by deleting the file
rm k8s-helper-memory.json

# Run agent again - it will create a new memory file
aofctl run agent k8s-helper.yaml --input "Hello"
```

### MCP server not starting
```bash
# Test MCP server manually
npx -y @modelcontextprotocol/server-kubectl

# Check Node.js version
node --version  # Should be v18+
```

---

**🎉 Congratulations!** You've built your first production agent. → [Next: Build a Slack Bot](slack-bot.md)
