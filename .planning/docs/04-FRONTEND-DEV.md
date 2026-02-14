# Phase 4: Mission Control UI - Frontend Development Guide

## Overview

Phase 4 delivers the Mission Control UI - a real-time React dashboard connected to Phase 1's WebSocket event stream.

## Technology Stack

- **React 19.2** + **TypeScript 5.9** (strict mode)
- **Redux Toolkit 2.11** + **React Redux 9.2**
- **Tailwind CSS 4.1** + **Vite 7.3**

## Key Architecture Decisions

### Redux Store Structure

```typescript
{
  events: {
    events: CoordinationEvent[],  // Capped at 500
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

### Custom Hooks

**useWebSocket:** Automatic reconnection with exponential backoff (1s → 30s cap)

**useAgentsConfig / useToolsConfig:** Graceful 404 handling, version tracking

### TypeScript Patterns

- **Type-only imports:** Required by `verbatimModuleSyntax`
- **No enums:** Use string literal types
- **Centralized exports:** `src/types/index.ts`

## WebSocket Connection

**Dev:** Browser → Vite proxy → localhost:8080/ws
**Prod:** Browser → location.host/ws (wss:// if HTTPS)

## Build Optimization

- **Bundle:** 71KB gzipped (target <500KB)
- **Terser:** Drops console.log in production
- **Manual chunks:** Vendor (React/Redux) separated

## Phase Handoff

**For 04-02:** Redux store + StatusIndicator + useAgentsConfig
**For 04-03:** WebSocket infrastructure + event streaming
**For 04-04:** Optimized dist/ folder ready for static serving

---

**Last Updated:** 2026-02-14
**Phase:** 4-01 ✓
