# Performance Testing Guide (Internal Developer Documentation)

This guide explains AOF's 3-tier performance testing pyramid, how to run tests locally, interpret results, and add new benchmarks.

---

## Performance Testing Pyramid

AOF uses a 3-tier testing approach to ensure production performance targets are met:

```
                   ┌─────────────────────┐
                   │   k6 Load Tests     │  ← System-level, 50-100 WebSocket clients
                   │   (E2E, Production) │
                   └─────────────────────┘
                  ┌─────────────────────────┐
                  │ Integration Perf Tests  │  ← 20 concurrent agents, latency checks
                  │  (Realistic Scenarios)  │
                  └─────────────────────────┘
          ┌───────────────────────────────────────┐
          │    Criterion Micro-benchmarks         │  ← Hot paths, serialization, overhead
          │  (Tight Loops, Statistical Analysis)  │
          └───────────────────────────────────────┘
```

### Tier 1: Criterion Micro-benchmarks (Fastest Feedback)

**Purpose:** Measure performance of individual hot paths with statistical rigor.

**Location:** `crates/*/benches/`

**Benchmarks:**
- `event_serialization.rs` - CoordinationEvent JSON serialization/deserialization, clone overhead
- `broadcaster_throughput.rs` - EventBroadcaster fanout to 1-50 subscribers
- `coordination_overhead.rs` - Token accounting, overhead calculation, health snapshots

**Run:**
```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench event_serialization

# Quick test mode (no statistical analysis)
cargo bench --bench broadcaster_throughput -- --test
```

**Output:** HTML reports in `target/criterion/` with:
- Mean time, standard deviation, outliers
- Comparison against previous runs (if baseline exists)
- Statistical analysis (p-values, confidence intervals)

---

### Tier 2: Integration Performance Tests (Realistic Scenarios)

**Purpose:** Validate production performance targets in realistic multi-agent scenarios.

**Location:** `tests/perf_*.rs`

**Tests:**
- `perf_baseline_single_agent.rs` - Event emission latency (<50ms p95), session persistence roundtrip
- `perf_concurrent_agents.rs` - 20 concurrent agents (<10s total), throughput scaling
- `perf_memory_stability.rs` - Memory leak detection (10k events, session churn) [marked `#[ignore]`]

**Run:**
```bash
# All performance tests
cargo test --test perf_baseline_single_agent --release
cargo test --test perf_concurrent_agents --release

# Memory stability (long-running, ignored by default)
cargo test --test perf_memory_stability --release -- --ignored --nocapture
```

**Assertions:**
- 20 concurrent agents complete in <10 seconds
- P95 event fanout latency <100ms
- Memory growth <10MB over 10k events

---

### Tier 3: k6 Load Tests (End-to-End System)

**Purpose:** Validate WebSocket scalability and resilience under production-like load.

**Location:** `tests/load/*.js`

**Tests:**
- `websocket_broadcast.js` - 10 VUs baseline (>100 events, <100ms p95)
- `50_websocket_clients.js` - Staged ramp to 50 clients (production target)
- `spike_100_clients.js` - Spike traffic resilience test

**Prerequisites:**
```bash
# Install k6
brew install k6  # macOS
snap install k6  # Linux

# Start AOF daemon
cargo run --release -- serve --port 8080
```

**Run:**
```bash
cd tests/load
k6 run 50_websocket_clients.js
```

**Output:** Real-time metrics (events/sec, latency percentiles, connection errors) with pass/fail thresholds.

---

## How to Interpret Criterion HTML Reports

Criterion generates detailed HTML reports in `target/criterion/{benchmark_name}/report/index.html`.

### Key Metrics

**Mean Time:**
- Average execution time across all samples
- Look for: Consistent mean across runs (low variance)

**Standard Deviation:**
- Measure of timing consistency
- Look for: <10% of mean (low jitter)

**Outliers:**
- Samples that fall outside normal distribution
- Look for: <5% mild outliers, 0% severe outliers

**Slope (for parameterized benchmarks):**
- How performance scales with input size
- Look for: Sub-linear growth (O(log n) or O(1) ideal)

### Regression Detection

Criterion compares against previous baseline and reports:
- **p-value:** Probability that difference is random noise (lower = more confident in change)
- **Change estimate:** Percentage change from baseline
- **Regression detected:** If p-value < 0.1 (10% significance level)

**Example output:**
```
event_serialization/serialize_coordination_event
                        time:   [1.2450 µs 1.2523 µs 1.2599 µs]
                        change: [-2.5437% -1.4294% -0.2734%] (p = 0.02 < 0.10)
                        Performance has improved.
```

- Mean time: 1.25µs
- Change: -1.43% (faster)
- p-value: 0.02 (2% chance this is noise - confident improvement!)

---

## Using tokio-console for Async Profiling

tokio-console provides real-time visibility into async runtime behavior.

### Setup

1. **Build with tokio-console feature:**
   ```bash
   RUSTFLAGS="--cfg tokio_unstable" cargo build --features tokio-console
   ```

2. **Run aofctl serve:**
   ```bash
   RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio-console -- serve
   ```

3. **Launch tokio-console (in another terminal):**
   ```bash
   tokio-console
   ```

### Console Interface

**Tasks view:**
- CPU time per task
- Poll time (time spent in Future::poll)
- Idle time (waiting on I/O, locks)
- Task state (Running, Idle, Completed)

**Resources view:**
- Async resources (channels, mutexes, semaphores)
- Wait counts, total wait time
- Contention metrics for locks

**Useful metrics:**
- High poll time → CPU-bound task (consider `spawn_blocking`)
- High idle time → I/O-bound (expected for network tasks)
- Many wakers → potential thundering herd
- Long-lived blocking → deadlock risk

### Example: Diagnosing Slow WebSocket

1. Connect tokio-console
2. Sort tasks by "Total Time" (descending)
3. Look for WebSocket handler tasks with high poll time
4. Switch to Resources view
5. Check for contention on broadcast channel

**Common issues:**
- Broadcaster channel full → increase capacity
- Mutex contention → consider lock-free alternatives (dashmap)
- Blocking I/O in async context → use `spawn_blocking`

---

## Using cargo-flamegraph for CPU Profiling

cargo-flamegraph generates visual flame graphs showing CPU hotspots.

### Setup (macOS)

```bash
cargo install flamegraph
sudo dtrace -n 'profile-997 /pid == $target/ { @[ustack(100)] = count(); }' -p <PID>
```

### Run

```bash
# Profile the serve command
cargo flamegraph -- serve --port 8080

# Let it run for 30-60 seconds under load (run k6 test)
# Then Ctrl+C

# Open flamegraph.svg in browser
open flamegraph.svg
```

### Interpreting Flame Graphs

- **Width:** Percentage of CPU time
- **Height:** Call stack depth (top = leaf functions)
- **Color:** Random (for visual grouping only)

**What to look for:**
- Wide bars = CPU hotspots
- `serde_json::ser` wide → serialization bottleneck
- `tokio::runtime::scheduler` wide → task scheduling overhead
- Unexpected wide bars → investigate function

---

## Performance Baseline Table (v0.4.0-beta)

Measured on: MacBook Pro M1 Max (10 cores, 32GB RAM), AOF v0.4.0-beta

### Criterion Micro-benchmarks

| Benchmark | Mean | Std Dev | Threshold |
|-----------|------|---------|-----------|
| serialize_coordination_event | 1.25µs | 0.04µs | <2µs |
| deserialize_coordination_event | 2.10µs | 0.08µs | <5µs |
| broadcast_1000_events_1_subscriber | 3.2ms | 0.2ms | <5ms |
| broadcast_1000_events_50_subscribers | 18ms | 1.5ms | <30ms |
| record_10000_token_events | 1.1ms | 0.05ms | <2ms |

### Integration Performance Tests

| Test | Metric | Value | Threshold |
|------|--------|-------|-----------|
| test_20_concurrent_agents | Total time | 6.8s | <10s |
| test_20_concurrent_agents | P95 fanout latency | 78ms | <100ms |
| test_agent_throughput_scaling | 20-agent throughput | 2900 events/s | >5x single-agent |

### k6 Load Tests

| Test | VUs | Events/sec | P95 Latency | P95 Connection |
|------|-----|------------|-------------|----------------|
| websocket_broadcast.js | 10 | 75/s | 45ms | 210ms |
| 50_websocket_clients.js | 50 | 320/s | 82ms | 380ms |
| spike_100_clients.js | 100 | 580/s | 145ms | 720ms |

---

## How CI Regression Detection Works

The `.github/workflows/performance.yml` workflow runs on every PR and main branch push.

### Workflow Jobs

1. **micro-benchmarks** (PR + main):
   - Runs all Criterion benchmarks
   - On main: saves baseline with `--save-baseline main`
   - On PR: compares against main baseline
   - **Fails if:** Criterion detects >10% regression (p < 0.1)

2. **integration-performance** (main only):
   - Runs integration perf tests in release mode
   - **Fails if:** Assertions not met (20 agents >10s, p95 >100ms)

3. **regression-check** (always):
   - Aggregates results and reports status

### Failure Scenarios

**Criterion regression:**
```
Error: Benchmark event_serialization/serialize_coordination_event has regressed
  Previous: 1.25µs
  Current:  1.52µs
  Change:   +21.6% (p = 0.003)
  Threshold: 10% significance level
```

**Integration test failure:**
```
thread 'test_20_concurrent_agents' panicked at 'assertion failed:
  Total execution time 12.4s exceeds 10s threshold'
```

---

## Adding New Benchmarks

### Criterion Micro-benchmark

1. **Create benchmark file:**
   ```rust
   // crates/my-crate/benches/my_benchmark.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn bench_my_function(c: &mut Criterion) {
       c.bench_function("my_function", |b| {
           b.iter(|| {
               my_function(black_box(42));
           });
       });
   }

   criterion_group! {
       name = my_benchmarks;
       config = Criterion::default().sample_size(100).significance_level(0.1);
       targets = bench_my_function
   }

   criterion_main!(my_benchmarks);
   ```

2. **Add to Cargo.toml:**
   ```toml
   [[bench]]
   name = "my_benchmark"
   harness = false
   ```

3. **Run and verify:**
   ```bash
   cargo bench --bench my_benchmark
   ```

### Integration Performance Test

1. **Create test file:**
   ```rust
   // tests/perf_my_feature.rs
   use std::time::{Duration, Instant};

   #[tokio::test]
   async fn test_my_feature_performance() {
       let start = Instant::now();

       // ... test implementation ...

       let elapsed = start.elapsed();
       assert!(elapsed < Duration::from_secs(5),
           "Test took {:?}, threshold is 5s", elapsed);
   }
   ```

2. **Run:**
   ```bash
   cargo test --test perf_my_feature --release -- --nocapture
   ```

3. **Add to CI workflow** if appropriate

---

## Troubleshooting

### Benchmark results inconsistent

**Cause:** CPU throttling, background processes

**Fix:**
- Close other applications
- Run on AC power (not battery)
- Use `cargo bench --bench <name> -- --warm-up-time 5` for longer warm-up

### Integration test timeout

**Cause:** Debug build (slower), insufficient resources

**Fix:**
- Always use `--release` for performance tests
- Increase timeout threshold temporarily to diagnose
- Check for deadlocks with `RUST_LOG=debug`

### k6 connection failures

**Cause:** Daemon not running, port conflict, ulimit too low

**Fix:**
```bash
# Check daemon is running
lsof -i :8080

# Increase file descriptor limit
ulimit -n 4096

# Use different port
aofctl serve --port 9000
k6 run -e PORT=9000 tests/load/50_websocket_clients.js
```

---

## Best Practices

1. **Always run benchmarks in release mode** (except Criterion, which handles this)
2. **Establish baselines early** - first benchmark run sets the reference
3. **Run benchmarks consistently** - same machine, same conditions
4. **Don't over-optimize** - focus on hot paths identified by profiling
5. **Document performance changes** - if you regress intentionally, explain why
6. **Use statistical significance** - don't react to noise (<10% change, p>0.1)

---

## References

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [tokio-console Guide](https://tokio.rs/tokio/topics/tracing-next-steps)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [k6 Documentation](https://k6.io/docs/)
