# Web UI Setup Guide

## Prerequisites

1. **pnpm** installed - Install globally if needed:
   ```bash
   npm install -g pnpm
   ```

2. **Node.js 18+** - Required for the web UI

## First-Time Setup

### Step 1: Install Dependencies
```bash
cd /Users/gshah/work/opsflow-sh/aof/web-ui

# Install dependencies using pnpm
pnpm install
```

### Step 2: Create Environment Config
```bash
# Create .env.local from template
cp .env.local.template .env.local

# Verify it has correct port (should be 7777)
cat .env.local
# Expected output:
# VITE_API_URL=http://localhost:7777
# VITE_WS_URL=ws://localhost:7777
```

### Step 3: Start Development Server
```bash
# From web-ui directory
pnpm run dev

# Expected output:
# VITE v7.3.1  ready in XXX ms
# ➜  Local:   http://localhost:5173/
# ➜  press h + enter to show help
```

## Usage

### For Full Stack Testing
**Terminal 1: Backend Daemon**
```bash
cd /Users/gshah/work/opsflow-sh/aof
export GOOGLE_API_KEY="your-key"
cargo run -p aofctl -- serve --config quickstart/serve-config.yaml
```

**Terminal 2: Web UI**
```bash
cd web-ui
pnpm run dev
# Visit: http://localhost:5173
```

**Terminal 3: Run Agents**
```bash
cd /Users/gshah/work/opsflow-sh/aof
export GOOGLE_API_KEY="your-key"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Hello!"
```

## Available Commands

```bash
pnpm run dev          # Start development server (http://localhost:5173)
pnpm run build        # Production build
pnpm run build:analyze # Build with analysis
pnpm run lint         # Run ESLint
pnpm run preview      # Preview production build
pnpm run test         # Run tests with Vitest
pnpm run test:ui      # Run tests with UI
pnpm run test:coverage # Run tests with coverage
```

## Troubleshooting

### "Missing script: dev"
```bash
# Likely cause: Dependencies not installed
pnpm install

# Then try again
pnpm run dev
```

### "ERR_PNPM_NO_MATCHING_VERSION"
```bash
# Clear pnpm cache
pnpm store prune

# Reinstall dependencies
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Port 5173 already in use
```bash
# Find what's using port 5173
lsof -i :5173

# Kill the process
kill -9 <PID>

# Or use Vite's port override
pnpm run dev -- --port 5174
```

### Web UI shows "Disconnected"
```bash
# Ensure:
# 1. Backend daemon is running (see Terminal 1 above)
# 2. Port 7777 is accessible: curl http://localhost:7777/health
# 3. .env.local has correct URLs:
cat web-ui/.env.local
# Should show:
# VITE_API_URL=http://localhost:7777
# VITE_WS_URL=ws://localhost:7777

# If still disconnected, restart web UI
# Ctrl+C in Terminal 2, then: pnpm run dev
```

## Performance

- **First build**: 5-10 seconds (depends on system)
- **HMR (hot reload)**: < 500ms
- **Production build**: 10-20 seconds

## Development Workflow

1. Start daemon in Terminal 1
2. Start web UI in Terminal 2
3. Make changes to web UI code
4. Changes auto-reload (HMR)
5. Run agents in Terminal 3 to see real-time updates

---

**Web UI is now ready for development!** 🎉
