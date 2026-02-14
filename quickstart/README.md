# AOF Local Testing Setup (quickstart/)

This directory contains working test configurations for AOF using **Google Gemini 2.5 Flash** with **11 specialized agent minions**.

## Quick Facts

- ✅ **11 agents ready to use** - All validated with Google Gemini 2.5 Flash
- ✅ **Server configuration working** - Agent discovery auto-enabled
- ✅ **Full stack tested** - Daemon + Web UI + Agents all functional
- ✅ **Documentation complete** - Quick start, detailed guides, and minion reference

## Prerequisites

1. **Google API Key** - Get from https://aistudio.google.com/apikey
2. **Rust** - Build the project
3. **Cargo** - For running commands

## Quick Start (30 seconds)

```bash
# 1. Set API key
export GOOGLE_API_KEY="your-key-from-aistudio.google.com"

# 2. Run your first minion
cd /Users/gshah/work/opsflow-sh/aof
aofctl run agent quickstart/agents/kubo.yaml --prompt "Tell me about yourself"
```

## Available Minions

| Bot | Name | What It Does |
|-----|------|------------|
| 🐴 | **kubo** | Kubernetes expert - cluster health, troubleshooting, architecture |
| 🐳 | **doku** | Docker specialist - Dockerfile optimization, container best practices |
| 🏗️ | **rafo** | Terraform wizard - IaC design, module structure, state management |
| ⚙️ | **ergo** | Argo orchestrator - CI/CD pipelines, GitOps workflows, deployments |
| ☁️ | **wos** | AWS champion - cloud architecture, serverless, cost optimization |
| 🔵 | **zure** | Azure specialist - enterprise cloud, hybrid, DevOps |
| 🐧 | **nux** | Linux admin - system troubleshooting, shell scripting, optimization |
| 📋 | **zibl** | Ansible master - playbooks, configuration management, automation |
| ✅ | **quick-test** | General-purpose test agent |
| 🔍 | **k8s-checker** | Kubernetes health diagnostics |
| 📊 | **system-monitor** | System resource monitoring |

## Testing Scenarios

### Scenario 1: Quick Single Agent Test (2 minutes)

```bash
export GOOGLE_API_KEY="your-api-key-here"
cd /Users/gshah/work/opsflow-sh/aof

# Try any minion
aofctl run agent quickstart/agents/kubo.yaml --prompt "What can you help me with?"
aofctl run agent quickstart/agents/doku.yaml --prompt "How do I optimize Docker images?"
aofctl run agent quickstart/agents/rafo.yaml --prompt "Design a Terraform setup"
```

### Scenario 2: Full Stack Integration (15 minutes)

```bash
# Terminal 1: Start server daemon
export GOOGLE_API_KEY="your-api-key-here"
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml

# Terminal 2: Start web UI
cd web-ui
npm run dev
# Visit http://localhost:5173

# Terminal 3: Run agents
export GOOGLE_API_KEY="your-api-key-here"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Check cluster health"

# You'll see:
# - Agent execution in Terminal 1 logs
# - Real-time events in Web UI
# - Response in Terminal 3
```

### Scenario 3: Interactive Conversation

Have a multi-turn chat with any minion:

```bash
export GOOGLE_API_KEY="your-api-key-here"

# Interactive mode with any agent
aofctl run agent quickstart/agents/kubo.yaml --prompt "Your question"
# Now ask follow-up questions, have real conversation
# Press Ctrl+C to exit
```

## Health Checks

Once server is running (from Scenario 2, Terminal 1):

```bash
# Liveness check
curl http://localhost:7777/health

# Readiness check  
curl http://localhost:7777/ready

# Metrics endpoint
curl http://localhost:7777/metrics
```

## Configuration

### serve-config.yaml

Main server configuration. Key features:
- Auto-discovers agents from `quickstart/agents/` directory
- Uses Google Gemini 2.5 Flash as default LLM
- Exposes WebSocket at `ws://localhost:7777/ws`
- Health checks every 30 seconds
- Graceful shutdown (30 second timeout)

### Agent Files

Each agent YAML in `quickstart/agents/` is fully independent and can be run directly:

```bash
# Run any agent directly
aofctl run agent quickstart/agents/kubo.yaml --prompt "Your question"

# Or use with custom model
aofctl run agent quickstart/agents/kubo.yaml \
  --model "anthropic:claude-3-5-sonnet" \
  --prompt "Your question"
```

## Switching to Claude (Anthropic)

If you have an Anthropic API key, you can use Claude instead:

```bash
export ANTHROPIC_API_KEY="sk-..."

# Use with specific agent
aofctl run agent quickstart/agents/kubo.yaml \
  --model "anthropic:claude-3-5-sonnet" \
  --prompt "Your question"

# Or modify serve-config.yaml:
# Change: model: "gemini-2.5-flash"
# To: model: "claude-3-5-sonnet"
```

## File Structure

```
quickstart/
├── SETUP.md                  # Quick start guide
├── MINIONS.md                # Quick reference card
├── AGENTS.md                 # Detailed specifications
├── README.md                 # This file
├── serve-config.yaml         # Server configuration
├── test-quick.sh             # 5-minute test script
├── test-interactive.sh       # Interactive test script
└── agents/                   # Your minion squad
    ├── kubo.yaml            # ✓ Kubernetes expert
    ├── doku.yaml            # ✓ Docker specialist
    ├── rafo.yaml            # ✓ Terraform wizard
    ├── ergo.yaml            # ✓ Argo orchestrator
    ├── wos.yaml             # ✓ AWS champion
    ├── zure.yaml            # ✓ Azure specialist
    ├── nux.yaml             # ✓ Linux admin
    ├── zibl.yaml            # ✓ Ansible master
    ├── quick-test.yaml      # ✓ General test
    ├── k8s-checker.yaml     # ✓ K8s diagnostics
    └── system-monitor.yaml  # ✓ System monitor
```

## Troubleshooting

### "No API key configured"
```bash
export GOOGLE_API_KEY="your-api-key-here"
```

### "Address already in use" (port 7777)
```bash
# Something is using port 7777
lsof -i :7777
# Kill it or wait for timeout, then retry
```

### "Connection refused"
- Make sure daemon is running in Terminal 1
- Check if server started with: `cargo run -p aofctl -- serve ...`
- Verify no firewall blocking localhost:7777

### "YAML parsing error"
- All provided configs are pre-validated
- Don't edit YAML files manually (2-space indentation required)
- Use provided files in `quickstart/` directory

## Documentation Guide

- **SETUP.md** ← Start here for quick setup
- **MINIONS.md** ← Quick reference of all 11 agents
- **AGENTS.md** ← Detailed specs and use cases for each agent
- **README.md** ← This file (overview and scenarios)

## Next Steps

1. **Set API key:** `export GOOGLE_API_KEY="your-key"`
2. **Try first agent:** `aofctl run agent quickstart/agents/kubo.yaml --prompt "Hello!"`
3. **Run full stack:** Follow Scenario 2 above
4. **Pick your minion:** Choose your favorite from `quickstart/agents/` and start working
5. **Create workflows:** Combine multiple minions for complex tasks

---

**Your testing infrastructure is ready! 🚀**

For all 11 agent details, see **MINIONS.md**.
