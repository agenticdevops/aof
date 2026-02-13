# Phase 1: Event Infrastructure Foundation - Research

**Researched:** 2026-02-11
**Domain:** Real-time event streaming, WebSocket daemon, tokio async runtime, broadcast channels
**Confidence:** HIGH

## Summary

Phase 1 adds a control plane layer to AOF's existing execution runtime, enabling real-time observability of agent activities through an event streaming architecture. The phase extends existing crates (aof-core, aof-runtime) and adds new components (aof-coordination crate, daemon mode in aofctl) without rewriting the 13-crate foundation.

The architecture follows a local-first daemon pattern: agents execute on your machine, WebSocket clients (future Mission Control UI, messaging gateways) connect for real-time event streams. AOF already has the necessary pieces — activity events (aof-core/activity.rs), agent execution (aof-runtime), and a serve command (aofctl/commands/serve.rs) that currently handles webhook-based triggers. Phase 1 extends serve.rs to add WebSocket support and injects event broadcasting into the execution pipeline.

**Primary recommendation:** Use tokio::sync::broadcast for in-memory event streaming (sufficient for single-daemon instance, 1000+ events/sec throughput), Axum 0.8 for HTTP/WebSocket server (modern, excellent ergonomics, integrates with tower ecosystem), and extend existing ActivityEvent types rather than creating new event schemas.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio` | 1.35 (workspace) | Async runtime, broadcast channels | Already in workspace, powers all async |
| `axum` | 0.7 | HTTP server + WebSocket | Modern, well-maintained, excellent ergonomics, tower integration |
| `axum-tungstenite` | 0.2 | WebSocket protocol for Axum | Official WebSocket support for Axum |
| `tower-http` | 0.5 | CORS, static file serving | Standard HTTP middleware for tower/axum |
| `serde_json` | 1.0 (workspace) | JSON serialization for events | Already in workspace, universal JSON support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `chrono` | 0.4 (workspace) | Timestamps in events | Already in workspace, ActivityEvent uses it |
| `uuid` | 1.6 (workspace) | Session IDs, event IDs | Already in workspace, existing in aof-core |
| `tracing` | 0.1 (workspace) | Structured logging | Already in workspace, debugging daemon |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tokio::broadcast | crossbeam-channel | Better for single-producer, but broadcast is multi-subscriber native |
| Axum | warp, actix-web | Warp aging, actix more complex, Axum is modern sweet spot |
| WebSocket | SSE (Server-Sent Events) | SSE simpler but one-way only, need bidirectional for future control plane |

**Installation:**
```toml
# Add to workspace Cargo.toml dependencies
axum = { version = "0.7", features = ["ws"] }
axum-tungstenite = "0.2"
tower-http = { version = "0.5", features = ["fs", "cors"] }
```

## Architecture Patterns

### Recommended Project Structure (New Crate)
```
crates/aof-coordination/
├── src/
│   ├── lib.rs                  # Public API
│   ├── events.rs               # CoordinationEvent enum (extends ActivityEvent)
│   ├── broadcaster.rs          # EventBroadcaster wrapper around tokio::broadcast
│   ├── protocol/               # Coordination protocol types (future)
│   │   ├── mod.rs
│   │   └── heartbeat.rs        # (Phase 7)
│   └── persistence.rs          # Session state (leverage existing Memory backends)
└── Cargo.toml
```

### Pattern 1: Event-Driven Control Plane with Broadcast Channel

**What:** Central event bus using `tokio::sync::broadcast` channel. Producers emit events, multiple consumers subscribe without coupling.

**When to use:** Real-time dashboards, multi-subscriber scenarios, audit trails. Perfect for Phase 1 (single daemon instance, <100 subscribers expected).

**How it works:**
1. Daemon creates broadcast channel on startup
2. Channel sender injected into AgentExecutor, FleetCoordinator
3. Agent lifecycle emits events (started, thinking, tool_call, completed, error)
4. WebSocket handler subscribes to receiver, forwards JSON to connected clients
5. Multiple WebSocket clients each get independent receiver

**Example:**
```rust
// In aofctl serve.rs startup
let (event_tx, _) = tokio::sync::broadcast::channel::<CoordinationEvent>(1000);
let event_bus = Arc::new(EventBroadcaster::new(event_tx));

// Inject into runtime
let runtime = Runtime::with_event_bus(event_bus.clone());

// In AgentExecutor (aof-runtime/executor/agent_executor.rs)
impl AgentExecutor {
    async fn execute(&mut self) {
        // Agent starts
        if let Some(ref bus) = self.event_bus {
            bus.emit(CoordinationEvent::AgentStarted {
                agent_id: self.agent_id.clone(),
                timestamp: Utc::now(),
            });
        }

        // Tool call
        if let Some(ref bus) = self.event_bus {
            bus.emit(CoordinationEvent::ToolCalling {
                agent_id: self.agent_id.clone(),
                tool_name: tool.name.clone(),
                args: serde_json::to_value(&tool.input)?,
            });
        }

        // Completion
        if let Some(ref bus) = self.event_bus {
            bus.emit(CoordinationEvent::AgentCompleted {
                agent_id: self.agent_id.clone(),
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }
    }
}

// In WebSocket handler (aofctl serve.rs)
async fn handle_websocket(ws: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let mut rx = event_bus.subscribe();

    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event)?;
        if ws.send(Message::Text(json)).await.is_err() {
            break; // Client disconnected
        }
    }
}
```

**Scaling limits:**
- Single daemon: 1000+ events/sec, 50+ WebSocket clients
- Buffer size 1000 events sufficient (events ~1KB each)
- Slow consumers handled by tokio::broadcast (lagging subscribers skip events)

### Pattern 2: Extend Existing Event Types, Don't Replace

**What:** AOF already has `ActivityEvent` in aof-core/activity.rs with rich event types (Thinking, ToolExecuting, LlmCall, etc.). Extend this for coordination instead of creating parallel event system.

**When to use:** When existing infrastructure already tracks what you need. Prevents duplication and maintains consistency.

**How:**
```rust
// In aof-core/src/coordination.rs (NEW FILE)
use crate::activity::{ActivityEvent, ActivityType};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Coordination event wraps ActivityEvent with routing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEvent {
    /// Underlying activity event
    pub activity: ActivityEvent,

    /// Agent ID that emitted this event
    pub agent_id: String,

    /// Session ID for grouping related events
    pub session_id: String,

    /// Event ID for deduplication
    pub event_id: String,
}

impl CoordinationEvent {
    pub fn from_activity(activity: ActivityEvent, agent_id: String, session_id: String) -> Self {
        Self {
            activity,
            agent_id,
            session_id,
            event_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
```

**Why this works:**
- Reuses existing 21 activity types (Thinking, Analyzing, LlmCall, ToolExecuting, etc.)
- ActivityEvent already has timestamps, details, tool names
- Just adds routing metadata (agent_id, session_id) for control plane
- WebSocket clients get familiar event structure

### Pattern 3: Daemon Mode Extends Serve Command

**What:** AOF already has `aofctl serve` command (aofctl/commands/serve.rs) that starts long-running HTTP server for webhook triggers (Slack, Discord, GitHub, Jira). Extend this command to add WebSocket server on same port.

**When to use:** When existing command already does 80% of what you need. Avoids new CLI surface area.

**How:**
```rust
// In aofctl/commands/serve.rs (MODIFY EXISTING)

// Current: Axum router with webhook routes
let app = Router::new()
    .route("/webhook/:platform", post(handle_webhook))
    .route("/health", get(health_check));

// Extended: Add WebSocket route
let app = Router::new()
    .route("/webhook/:platform", post(handle_webhook))
    .route("/ws", get(handle_websocket_upgrade))  // NEW
    .route("/health", get(health_check));

// New handler
async fn handle_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_handler(socket, state.event_bus.clone()))
}

async fn websocket_handler(socket: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let (mut sender, _receiver) = socket.split();
    let mut rx = event_bus.subscribe();

    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event).unwrap();
        if sender.send(Message::Text(json)).await.is_err() {
            break; // Client disconnected
        }
    }
}
```

**Benefits:**
- Single process, single port (8080)
- Reuses existing HTTP server infrastructure
- Health check endpoint works for both webhook and WebSocket
- Future: Can add HTTP API routes alongside WebSocket

### Pattern 4: Session Persistence with Existing Memory Backends

**What:** AOF has multiple memory backends (InMemoryBackend, FileBackend, optional Redis/Sled). Use FileBackend for session state persistence instead of building custom storage.

**When to use:** When you need state to survive daemon restarts without complex database setup.

**How:**
```rust
// In aof-coordination/src/persistence.rs (NEW)
use aof_memory::{SimpleMemory, MemoryBackend};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub agent_states: HashMap<String, AgentState>,
    pub task_queue: Vec<TaskInfo>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: String,
    pub status: AgentStatus,
    pub last_activity: DateTime<Utc>,
}

pub struct SessionPersistence {
    memory: SimpleMemory,
}

impl SessionPersistence {
    pub async fn new(persist_path: PathBuf) -> Result<Self> {
        let memory = SimpleMemory::file(persist_path).await?;
        Ok(Self { memory })
    }

    pub async fn save_session(&self, state: &SessionState) -> Result<()> {
        let json = serde_json::to_string(state)?;
        self.memory.set(&state.session_id, json).await?;
        Ok(())
    }

    pub async fn restore_session(&self, session_id: &str) -> Result<Option<SessionState>> {
        if let Some(json) = self.memory.get(session_id).await? {
            let state: SessionState = serde_json::from_str(&json)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}
```

**Why this works:**
- FileBackend uses JSON storage (aof-memory/backend/file.rs)
- Automatic serialization through existing Memory trait
- No new storage abstraction needed
- Can swap to Redis/Sled later without changing interface

### Anti-Patterns to Avoid

- **Don't create parallel event system:** ActivityEvent already exists with 21 types. Extend it, don't replace it.
- **Don't use REST polling:** WebSocket push is the whole point. No `/events?since=timestamp` endpoints.
- **Don't block tokio runtime:** All file I/O must use `tokio::fs`, not `std::fs`. HTTP must use async clients.
- **Don't ignore slow consumers:** tokio::broadcast handles lagging subscribers by skipping events. Monitor receiver lag.
- **Don't build custom persistence:** Use existing Memory backends (FileBackend for Phase 1, Redis for Phase 8 if needed).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WebSocket protocol | Custom WebSocket framing | axum-tungstenite | Handles ping/pong, fragmentation, close handshake, compression |
| Event deduplication | Custom event ID tracking | UUID v4 in CoordinationEvent | Universally unique, collision-resistant |
| Session recovery | Custom checkpoint files | FileBackend (aof-memory) | Atomic writes, JSON serialization, already tested |
| Broadcast buffering | Custom ring buffer | tokio::sync::broadcast | Lock-free, handles lagging subscribers, battle-tested |
| CORS handling | Custom headers | tower-http CORS layer | Handles preflight, credentials, wildcard origins correctly |

**Key insight:** WebSocket protocol has edge cases (concurrent writes, client disconnects mid-frame, slow consumers blocking sender). Axum handles these. Broadcast channels have race conditions (fast producer, slow consumer, buffer overflow). tokio::broadcast handles these. Don't rebuild solved problems.

## Common Pitfalls

### Pitfall 1: Blocking the Tokio Runtime with Sync I/O

**What goes wrong:** Using `std::fs::read_to_string()` or synchronous HTTP clients in async context blocks executor thread, kills concurrency.

**Why it happens:** Muscle memory from sync Rust, forgetting async requires async I/O.

**How to avoid:**
- Use `tokio::fs` for all file operations
- Use `reqwest` (async HTTP) already in workspace
- Use `spawn_blocking` if you must call blocking code

**Warning signs:**
- Latency spikes when agent writes to memory
- WebSocket handler becomes unresponsive during file operations
- `tokio::time::sleep` doesn't wake on time

**Example fix:**
```rust
// ❌ Bad: Blocks tokio runtime
let content = std::fs::read_to_string("agent-state.json")?;

// ✅ Good: Async I/O
let content = tokio::fs::read_to_string("agent-state.json").await?;

// ✅ Good: Blocking operation isolated
let content = tokio::task::spawn_blocking(|| {
    std::fs::read_to_string("agent-state.json")
}).await??;
```

### Pitfall 2: WebSocket Send from Multiple Tasks Without Coordination

**What goes wrong:** Concurrent tasks try to write to same WebSocket. axum WebSocket sender is not `Clone`, so you get "send while another send is in progress" errors or panics.

**Why it happens:** Natural instinct to broadcast event from agent executor task directly to WebSocket, but WebSocket sender must be single-writer.

**How to avoid:**
- Split WebSocket into sender/receiver immediately: `let (mut sender, receiver) = socket.split();`
- Spawn single task that owns sender, receives from channel
- Agent tasks send to channel, sender task serializes writes

**Warning signs:**
- Panics: "WebSocket send called while another send is in progress"
- Events arrive out of order
- WebSocket connection drops randomly

**Example fix:**
```rust
// ❌ Bad: Multiple tasks try to send
let ws = socket; // WebSocket not split
tokio::spawn(async move {
    ws.send(event1).await?; // Error: sender moved
});
tokio::spawn(async move {
    ws.send(event2).await?; // Error: sender already moved
});

// ✅ Good: Single sender task
let (mut sender, _receiver) = socket.split();
let mut rx = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event)?;
        if sender.send(Message::Text(json)).await.is_err() {
            break; // Client disconnected
        }
    }
});
```

### Pitfall 3: Broadcast Channel Buffer Overflow with Slow Consumers

**What goes wrong:** Fast producer (agent emits 100 events/sec), slow consumer (WebSocket client on slow network). Buffer fills, old events discarded, consumer sees gaps.

**Why it happens:** tokio::broadcast behavior — when buffer full, oldest message dropped, `RecvError::Lagged` returned.

**How to avoid:**
- Set buffer size appropriately (1000 for Phase 1)
- Handle `RecvError::Lagged` explicitly (log warning, continue)
- Add client-side filtering (agent_id, event_type) to reduce event rate
- Future: Add backpressure (drop low-priority events like Thinking when lagged)

**Warning signs:**
- WebSocket clients report missing events
- High memory usage in daemon
- `RecvError::Lagged` in logs

**Example fix:**
```rust
// ❌ Bad: Panics on lagged receiver
while let Ok(event) = rx.recv().await {
    send_to_websocket(event).await?;
}

// ✅ Good: Handles lagged consumer
loop {
    match rx.recv().await {
        Ok(event) => {
            if send_to_websocket(event).await.is_err() {
                break; // Client disconnected
            }
        }
        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
            tracing::warn!("WebSocket client lagged, dropped {} events", n);
            // Continue receiving, client will catch up
        }
        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
            break; // Channel closed, daemon shutting down
        }
    }
}
```

### Pitfall 4: Not Handling WebSocket Client Disconnects Gracefully

**What goes wrong:** Client closes WebSocket, but server task keeps trying to send, panics or loops forever consuming CPU.

**Why it happens:** WebSocket `send()` returns error on disconnect, but error handling missing or wrong.

**How to avoid:**
- Check send result: `if sender.send(msg).await.is_err() { break; }`
- Spawn task per WebSocket connection, task exits on disconnect
- Use `tokio::select!` to listen for shutdown signal alongside event stream

**Warning signs:**
- Zombie tasks after client disconnect
- Memory leak (tasks never cleaned up)
- CPU spike from infinite error loop

**Example fix:**
```rust
// ❌ Bad: Ignores send errors
loop {
    let event = rx.recv().await.unwrap();
    let _ = sender.send(Message::Text(json)).await; // Ignores error
}

// ✅ Good: Exits on disconnect
while let Ok(event) = rx.recv().await {
    let json = serde_json::to_string(&event)?;
    if sender.send(Message::Text(json)).await.is_err() {
        tracing::info!("WebSocket client disconnected");
        break;
    }
}
```

### Pitfall 5: Forgetting to Clone Broadcast Sender Before Injecting

**What goes wrong:** Pass broadcast sender directly to AgentExecutor. First agent consumes sender, second agent can't emit events.

**Why it happens:** Broadcast sender is `Clone`, but easy to forget. Passing by value moves it.

**How to avoid:**
- Wrap broadcast sender in Arc: `Arc<EventBroadcaster>` where EventBroadcaster holds sender
- Clone Arc before each injection: `runtime.with_event_bus(event_bus.clone())`
- Use newtype wrapper that forces Arc usage

**Warning signs:**
- First agent emits events fine, second agent silently drops events
- Compile error: "value moved into closure"
- Events stop after first agent completes

**Example fix:**
```rust
// ❌ Bad: Moves sender
let (tx, _rx) = tokio::sync::broadcast::channel(1000);
let executor1 = AgentExecutor::with_event_sender(tx); // tx moved
let executor2 = AgentExecutor::with_event_sender(tx); // Error: tx moved

// ✅ Good: Arc wrapper
pub struct EventBroadcaster {
    tx: tokio::sync::broadcast::Sender<CoordinationEvent>,
}

impl EventBroadcaster {
    pub fn new(tx: tokio::sync::broadcast::Sender<CoordinationEvent>) -> Self {
        Self { tx }
    }

    pub fn emit(&self, event: CoordinationEvent) {
        let _ = self.tx.send(event); // Ignoring send errors is OK (no subscribers)
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CoordinationEvent> {
        self.tx.subscribe()
    }
}

let (tx, _) = tokio::sync::broadcast::channel(1000);
let event_bus = Arc::new(EventBroadcaster::new(tx));

// Clone Arc for each use
let executor1 = AgentExecutor::with_event_bus(event_bus.clone());
let executor2 = AgentExecutor::with_event_bus(event_bus.clone());
```

## Code Examples

Verified patterns from existing AOF codebase and official Axum docs:

### WebSocket Upgrade Handler (Axum)
```rust
// Source: Axum docs + aofctl/commands/serve.rs pattern
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::get,
    Router,
};

async fn handle_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_handler(socket, state.event_bus.clone()))
}

async fn websocket_handler(socket: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = event_bus.subscribe();

    // Spawn task to forward events to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Listen for client messages (ping/pong, close)
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Close(_) => break,
            _ => {} // Ignore other messages for now
        }
    }

    send_task.abort(); // Clean up sender task
}
```

### Activity Event Emission (Existing Pattern)
```rust
// Source: aof-core/activity.rs + aof-runtime/executor/agent_executor.rs

// In AgentExecutor::execute() (MODIFY EXISTING)
use aof_core::{ActivityEvent, ActivityType};

// Existing pattern: TUI activity logger
if let Some(ref logger) = self.activity_logger {
    logger.log(ActivityEvent::thinking("Processing user request"));
}

// New pattern: Coordination event bus (ADD THIS)
if let Some(ref event_bus) = self.event_bus {
    let activity = ActivityEvent::thinking("Processing user request");
    let coord_event = CoordinationEvent::from_activity(
        activity,
        self.agent_id.clone(),
        self.session_id.clone(),
    );
    event_bus.emit(coord_event);
}
```

### Session Persistence (FileBackend Pattern)
```rust
// Source: aof-memory/backend/file.rs
use aof_memory::SimpleMemory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DaemonSession {
    session_id: String,
    started_at: DateTime<Utc>,
    agent_states: HashMap<String, String>,
}

// Initialize persistence
let session_store = SimpleMemory::file("./aof-session.json").await?;

// Save session state
let session = DaemonSession { /* ... */ };
let json = serde_json::to_string(&session)?;
session_store.set("current", json).await?;

// Restore session state on daemon restart
if let Some(json) = session_store.get("current").await? {
    let session: DaemonSession = serde_json::from_str(&json)?;
    println!("Restored session: {}", session.session_id);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Warp 0.3 | Axum 0.7 | 2023 | Axum superseded Warp, better ergonomics, active maintenance |
| Separate WebSocket crate | Axum built-in | 2022 | axum-tungstenite integrates seamlessly with Axum routing |
| Manual CORS headers | tower-http CORS layer | 2021 | Handles preflight correctly, configurable |
| mpsc channels | broadcast channels | Always available | broadcast native for pub/sub, mpsc for single consumer |

**Deprecated/outdated:**
- Warp: Still works but less actively maintained, Axum is the modern choice
- Manual WebSocket frame handling: Use axum-tungstenite, handles protocol correctly
- Custom session storage: Use existing Memory backends (FileBackend sufficient for Phase 1)

## Existing Codebase Context

### What Already Exists
- **ActivityEvent (aof-core/activity.rs):** Complete event system with 21 types (Thinking, Analyzing, LlmCall, ToolExecuting, ToolComplete, etc.)
- **ActivityLogger:** Channel-based logger used in TUI mode (std::sync::mpsc sender)
- **aofctl serve:** Long-running daemon (serve.rs) that handles webhook triggers (Slack, Discord, GitHub, Jira)
- **Memory backends:** InMemoryBackend, FileBackend, optional Redis/Sled (aof-memory crate)
- **AgentExecutor:** Core execution engine (aof-runtime/executor/agent_executor.rs) with activity logging
- **Tokio runtime:** Already used throughout workspace (version 1.35)

### What Needs Extension
- **aof-core:** Add CoordinationEvent type that wraps ActivityEvent with routing metadata (agent_id, session_id, event_id)
- **aof-runtime AgentExecutor:** Inject optional EventBroadcaster, emit coordination events alongside existing activity logging
- **aofctl serve command:** Add WebSocket route (`/ws`) to existing HTTP server, create event broadcaster on startup
- **New aof-coordination crate:** EventBroadcaster wrapper, session persistence, protocol types (Phase 7)

### Integration Points
1. **Event emission in AgentExecutor:**
   - Existing: `self.activity_logger.log(ActivityEvent)` sends to TUI
   - New: `self.event_bus.emit(CoordinationEvent)` broadcasts to WebSocket clients
   - Both can coexist (TUI and daemon modes)

2. **Daemon startup in serve.rs:**
   - Existing: Creates TriggerHandler, registers platform webhooks, starts Axum server
   - New: Creates EventBroadcaster, injects into Runtime, adds `/ws` route

3. **Session persistence:**
   - Existing: Runtime has no session concept
   - New: Store session state (agent IDs, task queue) in FileBackend, restore on daemon restart

## Open Questions

1. **Event filtering at server or client?**
   - What we know: Phase 1 has no UI, filtering not needed yet
   - What's unclear: When UI added (Phase 4), should server filter by agent_id or client?
   - Recommendation: Client-side filtering in Phase 4. Server broadcasts all events, UI filters locally. Simpler server, more flexible client.

2. **Session ID generation strategy?**
   - What we know: Need unique ID for session grouping
   - What's unclear: Should session ID be daemon-lifetime (1 per restart) or time-based (1 per day)?
   - Recommendation: Daemon-lifetime for Phase 1 (UUID v4 on startup). Time-based sessions defer to Phase 4 when UI adds session management.

3. **How to validate event subscription is working?**
   - What we know: Need to test WebSocket connection and event flow
   - What's unclear: Build test client or use existing tool?
   - Recommendation: Use `websocat` CLI tool for testing (simple, no code needed). Create test: start daemon, run agent, verify events appear in websocat.

## Sources

### Primary (HIGH confidence)
- **aof-core/activity.rs:** Existing ActivityEvent implementation with 21 types
- **aof-runtime/executor/agent_executor.rs:** Existing agent execution with activity logging
- **aofctl/commands/serve.rs:** Existing daemon command with webhook handling
- **aof-memory/backend/:** Existing memory backends (InMemoryBackend, FileBackend)
- **Tokio docs:** https://tokio.rs/tokio/tutorial/channels (broadcast channel documentation)
- **Axum docs:** https://docs.rs/axum/latest/axum/ (WebSocket upgrade handler)

### Secondary (MEDIUM confidence)
- **Axum WebSocket example:** https://github.com/tokio-rs/axum/tree/main/examples/websockets (official example)
- **tokio broadcast performance:** https://tokio.rs/tokio/tutorial/channels#broadcast-channel (capacity recommendations)

### Tertiary (LOW confidence)
- None (all findings verified against official sources)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in workspace or official Axum ecosystem
- Architecture: HIGH - Extends existing patterns (ActivityEvent, serve command, Memory backends)
- Pitfalls: HIGH - Tokio broadcast and WebSocket pitfalls well-documented, verified against official docs

**Research date:** 2026-02-11
**Valid until:** 2026-03-11 (30 days - stable ecosystem)

---

**Ready for planning:** Research complete. Planner can create PLAN.md files with confidence in stack choices and architecture patterns.
