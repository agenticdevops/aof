# Phase 7: Coordination Protocols - Research

**Researched:** 2026-02-14
**Domain:** Inter-agent communication, heartbeat monitoring, standup protocols, roundtable discussions, token efficiency
**Confidence:** MEDIUM-HIGH

## 1. Executive Summary

Phase 7 implements coordination protocols that make agents proactively report status, coordinate with each other, and maintain visible accountability—without creating unsustainable LLM token costs. The architecture uses **async message queues** (tokio mpsc channels) for agent-to-agent communication (session tools model), **scheduled heartbeats** (30s frequency, 60s timeout) for failure detection, and **daily standups** (9am user-local time, structured text responses) for progress reporting. The critical innovation is **aggressive token efficiency**: heartbeat checks use Claude Haiku (~$0.0003/call), standups use structured templates to minimize tokens, and auto-degradation kicks in if coordination overhead exceeds 30% of total token spend.

**Primary recommendation:** Create `aof-coordination-protocols` crate with three subsystems: (1) **SessionTools** - per-agent-pair tokio mpsc channels for async messaging, (2) **HeartbeatScheduler** - tokio interval task that pings agents every 30s and detects timeouts at 60s, (3) **StandupScheduler** - cron-based daily trigger using existing `schedule.rs` pattern with Haiku-powered structured responses. Implement **token tracking** at the coordination layer with automatic protocol degradation if overhead > 30%. Defer roundtable discussions to Phase 8 (expensive, complex).

**Key insight from existing codebase:** AOF already has robust scheduling infrastructure in `aof-conversational/schedule.rs` (regex + LLM fallback, timezone support, cron validation). The coordination event infrastructure from Phase 1 (tokio broadcast channel, WebSocket streaming) provides the foundation for broadcasting coordination activities. Token measurement can piggyback on existing `Usage` tracking in `aof-core::ModelResponse`.

**User decision (LOCKED):** Implementation order is heartbeat first, then standup, with roundtable discussions deferred to Phase 8 due to token cost complexity.

## 2. Session Tools Architecture

### What Are Session Tools?

Session tools implement **async message queues** between agent pairs, enabling Agent A to send context-rich messages to Agent B without blocking. This is the "announce queue" from COMM-02 requirements. Unlike synchronous RPC (which creates deadlock risks in multi-agent systems), session tools use **fire-and-forget async messaging** with optional acknowledgment.

### Tokio Channel Pattern: mpsc vs broadcast

**Comparison:**

| Channel Type | Use Case | AOF Usage |
|-------------|----------|-----------|
| **tokio::sync::mpsc** | Multi-producer, single-consumer queue | Session tools (Agent A → Agent B) |
| **tokio::sync::broadcast** | Pub/sub event bus | Coordination events (Phase 1 existing) |

**Why mpsc for session tools:**
- Each agent pair has isolated queue (no cross-talk)
- Backpressure control via bounded capacity
- Message ordering guaranteed per pair
- Agent B can drain queue at its own pace

**Implementation:**

```rust
// In aof-coordination-protocols/src/session_tools.rs

use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session tool message (agent-to-agent communication)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: MessageType,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Informational announcement (e.g., "I found a pattern in logs")
    Announcement,
    /// Request for collaboration
    CollaborationRequest,
    /// Task assignment (agent A delegates to agent B)
    TaskAssignment,
    /// Human escalation notice
    HumanEscalation,
    /// Custom message type
    Custom(String),
}

/// Session tools manager - owns all agent-to-agent message queues
pub struct SessionTools {
    /// Map of (from_agent, to_agent) -> mpsc sender
    /// e.g., ("log-analyzer", "k8s-monitor") -> Sender<SessionMessage>
    queues: Arc<RwLock<HashMap<(String, String), mpsc::Sender<SessionMessage>>>>,
    /// Queue capacity (default: 100 messages per pair)
    capacity: usize,
}

impl SessionTools {
    pub fn new(capacity: usize) -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// Send message from Agent A to Agent B
    ///
    /// Creates queue on first send if not exists. Returns error if queue full.
    pub async fn send_message(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
        message: SessionMessage,
    ) -> Result<(), CoordinationError> {
        let from_str = from.into();
        let to_str = to.into();
        let key = (from_str.clone(), to_str.clone());

        let mut queues = self.queues.write().await;
        let sender = queues.entry(key.clone()).or_insert_with(|| {
            let (tx, _rx) = mpsc::channel(self.capacity);
            // Store receiver for target agent to drain
            // (receiver stored separately in agent context)
            tx
        });

        sender.send(message).await
            .map_err(|_| CoordinationError::QueueFull(from_str, to_str))
    }

    /// Subscribe to messages for a specific agent
    ///
    /// Returns receiver that agent can drain in its execution loop.
    pub async fn subscribe(&self, agent_id: impl Into<String>) -> mpsc::Receiver<SessionMessage> {
        let agent_str = agent_id.into();
        // Create receiver for all messages TO this agent
        // (This is a simplified pattern; real implementation would aggregate from all senders)
        let (tx, rx) = mpsc::channel(self.capacity);

        // Register sender in map for all potential FROM agents
        // (Real implementation: lazy create on first send)

        rx
    }

    /// Drain all pending messages for an agent
    ///
    /// Non-blocking: returns empty vec if no messages.
    pub async fn drain_messages(&self, agent_id: &str) -> Vec<SessionMessage> {
        // Get receiver for agent, drain available messages
        // (Implementation detail: receiver stored in agent's runtime context)
        vec![]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    #[error("Queue full: {0} -> {1} (messages dropped)")]
    QueueFull(String, String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),
}
```

### Storage and Persistence

**Question:** Do queued messages persist if agent crashes?

**Answer (MVP):** No. Messages are in-memory only. If Agent B crashes, its message queue is lost. This is acceptable because:
1. Messages are coordination hints, not critical data
2. Agents can retry via heartbeat/standup protocols
3. Simplifies implementation (no disk I/O overhead)

**Future (Phase 8):** Persist queues to file backend (same pattern as SessionPersistence from Phase 1).

### Deadlock Prevention

**Risk:** Agent A waits for response from Agent B, Agent B waits for Agent A → deadlock.

**Mitigation:**
- **No synchronous RPC:** Session tools are fire-and-forget only
- **Timeout on message drain:** Agents check queue with 100ms timeout, then continue
- **Message TTL:** Messages expire after 5 minutes (dropped if not consumed)

**Implementation:**

```rust
// In agent execution loop
#[tokio::main]
async fn agent_execution_loop(session_tools: Arc<SessionTools>) {
    loop {
        // 1. Check for incoming messages (non-blocking)
        let messages = session_tools.drain_messages("agent-id").await;
        if !messages.is_empty() {
            // Process messages as context for next LLM call
            add_messages_to_context(messages);
        }

        // 2. Execute agent task
        execute_agent_task().await;

        // 3. Optionally send messages to other agents
        if needs_collaboration {
            session_tools.send_message(
                "agent-id",
                "other-agent",
                SessionMessage { ... }
            ).await.ok(); // Fire-and-forget
        }

        // 4. Sleep briefly to avoid busy loop
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### Trade-Offs: mpsc vs Redis vs DashMap

| Storage | Latency | Persistence | Complexity | Cost |
|---------|---------|-------------|------------|------|
| **tokio::mpsc** | <1ms | None | Low | None |
| **Redis pub/sub** | 2-5ms | Optional | Medium | Redis instance |
| **DashMap in-memory** | <1ms | None | Low | None |

**Recommendation:** tokio::mpsc for MVP. Redis deferred to multi-daemon deployments (Phase 8).

## 3. Heartbeat Protocol Design

### What Is a Heartbeat?

A heartbeat is a **periodic liveness check** that detects unresponsive agents within a known timeout window. The pattern is proven in distributed systems (etcd Raft uses 100ms heartbeat, 1000ms timeout).

**AOF requirements:**
- **30s frequency:** Heartbeat request sent every 30 seconds
- **60s timeout:** If no response within 60s, agent declared down
- **Model choice:** Use Haiku for cheap "are you alive?" checks

### Heartbeat Scheduler Implementation

**Pattern:** tokio interval task sends HeartbeatRequest to broadcast channel, agents respond with HeartbeatResponse.

```rust
// In aof-coordination-protocols/src/heartbeat.rs

use tokio::time::{interval, Duration};
use tokio::sync::broadcast;
use chrono::{DateTime, Utc};

/// Heartbeat request event (broadcast to all agents)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Heartbeat response event (agent → coordinator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub request_id: String,
    pub agent_id: String,
    pub status: AgentHealthStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentHealthStatus {
    Healthy,
    Degraded { reason: String },
    Unresponsive,
}

/// Heartbeat scheduler - runs periodic checks
pub struct HeartbeatScheduler {
    frequency: Duration,
    timeout: Duration,
    event_tx: broadcast::Sender<CoordinationEvent>,
    pending_requests: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl HeartbeatScheduler {
    pub fn new(
        frequency: Duration,
        timeout: Duration,
        event_tx: broadcast::Sender<CoordinationEvent>,
    ) -> Self {
        Self {
            frequency,
            timeout,
            event_tx,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start heartbeat scheduler (runs indefinitely)
    pub async fn run(self: Arc<Self>) -> Result<(), CoordinationError> {
        let mut tick = interval(self.frequency);

        loop {
            tick.tick().await;

            // Emit heartbeat request
            let request_id = uuid::Uuid::new_v4().to_string();
            let request = HeartbeatRequest {
                request_id: request_id.clone(),
                timestamp: Utc::now(),
            };

            // Track pending request
            {
                let mut pending = self.pending_requests.write().await;
                pending.insert(request_id.clone(), Utc::now());
            }

            // Broadcast request
            let event = CoordinationEvent::heartbeat_request(request);
            self.event_tx.send(event).ok();

            // Spawn timeout checker
            let scheduler = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(scheduler.timeout).await;
                scheduler.check_timeout(request_id).await;
            });
        }
    }

    /// Check for timeout on heartbeat request
    async fn check_timeout(&self, request_id: String) {
        let mut pending = self.pending_requests.write().await;

        if pending.remove(&request_id).is_some() {
            // Request still pending → timeout occurred
            tracing::warn!("Heartbeat timeout for request: {}", request_id);

            // Emit timeout alert
            let alert = CoordinationEvent::heartbeat_timeout(request_id);
            self.event_tx.send(alert).ok();
        }
    }

    /// Mark heartbeat response received
    pub async fn mark_response(&self, request_id: &str) {
        let mut pending = self.pending_requests.write().await;
        pending.remove(request_id);
    }
}
```

### Agent Response Pattern

**How agents respond:**

```rust
// In AgentExecutor runtime
async fn handle_heartbeat_request(request: HeartbeatRequest, agent_id: &str) {
    // Use Haiku for cheap health check
    let prompt = "Respond with 'healthy' if you're functioning normally.";

    let haiku_config = ModelConfig {
        model: "claude-3-haiku-20240307".to_string(),
        provider: ModelProvider::Anthropic,
        temperature: 0.0,
        max_tokens: Some(10),
        ..Default::default()
    };

    let model = create_model(haiku_config).await?;
    let response = model.generate(&ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: prompt.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        system: None,
        tools: vec![],
        temperature: None,
        max_tokens: Some(10),
        stream: false,
        extra: HashMap::new(),
    }).await?;

    // Parse response
    let status = if response.content.to_lowercase().contains("healthy") {
        AgentHealthStatus::Healthy
    } else {
        AgentHealthStatus::Degraded {
            reason: response.content.clone(),
        }
    };

    // Emit response event
    let event = CoordinationEvent::heartbeat_response(HeartbeatResponse {
        request_id: request.request_id,
        agent_id: agent_id.to_string(),
        status,
        timestamp: Utc::now(),
    });

    event_tx.send(event).ok();
}
```

### Heartbeat Frequency: Adaptive vs Fixed

**Question:** Should heartbeat frequency adapt based on agent load?

**Trade-offs:**

| Approach | Pros | Cons |
|----------|------|------|
| **Fixed 30s** | Simple, predictable token cost | Wastes tokens when agents idle |
| **Adaptive (backoff when idle)** | Lower cost, scales with load | Complex, unpredictable budget |

**Recommendation (MVP):** Fixed 30s frequency. Adaptive backoff deferred to Phase 8.

### Timeout Value: 60s vs Other

**Distributed systems research:**
- etcd Raft: 100ms heartbeat, 1000ms timeout (10x ratio)
- Kubernetes kubelet: 10s heartbeat, 40s timeout (4x ratio)
- **AOF recommendation:** 30s heartbeat, 60s timeout (2x ratio)

**Why 2x ratio:**
- Agents execute LLM calls (takes 2-5s), not instant RPC
- Network variability in cloud environments
- False positives expensive (spurious restarts)

**Alternative:** Make timeout configurable per agent (fast agents 45s, slow agents 90s).

## 4. Standup Protocol Design

### What Is a Standup?

A standup is a **structured status report** where agents answer three questions:
1. What did I do since last standup?
2. What am I working on next?
3. What blockers do I have?

**AOF requirements:**
- **Daily 9am trigger:** User's local timezone (extracted from config or inferred)
- **Structured text responses:** Minimize token usage via templates
- **No LLM summarization by default:** Agents respond directly, Sonnet summary optional

### Standup Scheduler Implementation

**Pattern:** Reuse existing `aof-conversational/schedule.rs` cron infrastructure.

```rust
// In aof-coordination-protocols/src/standup.rs

use cron::Schedule;
use std::str::FromStr;
use chrono_tz::Tz;

/// Standup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandupConfig {
    /// Cron expression (default: "0 0 9 * * *" for daily 9am)
    pub cron: String,
    /// Timezone (IANA format, e.g., "America/New_York")
    pub timezone: String,
    /// Enable Sonnet summarization (optional, adds token cost)
    pub summarize: bool,
}

impl Default for StandupConfig {
    fn default() -> Self {
        Self {
            cron: "0 0 9 * * *".to_string(),
            timezone: "UTC".to_string(),
            summarize: false,
        }
    }
}

/// Standup scheduler - triggers daily standup
pub struct StandupScheduler {
    config: StandupConfig,
    event_tx: broadcast::Sender<CoordinationEvent>,
}

impl StandupScheduler {
    pub fn new(config: StandupConfig, event_tx: broadcast::Sender<CoordinationEvent>) -> Self {
        Self { config, event_tx }
    }

    /// Start standup scheduler (runs indefinitely)
    pub async fn run(self: Arc<Self>) -> Result<(), CoordinationError> {
        let schedule = Schedule::from_str(&self.config.cron)
            .map_err(|e| CoordinationError::InvalidCron(e.to_string()))?;
        let tz: Tz = self.config.timezone.parse()
            .map_err(|_| CoordinationError::InvalidTimezone(self.config.timezone.clone()))?;

        loop {
            // Calculate next standup time
            let next = schedule.upcoming(tz).next()
                .ok_or(CoordinationError::InvalidCron("No future runs".to_string()))?;
            let now = Utc::now().with_timezone(&tz);
            let delay = (next - now).to_std()
                .map_err(|_| CoordinationError::InvalidCron("Invalid delay".to_string()))?;

            // Sleep until next standup
            tracing::info!("Next standup scheduled for: {}", next);
            tokio::time::sleep(delay).await;

            // Emit standup request
            let request = StandupRequest {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
            };
            let event = CoordinationEvent::standup_request(request);
            self.event_tx.send(event).ok();

            // Wait 5 minutes for responses, then optionally summarize
            tokio::time::sleep(Duration::from_secs(300)).await;
            if self.config.summarize {
                self.summarize_standup_responses().await;
            }
        }
    }

    /// Summarize standup responses using Sonnet (optional)
    async fn summarize_standup_responses(&self) {
        // Query all StandupResponse events from last 5 min
        // Use Sonnet to generate human-readable summary
        // Emit summary to virtual office (squad chat)
    }
}

/// Standup request (broadcast to all agents)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandupRequest {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Standup response (agent → coordinator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandupResponse {
    pub request_id: String,
    pub agent_id: String,
    pub what_i_did: String,
    pub what_im_doing: String,
    pub blockers: Vec<String>,
    pub timestamp: DateTime<Utc>,
}
```

### Agent Response Pattern: Structured Templates

**Goal:** Minimize LLM tokens by providing structured prompt that guides concise responses.

```rust
// In AgentExecutor runtime
async fn handle_standup_request(request: StandupRequest, agent_id: &str) {
    // Use Haiku for cheap standup (structured response)
    let prompt = format!(
        r#"You are {agent_id}. Answer these three questions concisely (max 50 words each):

1. What did you do since yesterday's standup?
2. What are you working on today?
3. Do you have any blockers?

Respond in this exact format:
DID: <your answer>
DOING: <your answer>
BLOCKERS: <your answer or "none">

Be brief and specific. Focus on results, not process."#,
        agent_id = agent_id
    );

    let haiku_config = ModelConfig {
        model: "claude-3-haiku-20240307".to_string(),
        provider: ModelProvider::Anthropic,
        temperature: 0.3,
        max_tokens: Some(200),
        ..Default::default()
    };

    let model = create_model(haiku_config).await?;
    let response = model.generate(&ModelRequest {
        messages: vec![RequestMessage {
            role: MessageRole::User,
            content: prompt,
            tool_calls: None,
            tool_call_id: None,
        }],
        system: None,
        tools: vec![],
        temperature: Some(0.3),
        max_tokens: Some(200),
        stream: false,
        extra: HashMap::new(),
    }).await?;

    // Parse structured response
    let (did, doing, blockers) = parse_standup_response(&response.content);

    // Emit response event
    let event = CoordinationEvent::standup_response(StandupResponse {
        request_id: request.request_id,
        agent_id: agent_id.to_string(),
        what_i_did: did,
        what_im_doing: doing,
        blockers: blockers,
        timestamp: Utc::now(),
    });

    event_tx.send(event).ok();
}

fn parse_standup_response(content: &str) -> (String, String, Vec<String>) {
    let did = extract_field(content, "DID:");
    let doing = extract_field(content, "DOING:");
    let blockers_str = extract_field(content, "BLOCKERS:");
    let blockers = if blockers_str.to_lowercase() == "none" {
        vec![]
    } else {
        vec![blockers_str]
    };
    (did, doing, blockers)
}
```

### Timezone Detection

**Question:** How to determine user's local timezone for 9am trigger?

**Options:**

1. **Config file:** User sets `standup_timezone: "America/New_York"` in `serve-config.yaml`
2. **System timezone:** Read from system (`chrono::Local::now().timezone()`)
3. **Inferred from first interaction:** Ask user "What's your timezone?" on first startup

**Recommendation:** Config file (explicit, no guessing). Fallback to system timezone if not set.

### Optional Sonnet Summarization

**When to use:**
- **Opt-in only:** User sets `standup_summarize: true` in config
- **After all responses collected:** Wait 5 minutes for agents to respond, then summarize
- **Human-readable format:** Sonnet generates prose summary posted to virtual office

**Cost analysis:**
- 10 agents × 200 tokens each = 2,000 input tokens
- Sonnet summary output = ~300 tokens
- Total: ~$0.01 per standup (if using Sonnet 3.5)

**Trade-off:** Summarization improves readability but adds 30% to standup cost.

## 5. Token Measurement & Auto-Degradation Strategy

### Token Tracking

**Goal:** Measure coordination token usage vs production work, auto-degrade if >30% overhead.

**Implementation:**

```rust
// In aof-coordination-protocols/src/metrics.rs

use std::sync::atomic::{AtomicU64, Ordering};

/// Token usage tracker
pub struct TokenMetrics {
    /// Tokens spent on coordination protocols (heartbeat, standup)
    coordination_tokens: AtomicU64,
    /// Tokens spent on production work (agent tasks)
    production_tokens: AtomicU64,
}

impl TokenMetrics {
    pub fn new() -> Self {
        Self {
            coordination_tokens: AtomicU64::new(0),
            production_tokens: AtomicU64::new(0),
        }
    }

    /// Record coordination tokens (heartbeat/standup)
    pub fn record_coordination(&self, tokens: u64) {
        self.coordination_tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Record production tokens (agent tasks)
    pub fn record_production(&self, tokens: u64) {
        self.production_tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Calculate coordination overhead percentage
    pub fn coordination_overhead(&self) -> f64 {
        let coord = self.coordination_tokens.load(Ordering::Relaxed) as f64;
        let prod = self.production_tokens.load(Ordering::Relaxed) as f64;
        let total = coord + prod;

        if total == 0.0 {
            0.0
        } else {
            (coord / total) * 100.0
        }
    }

    /// Check if degradation needed
    pub fn should_degrade(&self, threshold_percent: f64) -> bool {
        self.coordination_overhead() > threshold_percent
    }
}
```

### Auto-Degradation Modes

**Degradation strategy (if overhead > 30%):**

1. **Level 1 (30-40% overhead):** Disable roundtable discussions (most expensive, not yet implemented)
2. **Level 2 (40-50% overhead):** Reduce heartbeat frequency from 30s to 60s
3. **Level 3 (50%+ overhead):** Switch to heartbeat-only mode (disable standups)

**Implementation:**

```rust
pub struct CoordinationManager {
    mode: Arc<RwLock<CoordinationMode>>,
    metrics: Arc<TokenMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMode {
    /// All protocols enabled
    Full,
    /// Heartbeat + standup (no roundtables)
    Standard,
    /// Heartbeat at reduced frequency (60s instead of 30s)
    Reduced,
    /// Heartbeat only (no standups)
    HeartbeatOnly,
    /// All coordination disabled
    Disabled,
}

impl CoordinationManager {
    pub async fn check_and_degrade(&self) {
        let overhead = self.metrics.coordination_overhead();

        if overhead > 50.0 {
            tracing::warn!("Coordination overhead {}% > 50%, switching to heartbeat-only", overhead);
            *self.mode.write().await = CoordinationMode::HeartbeatOnly;
        } else if overhead > 40.0 {
            tracing::warn!("Coordination overhead {}% > 40%, reducing heartbeat frequency", overhead);
            *self.mode.write().await = CoordinationMode::Reduced;
        } else if overhead > 30.0 {
            tracing::info!("Coordination overhead {}% > 30%, disabling roundtables", overhead);
            *self.mode.write().await = CoordinationMode::Standard;
        }
    }

    /// Get current coordination mode
    pub async fn mode(&self) -> CoordinationMode {
        self.mode.read().await.clone()
    }
}
```

### Token Efficiency Best Practices

1. **Pre-compute static prompts:** Heartbeat and standup prompts are static → cache them
2. **Use prompt caching (Anthropic):** Mark system prompts as cacheable to save tokens
3. **Structured outputs:** Use templates to guide concise responses (avoid rambling)
4. **Model selection:** Haiku for simple checks ($0.25/MTok), Sonnet only for summarization ($3/MTok)

**Cost projection (10 agents, daily):**

| Protocol | Frequency | Tokens/Call | Calls/Day | Daily Cost (Haiku) |
|----------|-----------|-------------|-----------|-------------------|
| Heartbeat | 30s | 50 | 2,880 | $0.036 |
| Standup | Daily 9am | 200 | 10 | $0.0005 |
| **Total** | | | | **$0.037/day** |

**Comparison to production work:** If agents do 100 tasks/day at 1,000 tokens each = 1,000,000 tokens = $250/day (Haiku). Coordination overhead = 0.015% (well below 30%).

## 6. Agent Coordination Modes

### Per-Agent Opt-In/Opt-Out

**Goal:** Allow agents to opt out of coordination protocols if not needed.

**Configuration:**

```yaml
# In AGENTS.md or agent config
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    coordination_mode: full  # full | heartbeat_only | disabled
    ...

  - id: log-analyzer
    name: Log Analyzer
    coordination_mode: heartbeat_only  # No standups, just health checks
    ...

  - id: batch-processor
    name: Batch Processor
    coordination_mode: disabled  # Silent agent, no coordination
    ...
```

**Implementation:**

```rust
pub async fn should_send_heartbeat(&self, agent_id: &str) -> bool {
    let config = self.load_agent_config(agent_id).await;
    matches!(config.coordination_mode, CoordinationMode::Full | CoordinationMode::HeartbeatOnly)
}

pub async fn should_send_standup(&self, agent_id: &str) -> bool {
    let config = self.load_agent_config(agent_id).await;
    matches!(config.coordination_mode, CoordinationMode::Full)
}
```

### Graceful Handling of Disabled Agents

**Question:** What happens if agent has `coordination_mode: disabled` but receives a session message?

**Answer:** Log + ignore. No error, no response.

```rust
async fn handle_session_message(msg: SessionMessage, agent_config: &AgentConfig) {
    if matches!(agent_config.coordination_mode, CoordinationMode::Disabled) {
        tracing::debug!("Agent {} has coordination disabled, ignoring message from {}",
            msg.to_agent, msg.from_agent);
        return;
    }

    // Process message normally
    process_message(msg).await;
}
```

### Default Mode

**Question:** Is coordination on by default or opt-in?

**Recommendation:** **Opt-in** for MVP to avoid surprise token costs. Agents must explicitly set `coordination_mode: full` to participate.

**Rationale:**
- Users may not understand coordination cost implications
- Surprising token bills erode trust
- Explicit opt-in = user knows what they're paying for

## 7. Crate Architecture

### New Crate: aof-coordination-protocols

```
crates/aof-coordination-protocols/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Re-exports
│   ├── session_tools.rs        # Agent-to-agent message queues
│   ├── heartbeat.rs            # Heartbeat scheduler + response handling
│   ├── standup.rs              # Standup scheduler + response parsing
│   ├── metrics.rs              # Token tracking + auto-degradation
│   ├── manager.rs              # CoordinationManager (orchestrates all protocols)
│   └── events.rs               # Event types (HeartbeatRequest, StandupResponse, etc.)
└── tests/
    ├── heartbeat_tests.rs
    ├── standup_tests.rs
    └── session_tools_tests.rs
```

### Dependencies

```toml
[dependencies]
aof-core = { workspace = true }
aof-coordination = { workspace = true }  # For CoordinationEvent
aof-llm = { workspace = true }  # For Model trait
tokio = { workspace = true, features = ["sync", "time"] }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
chrono-tz = "0.8"  # Timezone support
cron = "0.12"  # Cron expression parsing
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
```

## 8. Integration Points

### aof-triggers (Scheduler Integration)

**Existing:** `aof-triggers` has trigger server but no built-in scheduler loop.

**New:** Heartbeat and standup schedulers run as independent tokio tasks spawned by `aofctl serve`.

```rust
// In aofctl/src/commands/serve.rs

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup ...

    // Create coordination manager
    let coordination = Arc::new(CoordinationManager::new(event_tx.clone()));

    // Spawn heartbeat scheduler
    let heartbeat = Arc::new(HeartbeatScheduler::new(
        Duration::from_secs(30),  // frequency
        Duration::from_secs(60),  // timeout
        event_tx.clone(),
    ));
    tokio::spawn(async move {
        heartbeat.run().await.ok();
    });

    // Spawn standup scheduler
    let standup_config = StandupConfig {
        cron: "0 0 9 * * *".to_string(),
        timezone: config.standup_timezone.unwrap_or("UTC".to_string()),
        summarize: config.standup_summarize.unwrap_or(false),
    };
    let standup = Arc::new(StandupScheduler::new(standup_config, event_tx.clone()));
    tokio::spawn(async move {
        standup.run().await.ok();
    });

    // ... start WebSocket server ...
}
```

### aof-core (New Event Types)

**Add to `aof-core/src/coordination.rs`:**

```rust
/// Coordination activity variants (extend existing enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CoordinationActivity {
    // ... existing variants ...

    /// Heartbeat request (broadcast to all agents)
    HeartbeatRequest {
        request_id: String,
    },

    /// Heartbeat response (agent health status)
    HeartbeatResponse {
        request_id: String,
        agent_id: String,
        status: AgentHealthStatus,
    },

    /// Heartbeat timeout alert
    HeartbeatTimeout {
        request_id: String,
        unresponsive_agents: Vec<String>,
    },

    /// Standup request (daily trigger)
    StandupRequest {
        request_id: String,
    },

    /// Standup response (agent status report)
    StandupResponse {
        request_id: String,
        agent_id: String,
        what_i_did: String,
        what_im_doing: String,
        blockers: Vec<String>,
    },

    /// Session message (agent-to-agent communication)
    SessionMessage {
        from_agent: String,
        to_agent: String,
        message_type: String,
        content: String,
    },
}
```

### aofctl (Configuration)

**Add to `serve-config.yaml`:**

```yaml
coordination:
  enabled: true
  mode: full  # full | heartbeat_only | disabled

  heartbeat:
    frequency_secs: 30
    timeout_secs: 60

  standup:
    cron: "0 0 9 * * *"
    timezone: "America/New_York"
    summarize: false

  token_limits:
    max_overhead_percent: 30
    auto_degrade: true
```

### Mission Control UI (Phase 4)

**New components:**

1. **HeartbeatDashboard:** Show agent health status, last heartbeat time, timeout alerts
2. **StandupFeed:** Display standup responses in chronological order
3. **SessionMessageLog:** View agent-to-agent messages in squad chat

**Event subscriptions:**

```typescript
// Subscribe to heartbeat events
websocket.on('HeartbeatResponse', (event) => {
  updateAgentHealth(event.agent_id, event.status);
});

websocket.on('HeartbeatTimeout', (event) => {
  showAlert(`Agents unresponsive: ${event.unresponsive_agents.join(', ')}`);
});

// Subscribe to standup events
websocket.on('StandupResponse', (event) => {
  addStandupToFeed(event);
});
```

## 9. Error Handling & Resilience

### Agent Crashes

**Scenario:** Agent crashes mid-heartbeat check.

**Handling:**
- Heartbeat timeout (60s) detects unresponsive agent
- Alert emitted to Mission Control UI
- Session message queue persists (in-memory, lost on crash)

**Recovery:**
- Daemon restarts agent (if auto-restart enabled)
- New heartbeat cycle begins
- Message queue empty (messages lost)

### Network Partitions

**Scenario:** Agent disconnected from event bus (network split).

**Handling:**
- Heartbeat timeout detects partition
- Agent marked as `Disconnected` in coordination layer
- No new messages routed to disconnected agent

**Recovery:**
- Network heals, agent reconnects
- Agent re-subscribes to event bus
- Heartbeat resumes, agent marked `Healthy`

### Duplicate Responses

**Scenario:** Agent responds to same heartbeat request twice (network retry).

**Handling:**
- Track `request_id` in pending map
- First response removes from map
- Second response ignored (not in map)

**Implementation:**

```rust
pub async fn mark_response(&self, request_id: &str) {
    let mut pending = self.pending_requests.write().await;
    if pending.remove(request_id).is_some() {
        tracing::debug!("Heartbeat response received: {}", request_id);
    } else {
        tracing::warn!("Duplicate heartbeat response ignored: {}", request_id);
    }
}
```

### Message Ordering

**Question:** Does message order matter for session tools?

**Answer:** **Yes** for audit trail (sequence of decisions), **No** for coordination (idempotent operations).

**Mitigation:**
- tokio::mpsc guarantees FIFO per sender-receiver pair
- Timestamp all messages for post-hoc ordering
- UI displays messages in timestamp order (not arrival order)

## 10. Performance & Scalability

### N Agents: How Many Message Queues?

**Question:** For N agents, how many queues?

**Worst case:** N² (every agent talks to every other agent)
- 10 agents → 100 queues
- 100 agents → 10,000 queues

**Realistic:** N×K where K = avg collaboration partners (sparse graph)
- 10 agents, avg 3 collaborators → 30 queues
- 100 agents, avg 5 collaborators → 500 queues

**Memory estimate:**
- tokio::mpsc queue: ~1KB per queue (empty)
- 100 messages @ 1KB each = 100KB per queue
- 500 queues × 100KB = 50MB total

**Recommendation:** Bounded queue capacity (100 messages) to prevent unbounded growth.

### Memory: Bounded Queues + TTL

**Implementation:**

```rust
pub struct SessionTools {
    queues: Arc<RwLock<HashMap<(String, String), mpsc::Sender<SessionMessage>>>>,
    capacity: usize,  // Max 100 messages per queue
    ttl: Duration,    // Messages expire after 5 minutes
}

// Spawn cleanup task
pub async fn cleanup_expired_messages(self: Arc<Self>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        self.remove_expired_messages().await;
    }
}
```

### Latency: Acceptable Delay for Standups?

**Async design:** Standup responses arrive over 5-minute window.

**UI expectation:** Humans don't care about sub-second latency for standup feed.

**Recommendation:** 5-minute collection window is acceptable. Real-time heartbeat responses more critical (1-2 second target).

## 11. Comparison to OpenClaw

### OpenClaw's Session Tools Model

From research ([OpenClaw Session Management](https://docs.openclaw.ai/concepts/session)):

**Key patterns:**
1. **Session-owned state:** All session state owned by gateway (master OpenClaw)
2. **No automatic sharing:** Agents don't share memory by default (separate JSONL logs)
3. **Tool-based communication:** Agents talk via tool calls (not direct message passing)
4. **JSONL history:** Persistent conversation logs per session

**What AOF borrows:**
- Session isolation (agents can opt into/out of coordination)
- Tool-based communication (session tools are "tools" agents can call)
- State persistence (session state survives restarts)

**What AOF adapts:**
- **Async queues instead of blocking RPC:** OpenClaw tools are synchronous, AOF uses async mpsc
- **Broadcast coordination events:** Heartbeat/standup broadcast to all agents (OpenClaw is peer-to-peer)
- **Token efficiency focus:** OpenClaw doesn't emphasize token cost; AOF makes it first-class concern

## 12. Open Questions / Recommendations

### 1. Roundtable Discussions (Deferred to Phase 8)

**Why defer:**
- Most expensive protocol (multi-agent LLM conversation)
- Complex to implement well (consensus, interruption, turn-taking)
- Token costs unpredictable (conversation can spiral)

**MVP:** Agents can send session messages to each other. Structured roundtables with moderator deferred.

### 2. Human-in-the-Loop Task Assignment

**Implementation:** Agents emit `HumanTask` event with context.

```rust
pub struct HumanTask {
    pub task_id: String,
    pub assigned_to: String,  // "human" or specific user
    pub description: String,
    pub context: String,
    pub created_by: String,   // agent_id
    pub priority: TaskPriority,
}
```

**UI integration:** Mission Control displays human tasks in Kanban board "Assigned to Human" column.

**Slack integration (Phase 3):** Gateway routes HumanTask events to Slack DM or channel mention.

### 3. What to Include in Phase 7 vs Phase 8

**Phase 7 (2-3 weeks):**
- ✅ Session tools (async message queues)
- ✅ Heartbeat protocol (30s frequency, 60s timeout, Haiku checks)
- ✅ Standup protocol (daily 9am, structured templates, optional Sonnet summary)
- ✅ Token tracking + auto-degradation (30% threshold)
- ✅ Per-agent coordination modes (full/heartbeat_only/disabled)
- ✅ Mission Control UI integration (heartbeat dashboard, standup feed)

**Phase 8 (Production Readiness):**
- ❌ Roundtable discussions (multi-agent conversations)
- ❌ Adaptive heartbeat frequency (backoff when idle)
- ❌ Message queue persistence (disk-backed queues)
- ❌ Cross-daemon session tools (Redis pub/sub)
- ❌ Human task assignment workflow (approval flow, SLA tracking)

## Sources

**Primary (HIGH confidence - codebase verified):**
- [aof-conversational/schedule.rs](file:///Users/gshah/work/opsflow-sh/aof/crates/aof-conversational/src/schedule.rs) - Existing cron scheduler with timezone support
- [aof-core/src/coordination.rs](file:///Users/gshah/work/opsflow-sh/aof/crates/aof-core/src/coordination.rs) - CoordinationEvent types
- [aof-core/src/model.rs](file:///Users/gshah/work/opsflow-sh/aof/crates/aof-core/src/model.rs) - Model trait, Usage tracking
- [Phase 1 RESEARCH.md](file:///Users/gshah/work/opsflow-sh/aof/.planning/phases/01-event-infrastructure/01-RESEARCH.md) - Event broadcast architecture

**Secondary (MEDIUM confidence - external research):**
- [OpenClaw Session Management](https://docs.openclaw.ai/concepts/session)
- [OpenClaw Architecture Explained](https://ppaolo.substack.com/p/openclaw-system-architecture-overview)
- [Tokio mpsc Channels](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)
- [Mastering Tokio Channels Guide](https://medium.com/@Murtza/mastering-tokio-channels-a-comprehensive-guide-to-inter-task-communication-in-rust-09d860c48010)

**Tertiary (MEDIUM confidence - distributed systems research):**
- [Heartbeats in Distributed Systems](https://arpitbhayani.me/blogs/heartbeats-in-distributed-systems/)
- [Understanding the Heartbeat Pattern](https://medium.com/@a.mousavi/understanding-the-heartbeat-pattern-in-distributed-systems-5d2264bbfda6)
- [Martin Fowler: HeartBeat Pattern](https://martinfowler.com/articles/patterns-of-distributed-systems/heartbeat.html)
- [Heartbeat Detection in Distributed Systems - GeeksforGeeks](https://www.geeksforgeeks.org/heartbeats-detection-a-solution-to-network-failures-in-distributed-systems/)

## Metadata

**Confidence breakdown:**
- **Session tools architecture:** MEDIUM-HIGH - tokio mpsc proven pattern, but inter-agent messaging untested in AOF
- **Heartbeat protocol:** HIGH - Distributed systems standard pattern, existing event infrastructure supports it
- **Standup protocol:** MEDIUM-HIGH - Cron scheduler exists, structured prompts proven in Phase 6, but daily standup flow untested
- **Token measurement:** MEDIUM - Usage tracking exists in ModelResponse, but coordination vs production accounting is new
- **Auto-degradation:** MEDIUM - Logic straightforward, but threshold tuning requires real-world data

**Research date:** 2026-02-14
**Valid until:** 2026-03-14 (28 days - coordination patterns stable, but LLM pricing evolving)

**Key uncertainties:**
- Optimal heartbeat frequency for LLM-based agents (30s may be too aggressive or too conservative)
- Real-world token overhead (projection assumes 10 agents, actual may vary widely)
- User preference for standup summarization (opt-in vs opt-out default)
- Session message queue sizing (100 capacity may be too small for high-traffic agents)
- Timezone inference reliability (config vs system vs user input)

---

## RESEARCH COMPLETE

Ready for planning. Research provides sufficient technical direction to create PLAN.md files for:

**Wave 1: Session Tools Foundation**
- 07-01-PLAN.md: Session tools implementation (tokio mpsc queues, SessionMessage types)

**Wave 2: Heartbeat Protocol**
- 07-02-PLAN.md: Heartbeat scheduler + response handling (30s frequency, 60s timeout, Haiku checks)

**Wave 3: Standup Protocol**
- 07-03-PLAN.md: Standup scheduler + structured responses (daily 9am, cron, timezone support)

**Wave 4: Token Efficiency**
- 07-04-PLAN.md: Token tracking + auto-degradation (metrics, mode switching, 30% threshold)

**Wave 5: UI Integration**
- 07-05-PLAN.md: Mission Control components (heartbeat dashboard, standup feed, session log)

**Wave 6: Testing & Documentation**
- 07-06-PLAN.md: Integration tests, user docs, internal dev docs

**Success criteria:**
- Session tools route messages between agents asynchronously
- Heartbeat detects unresponsive agents within 60 seconds
- Standups run daily at 9am user-local time with structured responses
- Coordination overhead measured and auto-degrades if >30%
- Mission Control UI displays coordination events in real-time
- Token costs predictable and documented
