# Event Streaming

## What is Event Streaming?

Event streaming in AOF enables **real-time visibility into agent activities**. As agents execute tasks, they emit events describing what they're doing (thinking, calling tools, completing work). These events stream to connected clients in real-time via WebSocket, allowing you to:

- Monitor agent behavior as it happens
- Debug agent decision-making processes
- Build dashboards showing agent activity
- Feed events to logging or alerting systems
- Create real-time Mission Control interfaces

Unlike traditional log files (which you read after the fact), event streaming gives you a **live view into agent execution** as it unfolds.

## Event Types

Agents emit events at specific lifecycle points. Each event type represents a different stage of agent execution:

| Event Type | When Emitted | Example Message |
|------------|--------------|-----------------|
| **Started** | Agent begins execution | "Starting execution for agent: k8s-monitor" |
| **Thinking** | Agent processing/reasoning | "Analyzing cluster health metrics" |
| **IterationStart** | Each agentic loop iteration | "Iteration 1/5" |
| **LLMCall** | Before calling language model | "Calling model for iteration 2" |
| **ToolExecuting** | Tool call begins | "Executing tool: kubectl" |
| **ToolComplete** | Tool call succeeds | "Tool completed: kubectl (234ms)" |
| **ToolFailed** | Tool call fails | "Tool failed: kubectl - connection timeout" |
| **Completed** | Agent finishes successfully | "Execution completed in 5230ms" |
| **Error** | Agent encounters error | "Exceeded max iterations (5)" |

**Key insight:** These events cover **every observable state transition** in agent execution. You can reconstruct the complete agent behavior timeline from the event stream.

## Connecting to the Event Stream

### Starting the Daemon

The AOF daemon must be running to stream events:

```bash
# Start with default port (8080)
aofctl serve

# Or specify custom port
aofctl serve --port 9000
```

**Output:**
```
Event bus: initialized (buffer: 1000)
Session ID: a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6
WebSocket: ws://0.0.0.0:8080/ws
Server listening on 0.0.0.0:8080
```

### Connecting with WebSocket Clients

**Using websocat (recommended for testing):**
```bash
# Install websocat
brew install websocat  # macOS
# or
cargo install websocat

# Connect to event stream
websocat ws://localhost:8080/ws
```

**Using curl (if wscat not available):**
```bash
# Note: curl WebSocket support requires recent version
curl --include \
     --no-buffer \
     --header "Connection: Upgrade" \
     --header "Upgrade: websocket" \
     --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     --header "Sec-WebSocket-Version: 13" \
     ws://localhost:8080/ws
```

**Using JavaScript (browser or Node.js):**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected to AOF event stream');
};

ws.onmessage = (event) => {
    const coordEvent = JSON.parse(event.data);
    console.log(`[${coordEvent.agent_id}] ${coordEvent.activity.message}`);
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected from event stream');
};
```

**Using Python:**
```python
import asyncio
import websockets
import json

async def stream_events():
    uri = "ws://localhost:8080/ws"
    async with websockets.connect(uri) as websocket:
        print("Connected to AOF event stream")
        async for message in websocket:
            event = json.loads(message)
            print(f"[{event['agent_id']}] {event['activity']['message']}")

asyncio.run(stream_events())
```

**Using Rust:**
```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:8080/ws").await.unwrap();
    let (_, read) = ws_stream.split();

    read.for_each(|message| async {
        if let Ok(Message::Text(text)) = message {
            let event: CoordinationEvent = serde_json::from_str(&text).unwrap();
            println!("[{}] {}", event.agent_id, event.activity.message);
        }
    }).await;
}
```

## Event Format

Events are sent as JSON over WebSocket. Each event is a `CoordinationEvent` with the following structure:

### CoordinationEvent Structure

```json
{
  "activity": {
    "activity_type": "ToolExecuting",
    "message": "Executing tool: kubectl",
    "timestamp": "2026-02-11T10:30:00Z",
    "details": {
      "tool_name": "kubectl",
      "tool_args": "get pods -n default"
    }
  },
  "agent_id": "k8s-monitor",
  "session_id": "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
  "event_id": "e5f6g7h8-9i0j-1k2l-3m4n-5o6p7q8r9s0t",
  "timestamp": "2026-02-11T10:30:00Z"
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `activity` | Object | The underlying activity event (what happened) |
| `activity.activity_type` | String | Event type (Started, Thinking, ToolExecuting, etc.) |
| `activity.message` | String | Human-readable event message |
| `activity.timestamp` | String (ISO 8601) | When activity occurred |
| `activity.details` | Object (optional) | Additional event-specific data |
| `agent_id` | String | Agent that emitted this event |
| `session_id` | String | Session grouping (unique per daemon run) |
| `event_id` | String | Unique event identifier (UUID v4) |
| `timestamp` | String (ISO 8601) | When coordination event was created |

### Activity Details by Type

Different event types include different `details` fields:

**Started:**
```json
{
  "activity_type": "Started",
  "message": "Starting execution for agent: k8s-monitor",
  "details": {
    "agent_name": "k8s-monitor"
  }
}
```

**ToolExecuting:**
```json
{
  "activity_type": "ToolExecuting",
  "message": "Executing tool: kubectl",
  "details": {
    "tool_name": "kubectl",
    "tool_args": "get pods -n default"
  }
}
```

**ToolComplete:**
```json
{
  "activity_type": "ToolComplete",
  "message": "Tool completed: kubectl (234ms)",
  "details": {
    "tool_name": "kubectl",
    "duration_ms": 234,
    "success": true
  }
}
```

**Completed:**
```json
{
  "activity_type": "Completed",
  "message": "Execution completed in 5230ms",
  "details": {
    "duration_ms": 5230,
    "iterations": 3
  }
}
```

**Error:**
```json
{
  "activity_type": "Error",
  "message": "Exceeded max iterations (5)",
  "details": {
    "error_type": "MaxIterations",
    "max_iterations": 5
  }
}
```

## Session Persistence

AOF persists agent session state across daemon restarts, enabling agent execution to survive infrastructure changes.

### How It Works

**On daemon startup:**
1. Generate unique `session_id` (UUID v4)
2. Check for previous sessions in storage
3. Print session ID to logs for tracking

**During execution:**
- Agent states update in memory (Phase 2+)
- Task queue tracks pending work (Phase 2+)

**On daemon shutdown (Ctrl+C):**
- Save session state to `$DATA_DIR/aof/sessions/session-state.json`
- Include agent states, task queue, timestamps

**On next startup:**
- Restore previous session state (Phase 2+)
- Resume agents that were Running
- Re-queue pending tasks

### Storage Locations

**macOS:**
```
~/Library/Application Support/aof/sessions/session-state.json
```

**Linux:**
```
~/.local/share/aof/sessions/session-state.json
```

**Windows:**
```
%APPDATA%/aof/sessions/session-state.json
```

### Session State Format

```json
{
  "session_id": "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
  "agent_states": {
    "k8s-monitor": {
      "agent_id": "k8s-monitor",
      "status": "Running",
      "last_activity": "2026-02-11T10:30:00Z",
      "current_task": "Analyzing cluster health"
    }
  },
  "task_queue": [
    {
      "task_id": "task-1",
      "description": "Check pod status",
      "assigned_agent": "k8s-monitor",
      "status": "InProgress",
      "created_at": "2026-02-11T10:25:00Z"
    }
  ],
  "created_at": "2026-02-11T10:00:00Z",
  "last_updated": "2026-02-11T10:30:00Z"
}
```

**Note:** Phase 1 implementation saves session metadata but `agent_states` and `task_queue` are empty. Phase 2+ will populate these during execution.

## Use Cases

### 1. Real-Time Monitoring Dashboard

Build a web dashboard that shows agent activity in real-time:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
const agentCards = {}; // agent_id -> DOM element

ws.onmessage = (event) => {
    const coordEvent = JSON.parse(event.data);
    const agentId = coordEvent.agent_id;

    if (!agentCards[agentId]) {
        agentCards[agentId] = createAgentCard(agentId);
    }

    updateAgentCard(agentCards[agentId], coordEvent);
};

function updateAgentCard(card, event) {
    const activity = event.activity;

    // Update status indicator
    card.querySelector('.status').textContent = activity.activity_type;

    // Update last message
    card.querySelector('.message').textContent = activity.message;

    // Update timestamp
    card.querySelector('.timestamp').textContent =
        new Date(event.timestamp).toLocaleTimeString();

    // Highlight tool executions
    if (activity.activity_type === 'ToolExecuting') {
        card.classList.add('tool-active');
    } else if (activity.activity_type === 'ToolComplete') {
        card.classList.remove('tool-active');
    }
}
```

### 2. Debugging Agent Behavior

Filter events to specific agent for debugging:

```python
async def debug_agent(agent_id):
    uri = "ws://localhost:8080/ws"
    async with websockets.connect(uri) as ws:
        async for message in ws:
            event = json.loads(message)

            # Filter to specific agent
            if event['agent_id'] != agent_id:
                continue

            activity = event['activity']
            timestamp = event['timestamp']

            # Log with timestamps for debugging
            print(f"{timestamp} [{activity['activity_type']}] {activity['message']}")

            # Show tool call details
            if 'details' in activity:
                print(f"  Details: {json.dumps(activity['details'], indent=2)}")
```

### 3. Alerting on Errors

Send alerts when agents encounter errors:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
    const coordEvent = JSON.parse(event.data);

    if (coordEvent.activity.activity_type === 'Error') {
        sendSlackAlert({
            agent: coordEvent.agent_id,
            error: coordEvent.activity.message,
            timestamp: coordEvent.timestamp,
            session: coordEvent.session_id
        });
    }
};

function sendSlackAlert(alert) {
    // Send to Slack webhook
    fetch('https://hooks.slack.com/services/YOUR/WEBHOOK/URL', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            text: `🚨 Agent Error: ${alert.agent}`,
            attachments: [{
                color: 'danger',
                fields: [
                    { title: 'Error', value: alert.error },
                    { title: 'Session', value: alert.session },
                    { title: 'Time', value: alert.timestamp }
                ]
            }]
        })
    });
}
```

### 4. Feeding to Logging Systems

Forward events to centralized logging (Elasticsearch, Splunk, etc.):

```rust
use tokio_tungstenite::connect_async;
use elasticsearch::{Elasticsearch, IndexParts};

#[tokio::main]
async fn main() {
    let es_client = Elasticsearch::default();
    let (ws_stream, _) = connect_async("ws://localhost:8080/ws").await.unwrap();

    let (_, mut read) = ws_stream.split();

    while let Some(Ok(message)) = read.next().await {
        if let Message::Text(text) = message {
            let event: CoordinationEvent = serde_json::from_str(&text).unwrap();

            // Index to Elasticsearch
            es_client
                .index(IndexParts::IndexId("aof-events", &event.event_id))
                .body(&event)
                .send()
                .await
                .unwrap();
        }
    }
}
```

### 5. Mission Control UI (Phase 4)

The foundation for AOF's Mission Control UI, a WASM-based real-time interface showing:
- Agent cards with status indicators (Idle, Running, Completed, Error)
- Live activity feed showing event messages
- Task queue with assigned agents
- Tool execution timeline
- Agent coordination visualization

**Coming in Phase 4:** Full-featured Mission Control UI with real-time updates, filtering, and agent control.

## Multiple Clients

AOF's event streaming supports **multiple simultaneous clients**. Each client receives an independent copy of every event:

```bash
# Terminal 1
websocat ws://localhost:8080/ws

# Terminal 2
websocat ws://localhost:8080/ws

# Terminal 3
websocat ws://localhost:8080/ws
```

All three terminals receive identical events (same `event_id`, same `timestamp`). Events are distributed efficiently using tokio's broadcast channel (zero-copy Arc internally).

**Use cases:**
- Dashboard + logging system + alerting simultaneously
- Multiple developers debugging different aspects
- Separate monitoring systems (metrics, traces, logs)

## Performance Characteristics

### Throughput
- **Event rate:** 1000+ events/second typical
- **Latency:** <10ms from emit to WebSocket send
- **Overhead:** ~10-50μs per event for JSON serialization

### Buffering
- **Buffer size:** 1000 events per subscriber (configurable)
- **Lagging behavior:** Slow clients skip old events (RecvError::Lagged)
- **Warning logged:** "Client lagged, dropped N events"

### Scaling
- **Clients supported:** 50+ simultaneous WebSocket connections per daemon
- **Memory per client:** ~2KB (receiver + send task)
- **Network throughput:** Limited by client, not server

### Disabled Overhead
- **When event_bus=None:** Zero overhead (no allocations, no channel sends)
- **Opt-in via builder:** `AgentExecutor::new(...).with_event_bus(...)`

## Troubleshooting

### "Connection refused" when connecting

**Problem:** Daemon not running or wrong port.

**Solution:**
```bash
# Check daemon is running
ps aux | grep aofctl

# Start daemon with explicit port
aofctl serve --port 8080

# Connect to correct port
websocat ws://localhost:8080/ws
```

### No events appearing

**Problem:** Agent execution hasn't started, or event_bus not wired to executor.

**Solution:**
- Verify agent is executing (trigger via webhook or run agent directly)
- Check logs for "Event bus: initialized"
- Phase 1: Event bus exists but may not be wired through TriggerHandler yet (infrastructure complete, wiring in progress)

### "Client lagged, dropped N events" warnings

**Problem:** Your WebSocket client is processing events slower than they're emitted.

**Solution:**
- Process events asynchronously (don't block on I/O)
- Increase client-side buffering
- Filter events (only process specific agent_ids or event types)
- Future: Server-side filtering (Phase 3)

### Events have same timestamp

**Problem:** High event rate, system clock resolution limited.

**Explanation:** This is expected. Events within same millisecond share timestamp. Use `event_id` (UUID) for uniqueness, not timestamp.

## Next Steps

- **Try it:** Start daemon, connect with websocat, trigger agent
- **Build a dashboard:** Use JavaScript example above
- **Integrate logging:** Forward events to your logging system
- **Phase 3 features:** Event filtering, bidirectional commands
- **Phase 4 features:** Mission Control UI

For architecture details, see [Control Plane Architecture](../architecture/control-plane.md).

For internal implementation details, see [Event Infrastructure Developer Docs](../dev/event-infrastructure.md).
