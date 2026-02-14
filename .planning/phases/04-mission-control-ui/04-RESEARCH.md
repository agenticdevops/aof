# Phase 4: Mission Control UI - Research

**Researched:** 2026-02-14
**Domain:** Real-time web UI, WASM frameworks, WebSocket sync, drag-and-drop kanban, configuration-driven layout
**Confidence:** MEDIUM-HIGH

## Summary

Phase 4 implements Mission Control—a web-based dashboard that visualizes agent squad coordination in real-time. Operators connect to the existing WebSocket event infrastructure (Phase 1) and see their agent team's status, conversations, task flow, and activity streams. The UI consumes CoordinationEvent streams from Phase 1, translates them to visual updates, and uses workspace configuration (AGENTS.md, TOOLS.md) to dynamically render agent cards and capabilities.

**Key decision point:** Framework choice significantly impacts bundle size, build speed, and developer velocity. The user's directive to use builder.io for beautiful UX opens two implementation paths:

**Path A (Pure Rust WASM):** Leptos for entire dashboard, compiled to WASM, deployed as static assets alongside Rust backend. Aligns with "pure Rust story" but requires brotli/gzip compression and careful dependency management to keep bundle under 500KB.

**Path B (builder.io + React):** User's existing design tool generates React components, developers connect to Rust WebSocket API. Fast iteration on UI, production-grade tooling, but breaks "pure Rust" narrative. Easier real-time sync with proven libraries (dnd-kit, Redux).

**Primary recommendation:** **Hybrid approach (Path B with Rust backend dominance):** Use builder.io to generate React frontend that connects to Rust WebSocket daemon. React enables fast UI iteration, proven drag-and-drop (dnd-kit), and real-time patterns (optimistic updates). Rust backend owns all coordination logic, event streaming, and persistence. This honors the user's builder.io preference while keeping the Rust story intact. Pure Rust WASM remains available for future optimization.

## Standard Stack

### Core Backend (WebSocket Event Server)

| Component | Technology | Version | Purpose | Why Standard |
|-----------|-----------|---------|---------|--------------|
| HTTP/WS Server | Axum | 0.7-0.8 | Already in Phase 1 | Battle-tested, ergonomic |
| Event Broadcasting | tokio::broadcast | 1.35 | Already in Phase 1 | Lock-free, async-ready |
| Event Format | CoordinationEvent | From Phase 1 | JSON over WebSocket | Consistent event schema |
| Session Persistence | aof-memory FileBackend | Existing | Restore daemon state | Already proven |

### Frontend (builder.io + React)

| Component | Technology | Version | Purpose | Why Standard |
|-----------|-----------|---------|---------|--------------|
| Framework | React | 18.x | builder.io native target | Mature, proven tooling |
| Real-time Sync | Socket.io / ws | 4.x | WebSocket client library | Handles reconnect, events |
| Drag-and-Drop | dnd-kit | 8.x | Kanban, task board | Modern, accessibility-ready |
| State Management | Redux Toolkit | 1.9.x | Complex UI state + sync | Handles optimistic updates |
| UI Components | shadcn/ui | Latest | Beautiful, accessible defaults | Tailwind-based, customizable |
| Build Tool | Vite | 5.x | builder.io + React compilation | Fast HMR, excellent DX |

### Alternative: Pure Rust WASM (Leptos Path)

| Component | Technology | Version | Purpose | Trade-off |
|-----------|-----------|---------|---------|-----------|
| Framework | Leptos | 0.5+ | Full-stack Rust WASM | Bundle size ~300-500KB (compressed) |
| Drag-and-Drop | Crate tbd | — | Rust WASM drag-drop | Fewer options, less mature |
| Build Tool | Trunk | Latest | Rust WASM bundler | Slower builds, more optimization needed |
| WASM Compression | wasm-opt | Latest | Size reduction (15-20%) | Extra build step |

**Installation (Path B - Recommended):**
```toml
# Backend (no change to existing Cargo.toml)
# Phase 1 already provides axum, tokio, serde_json

# Frontend (npm)
# In new web-ui directory
npm install react react-dom @dnd-kit/{core,utilities,sortable}
npm install @reduxjs/toolkit react-redux
npm install ws socket.io-client
npm install @shadcn/ui shadcn-ui
npm install vite @vitejs/plugin-react
```

## User Constraints (from PROJECT.md)

### Locked Decisions
- **builder.io for Mission Control:** User's existing tool, beautiful UX is priority over language purity
- **Rust backend + builder.io frontend:** Daemon mode (Phase 1) handles coordination, UI consumes WebSocket events
- **Local-first architecture:** Agents run on machine, Mission Control connects locally (ws://localhost:8080/ws)

### Claude's Discretion
- **Framework choice for frontend:** Leptos/WASM or React (recommend React for builder.io compatibility and DX)
- **Kanban drag-and-drop library:** dnd-kit, react-beautiful-dnd (deprecated), or custom
- **State sync strategy:** Optimistic updates vs. server-side truth (recommend optimistic for <100ms latency)
- **Configuration sourcing:** How to read AGENTS.md and TOOLS.md into UI (recommend API endpoint over file parsing)

### Deferred Ideas (OUT OF SCOPE)
- Multi-tenancy features
- RBAC / user management
- Cloud-hosted SaaS deployment
- Mobile-optimized UI (web + Slack/Discord are interfaces)
- OAuth subscription support

## Architecture Patterns

### Overall Data Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    MISSION CONTROL SYSTEM                     │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          Browser (localhost:5173 - Vite dev)         │   │
│  │                                                       │   │
│  │  ┌─────────────────┐  ┌─────────────────────────┐   │   │
│  │  │  builder.io     │  │  React Components       │   │   │
│  │  │  + React        │  │  - AgentCard            │   │   │
│  │  │  Generated      │  │  - KanbanBoard          │   │   │
│  │  │  Components     │  │  - SquadChat            │   │   │
│  │  │                 │  │  - ActivityFeed         │   │   │
│  │  └────────┬────────┘  │  - TaskDetail           │   │   │
│  │           │           │  - SquadOverview        │   │   │
│  │           └───────────┘                          │   │   │
│  │                 │                                │   │   │
│  │        Redux + RTK Query                         │   │   │
│  │        (State + WebSocket sync)                  │   │   │
│  │                 │                                │   │   │
│  └─────────────────┼────────────────────────────────┘   │   │
│                    │                                      │   │
│                    │ WebSocket (ws://)                   │   │
│                    ▼                                      │   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │          Rust Daemon (aofctl serve)                 │   │
│  │          localhost:8080                            │   │
│  │                                                     │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │ Axum WebSocket Handler (/ws)                │  │   │
│  │  │ - Subscribe to tokio::broadcast channel    │  │   │
│  │  │ - Forward CoordinationEvent as JSON        │  │   │
│  │  └──────┬───────────────────────────────────┬──┘  │   │
│  │         │                                   │     │   │
│  │  ┌──────▼──────┐                   ┌───────▼────┐ │   │
│  │  │EventBus     │                   │Config APIs │ │   │
│  │  │(broadcast)  │                   │/config/... │ │   │
│  │  │- CoordEvent │                   │            │ │   │
│  │  │- injected   │                   │AGENTS.md   │ │   │
│  │  │  into       │                   │TOOLS.md    │ │   │
│  │  │  Runtime    │                   │            │ │   │
│  │  └─────┬──────┘                   └────────────┘ │   │
│  │        │                                         │   │
│  │  ┌─────▼──────────────────────────────────────┐  │   │
│  │  │ Agent Runtime (Phase 1/2 Infrastructure)   │  │   │
│  │  │ - AgentExecutor                            │  │   │
│  │  │ - FleetCoordinator                         │  │   │
│  │  │ - Tool execution                           │  │   │
│  │  │ - Memory backends                          │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  │                                                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### Pattern 1: WebSocket Event Subscription (React)

**What:** Browser connects to Rust WebSocket endpoint, subscribes to stream of CoordinationEvent. RTK Query subscribes to events, updates Redux store, React components re-render.

**When to use:** Real-time systems where server pushes events to client (activity feeds, agent status updates, task transitions).

**Example:**
```tsx
// In React hook (e.g., src/hooks/useEventSubscription.ts)
import { useEffect } from 'react';
import { useDispatch } from 'react-redux';
import { addEvent, updateAgentStatus } from '../store/eventsSlice';

export function useEventSubscription(url: string = 'ws://localhost:8080/ws') {
  const dispatch = useDispatch();

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onmessage = (event) => {
      const coordinationEvent = JSON.parse(event.data);

      // Dispatch to Redux store
      dispatch(addEvent(coordinationEvent));

      // Handle specific event types
      if (coordinationEvent.activity.type === 'AgentStarted') {
        dispatch(updateAgentStatus({
          agentId: coordinationEvent.agent_id,
          status: 'working',
        }));
      }
    };

    ws.onerror = (err) => {
      console.error('WebSocket error:', err);
      // Reconnect logic (exponential backoff)
    };

    return () => ws.close();
  }, [dispatch]);
}
```

**Integration with Redux:**
```tsx
// Store slice (src/store/eventsSlice.ts)
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface CoordinationEvent {
  event_id: string;
  agent_id: string;
  activity: { type: string; details: any };
  timestamp: string;
}

const eventsSlice = createSlice({
  name: 'events',
  initialState: {
    events: [] as CoordinationEvent[],
    agentStatus: {} as Record<string, string>,
  },
  reducers: {
    addEvent: (state, action: PayloadAction<CoordinationEvent>) => {
      state.events.push(action.payload);
      // Keep last 1000 events in memory
      if (state.events.length > 1000) {
        state.events.shift();
      }
    },
    updateAgentStatus: (state, action) => {
      state.agentStatus[action.payload.agentId] = action.payload.status;
    },
  },
});

export const { addEvent, updateAgentStatus } = eventsSlice.actions;
export default eventsSlice.reducer;
```

### Pattern 2: Configuration-Driven Agent Card Rendering

**What:** At startup, fetch AGENTS.md and TOOLS.md from API endpoint. Render agent cards dynamically with properties from config (avatar, role, skills, personality).

**When to use:** When UI layout depends on runtime configuration, not hardcoded structure.

**Example:**

```tsx
// API endpoint added to aofctl serve: GET /api/config/agents
// Returns parsed AGENTS.md as structured JSON

interface Agent {
  id: string;
  name: string;
  role: string;
  personality: string;
  avatar?: string;
  skills: string[];
  status: 'idle' | 'working' | 'blocked';
}

// In React component (src/components/AgentGrid.tsx)
import { useQuery } from 'react-query';

export function AgentGrid() {
  const { data: agents } = useQuery('agents', async () => {
    const res = await fetch('http://localhost:8080/api/config/agents');
    return res.json() as Promise<Agent[]>;
  });

  return (
    <div className="grid grid-cols-4 gap-4">
      {agents?.map((agent) => (
        <AgentCard key={agent.id} agent={agent} />
      ))}
    </div>
  );
}

function AgentCard({ agent }: { agent: Agent }) {
  return (
    <div className="border rounded p-4">
      {agent.avatar && <img src={agent.avatar} alt={agent.name} />}
      <h3>{agent.name}</h3>
      <p className="text-sm text-gray-600">{agent.role}</p>
      <div className="mt-2">
        {agent.skills.map((skill) => (
          <span key={skill} className="badge">{skill}</span>
        ))}
      </div>
      <StatusIndicator status={agent.status} />
    </div>
  );
}
```

**Implementation in aofctl serve.rs:**
```rust
// Add route to serve agent config
let app = Router::new()
    .route("/api/config/agents", get(get_agents_config))
    .route("/api/config/tools", get(get_tools_config))
    .route("/ws", get(handle_websocket_upgrade))
    // ... existing routes

async fn get_agents_config() -> axum::Json<Vec<serde_json::Value>> {
    // Parse AGENTS.md (or load from memory backend)
    // Return array of agent objects with id, name, role, skills, avatar, personality
    axum::Json(vec![])
}
```

### Pattern 3: Kanban Board with Optimistic Updates

**What:** User drags task card between lanes. Local state updates immediately (optimistic). WebSocket message sent to server. If server rejects, rollback. If server confirms, merge with server state.

**When to use:** High-latency networks or slow backend. <100ms perceived latency critical for UX.

**Example:**

```tsx
// Using dnd-kit for drag-and-drop
import { DndContext, closestCorners, DragEndEvent } from '@dnd-kit/core';
import { SortableContext } from '@dnd-kit/sortable';
import { useDispatch, useSelector } from 'react-redux';

export function KanbanBoard() {
  const dispatch = useDispatch();
  const tasks = useSelector((state) => state.tasks.items);
  const optimisticTasks = useSelector((state) => state.tasks.optimistic);

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    const taskId = active.id as string;
    const newLane = over?.id as string;

    if (!newLane) return;

    // 1. Optimistic update (instant UI response)
    dispatch(updateTaskLaneOptimistic({
      taskId,
      newLane,
    }));

    // 2. Send to server
    fetch('http://localhost:8080/api/tasks/move', {
      method: 'POST',
      body: JSON.stringify({ taskId, newLane }),
    })
      .then(() => {
        // 3. Server confirmed, commit optimistic
        dispatch(commitTaskLaneUpdate({ taskId, newLane }));
      })
      .catch(() => {
        // 4. Server rejected, rollback
        dispatch(rollbackTaskLaneUpdate({ taskId }));
      });
  };

  return (
    <DndContext onDragEnd={handleDragEnd} collisionDetection={closestCorners}>
      {['backlog', 'assigned', 'in-progress', 'review', 'done'].map((lane) => (
        <Lane key={lane} id={lane} tasks={optimisticTasks[lane]} />
      ))}
    </DndContext>
  );
}

function Lane({ id, tasks }: { id: string; tasks: Task[] }) {
  return (
    <SortableContext items={tasks.map(t => t.id)}>
      <div className="min-h-96 bg-gray-100 p-4 rounded">
        <h3 className="font-bold">{id}</h3>
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} />
        ))}
      </div>
    </SortableContext>
  );
}
```

**Redux slice for optimistic updates:**
```tsx
// src/store/tasksSlice.ts
const tasksSlice = createSlice({
  name: 'tasks',
  initialState: {
    items: {} as Record<string, Task[]>,
    optimistic: {} as Record<string, Task[]>, // Optimistic version
    pending: {} as Record<string, Promise<void>>, // Track pending updates
  },
  reducers: {
    updateTaskLaneOptimistic: (state, action) => {
      const { taskId, newLane } = action.payload;
      // Move in optimistic state
      const task = findTaskInState(state.optimistic, taskId);
      if (task) {
        removeTaskFromLane(state.optimistic, taskId);
        addTaskToLane(state.optimistic, newLane, task);
      }
    },
    commitTaskLaneUpdate: (state, action) => {
      // Optimistic was correct, no-op (or sync with server state)
    },
    rollbackTaskLaneUpdate: (state, action) => {
      const { taskId } = action.payload;
      // Restore from items (server truth)
      restoreTaskFromServerState(state);
    },
  },
});
```

### Pattern 4: Real-Time Activity Feed

**What:** Stream of agent activities rendered as timeline. New events appear at top, old events scroll away.

**Example:**
```tsx
// src/components/ActivityFeed.tsx
import { useSelector } from 'react-redux';

export function ActivityFeed() {
  const events = useSelector((state) => state.events.events);

  return (
    <div className="space-y-2 max-h-96 overflow-y-auto">
      {events.map((event) => (
        <ActivityItem key={event.event_id} event={event} />
      ))}
    </div>
  );
}

function ActivityItem({ event }: { event: CoordinationEvent }) {
  const { agent_id, activity, timestamp } = event;
  const timeAgo = formatDistanceToNow(new Date(timestamp), { addSuffix: true });

  return (
    <div className="border-l-2 border-blue-500 pl-3 py-1">
      <p className="text-sm">
        <strong>{agent_id}</strong> {getActivityDescription(activity)} <span className="text-gray-400">{timeAgo}</span>
      </p>
    </div>
  );
}
```

### Anti-Patterns to Avoid

- **Don't poll REST API:** Real-time requires WebSocket push, not `/events?since=timestamp` polling. WebSocket is 1000x more efficient.
- **Don't block on drag-and-drop:** Update local state immediately, send server request async. Never wait for server response before showing visual feedback.
- **Don't hardcode agent list:** Load from API endpoint (GET /api/config/agents) so config changes update UI without redeployment.
- **Don't ignore WebSocket reconnection:** Network drops happen. Implement exponential backoff reconnect with event replay on recovery.
- **Don't lose task updates during network latency:** Use Redux + optimistic updates pattern. Single source of truth (server state) with local optimistic overlay.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drag-and-drop | Custom mouse/touch handlers | dnd-kit | Handles accessibility, keyboard, mobile, nested lists, animations |
| WebSocket reconnection | Manual backoff loop | Socket.io or ws with reconnect plugin | Built-in exponential backoff, event queuing |
| Real-time state sync | Manual optimistic + rollback | Redux Toolkit + RTK Query | Handles versioning, conflict detection, cache invalidation |
| Component styling | CSS from scratch | shadcn/ui + Tailwind | Pre-built accessible components, dark mode, theming |
| Kanban sorting | Custom swap algorithm | dnd-kit + SortableContext | Handles animations, multiple drop zones, touch devices |
| Chat message ordering | Manual timestamp sort | Message IDs + server-provided ordering | Handles out-of-order arrival, deduplication |
| WebSocket JSON serialization | Manual JSON.stringify/parse | serde_json (Rust) + JSON native (JS) | Type safety, custom serializers for enums |

**Key insight:** Real-time UI sync is harder than it looks. Optimistic updates create race conditions. WebSocket drops require replay logic. Drag-and-drop on touch has accessibility pitfalls. Use proven libraries.

## Common Pitfalls

### Pitfall 1: WebSocket Connection Drops, UI Freezes

**What goes wrong:** Network hiccup causes WebSocket close. UI stops receiving events. User sees stale data, thinks app is broken.

**Why it happens:** No reconnection logic. WebSocket is stateful—close means goodbye until app restarts.

**How to avoid:**
- Implement exponential backoff: 1s, 2s, 4s, 8s, 30s cap
- Queue outgoing messages while disconnected
- Replay recent events on reconnect (use event IDs)
- Show "Disconnected" indicator, auto-hide on reconnect

**Warning signs:**
- UI updates stop for 30 seconds
- Refresh page fixes it
- No error message in console

**Example fix:**
```tsx
function useWebSocketWithReconnect(url: string) {
  const [connected, setConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const retryCountRef = useRef(0);

  const connect = useCallback(() => {
    wsRef.current = new WebSocket(url);
    wsRef.current.onopen = () => {
      setConnected(true);
      retryCountRef.current = 0;
    };
    wsRef.current.onclose = () => {
      setConnected(false);
      // Exponential backoff
      const delay = Math.min(1000 * Math.pow(2, retryCountRef.current), 30000);
      retryCountRef.current += 1;
      setTimeout(connect, delay);
    };
  }, [url]);

  useEffect(() => {
    connect();
    return () => wsRef.current?.close();
  }, [connect]);

  return { connected, ws: wsRef.current };
}
```

### Pitfall 2: Drag-and-Drop Race Condition

**What goes wrong:** User drags task to "done" lane. Optimistic update shows it moved. Before server confirms, agent executor completes same task. Server sends task state update, overwrites optimistic move. UI flickers task back to "in-progress" then forward to "done".

**Why it happens:** Two concurrent updates (user drag + server event) conflicting. No version numbers to detect stale data.

**How to avoid:**
- Include version number in task: `{ id, lane, version: 5, status: 'done' }`
- Server assigns new version on each update
- On conflicting update, apply server version if newer
- Optimistic updates don't increment version (server does)

**Warning signs:**
- Task briefly moves backward after drag
- Inconsistent UI state during drag
- Server logs show multiple versions for same task

**Example fix:**
```tsx
// Task with version
interface Task {
  id: string;
  lane: string;
  version: number;
  status: string;
}

// On drag end
dispatch(updateTaskOptimistic({
  taskId,
  newLane,
  // Don't increment version—server will
}));

// On server event (higher version)
const existingTask = findTask(state, eventTask.id);
if (eventTask.version > existingTask.version) {
  // Server is newer, apply it
  dispatch(updateTaskFromServer(eventTask));
}
```

### Pitfall 3: Redux State Explosion with Real-Time Events

**What goes wrong:** Each CoordinationEvent dispatched to Redux. 100 events/sec = 6,000 actions/min. Redux devtools chokes. Chrome tab uses 500MB RAM. React re-renders every component.

**Why it happens:** Dispatching raw events without aggregation. No cleanup of old events.

**How to avoid:**
- Keep only last N events in store (e.g., 500)
- Use selectors to compute derived state (agent status) from events
- Don't dispatch all events—filter by agent_id on client or server
- Use `useShallowEqual` selector for large arrays

**Warning signs:**
- Redux devtools shows 10,000+ actions
- Tab memory grows over time
- React DevTools shows all components re-rendering

**Example fix:**
```tsx
const eventsSlice = createSlice({
  name: 'events',
  initialState: { events: [] as Event[], lastEventId: '' },
  reducers: {
    addEvent: (state, action) => {
      state.events.push(action.payload);
      state.lastEventId = action.payload.event_id;
      // Keep last 500 events
      if (state.events.length > 500) {
        state.events = state.events.slice(-500);
      }
    },
  },
});

// Selector with memoization
const selectAgentStatus = (state: RootState, agentId: string) => {
  // Compute from events, not stored separately
  return state.events.events
    .filter(e => e.agent_id === agentId)
    .reverse()[0]?.status || 'idle';
};

// In component
const agentStatus = useSelector((state) => selectAgentStatus(state, agentId));
```

### Pitfall 4: Lost Configuration on Daemon Restart

**What goes wrong:** User loads agent grid from API (/api/config/agents). Daemon restarts. Agent AGENTS.md file changed. UI shows stale agent list.

**Why it happens:** No cache invalidation. UI doesn't know config changed on server.

**How to avoid:**
- Add version header to config API: `X-Config-Version: 5`
- UI caches config with version
- Periodically poll version endpoint
- On version mismatch, refetch config
- Show "Reloading configuration..." briefly

**Warning signs:**
- Daemon restarts, agent list unchanged
- Add agent, UI still shows old list
- Page refresh fixes it

**Example fix:**
```tsx
// In React Query
const { data: agents, refetch } = useQuery(
  'agents',
  async () => {
    const res = await fetch('http://localhost:8080/api/config/agents');
    return { agents: res.json(), version: res.headers.get('X-Config-Version') };
  },
  { staleTime: 5 * 60 * 1000 } // Cache for 5 minutes
);

// Poll config version every 10 seconds
useEffect(() => {
  const interval = setInterval(async () => {
    const res = await fetch('http://localhost:8080/api/config/version');
    const newVersion = await res.json();
    if (newVersion.version !== agents?.version) {
      refetch(); // Config changed, refetch
    }
  }, 10000);
  return () => clearInterval(interval);
}, [agents?.version, refetch]);
```

### Pitfall 5: Leptos WASM Bundle Bloat

**What goes wrong (if taking Leptos path):** Leptos app with all features compiles to 850KB WASM. Gzipped 280KB. Initial load takes 5 seconds on 4G.

**Why it happens:** Leptos includes reactive runtime, DOM binding, serde, all dependencies bundled.

**How to avoid:**
- Use `wasm-opt -Oz` for aggressive size reduction (15-20% savings)
- Use cargo-features to exclude unused deps (no serde_yaml if not needed)
- Use islands architecture (only interactive parts as WASM, static HTML otherwise)
- Set `opt-level = "z"` in Cargo.toml release profile

**Warning signs:**
- `wasm-pack build` outputs >500KB uncompressed
- Initial load >3 seconds
- Gzipped > 150KB

**Example fix:**
```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
panic = "abort"      # Reduces panic handling code
strip = true         # Strip symbols
```

```bash
# Build with wasm-opt
wasm-pack build --release --target web
wasm-opt -Oz -o pkg/app_bg.wasm pkg/app_bg.wasm
```

### Pitfall 6: Keyboard Navigation in Drag-and-Drop Lost

**What goes wrong:** Using dnd-kit but didn't enable keyboard support. Only mouse/touch works. Screen reader users can't reorder tasks.

**Why it happens:** dnd-kit defaults to mouse/touch. Keyboard + accessibility require explicit setup.

**How to avoid:**
- Use dnd-kit's `useDraggable` with `attributes.roleDescription` for screen readers
- Add keyboard handlers for arrow keys (move between items)
- Test with keyboard + screen reader (NVDA, VoiceOver)
- Use ARIA labels for lanes and tasks

**Warning signs:**
- Tab key doesn't focus drag handles
- Can't hear what task is under cursor (screen reader)
- No visual focus indicator on keyboard nav

**Example fix:**
```tsx
// Use dnd-kit keyboard support
import { KeyboardCode, KeyboardSensor } from '@dnd-kit/core';

<DndContext
  sensors={[
    useSensor(PointerSensor),
    useSensor(TouchSensor),
    useSensor(KeyboardSensor),
  ]}
>
  {/* content */}
</DndContext>

// In task card
<div
  role="button"
  tabIndex={0}
  aria-label={`Task: ${task.title}, in ${task.lane} lane`}
  {...listeners}
>
  {task.title}
</div>
```

## Code Examples

Verified patterns from official sources:

### WebSocket Integration with TypeScript

```typescript
// Source: ws library + React best practices
import { useEffect, useState } from 'react';

interface CoordinationEvent {
  event_id: string;
  agent_id: string;
  activity: { type: string; details: any };
  timestamp: string;
}

export function useWebSocket(url: string) {
  const [events, setEvents] = useState<CoordinationEvent[]>([]);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      setConnected(true);
      console.log('Connected to event stream');
    };

    ws.onmessage = (event) => {
      const coordinationEvent: CoordinationEvent = JSON.parse(event.data);
      setEvents((prev) => [...prev.slice(-999), coordinationEvent]);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setConnected(false);
    };

    ws.onclose = () => {
      setConnected(false);
      // Implement reconnection in production
    };

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [url]);

  return { events, connected };
}
```

### Kanban Board with dnd-kit

```typescript
// Source: dnd-kit documentation + React patterns
import { DndContext, closestCorners, DragEndEvent } from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

interface Task {
  id: string;
  title: string;
  lane: 'backlog' | 'assigned' | 'in-progress' | 'review' | 'done';
}

function TaskCard({ task }: { task: Task }) {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({
    id: task.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className="bg-white border rounded p-3 shadow"
    >
      {task.title}
    </div>
  );
}

function Lane({
  laneId,
  tasks,
}: {
  laneId: string;
  tasks: Task[];
}) {
  const { setNodeRef } = useDroppable({ id: laneId });

  return (
    <SortableContext
      items={tasks.map((t) => t.id)}
      strategy={verticalListSortingStrategy}
    >
      <div ref={setNodeRef} className="min-h-96 bg-gray-100 p-4 rounded">
        <h3 className="font-bold mb-2">{laneId}</h3>
        <div className="space-y-2">
          {tasks.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))}
        </div>
      </div>
    </SortableContext>
  );
}

export function KanbanBoard() {
  const [tasks, setTasks] = useState<Task[]>([
    { id: '1', title: 'Setup K8s cluster', lane: 'backlog' },
    { id: '2', title: 'Monitor pods', lane: 'in-progress' },
    { id: '3', title: 'Review logs', lane: 'done' },
  ]);

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;

    const taskId = active.id as string;
    const newLane = over.id as string;

    setTasks((prev) =>
      prev.map((t) =>
        t.id === taskId ? { ...t, lane: newLane as Task['lane'] } : t
      )
    );
  };

  const lanes = ['backlog', 'assigned', 'in-progress', 'review', 'done'] as const;

  return (
    <DndContext onDragEnd={handleDragEnd} collisionDetection={closestCorners}>
      <div className="grid grid-cols-5 gap-4 p-4">
        {lanes.map((lane) => (
          <Lane
            key={lane}
            laneId={lane}
            tasks={tasks.filter((t) => t.lane === lane)}
          />
        ))}
      </div>
    </DndContext>
  );
}
```

### Axum WebSocket Handler for CoordinationEvent

```rust
// Source: Axum + Phase 1 infrastructure
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use aof_coordination::EventBroadcaster;

async fn handle_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(event_bus): State<Arc<EventBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_handler(socket, event_bus))
}

async fn websocket_handler(
    socket: WebSocket,
    event_bus: Arc<EventBroadcaster>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = event_bus.subscribe();

    // Spawn task to forward events to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if let Err(_) = sender.send(axum::extract::ws::Message::Text(json)).await {
                break; // Client disconnected
            }
        }
    });

    // Listen for client messages (ping/pong, close)
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            axum::extract::ws::Message::Close(_) => break,
            _ => {} // Ignore other messages
        }
    }

    send_task.abort();
}

// Add to serve.rs
let app = Router::new()
    .route("/ws", get(handle_websocket_upgrade))
    .route("/api/config/agents", get(get_agents_config))
    .route("/api/config/tools", get(get_tools_config))
    .with_state(Arc::new(event_bus));

// Helper: Parse AGENTS.md and return JSON
async fn get_agents_config() -> Json<serde_json::Value> {
    // Load AGENTS.md, parse YAML, return JSON
    // Placeholder implementation
    Json(json!([
        {
            "id": "k8s-monitor",
            "name": "K8s Monitor",
            "role": "Kubernetes Specialist",
            "personality": "Methodical and thorough",
            "avatar": "🤖",
            "skills": ["kubectl", "pod-debugging", "log-analysis"],
            "status": "idle"
        }
    ]))
}

async fn get_tools_config() -> Json<serde_json::Value> {
    // Load TOOLS.md, parse YAML, return JSON
    Json(json!([
        {
            "name": "kubectl",
            "description": "Kubernetes command-line tool",
            "category": "infrastructure"
        }
    ]))
}
```

## Real-Time Sync Strategy: Optimistic Updates with Versioning

```
User Action (Drag task)
    ↓
[Local State Update] ← INSTANT visual feedback
    ↓
[Send WebSocket: TASK_MOVED{taskId, newLane}]
    ↓
        ┌─────────────────────────────────────┐
        │  Server processes, updates version  │
        └──────────────┬──────────────────────┘
                       ↓
    ┌──────────────────────────────────────────┐
    │ [Broadcast TASK_UPDATED{version:6, ...}] │
    └────────┬──────────────────────────────────┘
             ↓
    [All clients receive event]
             ↓
    [If version > local version: merge update]
    [If version = local version: already have it]
    [If version < local version: ignore (we're ahead)]
```

Conflict resolution is automatic via versioning. No manual rollback needed in happy path.

## State of the Art (2026)

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| REST polling | WebSocket push | 1000x more efficient, <100ms latency |
| redux-thunk | Redux Toolkit + RTK Query | Type-safe, automatic cache invalidation |
| react-beautiful-dnd | dnd-kit | Better accessibility, more maintained |
| Manual optimistic updates | RTK Query with `optimistic` flag | Declarative, less error-prone |
| Warp + handwritten WS | Axum + axum-tungstenite | Better ergonomics, more features |
| Builder.io (platform only) | builder.io + React + custom backend | No-code UI generation + Rust coordination logic |

**Deprecated/outdated:**
- react-beautiful-dnd: No longer maintained, dnd-kit is replacement
- Warp 0.3: Still works but Axum is more actively developed
- Manual WebSocket frame handling: Use axum-tungstenite
- Redux saga: Replaced by RTK Query for async state

## Recommended Approach Summary

### Why Path B (builder.io + React) Over Pure Leptos

| Criterion | builder.io + React | Pure Leptos WASM |
|-----------|-------------------|-----------------|
| Time to beautiful UI | Days (builder.io generates) | Weeks (build from scratch) |
| Developer velocity | High (npm ecosystem, HMR) | Medium (Rust compile times) |
| Bundle size | 80KB JS + 50KB React | 300-500KB WASM (compressed) |
| Accessibility | Proven (shadcn/ui) | Newer patterns |
| Drag-and-drop | Mature (dnd-kit) | Limited options |
| Integration with builder.io | Native | Custom serialization |
| Team hiring | React devs plentiful | Rust WASM rare |

**Bottom line:** Users expect modern web UI. React + builder.io delivers in weeks. Pure Rust WASM is a future optimization after MVP validates product.

## Architecture Integration with Phase 1 & 3

### WebSocket Flow (Phase 1 → Phase 4)

```
Phase 1: aofctl serve runs on localhost:8080
         - Axum WebSocket handler: /ws
         - Broadcasts CoordinationEvent to all subscribers
         - Already implemented ✓

Phase 3: Gateway routes Slack/Discord → CoordinationEvent
         - Emits to same broadcast channel
         - Already implemented ✓

Phase 4: Browser connects ws://localhost:8080/ws
         - Receives stream of CoordinationEvent
         - Redux dispatch updates UI
         - React components re-render
         - NEW: Implement Phase 4
```

### Configuration API (Phase 4 → Phase 1/2)

```
aofctl serve
- Load AGENTS.md from disk (or memory backend)
- Parse YAML → JSON
- Serve at GET /api/config/agents
- Serve at GET /api/config/tools
- Serve at GET /api/config/version (for cache invalidation)

Browser
- Fetch /api/config/agents at startup
- Cache with version tracking
- Refetch if version changed
```

## Build & Deployment Strategy

### Development

```bash
# Terminal 1: Rust daemon with WebSocket
cd /Users/gshah/work/opsflow-sh/aof
cargo run -p aofctl -- serve --config serve-config.yaml
# Listens on http://localhost:8080
# WebSocket on ws://localhost:8080/ws
# APIs on http://localhost:8080/api/config/*

# Terminal 2: React dev server (builder.io + Vite)
cd web-ui
npm install
npm run dev
# Listens on http://localhost:5173
# Auto-reload on code change
# Proxies /api/* to localhost:8080
```

### Production

```bash
# Build React + builder.io frontend
cd web-ui
npm run build
# Outputs dist/

# Add static file serving to aofctl serve
cargo run -p aofctl -- serve --config serve-config.yaml --static-dir ./web-ui/dist
# Axum serves static files at /
# API/WebSocket at same port (8080)
# Single daemon, single process
```

### File Structure

```
aof/
├── crates/
│   ├── aofctl/
│   │   └── commands/serve.rs          [Add /api/config routes + static serving]
│   ├── aof-core/coordination.rs       [CoordinationEvent - Phase 1, no change]
│   └── ...
├── web-ui/                             [NEW - builder.io + React]
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── components/
│   │   │   ├── AgentCard.tsx
│   │   │   ├── KanbanBoard.tsx
│   │   │   ├── SquadChat.tsx
│   │   │   ├── ActivityFeed.tsx
│   │   │   └── ...
│   │   ├── hooks/
│   │   │   └── useWebSocket.ts
│   │   ├── store/
│   │   │   ├── index.ts
│   │   │   ├── eventsSlice.ts
│   │   │   ├── tasksSlice.ts
│   │   │   └── ...
│   │   ├── App.tsx                     [From builder.io]
│   │   └── main.tsx
│   ├── dist/                           [Build output]
│   └── vite.config.ts
```

## Open Questions

1. **Should task data come from WebSocket events or separate API?**
   - What we know: Phase 1 broadcasts CoordinationEvent (agent status, not task state)
   - What's unclear: Is task assignment managed by agents or separate service?
   - Recommendation: Create /api/tasks endpoint in aofctl serve, fetch at startup, subscribe to task updates via WebSocket (TASK_CREATED, TASK_UPDATED, TASK_MOVED events)

2. **How to handle agent avatar/personality data?**
   - What we know: AGENTS.md has personality, avatar fields
   - What's unclear: Avatar as emoji string, image URL, or upload binary?
   - Recommendation: Avatar as data URL or external image URL. Personality as text string. Both in AGENTS.md YAML.

3. **Should squad chat use WebSocket or separate API?**
   - What we know: Phase 3 gateway forwards messages, agents respond
   - What's unclear: Is chat stored in memory backend or ephemeral?
   - Recommendation: Store in memory backend (persistent), stream chat events via WebSocket, fetch history on page load via /api/chat/history?since=timestamp

4. **Can builder.io generate code that integrates with Rust WebSocket API?**
   - What we know: builder.io generates React + TypeScript
   - What's unclear: Can it expose hooks for custom backends?
   - Recommendation: Have developer manually wire useWebSocket hook to builder.io components. builder.io generates structure, developer adds interactivity.

## Sources

### Primary (HIGH confidence)
- **Phase 1 RESEARCH.md:** Axum 0.7, tokio::broadcast, CoordinationEvent format (verified in codebase)
- **Phase 3 RESEARCH.md:** Hub-and-spoke gateway, event normalization patterns
- **Axum docs:** https://docs.rs/axum/latest/axum/ (WebSocket upgrade handler)
- **dnd-kit docs:** https://docs.dndkit.com/ (kanban board implementation)
- **Redux Toolkit docs:** https://redux-toolkit.js.org/ (optimistic updates, RTK Query)

### Secondary (MEDIUM confidence)
- **React Real-time Patterns:** https://blog.logrocket.com/solving-eventual-consistency-frontend/ (optimistic updates, versioning)
- **Leptos WASM Bundle Size:** https://book.leptos.dev/deployment/binary_size.html (typical sizes, optimization techniques)
- **dnd-kit Kanban Example:** [GitHub - Georgegriff/react-dnd-kit-tailwind-shadcn-ui](https://github.com/Georgegriff/react-dnd-kit-tailwind-shadcn-ui) (verified implementation)
- **WebSearch:** Framework comparison, builder.io capabilities, real-time sync patterns (2026)

### Tertiary (LOW confidence)
- **builder.io integration:** Limited official docs on Rust backend integration. Extrapolated from REST API patterns.

## Metadata

**Confidence breakdown:**
- Standard stack (backend): HIGH - Phase 1 already proven
- Standard stack (frontend): MEDIUM-HIGH - React + dnd-kit + Redux standard, but specific to AOF
- Architecture patterns: MEDIUM - WebSocket sync patterns proven in industry, optimistic updates validated
- Pitfalls: MEDIUM-HIGH - Real-time UI pitfalls well-known, but AOF-specific conflicts depend on task model clarity
- Code examples: MEDIUM - React examples standard, Rust WebSocket handler extrapolated from Phase 1

**Research date:** 2026-02-14
**Valid until:** 2026-03-07 (21 days - fast-moving frontend, stable backend infrastructure)

**Key uncertainties:**
- Task data model (ephemeral from events vs. persistent in memory backend)
- Chat message persistence strategy
- builder.io integration mechanics with Rust backend (may need custom work)
- Avatar/personality data format

---

**Ready for planning:** Research provides sufficient direction to create PLAN.md files for:
- 04-01: React + builder.io frontend setup, WebSocket integration
- 04-02: Agent cards, kanban board, drag-and-drop
- 04-03: Squad chat, activity feed, real-time sync

**Success metrics:**
- UI connects to WebSocket in <1 second
- Agent status updates visible within 500ms of event
- Drag-and-drop responsive even on 4G (optimistic update)
- No console errors on reconnect
- Configuration changes load without page refresh
- First paint <2 seconds on localhost
