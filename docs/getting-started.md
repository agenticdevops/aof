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

A **Flow** chains agents together in a workflow. Let's create a Docker troubleshooting pipeline.

### Create the Flow

Create `docker-troubleshoot.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: docker-troubleshoot

spec:
  description: "Diagnose and fix Docker container issues"

  nodes:
    # Step 1: Check status
    - id: check-status
      type: Agent
      config:
        inline:
          name: status-checker
          model: google:gemini-2.5-flash
          instructions: |
            Run docker ps -a to check container status.
            Report: running count, stopped count, any unhealthy containers.
          tools:
            - docker

    # Step 2: Analyze logs
    - id: analyze-logs
      type: Agent
      config:
        inline:
          name: log-analyzer
          model: google:gemini-2.5-flash
          instructions: |
            For any unhealthy or exited containers, check their logs.
            Look for error patterns and identify root causes.
          tools:
            - docker

    # Step 3: Recommend fixes
    - id: recommend-fixes
      type: Agent
      config:
        inline:
          name: fixer
          model: google:gemini-2.5-flash
          instructions: |
            Based on the analysis, provide specific commands to fix issues.
            Format: Issue → Cause → Fix command → Notes
          tools:
            - docker

  connections:
    - from: start
      to: check-status
    - from: check-status
      to: analyze-logs
    - from: analyze-logs
      to: recommend-fixes
```

### Run It

```bash
# Diagnose your Docker environment
aofctl run flow docker-troubleshoot.yaml --input "diagnose"

# Focus on a specific container
aofctl run flow docker-troubleshoot.yaml --input "diagnose container nginx"
```

The flow executes each step in sequence, passing context between agents.

## Quick Reference

| Concept | What It Does | When to Use |
|---------|--------------|-------------|
| **Agent** | Single AI specialist | Simple tasks, one skill |
| **Fleet** | Multiple agents in parallel | Reviews, analysis, multi-perspective tasks |
| **Flow** | Sequential pipeline | Multi-step workflows, troubleshooting |

## Add Tools

Agents can use these built-in tools:

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
