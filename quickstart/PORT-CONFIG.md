# Port Configuration Guide

## Current Setup

- **Default Port: 7777** (uncommon port to avoid conflicts)
- **Why 7777?** Avoids conflicts with common development ports (8000, 8080, 3000, 5000)
- **All services use 7777:**
  - Backend API: `http://localhost:7777`
  - WebSocket: `ws://localhost:7777`
  - Health check: `http://localhost:7777/health`
  - Metrics: `http://localhost:7777/metrics`

## How It Works

### 1. Server Configuration
**File:** `quickstart/serve-config.yaml`
```yaml
spec:
  server:
    host: "127.0.0.1"
    port: 7777  # ← Change here for server
```

### 2. Web UI Configuration
**File:** `web-ui/.env.local`
```bash
VITE_API_URL=http://localhost:7777     # ← Change here for web-ui API
VITE_WS_URL=ws://localhost:7777        # ← Change here for web-ui WebSocket
```

### 3. No Code Changes Needed ✅

The port is **fully configurable** via:
- ✅ Configuration file (`serve-config.yaml`)
- ✅ Environment variables (`web-ui/.env.local`)
- ✅ NO changes to Rust source code required
- ✅ NO changes to TypeScript/React code required

## Changing to a Different Port

To use a different port (e.g., 9000):

### Step 1: Update Server Config
```bash
# Edit quickstart/serve-config.yaml
# Change: port: 7777
# To: port: 9000
```

### Step 2: Update Web UI Config
```bash
# Edit web-ui/.env.local
# Change: VITE_API_URL=http://localhost:7777
# To: VITE_API_URL=http://localhost:9000
# 
# Change: VITE_WS_URL=ws://localhost:7777
# To: VITE_WS_URL=ws://localhost:9000
```

### Step 3: Restart Services
```bash
# Terminal 1: Stop old server (Ctrl+C), start new:
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml

# Terminal 2: Stop old web-ui (Ctrl+C), start new:
cd web-ui
pnpm run dev
```

## Why This Works

The architecture is **port-agnostic**:

1. **Backend (Rust):**
   - Reads port from `serve-config.yaml`
   - No hardcoded port values
   - All endpoints dynamically use configured port

2. **Web UI (React/Vite):**
   - Reads API URL from environment variables (`VITE_API_URL`, `VITE_WS_URL`)
   - Configured via `.env.local` file
   - Fully dynamic - can point to any port

3. **CLI (aofctl):**
   - Agents run independently
   - No port dependency for agent execution
   - Only daemon needs port config

## Files Changed

- ✅ `quickstart/serve-config.yaml` - Port set to 7777
- ✅ `web-ui/.env.local` - URLs point to 7777
- ✅ Documentation (SETUP.md, README.md, AGENTS.md, MINIONS.md) - Updated to 7777

## Port Conflicts?

If you still see "Address already in use":

```bash
# Find what's using the port
lsof -i :7777

# Kill the process
kill -9 <PID>

# Or use a completely different port:
# Edit serve-config.yaml: port: 6666
# Edit web-ui/.env.local: VITE_API_URL=http://localhost:6666
# Restart both services
```

## Testing the New Port

```bash
# Verify server is on port 7777
curl http://localhost:7777/health

# Check WebSocket
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" ws://localhost:7777/ws

# Verify web-ui config
cat web-ui/.env.local
```

---

**Summary:** Port 7777 is configured, zero code changes needed, fully customizable! 🚀
