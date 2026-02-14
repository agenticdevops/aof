# SRE Hardening - Internal Developer Guide

This guide covers how resilience patterns are implemented in AOF and how to extend them.

## Resilience Patterns Overview

AOF implements five core resilience patterns in `crates/aof-runtime/src/resilience/`:

1. **Circuit Breaker** - Prevents cascading failures to external services
2. **Bulkhead** - Resource isolation and capacity limits
3. **Retry** - Exponential backoff for transient failures
4. **Supervisor** - Agent crash recovery
5. **Degradation** - Adaptive system behavior under pressure

## Adding Circuit Breaker Protection

To protect a new external service call:

```rust
use aof_runtime::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

// 1. Create circuit breaker
let config = CircuitBreakerConfig {
    failure_threshold: 5,
    success_threshold: 3,
    timeout: Duration::from_secs(30),
    name: "my-service".to_string(),
};
let breaker = CircuitBreaker::new(config);

// 2. Wrap your service call
let result = breaker.call(async {
    // Your external service call here
    my_service.call().await
}).await;

// 3. Handle circuit breaker errors
match result {
    Ok(response) => {
        // Success
    }
    Err(CircuitBreakerError::Open) => {
        // Circuit is open, use fallback
    }
    Err(CircuitBreakerError::Inner(err)) => {
        // Service call failed
    }
}
```

**When to use:**
- External API calls (LLM providers, MCP servers)
- Database queries
- Network operations
- Any dependency that can fail

## Configuring Bulkhead Limits

Current limits (see `BulkheadConfig`):
- `max_concurrent_agents`: 20 (default)
- `max_event_queue_size`: 1000
- `max_websocket_clients`: 100

**Tuning guidance:**
- Set to 60-80% of system capacity
- Monitor utilization metrics
- Adjust based on resource availability

**Adding new bulkhead:**

```rust
use aof_runtime::resilience::bulkhead::{Bulkhead, BulkheadConfig};

let config = BulkheadConfig {
    max_concurrent_agents: 50,  // Higher for more powerful servers
    ..Default::default()
};
let bulkhead = Arc::new(Bulkhead::new(config));

// Acquire slot before starting work
let permit = bulkhead.acquire_agent_slot().await?;

// Do work...
// Permit automatically released on drop
```

## DegradationEngine Integration Points

The degradation engine monitors system health and takes automatic actions.

**Current integration:**
- Runs in background task (see `serve.rs`)
- Checks every 10 seconds
- Monitors: memory, CPU, agent capacity, queue depth

**Adding new health check:**

Edit `crates/aof-runtime/src/resilience/degradation.rs`:

```rust
impl DegradationEngine {
    async fn assess_health(&self, bulkhead: &Bulkhead, metrics: &AofMetrics) {
        // Existing checks...

        // Add your check
        let my_metric = self.get_my_metric();
        if my_metric >= self.thresholds.my_threshold {
            degraded_components.push(DegradedComponent::MyComponentHigh);
        }
    }
}
```

**Adding degradation action:**

```rust
pub async fn apply_degradation(&self, health: &SystemHealth) {
    match health {
        SystemHealth::Degraded(components) => {
            for component in components {
                match component {
                    DegradedComponent::MyComponentHigh => {
                        // Your mitigation action
                        tracing::warn!("Applying mitigation for MyComponent");
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
```

## Chaos Test Infrastructure

Chaos tests are located in `tests/chaos_*.rs`.

**Adding a new chaos scenario:**

1. Create test file: `tests/chaos_your_scenario.rs`
2. Import resilience components
3. Write test that triggers failure condition
4. Assert recovery behavior

**Example:**

```rust
#[tokio::test]
async fn test_new_failure_scenario() {
    // Setup
    let supervisor = create_test_supervisor();

    // Trigger failure
    let result = supervisor.supervise("agent", || async {
        // Your failure condition
        Err(AofError::runtime("Simulated failure"))
    }).await;

    // Assert recovery
    assert!(result.is_ok() || is_expected_failure(result));
}
```

**Test requirements:**
- Complete in < 30 seconds
- No external dependencies
- Assert on recovery, not just detection
- Record metrics for validation

## SLI/SLO Implementation

SLO definitions are in `config/slo-definitions.yaml`.

**Adding a new SLO:**

```yaml
slos:
  - name: my_new_slo
    description: "Description of what we're measuring"
    sli:
      type: latency|availability|success_rate
      metric: "PromQL query here"
    target: 99.0  # Your target
    window: 7d
    error_budget:
      total: 1.0%
```

**Metric types:**
- **availability**: % of successful requests
- **latency**: Response time percentiles
- **success_rate**: % of operations completing successfully

**PromQL patterns:**

Success rate:
```promql
rate(metric_total{status="success"}[1h]) / rate(metric_total[1h])
```

Latency percentile:
```promql
histogram_quantile(0.99, rate(metric_bucket[5m]))
```

## Testing Checklist

Before submitting resilience-related PRs:

- [ ] Unit tests pass (`cargo test -p aof-runtime resilience`)
- [ ] Chaos tests pass (relevant scenario)
- [ ] Metrics are recorded correctly
- [ ] Circuit breaker states transition correctly
- [ ] Degradation actions are logged
- [ ] Documentation updated (this file + runbooks)

## Metrics Reference

All resilience metrics are in `AofMetrics`:

```rust
pub struct AofMetrics {
    pub agent_restarts_total: Counter,
    pub agent_failures_total: Counter,
    // ... other metrics
}
```

**When to add a metric:**
- New resilience pattern
- New failure mode
- New degradation action
- Tracking SLI

## Resources

- **Resilience patterns:** `crates/aof-runtime/src/resilience/`
- **Chaos tests:** `tests/chaos_*.rs`
- **SLO definitions:** `config/slo-definitions.yaml`
- **User-facing guide:** `docs/guides/sre-operations.md`
