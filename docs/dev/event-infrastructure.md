# Event Infrastructure - Internal Developer Documentation

## Overview

The event infrastructure enables **real-time observability** of agent activities through a broadcast channel and WebSocket streaming architecture. Agents executing within AOF emit lifecycle events (started, thinking, tool execution, completed, errors) that are distributed to multiple subscribers (WebSocket clients, monitoring systems, Mission Control UI) simultaneously.

**Key capabilities:**
- Multi-subscriber event distribution via tokio::broadcast
- WebSocket streaming at `/ws` endpoint
- Session state persistence across daemon restarts
- Zero-overhead when disabled (opt-in via builder pattern)
- 8 lifecycle event points cover all observable agent state transitions

## Crate Map

The event infrastructure spans four crates with clear separation of concerns:

```
aof-core (foundation types)
  ├─ coordination.rs
  │  ├─ CoordinationEvent (event envelope with routing metadata)
  │  ├─ AgentIntroduction (persona announcement data)
  │  ├─ SessionState (serializable session snapshot)
  │  ├─ AgentState (individual agent status)
  │  └─ TaskInfo (task coordination queue)
  │
  ↓
aof-coordination (event bus + persistence)
  ├─ broadcaster.rs (EventBroadcaster - tokio::broadcast wrapper)
  ├─ persistence.rs (SessionPersistence - FileBackend wrapper)
  └─ events.rs (convenience constructor re-exports)
       ↓                           ↓
aof-runtime                  aof-triggers
(agent execution)            (WebSocket server)
  ├─ AgentExecutor             ├─ TriggerServer
  │  ├─ with_event_bus()       │  └─ TriggerServerConfig
  │  └─ emit_event()           │     └─ event_bus: Option<Arc<EventBroadcaster>>
  └─ 8 lifecycle points        └─ WebSocket /ws route
                                  ├─ handle_websocket_upgrade()
                                  └─ websocket_handler()
       ↓
aof-personas (persona events)
  └─ events.rs
     ├─ build_introduction_event() (single agent)
     └─ build_introduction_event_batch() (all agents)
       ↓                           ↓
aofctl serve (orchestration)
  ├─ Create EventBroadcaster (1000 buffer)
  ├─ Create SessionPersistence (data_dir/aof/sessions)
  ├─ Generate session_id (UUID v4)
  ├─ Wire event_bus to TriggerServerConfig
  └─ Save session on shutdown
```

## Key Types

### CoordinationEvent

**Location:** `aof-core/src/coordination.rs`

Event envelope that wraps ActivityEvent with routing metadata for multi-agent coordination.

**Fields:**
- `activity: ActivityEvent` - The underlying activity (what happened)
- `agent_id: String` - Agent that emitted this event (for filtering/routing)
- `session_id: String` - Session grouping (UUID v4, generated once per daemon lifetime)
- `event_id: String` - Unique event ID (UUID v4, for deduplication across subscribers)
- `timestamp: DateTime<Utc>` - When coordination event was created

**Convenience constructors:**
```rust
CoordinationEvent::agent_started(agent_id, session_id)
CoordinationEvent::agent_completed(agent_id, session_id, duration_ms)
CoordinationEvent::tool_executing(agent_id, session_id, tool_name, args)
CoordinationEvent::thinking(agent_id, session_id, message)
CoordinationEvent::error(agent_id, session_id, message)
```

**Serialization:** Implements `Serialize` + `Deserialize` for JSON over WebSocket.

### EventBroadcaster

**Location:** `aof-coordination/src/broadcaster.rs`

Wrapper around `tokio::sync::broadcast::Sender<CoordinationEvent>` that provides pub/sub event distribution.

**API:**
```rust
// Create with capacity (default: 1000 events)
let broadcaster = EventBroadcaster::new(1000);

// Emit event to all subscribers (ignores errors if no subscribers)
broadcaster.emit(event);

// Subscribe to events (returns independent receiver)
let mut receiver = broadcaster.subscribe();

// Health check
let count = broadcaster.subscriber_count();
```

**Behavior:**
- **Clone-able:** Multiple emitters can share same broadcast channel
- **Best-effort delivery:** Ignores send errors when no subscribers active
- **Lagging handling:** Subscribers that fall behind skip old events (RecvError::Lagged)
- **Thread-safe:** Lock-free tokio::broadcast implementation

### SessionPersistence

**Location:** `aof-coordination/src/persistence.rs`

Wrapper around `aof_memory::SimpleMemory` with FileBackend for session state storage.

**API:**
```rust
// Create persistence manager (stores at persist_dir/session-state.json)
let persistence = SessionPersistence::new(persist_dir).await?;

// Save session state (serialized to JSON, keyed by session_id)
persistence.save_session(&state).await?;

// Restore session by ID
let state = persistence.restore_session(session_id).await?;

// List all session IDs
let sessions = persistence.list_sessions().await?;

// Delete session
persistence.delete_session(session_id).await?;

// Clear all sessions
persistence.clear_all().await?;
```

**Storage location:** `$DATA_DIR/aof/sessions/session-state.json`
- macOS: `~/Library/Application Support/aof/sessions/`
- Linux: `~/.local/share/aof/sessions/`
- Windows: `%APPDATA%/aof/sessions/`

### SessionState

**Location:** `aof-core/src/coordination.rs`

Serializable snapshot of coordination session state.

**Fields:**
- `session_id: String` - Session identifier
- `agent_states: HashMap<String, AgentState>` - Agent states keyed by agent_id
- `task_queue: Vec<TaskInfo>` - Pending tasks
- `created_at: DateTime<Utc>` - Session creation time
- `last_updated: DateTime<Utc>` - Last state update time

**Methods:** `new()`, `touch()`, `update_agent()`, `add_task()`, `remove_task()`

### AgentState

**Location:** `aof-core/src/coordination.rs`

Individual agent status for session tracking.

**Fields:**
- `agent_id: String` - Agent identifier
- `status: AgentStatus` - Current agent status (Idle, Running, Completed, Error, Disconnected)
- `last_activity: DateTime<Utc>` - Last activity timestamp
- `current_task: Option<String>` - Current task description

### TaskInfo

**Location:** `aof-core/src/coordination.rs`

Task coordination metadata.

**Fields:**
- `task_id: String` - Unique task identifier
- `description: String` - Task description
- `assigned_agent: Option<String>` - Agent assigned to task
- `status: TaskStatus` - Current status (Pending, InProgress, Completed, Failed, Cancelled)
- `created_at: DateTime<Utc>` - Task creation time

## Data Flow

Step-by-step flow from agent execution to WebSocket client:

### 1. Daemon Startup (aofctl serve)
```rust
// Create event broadcaster with 1000-event buffer
let event_bus = Arc::new(EventBroadcaster::new(1000));

// Create session persistence
let session_persistence = SessionPersistence::new(
    data_dir.join("aof/sessions")
).await?;

// Generate session ID (unique per daemon lifetime)
let session_id = uuid::Uuid::new_v4().to_string();

// Pass event_bus to TriggerServerConfig
let server_config = TriggerServerConfig {
    event_bus: Some(event_bus.clone()),
    // ...
};
```

### 2. AgentExecutor Creation
```rust
// Create executor with event bus (opt-in via builder)
let executor = AgentExecutor::new(config, model, tool_executor, memory)
    .with_event_bus(event_bus.clone(), session_id.clone());
```

### 3. Agent Execution
```rust
// AgentExecutor emits events at 8 lifecycle points
self.emit_event(ActivityEvent::started(&self.config.name));
// -> Wraps in CoordinationEvent with agent_id, session_id, event_id
// -> Calls event_bus.emit(coord_event)
// -> tokio::broadcast sends to all subscribers
```

### 4. WebSocket Handler Subscription
```rust
// Client connects to ws://localhost:8080/ws
// Handler subscribes to event bus
let mut receiver = event_bus.subscribe();

// Spawn task to forward events
tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        let json = serde_json::to_string(&event)?;
        sender.send(Message::Text(json)).await?;
    }
});
```

### 5. Multi-Client Distribution
- Each WebSocket client calls `event_bus.subscribe()` → gets independent receiver
- tokio::broadcast clones event to all receivers (zero-copy Arc internally)
- Receivers process at their own pace (lagging handled gracefully)

### 6. Session Persistence on Shutdown
```rust
// Ctrl+C handler
let final_state = SessionState {
    session_id: session_id.clone(),
    agent_states: HashMap::new(), // Phase 1: empty, Phase 2+: populated
    task_queue: Vec::new(),
    created_at: start_time,
    last_updated: Utc::now(),
};

session_persistence.save_session(&final_state).await?;
```

## Event Lifecycle Points

AgentExecutor emits events at 8 specific points in `execute_streaming()`:

| Point | ActivityEvent Type | When Emitted | Example Message |
|-------|-------------------|--------------|-----------------|
| **1. Agent Start** | `Started` | Beginning of execution | "Starting execution for agent: k8s-monitor" |
| **2. Iteration Start** | `Info` | Each iteration of agentic loop | "Iteration 1/5" |
| **3. LLM Call** | `Info` | Before `model.generate_stream()` | "Calling model for iteration 1" |
| **4. Tool Executing** | `ToolExecuting` | Per tool_call before execution | "Executing tool: kubectl" |
| **5. Tool Complete** | `ToolComplete` | Per successful tool result | "Tool completed: kubectl (234ms)" |
| **6. Tool Failed** | `ToolFailed` | Per failed tool result | "Tool failed: kubectl - connection timeout" |
| **7. Agent Complete** | `Completed` | On EndTurn/MaxTokens/StopSequence | "Execution completed in 5230ms" |
| **8. Agent Error** | `Error` | On max iterations, model errors, stream errors | "Exceeded max iterations (5)" |

**Implementation locations:**
- `execute_streaming()` in `aof-runtime/src/executor/agent_executor.rs` (lines 192, 221, 235, tool loop, completion)
- `execute()` (non-streaming) has parallel implementation at same lifecycle points

## Session Persistence

### Session ID Generation
- Generated on daemon startup: `uuid::Uuid::new_v4().to_string()`
- Unique per daemon lifetime (new ID on each restart)
- Included in every CoordinationEvent for grouping

### State Saved on Shutdown
```rust
SessionState {
    session_id: "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
    agent_states: HashMap::new(), // Phase 1: empty, Phase 2+: populated during execution
    task_queue: Vec::new(),       // Phase 1: empty, Phase 2+: populated during execution
    created_at: "2026-02-11T10:00:00Z",
    last_updated: "2026-02-11T10:30:00Z",
}
```

### Restore on Next Startup
```rust
// Phase 1: Just list previous sessions for debugging
let sessions = session_persistence.list_sessions().await?;
println!("Previous sessions: {} found", sessions.len());

// Phase 2+: Restore session state, resume agents
if let Some(previous_state) = session_persistence.restore_session(&last_session_id).await? {
    // Resume agents from agent_states
    // Re-queue tasks from task_queue
}
```

### File Format
Human-readable JSON stored at `data_dir/aof/sessions/session-state.json`:
```json
{
  "session-id": {
    "session_id": "uuid",
    "agent_states": {},
    "task_queue": [],
    "created_at": "2026-02-11T10:00:00Z",
    "last_updated": "2026-02-11T10:30:00Z"
  }
}
```

## Error Handling

### Broadcast Buffer Overflow
**Problem:** Slow subscribers can't keep up, broadcast buffer fills (1000 events).

**Mitigation:**
- `receiver.recv()` returns `RecvError::Lagged(dropped_count)`
- WebSocket handler logs warning: `"Client lagged, dropped {} events", dropped_count`
- Continues sending (client eventually catches up)
- Does NOT disconnect client (harsh penalty avoided)

**Code:**
```rust
match receiver.recv().await {
    Ok(event) => { /* send to client */ },
    Err(RecvError::Lagged(dropped)) => {
        warn!("WebSocket client lagged, dropped {} events", dropped);
        continue; // Keep sending
    },
    Err(RecvError::Closed) => break, // Channel closed, shutdown
}
```

### WebSocket Disconnect
**Problem:** Client closes connection, send task still running.

**Mitigation:**
- `sender.send()` returns error when client disconnected
- Send task breaks loop on error
- Parent task aborts send task: `send_task.abort()`
- Receiver dropped, tokio::broadcast decrements subscriber count

**Code:**
```rust
let send_task = tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        if sender.send(Message::Text(json)).await.is_err() {
            break; // Client disconnected
        }
    }
});

// On disconnect or close frame
send_task.abort();
```

### No Subscribers
**Problem:** Agent emits event, but no WebSocket clients connected.

**Mitigation:**
- `broadcaster.emit()` calls `sender.send(event)`
- Returns `Err` when no receivers active
- EventBroadcaster ignores error, logs debug message
- Valid operational state (daemon running before clients connect)

**Code:**
```rust
match self.sender.send(event) {
    Ok(receiver_count) => {
        debug!("Event broadcasted to {} subscribers", receiver_count);
    }
    Err(_) => {
        debug!("Event emitted with no active subscribers"); // OK
    }
}
```

### Blocking I/O
**Problem:** Session persistence uses file I/O, could block async runtime.

**Mitigation:**
- All file operations use `tokio::fs` (async I/O)
- `SimpleMemory::file()` with FileBackend uses async storage backend
- No blocking `std::fs` calls in async context

## Testing

### Unit Tests

**aof-core coordination module (14 tests):**
```bash
cargo test -p aof-core coordination
```
- Event creation, unique ID generation, serialization
- SessionState management (add/remove agents, add/remove tasks)
- Convenience constructors (agent_started, agent_completed, tool_executing, thinking, error)

**aof-coordination broadcaster (6 tests):**
```bash
cargo test -p aof-coordination broadcaster
```
- Single producer/single consumer
- Single producer/multiple consumers (same event delivered to all)
- Emit with no subscribers (no panic)
- Subscriber count tracking
- Broadcaster clone behavior

**aof-coordination persistence (5 tests):**
```bash
cargo test -p aof-coordination persistence
```
- Save/restore session state
- Restore nonexistent session (returns None)
- List sessions
- Delete session
- Persistence across instances (survives process restart)

**aof-runtime executor (26 tests):**
```bash
cargo test -p aof-runtime
```
- AgentExecutor with `event_bus=None` (default, no breaking changes)
- Event emission opt-in via `with_event_bus()`

### Manual Testing

**Start daemon:**
```bash
cargo build --release
./target/release/aofctl serve --port 8080
```

**Connect WebSocket client (websocat):**
```bash
websocat ws://localhost:8080/ws
```

**Run agent via trigger:**
```bash
# Trigger agent execution (HTTP POST to /webhook/:platform)
# Or run agent directly via aofctl
```

**Verify events stream to websocat output as JSON.**

### Multi-Client Testing

**Open two terminals:**
```bash
# Terminal 1
websocat ws://localhost:8080/ws

# Terminal 2
websocat ws://localhost:8080/ws
```

**Run agent, verify both terminals receive identical events** (same event_id, same timestamp).

### Session Persistence Testing

**Save session:**
```bash
# Start daemon
./target/release/aofctl serve

# Run agent (generates events)

# Ctrl+C to shutdown (saves session state)
```

**Verify file created:**
```bash
# macOS
cat ~/Library/Application\ Support/aof/sessions/session-state.json

# Linux
cat ~/.local/share/aof/sessions/session-state.json
```

**Restart daemon, verify session restored:**
```bash
# Check logs for "Previous sessions: N found"
./target/release/aofctl serve
```

## Future Work

### Phase 2: Real Ops Capabilities
- **Populate agent_states during execution:** Update AgentState on agent start/complete/error
- **Populate task_queue:** Track tasks assigned to agents, update status
- **Resume agents on restore:** Read SessionState.agent_states, resume Running agents

### Phase 3: Messaging Gateway
- **Event filtering:** Subscribe to specific agent_ids or event types
- **Bidirectional commands:** WebSocket clients send commands to agents (pause, cancel, priority)

### Phase 4: Mission Control UI
- **WASM UI subscribes to /ws:** Real-time agent activity visualization
- **Agent cards:** Show AgentState (status, current_task, last_activity)
- **Task queue:** Show TaskInfo list with status indicators

### Phase 7: Coordination Protocols
- **Heartbeat protocol:** Agents send periodic heartbeat events
- **Agent discovery:** Broadcast agent capabilities on startup
- **Task delegation:** Agents communicate via CoordinationEvent protocol messages

### Phase 5: Agent Introduction Events

Introduction events are emitted at daemon startup for each configured agent. They flow through the same broadcast channel as regular coordination events.

**Event structure:**

```rust
// CoordinationEvent with introduction data
CoordinationEvent {
    activity: ActivityEvent::info("Kubernetes Monitor introduced: I'm..."),
    agent_id: "k8s-monitor",
    session_id: "uuid",
    event_id: "uuid",
    timestamp: Utc::now(),
    introduction: Some(AgentIntroduction {
        agent_id: "k8s-monitor",
        agent_name: "Kubernetes Monitor",
        role: "Infrastructure Specialist",
        avatar: "\u{1F916}",
        intro_message: "I'm Kubernetes Monitor...",
        personality_summary: "A methodical specialist...",
        skills: vec!["kubectl", "pod-debugging"],
    }),
}
```

**Building introduction events:**

```rust
use aof_personas::events::{build_introduction_event, build_introduction_event_batch};

// Single agent
let event = build_introduction_event(&agent, Some(&soul), &session_id);

// All agents at startup
let events = build_introduction_event_batch(&agents, &souls, &session_id);
for event in events {
    event_bus.emit(event);
}
```

**Squad overrides:** Optional `workspace/squads.yaml` provides per-squad introduction overrides. When present, `intro_override` replaces `default_intro` from SOUL.md.

**Gateway integration:** `GatewayHub::handle_introduction_event()` routes introductions to messaging platforms (Slack, Discord, etc.) via broadcast.

### Phase 8: Production Readiness
- **Multi-daemon coordination:** Event bus spans multiple daemons (NATS, Redis Pub/Sub)
- **Event persistence:** Store events to database for replay/audit
- **Metrics:** Track event throughput, subscriber lag, buffer overflow rates
- **Authentication:** WebSocket clients authenticate via API key or JWT
- **TLS support:** wss:// for encrypted WebSocket connections
- **Origin checking:** CORS for WebSocket upgrade requests
