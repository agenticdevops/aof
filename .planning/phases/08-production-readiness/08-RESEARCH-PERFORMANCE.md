# Phase 8: Production Readiness - Performance Testing Research

**Researched:** 2026-02-14
**Domain:** Load testing, performance optimization, Rust async profiling, WebSocket scaling, production observability
**Confidence:** HIGH

## Executive Summary

Phase 8 performance testing validates AOF's production readiness against aggressive targets: 20 concurrent agents, 50+ simultaneous WebSocket clients, <100ms event latency, and <30% coordination protocol overhead. The research reveals a **multi-layered testing strategy** combining Rust-native benchmarks (Criterion), async runtime profiling (tokio-console), load testing (k6 for WebSocket, Apache Bench for HTTP), and continuous performance regression tracking in CI.

**Primary recommendation:** Implement a **3-tier performance validation pyramid**:
1. **Tier 1: Micro-benchmarks** (Criterion) - Unit-level performance for hot paths (LLM token counting, event serialization, channel throughput)
2. **Tier 2: Integration benchmarks** (custom Rust harness) - Component-level performance (agent execution, WebSocket broadcast, coordination protocol overhead)
3. **Tier 3: Load tests** (k6 + custom scenarios) - System-level stress testing (concurrent agents, sustained WebSocket connections, spike traffic)

**Key insight from existing codebase:** AOF already has token efficiency benchmarking infrastructure in `aof-tools/benchmark/` that measures LLM token overhead with detailed breakdowns. This pattern should be extended to coordination protocols (Phase 7) and used as a template for latency benchmarking.

**Critical success factor:** Establish **performance baselines BEFORE Phase 8 implementation** by running benchmark suite on current v0.4.0-beta codebase. This prevents "optimization theater" (improving metrics that weren't problems) and enables regression detection.

## 1. Load Testing Tools & Rationale

### Recommended Toolchain

| Tool | Purpose | When to Use | Why This Tool |
|------|---------|-------------|---------------|
| **Criterion** | Micro-benchmarks | Hot path optimization, regression detection | Rust-native, statistical rigor, flamegraph integration |
| **k6** | WebSocket load testing | Concurrent client simulation, sustained load | Modern, scriptable (JS), excellent WebSocket support, free |
| **tokio-console** | Async runtime profiling | Tokio task bottlenecks, lock contention | Official Tokio tooling, real-time insight into async behavior |
| **cargo-flamegraph** | CPU profiling | CPU hotspot identification | Rust-native, minimal overhead, visual flame graphs |
| **hyperfine** | CLI benchmark | Command-line latency testing | Simple, accurate, confidence intervals |
| **Custom Rust harness** | Integration benchmarks | Multi-agent coordination, event throughput | Full control, reuse existing AOF types |

### Tool Details

#### Criterion (Micro-Benchmarks)

**Installation:**
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

**Rationale:**
- Statistical analysis (outlier detection, confidence intervals)
- HTML reports with charts
- Regression detection between runs
- Flamegraph integration via `cargo-flamegraph`
- Already proven in Rust ecosystem (60k+ GitHub stars)

**Use cases:**
- Token counting performance (existing in `aof-tools/benchmark/mod.rs`)
- JSON serialization/deserialization of events
- Channel send/recv throughput (tokio::broadcast)
- Memory allocation patterns in hot loops

**Example benchmark:**
```rust
// In crates/aof-core/benches/event_serialization.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aof_core::CoordinationEvent;

fn bench_event_serialization(c: &mut Criterion) {
    let event = CoordinationEvent::agent_started("agent-1", "session-123");

    c.bench_function("serialize_coordination_event", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&event)).unwrap()
        })
    });
}

criterion_group!(benches, bench_event_serialization);
criterion_main!(benches);
```

**Running benchmarks:**
```bash
cargo bench --bench event_serialization
# Generates HTML report in target/criterion/
```

#### k6 (WebSocket Load Testing)

**Installation:**
```bash
brew install k6  # macOS
# or
curl -fsSL https://github.com/grafana/k6/releases/download/v0.49.0/k6-v0.49.0-linux-amd64.tar.gz | tar xvz
```

**Rationale:**
- Native WebSocket support (ws:// and wss://)
- JavaScript-based scenarios (easy to write, version control)
- Metrics: connection time, message latency, throughput
- Thresholds: fail CI if latency > 100ms
- Free, open-source (Grafana Labs)

**Use cases:**
- 50+ concurrent WebSocket clients
- Sustained event streaming (10 events/sec per client for 5 minutes)
- Spike testing (0 → 100 clients in 10 seconds)
- Latency distribution analysis

**Example k6 script:**
```javascript
// tests/load/websocket_broadcast.js
import ws from 'k6/ws';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up to 10 clients
    { duration: '1m', target: 50 },   // Ramp to 50 clients
    { duration: '2m', target: 50 },   // Sustain 50 clients
    { duration: '30s', target: 0 },   // Ramp down
  ],
  thresholds: {
    'ws_msgs_received': ['count > 1000'],
    'ws_session_duration': ['p(95) < 5000'], // 95th percentile < 5s
  },
};

export default function() {
  const url = 'ws://localhost:8080/ws';
  const params = { tags: { name: 'EventStream' } };

  ws.connect(url, params, function(socket) {
    socket.on('open', function() {
      console.log('WebSocket connected');
    });

    socket.on('message', function(data) {
      const event = JSON.parse(data);
      check(event, {
        'has agent_id': (e) => e.agent_id !== undefined,
        'has timestamp': (e) => e.activity?.timestamp !== undefined,
      });
    });

    socket.on('close', function() {
      console.log('WebSocket closed');
    });

    socket.on('error', function(e) {
      console.log('WebSocket error:', e);
    });

    // Keep connection alive for 3 minutes
    sleep(180);
  });
}
```

**Running k6 tests:**
```bash
k6 run tests/load/websocket_broadcast.js
# Output includes:
# - Connection success rate
# - Messages received count
# - Duration percentiles (p50, p95, p99)
# - Failed checks (assertions)
```

#### tokio-console (Async Runtime Profiling)

**Installation:**
```toml
# Cargo.toml
[dependencies]
console-subscriber = "0.2"

# Enable tokio unstable features
tokio = { version = "1.35", features = ["full", "tracing"] }
```

**Setup:**
```rust
// In aofctl/src/main.rs (conditional compilation)
#[cfg(feature = "tokio-console")]
{
    console_subscriber::init();
}
```

**Rationale:**
- Real-time view of tokio tasks (active, idle, blocking)
- Lock contention detection (RwLock, Mutex)
- Async task latency tracking
- Resource leak detection (tasks never completing)

**Use cases:**
- Identify blocking operations in async code (e.g., `std::fs::read` instead of `tokio::fs::read`)
- Detect lock contention (multiple agents waiting on same RwLock)
- Find async tasks with high latency (event broadcast taking >10ms)

**Running tokio-console:**
```bash
# Terminal 1: Run daemon with console enabled
cargo run --features tokio-console --bin aofctl -- serve

# Terminal 2: Launch console UI
tokio-console
```

**Output:** TUI showing:
- Task list (name, state, busy time, idle time, polls)
- Resource usage (locks held, futures awaiting)
- Warnings (tasks blocking for >1s, lock held for >100ms)

#### cargo-flamegraph (CPU Profiling)

**Installation:**
```bash
cargo install flamegraph
```

**Rationale:**
- Visual flame graphs showing CPU hotspots
- Samples stack traces at 99Hz (minimal overhead)
- Works with release builds (optimized code)
- Interactive SVG output (click to zoom)

**Use cases:**
- Identify CPU-bound code paths (JSON serialization, regex matching)
- Validate optimization impact (before/after comparison)
- Profile agent execution under load

**Running flamegraph:**
```bash
# Profile a specific benchmark
cargo flamegraph --bench event_serialization

# Profile integration test
cargo flamegraph --test websocket_broadcast_test

# Profile live daemon (attach to running process)
sudo flamegraph --pid $(pgrep aofctl)
```

**Output:** `flamegraph.svg` - Visual stack trace showing:
- Width = CPU time spent in function
- Height = call stack depth
- Color = random (for visual separation)

**Interpreting results:**
- Wide bars = optimization targets
- Deep stacks = recursive or deeply nested code
- Surprising hot paths = bugs or inefficiencies

### Tool Comparison: k6 vs Apache JMeter vs Artillery

| Feature | k6 | JMeter | Artillery | Recommendation |
|---------|-----|--------|-----------|----------------|
| **WebSocket support** | ✅ Native | ✅ Plugin | ✅ Native | k6 (simplest) |
| **Scripting** | JavaScript | Java/Groovy | YAML/JS | k6 (familiar) |
| **CI integration** | ✅ CLI | ⚠️ Complex | ✅ CLI | k6 or Artillery |
| **Metrics** | Detailed | Very detailed | Basic | k6 (good enough) |
| **Learning curve** | Low | High | Low | k6 or Artillery |
| **Cost** | Free | Free | Free/Paid | All free |
| **Rust integration** | CLI only | CLI only | CLI only | All equal |

**Verdict:** **k6** for AOF because:
- Native WebSocket support (no plugins)
- JavaScript scenarios (widely understood)
- Excellent CLI output (no GUI needed for CI)
- Active development (Grafana Labs)

Alternative: **Artillery** if YAML config preferred over JavaScript.

## 2. Test Scenarios & Success Criteria

### Scenario 1: Baseline Single Agent Performance

**Goal:** Establish baseline latency/throughput before scaling.

**Setup:**
- 1 agent executing simple task (list containers)
- 1 WebSocket client observing events
- No coordination protocols enabled

**Measurements:**
- Agent execution time (start to completion)
- Event emission latency (activity → WebSocket client)
- Memory usage (baseline RAM footprint)

**Success criteria:**
- Agent completes task in <2 seconds
- Event latency <50ms (p95)
- Memory usage <50MB

**Implementation:**
```rust
// tests/performance/baseline_single_agent.rs
#[tokio::test]
async fn test_baseline_single_agent() {
    let daemon = spawn_daemon().await;
    let ws_client = connect_websocket(&daemon).await;

    let start = Instant::now();
    daemon.run_agent("docker-monitor", "list running containers").await;

    let events = ws_client.collect_until_complete().await;
    let duration = start.elapsed();

    assert!(duration < Duration::from_secs(2), "Task took {:?}", duration);
    assert!(events.len() >= 3, "Expected at least 3 events (start, tool_call, complete)");

    for event in &events {
        let latency = event.received_at - event.activity.timestamp;
        assert!(latency < Duration::from_millis(50), "Event latency {:?} > 50ms", latency);
    }
}
```

### Scenario 2: Concurrent Agent Execution (20 Agents)

**Goal:** Verify AOF handles 20 concurrent agents without degradation.

**Setup:**
- 20 agents executing different tasks in parallel
- 5 WebSocket clients (simulating multiple UI sessions)
- Coordination protocols enabled (heartbeat, standup)

**Measurements:**
- Agent throughput (tasks completed per second)
- Event broadcast fanout time (emit → all clients receive)
- Coordination overhead (% of total tokens)
- CPU usage (% across all cores)
- Memory growth (before/after agent execution)

**Success criteria:**
- 20 agents complete within 10 seconds (2 agents/sec throughput)
- Event fanout <100ms to all 5 clients (p95)
- Coordination overhead <30% of tokens
- CPU usage <80% (leaves headroom for 25+ agents)
- Memory growth <200MB (10MB per agent)

**Implementation:**
```rust
// tests/performance/concurrent_agents.rs
#[tokio::test]
async fn test_20_concurrent_agents() {
    let daemon = spawn_daemon_with_coordination().await;
    let ws_clients: Vec<_> = (0..5).map(|_| connect_websocket(&daemon)).collect().await;

    let agent_tasks: Vec<_> = (0..20).map(|i| {
        tokio::spawn(async move {
            daemon.run_agent(&format!("agent-{}", i), "get docker stats").await
        })
    }).collect();

    let start = Instant::now();
    let results = futures::future::join_all(agent_tasks).await;
    let duration = start.elapsed();

    assert!(duration < Duration::from_secs(10), "20 agents took {:?}", duration);
    assert_eq!(results.len(), 20, "All agents completed");

    // Verify event fanout
    for client in ws_clients {
        let events = client.drain_events().await;
        assert!(events.len() >= 60, "Client missing events (expected 3 per agent * 20 = 60)");

        for event in &events {
            let fanout_latency = event.received_at - event.activity.timestamp;
            assert!(fanout_latency < Duration::from_millis(100), "Fanout latency {:?} > 100ms", fanout_latency);
        }
    }

    // Check coordination overhead
    let metrics = daemon.get_token_metrics().await;
    let overhead_pct = metrics.coordination_overhead();
    assert!(overhead_pct < 30.0, "Coordination overhead {:.1}% > 30%", overhead_pct);
}
```

### Scenario 3: WebSocket Client Scaling (50+ Connections)

**Goal:** Validate WebSocket broadcast handles 50+ clients without latency degradation.

**Setup:**
- 5 agents executing continuous tasks (emit events every 2 seconds)
- 50 concurrent WebSocket clients
- 5-minute sustained test (150 events per agent = 750 total events)

**Measurements:**
- Connection establishment time (TCP handshake + WebSocket upgrade)
- Event delivery latency (emit → client receives)
- Memory per WebSocket connection
- Network throughput (bytes/sec across all connections)

**Success criteria:**
- All 50 clients connect successfully
- Connection time <500ms per client
- Event latency <100ms (p95) even under load
- Memory per connection <5MB (250MB total for 50 clients)
- No dropped events (broadcast channel buffer sufficient)

**Implementation (k6 script):**
```javascript
// tests/load/50_websocket_clients.js
import ws from 'k6/ws';
import { check } from 'k6';
import { Counter, Trend } from 'k6/metrics';

const eventsReceived = new Counter('events_received');
const eventLatency = new Trend('event_latency_ms');

export const options = {
  vus: 50,  // 50 virtual users (concurrent WebSocket clients)
  duration: '5m',
  thresholds: {
    'events_received': ['count > 700'], // At least 700 events (allowing for timing variance)
    'event_latency_ms': ['p(95) < 100'], // 95th percentile latency < 100ms
    'ws_connecting': ['p(95) < 500'],    // Connection time < 500ms
  },
};

export default function() {
  const url = 'ws://localhost:8080/ws';
  const startTime = Date.now();

  ws.connect(url, function(socket) {
    socket.on('open', function() {
      const connectTime = Date.now() - startTime;
      console.log(`Connected in ${connectTime}ms`);
    });

    socket.on('message', function(data) {
      const event = JSON.parse(data);
      const now = Date.now();
      const eventTime = new Date(event.activity.timestamp).getTime();
      const latency = now - eventTime;

      eventLatency.add(latency);
      eventsReceived.add(1);

      check(event, {
        'event has agent_id': (e) => e.agent_id !== undefined,
        'latency < 200ms': (e) => latency < 200,
      });
    });

    socket.setTimeout(function() {
      console.log('5 minutes elapsed, closing connection');
      socket.close();
    }, 300000); // 5 minutes
  });
}
```

**Running test:**
```bash
# Start daemon with 5 agents emitting events
cargo run --release --bin aofctl -- serve &

# Run k6 load test
k6 run tests/load/50_websocket_clients.js

# Expected output:
# ✓ events_received....: 750 (success)
# ✓ event_latency_ms...: p(95)=85ms (success)
# ✓ ws_connecting......: p(95)=320ms (success)
```

### Scenario 4: Spike Traffic Test (0 → 100 Clients in 10s)

**Goal:** Verify daemon handles sudden connection bursts without degradation.

**Setup:**
- 0 active agents initially
- 100 WebSocket clients connect simultaneously
- Trigger 10 agents to start after clients connected

**Measurements:**
- Connection acceptance rate (connections/sec)
- First event latency (connection → first event received)
- Event broadcast latency under burst load
- Error rate (failed connections, dropped events)

**Success criteria:**
- All 100 clients connect successfully
- Connection rate >10 clients/sec
- First event latency <1 second after connection
- Event latency <200ms (p95) even during spike
- Zero connection failures

**Implementation (k6 script):**
```javascript
// tests/load/spike_100_clients.js
export const options = {
  stages: [
    { duration: '10s', target: 100 },  // Ramp from 0 to 100 in 10 seconds
    { duration: '2m', target: 100 },   // Sustain 100 clients
    { duration: '10s', target: 0 },    // Ramp down
  ],
  thresholds: {
    'http_req_failed': ['rate < 0.01'],    // <1% failure rate
    'ws_connecting': ['p(95) < 1000'],     // Connection time < 1s
    'event_latency_ms': ['p(95) < 200'],   // Event latency < 200ms
  },
};

// ... similar to Scenario 3 WebSocket handling
```

### Scenario 5: Coordination Protocol Overhead (Token Efficiency)

**Goal:** Validate coordination protocols (Phase 7) stay within 30% overhead budget.

**Setup:**
- 10 agents with coordination enabled (heartbeat 30s, standup daily)
- Run for 1 hour (120 heartbeats per agent = 1200 total checks)
- Agents execute 100 production tasks during test period

**Measurements:**
- Coordination tokens (heartbeat + standup responses)
- Production tokens (agent task execution)
- Overhead percentage (coordination / total)
- Auto-degradation trigger count (if overhead >30%)

**Success criteria:**
- Coordination overhead <30% of total tokens
- No auto-degradation triggered (overhead stays below threshold)
- Heartbeat response rate >99% (agents healthy)
- Standup completion rate 100% (all agents respond)

**Implementation:**
```rust
// tests/performance/coordination_overhead.rs
#[tokio::test]
async fn test_coordination_token_overhead() {
    let daemon = spawn_daemon_with_coordination().await;

    // Enable 10 agents with full coordination mode
    for i in 0..10 {
        daemon.register_agent(&format!("agent-{}", i), CoordinationMode::Full).await;
    }

    // Run agents executing tasks for 1 hour (simulated as 1 minute for testing)
    let tasks_per_agent = 100;
    for i in 0..10 {
        for j in 0..tasks_per_agent {
            daemon.run_agent(&format!("agent-{}", i), "check container status").await;
            tokio::time::sleep(Duration::from_millis(600)).await; // Simulate 1 min as 60ms
        }
    }

    // Check token metrics
    let metrics = daemon.get_token_metrics().await;
    let overhead = metrics.coordination_overhead();

    assert!(overhead < 30.0, "Coordination overhead {:.1}% exceeds 30%", overhead);

    // Verify no auto-degradation occurred
    let degradation_events = daemon.get_coordination_events()
        .await
        .iter()
        .filter(|e| matches!(e, CoordinationEvent::ModeDegraded { .. }))
        .count();

    assert_eq!(degradation_events, 0, "Auto-degradation triggered {} times", degradation_events);

    // Check heartbeat success rate
    let heartbeat_responses = daemon.get_heartbeat_stats().await;
    let success_rate = heartbeat_responses.successful as f64 / heartbeat_responses.total as f64;
    assert!(success_rate > 0.99, "Heartbeat response rate {:.1}% < 99%", success_rate * 100.0);
}
```

### Scenario 6: Memory Leak Detection (Long-Running Stability)

**Goal:** Verify memory usage stabilizes and doesn't grow unbounded.

**Setup:**
- 5 agents running continuously for 24 hours (simulated as 10 minutes)
- 10 WebSocket clients connecting/disconnecting every 30 seconds
- Coordination protocols enabled (heartbeat, standup)

**Measurements:**
- Memory usage at startup
- Memory usage after 1 hour
- Memory usage after 24 hours
- Memory growth rate (MB/hour)

**Success criteria:**
- Memory growth <10MB/hour after initial stabilization
- No unbounded growth (linear or exponential increase)
- Memory usage <1GB after 24 hours

**Implementation:**
```rust
// tests/performance/memory_stability.rs
#[tokio::test]
async fn test_memory_stability_long_running() {
    let daemon = spawn_daemon().await;

    // Baseline memory
    let mem_start = get_process_memory();

    // Run agents for simulated 24 hours (10 minutes compressed)
    let test_duration = Duration::from_secs(600); // 10 minutes
    let samples = 10; // Sample memory 10 times
    let sample_interval = test_duration / samples;

    let mut memory_samples = Vec::new();
    memory_samples.push(mem_start);

    for i in 0..samples {
        tokio::time::sleep(sample_interval).await;

        // Simulate agent activity
        for j in 0..5 {
            daemon.run_agent(&format!("agent-{}", j), "periodic health check").await;
        }

        // Measure memory
        let mem = get_process_memory();
        memory_samples.push(mem);

        println!("Sample {}: {} MB", i + 1, mem / 1_000_000);
    }

    // Check for unbounded growth
    let mem_end = memory_samples.last().unwrap();
    let growth = mem_end - mem_start;
    let growth_rate_mb_per_hour = (growth as f64 / 1_000_000.0) * 6.0; // Extrapolate to 1 hour

    assert!(growth_rate_mb_per_hour < 10.0, "Memory growing at {:.1} MB/hour (exceeds 10 MB/hour limit)", growth_rate_mb_per_hour);
    assert!(mem_end < &1_000_000_000, "Memory usage {} MB > 1000 MB", mem_end / 1_000_000);

    // Linear regression to detect unbounded growth
    let is_stable = check_memory_stability(&memory_samples);
    assert!(is_stable, "Memory growth pattern indicates leak");
}

fn check_memory_stability(samples: &[u64]) -> bool {
    // Simple linear regression: if R² > 0.8 and slope positive, growth is concerning
    // Implementation: least-squares fit, calculate correlation coefficient
    // Return false if strong positive linear trend detected
    // (Full implementation omitted for brevity)
    true
}
```

## 3. Bottleneck Identification Techniques

### Technique 1: tokio-console for Async Task Analysis

**Problem:** Slow event delivery to WebSocket clients (latency spikes >500ms).

**Investigation process:**
1. Enable tokio-console in daemon
2. Launch console UI: `tokio-console`
3. Identify tasks with high "busy time" or "poll count"
4. Check for blocking operations (task state stuck in "busy")
5. Inspect lock resources (RwLock/Mutex contention)

**Common bottlenecks:**
- **Blocking I/O in async context:** `std::fs::read` instead of `tokio::fs::read`
- **Lock contention:** Multiple tasks waiting on same RwLock
- **CPU-bound work in async task:** JSON serialization of large events
- **Unbounded channel backlog:** Slow consumer not draining channel

**Example diagnosis:**
```
Task: websocket_handler
State: busy (1.2s)  ← PROBLEM: Task blocking for 1.2 seconds
Polls: 150
Resources: RwLock<SessionState> (waiting)  ← Contention on SessionState lock
```

**Fix:** Move JSON serialization outside critical section:
```rust
// Before (blocking lock for serialization)
let session = session_state.read().await;
let json = serde_json::to_string(&*session)?; // Lock held during serialization

// After (serialize after releasing lock)
let session_clone = {
    let session = session_state.read().await;
    session.clone() // Quick clone, release lock
};
let json = serde_json::to_string(&session_clone)?; // Serialize without lock
```

### Technique 2: Flamegraph for CPU Hotspot Analysis

**Problem:** High CPU usage (>80%) during 20 concurrent agents test.

**Investigation process:**
1. Run load test with profiling: `cargo flamegraph --test concurrent_agents`
2. Open `flamegraph.svg` in browser
3. Identify widest bars (most CPU time)
4. Drill into hot functions (click to zoom)

**Common hotspots:**
- **JSON serialization:** Wide bar in `serde_json::to_string`
- **Regex matching:** Wide bar in `regex::Regex::is_match`
- **LLM token counting:** Wide bar in `TokenCounter::count`
- **Event cloning:** Wide bar in `Clone::clone` (unnecessary copies)

**Example flamegraph analysis:**
```
Total CPU time: 100%
├─ serde_json::to_string: 35% ← HOTSPOT
│  └─ CoordinationEvent::serialize
├─ tokio::sync::broadcast::send: 20%
├─ regex::Regex::is_match: 15% ← HOTSPOT
│  └─ schedule::parse_cron
└─ agent_executor::execute: 30%
```

**Optimization strategies:**
- **Pre-serialize static data:** Cache JSON for tool definitions (don't serialize on every LLM call)
- **Use faster serialization:** Consider `simd-json` for hot paths
- **Compile regexes once:** Use `lazy_static!` or `OnceCell` for regex patterns
- **Reduce cloning:** Use `Arc<T>` for shared data instead of `T.clone()`

### Technique 3: Memory Profiling with Valgrind Massif

**Problem:** Memory usage growing beyond expected limits.

**Investigation process:**
1. Run daemon under Valgrind: `valgrind --tool=massif cargo run --bin aofctl -- serve`
2. Generate heap snapshot: `ms_print massif.out.<pid>`
3. Identify allocation hotspots
4. Track memory growth over time

**Common memory issues:**
- **Event buffer unbounded:** Broadcast channel buffer growing without bound
- **WebSocket connection leak:** Clients disconnecting but connections not freed
- **Session state accumulation:** Old sessions not cleaned up
- **LLM response caching:** Caching responses without TTL/eviction

**Example Massif output:**
```
--------------------------------------------------------------------------------
  n        time(i)         total(B)   useful-heap(B) extra-heap(B)    stacks(B)
--------------------------------------------------------------------------------
 10      1,000,000      500,000,000      450,000,000    50,000,000            0

KB
500,000|                                                 :::::::::::::::::::::
       |                                        ::::::::::                   :
       |                               :::::::::                             :
       |                      :::::::::                                      :
       |             :::::::::                                                :
       |    :::::::::                                                         :
       +----------------------------------------------------------------------->
         0                                                                Time (i)

Top allocators:
  45.2% (226 MB): tokio::sync::broadcast::channel (event buffer)  ← LEAK
  30.1% (150 MB): axum::WebSocket connections
  15.3% (76 MB):  aof_memory::SessionState
```

**Fix:** Implement bounded event buffer with eviction:
```rust
// Before: Unbounded buffer
let (tx, _) = tokio::sync::broadcast::channel(1000); // Only limits backlog, not history

// After: Manual ring buffer with max size
struct BoundedEventLog {
    events: VecDeque<CoordinationEvent>,
    max_size: usize,
}

impl BoundedEventLog {
    fn push(&mut self, event: CoordinationEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front(); // Evict oldest
        }
        self.events.push_back(event);
    }
}
```

### Technique 4: Custom Instrumentation with Metrics

**Problem:** Unknown performance degradation after Phase 7 coordination protocols added.

**Investigation process:**
1. Add custom metrics to coordination layer
2. Export metrics to Prometheus (or log to console)
3. Compare baseline vs coordination-enabled performance
4. Identify metric that shows regression

**Instrumentation points:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct CoordinationMetrics {
    pub heartbeat_checks: AtomicU64,
    pub heartbeat_latency_ms: AtomicU64,
    pub standup_responses: AtomicU64,
    pub session_messages_sent: AtomicU64,
    pub broadcast_latency_ms: AtomicU64,
}

impl CoordinationMetrics {
    pub fn record_heartbeat(&self, latency_ms: u64) {
        self.heartbeat_checks.fetch_add(1, Ordering::Relaxed);
        self.heartbeat_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn avg_heartbeat_latency(&self) -> f64 {
        let total = self.heartbeat_latency_ms.load(Ordering::Relaxed);
        let count = self.heartbeat_checks.load(Ordering::Relaxed);
        if count == 0 { 0.0 } else { total as f64 / count as f64 }
    }
}
```

**Prometheus export:**
```rust
// In aofctl serve.rs
use prometheus::{Encoder, TextEncoder, Registry, Counter, Histogram};

let registry = Registry::new();
let heartbeat_counter = Counter::new("aof_heartbeat_total", "Total heartbeat checks")?;
let event_latency = Histogram::new(histogram_opts!(
    "aof_event_latency_seconds",
    "Event broadcast latency"
))?;

registry.register(Box::new(heartbeat_counter.clone()))?;
registry.register(Box::new(event_latency.clone()))?;

// HTTP endpoint for Prometheus scraping
app.route("/metrics", get(|| async move {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(buffer)
}));
```

**Grafana dashboard queries:**
```promql
# Average event latency over 5 minutes
rate(aof_event_latency_seconds_sum[5m]) / rate(aof_event_latency_seconds_count[5m])

# Heartbeat success rate
rate(aof_heartbeat_total{status="success"}[5m]) / rate(aof_heartbeat_total[5m])

# WebSocket clients connected
aof_websocket_clients_total
```

## 4. Rust Async Best Practices for Performance

### Practice 1: Avoid Blocking Operations in Async Context

**Problem:** Using `std::fs::read` blocks the tokio thread pool.

**Why this matters:**
- Tokio runs async tasks on a fixed-size thread pool (default: num CPUs)
- Blocking one thread reduces available concurrency
- Under load, all threads can become blocked → deadlock

**Solution:** Use `tokio::fs` for all file I/O.

**Example:**
```rust
// ❌ BAD: Blocks tokio thread
let content = std::fs::read_to_string("session.json")?;

// ✅ GOOD: Async I/O
let content = tokio::fs::read_to_string("session.json").await?;
```

**Detection:** tokio-console shows task state "busy" for extended periods.

### Practice 2: Use `spawn_blocking` for CPU-Bound Work

**Problem:** JSON serialization of large events blocks event loop.

**Why this matters:**
- Async tasks should be I/O-bound, not CPU-bound
- CPU-heavy work (>10ms) should run on blocking thread pool
- Prevents starvation of other async tasks

**Solution:** Use `tokio::task::spawn_blocking` for CPU work >10ms.

**Example:**
```rust
// ❌ BAD: CPU work blocks async executor
let json = serde_json::to_string(&large_event)?;

// ✅ GOOD: Offload to blocking pool
let json = tokio::task::spawn_blocking(move || {
    serde_json::to_string(&large_event)
}).await??;
```

**Guideline:** If operation takes >10ms on average, use `spawn_blocking`.

### Practice 3: Minimize Lock Hold Time

**Problem:** Holding `RwLock` across `.await` points causes contention.

**Why this matters:**
- Async task can be suspended at `.await`
- Lock held during suspension blocks other tasks
- Amplified under concurrent load (20 agents = 20 tasks competing)

**Solution:** Clone data before `.await` or use message passing.

**Example:**
```rust
// ❌ BAD: Lock held across async call
let session = session_state.read().await;
let result = llm_client.generate(&session.context).await?; // Lock held during LLM call
session_state.write().await.update(result);

// ✅ GOOD: Release lock before async call
let context = {
    let session = session_state.read().await;
    session.context.clone() // Clone, then release lock
};
let result = llm_client.generate(&context).await?;
session_state.write().await.update(result);
```

**Detection:** tokio-console shows "RwLock" resource with many waiters.

### Practice 4: Use Bounded Channels to Prevent Backpressure

**Problem:** Unbounded `mpsc` channel causes memory growth when consumer is slow.

**Why this matters:**
- Producer keeps sending events even if consumer can't keep up
- Memory usage grows unbounded
- No signal to producer to slow down

**Solution:** Use bounded channels with appropriate capacity.

**Example:**
```rust
// ❌ BAD: Unbounded channel
let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

// ✅ GOOD: Bounded channel (100 events)
let (tx, rx) = tokio::sync::mpsc::channel(100);

// Send with backpressure handling
match tx.send(event).await {
    Ok(_) => {},
    Err(_) => {
        // Receiver dropped or channel full
        tracing::warn!("Event dropped: channel full");
    }
}
```

**Capacity sizing:**
- **Low latency:** 10-100 (faster feedback on slow consumer)
- **High throughput:** 1000-10000 (amortize send overhead)
- **AOF recommendation:** 100 for session tools (balance latency vs throughput)

### Practice 5: Prefer `Arc<T>` Over `T.clone()` for Large Data

**Problem:** Cloning large events for broadcast to 50 WebSocket clients.

**Why this matters:**
- `clone()` creates deep copy (expensive for large structs)
- 50 clients = 50 clones of same event
- Memory and CPU overhead multiply with client count

**Solution:** Wrap shared data in `Arc<T>` (atomic reference counting).

**Example:**
```rust
// ❌ BAD: Clone event for each client
for client in &clients {
    client.send(event.clone()).await; // 50 clones
}

// ✅ GOOD: Share event via Arc
let event = Arc::new(event);
for client in &clients {
    client.send(Arc::clone(&event)).await; // Just increment ref count
}
```

**When to use `Arc`:**
- Data >1KB and shared across many tasks
- Read-mostly data (write requires `Arc<RwLock<T>>`)
- Event broadcasting (Phase 1 use case)

### Practice 6: Batch Operations to Reduce Syscall Overhead

**Problem:** Writing session state to disk on every event (1000s of writes).

**Why this matters:**
- Each file write is a syscall (expensive)
- Batching reduces syscall count (10x speedup possible)
- File I/O is slowest part of async pipeline

**Solution:** Buffer writes and flush periodically or on threshold.

**Example:**
```rust
// ❌ BAD: Write on every event
for event in events {
    tokio::fs::write("session.json", serde_json::to_string(&event)?).await?;
}

// ✅ GOOD: Batch writes every 10 events or 1 second
let mut buffer = Vec::new();
let mut last_flush = Instant::now();

for event in events {
    buffer.push(event);

    if buffer.len() >= 10 || last_flush.elapsed() > Duration::from_secs(1) {
        let json = serde_json::to_string(&buffer)?;
        tokio::fs::write("session.json", json).await?;
        buffer.clear();
        last_flush = Instant::now();
    }
}

// Flush remaining
if !buffer.is_empty() {
    tokio::fs::write("session.json", serde_json::to_string(&buffer)?).await?;
}
```

### Practice 7: Use `select!` for Timeout and Cancellation

**Problem:** Agent execution hangs indefinitely if LLM provider is down.

**Why this matters:**
- Without timeout, task never completes
- Resources (memory, connections) leak
- System becomes unresponsive

**Solution:** Use `tokio::select!` for timeout or cancellation.

**Example:**
```rust
// ❌ BAD: No timeout
let response = llm_client.generate(&request).await?;

// ✅ GOOD: 30-second timeout
tokio::select! {
    result = llm_client.generate(&request) => {
        result?
    }
    _ = tokio::time::sleep(Duration::from_secs(30)) => {
        return Err(anyhow!("LLM request timed out after 30s"));
    }
}
```

**AOF use cases:**
- Agent execution timeout (max 5 minutes)
- Heartbeat response timeout (60 seconds)
- WebSocket ping timeout (30 seconds)

## 5. Performance Regression Prevention (CI Integration)

### Strategy 1: Criterion Benchmark Baseline Tracking

**Goal:** Fail CI if benchmark regresses by >10%.

**Setup:**
```yaml
# .github/workflows/performance.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Cache Cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run Criterion benchmarks
        run: cargo bench --bench event_serialization -- --save-baseline main

      - name: Compare with baseline
        run: |
          cargo bench --bench event_serialization -- --baseline main
          # Criterion outputs regression warnings to stdout
          # Parse output for "change: +15%" (>10% regression)

      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: criterion-results
          path: target/criterion/
```

**Criterion regression detection:**
```rust
// In benches/event_serialization.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn event_serialization_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_serialization");

    // Set regression threshold to 10%
    group.significance_level(0.1);
    group.sample_size(100);

    let event = CoordinationEvent::agent_started("agent-1", "session-123");

    group.bench_function("serialize_coordination_event", |b| {
        b.iter(|| serde_json::to_string(&event).unwrap());
    });

    group.finish();
}

criterion_group!(benches, event_serialization_suite);
criterion_main!(benches);
```

**CI failure criteria:**
- Benchmark time increases >10% vs baseline
- Baseline updated only on `main` branch merge
- PR builds compare against latest `main` baseline

### Strategy 2: k6 Load Test Thresholds in CI

**Goal:** Fail CI if load test doesn't meet SLOs.

**Setup:**
```yaml
# .github/workflows/load-tests.yml
name: Load Tests

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Nightly at 2 AM

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build release binary
        run: cargo build --release --bin aofctl

      - name: Start daemon
        run: |
          ./target/release/aofctl serve &
          sleep 5  # Wait for startup

      - name: Install k6
        run: |
          curl -fsSL https://github.com/grafana/k6/releases/download/v0.49.0/k6-v0.49.0-linux-amd64.tar.gz | tar xvz
          sudo mv k6-v0.49.0-linux-amd64/k6 /usr/local/bin/

      - name: Run WebSocket load test
        run: |
          k6 run tests/load/50_websocket_clients.js \
            --out json=results.json

      - name: Check thresholds
        run: |
          # k6 exits with code 99 if thresholds failed
          if [ $? -eq 99 ]; then
            echo "Load test thresholds failed"
            exit 1
          fi

      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: k6-results
          path: results.json
```

**k6 thresholds (in test script):**
```javascript
export const options = {
  thresholds: {
    'events_received': ['count > 700'],
    'event_latency_ms': ['p(95) < 100'],  // CI FAILS if p95 > 100ms
    'ws_connecting': ['p(95) < 500'],
    'http_req_failed': ['rate < 0.01'],   // CI FAILS if >1% errors
  },
};
```

**CI failure scenarios:**
- Event latency p95 >100ms
- Connection failure rate >1%
- Less than 700 events received

### Strategy 3: Memory Leak Detection in CI

**Goal:** Detect memory leaks before merge.

**Setup:**
```yaml
# .github/workflows/memory-check.yml
name: Memory Leak Detection

on:
  pull_request:
    branches: [main]

jobs:
  memory-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Valgrind
        run: sudo apt-get install -y valgrind

      - name: Build debug binary
        run: cargo build --bin aofctl

      - name: Run Valgrind leak check
        run: |
          valgrind --leak-check=full --show-leak-kinds=all \
            --error-exitcode=1 \
            ./target/debug/aofctl serve &
          DAEMON_PID=$!
          sleep 30  # Run for 30 seconds
          kill $DAEMON_PID
          wait $DAEMON_PID

      - name: Parse Valgrind output
        run: |
          # Check for "definitely lost" or "indirectly lost" memory
          grep "definitely lost" valgrind.log && exit 1
          grep "indirectly lost" valgrind.log && exit 1
          exit 0
```

**Valgrind leak categories:**
- **"definitely lost":** Memory allocated but not freed (FAIL CI)
- **"indirectly lost":** Lost because parent was lost (FAIL CI)
- **"still reachable":** Memory held at exit (WARN, acceptable for daemon)

### Strategy 4: Performance Dashboard (Grafana + InfluxDB)

**Goal:** Track performance trends over time.

**Setup:**
1. **InfluxDB:** Store benchmark results
2. **Grafana:** Visualize trends
3. **CI integration:** Push results to InfluxDB after each build

**CI step:**
```yaml
- name: Push benchmark results to InfluxDB
  run: |
    # Parse Criterion JSON output
    LATENCY=$(jq '.mean.point_estimate' target/criterion/event_serialization/base/estimates.json)

    # Push to InfluxDB
    curl -XPOST "https://influxdb.example.com/write?db=aof_benchmarks" \
      --data-binary "event_serialization,commit=$GITHUB_SHA latency_ns=$LATENCY $(date +%s)000000000"
```

**Grafana dashboard:**
- Chart: Event serialization latency over time (line graph, commits on X-axis)
- Alert: Trigger if latency >10% above 7-day moving average
- Comparison: Show PR branch vs `main` baseline

**Metrics to track:**
- Event serialization latency (ns)
- WebSocket connection time (ms)
- Agent execution time (ms)
- Memory usage (MB)
- Token efficiency (coordination overhead %)

## 6. WebSocket Scaling Patterns

### Pattern 1: Connection Pooling with Shared Broadcast Channel

**Problem:** Each WebSocket connection creates separate broadcast receiver.

**Current architecture (Phase 1):**
```rust
// Each WebSocket gets independent receiver
async fn handle_websocket(socket: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let mut rx = event_bus.subscribe(); // New receiver per connection
    while let Ok(event) = rx.recv().await {
        socket.send(event).await;
    }
}
```

**Scaling:** 50 clients = 50 receivers. `tokio::broadcast` handles this efficiently (minimal overhead per receiver).

**Optimization (if >100 clients):** Connection pooling.

```rust
// Group clients into pools, broadcast to pools instead of individuals
struct WebSocketPool {
    clients: Vec<WebSocket>,
}

async fn handle_websocket_pool(pool: Arc<Mutex<WebSocketPool>>, event_bus: Arc<EventBroadcaster>) {
    let mut rx = event_bus.subscribe();
    while let Ok(event) = rx.recv().await {
        let clients = pool.lock().await;
        for client in &*clients {
            client.send(event.clone()).await.ok(); // Ignore send errors
        }
    }
}
```

**Trade-off:** Single receiver per pool (lower memory) vs independent receivers (better isolation).

**AOF recommendation:** Individual receivers for <100 clients, pooling for >100.

### Pattern 2: Backpressure Handling with Slow Client Detection

**Problem:** One slow client shouldn't block event broadcast to fast clients.

**tokio::broadcast behavior:**
- Slow receiver lags behind
- If lag >buffer size (1000 events), receiver gets `RecvError::Lagged`
- Receiver skips missed events

**Handling lagged receivers:**
```rust
async fn handle_websocket(socket: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let mut rx = event_bus.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                if socket.send(event).await.is_err() {
                    break; // Client disconnected
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("WebSocket client lagged by {} events, skipping", n);
                // Optionally send notification to client
                socket.send(Event::SystemMessage("You missed some events due to slow connection")).await.ok();
            }
            Err(_) => break, // Channel closed
        }
    }
}
```

**Metrics to track:**
- Lag count per client (how many `Lagged` errors)
- Average lag size (how many events skipped)
- Clients with frequent lag (disconnect them?)

### Pattern 3: Event Filtering to Reduce Bandwidth

**Problem:** Mobile clients on cellular don't need all events.

**Solution:** Client-side event filtering (subscription model).

**Example:**
```rust
// Client subscribes to specific event types
struct WebSocketSubscription {
    agent_ids: HashSet<String>,
    event_types: HashSet<ActivityType>,
}

async fn handle_websocket_filtered(
    socket: WebSocket,
    subscription: WebSocketSubscription,
    event_bus: Arc<EventBroadcaster>,
) {
    let mut rx = event_bus.subscribe();

    while let Ok(event) = rx.recv().await {
        // Filter by subscription
        if subscription.agent_ids.contains(&event.agent_id)
            && subscription.event_types.contains(&event.activity.activity_type)
        {
            socket.send(event).await.ok();
        }
    }
}
```

**Bandwidth savings:**
- Without filtering: 10 agents × 3 events/sec = 30 events/sec × 1KB = 30 KB/sec
- With filtering (1 agent): 1 agent × 3 events/sec = 3 KB/sec (10× reduction)

**UI implementation:** Mission Control sends subscription via initial WebSocket message.

```javascript
// Client-side (Mission Control UI)
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // Subscribe to specific agents and event types
  ws.send(JSON.stringify({
    type: 'subscribe',
    agent_ids: ['k8s-monitor', 'log-analyzer'],
    event_types: ['ToolExecuting', 'AgentCompleted'],
  }));
};
```

### Pattern 4: WebSocket Ping/Pong for Connection Health

**Problem:** Detect dead connections (client closed without notification).

**Solution:** Periodic ping/pong with timeout.

**Implementation:**
```rust
use tokio::time::{interval, Duration};

async fn handle_websocket_with_ping(socket: WebSocket, event_bus: Arc<EventBroadcaster>) {
    let (mut tx, mut rx) = socket.split();
    let mut event_rx = event_bus.subscribe();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Send events
            Ok(event) = event_rx.recv() => {
                if tx.send(Message::Text(serde_json::to_string(&event)?)).await.is_err() {
                    break; // Send failed, connection dead
                }
            }

            // Send ping
            _ = ping_interval.tick() => {
                if tx.send(Message::Ping(vec![])).await.is_err() {
                    tracing::warn!("WebSocket ping failed, closing connection");
                    break;
                }
            }

            // Receive pong (or close)
            msg = rx.next() => {
                match msg {
                    Some(Ok(Message::Pong(_))) => {
                        // Client alive
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // Client requested close or disconnected
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
```

**Timeout:** If no pong within 30 seconds, close connection.

**Benefit:** Free up resources from dead connections (memory leak prevention).

## 7. Metrics & Observability

### Prometheus Metrics

**Add to `aofctl` serve command:**

```toml
# Cargo.toml
[dependencies]
prometheus = "0.13"
```

**Metrics to expose:**
```rust
use prometheus::{Registry, Counter, Histogram, Gauge, histogram_opts, opts};

pub struct AofMetrics {
    // Event metrics
    pub events_emitted: Counter,
    pub event_broadcast_latency: Histogram,

    // Agent metrics
    pub agents_active: Gauge,
    pub agent_execution_time: Histogram,

    // WebSocket metrics
    pub websocket_clients: Gauge,
    pub websocket_messages_sent: Counter,
    pub websocket_messages_failed: Counter,

    // Coordination metrics (Phase 7)
    pub heartbeat_checks: Counter,
    pub heartbeat_failures: Counter,
    pub standup_responses: Counter,
    pub coordination_tokens: Counter,
    pub production_tokens: Counter,
}

impl AofMetrics {
    pub fn new() -> Result<(Self, Registry)> {
        let registry = Registry::new();

        let events_emitted = Counter::new("aof_events_emitted_total", "Total events emitted")?;
        let event_broadcast_latency = Histogram::with_opts(histogram_opts!(
            "aof_event_broadcast_latency_seconds",
            "Event broadcast latency",
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5] // Buckets: 1ms to 500ms
        ))?;

        let agents_active = Gauge::new("aof_agents_active", "Number of active agents")?;
        let agent_execution_time = Histogram::with_opts(histogram_opts!(
            "aof_agent_execution_seconds",
            "Agent task execution time",
            vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0] // Buckets: 0.5s to 1 minute
        ))?;

        let websocket_clients = Gauge::new("aof_websocket_clients", "Number of connected WebSocket clients")?;
        let websocket_messages_sent = Counter::new("aof_websocket_messages_sent_total", "Total WebSocket messages sent")?;
        let websocket_messages_failed = Counter::new("aof_websocket_messages_failed_total", "Failed WebSocket sends")?;

        registry.register(Box::new(events_emitted.clone()))?;
        registry.register(Box::new(event_broadcast_latency.clone()))?;
        registry.register(Box::new(agents_active.clone()))?;
        registry.register(Box::new(agent_execution_time.clone()))?;
        registry.register(Box::new(websocket_clients.clone()))?;
        registry.register(Box::new(websocket_messages_sent.clone()))?;
        registry.register(Box::new(websocket_messages_failed.clone()))?;

        Ok((Self {
            events_emitted,
            event_broadcast_latency,
            agents_active,
            agent_execution_time,
            websocket_clients,
            websocket_messages_sent,
            websocket_messages_failed,
        }, registry))
    }
}
```

**Instrumentation example:**
```rust
// In event broadcaster
impl EventBroadcaster {
    pub fn emit(&self, event: CoordinationEvent) {
        let start = Instant::now();
        self.tx.send(event).ok();
        let latency = start.elapsed().as_secs_f64();

        self.metrics.events_emitted.inc();
        self.metrics.event_broadcast_latency.observe(latency);
    }
}
```

**HTTP endpoint for Prometheus:**
```rust
// In aofctl serve.rs
use prometheus::{Encoder, TextEncoder};

app.route("/metrics", get(metrics_handler));

async fn metrics_handler(State(metrics): State<Arc<AofMetrics>>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        buffer,
    )
}
```

### Structured Logging with `tracing`

**Already in AOF workspace:** `tracing = "0.1"`

**Instrumentation best practices:**
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(event_bus))]
async fn execute_agent(agent_id: &str, task: &str, event_bus: Arc<EventBroadcaster>) -> Result<()> {
    info!("Starting agent execution");

    let start = Instant::now();

    // Agent execution logic
    let result = perform_task(task).await?;

    let duration = start.elapsed();
    info!(duration_ms = duration.as_millis(), "Agent execution completed");

    Ok(())
}
```

**Log levels:**
- **ERROR:** Unrecoverable failures (LLM provider down, file write failed)
- **WARN:** Recoverable issues (heartbeat timeout, WebSocket lag)
- **INFO:** High-level operations (agent started, task completed)
- **DEBUG:** Detailed execution (tool call parameters, event details)
- **TRACE:** Very verbose (token counting, JSON serialization)

**Structured fields:**
```rust
info!(
    agent_id = %agent_id,
    task_type = %task_type,
    duration_ms = duration.as_millis(),
    tokens_used = usage.total_tokens,
    "Agent task completed"
);
```

**JSON output (for log aggregation):**
```bash
RUST_LOG=info cargo run --bin aofctl -- serve 2>&1 | jq .
```

**Output:**
```json
{
  "timestamp": "2026-02-14T10:30:45.123Z",
  "level": "INFO",
  "target": "aofctl::commands::serve",
  "fields": {
    "agent_id": "k8s-monitor",
    "task_type": "health_check",
    "duration_ms": 1234,
    "tokens_used": 567
  },
  "message": "Agent task completed"
}
```

## 8. Integration with CI/CD

### GitHub Actions Workflow

```yaml
# .github/workflows/performance-suite.yml
name: Performance Test Suite

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  schedule:
    - cron: '0 3 * * *'  # Nightly at 3 AM

jobs:
  micro-benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Criterion benchmarks
        run: |
          cargo bench --bench event_serialization -- --save-baseline pr-${{ github.event.pull_request.number }}

      - name: Compare with main
        if: github.event_name == 'pull_request'
        run: |
          git fetch origin main
          git checkout origin/main
          cargo bench --bench event_serialization -- --save-baseline main
          git checkout -
          cargo bench --bench event_serialization -- --baseline main

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: criterion-results
          path: target/criterion/

  load-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build release
        run: cargo build --release --bin aofctl

      - name: Start daemon
        run: |
          ./target/release/aofctl serve &
          sleep 5

      - name: Install k6
        run: |
          curl -fsSL https://github.com/grafana/k6/releases/download/v0.49.0/k6-v0.49.0-linux-amd64.tar.gz | tar xvz
          sudo mv k6*/k6 /usr/local/bin/

      - name: Run WebSocket load test
        run: k6 run tests/load/50_websocket_clients.js

      - name: Run concurrent agents test
        run: k6 run tests/load/20_concurrent_agents.js

      - name: Upload k6 results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: k6-results
          path: '*.json'

  memory-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Valgrind
        run: sudo apt-get install -y valgrind

      - name: Build debug
        run: cargo build --bin aofctl

      - name: Run memory leak check
        run: |
          valgrind --leak-check=full --error-exitcode=1 \
            timeout 30s ./target/debug/aofctl serve || true

      - name: Parse Valgrind output
        run: |
          if grep -q "definitely lost" valgrind.log; then
            echo "Memory leak detected"
            exit 1
          fi
```

## 9. Known Rust Async Gotchas

### Gotcha 1: `Send` Bounds on Async Functions

**Problem:** Compiler error: "future cannot be sent between threads safely".

**Cause:** Holding non-`Send` type (like `Rc`) across `.await` point.

**Example:**
```rust
// ❌ Fails to compile: Rc is not Send
async fn process_events(events: Rc<Vec<Event>>) {
    tokio::time::sleep(Duration::from_secs(1)).await; // Await point
    process(events);
}

// ✅ Fixed: Use Arc instead of Rc
async fn process_events(events: Arc<Vec<Event>>) {
    tokio::time::sleep(Duration::from_secs(1)).await;
    process(events);
}
```

**Rule:** Async functions must hold only `Send` types across `.await`.

### Gotcha 2: Holding `MutexGuard` Across `.await`

**Problem:** Compiler error: "`MutexGuard` cannot be sent between threads".

**Cause:** Mutex guard held when task is suspended at `.await`.

**Example:**
```rust
// ❌ Fails: MutexGuard held across await
let mut state = mutex.lock().unwrap();
some_async_operation().await;
state.update();

// ✅ Fixed: Release guard before await
{
    let mut state = mutex.lock().unwrap();
    state.prepare();
} // Guard dropped here
some_async_operation().await;
```

**Rule:** Always drop `MutexGuard` before `.await`.

### Gotcha 3: Spawning Tasks Without `.await`

**Problem:** Task spawned but never polled to completion.

**Cause:** Forgetting to `.await` on `JoinHandle`.

**Example:**
```rust
// ❌ Task orphaned: result ignored
tokio::spawn(async {
    process_data().await
});

// ✅ Task managed: await completion
let handle = tokio::spawn(async {
    process_data().await
});
handle.await?;
```

**Rule:** Always `.await` task handles or store them for later.

### Gotcha 4: Unbounded Recursion in Async Functions

**Problem:** Stack overflow in async recursion.

**Cause:** Each recursive call creates new future on stack.

**Example:**
```rust
// ❌ Stack overflow for deep recursion
async fn recursive_process(depth: usize) {
    if depth > 0 {
        recursive_process(depth - 1).await;
    }
}

// ✅ Use iteration instead
async fn iterative_process(depth: usize) {
    for i in (0..depth).rev() {
        process_level(i).await;
    }
}
```

**Rule:** Prefer iteration over recursion in async code.

### Gotcha 5: Slow `Drop` Implementation

**Problem:** Blocking on drop in async context.

**Cause:** `Drop::drop` is synchronous but does I/O.

**Example:**
```rust
// ❌ Blocking drop (writes to disk)
impl Drop for SessionState {
    fn drop(&mut self) {
        std::fs::write("session.json", &self.data).unwrap(); // BLOCKS
    }
}

// ✅ Explicit async cleanup
impl SessionState {
    async fn cleanup(&self) -> Result<()> {
        tokio::fs::write("session.json", &self.data).await?;
        Ok(())
    }
}

// Call cleanup explicitly before drop
session.cleanup().await?;
drop(session);
```

**Rule:** Never do I/O in `Drop`, use explicit async cleanup.

## Sources & Further Reading

**Primary (codebase):**
- [aof-tools/benchmark/mod.rs](file:///Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/benchmark/mod.rs) - Existing token efficiency benchmarking
- [Phase 7 Research](file:///Users/gshah/work/opsflow-sh/aof/.planning/phases/07-coordination-protocols/07-RESEARCH.md) - Coordination protocol token overhead analysis
- [Phase 1 Research](file:///Users/gshah/work/opsflow-sh/aof/.planning/phases/01-event-infrastructure/01-RESEARCH.md) - WebSocket architecture and tokio::broadcast patterns

**Rust Performance:**
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/) - Comprehensive guide to Rust optimization
- [Tokio Performance Guide](https://tokio.rs/tokio/topics/performance) - Async runtime best practices
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/) - Statistical benchmarking

**Load Testing:**
- [k6 Documentation](https://k6.io/docs/) - WebSocket and HTTP load testing
- [k6 WebSocket Guide](https://k6.io/docs/using-k6/protocols/websockets/) - Scripting WebSocket scenarios

**Profiling:**
- [tokio-console](https://github.com/tokio-rs/console) - Async runtime debugging
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU profiling for Rust

**Observability:**
- [Prometheus Rust Client](https://docs.rs/prometheus/latest/prometheus/) - Metrics collection
- [tracing](https://docs.rs/tracing/latest/tracing/) - Structured logging

**Research Papers:**
- [Understanding and Detecting Software Performance Antipatterns in Rust](https://arxiv.org/abs/2308.05098) - Academic study of Rust performance issues
- [Performance Characteristics of WebSockets](https://ieeexplore.ieee.org/document/6573738) - WebSocket scaling research

## Metadata

**Research date:** 2026-02-14
**Valid until:** 2026-03-14 (28 days - tooling stable, patterns evolve)
**Confidence:** HIGH

**Key uncertainties:**
- Exact performance baseline (requires running benchmarks on v0.4.0-beta)
- Real-world coordination overhead (Phase 7 not yet implemented)
- Optimal k6 scenario parameters (needs empirical tuning)
- CI resource limits (GitHub Actions runner specs)

**Next steps:**
1. Establish performance baselines (run benchmark suite on current codebase)
2. Implement Criterion micro-benchmarks for hot paths
3. Create k6 load test scenarios
4. Add tokio-console instrumentation
5. Set up Prometheus metrics endpoint
6. Integrate benchmarks into CI pipeline

---

## RESEARCH COMPLETE

Ready for Phase 8 planning. Research provides comprehensive technical foundation for:

**08-01-PLAN: Performance Baseline Establishment**
- Run Criterion benchmarks on v0.4.0-beta
- Establish latency/throughput baselines
- Document baseline metrics

**08-02-PLAN: Load Testing Infrastructure**
- Create k6 WebSocket scenarios
- Implement concurrent agent tests
- Set up CI integration

**08-03-PLAN: Profiling & Optimization**
- Add tokio-console instrumentation
- Implement flamegraph generation
- Identify and fix bottlenecks

**08-04-PLAN: Observability & Metrics**
- Add Prometheus metrics endpoint
- Implement structured logging
- Create Grafana dashboards

**08-05-PLAN: CI/CD Performance Gates**
- Integrate Criterion into CI
- Add k6 threshold checks
- Set up memory leak detection

**Success criteria:**
- 20 concurrent agents without degradation
- 50+ WebSocket clients with <100ms latency
- <30% coordination overhead
- No memory leaks detected
- Performance regression CI gates active
