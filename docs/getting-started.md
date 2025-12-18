# Getting Started with AOF

Get up and running with your first AI agent in 5 minutes.

## Prerequisites

### Required
- **API Key**: Get one from:
  - [OpenAI](https://platform.openai.com/api-keys) (ChatGPT, GPT-4)
  - [Anthropic](https://console.anthropic.com/) (Claude)
  - [Google AI Studio](https://aistudio.google.com/apikey) (Gemini)
  - Or use [Ollama](https://ollama.ai/) locally (no key needed)
- **Terminal**: Any Unix shell (bash, zsh, fish)

### Optional
- **Rust**: Only needed if building from source
- **kubectl**: For Kubernetes-related agents
- **Docker**: For containerized deployments

## Installation

### Step 1: Install aofctl

Choose your preferred method:

#### Option A: Binary Download (Recommended)
```bash
# Detect your platform and install
curl -sSL https://docs.aof.sh/install.sh | bash

# Verify installation
aofctl version
```

#### Option B: Cargo Install
```bash
cargo install aofctl

# Verify installation
aofctl version
```

#### Option C: Build from Source
```bash
git clone https://github.com/agenticdevops/aof.git
cd aof
cargo build --release
sudo cp target/release/aofctl /usr/local/bin/

# Verify installation
aofctl version
```

### Step 2: Configure API Keys

Set your LLM provider API key:

```bash
# OpenAI
export OPENAI_API_KEY=sk-...

# OR Anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# OR Google Gemini
export GOOGLE_API_KEY=AIza...

# OR Ollama (runs locally, no key needed)
# Just install: brew install ollama && ollama serve
```

**💡 Tip**: Add these to your `~/.zshrc` or `~/.bashrc` to persist across sessions.

## Create Your First Agent

### Step 3: Create an Agent YAML

Create a file called `hello-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: hello-assistant
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a friendly assistant that helps DevOps engineers.
    Keep responses concise and practical.
```

> **Note**: You can use any supported model:
> - `google:gemini-2.5-flash` - Google Gemini (requires `GOOGLE_API_KEY`)
> - `openai:gpt-4o` - OpenAI GPT-4 (requires `OPENAI_API_KEY`)
> - `anthropic:claude-3-5-sonnet-20241022` - Anthropic Claude (requires `ANTHROPIC_API_KEY`)
> - `ollama:llama3` - Local Ollama (no API key needed)

### Step 4: Run Your Agent

```bash
# Run with a query
aofctl run agent hello-agent.yaml --input "What's the difference between a Deployment and a StatefulSet?"

# You'll see output like:
# Agent: hello-assistant
# Result: A Deployment is designed for stateless applications...
```

### Step 5: Verify It Works

Your agent should respond with a clear explanation. If you see a response, congratulations! 🎉

## Add Some Tools

Let's make the agent more useful by adding shell access:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Kubernetes expert assistant. Help users run kubectl commands
    and troubleshoot their clusters. Always explain what commands do before running them.

  tools:
    - shell
    - kubectl
```

Save this as `k8s-agent.yaml` and run:

```bash
aofctl run agent k8s-agent.yaml --input "Show me all pods in the default namespace"
```

The agent will explain what it's doing and run `kubectl get pods -n default`.

## Create Your First Fleet

**AgentFleet** lets you run multiple agents in parallel for collaborative tasks. Here's a simple code review team:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-team
spec:
  agents:
    - name: security-reviewer
      role: worker
      replicas: 1
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on security issues: authentication, input validation,
          SQL injection, XSS, and secrets in code.
        tools:
          - read_file

    - name: quality-reviewer
      role: worker
      replicas: 1
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on code quality: readability, error handling,
          design patterns, and test coverage.
        tools:
          - read_file

  coordination:
    mode: peer
    distribution: round-robin
```

Save this as `code-review-fleet.yaml` and run:

```bash
aofctl run fleet code-review-fleet.yaml --input "Review: function add(a, b) { return a + b; }"
```

Both agents run in parallel and their responses are aggregated. See [Core Concepts](concepts.md) for more on coordination modes.

## Create Your First AgentFlow

**AgentFlow** enables event-driven workflows with multi-tenant routing. Here's a Slack bot that routes to different clusters:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-k8s-bot
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      channels: [production, prod-alerts]  # Only these channels
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: default
    cluster: prod-cluster
    env:
      REQUIRE_APPROVAL: "true"

  nodes:
    - id: process
      type: Agent
      config:
        agent: k8s-helper
        input: ${event.text}

    - id: respond
      type: Slack
      config:
        channel: ${event.channel}
        thread_ts: ${event.ts}
        message: ${process.output}

  connections:
    - from: trigger
      to: process
    - from: process
      to: respond
```

Save as `slack-prod-flow.yaml` and start the daemon:

```bash
# Set environment variables
export SLACK_BOT_TOKEN="xoxb-your-token"
export SLACK_SIGNING_SECRET="your-secret"
export KUBECONFIG_PROD="~/.kube/prod-config"

# Start daemon with flows
aofctl serve --flows-dir ./flows --agents-dir ./agents --port 3000

# Expose via tunnel (for Slack webhooks)
cloudflared tunnel --url http://localhost:3000
```

Now messages in `#production` channel go to your prod-cluster agent! See [AgentFlow Spec](reference/agentflow-spec.md) for multi-tenant routing options.

## Next Steps

You now have a working AI agent! Here's where to go next:

### Learn Core Concepts
- **[Core Concepts](concepts.md)** - Understand Agents, Fleets, and Flows

### Follow Tutorials
- **[Build Your First Agent](tutorials/first-agent.md)** - Deeper dive into Agent specs
- **[Create a Slack Bot](tutorials/slack-bot.md)** - Build a production bot with AgentFlow
- **[Incident Response Flow](tutorials/incident-response.md)** - Auto-remediation workflow

### Explore Examples
- **[Copy-paste Examples](examples/)** - Ready-to-use agent configurations

### Read Reference Docs
- **[Agent Spec](reference/agent-spec.md)** - Complete YAML reference
- **[AgentFlow Spec](reference/agentflow-spec.md)** - Event-driven workflow reference
- **[aofctl CLI](reference/aofctl.md)** - All CLI commands

## Common Issues

### "API key not found"
```bash
# Make sure you've exported your key
echo $OPENAI_API_KEY

# If empty, set it:
export OPENAI_API_KEY=sk-...
```

### "Command not found: kubectl"
The agent can't use tools you don't have installed. Either:
1. Install the tool: `brew install kubectl`
2. Remove it from `allowed_commands`

### "Model not supported"
Check your provider:model format:
- ✅ `google:gemini-2.5-flash`
- ✅ `openai:gpt-4o`
- ✅ `anthropic:claude-3-5-sonnet-20241022`
- ✅ `ollama:llama3`
- ⚠️ `gpt-4` (defaults to Anthropic - better to specify provider)

## Getting Help

- **Documentation**: Full docs at [https://docs.aof.sh](https://docs.aof.sh)
- **Examples**: Check [docs/examples/](examples/) for copy-paste configs
- **Issues**: Report bugs at [GitHub Issues](https://github.com/agenticdevops/aof/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/agenticdevops/aof/discussions)

---

**Ready to build something real?** → [Build Your First Agent Tutorial](tutorials/first-agent.md)
