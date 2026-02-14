# Observability Architecture (Internal Developer Docs)

This document describes AOF's production observability infrastructure for contributors adding new features.

## Prometheus Metrics

### Metrics Registry (`aof-runtime/src/metrics.rs`)

Centralized `AofMetrics` struct tracks all subsystem metrics:

```rust
pub struct AofMetrics {
    pub registry: Registry,

    // Agent metrics
    pub agent_executions_total: CounterVec,      // Labels: agent_id, status
    pub agent_execution_duration: Histogram,      // Buckets: 0.5, 1, 2, 5, 10, 30, 60s
    pub agents_active: Gauge,

    // Event metrics
    pub events_emitted_total: Counter,
    pub event_broadcast_latency: Histogram,       // Buckets: 0.0001 to 0.1s

    // WebSocket metrics
    pub websocket_clients: Gauge,
    pub websocket_messages_sent_total: Counter,
    pub websocket_messages_failed_total: Counter,

    // LLM metrics
    pub llm_requests_total: CounterVec,           // Labels: provider, model
    pub llm_tokens_total: CounterVec,             // Labels: provider, type (input/output)
    pub llm_latency: Histogram,

    // Coordination metrics
    pub heartbeat_checks_total: Counter,
    pub heartbeat_failures_total: Counter,
    pub coordination_overhead_percent: Gauge,

    // System metrics
    pub uptime_seconds: Gauge,
    pub session_count: Gauge,
}
```

### Naming Conventions

All AOF metrics follow Prometheus best practices:

1. **Prefix**: All metrics start with `aof_`
2. **Counters**: End with `_total` (e.g., `aof_agent_executions_total`)
3. **Histograms**: End with unit (e.g., `aof_agent_execution_duration_seconds`)
4. **Gauges**: Describe current state (e.g., `aof_agents_active`)
5. **Labels**: Use snake_case (e.g., `agent_id`, `status`)

### Histogram Bucket Design

Chosen for AOF's operational characteristics:

- **Agent execution**: `[0.5, 1, 2, 5, 10, 30, 60]` seconds
  - Agents typically complete in 1-5s (tool calls)
  - Long-running agents: 30-60s
  - Captures p50, p95, p99 accurately

- **Event broadcast**: `[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]` seconds
  - In-process broadcast is sub-millisecond
  - Network broadcast to WebSocket clients: 1-10ms
  - Alerts if >50ms (network congestion)

- **LLM latency**: `[0.5, 1, 2, 5, 10, 30, 60]` seconds
  - Claude API typically responds in 1-3s
  - Long prompts or rate limits: 5-10s
  - Timeout after 60s

### Adding New Metrics

**Step 1: Add to AofMetrics struct**

```rust
// In crates/aof-runtime/src/metrics.rs
pub struct AofMetrics {
    // ... existing metrics ...

    /// Total database queries
    pub db_queries_total: CounterVec,  // Labels: operation, status
}
```

**Step 2: Register in `new()` method**

```rust
let db_queries_total = CounterVec::new(
    Opts::new(
        "aof_db_queries_total",
        "Total database queries"
    ),
    &["operation", "status"]  // operation: select/insert/update, status: success/error
)?;
registry.register(Box::new(db_queries_total.clone()))?;
```

**Step 3: Return in struct construction**

```rust
Ok(Self {
    registry,
    // ... existing fields ...
    db_queries_total,
})
```

**Step 4: Instrument code**

```rust
// In your database layer
metrics.db_queries_total
    .with_label_values(&["select", "success"])
    .inc();
```

### Structured Logging

AOF uses `tracing` with JSON output for production:

```rust
use tracing::{info, warn, error};

// Always include contextual fields
info!(
    agent_id = %agent.id,
    execution_id = %execution_id,
    duration_ms = duration.as_millis(),
    "Agent execution completed"
);

// Warning with reason
warn!(
    tool_name = %tool,
    error = %e,
    "Tool execution failed, retrying"
);

// Error with full context
error!(
    agent_id = %agent.id,
    task_id = %task_id,
    error = %e,
    "Agent execution failed after retries"
);
```

### Field Conventions

Consistent field names across codebase:

- `agent_id` - Agent identifier (string)
- `execution_id` - Execution UUID (string)
- `task_id` - Task identifier (string)
- `duration_ms` - Duration in milliseconds (u64)
- `duration_secs` - Duration in seconds (f64)
- `status` - Outcome: success, error, timeout
- `tool_name` - Tool identifier
- `provider` - LLM provider: anthropic, openai, etc.
- `model` - Model name: claude-opus-4, gpt-4, etc.
- `error` - Error message (string)

### Health Check Architecture

Two separate endpoints for different purposes:

#### `/health` - Liveness Probe

Returns 200 if process is alive, never returns error:

```json
{
  "status": "ok",
  "version": "0.4.0-beta",
  "uptime_seconds": 3600,
  "git_commit": "abc123def"
}
```

**Use case**: Kubernetes liveness probe. If this fails, kill the pod.

**Implementation**: `crates/aof-runtime/src/health.rs:HealthResponse`

#### `/ready` - Readiness Probe

Returns 200 if ready to serve traffic, 503 if not:

```json
{
  "status": "ready",  // or "not_ready"
  "dependencies": {
    "disk_space": { "status": "ok" },
    "event_bus": { "status": "ok" },
    "session_persistence": { "status": "ok" }
  }
}
```

Returns 503 if any dependency is unavailable:

```json
{
  "status": "not_ready",
  "dependencies": {
    "disk_space": { "status": "unavailable", "reason": "Critically low disk space: 5 MB available" },
    "event_bus": { "status": "degraded", "reason": "No active event subscribers" },
    "session_persistence": { "status": "ok" }
  }
}
```

**Use case**: Kubernetes readiness probe. If this fails, remove from load balancer but don't kill.

**Checks performed**:
1. **Disk space**: >100MB required, 10-100MB degraded, <10MB unavailable
2. **Event bus**: Functional if subscribers exist, degraded if zero subscribers
3. **Session persistence**: Directory writable

**Implementation**: `crates/aof-runtime/src/health.rs:check_readiness()`

### Graceful Shutdown

Shutdown sequence is critical for preventing data loss:

```rust
// 1. Signal received (SIGTERM or SIGINT)
info!("Received SIGTERM, starting graceful shutdown");

// 2. Stop accepting new connections
// (Axum handles this automatically via with_graceful_shutdown)

// 3. Broadcast shutdown signal to all components
let subscriber_count = shutdown_tx.send(()).unwrap_or(0);
info!(subscribers = subscriber_count, "Broadcasting shutdown signal");

// 4. WebSocket connections drain (send close frames)
for ws in active_connections {
    ws.send(CloseFrame { code: 1001, reason: "Server shutting down" }).await;
}

// 5. Save session state
session_persistence.save_session(&final_state).await?;
info!("Session state saved");

// 6. Flush pending logs
// (tracing-subscriber handles this automatically)

// 7. Finalize metrics
info!("Graceful shutdown complete");

// 8. Exit (or timeout after 30s and force exit)
```

**Implementation**: `crates/aof-runtime/src/shutdown.rs:GracefulShutdown`

**Customization**: Implement `ShutdownHandler` trait for your state:

```rust
#[async_trait]
impl ShutdownHandler for AppState {
    async fn shutdown(&self) -> AofResult<()> {
        // Close database connections
        self.db_pool.close().await?;

        // Save coordinator state
        self.coordinator.save_state().await?;

        // Close gateway hub
        self.gateway.shutdown().await?;

        Ok(())
    }
}
```

## Monitoring Queries

### Prometheus Queries

**Agent execution rate (requests per second)**:
```promql
rate(aof_agent_executions_total[5m])
```

**Agent error rate**:
```promql
rate(aof_agent_executions_total{status="error"}[5m])
 / rate(aof_agent_executions_total[5m])
```

**Agent execution p95 latency**:
```promql
histogram_quantile(0.95, rate(aof_agent_execution_duration_seconds_bucket[5m]))
```

**Active agents**:
```promql
aof_agents_active
```

**WebSocket connection churn**:
```promql
rate(aof_websocket_clients[5m])
```

**LLM token consumption (input tokens per minute)**:
```promql
rate(aof_llm_tokens_total{type="input"}[1m]) * 60
```

**Coordination overhead**:
```promql
aof_coordination_overhead_percent
```

### Grafana Dashboard

Key panels for AOF dashboard:

1. **System Health**
   - Uptime gauge
   - Active agents gauge
   - WebSocket clients gauge
   - Session count gauge

2. **Agent Execution**
   - Execution rate graph
   - Error rate graph
   - P50/P95/P99 latency graph
   - Status breakdown (success/error/timeout) pie chart

3. **LLM Usage**
   - Request rate by provider
   - Token consumption by type (input/output)
   - Latency histogram
   - Cost estimation (tokens × provider cost)

4. **Coordination**
   - Heartbeat check rate
   - Heartbeat failure rate
   - Overhead percentage gauge
   - Standup frequency

5. **Infrastructure**
   - Disk space gauge
   - Event broadcast latency histogram
   - WebSocket message send rate
   - WebSocket message failure rate

### Alerting Rules

**Critical alerts**:

```yaml
groups:
- name: aof-critical
  interval: 30s
  rules:
  - alert: AofDaemonDown
    expr: up{job="aof-daemon"} == 0
    for: 1m
    annotations:
      summary: "AOF daemon is down"

  - alert: AofHighErrorRate
    expr: rate(aof_agent_executions_total{status="error"}[5m]) / rate(aof_agent_executions_total[5m]) > 0.1
    for: 5m
    annotations:
      summary: "AOF agent error rate >10%"

  - alert: AofHighLatency
    expr: histogram_quantile(0.95, rate(aof_agent_execution_duration_seconds_bucket[5m])) > 30
    for: 5m
    annotations:
      summary: "AOF agent p95 latency >30s"

  - alert: AofDiskSpaceLow
    expr: aof_disk_space_available_bytes < 100000000  # 100MB
    for: 5m
    annotations:
      summary: "AOF disk space <100MB"
```

**Warning alerts**:

```yaml
- alert: AofCoordinationOverhead
  expr: aof_coordination_overhead_percent > 30
  for: 10m
  annotations:
    summary: "AOF coordination overhead >30%"

- alert: AofHeartbeatFailures
  expr: rate(aof_heartbeat_failures_total[5m]) > 0
  for: 5m
  annotations:
    summary: "AOF heartbeat failures detected"
```

## Testing Observability

### Local Testing

```bash
# Start daemon with metrics enabled
cargo run -- serve --port 8080

# Check health endpoint
curl http://localhost:8080/health

# Check readiness endpoint
curl http://localhost:8080/ready

# Scrape metrics
curl http://localhost:8080/metrics

# Run agent and observe metrics update
cargo run -- run agent hello-world --input "test"

# Check metrics again
curl http://localhost:8080/metrics | grep aof_agent
```

### Integration Testing

See `crates/aofctl/tests/metrics_integration_test.rs` for examples of:
- Verifying metric updates after agent execution
- Checking histogram bucket distribution
- Validating label values
- Testing health/readiness probe behavior

## References

- [Prometheus naming conventions](https://prometheus.io/docs/practices/naming/)
- [Histogram bucket design guide](https://prometheus.io/docs/practices/histograms/)
- [Grafana dashboard best practices](https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/)
