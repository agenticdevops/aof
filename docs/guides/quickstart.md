# Quickstart Guide

Get from zero to running your first OpenAgentiX agent in under 5 minutes.

## 1. Installation

### Option A: Cargo (from source)
```bash
cargo install agentix
```

### Option B: Binary download
```bash
# macOS / Linux
curl -sSL https://docs.aof.sh/install.sh | bash

# Or download from GitHub Releases
# https://github.com/agenticdevops/aof/releases
```

Verify the installation:
```bash
agentix version
# agentix 2.0.0-alpha
# OpenAgentiX — Enterprise Agent Automation Platform
```

---

## 2. Set Up a Workspace

Run the interactive onboarding wizard to create your workspace:

```bash
agentix onboard
```

The wizard will:
1. Ask for your workspace name
2. Ask which LLM provider to use (Anthropic, OpenAI, Google, Ollama)
3. Ask for your API key
4. Create `agentix.yaml` and a sample `hello-world` agent

When it finishes, you'll have this structure:

```
my-workspace/
├── agentix.yaml              # Workspace config (providers, defaults)
└── agents/
    └── hello-world/
        ├── agent.yaml        # Minimal manifest
        └── SOUL.md           # Agent identity and system prompt
```

---

## 3. Understand the Structure

### agentix.yaml (workspace config)

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-workspace
spec:
  agents_dir: ./agents
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 10
    timeout_secs: 120
```

### agents/hello-world/agent.yaml (agent manifest)

```yaml
spec_version: "1"
name: hello-world
description: A friendly greeter agent
model:
  preferred: anthropic/claude-sonnet-4-6
```

### agents/hello-world/SOUL.md (agent identity)

```markdown
# Hello World Agent

You are a helpful assistant named Hello World.
You answer questions clearly and concisely.
You are friendly and professional.
```

---

## 4. Edit Your Agent

Open `agents/hello-world/SOUL.md` and define your agent's identity:

```markdown
# My Custom Agent

You are an expert DevOps assistant specializing in Kubernetes.

When asked about cluster issues:
1. Ask for the relevant pod names or namespaces
2. Suggest kubectl commands to diagnose the issue
3. Explain what the output means
4. Recommend a fix

Always be concise and actionable.
```

---

## 5. Start the Gateway

```bash
agentix gateway start
```

The gateway starts on port 7777 by default. You'll see:

```
[INFO] OpenAgentiX Gateway v2.0.0-alpha
[INFO] Loaded agent 'hello-world' from ./agents/hello-world
[INFO] Gateway listening on http://0.0.0.0:7777
```

---

## 6. List Agents

In a new terminal:

```bash
agentix agents
```

Output:
```
NAME          MODEL                          STATUS    LOADED
hello-world   anthropic/claude-sonnet-4-6   ready     just now
```

---

## 7. Run an Agent

### Via CLI

```bash
agentix run hello-world "What is Kubernetes?"
```

### Via API (with streaming)

```bash
curl -X POST http://localhost:7777/api/v1/agents/hello-world/run \
  -H 'Content-Type: application/json' \
  -d '{"input": "What is Kubernetes?"}' \
  --no-buffer
```

Example SSE response stream:

```
data: {"type":"step","step":{"plan":"The user is asking about Kubernetes...","action":null,"observation":"","reflection":""}}

data: {"type":"complete","result":{"output":"Kubernetes is an open-source container orchestration platform...","iterations":1,"tool_calls":[],"reached_max_iterations":false}}
```

---

## 8. View Run Logs

```bash
# List recent runs
agentix runs hello-world

# View logs for a specific run
agentix logs <run-id>
```

---

## 9. Create Another Agent

```bash
agentix init --name my-agent
```

This creates `agents/my-agent/` with `agent.yaml` and `SOUL.md`.

---

## 10. Validate Before Running

```bash
agentix validate agents/my-agent
```

Output on success:
```
Agent directory 'agents/my-agent' is valid
  Name: my-agent
```

---

## 11. Add Skills (Optional)

Skills are reusable system prompt modules. Create a skills directory:

```
agents/my-agent/
├── agent.yaml
├── SOUL.md
└── skills/
    └── kubernetes.md    # Kubernetes expertise module
```

Example `skills/kubernetes.md`:

```markdown
## Kubernetes Expertise

You have deep knowledge of Kubernetes:
- Pod lifecycle and status conditions
- Common kubectl commands and their output
- Troubleshooting CrashLoopBackOff, OOMKilled, Pending states
- Helm chart management
- RBAC and security contexts
```

The DirectoryLoader automatically includes all `.md` files in `skills/` in the agent's system prompt.

---

## 12. Add Tools (Optional)

Tools let agents execute shell commands. Create a tools directory:

```
agents/my-agent/
├── agent.yaml
├── SOUL.md
└── tools/
    └── kubectl-get-pods.yaml
```

Example `tools/kubectl-get-pods.yaml`:

```yaml
name: get_pods
description: List pods in a Kubernetes namespace
type: shell
command: kubectl
args:
  - get
  - pods
  - -n
  - "{{namespace}}"
  - -o
  - wide
parameters:
  namespace:
    type: string
    description: Kubernetes namespace
    required: true
```

---

## Next Steps

- [Agent Directory Format](../spec/agent-directory-structure.md) — full directory spec
- [Agent Manifest (agent.yaml)](../spec/agent-yaml-v1.md) — all manifest fields
- [Workspace Configuration](../spec/workspace-config.md) — workspace config reference
- [Gateway API Reference](../reference/gateway-api.md) — REST API docs
