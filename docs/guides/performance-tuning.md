# Performance Tuning Guide

This guide helps you configure AOF for optimal performance in production deployments.

---

## System Requirements

### Minimum Requirements (5 agents)

- **CPU:** 2 cores
- **RAM:** 2GB
- **Disk:** 1GB
- **Network:** 10 Mbps

### Recommended Requirements (20 agents)

- **CPU:** 4 cores
- **RAM:** 4GB
- **Disk:** 5GB
- **Network:** 100 Mbps

### High-Performance (50+ agents)

- **CPU:** 8+ cores
- **RAM:** 8GB+
- **Disk:** 10GB+ (SSD recommended)
- **Network:** 1 Gbps

---

## Resource Allocation by Agent Count

AOF's resource usage scales sub-linearly with agent count due to efficient event broadcasting and coordination protocols.

### Recommended Allocations

| Agents | CPU Cores | RAM (GB) | Disk (GB) | WebSocket Clients |
|--------|-----------|----------|-----------|-------------------|
| 1-5    | 2         | 2        | 1         | 10                |
| 5-10   | 2-4       | 3        | 3         | 25                |
| 10-20  | 4         | 4        | 5         | 50                |
| 20-50  | 6-8       | 6-8      | 10        | 100               |
| 50+    | 8+        | 8+       | 20+       | 200+              |

### Memory Usage Breakdown

Approximate memory usage per component:

- **Base runtime:** ~100MB
- **Per agent:** ~20-50MB (depends on model context size)
- **EventBroadcaster:** ~5MB per 1000 buffered events
- **SessionPersistence:** ~1MB per saved session
- **CoordinationManager:** ~10MB for protocol state

**Example:** 20 agents with 50 WebSocket clients:
- Base: 100MB
- Agents: 20 × 40MB = 800MB
- EventBroadcaster: 50MB (10k event buffer)
- Coordination: 10MB
- **Total:** ~960MB (fits in 2GB with headroom)

---

## WebSocket Connection Scaling

### EventBroadcaster Configuration

The EventBroadcaster uses a `tokio::sync::broadcast` channel with configurable capacity.

**Default capacity:** 1000 events per subscriber

**Tuning:**

```rust
// In serve.rs or custom daemon
let broadcaster = EventBroadcaster::new(5000); // Increase to 5000 for high event volume
```

**Guidance:**
- **10 clients:** 1000 capacity (default)
- **50 clients:** 2000-5000 capacity
- **100+ clients:** 5000-10000 capacity

**Trade-off:** Higher capacity = more memory (each event ~500 bytes), but prevents lagged subscriber warnings.

### Connection Limits

**OS-level file descriptor limit:**

```bash
# Check current limit
ulimit -n

# Increase temporarily (macOS/Linux)
ulimit -n 4096

# Increase permanently (add to .bashrc or .zshrc)
echo "ulimit -n 4096" >> ~/.bashrc
```

**System-wide limit (Linux):**

```bash
# Edit /etc/sysctl.conf
fs.file-max = 100000

# Apply
sudo sysctl -p
```

**Recommended limits:**
- Development: 1024 (default usually fine)
- Production (50 clients): 2048
- Production (100+ clients): 4096+

---

## Coordination Overhead Monitoring

AOF tracks coordination protocol overhead (heartbeat, standup) as a percentage of total tokens.

### Overhead Budget

**Target:** <30% coordination overhead

**Measurement:**

```bash
# Enable coordination metrics logging
RUST_LOG=aof_coordination_protocols=info cargo run -- serve

# Check logs for overhead percentage
# Example output:
# [INFO] Coordination overhead: 18.4% (heartbeat: 12%, standup: 6%)
```

### Auto-Degradation

If overhead exceeds 30%, AOF automatically degrades coordination mode:

1. **Full** → **Standard** (disable standup)
2. **Standard** → **Reduced** (reduce heartbeat frequency)
3. **Reduced** → **HeartbeatOnly** (disable standup, minimal heartbeat)
4. **HeartbeatOnly** → **Disabled** (no coordination)

**Recovery:** When overhead drops below 20%, AOF auto-recovers in reverse order.

### Tuning Coordination

**Heartbeat configuration:**

```yaml
# coordination-config.yaml
heartbeat:
  interval_seconds: 30  # Increase to reduce overhead (default: 30)
  timeout_seconds: 90   # 3x interval recommended
  max_retries: 3
```

**Standup configuration:**

```yaml
standup:
  schedule: "0 9 * * MON-FRI"  # Daily standup schedule (cron format)
  timezone: "America/Los_Angeles"
```

**Disable coordination entirely:**

```yaml
coordination:
  enabled: false
```

---

## RUST_LOG Levels and Performance Impact

Logging has measurable performance impact. Choose log levels appropriate for your environment.

### Log Level Performance Impact

| Level | Overhead | Use Case |
|-------|----------|----------|
| `off` | 0% | Production (no debugging) |
| `error` | <1% | Production (errors only) |
| `warn` | ~2% | Production (default) |
| `info` | ~5% | Staging, troubleshooting |
| `debug` | ~10% | Development |
| `trace` | ~20% | Deep debugging only |

### Recommended Settings

**Production:**
```bash
RUST_LOG=error cargo run --release -- serve
```

**Staging:**
```bash
RUST_LOG=aof=info,warn cargo run --release -- serve
```

**Development:**
```bash
RUST_LOG=aof=debug cargo run -- serve
```

**Per-crate filtering:**
```bash
# Info for coordination, error for everything else
RUST_LOG=aof_coordination=info,aof_coordination_protocols=info,error
```

---

## When to Use --release Builds

**Always use `--release` builds in production.** The performance difference is dramatic.

### Performance Comparison

| Metric | Debug Build | Release Build | Speedup |
|--------|-------------|---------------|---------|
| Event serialization | 5.2µs | 1.2µs | 4.3x |
| 20 concurrent agents | 24s | 6.8s | 3.5x |
| WebSocket throughput | 95 events/s | 320 events/s | 3.4x |

**Release build benefits:**
- Compiler optimizations (inlining, loop unrolling)
- Dead code elimination
- LTO (Link-Time Optimization)
- Smaller binary size

**Build command:**
```bash
cargo build --release

# Binary at: target/release/aofctl
```

---

## /metrics Endpoint (Future: Phase 08-04)

AOF will expose a `/metrics` endpoint for Prometheus scraping in Phase 08-04 (Observability).

**Planned metrics:**
- `aof_events_emitted_total` - Total events emitted
- `aof_event_fanout_latency_seconds` - Event fanout latency histogram
- `aof_coordination_overhead_percent` - Current coordination overhead
- `aof_active_agents` - Number of active agents
- `aof_websocket_connections` - Active WebSocket connections

**Usage (after 08-04):**
```bash
curl http://localhost:8080/metrics
```

**Prometheus scrape config:**
```yaml
scrape_configs:
  - job_name: 'aof'
    static_configs:
      - targets: ['localhost:8080']
```

---

## Performance Baselines (v0.4.0-beta)

Use these baselines to validate your deployment performance.

### Micro-benchmark Baselines

| Benchmark | Expected Mean | Your Result | Status |
|-----------|---------------|-------------|--------|
| serialize_coordination_event | ~1.2µs | _____ | _____ |
| broadcast_50_subscribers (1000 events) | ~18ms | _____ | _____ |
| record_10000_token_events | ~1.1ms | _____ | _____ |

**Run benchmarks:**
```bash
cargo bench
open target/criterion/report/index.html
```

### Integration Test Baselines

| Test | Expected | Your Result | Status |
|------|----------|-------------|--------|
| 20 concurrent agents | <10s | _____ | _____ |
| P95 event fanout latency | <100ms | _____ | _____ |

**Run tests:**
```bash
cargo test --test perf_concurrent_agents --release -- --nocapture
```

### Load Test Baselines (k6)

| Test | VUs | Expected P95 Latency | Your Result | Status |
|------|-----|----------------------|-------------|--------|
| websocket_broadcast | 10 | <100ms | _____ | _____ |
| 50_websocket_clients | 50 | <100ms | _____ | _____ |
| spike_100_clients | 100 | <200ms | _____ | _____ |

**Run load tests:**
```bash
# Start daemon
cargo run --release -- serve --port 8080

# In another terminal
cd tests/load
k6 run 50_websocket_clients.js
```

---

## Troubleshooting Performance Issues

### High Memory Usage

**Symptom:** Memory usage grows over time

**Diagnosis:**
```bash
# Check for memory leaks with valgrind (Linux)
cargo build
valgrind --leak-check=full ./target/debug/aofctl serve

# Or use cargo-instruments (macOS)
cargo instruments -t Allocations -- serve
```

**Common causes:**
- EventBroadcaster buffer too large (reduce capacity)
- Session persistence not cleaning up old sessions
- Agent context accumulation (restart periodically)

**Fix:**
- Reduce EventBroadcaster capacity
- Implement session TTL cleanup
- Monitor with `/metrics` endpoint (Phase 08-04)

---

### High CPU Usage

**Symptom:** CPU usage >80% sustained

**Diagnosis:**
```bash
# CPU profiling with flamegraph
cargo install flamegraph
sudo cargo flamegraph -- serve
```

**Common causes:**
- Too many agents on single instance
- Debug build in production
- Excessive logging (RUST_LOG=trace)
- Coordination overhead >30%

**Fix:**
- Horizontal scaling (multiple AOF instances)
- Use `--release` build
- Reduce RUST_LOG to `error` or `warn`
- Increase heartbeat interval

---

### High Latency

**Symptom:** P95 event latency >100ms

**Diagnosis:**
```bash
# Run k6 load test with detailed output
k6 run --out json=results.json tests/load/50_websocket_clients.js

# Check latency distribution
cat results.json | jq '.metrics.event_latency_ms'
```

**Common causes:**
- Network issues (check ping times)
- EventBroadcaster channel lagged (subscribers too slow)
- CPU throttling (check system load)

**Fix:**
- Increase EventBroadcaster capacity
- Reduce subscriber count per instance
- Use dedicated network interface
- Increase CPU allocation

---

## Scaling Strategies

### Vertical Scaling (Single Instance)

**When:** <50 agents, <100 WebSocket clients

**Approach:**
- Increase CPU cores (4 → 8)
- Increase RAM (4GB → 8GB)
- Use SSD for session persistence

**Limits:** ~50 agents, ~100 WebSocket clients per instance

---

### Horizontal Scaling (Multiple Instances)

**When:** >50 agents, >100 WebSocket clients

**Approach:**
- Deploy multiple AOF instances
- Load balancer for WebSocket connections
- Shared session persistence (Redis or S3)

**Example architecture:**
```
        ┌─────────────────┐
        │  Load Balancer  │
        └────────┬────────┘
                 │
       ┌─────────┴─────────┐
       │                   │
  ┌────▼────┐         ┌────▼────┐
  │ AOF #1  │         │ AOF #2  │
  │ 25 agents│        │ 25 agents│
  └────┬────┘         └────┬────┘
       │                   │
       └─────────┬─────────┘
                 │
           ┌─────▼─────┐
           │   Redis   │
           │ (Sessions)│
           └───────────┘
```

**Configuration:**
```yaml
# Each instance
session_persistence:
  backend: redis
  redis_url: "redis://localhost:6379"
```

---

## Production Checklist

Before deploying AOF to production, verify:

- [ ] Using `--release` build
- [ ] RUST_LOG set to `error` or `warn`
- [ ] Coordination overhead <30%
- [ ] Sufficient file descriptor limit (`ulimit -n 4096+`)
- [ ] EventBroadcaster capacity appropriate for WebSocket client count
- [ ] Performance baselines validated (benchmarks, integration tests, k6)
- [ ] Monitoring configured (Phase 08-04 metrics endpoint)
- [ ] Backup strategy for session persistence

---

## Getting Help

If you're experiencing performance issues not covered in this guide:

1. Run performance tests to identify bottleneck layer (micro/integration/load)
2. Check logs for coordination overhead warnings
3. Profile with tokio-console or cargo-flamegraph
4. Open GitHub issue with:
   - Performance test results
   - System specs (CPU, RAM, OS)
   - AOF version
   - Configuration files

---

## References

- [Performance Testing Guide (Internal)](../dev/performance-testing.md)
- [Criterion Benchmarks](../../crates/aof-core/benches/)
- [Integration Tests](../../tests/perf_*.rs)
- [k6 Load Tests](../../tests/load/)
