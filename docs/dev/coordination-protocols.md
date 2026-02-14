# Coordination Protocols - Internal Developer Documentation

**Status:** Phase 7 Plan 01 complete (Session Tools Foundation)
**Last Updated:** 2026-02-14

## Overview

Coordination protocols enable agents to communicate, report health, and coordinate work without human intervention. The `aof-coordination-protocols` crate provides the infrastructure for:

- **Session Tools** (Plan 01): Agent-to-agent async messaging via tokio mpsc channels
- **Heartbeat Protocol** (Plan 02): Proactive health monitoring on configurable schedules
- **Standup Protocol** (Plan 03): Daily status reports with aggregation
- **Token Metrics** (Plan 04): Coordination overhead tracking (<30% target)

## Crate Architecture

```
aof-core (CoordinationActivity types)
    ↓
aof-coordination (EventBroadcaster, pub/sub)
    ↓
aof-coordination-protocols (SessionTools, Heartbeat, Standup, Metrics)
    ↓
aofctl serve (spawns schedulers, routes events)
```

### Layer Responsibilities

| Layer | Responsibility | Key Types |
|-------|---------------|-----------|
| **aof-core** | Define protocol event types | `CoordinationActivity` enum |
| **aof-coordination** | Broadcast events to subscribers | `EventBroadcaster` |
| **aof-coordination-protocols** | Implement protocols (message queues, schedulers) | `SessionTools`, `Heartbeat`, `Standup` |
| **aofctl serve** | Spawn schedulers, route protocol events | CLI daemon integration |

## Session Tools

### Architecture

Session tools manage **per-agent inbound message queues** using tokio mpsc channels.

```
Agent A ──────┐
Agent B ──────┤──> SessionTools ──> Agent D (inbound queue)
Agent C ──────┘
```

Each agent has **ONE** inbound queue. All senders write to it.

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **tokio mpsc (not broadcast)** | Point-to-point messaging, bounded queues, backpressure |
| **Fire-and-forget (try_send)** | Non-blocking, prevents deadlocks, bounded capacity enforced |
| **TTL filtering on drain** | Expired messages dropped at receive time, not send time |
| **Bounded queues (100 messages)** | Prevents memory bloat, forces backpressure at send |
| **30-minute TTL default** | Reasonable for async coordination, configurable per use case |

### Message Flow

```rust
// 1. Register agents
session_tools.register_agent("agent-a").await?;
session_tools.register_agent("agent-b").await?;

// 2. Agent A sends to Agent B (fire-and-forget)
let msg = SessionMessage::new(
    "agent-a", "agent-b",
    MessageType::Announcement,
    "Starting analysis",
    Duration::from_secs(30 * 60), // TTL
);
session_tools.send_message(msg).await?; // Non-blocking try_send

// 3. Agent B drains messages (filters expired)
let messages = session_tools.drain_messages("agent-b").await;
for msg in messages {
    // Process non-expired messages
}
```

### Error Handling

| Error | Trigger | Recovery |
|-------|---------|----------|
| `QueueFull` | Recipient queue at capacity | Retry with backoff, or drop message |
| `AgentNotFound` | Recipient not registered | Register agent first |
| `MessageExpired` | TTL exceeded | Automatic filter on drain (not an error) |

### Concurrency Safety

- **RwLock**: Allows concurrent reads (multiple senders lookup recipient)
- **try_send**: Non-blocking, no deadlock risk
- **try_recv loop**: Non-blocking drain, no `.recv().await` blocking

## Message Types

### SessionMessage Structure

```rust
pub struct SessionMessage {
    pub id: String,                    // UUID v4
    pub from_agent: String,            // Sender
    pub to_agent: String,              // Recipient
    pub message_type: MessageType,     // Enum variant
    pub content: String,               // Message payload
    pub metadata: HashMap<String, serde_json::Value>, // Extensible
    pub timestamp: DateTime<Utc>,      // Created time
    pub expires_at: DateTime<Utc>,     // Timestamp + TTL
}
```

### MessageType Variants

| Variant | When Used | Example Content |
|---------|-----------|-----------------|
| `Announcement` | Informational broadcast | "Starting cluster analysis" |
| `CollaborationRequest` | Request help from another agent | "Need help analyzing logs for incident-001" |
| `TaskAssignment` | Delegate work | "Analyze pod logs for ns/default" |
| `HumanEscalation` | Route to human | "Cannot resolve: SEV1 database outage" |
| `HeartbeatRequest` | Health check ping | "Heartbeat req-001" |
| `HeartbeatResponse` | Health check pong | "Healthy" or "Degraded: high latency" |
| `StandupRequest` | Daily standup trigger | "Daily standup standup-20260214" |
| `StandupResponse` | Standup report | JSON with what_i_did, what_im_doing, blockers |
| `Custom(String)` | Extensible | Any custom protocol |

### AgentHealthStatus

```rust
pub enum AgentHealthStatus {
    Healthy,                         // Agent responsive, normal operation
    Degraded { reason: String },     // Slow responses, partial functionality
    Unresponsive,                    // No heartbeat response
}
```

### StandupReport

```rust
pub struct StandupReport {
    pub what_i_did: String,          // Recent accomplishments
    pub what_im_doing: String,       // Current work
    pub blockers: Vec<String>,       // Impediments
}
```

## Coordination Modes

Coordination is **opt-in per agent** via `CoordinationMode`:

| Mode | Heartbeat | Standup | Messages | Roundtables | Token Efficiency |
|------|-----------|---------|----------|-------------|------------------|
| `Full` | ✅ 30s | ✅ Daily | ✅ | ✅ | ~30% overhead |
| `Standard` | ✅ 30s | ✅ Daily | ✅ | ❌ | ~20% overhead |
| `Reduced` | ✅ 5min | ❌ | ✅ | ❌ | ~10% overhead |
| `HeartbeatOnly` | ✅ 1min | ❌ | ❌ | ❌ | ~5% overhead |
| `Disabled` | ❌ | ❌ | ❌ | ❌ | 0% overhead |

**Default:** `Standard` (heartbeat + standup + messages, no roundtables)

### Per-Agent Configuration

```yaml
agents:
  - id: k8s-monitor
    coordination_mode: full        # All protocols
  - id: batch-processor
    coordination_mode: disabled    # No coordination overhead
  - id: incident-triage
    coordination_mode: standard    # Default balance
```

## CoordinationActivity Integration

### Extension to CoordinationEvent

The `CoordinationActivity` enum is added to `aof-core::CoordinationEvent` as an optional field:

```rust
pub struct CoordinationEvent {
    // ... existing fields ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordination_activity: Option<CoordinationActivity>,
}
```

### Protocol Event Variants

```rust
pub enum CoordinationActivity {
    HeartbeatRequest { request_id: String },
    HeartbeatResponse { request_id: String, agent_id: String, status: String },
    HeartbeatTimeout { request_id: String, unresponsive_agents: Vec<String> },
    StandupRequest { request_id: String },
    StandupResponse { request_id: String, agent_id: String, what_i_did: String, what_im_doing: String, blockers: Vec<String> },
    StandupSummary { request_id: String, summary: String, agent_count: usize },
    SessionMessage { from_agent: String, to_agent: String, message_type: String, content: String },
}
```

### Convenience Constructors

```rust
// Heartbeat
let event = CoordinationEvent::heartbeat_request("session-123", "req-001");
let event = CoordinationEvent::heartbeat_response("session-123", "req-001", "agent-a", "healthy");
let event = CoordinationEvent::heartbeat_timeout("session-123", "req-001", vec!["agent-x"]);

// Standup
let event = CoordinationEvent::standup_request("session-123", "standup-001");
let event = CoordinationEvent::standup_response("session-123", "standup-001", "agent-a", "Fixed bug", "Working on feature", vec![]);

// Session message
let event = CoordinationEvent::session_message("session-123", "agent-a", "agent-b", "announcement", "Hello");
```

## Heartbeat Protocol

### Architecture

The heartbeat protocol provides **proactive health monitoring** for all registered agents. Unlike traditional polling, agents don't need to be called - the HeartbeatScheduler automatically checks their liveness every 60 seconds.

```
                      60s interval
                          │
                          ▼
┌───────────────────────────────────────────┐
│       HeartbeatScheduler (tokio task)     │
│                                           │
│  tokio::interval(60s) ───> generate UUID │
│                                           │
│  emit HeartbeatRequest ───> broadcast    │
│                                           │
│  track PendingHeartbeat ───> timeout     │
└───────────────────┬───────────────────────┘
                    │
                    ▼
          EventBroadcaster (pub/sub)
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    Agent A     Agent B     Agent C
        │           │           │
        └───────────┴───────────┘
                    │
                    ▼ HeartbeatResponse
        HeartbeatScheduler::handle_response()
                    │
                    ▼
        Update AgentHealthRecord
        ├─ status: Healthy
        ├─ last_heartbeat: now()
        ├─ consecutive_misses: 0
        └─ last_response_ms: 1200

After 120s timeout:
    check_timeout() ───> identify unresponsive
                    │
                    ▼
        emit HeartbeatTimeout alert
                    │
                    ▼
        Update AgentHealthRecord
        ├─ status: Unresponsive
        └─ consecutive_misses: +1
```

### Heartbeat Lifecycle

**1. Registration**
```rust
// CoordinationManager registers agents based on mode
manager.register_agent("k8s-monitor", CoordinationMode::Full).await?;
// Agent added to HeartbeatScheduler's tracked agents
```

**2. Heartbeat Request (every 60s)**
```rust
// HeartbeatScheduler::run() loop
let request_id = Uuid::new_v4();
let event = CoordinationEvent::heartbeat_request(&session_id, &request_id);
event_tx.send(event)?; // Broadcast to all agents

// Track pending request
pending_requests.insert(request_id, PendingHeartbeat {
    timestamp: Utc::now(),
    expected_agents: registered_agents.clone(),
    responded_agents: HashSet::new(),
});

// Spawn timeout checker (120s delay)
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(120)).await;
    scheduler.check_timeout(request_id).await;
});
```

**3. Heartbeat Response**
```rust
// Agent receives HeartbeatRequest, responds with HeartbeatResponse
// (Agent executor handles this automatically)

// CoordinationManager routes response to HeartbeatScheduler
scheduler.handle_response(request_id, agent_id, response_time_ms).await;

// Updates agent health:
agent_health[agent_id] = AgentHealthRecord {
    status: Healthy,
    last_heartbeat: Some(now()),
    consecutive_misses: 0,
    last_response_ms: Some(1200),
};
```

**4. Timeout Detection (120s)**
```rust
// check_timeout() runs after 120s
let pending = pending_requests.remove(&request_id);
let unresponsive = pending.expected_agents - pending.responded_agents;

for agent_id in unresponsive {
    agent_health[agent_id].consecutive_misses += 1;
    agent_health[agent_id].status = Unresponsive;
}

// Emit alert visible in Mission Control
let event = CoordinationEvent::heartbeat_timeout(&session_id, &request_id, unresponsive);
event_tx.send(event)?;
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **60s frequency (not 30s)** | Reduced token cost, still responsive. 60s = $0.01/day for 10 agents. |
| **120s timeout (2x interval)** | LLM-based agents can be slow. 2x interval allows 1 missed heartbeat before alert. |
| **Haiku model (~50 tokens)** | Cheapest Claude variant. Static prompt "Are you alive?" - no context loading. |
| **No context loading** | Heartbeat is super-lightweight. No AGENTS.md, SOUL.md, or memories loaded. |
| **Long-lived agents** | Agents remain available. Heartbeat is validation, not spawn/respawn. |
| **Separate token tracking** | Heartbeat tokens tracked separately for visibility in metrics (Plan 04). |
| **Per-agent opt-in** | Only Full/Standard/Reduced/HeartbeatOnly modes participate. Disabled = no heartbeat. |
| **Arc<HeartbeatScheduler>** | Shared between manager and timeout tasks. Cloned into tokio::spawn closures. |

### Token Efficiency

Heartbeat is designed to minimize token waste:

```
Cost Calculation (10 agents, 60s frequency):
- Requests per day: 10 agents × 1440 minutes ÷ 1 minute = 14,400 heartbeats
- Tokens per heartbeat: ~50 (static "Are you alive?" prompt)
- Total tokens/day: 14,400 × 50 = 720,000 tokens
- Cost (Haiku): ~$0.01/day

Compare to 30s frequency:
- Tokens/day: 1,440,000 (2x)
- Cost: ~$0.02/day
```

**60s frequency reduces cost by 50% while maintaining adequate responsiveness.**

### Health Status Types

```rust
pub enum AgentHealthStatus {
    Healthy,                        // Agent responded within timeout
    Degraded { reason: String },    // Reserved for future use (slow responses, partial failures)
    Unresponsive,                   // Agent missed heartbeat timeout
}
```

### Integration with CoordinationManager

```rust
// CoordinationManager orchestrates all protocols
let manager = CoordinationManager::new(config, event_tx, session_id);

// Register agents with coordination modes
manager.register_agent("k8s-monitor", CoordinationMode::Full).await?;
manager.register_agent("log-analyzer", CoordinationMode::Disabled).await?;
// ↑ Disabled agents NOT registered in heartbeat scheduler

// Start background tasks
let handles = manager.start().await?; // Spawns HeartbeatScheduler::run()

// Query health for REST API
let health = manager.health_snapshot().await; // Vec<AgentHealthRecord>
```

### REST API Endpoint

**GET /api/coordination/health**

Response:
```json
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "Healthy",
      "last_heartbeat": "2026-02-14T10:30:00Z",
      "consecutive_misses": 0,
      "last_response_ms": 1200
    },
    {
      "agent_id": "log-analyzer",
      "status": "Unresponsive",
      "last_heartbeat": "2026-02-14T10:28:30Z",
      "consecutive_misses": 3,
      "last_response_ms": null
    }
  ],
  "heartbeat_config": {
    "frequency_secs": 60,
    "timeout_secs": 120
  }
}
```

If coordination disabled:
```json
{
  "agents": [],
  "coordination_enabled": false
}
```

### Configuration

**serve-config.yaml:**
```yaml
spec:
  coordination:
    enabled: true
    mode: full  # or: standard, reduced, heartbeat_only, disabled
    heartbeat:
      frequency_secs: 60   # How often to check (default: 60)
      timeout_secs: 120    # When to mark unresponsive (default: 120, must be >= frequency)
```

**Per-agent coordination mode** (future enhancement via AGENTS.md):
```yaml
agents:
  - id: k8s-monitor
    coordination_mode: full        # Participates in all protocols
  - id: batch-processor
    coordination_mode: disabled    # No coordination overhead
```

## Implementation Checklist

### Phase 7 Plan 01: Session Tools Foundation ✓

- [x] Create aof-coordination-protocols crate
- [x] Define SessionMessage, MessageType, AgentHealthStatus types
- [x] Implement SessionTools with tokio mpsc
- [x] register_agent, unregister_agent lifecycle
- [x] send_message (fire-and-forget try_send)
- [x] drain_messages (TTL filtering)
- [x] Bounded queues (100 messages, configurable)
- [x] 30-minute TTL (configurable)
- [x] Extend CoordinationEvent with CoordinationActivity
- [x] Convenience constructors for protocol events
- [x] 25 unit tests passing (error + events + session_tools)
- [x] Internal developer documentation
- [x] User-facing concept documentation

### Phase 7 Plan 02: Heartbeat Protocol ✓

- [x] HeartbeatScheduler with tokio::interval
- [x] Send HeartbeatRequest every 60 seconds (configurable)
- [x] Collect HeartbeatResponse from agents
- [x] Detect unresponsive agents (120-second timeout, 2x interval)
- [x] Emit HeartbeatTimeout event to virtual office
- [x] Integration with CoordinationMode (disabled for Disabled mode)
- [x] CoordinationManager orchestrates all protocols
- [x] REST endpoint GET /api/coordination/health
- [x] 16 unit tests for scheduler logic + manager
- [x] Integration with aofctl serve daemon
- [x] Internal developer documentation
- [x] User-facing heartbeat monitoring docs

### Phase 7 Plan 03: Standup Protocol

- [ ] StandupScheduler with cron + timezone
- [ ] Daily trigger (configurable time, e.g., 9am EST)
- [ ] Collect StandupResponse from all agents
- [ ] Aggregate to StandupSummary (LLM summarization)
- [ ] Emit to virtual office (visible in Mission Control)
- [ ] Integration with CoordinationMode (disabled for Reduced/HeartbeatOnly)

### Phase 7 Plan 04: Token Metrics

- [ ] Track tokens spent on coordination vs. production work
- [ ] Measure overhead % per agent
- [ ] Alert if >30% overhead detected
- [ ] Suggest fallback to lower coordination mode

## Testing Strategy

### Unit Tests (25 passing)

- **Error module (7 tests)**: All error variants serialize correctly
- **Events module (7 tests)**: SessionMessage creation, expiry, TTL, serialization
- **SessionTools module (10 tests)**: Registration, send, drain, capacity, multi-sender, fire-and-forget, idempotency
- **Integration tests (1)**: Message expiry filtering

### Integration Testing (Plan 02+)

- Heartbeat scheduler emits events every 30 seconds
- Agents respond to heartbeat requests
- Timeout detection after 60 seconds
- Standup scheduler triggers daily at configured time
- LLM summarization of standup responses

### Performance Testing (Plan 04)

- Measure token overhead % for Full vs. Standard vs. Reduced modes
- Verify <30% coordination overhead target
- Load test: 20 agents, 100 messages/sec, no queue overflow

## Debugging Tips

### Enable Tracing

```bash
export RUST_LOG=aof_coordination_protocols=debug
aofctl serve
```

### Common Issues

| Issue | Symptom | Fix |
|-------|---------|-----|
| Messages not received | drain_messages returns empty | Check agent registered, message not expired |
| QueueFull errors | try_send fails | Increase capacity or agent drains too slowly |
| Messages expire | TTL too short | Increase TTL or drain more frequently |

### Inspect SessionTools State

```rust
// Check registered agents
let agents = session_tools.registered_agents().await;
println!("Registered: {:?}", agents);

// Check capacity
println!("Capacity: {}", session_tools.capacity());
println!("TTL: {:?}", session_tools.ttl());
```

## Future Enhancements

### Post-Phase 7

- **Priority queues**: High-priority messages bypass normal queue
- **Message persistence**: Survive daemon restarts (via SessionPersistence backend)
- **Rate limiting**: Per-agent send limits to prevent spam
- **Message acknowledgment**: Optional ACK/NACK for critical messages
- **Dead letter queue**: Failed messages routed to DLQ for inspection

### Post-Phase 8

- **Distributed coordination**: Multi-daemon session tools (Redis pub/sub backend)
- **Encryption**: E2E encrypted agent messages for sensitive data
- **Audit trail**: All session messages logged for compliance
