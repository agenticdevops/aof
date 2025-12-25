# AOF Examples

Copy-paste ready YAML configurations for common use cases.

> **Note:** AOF supports running agents with `aofctl run agent` and fleets with `aofctl run fleet`. AgentFlow support is planned for future releases.

## Quick Start Examples

### 1. Hello World Agent
**File:** `hello-agent.yaml`
**Use Case:** Getting started with AOF

**Quick Start:**
```bash
# Set your API key
export GOOGLE_API_KEY=AIza...   # For Gemini
# OR
export OPENAI_API_KEY=sk-...     # For OpenAI

# Run the agent
aofctl run agent examples/hello-agent.yaml --input "Hello, what can you do?"
```

---

### 2. Gemini Agent with Tools
**File:** `test-gemini-with-tools.yaml`
**Use Case:** Agent with file system and shell access

**Features:**
- Shell command execution
- File reading
- Directory listing
- Google Gemini model

**Quick Start:**
```bash
# Set Gemini API key
export GOOGLE_API_KEY=AIza...

# Run agent with tools
aofctl run agent examples/test-gemini-with-tools.yaml --input "List the files in the current directory"

# Check git status
aofctl run agent examples/test-unified-tools.yaml --input "What is the current git status?"
```

---

### 3. Kubernetes Operations Agent
**File:** `kubectl-agent.yaml`
**Use Case:** K8s cluster management using kubectl-ai MCP server

**Features:**
- kubectl commands via MCP
- Pod/deployment diagnostics
- Service health checks

**Quick Start:**
```bash
# Ensure kubectl-ai is installed
# Install: brew install kubectl-ai

# Set your API key
export OPENAI_API_KEY=sk-...

# Run the agent
aofctl run agent examples/kubectl-agent.yaml --input "Show me all pods"
```

---

### 4. Code Review Fleet
**File:** `examples/quickstart/code-review-fleet.yaml`
**Use Case:** Multi-agent code review with security and quality reviewers

**Features:**
- 2 specialized reviewers running in parallel
- Peer coordination with result aggregation (not consensus)
- Collects and merges ALL review findings

**Quick Start:**
```bash
# Set Gemini API key
export GOOGLE_API_KEY=AIza...

# Run fleet with code to review
aofctl run fleet examples/quickstart/code-review-fleet.yaml --input "Review: function login(user, pass) { const query = 'SELECT * FROM users WHERE name=' + user; return db.query(query); }"
```

---

### 5. MCP Tools Agent
**File:** `mcp-tools-agent.yaml`
**Use Case:** Agent with MCP filesystem server

**Features:**
- Filesystem MCP server for file operations
- Read, write, list, search files
- Directory tree exploration

**Quick Start:**
```bash
# Set Gemini API key
export GOOGLE_API_KEY=AIza...

# Run with filesystem MCP
aofctl run agent examples/mcp-tools-agent.yaml --input "List files in the examples directory"
```

> **Note:** For advanced MCP configurations (multiple servers, SSE/HTTP transports), see `examples/mcp-advanced-example.yaml`

---

## Built-in Tools

AOF provides several built-in tools that can be used in agent configurations:

| Tool | Description |
|------|-------------|
| `shell` | Execute shell commands |
| `read_file` | Read file contents |
| `list_directory` | List directory contents |
| `git` | Execute git commands |

**Example using built-in tools:**
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: dev-helper
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a helpful DevOps assistant.
  tools:
    - shell
    - read_file
    - list_directory
    - git
```

---

## AgentFleet Examples

AgentFleet enables multi-agent coordination for complex tasks. All fleet examples are tested and working.

### Available Fleet Examples

| Fleet | Description | Location |
|-------|-------------|----------|
| **code-review-fleet** | Multi-specialist code review with aggregation | `examples/quickstart/code-review-fleet.yaml` |
| **code-review-team** | Multi-perspective code review (legacy) | `examples/fleets/code-review-team.yaml` |
| **incident-response-team** | Incident triage and response coordination | `examples/fleets/incident-response-team.yaml` |
| **k8s-rca-team** | Kubernetes root cause analysis | `examples/fleets/k8s-rca-team.yaml` |
| **dockerizer-team** | Containerize applications | `examples/fleets/dockerizer-team.yaml` |
| **simple-test-fleet** | Basic fleet for testing | `examples/fleets/simple-test-fleet.yaml` |

## Planned Features (AgentFlow)

> **Coming Soon:** AgentFlow will enable workflow orchestration:

- **Incident Response System** - Auto-remediation with PagerDuty integration
- **Slack Bot Flow** - Conversational assistant with approval workflows
- **Daily Reports** - Scheduled operational reports

---

## Example Comparison

### Agents

| Example | Complexity | Status | Prerequisites |
|---------|------------|--------|---------------|
| **hello-agent** | ⭐ Simple | ✅ Working | API key |
| **test-gemini-with-tools** | ⭐ Simple | ✅ Working | Gemini API key |
| **test-tools-agent** | ⭐ Simple | ✅ Working | Gemini API key |
| **kubectl-agent** | ⭐⭐ Medium | ✅ Working | kubectl-ai, API key |
| **mcp-tools-agent** | ⭐⭐ Medium | ✅ Working | Node.js, API key |

### Fleets

| Example | Complexity | Status | Prerequisites |
|---------|------------|--------|---------------|
| **simple-test-fleet** | ⭐ Simple | ✅ Working | Gemini API key |
| **code-review-fleet** | ⭐ Simple | ✅ Working | Gemini API key |
| **code-review-team** | ⭐⭐ Medium | ✅ Working | Gemini API key |
| **incident-response-team** | ⭐⭐ Medium | ✅ Working | Gemini API key |
| **k8s-rca-team** | ⭐⭐⭐ Advanced | ✅ Working | Gemini API key, kubectl |
| **dockerizer-team** | ⭐⭐ Medium | ✅ Working | Gemini API key |

---

## Customization Tips

### Change the Model

```yaml
spec:
  model: google:gemini-2.5-flash   # Default - fast and capable

  # Alternatives:
  model: openai:gpt-4o                        # OpenAI GPT-4o
  model: openai:gpt-4o-mini                   # Cheaper/faster
  model: anthropic:claude-3-5-sonnet-20241022 # Claude Sonnet
  model: ollama:llama3                        # Local (free)
```

### Add Built-in Tools

```yaml
spec:
  tools:
    - shell          # Execute shell commands
    - read_file      # Read file contents
    - list_directory # List directories
    - git            # Git operations
```

### Add MCP Servers

```yaml
spec:
  mcp_servers:
    # Filesystem MCP server
    - name: filesystem
      transport: stdio
      command: npx
      args:
        - "-y"
        - "@modelcontextprotocol/server-filesystem"
        - "/workspace"
      timeout_secs: 30

    # kubectl-ai MCP server
    - name: kubectl-ai
      transport: stdio
      command: kubectl-ai
      args:
        - "--mcp-server"
        - "--mcp-server-mode=stdio"
```

---

## Environment Variables

Common variables used across examples:

```bash
# LLM Providers
export GOOGLE_API_KEY=AIza...         # Google Gemini
export OPENAI_API_KEY=sk-...          # OpenAI
export ANTHROPIC_API_KEY=sk-ant-...   # Anthropic

# Kubernetes
export KUBECONFIG=~/.kube/config

# GitHub (for GitHub MCP server)
export GITHUB_TOKEN=ghp_...
```

Add to your `~/.zshrc` or `~/.bashrc` to persist across sessions.

---

## Testing Examples

### Run an Agent
```bash
# Run agent with input
aofctl run agent examples/hello-agent.yaml --input "test query"

# Run with JSON output
aofctl run agent examples/hello-agent.yaml --input "test" --output json
```

### List Available Agents
```bash
# List agents (from configured directory)
aofctl get agent
```

---

## Getting Help

- **Tutorials**: See [First Agent Tutorial](../tutorials/first-agent.md)
- **Reference**: See [aofctl CLI Reference](../reference/aofctl.md)
- **Issues**: [GitHub Issues](https://github.com/agenticdevops/aof/issues)

---

## Contributing Examples

Have a useful agent configuration? Submit it!

1. Create your YAML file in `examples/`
2. Test it: `aofctl run agent your-agent.yaml --input "test"`
3. Add documentation comments in the YAML
4. Submit a PR with description and usage examples

---

**Ready to build?** Start with `hello-agent.yaml` and add tools from there!
