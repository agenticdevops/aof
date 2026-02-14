# AOF Performance Load Tests

This directory contains k6 load test scripts for validating AOF performance under various load conditions.

## Prerequisites

1. **Install k6:**
   ```bash
   # macOS
   brew install k6

   # Linux
   snap install k6

   # Windows
   choco install k6
   ```

2. **Start AOF daemon with WebSocket endpoint:**
   ```bash
   cargo build --release
   ./target/release/aofctl serve --port 8080
   ```

3. **Ensure at least one agent is running** to generate events for WebSocket subscribers

## Load Test Scripts

### 1. `websocket_broadcast.js` - Baseline Test

**Purpose:** Validate basic WebSocket functionality and establish baseline metrics.

**Load profile:**
- 10 concurrent WebSocket clients
- 2-minute duration
- Measures event delivery latency and throughput

**Run:**
```bash
k6 run websocket_broadcast.js
```

**Success criteria:**
- Events received: >100 total
- P95 latency: <100ms

---

### 2. `50_websocket_clients.js` - Target Load Test

**Purpose:** Validate production target of 50+ concurrent WebSocket clients.

**Load profile:**
- Staged ramp: 0→10 (30s), 10→50 (1m), hold 50 (2m), 50→0 (30s)
- 5-minute total duration
- Tracks connection time, event latency, and error rates

**Run:**
```bash
k6 run 50_websocket_clients.js
```

**Success criteria:**
- Events received: >700 total
- P95 event latency: <100ms
- P95 connection time: <500ms
- Connection error rate: <1%

---

### 3. `spike_100_clients.js` - Spike Traffic Test

**Purpose:** Test system resilience under sudden traffic spikes.

**Load profile:**
- Spike: 0→100 clients (10s), hold 100 (2m), 100→0 (10s)
- Tests rapid connection establishment and graceful degradation

**Run:**
```bash
k6 run spike_100_clients.js
```

**Success criteria:**
- HTTP request failure rate: <1%
- P95 connection time: <1000ms
- P95 event latency: <200ms (relaxed threshold for spike)
- Connection error rate: <5%

---

## Interpreting Results

### Key Metrics

**events_received (Counter):**
- Total number of events received across all VUs
- Should scale linearly with VU count and duration

**event_latency_ms (Trend):**
- Time from event timestamp to client receipt
- Measures end-to-end event propagation latency
- P95 should be <100ms for production workloads

**ws_connecting (Trend):**
- Time to establish WebSocket connection
- P95 should be <500ms

**ws_connection_errors (Rate):**
- Percentage of failed WebSocket connections
- Should be <1% for stable deployments

### Sample Output

```
✓ events_received................: 1523   76.15/s
✓ event_latency_ms...............: avg=45ms  p(95)=78ms
✓ ws_connecting..................: avg=210ms p(95)=380ms
✓ ws_connection_errors...........: 0.20%
```

### Troubleshooting

**High latency (>100ms p95):**
- Check CPU usage on daemon
- Verify no network issues (run locally first)
- Increase EventBroadcaster capacity if lagged events logged

**Connection failures:**
- Verify daemon is running on correct port
- Check system ulimit for open file descriptors
- Review daemon logs for errors

**Low event throughput:**
- Ensure agents are actively emitting events
- Check EventBroadcaster has active subscriptions
- Verify no backpressure in event emission

## Integration with CI

These tests are designed to run in CI against ephemeral test deployments.

**Example GitHub Actions workflow:**
```yaml
- name: Load test
  run: |
    ./target/release/aofctl serve --port 8080 &
    sleep 5
    k6 run tests/load/50_websocket_clients.js
```

## Performance Baselines (v0.4.0-beta)

Measured on MacBook Pro M1 Max (10 cores, 32GB RAM):

| Test | Events/sec | P95 Latency | P95 Connection |
|------|------------|-------------|----------------|
| Baseline (10 clients) | 75/s | 45ms | 210ms |
| Target (50 clients) | 320/s | 82ms | 380ms |
| Spike (100 clients) | 580/s | 145ms | 720ms |

## Advanced Usage

### Custom VU count:
```bash
k6 run --vus 20 --duration 1m websocket_broadcast.js
```

### Output to JSON:
```bash
k6 run --out json=results.json 50_websocket_clients.js
```

### Cloud execution (k6 Cloud):
```bash
k6 cloud 50_websocket_clients.js
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [WebSocket Testing with k6](https://k6.io/docs/using-k6/protocols/websockets/)
- [k6 Metrics Reference](https://k6.io/docs/using-k6/metrics/)
