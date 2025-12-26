# Getting Started with AOF

Get up and running with your first AI agent in 15 minutes. No complex setup required.

## Prerequisites

You'll need:
- **Google AI Studio API Key** (free): [Get one here](https://aistudio.google.com/apikey)
- **Docker** running locally (for the examples below)
- **Git** installed

That's it. No Kubernetes cluster, no Slack workspace, no cloud accounts.

## Installation

### Step 1: Install aofctl

```bash
# Auto-detect your platform and install
curl -sSL https://docs.aof.sh/install.sh | bash

# Verify installation
aofctl version
```

Alternative methods:
```bash
# Via Cargo
cargo install aofctl

# From source
git clone https://github.com/agenticdevops/aof.git
cd aof && cargo build --release
sudo cp target/release/aofctl /usr/local/bin/
```

### Step 2: Set Your API Key

```bash
# Get a free API key from https://aistudio.google.com/apikey
export GOOGLE_API_KEY=AIza...

# Add to your shell profile to persist:
echo 'export GOOGLE_API_KEY=AIza...' >> ~/.zshrc
```

## Your First Agent (2 minutes)

Let's create an agent that checks your Docker containers.

### Create the Agent

Create `docker-health.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: docker-health

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are a Docker health checker. Help users understand what's
    running in their Docker environment.

    Available commands: ps, stats, logs, images, inspect, exec
    Keep responses concise and actionable.

  tools:
    - docker
```

### Run It

```bash
# Check container status
aofctl run agent docker-health.yaml --input "what containers are running?"

# Get more details
aofctl run agent docker-health.yaml --input "show me stats for all containers"

# Investigate issues
aofctl run agent docker-health.yaml --input "check logs for any unhealthy containers"
```

**That's it!** You have a working AI agent.

## Your First Fleet (5 minutes)

A **Fleet** runs multiple agents in parallel. Let's create a code review team.

### Create the Fleet

Create `code-review-fleet.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-fleet

spec:
  agents:
    # Security specialist
    - name: security-reviewer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Review code for security issues:
          - SQL injection, XSS, command injection
          - Hardcoded secrets
          - Authentication vulnerabilities

          Format: List issues by severity (Critical/High/Medium)

    # Quality specialist
    - name: quality-reviewer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Review code for quality issues:
          - Readability and naming
          - Error handling
          - Code structure

          Give a score from 1-10 with justification.

  coordination:
    mode: peer
    distribution: round-robin
    aggregation: merge  # Collect ALL agent results
```

### Run It

```bash
# Review a file
aofctl run fleet code-review-fleet.yaml --input "Review: $(cat main.py)"

# Or paste code directly
aofctl run fleet code-review-fleet.yaml --input "Review this:
function login(user, pass) {
  const query = 'SELECT * FROM users WHERE name=' + user;
  return db.query(query);
}"
```

Both agents analyze the code in parallel and return their findings.

## Your First Flow (5 minutes)

A **Flow** chains steps together in a workflow. Flows can use:
- **Agent nodes** - For intelligent analysis (uses LLM)
- **Script nodes** - For deterministic operations like running commands (no LLM)

Let's create a Docker troubleshooting pipeline that uses Script nodes to collect data efficiently, then an Agent for intelligent analysis.

### Create the Flow

Create `docker-troubleshoot.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: docker-troubleshoot

spec:
  description: "Diagnose Docker issues with Script + Agent nodes"

  nodes:
    # Step 1: Get container status (Script - no LLM, fast)
    - id: check-status
      type: Script
      config:
        scriptConfig:
          tool: docker
          action: ps
          args:
            all: true

    # Step 2: Get container stats (Script - no LLM)
    - id: get-stats
      type: Script
      config:
        scriptConfig:
          tool: docker
          action: stats

    # Step 3: Get logs from exited containers (Script - shell command)
    - id: get-logs
      type: Script
      config:
        scriptConfig:
          command: |
            docker ps -a --filter "status=exited" --format "{{.Names}}" | head -3 | while read name; do
              echo "=== Logs for $name ==="
              docker logs --tail 30 "$name" 2>&1
            done
          parse: text
          timeout_seconds: 30

    # Step 4: Analyze everything with AI (Agent - uses LLM)
    - id: analyze
      type: Agent
      config:
        inline:
          name: docker-analyst
          model: google:gemini-2.5-flash
          instructions: |
            Analyze the Docker diagnostics and provide:
            1. **Status Summary**: Overview of container health
            2. **Issues Found**: Any problems detected
            3. **Root Causes**: Likely reasons for failures
            4. **Fix Commands**: Specific docker commands to resolve issues

            Be concise and actionable.
          temperature: 0.1
        input: |
          Container Status:
          ${check-status.output}

          Resource Stats:
          ${get-stats.output}

          Logs from Exited Containers:
          ${get-logs.output}

  connections:
    - from: start
      to: check-status
    - from: check-status
      to: get-stats
    - from: get-stats
      to: get-logs
    - from: get-logs
      to: analyze
```

### Run It

```bash
# Diagnose your Docker environment
aofctl run flow docker-troubleshoot.yaml --input "diagnose"
```

**Why this is efficient:** Script nodes collect data without using LLM tokens. Only the final analysis step uses the AI, making the flow faster and cheaper.

### Script Nodes: No LLM, Just Commands

Script nodes run shell commands or native tools directly:

```yaml
# Shell command
- id: get-logs
  type: Script
  config:
    scriptConfig:
      command: docker logs --tail 50 myapp
      parse: lines   # Split output into array

# Native tool (built-in support)
- id: check-pods
  type: Script
  config:
    scriptConfig:
      tool: kubectl
      action: get
      args:
        resource: pods
        namespace: default
```

**Available native tools:** `docker`, `kubectl`, `http`, `json`, `file`

The flow executes each step in sequence, passing context between nodes.

## Quick Reference

| Concept | What It Does | When to Use |
|---------|--------------|-------------|
| **Agent** | Single AI specialist | Simple tasks, intelligent analysis |
| **Fleet** | Multiple agents in parallel | Reviews, analysis, multi-perspective tasks |
| **Flow** | Sequential pipeline | Multi-step workflows, troubleshooting |
| **Script Node** | Run commands (no LLM) | Data collection, CLI operations, file ops |

## Add Tools

### Agent Tools (LLM-driven)

Agents can use these built-in tools via the LLM:

```yaml
spec:
  tools:
    - docker      # Docker commands
    - kubectl     # Kubernetes commands
    - git         # Git operations
    - shell       # Shell commands (use carefully)
    - aws         # AWS CLI
    - terraform   # Terraform commands
    - http        # HTTP requests
```

### Script Node Tools (No LLM)

Script nodes have native tools that run without LLM involvement:

```yaml
# In a Flow, use Script nodes for deterministic operations
- id: check-pods
  type: Script
  config:
    scriptConfig:
      tool: docker    # docker, kubectl, http, json, file
      action: ps
      args:
        all: true
```

**Tip:** Use Script nodes for data collection, then pass results to Agent nodes for analysis. This saves tokens and speeds up flows.

## Use Other Models

```yaml
spec:
  # Google Gemini (recommended - fast and cheap)
  model: google:gemini-2.5-flash

  # OpenAI
  model: openai:gpt-4o
  # Requires: export OPENAI_API_KEY=sk-...

  # Anthropic Claude
  model: anthropic:claude-3-5-sonnet-20241022
  # Requires: export ANTHROPIC_API_KEY=sk-ant-...

  # Local with Ollama (no API key needed)
  model: ollama:llama3
  # Requires: ollama serve
```

## Next Steps

You've learned the three core concepts. Now:

### Learn More
- **[Core Concepts](concepts.md)** - Deeper dive into Agents, Fleets, Flows
- **[Architecture](architecture/composable-design.md)** - How everything fits together

### Build Real Things
- **[Build a Slack Bot](tutorials/slack-bot.md)** - Add chat interface to your agents
- **[Incident Response](tutorials/incident-response.md)** - Auto-remediation workflow
- **[PR Review Automation](tutorials/pr-review-automation.md)** - GitHub integration

### Reference
- **[Agent Spec](reference/agent-spec.md)** - Complete YAML reference
- **[Fleet Spec](reference/fleet-spec.md)** - Multi-agent configuration
- **[AgentFlow Spec](reference/agentflow-spec.md)** - Flows, Script nodes, and more
- **[aofctl CLI](reference/aofctl.md)** - All CLI commands

## Troubleshooting

### "API key not found"
```bash
# Check if key is set
echo $GOOGLE_API_KEY

# If empty, set it
export GOOGLE_API_KEY=AIza...
```

### "Tool not found: docker"
The agent can't use tools you don't have installed:
```bash
# Check if Docker is running
docker --version

# If not installed
brew install docker
```

### "Model not supported"
Check your provider:model format:
- ✅ `google:gemini-2.5-flash`
- ✅ `openai:gpt-4o`
- ✅ `anthropic:claude-3-5-sonnet-20241022`
- ❌ `gpt-4` (missing provider prefix)

## Get Help

- **Documentation**: [https://docs.aof.sh](https://docs.aof.sh)
- **Examples**: [GitHub examples/](https://github.com/agenticdevops/aof/tree/main/examples)
- **Issues**: [GitHub Issues](https://github.com/agenticdevops/aof/issues)
- **Discussions**: [GitHub Discussions](https://github.com/agenticdevops/aof/discussions)
