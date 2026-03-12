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

Skills inject domain expertise into your agent's system prompt without any LLM routing call.

### Browse built-in skill packs

```bash
agentix skills list
```

Output:
```
NAME            DESCRIPTION
────────────────────────────────────────────────────────────────────────
aws             AWS cloud infrastructure management (EC2, S3, IAM, CloudWatch)
kubernetes      Kubernetes cluster management and workload operations (kubectl)
terraform       Terraform infrastructure-as-code management (plan, apply, state)
docker          Docker container and image management (build, run, inspect)
git             Git version control operations (commit, branch, rebase, merge)
database        Database operations and query optimization (PostgreSQL, MySQL)
security        Security scanning, vulnerability assessment, and hardening
observability   Observability, monitoring, log analysis, and distributed tracing
```

### Activate a built-in skill pack

To add Kubernetes expertise to your agent:

```bash
mkdir -p agents/my-agent/skills/kubernetes
```

The gateway injects the Kubernetes skill pack's instructions into the agent's system prompt on next startup. No code changes needed.

### Add custom skills

For team-specific knowledge, create your own `SKILL.md`:

```bash
mkdir -p agents/my-agent/skills/my-postgres
cat > agents/my-agent/skills/my-postgres/SKILL.md << 'EOF'
# PostgreSQL Tuning

Production cluster details:
- Primary: db-primary.internal:5432
- Read replicas on port 5433
- Connection limit: 100 per service
- Always use prepared statements for queries
EOF
```

Your agent directory with skills:

```
agents/my-agent/
├── agent.yaml
├── SOUL.md
└── skills/
    ├── kubernetes/          # Directory activates built-in kubernetes pack
    └── my-postgres/
        └── SKILL.md         # Custom skill — injected verbatim
```

See: [Skills Guide](skills.md)

---

## 12. Add Tools (Optional)

Tools let agents execute CLI commands and MCP servers during the ReAct loop.

### CLI / shell tools

Create a tools directory with YAML definitions:

```bash
mkdir -p agents/my-agent/tools
cat > agents/my-agent/tools/kubectl.yaml << 'EOF'
name: kubectl
type: cli
command: kubectl
description: Kubernetes cluster management
args:
  - "{{subcommand}}"
EOF
```

During a run, the agent can call the tool:

```
[Act] Calling kubectl with {"subcommand": "get pods -n default"}
[Observe] NAME                     READY   STATUS    RESTARTS   AGE
          nginx-6d4b5c46-xyz12     1/1     Running   0          2d
```

### MCP server tools

Connect your agent to any MCP server:

```yaml
# agents/my-agent/agent.yaml
spec_version: v1
name: my-agent
mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
```

See: [Tools Guide](tools.md) | [MCP Guide](mcp.md)

---

## Next Steps

- [Agent Directory Format](../spec/agent-directory-structure.md) — full directory spec
- [Agent Manifest (agent.yaml)](../spec/agent-yaml-v1.md) — all manifest fields
- [Workspace Configuration](../spec/workspace-config.md) — workspace config reference
- [Gateway API Reference](../reference/gateway-api.md) — REST API docs
