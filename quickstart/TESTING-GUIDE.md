# AOF Quickstart - Complete Testing Guide

Your first-bot testing infrastructure with 11 pre-configured agent minions.

## 📋 Table of Contents

1. [Setup](#setup)
2. [Test Levels](#test-levels)
3. [Testing Agents](#testing-agents)
4. [Full Stack Testing](#full-stack-testing)
5. [Integration Scenarios](#integration-scenarios)
6. [Troubleshooting](#troubleshooting)

---

## Setup

### Prerequisites
```bash
# 1. Get Google API Key
https://aistudio.google.com/apikey

# 2. Set environment variable
export GOOGLE_API_KEY="your-key-from-above"

# 3. Navigate to project
cd /Users/gshah/work/opsflow-sh/aof
```

### Optional: Create Web UI Config
```bash
# One-time setup for web UI (already configured for port 7777)
cd web-ui
cp .env.local.template .env.local
# Verify: cat .env.local shows correct port 7777
```

---

## Test Levels

### Level 1: Individual Agent (30 seconds)
**What it tests:** Single agent functionality, model connection, basic I/O

```bash
export GOOGLE_API_KEY="your-key"

# Run any agent with a prompt
aofctl run agent quickstart/agents/kubo.yaml --prompt "Say hello"

# Expected: Agent responds within 5-10 seconds
```

**Try each agent:**
```bash
aofctl run agent quickstart/agents/doku.yaml --prompt "What's Docker?"
aofctl run agent quickstart/agents/rafo.yaml --prompt "Explain Terraform"
aofctl run agent quickstart/agents/wos.yaml --prompt "What's serverless?"
aofctl run agent quickstart/agents/nux.yaml --prompt "How to check system load?"
```

### Level 2: Interactive Mode (2-5 minutes)
**What it tests:** Multi-turn conversation, context retention, follow-ups

```bash
export GOOGLE_API_KEY="your-key"

# Start interactive session
aofctl run agent quickstart/agents/kubo.yaml --interactive

# Then ask questions:
# Q: What are pod replicas?
# Q: How do I increase them?
# Q: What about resource limits?
# Press Ctrl+C to exit
```

**Key observations:**
- ✓ Agent maintains context
- ✓ Answers are coherent and related
- ✓ Can ask follow-up questions
- ✓ Graceful exit with Ctrl+C

### Level 3: Server Daemon (5 minutes)
**What it tests:** Server startup, port binding, health checks, agent discovery

```bash
export GOOGLE_API_KEY="your-key"

# Terminal 1: Start daemon
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml

# Expected output:
# Starting AOF Trigger Server
# Bind address: 0.0.0.0:7777
# Event bus: initialized
# Server starting...
# Press Ctrl+C to stop

# Terminal 2: Health check
curl http://localhost:7777/health
# Expected: {"status":"healthy",...}

# Terminal 2: List agents
curl http://localhost:7777/api/config/agents
# Expected: JSON with all 11 agents
```

**Key observations:**
- ✓ Server starts on port 7777
- ✓ Health check responsive
- ✓ Agents auto-discovered
- ✓ WebSocket ready at ws://localhost:7777/ws

### Level 4: Full Stack (10-15 minutes)
**What it tests:** Backend + Web UI integration, real-time events, complete workflow

```bash
# Terminal 1: Start backend daemon
export GOOGLE_API_KEY="your-key"
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml

# Terminal 2: Start web UI
cd web-ui
npm run dev
# Wait for: "Local: http://localhost:5173"

# Terminal 3: Run agent
export GOOGLE_API_KEY="your-key"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Check cluster"

# Meanwhile in Terminal 2: Open browser to http://localhost:5173
# You should see:
# - Real-time agent execution
# - Events streaming in
# - Agent response in Terminal 3
```

**Key observations:**
- ✓ Web UI connects to daemon
- ✓ Real-time events visible
- ✓ No connection errors
- ✓ Complete execution visible in both places

---

## Testing Agents

### Specialized Agents by Category

#### Infrastructure & Cloud
```bash
# Kubernetes testing
aofctl run agent quickstart/agents/kubo.yaml --interactive
# Test: "How do I create a deployment?"

# Docker testing
aofctl run agent quickstart/agents/doku.yaml --interactive
# Test: "Best practices for Dockerfile"

# AWS testing
aofctl run agent quickstart/agents/wos.yaml --interactive
# Test: "Design a Lambda-based microservice"

# Azure testing
aofctl run agent quickstart/agents/zure.yaml --interactive
# Test: "Azure DevOps pipeline setup"
```

#### Infrastructure-as-Code & Automation
```bash
# Terraform testing
aofctl run agent quickstart/agents/rafo.yaml --interactive
# Test: "Create a multi-environment setup"

# Ansible testing
aofctl run agent quickstart/agents/zibl.yaml --interactive
# Test: "Write a deployment playbook"

# GitOps testing
aofctl run agent quickstart/agents/ergo.yaml --interactive
# Test: "Design an Argo CD workflow"
```

#### System Operations
```bash
# Linux system admin
aofctl run agent quickstart/agents/nux.yaml --interactive
# Test: "System is running slow, diagnose it"

# System monitoring
aofctl run agent quickstart/agents/system-monitor.yaml --prompt "Show CPU usage"

# Kubernetes diagnostics
aofctl run agent quickstart/agents/k8s-checker.yaml --prompt "Cluster health?"
```

---

## Full Stack Testing

### Test Scenario 1: Development Workflow
```bash
# Terminal 1: Daemon
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml

# Terminal 2: Web UI
cd web-ui && npm run dev

# Terminal 3: Multiple sequential tests
export GOOGLE_API_KEY="your-key"

# Test 1: Infrastructure design
aofctl run agent quickstart/agents/rafo.yaml \
  --prompt "Design Terraform for 3-tier app on AWS"

# Test 2: Container strategy
aofctl run agent quickstart/agents/doku.yaml \
  --prompt "Multi-stage Dockerfile for Node.js"

# Test 3: Deployment automation
aofctl run agent quickstart/agents/zibl.yaml \
  --prompt "Ansible playbook for app deployment"
```

### Test Scenario 2: Troubleshooting Workflow
```bash
# Terminal 1-2: Full stack running

# Terminal 3: Simulated troubleshooting
export GOOGLE_API_KEY="your-key"

# Step 1: Check system
aofctl run agent quickstart/agents/nux.yaml \
  --prompt "System is at 95% CPU, what's using it?"

# Step 2: Investigate K8s if applicable
aofctl run agent quickstart/agents/kubo.yaml \
  --prompt "My pods are crashing, help debug"

# Step 3: Get suggestions
aofctl run agent quickstart/agents/kubo.yaml --interactive
# Ask: "What monitoring should I set up?"
# Ask: "How to prevent this in future?"
```

### Test Scenario 3: Learning Mode
```bash
# Great for understanding concepts

export GOOGLE_API_KEY="your-key"

# Deep dive into topic
aofctl run agent quickstart/agents/kubo.yaml --interactive

# Questions to ask:
# - "Explain Kubernetes architecture"
# - "What's a sidecar container?"
# - "How does service discovery work?"
# - "Design a multi-cluster setup"

# Take notes on what works for your use case
```

---

## Integration Scenarios

### Scenario A: Multi-Agent Design Review
Ask different agents to review the same problem:

```bash
PROMPT="Design a microservices architecture on AWS with auto-scaling"

# Get cloud perspective
aofctl run agent quickstart/agents/wos.yaml --prompt "$PROMPT"

# Get container perspective
aofctl run agent quickstart/agents/doku.yaml --prompt "$PROMPT"

# Get IaC perspective
aofctl run agent quickstart/agents/rafo.yaml --prompt "$PROMPT"

# Compare answers and synthesize best approach
```

### Scenario B: Implementation Planning
Sequential agents for end-to-end planning:

```bash
# 1. Design phase
aofctl run agent quickstart/agents/rafo.yaml \
  --prompt "Design Terraform modules for our app"

# 2. Container phase
aofctl run agent quickstart/agents/doku.yaml \
  --prompt "Create Dockerfiles for app and DB"

# 3. Orchestration phase
aofctl run agent quickstart/agents/kubo.yaml \
  --prompt "Design K8s manifests for deployment"

# 4. Automation phase
aofctl run agent quickstart/agents/zibl.yaml \
  --prompt "Create Ansible playbook for setup"

# 5. CI/CD phase
aofctl run agent quickstart/agents/ergo.yaml \
  --prompt "Design Argo CD workflow for deployment"
```

### Scenario C: Real-time Problem Solving
```bash
# Terminal 1-2: Full stack running

# Terminal 3: Interactive debugging
export GOOGLE_API_KEY="your-key"

aofctl run agent quickstart/agents/nux.yaml --interactive

# Type:
# Q: My app crashed, logs show "out of memory"
# Q: How do I find the memory leak?
# Q: What's the fastest way to fix it?
# Q: How do I prevent this?

# Real conversation, real solutions
```

---

## Troubleshooting

### Agent Tests

**Agent doesn't respond:**
```bash
# Check API key
echo $GOOGLE_API_KEY

# Try with verbose output
export RUST_LOG=debug
aofctl run agent quickstart/agents/quick-test.yaml --prompt "Test"
```

**Slow responses:**
- Network latency
- Model overload (Google Gemini)
- Try simpler prompt: `--prompt "Hello"`

**Agent output seems off:**
- Try interactive mode for context
- Ask follow-up question
- Switch to different model: `--model "anthropic:claude-3-5-sonnet"`

### Server Tests

**Port already in use:**
```bash
# Check what's using 7777
lsof -i :7777

# Kill it
kill -9 <PID>

# Or change port in quickstart/serve-config.yaml
# Then: export VITE_API_URL=http://localhost:9000 in web-ui/.env.local
```

**Connection refused:**
```bash
# Verify daemon is running
# Check Terminal 1 output for: "Server starting..."
# Verify correct port in serve-config.yaml

# Try health check with verbose
curl -v http://localhost:7777/health
```

**Web UI shows "Disconnected":**
```bash
# Check web-ui/.env.local
cat web-ui/.env.local

# Verify it matches serve-config port (7777)
# Restart web-ui: npm run dev in Terminal 2
```

### Full Stack Tests

**Agents don't appear in web UI:**
```bash
# Check agent discovery in serve-config.yaml
# Should be: - "./quickstart/agents"

# Verify agents exist
ls quickstart/agents/

# Restart daemon
```

**No real-time updates in web UI:**
```bash
# Check WebSocket connection
# Browser dev tools → Network → find ws connection

# Check Terminal 1 (daemon) for errors
# May need to clear browser cache and refresh
```

---

## Test Checklist

### Pre-Testing
- [ ] GOOGLE_API_KEY set
- [ ] quickstart/ directory exists with agents
- [ ] Port 7777 available
- [ ] Rust installed and can build

### Level 1 Tests
- [ ] Single agent responds to prompt (30s)
- [ ] Different agents give relevant answers
- [ ] Response quality is acceptable

### Level 2 Tests
- [ ] Interactive mode starts (Ctrl+C exits)
- [ ] Multi-turn conversation works
- [ ] Agent maintains context across questions

### Level 3 Tests
- [ ] Daemon starts on port 7777
- [ ] Health check returns 200
- [ ] Agent discovery finds all 11 agents
- [ ] WebSocket endpoint responds

### Level 4 Tests
- [ ] Web UI connects without errors
- [ ] Agent execution shows in web UI
- [ ] Real-time events stream
- [ ] Complete workflow end-to-end

### Integration Tests
- [ ] Multi-agent scenario (different perspectives)
- [ ] Sequential agent workflow
- [ ] Real-time problem solving

---

## Performance Targets

| Test | Target | Notes |
|------|--------|-------|
| Agent response | < 10s | Network + model latency |
| Interactive mode | < 5s per turn | Subsequent queries |
| Server startup | < 5s | Port binding + initialization |
| Web UI connection | < 2s | WebSocket handshake |
| Full stack execution | < 30s end-to-end | Daemon + UI + agent |

---

## Next Steps

After successful testing:

1. **Customize agents** - Modify prompts in `quickstart/agents/` YAML files
2. **Create workflows** - Use multiple agents for complex tasks
3. **Integrate with code** - Use agents in your applications
4. **Deploy to production** - Production-ready setup available

---

**Ready to test? Start with Level 1!** 🚀

For individual agent details, see **MINIONS.md** or **AGENTS.md**.
