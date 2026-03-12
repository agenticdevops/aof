# Production Infrastructure Integration Notes

## Task 5: Integrating Health/Metrics/Shutdown into serve.rs

### 1. Initialize Prometheus Metrics

```rust
use aof_runtime::AofMetrics;
use std::sync::Arc;

// In serve command setup
let metrics = Arc::new(AofMetrics::new()
    .map_err(|e| anyhow::anyhow!("Failed to create metrics: {}", e))?);

// Record startup
let start_time = std::time::Instant::now();
```

### 2. Initialize Structured JSON Logging

```rust
use tracing_subscriber::{fmt, EnvFilter};

// JSON logging for production (daemon mode)
if json_logs {
    let json_layer = fmt::layer()
        .json()
        .flatten_event(true)
        .with_current_span(false);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(json_layer)
        .init();
} else {
    // Human-readable for development
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
```

### 3. Register Health/Readiness/Metrics Routes

```rust
use axum::{Router, routing::get};
use aof_runtime::{check_readiness, HealthResponse};

// Handler functions
async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> axum::Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    axum::Json(HealthResponse::new(uptime))
}

async fn ready_handler(
    State(state): State<Arc<AppState>>,
) -> Result<axum::Json<ReadinessResponse>, StatusCode> {
    let response = check_readiness(
        &state.data_dir,
        &state.persist_dir,
        state.event_bus.subscriber_count(),
    ).await;

    if response.status == "ready" {
        Ok(axum::Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Update system metrics
    state.metrics.uptime_seconds.set(state.start_time.elapsed().as_secs() as f64);
    state.metrics.session_count.set(state.session_count() as f64);

    match state.metrics.render() {
        Ok(body) => Ok((
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4")],
            body
        )),
        Err(e) => {
            tracing::error!("Failed to render metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Add routes to router
let app = Router::new()
    .route("/health", get(health_handler))
    .route("/ready", get(ready_handler))
    .route("/metrics", get(metrics_handler))
    // ... existing routes
    .with_state(app_state);
```

### 4. Wire Graceful Shutdown

```rust
use aof_runtime::GracefulShutdown;

// Create shutdown handler
let shutdown = Arc::new(GracefulShutdown::new(Duration::from_secs(shutdown_timeout)));

// Use with axum serve
let server = axum::serve(listener, app)
    .with_graceful_shutdown(shutdown.wait_for_signal());

// After server stops, execute cleanup
shutdown.execute(Arc::new(app_state)).await?;
```

### 5. Instrument Existing Code Paths

```rust
// Agent execution
metrics.agent_executions_total
    .with_label_values(&[agent_id, "success"])
    .inc();
let timer = metrics.agent_execution_duration.start_timer();
// ... execute agent ...
timer.observe_duration();

// WebSocket connections
metrics.websocket_clients.inc(); // on connect
metrics.websocket_clients.dec(); // on disconnect
metrics.websocket_messages_sent_total.inc();

// LLM calls
metrics.llm_requests_total
    .with_label_values(&["anthropic", "claude-opus-4"])
    .inc();
metrics.llm_tokens_total
    .with_label_values(&["anthropic", "input"])
    .inc_by(input_tokens as f64);
```

### 6. CLI Flags

Add to aofctl serve:
- `--json-logs` - Enable JSON structured logging
- `--shutdown-timeout <seconds>` - Graceful shutdown timeout (default: 30)

## Next Steps

1. Modify serve.rs to initialize metrics, health, shutdown
2. Add route handlers
3. Instrument agent execution, WebSocket, LLM calls
4. Test endpoints with curl
5. Verify graceful shutdown with SIGTERM
