# AOF Local Testing Setup (gojo/)

This directory contains working test configurations for AOF using **Google Gemini 2.5 Flash**.

## Prerequisites

1. **Google API Key** - Get from https://aistudio.google.com/apikey
2. **Rust** - Build the project
3. **Cargo** - For running commands

## Setup

### Step 1: Set Your Google API Key

```bash
export GOOGLE_API_KEY="your-api-key-here"
```

**To persist across terminal sessions:**
```bash
echo 'export GOOGLE_API_KEY="your-api-key-here"' >> ~/.zshrc
source ~/.zshrc
```

### Step 2: Build AOF

```bash
cd /Users/gshah/work/opsflow-sh/aof
cargo build --release
```

## Testing

### Configuration Files

- **serve-config.yaml** - Main server configuration with 3 pre-configured test agents
- **agents/** - Individual agent YAML files for testing

### Available Test Agents

1. **quick-test** - Simple general-purpose agent (good for first test)
   ```bash
   aofctl run agent gojo/agents/quick-test.yaml --prompt "Say hello and tell me about yourself"
   ```

2. **k8s-checker** - Kubernetes cluster health checker
   ```bash
   aofctl run agent gojo/agents/k8s-checker.yaml --prompt "Check the health of my cluster"
   ```

3. **system-monitor** - System resource and performance monitor
   ```bash
   aofctl run agent gojo/agents/system-monitor.yaml --prompt "Check my system resources"
   ```

4. **general-assistant** - From serve-config (multi-turn capable)
   ```bash
   aofctl run agent --name general-assistant --interactive
   ```

### Test Scenarios

#### Scenario 1: Quick Agent Test (5 minutes)

```bash
# Terminal 1: Start the daemon
export GOOGLE_API_KEY="your-api-key-here"
cargo run -p aofctl -- serve --config gojo/serve-config.yaml

# Terminal 2: Run a quick test
export GOOGLE_API_KEY="your-api-key-here"
aofctl run agent gojo/agents/quick-test.yaml --prompt "What's the capital of France?"
```

**Expected output:**
```
Agent: quick-test
Status: executing...

Response: Paris is the capital of France. It's known as "The City of Light" and is...
```

#### Scenario 2: Full Integration Test (15 minutes)

```bash
# Terminal 1: Start the daemon
export GOOGLE_API_KEY="your-api-key-here"
cargo run -p aofctl -- serve --config gojo/serve-config.yaml

# Terminal 2: Open web UI
cd web-ui
npm run dev
# Visit http://localhost:5173

# Terminal 3: Run an agent
export GOOGLE_API_KEY="your-api-key-here"
aofctl run agent gojo/agents/quick-test.yaml --prompt "Tell me a joke"

# You should see:
# - Agent execution in Terminal 1 logs
# - Real-time events in web UI (http://localhost:5173)
# - Response in Terminal 3
```

#### Scenario 3: Interactive Agent (testing multi-turn conversation)

```bash
# Use --interactive flag for multi-turn conversation
export GOOGLE_API_KEY="your-api-key-here"

# Option A: Use pre-configured agent from serve-config
aofctl run agent --name general-assistant --interactive

# Option B: Use standalone agent file
aofctl run agent gojo/agents/quick-test.yaml --interactive

# Then type your questions and have a conversation
```

### Health Checks

Once serve is running, verify it's working:

```bash
# Health probe (liveness check)
curl http://localhost:8080/health
# Expected: {"status":"healthy","uptime":"...","version":"..."}

# Readiness probe
curl http://localhost:8080/ready
# Expected: {"ready":true,"checks":{...}}

# Metrics endpoint
curl http://localhost:8080/metrics
# Expected: Prometheus metrics in text format
```

## Troubleshooting

### Error: "No API key configured"

**Solution:**
```bash
export GOOGLE_API_KEY="your-api-key-here"
echo $GOOGLE_API_KEY  # Verify it's set
```

### Error: "YAML parsing error at line X"

**Cause:** YAML indentation or syntax error in config file

**Solution:** 
- Check indentation (2 spaces, not tabs)
- Use provided configs in this directory (they're pre-validated)
- Run: `cargo run -p aofctl -- validate --file gojo/serve-config.yaml`

### Error: "Connection refused on localhost:8080"

**Solution:**
- Make sure `aofctl serve` is running in Terminal 1
- Check the port: `lsof -i :8080`
- Verify config has `port: 8080`

### Web UI shows "Disconnected"

**Solution:**
- Ensure daemon is running: `aofctl serve --config gojo/serve-config.yaml`
- Check browser console for WebSocket errors
- Verify backend is on http://localhost:8080

## File Structure

```
gojo/
├── README.md                 # This file
├── SETUP.md                  # Quick reference
├── serve-config.yaml         # Main server config (3 pre-configured agents)
├── agents/
│   ├── quick-test.yaml       # Simple test agent
│   ├── k8s-checker.yaml      # Kubernetes health checker
│   └── system-monitor.yaml   # System resource monitor
├── test-quick.sh             # Quick 5-minute test
└── test-interactive.sh       # Interactive multi-turn test
```

## Using Anthropic (Claude) Instead

If you want to use your Anthropic subscription (Claude Sonnet, etc.):

1. Set your API key:
   ```bash
   export ANTHROPIC_API_KEY="sk-..."
   ```

2. Update models in config files:
   ```yaml
   # Change:
   model: "google:gemini-2.5-flash"
   
   # To:
   model: "anthropic:claude-3-5-sonnet"
   ```

3. Or use directly in CLI:
   ```bash
   aofctl run agent gojo/agents/quick-test.yaml \
     --model "anthropic:claude-3-5-sonnet" \
     --prompt "Your question here"
   ```

## Next Steps

After confirming basic functionality:

1. **Run full test suite:** `cargo test --all`
2. **Check metrics:** Visit http://localhost:8080/metrics while agent is running
3. **Try web UI:** http://localhost:5173 with daemon running
4. **Explore agents:** Create your own agent YAML files

## Documentation

- **User Guide:** ../../docs/introduction/quickstart.md
- **Architecture:** ../../docs/architecture/implementation-guide.md
- **Agent Specs:** ../../docs/user-guide/agents/
- **CLI Reference:** `aofctl --help`

---

**Ready to test? Start with Scenario 1 above! 🚀**
