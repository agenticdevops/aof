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
- API key for Google Gemini, OpenAI, Anthropic, or Ollama
  - **Google Gemini (recommended)**: Get key at [Google AI Studio](https://aistudio.google.com/apikey), set `GOOGLE_API_KEY` environment variable
  - OpenAI: Set `OPENAI_API_KEY`
  - Anthropic: Set `ANTHROPIC_API_KEY`
  - Ollama: No key needed, runs locally

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
  model: google:gemini-2.0-flash

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
# Interactive mode - recommended for testing
aofctl run agent k8s-helper.yaml

# Or with a single query
aofctl run agent k8s-helper.yaml --input "How do I check if my deployment is healthy?"
```

The agent will explain what steps to follow, but can't actually run kubectl yet. Let's add that.

## Step 2: Add Instructions for Common Tasks

Enhance the agent's instructions to guide kubectl usage effectively:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.0-flash

  instructions: |
    You are an expert Kubernetes operations assistant for DevOps engineers.

    Your role:
    - Help users manage Kubernetes clusters with kubectl
    - Troubleshoot pod, deployment, and service issues
    - Explain K8s concepts clearly and concisely
    - Always explain what a command does before running it

    When providing kubectl commands:
    - Always specify the namespace unless it's default
    - Suggest --dry-run for potentially destructive operations
    - Show the command first, then explain each part
    - Provide examples of successful vs. failing outputs

    Guidelines:
    - Ask for namespace if not specified
    - Provide YAML examples when helpful
    - Keep responses practical and actionable
    - Link to official Kubernetes docs when appropriate
```

Test it:
```bash
aofctl run agent k8s-helper.yaml
```

Try asking:
```
> How do I check if my deployment is healthy?
> Show me the command to list all pods in the default namespace
> What's a StatefulSet and when should I use it?
```

The agent will provide detailed explanations with kubectl commands.

## Step 3: Add GitHub Knowledge Integration

Enhance the agent to also help with GitHub repositories and DevOps workflows:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.0-flash

  instructions: |
    You are an expert DevOps engineer assistant covering both Kubernetes and GitHub.

    Your responsibilities:
    1. **Kubernetes Operations:**
       - Help users manage Kubernetes clusters with kubectl
       - Troubleshoot pod, deployment, and service issues
       - Explain K8s concepts clearly and concisely
       - Always explain what a command does before running it

    2. **GitHub Repository Assistance:**
       - Help understand Kubernetes project structure on GitHub
       - Explain GitHub workflows and CI/CD practices
       - Provide guidance on open source contributions
       - Assist with repository insights and best practices

    3. **Guidelines:**
       - Ask for namespace if not specified
       - Suggest --dry-run for destructive operations
       - Provide YAML examples when helpful
       - Keep responses practical and actionable
```

Test it:
```bash
aofctl run agent k8s-helper.yaml
```

Try asking:
```
> How do I check if my ArgoCD deployment is healthy?
> What are the main areas to contribute to in Kubernetes?
> What's the difference between a Pod and a Deployment?
```

**Note on MCP Servers:** While the tutorial demonstrates agent capabilities, actual MCP server integration (for structured tool execution) requires setting up servers like:
- `kubectl-ai --mcp-server` for advanced kubectl operations
- Official GitHub MCP server for repository queries
- Prometheus MCP server for monitoring metrics

These are advanced features for later in your journey.

## Step 4: Configure Model Parameters

Fine-tune the model behavior for more deterministic responses:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.0-flash

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
```

Test it with kubectl commands:
```bash
aofctl run agent k8s-helper.yaml
```

Try asking:
```
> Show me how to check if a deployment is healthy in the default namespace
> What's the command to list all running pods?
> Explain the difference between a Service and an Ingress
```

## Step 5: Extend with GitHub Knowledge

Enhance the agent to help with Kubernetes resources and DevOps practices:

Create `k8s-github-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-github-helper
  labels:
    purpose: operations
    team: platform

spec:
  model: google:gemini-2.0-flash

  model_config:
    temperature: 0.3
    max_tokens: 2000

  instructions: |
    You are an expert DevOps engineer assistant covering both Kubernetes and GitHub.

    Your responsibilities:
    1. **Kubernetes Operations:**
       - Help users manage Kubernetes clusters with kubectl
       - Troubleshoot pod, deployment, and service issues
       - Explain K8s concepts clearly and concisely
       - Always explain what a command does before running it

    2. **GitHub Repository Assistance:**
       - Help understand Kubernetes project structure on GitHub
       - Explain GitHub workflows and CI/CD practices
       - Provide guidance on open source contributions
       - Assist with repository insights and best practices

    3. **Guidelines:**
       - Ask for namespace if not specified
       - Suggest --dry-run for destructive operations
       - Provide YAML examples when helpful
       - Keep responses practical and actionable
```

Test it:
```bash
aofctl run agent k8s-github-agent.yaml
```

Try asking:
```
> How do I check if my ArgoCD deployment is healthy?
> What are the main areas to contribute to in Kubernetes?
> Show me the kubectl command to view deployment events
```

## Step 6: Advanced Usage with kubectl Commands

The agent can provide detailed kubectl commands and explanations:

```bash
# Interactive mode - great for learning
aofctl run agent k8s-helper.yaml

# Example interaction:
# > Show me all pods in kube-system namespace
# > What does a ConfigMap do?
# > How do I scale a deployment to 5 replicas?
```

The agent will explain each command before you run it, helping you understand Kubernetes better.

## Complete Final Agent

Here's the full production-ready configuration that combines all features:

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
  model: google:gemini-2.0-flash

  model_config:
    temperature: 0.3
    max_tokens: 2000
    top_p: 0.9

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
    - Reference Kubernetes official documentation when appropriate
```

## Testing Your Agent

Test the agent with various Kubernetes questions:

```bash
aofctl run agent k8s-helper.yaml
```

Example interactions:
```
> Show me all pods in the kube-system namespace
> Is the coredns deployment healthy?
> What's the kubectl command to view deployment events?
> Explain the difference between a Service and an Ingress
> How do I scale a deployment to 5 replicas?
> Show me how to check pod logs
```

The agent will provide detailed explanations and kubectl commands for all your Kubernetes operations.

## Next Steps

You now have a production-ready Kubernetes assistant! Here's what to try next:

1. **Build a Slack bot**: [Slack Bot Tutorial](slack-bot.md)
2. **Create an incident responder**: [Incident Response Tutorial](incident-response.md)
3. **Add more tools**: Check the [Agent Spec Reference](../reference/agent-spec.md)
4. **Team it up**: Learn about [AgentFleets](../concepts.md#agentfleet)

## Troubleshooting

### "API key not found" error
```bash
# Make sure GOOGLE_API_KEY is set
echo $GOOGLE_API_KEY

# If empty, set it in your shell
export GOOGLE_API_KEY=AIza...

# Or add to ~/.zshrc/.bashrc for persistence
echo 'export GOOGLE_API_KEY=AIza...' >> ~/.zshrc
source ~/.zshrc
```

### Agent not responding
```bash
# Check aofctl is installed
aofctl version

# Verify the agent YAML is valid
cat k8s-helper.yaml

# Run with verbose output
aofctl run agent k8s-helper.yaml
```

### kubectl commands not working
```bash
# Check kubectl is installed and accessible
which kubectl

# Verify cluster access
kubectl cluster-info

# Check current context
kubectl config current-context

# List available contexts
kubectl config get-contexts
```

---

**🎉 Congratulations!** You've built your first production agent. → [Next: Build a Slack Bot](slack-bot.md)
