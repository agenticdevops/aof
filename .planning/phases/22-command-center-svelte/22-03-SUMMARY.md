---
phase: 22-command-center-svelte
plan: "03"
subsystem: command-center-ui
tags: [svelte, typescript, websocket, dashboard, agents, runs, real-time]
dependency_graph:
  requires:
    - phase: 22-command-center-svelte
      plan: "01"
      provides: SvelteKit scaffold, API client, sidebar, UI components
    - phase: 22-command-center-svelte
      plan: "02"
      provides: /ws WebSocket endpoint on gateway
  provides:
    - WebSocket store with auto-reconnect and typed GatewayEvent distribution
    - Agent/run reactive stores wired to WebSocket events
    - Dashboard landing page (metrics bar + activity feed + quick panels)
    - Agents page (card grid with live status, search + filter)
    - Agent detail page (tabbed: overview, runs, config YAML)
    - Runs history page (filterable table with pagination)
  affects:
    - 22-04 (costs dashboard — can import agents store)
    - 22-05 (traces view — can import run data)
    - 22-06 (approvals — can import approval events from wsEvents)
tech_stack:
  added: []
  patterns:
    - Svelte 5 runes ($state, $derived, $effect, $props)
    - Writable stores with error/loading companions
    - WebSocket auto-reconnect with exponential backoff (1s→30s max)
    - Reactive WebSocket subscribers in stores (agent_status updates in-place)
    - Client-side filtering + pagination with $derived
key_files:
  created:
    - apps/command-center/src/lib/stores/websocket.ts
    - apps/command-center/src/lib/stores/agents.ts
    - apps/command-center/src/lib/components/metrics-bar.svelte
    - apps/command-center/src/lib/components/activity-feed.svelte
    - apps/command-center/src/lib/components/agent-card.svelte
    - apps/command-center/src/lib/components/agent-detail.svelte
    - apps/command-center/src/lib/components/run-row.svelte
    - apps/command-center/src/routes/agents/+page.svelte
    - apps/command-center/src/routes/agents/[name]/+page.svelte
    - apps/command-center/src/routes/runs/+page.svelte
  modified:
    - apps/command-center/src/routes/+layout.svelte
    - apps/command-center/src/routes/+page.svelte
decisions:
  - "Agent detail implemented as full page with back link (not slide-over) for URL-driven navigation"
  - "Activity feed keeps last 20 WebSocket events in local $state array, prepend-newest pattern"
  - "Run table uses client-side pagination (25/page) with $derived for filter/page combos"
  - "agent_status WebSocket events update agents store in-place (no full re-fetch for status changes)"
  - "run_completed events trigger full run list reload for the affected agent (freshness over optimization)"
  - "wsEvents store distributes null initially; subscribers must guard with 'if (!event) return'"
metrics:
  duration_minutes: 7
  completed_date: "2026-03-13"
  tasks_completed: 2
  tasks_total: 2
  files_created: 10
  files_modified: 2
---

# Phase 22 Plan 03: Dashboard, Agent Views & Run History Summary

**One-liner:** WebSocket store with exponential-backoff reconnect + agent/run reactive stores, wired to a full mission-control dashboard, live agent card grid, tabbed agent detail page, and paginated run history with date/status/agent filters.

---

## What Was Built

### Task 1: WebSocket store + agent/run stores (commit fbaefd3)

**websocket.ts:**
- `initWebSocket()` / `disconnectWebSocket()` lifecycle functions for layout mount/destroy
- Reads `gatewayUrl` store, converts `http://` → `ws://` and `https://` → `wss://`
- Connects to `${wsUrl}/ws`, parses JSON frames as typed `GatewayEvent` union
- `wsStatus` readable store: `'connecting' | 'connected' | 'disconnected'`
- `wsEvents` readable store: latest `GatewayEvent | null`
- Exponential backoff reconnect: 1s → 2s → 4s → ... → 30s max, reset on successful connection

**GatewayEvent union types:**
- `agent_status` — agent name + new status
- `run_started` — agent name, run_id, trigger_source
- `run_completed` — agent name, run_id, status, duration_ms, cost_usd
- `approval_requested` / `approval_decided` — approval id, agent, decision
- `cost_update` — agent name, cost, total_today_usd
- `connected` — gateway handshake confirmation

**agents.ts:**
- `agents` writable store (Agent[]) with `agentsLoading` and `agentsError` companions
- `selectedAgent` writable store (Agent | null) for detail page
- `agentRuns` writable store (AgentRun[]) for detail runs tab
- `loadAgents()` / `loadAgent(name)` / `loadAgentRuns(name)` async actions with try/catch error handling
- WebSocket subscriber: `agent_status` events update matching agent in-place; `run_started` marks agent as running; `run_completed` triggers full run reload if viewing that agent

### Task 2: Dashboard + pages + components (commit 5baeaeb)

**+layout.svelte** — Added `onMount(initWebSocket)` / `onDestroy(disconnectWebSocket)` for global WS connection.

**metrics-bar.svelte** — 4-metric responsive grid: Total Agents (Bot), Active Runs (PlayCircle), Costs Today (DollarSign), Pending Approvals (ShieldCheck). Tailwind grid-cols-1/2/4 with Lucide icons. Color-coded: info/success/warning/destructive.

**activity-feed.svelte** — Live event log card. Renders up to 20 GatewayEvent objects with per-type icons and human-readable descriptions. Empty state with Wifi icon and prompt text.

**agent-card.svelte** — Clickable anchor card navigating to `/agents/${name}`. Status badge (green=running, red=error, blue=scheduled, gray=idle), mode badge, trigger count, skill count. Hover shadow effect.

**agent-detail.svelte** — Tabbed component (Overview | Runs | Config). Overview: model, mode, triggers list, skills, budget. Runs: RunRow table. Config: auto-generated YAML display in `<pre>`. Run count badge on tab, total cost in overview summary.

**run-row.svelte** — Table row anchor linking to `/traces/${agent}/${runId}`. Shows: agent name, truncated run_id, status badge, trigger source, relative time, duration (formatted as "Xs" or "Xm Ys"), cost.

**routes/+page.svelte** — Dashboard landing: header with refresh button, error banner with retry, MetricsBar skeleton → MetricsBar, empty state (no agents), 2-col layout with ActivityFeed + quick panels (Active Runs + Pending Approvals cards).

**routes/agents/+page.svelte** — Agent card grid (1/2/3 cols). Search input + status filter dropdown. Skeleton grid (6 cards) while loading. Two empty states: no agents at all (→ builder), filtered to nothing (clear filters link).

**routes/agents/[name]/+page.svelte** — Full page with back link. Reads `page.params.name`, calls `loadAgent` + `loadAgentRuns` on mount. Skeleton, not-found, and detail states.

**routes/runs/+page.svelte** — Run history table. Agent dropdown, status dropdown, date start/end inputs, clear-filters button. Client-side pagination (25 per page). Previous/Next controls with disabled states. Run count summary line.

---

## Verification

```
npx tsc --noEmit: PASSED (no output)
npm run build: PASSED (✓ built in 7.55s — site written to build/)
```

svelte-check reports 2 errors in `cost-line-chart.svelte` and `costs/+page.svelte` — these are untracked files from Plan 22-04 (running in parallel), not created or modified by this plan. Deferred per scope boundary rule.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] page.params.name typed as string | undefined in SvelteKit 2.x**
- **Found during:** Task 2 svelte-check
- **Issue:** `$page.params.name` is `string | undefined` in SvelteKit's type system even for required params
- **Fix:** Added `?? ''` fallback and guarded `loadAgent` / `loadAgentRuns` calls with `if (agentName)`
- **Files modified:** `routes/agents/[name]/+page.svelte`
- **Commit:** 5baeaeb

### Out-of-Scope Issues (Deferred)

Two svelte-check errors in files created by Plan 22-04 (cost charts) are present in the working tree but not committed or modified by Plan 22-03. Per scope boundary rule, these are deferred to Plan 22-04's scope to fix before their commit.

---

## Self-Check: PASSED

All 12 files found. Both task commits verified: fbaefd3, 5baeaeb. Build: succeeded (7.55s). tsc --noEmit: clean.
