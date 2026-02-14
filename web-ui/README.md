# AOF Mission Control - Web UI

Real-time web dashboard for the AOF (Agentic Ops Framework) Mission Control UI, built with React, Redux Toolkit, and Tailwind CSS.

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Setup

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Start Phase 1 backend:**
   ```bash
   # In the parent aof directory
   cargo run -p aofctl -- serve --config serve-config.yaml
   ```

3. **Start development server:**
   ```bash
   npm run dev
   ```

4. **Open browser:**
   Visit http://localhost:5173

The development server will automatically proxy API requests and WebSocket connections to localhost:8080.

## Project Structure

```
web-ui/
├── src/
│   ├── components/     # React components (StatusIndicator, etc.)
│   ├── hooks/          # Custom hooks (useWebSocket, useAgentsConfig, useToolsConfig)
│   ├── store/          # Redux store and slices (eventsSlice, configSlice)
│   ├── types/          # TypeScript type definitions (CoordinationEvent, Agent, Tool)
│   ├── utils/          # Utility functions
│   ├── App.tsx         # Main application component
│   ├── main.tsx        # Application entry point
│   └── index.css       # Global styles with Tailwind directives
├── public/             # Static assets
├── dist/               # Production build output (gitignored)
├── vite.config.ts      # Vite configuration
├── tailwind.config.js  # Tailwind CSS configuration
└── tsconfig.json       # TypeScript configuration
```

## Redux Store

### Store Structure

```typescript
{
  events: {
    events: CoordinationEvent[],  // Last 500 events
    lastEventId: string,
    connected: boolean
  },
  config: {
    agents: Agent[],
    tools: Tool[],
    configVersion: string
  }
}
```

### Using Redux in Components

```typescript
import { useSelector, useDispatch } from 'react-redux';
import type { RootState } from './store';
import { addEvent, clearEvents } from './store/eventsSlice';

function MyComponent() {
  const events = useSelector((state: RootState) => state.events.events);
  const dispatch = useDispatch();

  // ...
}
```

## WebSocket Hook

### Basic Usage

```typescript
import { useWebSocket } from './hooks/useWebSocket';

function MyComponent() {
  const { connected, lastEvent, reconnectAttempts } = useWebSocket('ws://localhost:8080/ws');

  // connected: boolean - connection status
  // lastEvent: CoordinationEvent | null - last received event
  // reconnectAttempts: number - reconnection attempt count
}
```

### Features

- Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, 16s, 30s cap)
- Redux integration (events dispatched to store automatically)
- Cleanup on unmount (no memory leaks)

## Configuration API

### Fetching Agents Config

```typescript
import { useAgentsConfig } from './hooks/useAgentsConfig';

function MyComponent() {
  const { agents, version, loading, error, refetch } = useAgentsConfig();

  // agents: Agent[] - configured agents
  // version: string - config version from X-Config-Version header
  // loading: boolean - loading state
  // error: Error | null - error state
  // refetch: () => void - manually trigger refetch
}
```

### Fetching Tools Config

```typescript
import { useToolsConfig } from './hooks/useToolsConfig';

function MyComponent() {
  const { tools, version, loading, error, refetch } = useToolsConfig();
  // Same interface as useAgentsConfig
}
```

## Building & Deployment

### Development Build

```bash
npm run dev
```

Features:
- Hot module reload
- Redux DevTools enabled
- Source maps enabled
- Proxies API/WebSocket to localhost:8080

### Production Build

```bash
npm run build
```

Output:
- `dist/` directory with optimized bundle
- Minified with Terser (console.log removed)
- Gzipped: ~71KB
- Target: ES2020

### Serve Static Files

```bash
npx serve dist
```

Test production build locally.

## Troubleshooting

### WebSocket not connecting?

**Symptom:** Connection status shows "Disconnected" or "Reconnecting"

**Solution:**
1. Check if Phase 1 backend is running:
   ```bash
   cargo run -p aofctl -- serve --config serve-config.yaml
   ```
2. Verify backend is listening on http://localhost:8080
3. Check browser DevTools → Network → WS tab for connection errors

### CORS errors?

**Symptom:** Console shows "CORS policy blocked" errors

**Solution:**
1. Check `vite.config.ts` has proxy configuration for `/api` and `/ws`
2. Verify `server.cors: true` is set
3. Restart dev server: `npm run dev`

### Events not appearing?

**Symptom:** Activity log shows "No events received yet"

**Solution:**
1. Open Redux DevTools (browser extension)
2. Check `events.connected` is `true`
3. Trigger agent event in Phase 1:
   ```bash
   aofctl run agent --name test-agent
   ```
4. Check browser console for WebSocket messages
5. Refresh page to reset connection

## License

Apache 2.0 - See LICENSE.md in parent directory.
