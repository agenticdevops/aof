# Getting Started with AOF

Get up and running with your first AI agent in 5 minutes.

## Prerequisites

### Required
- **API Key**: Get one from [OpenAI](https://platform.openai.com/api-keys), [Anthropic](https://console.anthropic.com/), or use [Ollama](https://ollama.ai/) locally
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
curl -sSL https://aof.dev/install.sh | bash

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
git clone https://github.com/yourusername/aof.git
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

# OR Ollama (runs locally, no key needed)
# Just install: brew install ollama && ollama serve
```

**💡 Tip**: Add these to your `~/.zshrc` or `~/.bashrc` to persist across sessions.

## Create Your First Agent

### Step 3: Create an Agent YAML

Create a file called `hello-agent.yaml`:

```yaml
name: hello-assistant
model: claude-3-5-sonnet-20241022
provider: anthropic

instructions: |
  You are a friendly assistant that helps DevOps engineers.
  Keep responses concise and practical.

max_iterations: 10
temperature: 0.7
```

### Step 4: Run Your Agent

```bash
# Interactive chat mode
aofctl agent run hello-agent.yaml

# You'll see:
> Agent 'hello-assistant' is ready. Type your message (or 'exit' to quit):
```

Try asking:
```
> What's the difference between a Deployment and a StatefulSet?
```

### Step 5: Verify It Works

Your agent should respond with a clear explanation. If you see a response, congratulations! 🎉

## Add Some Tools

Let's make the agent more useful by adding kubectl and shell access:

```yaml
name: k8s-helper
model: google:gemini-2.5-flash
provider: google

instructions: |
  You are a Kubernetes expert assistant. Help users run kubectl commands
  and troubleshoot their clusters. Always explain what commands do before running them.

tools:
  - shell
  - kubectl

max_iterations: 5
temperature: 0.7
```

Save this as `k8s-agent.yaml` and run:

```bash
aofctl agent run k8s-agent.yaml
```

Now try:
```
> Show me all pods in the default namespace
```

The agent will explain what it's doing and run `kubectl get pods -n default`.

## Next Steps

You now have a working AI agent! Here's where to go next:

### Learn Core Concepts
- **[Core Concepts](concepts.md)** - Understand Agents, Fleets, and Flows

### Follow Tutorials
- **[Build Your First Agent](tutorials/first-agent.md)** - Deeper dive into Agent specs
- **[Create a Slack Bot](tutorials/slack-bot.md)** - Build a production bot
- **[Incident Response Flow](tutorials/incident-response.md)** - Auto-remediation workflow

### Explore Examples
- **[Copy-paste Examples](examples/)** - Ready-to-use agent configurations

### Read Reference Docs
- **[Agent Spec](reference/agent-spec.md)** - Complete YAML reference
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
Check your model format:
- ✅ `claude-3-5-sonnet-20241022` (with `provider: anthropic`)
- ✅ `gpt-4-turbo` (with `provider: openai`)
- ✅ `google:gemini-2.5-flash` (combined format)
- ✅ `llama3` (with `provider: ollama`)
- ❌ `gpt-4` (missing provider or incomplete model name)

## Getting Help

- **Documentation**: Full docs at [https://aof.dev/docs](https://aof.dev/docs)
- **Examples**: Check [docs/examples/](examples/) for copy-paste configs
- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/aof/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/yourusername/aof/discussions)

---

**Ready to build something real?** → [Build Your First Agent Tutorial](tutorials/first-agent.md)
