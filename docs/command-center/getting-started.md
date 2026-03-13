# Getting Started with Command Center

## Prerequisites

- Node.js 18 or newer
- A running agentix gateway (see [gateway quickstart](../guides/quickstart.md))

## Quick Start

```bash
# From the repository root
cd apps/command-center
npm install
npm run dev
```

The dev server starts on `http://localhost:5173`.

## First-Run Wizard

On the first visit (no gateway URL saved), the Command Center automatically opens a 4-step setup wizard:

1. **Welcome** — Introduction to the Command Center.
2. **Connect to Gateway** — Enter your gateway URL (default `http://localhost:7777`) and click **Test Connection** to verify the gateway is reachable. The wizard saves the URL to `localStorage` on success.
3. **Explore** — See how many agents are registered on the connected gateway. Quick links to the Dashboard and Agent Builder.
4. **Done** — Summary of available sections. Click **Open Dashboard** to complete setup.

You can re-launch the wizard at any time from **Settings → Re-launch Setup Wizard**.

## Connecting to a Remote Gateway

If your agentix gateway is running on a different host, update the Gateway URL in the wizard or in Settings:

```
http://your-gateway-host:7777
```

The URL is saved to `localStorage` and used for all API calls. No rebuild is required.

## Building for Production

```bash
cd apps/command-center
npm run build
```

This produces a fully static site in the `build/` directory. The adapter is configured with `fallback: 'index.html'` for SPA routing.

## Deploying

Serve the `build/` directory with any static file server:

```bash
# With Node.js serve package
npx serve build

# With Caddy
caddy file-server --root apps/command-center/build --listen :3000

# With nginx (minimal config)
server {
  listen 3000;
  root /path/to/apps/command-center/build;
  location / {
    try_files $uri $uri/ /index.html;
  }
}
```

## Environment

The Command Center reads the gateway URL from `localStorage` at key `agentix-gateway-url`. It does not use environment variables at runtime — all configuration is done in the browser via the Settings page or first-run wizard.

## Troubleshooting

**Connection failed in wizard**
- Ensure the gateway is running: `agentix gateway status`
- Check that CORS is not blocking the request (gateway allows all origins by default)
- Verify the URL includes the correct port (default `7777`)

**No agents shown**
- Register agents first: `agentix apply -f agents/my-agent/`
- Or create one in the Agent Builder

**WebSocket not connecting**
- The WebSocket endpoint is at `ws://your-gateway:7777/ws`
- Check the browser console for connection errors
- The Command Center auto-reconnects with exponential backoff (1s → 30s)
