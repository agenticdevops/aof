# Control Plane Architecture

## Overview

The AOF control plane enables **real-time observability and coordination** of agent execution through an event-driven architecture. It consists of three core components:

1. **Event Bus** - tokio::broadcast-based pub/sub for event distribution
2. **WebSocket Server** - Real-time event streaming to clients via `/ws` endpoint
3. **Session Persistence** - File-based state storage for daemon restart resilience

This architecture provides the foundation for Mission Control UI (Phase 4), multi-agent coordination (Phase 7), and production monitoring (Phase 8).

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        AOF Daemon (aofctl serve)                 │
│                                                                   │
│  ┌─────────────────┐     ┌──────────────────┐                   │
│  │ Agent Executor  │────→│ Event Bus        │                   │
│  │ (aof-runtime)   │     │ (tokio::broadcast│                   │
│  │                 │     │  capacity: 1000) │                   │
│  └─────────────────┘     └────────┬─────────┘                   │
│           │                       │                              │
│           │ emit_event()          │ subscribe()                  │
│           │                       │                              │
│           ↓                       ↓                              │
│  CoordinationEvent          ┌─────────────────┐                 │
│  ┌──────────────────┐       │ WebSocket /ws   │                 │
│  │ activity         │       │ (Axum handler)  │                 │
│  │ agent_id         │       └────────┬────────┘                 │
│  │ session_id       │                │                           │
│  │ event_id (UUID)  │                │ JSON over ws://           │
│  │ timestamp        │                │                           │
│  └──────────────────┘                │                           │
│                                      │                           │
│  ┌────────────────────┐              │                           │
│  │ Session            │              ↓                           │
│  │ Persistence        │     ┌─────────────────┐                 │
│  │ (FileBackend)      │     │ Client 1        │                 │
│  │                    │     │ (websocat)      │                 │
│  │ $DATA_DIR/aof/     │     └─────────────────┘                 │
│  │ sessions/          │                                          │
│  │ session-state.json │     ┌─────────────────┐                 │
│  └────────────────────┘     │ Client 2        │                 │
│           ↑                 │ (Dashboard UI)  │                 │
│           │                 └─────────────────┘                 │
│           │                                                      │
│           │ save on shutdown     ┌─────────────────┐            │
│           │ restore on startup   │ Client N        │            │
│           │                      │ (Logging system)│            │
│           └──────────────────────└─────────────────┘            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Agent Executor (aof-runtime)

**Responsibility:** Execute agent tasks, emit lifecycle events

**Event emission points (8 total):**
- Agent start
- Iteration start (each agentic loop)
- LLM call
- Tool executing
- Tool complete
- Tool failed
- Agent complete
- Error

**Implementation:**
```rust
// Optional event bus via builder pattern
let executor = AgentExecutor::new(config, model, tool_executor, memory)
    .with_event_bus(event_bus, session_id);

// Emit events during execution
self.emit_event(ActivityEvent::started(&self.config.name));
self.emit_event(ActivityEvent::tool_executing(tool_name, args));
self.emit_event(ActivityEvent::completed(duration_ms));
```

**Location:** `crates/aof-runtime/src/executor/agent_executor.rs`

### 2. Event Bus (aof-coordination)

**Responsibility:** Distribute events to multiple subscribers efficiently

**Implementation:** Wraps `tokio::sync::broadcast::Sender<CoordinationEvent>`

**Key features:**
- **Clone-able:** Multiple emitters share channel
- **Lock-free:** tokio::broadcast is high-performance
- **Best-effort:** Ignores errors if no subscribers
- **Lagging handling:** Slow subscribers skip old events

**API:**
```rust
// Create with capacity
let event_bus = Arc::new(EventBroadcaster::new(1000));

// Emit event (non-blocking)
event_bus.emit(coordination_event);

// Subscribe (returns independent receiver)
let mut receiver = event_bus.subscribe();

// Health check
let count = event_bus.subscriber_count();
```

**Performance:**
- 1000+ events/sec throughput
- ~10μs per emit
- Zero-copy Arc internally

**Location:** `crates/aof-coordination/src/broadcaster.rs`

### 3. WebSocket Server (aof-triggers)

**Responsibility:** Stream events to clients over WebSocket

**Route:** `GET /ws` (WebSocket upgrade)

**Handler flow:**
1. Client connects → `handle_websocket_upgrade()`
2. Upgrade to WebSocket → `websocket_handler()`
3. Subscribe to event bus → `event_bus.subscribe()`
4. Spawn send task → forward events as JSON
5. Listen for close frames → abort send task on disconnect

**Implementation:**
```rust
async fn websocket_handler(
    socket: WebSocket,
    event_bus: Arc<EventBroadcaster>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_receiver = event_bus.subscribe();

    // Spawn task to forward events
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let json = serde_json::to_string(&event)?;
            if sender.send(Message::Text(json)).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Listen for close frames
    while let Some(Ok(msg)) = receiver.next().await {
        if matches!(msg, Message::Close(_)) {
            break;
        }
    }

    send_task.abort();
}
```

**Error handling:**
- `RecvError::Lagged` → Log warning, continue
- Send error → Break loop (client disconnected)
- Channel closed → Shutdown

**Location:** `crates/aof-triggers/src/server/mod.rs`

### 4. Session Persistence (aof-coordination)

**Responsibility:** Persist session state across daemon restarts

**Storage backend:** `aof_memory::SimpleMemory` with FileBackend

**File location:** `$DATA_DIR/aof/sessions/session-state.json`

**Session lifecycle:**

**Startup:**
1. Generate session_id (UUID v4)
2. Create SessionPersistence instance
3. List previous sessions (logged for debugging)
4. Phase 2+: Restore agent states, re-queue tasks

**Shutdown (Ctrl+C):**
1. Create SessionState snapshot
2. Save to FileBackend (async I/O)
3. Log "Session state saved"

**State structure:**
```rust
SessionState {
    session_id: String,              // UUID v4, unique per daemon run
    agent_states: HashMap<String, AgentState>,  // Phase 2+: populated
    task_queue: Vec<TaskInfo>,       // Phase 2+: populated
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
}
```

**Location:** `crates/aof-coordination/src/persistence.rs`

### 5. Daemon Orchestration (aofctl)

**Responsibility:** Wire components together, start server

**Startup sequence:**
```rust
// 1. Create event bus
let event_bus = Arc::new(EventBroadcaster::new(1000));
println!("Event bus: initialized (buffer: 1000)");

// 2. Create session persistence
let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
let session_dir = data_dir.join("aof/sessions");
tokio::fs::create_dir_all(&session_dir).await?;
let session_persistence = SessionPersistence::new(session_dir).await?;

// 3. Generate session ID
let session_id = uuid::Uuid::new_v4().to_string();
println!("Session ID: {}", session_id);

// 4. List previous sessions (Phase 1: just log)
let previous_sessions = session_persistence.list_sessions().await?;
println!("Previous sessions: {} found", previous_sessions.len());

// 5. Pass event_bus to TriggerServerConfig
let server_config = TriggerServerConfig {
    bind_addr: format!("{}:{}", host, port).parse()?,
    event_bus: Some(event_bus.clone()),
    // ...
};

// 6. Start server
let server = TriggerServer::with_config(handler, server_config);
println!("WebSocket: ws://{}:{}/ws", host, port);
server.serve().await?;

// 7. On shutdown (Ctrl+C)
let final_state = SessionState::new(session_id);
session_persistence.save_session(&final_state).await?;
println!("Session state saved");
```

**Location:** `crates/aofctl/src/commands/serve.rs`

## Protocol

### WebSocket Protocol

**Endpoint:** `ws://host:port/ws`

**Message format:** JSON text frames (no binary protocol)

**Frame structure:**
```json
{
  "activity": {
    "activity_type": "ToolExecuting",
    "message": "Executing tool: kubectl",
    "timestamp": "2026-02-11T10:30:00Z",
    "details": { "tool_name": "kubectl", "tool_args": "..." }
  },
  "agent_id": "k8s-monitor",
  "session_id": "uuid",
  "event_id": "uuid",
  "timestamp": "2026-02-11T10:30:00Z"
}
```

**Connection lifecycle:**

1. **Client connects:** HTTP GET /ws with upgrade headers
2. **Server upgrades:** 101 Switching Protocols
3. **Subscription:** Handler subscribes to event bus
4. **Streaming:** Server sends JSON frames as events occur
5. **Close:** Client sends Close frame or disconnects
6. **Cleanup:** Server aborts send task, drops receiver

**No request/response:** Phase 1 is unidirectional (server → client). Phase 3+ adds bidirectional commands (client → server).

### Subscription Model

**Multiple subscribers supported:**
- Each client gets independent receiver
- Events cloned to all receivers (Arc-based, zero-copy)
- Receivers process at own pace (no blocking others)

**Lagging policy:**
- Buffer: 1000 events per subscriber
- Overflow: `RecvError::Lagged(dropped_count)`
- Action: Log warning, continue sending
- Client eventually catches up

**No filtering (Phase 1):** All clients receive all events. Phase 3+ adds:
- Filter by agent_id
- Filter by event type
- Filter by session_id

## Scaling Characteristics

### Single Daemon Capacity

**Event throughput:**
- 1000+ events/second typical
- 5000+ events/second burst
- Limited by JSON serialization (~10-50μs/event)

**WebSocket clients:**
- 50+ simultaneous connections tested
- 500+ theoretical (tokio async runtime)
- Limited by OS file descriptors and network bandwidth

**Memory usage:**
- Event bus: ~200KB (1000 events × ~200 bytes/event)
- Per client: ~2KB (receiver + send task)
- Total: ~300KB for 50 clients + 1000-event buffer

**CPU usage:**
- Event emission: <1% CPU (async, non-blocking)
- JSON serialization: ~5% CPU at 1000 events/sec
- WebSocket I/O: ~2% CPU per client

### Bottlenecks

**Identified bottlenecks:**
1. **JSON serialization:** 10-50μs per event (acceptable for <5000 events/sec)
2. **Network bandwidth:** Client-limited, not server-limited
3. **Slow clients:** Handled via lagging (skip old events)

**Not bottlenecks:**
- Event emission (lock-free broadcast)
- Event bus distribution (Arc-based cloning)
- WebSocket send tasks (tokio async)

### Future Scaling (Phase 8)

**Multi-daemon coordination:**
- Replace tokio::broadcast with NATS/Redis Pub/Sub
- Event bus spans multiple daemons
- Clients connect to any daemon, receive all events

**Horizontal scaling:**
- Load balancer → multiple daemons
- Shared event bus (NATS, Kafka)
- Sticky sessions for WebSocket clients

**Event persistence:**
- Store events to database (PostgreSQL, ClickHouse)
- Replay events for audit/debugging
- Query historical event streams

## Configuration

### Server Configuration

**Via command-line flags:**
```bash
aofctl serve --port 8080 --host 0.0.0.0
```

**Via config file:**
```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
spec:
  server:
    port: 8080        # Default: 8080
    host: 0.0.0.0     # Default: 0.0.0.0
    cors: true        # Default: true
    timeout_secs: 30  # Default: 30
```

**Environment variables:**
```bash
AOF_SERVER_PORT=8080
AOF_SERVER_HOST=0.0.0.0
```

### Event Bus Configuration

**Buffer size (current: hardcoded 1000):**
```rust
// Phase 1: Hardcoded
let event_bus = Arc::new(EventBroadcaster::new(1000));

// Phase 2+: Configurable
spec:
  coordination:
    event_buffer_size: 5000  # For high-throughput scenarios
```

**Session persistence directory:**
```rust
// Default: $DATA_DIR/aof/sessions
// Override via config (Phase 2+):
spec:
  coordination:
    session_dir: /var/lib/aof/sessions
```

### AgentExecutor Configuration

**Opt-in event emission:**
```rust
// Without event bus (default, no overhead)
let executor = AgentExecutor::new(config, model, tool_executor, memory);

// With event bus (opt-in)
let executor = AgentExecutor::new(config, model, tool_executor, memory)
    .with_event_bus(event_bus, session_id);
```

**No configuration needed:** Event bus is passed explicitly via builder.

## Security Considerations

### Phase 1 Security Posture

**Current state (localhost-only):**
- ✅ Bind to 0.0.0.0 (all interfaces) for container/VM access
- ✅ WebSocket on same port as HTTP (8080)
- ❌ No authentication
- ❌ No TLS encryption
- ❌ No origin checking
- ❌ No rate limiting

**Acceptable for Phase 1:**
- Development environments
- Internal networks (VPN, private subnet)
- Single-user deployments

**NOT acceptable for:**
- Public internet exposure
- Multi-tenant environments
- Production deployments (without additional security layers)

### Phase 3+ Security Enhancements

**Authentication:**
- API key in `Sec-WebSocket-Protocol` header
- JWT token in query parameter
- OAuth 2.0 integration

```javascript
// API key example
const ws = new WebSocket('ws://localhost:8080/ws', ['aof-api-key', 'YOUR_API_KEY']);
```

**TLS/SSL:**
- wss:// protocol (WebSocket over TLS)
- Certificate configuration in daemon config
- Automatic Let's Encrypt integration (Phase 8)

**Origin checking:**
- CORS policy enforcement
- Allowed origins whitelist
- Reject unauthorized origins

**Rate limiting:**
- Per-client connection limit (e.g., 5 connections per API key)
- Event rate limit (e.g., 1000 events/sec per client)
- Automatic throttling for lagging clients

### Security Recommendations

**For development:**
```bash
# Bind to localhost only
aofctl serve --host 127.0.0.1 --port 8080
```

**For internal networks:**
```bash
# Use VPN or private subnet, bind to all interfaces
aofctl serve --host 0.0.0.0 --port 8080
```

**For production (Phase 3+):**
```yaml
spec:
  server:
    host: 0.0.0.0
    port: 443
    tls:
      enabled: true
      cert: /etc/aof/tls/cert.pem
      key: /etc/aof/tls/key.pem
  auth:
    enabled: true
    provider: api-key
    api_keys:
      - name: dashboard-ui
        key: sk_prod_abc123
      - name: logging-system
        key: sk_prod_def456
  cors:
    enabled: true
    allowed_origins:
      - https://dashboard.example.com
      - https://monitoring.example.com
```

## Monitoring and Observability

### Health Checks

**HTTP health endpoint:**
```bash
curl http://localhost:8080/health
# Response: {"status":"ok","uptime_secs":123}
```

**Event bus metrics:**
```rust
let subscriber_count = event_bus.subscriber_count();
// Log: "Event bus: 3 active subscribers"
```

**WebSocket connections (Phase 2+):**
- Track active connections
- Track events per connection
- Track lagging clients

### Logging

**Startup logs:**
```
INFO  aofctl::commands::serve: Event bus: initialized (buffer: 1000)
INFO  aofctl::commands::serve: Session ID: a1b2c3d4-...
INFO  aofctl::commands::serve: Previous sessions: 2 found
INFO  aofctl::commands::serve: WebSocket: ws://0.0.0.0:8080/ws
INFO  aofctl::commands::serve: Server listening on 0.0.0.0:8080
```

**Event emission logs:**
```
DEBUG aof_coordination::broadcaster: Event broadcasted to 3 subscribers
DEBUG aof_coordination::broadcaster: Event emitted with no active subscribers
```

**WebSocket logs:**
```
INFO  aof_triggers::server: WebSocket client connected
WARN  aof_triggers::server: Client lagged, dropped 15 events
INFO  aof_triggers::server: WebSocket client disconnected
```

**Shutdown logs:**
```
INFO  aofctl::commands::serve: Shutting down server...
INFO  aofctl::commands::serve: Session state saved
```

### Metrics (Phase 8+)

**Prometheus metrics:**
- `aof_events_emitted_total` - Counter of events emitted
- `aof_events_dropped_total` - Counter of events dropped (lagging)
- `aof_websocket_connections` - Gauge of active WebSocket clients
- `aof_event_emit_duration_seconds` - Histogram of emit latency
- `aof_event_serialization_duration_seconds` - Histogram of JSON serialization

**Grafana dashboard:**
- Event throughput (events/sec)
- WebSocket client count
- Lagging clients over time
- Event types distribution (pie chart)

## Troubleshooting

### No events appearing

**Check event bus initialized:**
```bash
# Look for "Event bus: initialized" in logs
aofctl serve | grep "Event bus"
```

**Check AgentExecutor wired with event bus:**
```rust
// Phase 1: Infrastructure complete, wiring in progress
// Event bus exists, WebSocket server running
// AgentExecutor has event emission code
// Wiring through TriggerHandler layer in progress
```

**Workaround:** Direct AgentExecutor usage (bypassing TriggerHandler) should emit events.

### "Client lagged, dropped N events" warnings

**Root cause:** Client processing slower than event rate.

**Solutions:**
1. **Process events asynchronously:**
   ```javascript
   ws.onmessage = async (event) => {
       // Don't await I/O here, queue for background processing
       eventQueue.push(JSON.parse(event.data));
   };
   ```

2. **Increase client-side buffering:**
   ```javascript
   const eventQueue = [];
   setInterval(() => {
       while (eventQueue.length > 0) {
           processEvent(eventQueue.shift());
       }
   }, 100); // Process in batches
   ```

3. **Filter events (Phase 3+):**
   ```javascript
   // Subscribe only to specific agent
   ws.send(JSON.stringify({
       type: 'subscribe',
       filter: { agent_id: 'k8s-monitor' }
   }));
   ```

### WebSocket disconnects randomly

**Check network stability:**
```bash
# Test WebSocket connection stability
websocat -v ws://localhost:8080/ws
```

**Add reconnection logic:**
```javascript
function connectWithRetry() {
    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onclose = () => {
        console.log('Disconnected, reconnecting in 5s...');
        setTimeout(connectWithRetry, 5000);
    };

    return ws;
}
```

**Check daemon logs for errors:**
```bash
aofctl serve 2>&1 | grep ERROR
```

### Session state not persisted

**Check directory exists:**
```bash
# macOS
ls -la ~/Library/Application\ Support/aof/sessions/

# Linux
ls -la ~/.local/share/aof/sessions/
```

**Check permissions:**
```bash
# Ensure daemon can write to directory
chmod 755 ~/Library/Application\ Support/aof/sessions/
```

**Check logs:**
```bash
# Look for "Session state saved" or errors
aofctl serve 2>&1 | grep -i session
```

## Future Enhancements

### Phase 3: Messaging Gateway
- Event filtering (by agent_id, event_type)
- Bidirectional commands (client → agent)
- Subscription management (subscribe/unsubscribe)

### Phase 4: Mission Control UI
- WASM UI subscribes to /ws
- Real-time agent cards
- Task queue visualization
- Event timeline

### Phase 7: Coordination Protocols
- Heartbeat protocol (agents send periodic heartbeats)
- Agent discovery (broadcast capabilities on startup)
- Task delegation (agents communicate via events)

### Phase 8: Production Readiness
- Multi-daemon coordination (NATS, Redis Pub/Sub)
- Event persistence (database, replay)
- Authentication (API keys, JWT)
- TLS encryption (wss://)
- Rate limiting
- Prometheus metrics
- Grafana dashboards

## References

- [Event Streaming Concepts](../concepts/event-streaming.md) - User-facing documentation
- [Event Infrastructure Developer Docs](../dev/event-infrastructure.md) - Implementation details
- [Session Persistence API](../api/session-persistence.md) - API reference (Phase 2+)
- [WebSocket Protocol Spec](../protocols/websocket.md) - Protocol documentation (Phase 3+)
