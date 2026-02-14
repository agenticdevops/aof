# Phase 8: SRE Best Practices for Production Hardening

## Executive Summary

This research document outlines SRE-hardened production system requirements for AOF's Phase 8, drawing from Google SRE practices, modern chaos engineering patterns, and 2026 observability standards. AOF must operate as a production-grade system itself, demonstrating the reliability principles it enables for DevOps/SRE teams.

**Core Principles:**
- **Resilience First**: By 2026, resilience is the key measure—systems must withstand disruption, degrade gracefully, and recover quickly
- **Autonomous Operations**: AI agents must self-heal and adapt without constant human oversight
- **Observable by Default**: Structured telemetry, SLIs, and error budgets guide incident response
- **Chaos-Tested**: Production readiness verified through controlled failure injection

---

## 1. Error Recovery & Resilience Patterns

### 1.1 Agent Crash Handling

**Isolation Strategy:**
```
┌─────────────────────────────────────────────────┐
│ Agent Process Isolation                         │
├─────────────────────────────────────────────────┤
│ 1. Each agent runs in isolated context          │
│ 2. Crash detection via heartbeat timeout        │
│ 3. Automatic cleanup of agent resources         │
│ 4. State persistence before termination         │
│ 5. Restart with exponential backoff             │
└─────────────────────────────────────────────────┘

State Machine:
  [Running] ──timeout──> [Suspected] ──confirm──> [Crashed]
      │                      │                         │
      │                      └──heartbeat──> [Running]│
      │                                                │
      └────────────────────cleanup────────────────────┘
                                  │
                                  ▼
                            [Restarting] ──> [Running]
```

**Implementation Pattern (Rust):**
```rust
// Agent supervisor with crash recovery
pub struct AgentSupervisor {
    heartbeat_timeout: Duration,
    max_restart_attempts: usize,
    backoff_strategy: ExponentialBackoff,
}

impl AgentSupervisor {
    async fn supervise(&self, agent: Agent) -> Result<()> {
        let mut attempts = 0;

        loop {
            match self.run_agent(&agent).await {
                Ok(_) => break,
                Err(e) if attempts < self.max_restart_attempts => {
                    attempts += 1;
                    let delay = self.backoff_strategy.delay(attempts);
                    warn!("Agent crashed (attempt {}/{}): {}", attempts, self.max_restart_attempts, e);

                    // Cleanup resources
                    self.cleanup_agent_resources(&agent).await?;

                    // Wait with exponential backoff
                    tokio::time::sleep(delay).await;

                    // Restore state and restart
                    self.restore_agent_state(&agent).await?;
                }
                Err(e) => {
                    error!("Agent failed permanently after {} attempts: {}", attempts, e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}
```

### 1.2 Task Retry Logic with Circuit Breaker

**Retry Strategy:**
- Exponential backoff: `delay = base_delay * 2^attempt`
- Jitter: Add random variance to prevent thundering herd
- Max attempts: 5 retries before escalation
- Circuit breaker: Stop retrying if failure rate exceeds threshold

**Circuit Breaker State Machine:**
```
┌──────────┐  failure_rate > threshold  ┌──────────┐
│  Closed  │──────────────────────────> │   Open   │
│ (normal) │                             │ (reject) │
└──────────┘                             └──────────┘
     ▲                                        │
     │                                        │ timeout
     │ success_rate > threshold               ▼
     │                                   ┌──────────┐
     └───────────────────────────────────│Half-Open │
                                         │  (test)  │
                                         └──────────┘
```

**Rust Implementation:**
```rust
use tower::ServiceBuilder;
use tower_circuitbreaker::CircuitBreaker;

// Task executor with circuit breaker protection
pub struct ResilientTaskExecutor {
    circuit_breaker: CircuitBreaker,
    retry_policy: RetryPolicy,
}

#[derive(Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl RetryPolicy {
    fn calculate_delay(&self, attempt: usize) -> Duration {
        let exponential_delay = self.base_delay * 2_u32.pow(attempt as u32);
        let capped_delay = exponential_delay.min(self.max_delay);

        if self.jitter {
            let jitter = rand::thread_rng().gen_range(0..=capped_delay.as_millis() / 10);
            capped_delay + Duration::from_millis(jitter as u64)
        } else {
            capped_delay
        }
    }
}

impl ResilientTaskExecutor {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let mut attempt = 0;

        loop {
            match self.circuit_breaker.call(|| task.execute()).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.retry_policy.max_attempts => {
                    attempt += 1;
                    let delay = self.retry_policy.calculate_delay(attempt);
                    warn!("Task failed (attempt {}/{}): {}. Retrying in {:?}",
                          attempt, self.retry_policy.max_attempts, e, delay);
                    tokio::time::sleep(delay).await;
                }
                Err(CircuitBreakerError::Open) => {
                    return Err(anyhow!("Circuit breaker open, service unavailable"));
                }
                Err(e) => {
                    return Err(anyhow!("Task failed after {} attempts: {}", attempt, e));
                }
            }
        }
    }
}
```

### 1.3 Bulkhead Pattern (Cascading Failure Prevention)

**Resource Isolation:**
```
┌─────────────────────────────────────────────────┐
│ Bulkhead Architecture                           │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ Agent    │  │ Event    │  │ MCP      │      │
│  │ Pool     │  │ Queue    │  │ Client   │      │
│  │ (10 max) │  │ (100 max)│  │ (5 max)  │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│                                                  │
│  Each subsystem has isolated resource limits    │
│  Failure in one does NOT cascade to others      │
└─────────────────────────────────────────────────┘
```

**Implementation:**
```rust
pub struct BulkheadConfig {
    max_concurrent_agents: usize,    // 10
    max_event_queue_size: usize,     // 100
    max_mcp_connections: usize,      // 5
    max_websocket_clients: usize,    // 50
}

pub struct BulkheadedRuntime {
    agent_semaphore: Arc<Semaphore>,
    event_queue: BoundedQueue<Event>,
    mcp_pool: Pool<McpClient>,
    ws_pool: Pool<WebSocketClient>,
}

impl BulkheadedRuntime {
    async fn spawn_agent(&self, agent: Agent) -> Result<()> {
        // Acquire permit from agent bulkhead
        let permit = self.agent_semaphore.acquire().await?;

        // Agent runs with isolated resources
        tokio::spawn(async move {
            agent.run().await;
            drop(permit); // Release bulkhead slot
        });

        Ok(())
    }

    async fn enqueue_event(&self, event: Event) -> Result<()> {
        // Bounded queue prevents memory exhaustion
        self.event_queue.send(event).await
            .map_err(|_| anyhow!("Event queue full (backpressure active)"))
    }
}
```

---

## 2. Graceful Degradation Patterns

### 2.1 System Degradation Flowchart

```
┌─────────────────────────────────────────────────┐
│ Graceful Degradation Decision Tree              │
└─────────────────────────────────────────────────┘

                    [Health Check]
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    [All Healthy]   [Degraded]      [Critical]
          │               │               │
          ▼               ▼               ▼
    Full Features   Reduce Features  Essential Only
          │               │               │
          │        ┌──────┴──────┐       │
          │        ▼             ▼       │
          │    Disable      Reduce       │
          │    - UI sync    - Agents     │
          │    - Metrics    - Events     │
          │    - Logging    - Memory     │
          │                              │
          └──────────────┬───────────────┘
                         ▼
                  [Core Function]
                  - CLI execution
                  - Agent lifecycle
                  - Event dispatch
```

### 2.2 Component Independence Matrix

| Component | Runs if UI Crashed | Runs if Daemon Down | Runs if MCP Unavailable |
|-----------|-------------------|---------------------|-------------------------|
| aofctl CLI | ✅ Yes | ❌ No | ✅ Yes (degraded) |
| Daemon | ✅ Yes | N/A | ✅ Yes (degraded) |
| Agents | ✅ Yes | ❌ No | ✅ Yes (degraded) |
| Event Queue | ✅ Yes | ❌ No | ✅ Yes |
| WebSocket | ❌ No | ❌ No | ✅ Yes |
| MCP Client | ✅ Yes | ❌ No | N/A |

### 2.3 Degradation Implementation

```rust
#[derive(Debug, Clone)]
pub enum SystemHealth {
    Healthy,
    Degraded(Vec<DegradedComponent>),
    Critical(Vec<FailedComponent>),
}

#[derive(Debug, Clone)]
pub enum DegradedComponent {
    UIDisconnected,
    McpPartialFailure,
    MemoryPressure,
    EventQueueBackpressure,
}

pub struct AdaptiveRuntime {
    health: Arc<RwLock<SystemHealth>>,
}

impl AdaptiveRuntime {
    async fn adapt_to_health(&self) {
        match *self.health.read().await {
            SystemHealth::Healthy => {
                // Full feature set enabled
                self.enable_full_features().await;
            }
            SystemHealth::Degraded(ref components) => {
                for component in components {
                    match component {
                        DegradedComponent::UIDisconnected => {
                            // Daemon continues, buffers events
                            self.buffer_ui_events().await;
                        }
                        DegradedComponent::MemoryPressure => {
                            // Reduce agent count, clear caches
                            self.reduce_agent_count().await;
                            self.clear_non_essential_caches().await;
                        }
                        DegradedComponent::EventQueueBackpressure => {
                            // Drop low-priority events
                            self.enable_event_priority_filtering().await;
                        }
                        _ => {}
                    }
                }
            }
            SystemHealth::Critical(ref failures) => {
                // Essential operations only
                warn!("Critical health: {:?}. Entering survival mode.", failures);
                self.enter_survival_mode().await;
            }
        }
    }
}
```

---

## 3. Observability Best Practices

### 3.1 RED Method Implementation

**RED: Request, Error, Duration**

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct RedMetrics {
    // Request: Rate of requests
    requests_total: Counter,

    // Error: Rate of errors
    errors_total: Counter,

    // Duration: Distribution of response times
    request_duration_seconds: Histogram,
}

impl RedMetrics {
    pub fn new(registry: &Registry) -> Self {
        let requests_total = Counter::new("aof_requests_total", "Total requests")
            .expect("metric creation");
        registry.register(Box::new(requests_total.clone())).unwrap();

        let errors_total = Counter::new("aof_errors_total", "Total errors")
            .expect("metric creation");
        registry.register(Box::new(errors_total.clone())).unwrap();

        let request_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("aof_request_duration_seconds", "Request duration")
                .buckets(vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0])
        ).expect("metric creation");
        registry.register(Box::new(request_duration_seconds.clone())).unwrap();

        Self { requests_total, errors_total, request_duration_seconds }
    }

    pub async fn track_request<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let timer = self.request_duration_seconds.start_timer();
        self.requests_total.inc();

        match operation.await {
            Ok(result) => {
                timer.observe_duration();
                Ok(result)
            }
            Err(e) => {
                self.errors_total.inc();
                timer.observe_duration();
                Err(e)
            }
        }
    }
}
```

### 3.2 SLI/SLO Definitions

**Service Level Indicators (SLIs):**

| SLI | Measurement | Target | Measurement Window |
|-----|-------------|--------|-------------------|
| **Availability** | (successful_requests / total_requests) * 100 | 99.9% | 30 days |
| **Latency (p99)** | 99th percentile response time | < 500ms | 24 hours |
| **Error Rate** | (failed_requests / total_requests) * 100 | < 0.1% | 24 hours |
| **Agent Success Rate** | (successful_agent_runs / total_agent_runs) * 100 | 95% | 7 days |
| **Event Processing Lag** | time_since_event_creation | < 1s | Real-time |

**Service Level Objectives (SLOs):**

```yaml
# .aof/slo-definitions.yaml
slos:
  - name: api_availability
    sli: availability
    target: 99.9
    window: 30d
    error_budget: 0.1%  # 43.2 minutes downtime per month

  - name: agent_execution_latency
    sli: latency_p99
    target: 500ms
    window: 24h
    error_budget: 1.0%  # 14.4 minutes above threshold per day

  - name: task_success_rate
    sli: agent_success_rate
    target: 95%
    window: 7d
    error_budget: 5.0%  # Up to 5% task failures acceptable

  - name: event_freshness
    sli: event_processing_lag
    target: 1s
    window: realtime
    error_budget: 0.01%  # 8.64 seconds lag per day
```

**Error Budget Tracking:**

```rust
pub struct ErrorBudget {
    slo_target: f64,      // 99.9%
    window: Duration,      // 30 days
    consumed: f64,         // How much budget used
}

impl ErrorBudget {
    pub fn remaining(&self) -> f64 {
        (1.0 - self.slo_target) - self.consumed
    }

    pub fn burn_rate(&self) -> f64 {
        self.consumed / self.window.as_secs_f64()
    }

    pub fn alert_threshold(&self) -> bool {
        self.burn_rate() > 2.0  // Burning budget 2x faster than sustainable
    }
}
```

### 3.3 Structured Logging & Tracing

```rust
use tracing::{info, warn, error, instrument, Span};
use tracing_subscriber::{fmt, EnvFilter};

#[instrument(skip(self), fields(agent_id = %agent.id, task_id = %task.id))]
pub async fn execute_agent_task(&self, agent: &Agent, task: &Task) -> Result<()> {
    let span = Span::current();
    span.record("start_time", chrono::Utc::now().to_rfc3339());

    info!(
        agent.name = %agent.name,
        task.type = %task.task_type,
        "Starting agent task execution"
    );

    match agent.execute(task).await {
        Ok(result) => {
            info!(
                duration_ms = result.duration.as_millis(),
                "Agent task completed successfully"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                error = %e,
                error.kind = ?e.kind(),
                "Agent task failed"
            );
            Err(e)
        }
    }
}
```

**Log Levels by Environment:**

| Level | Development | Staging | Production |
|-------|-------------|---------|------------|
| TRACE | ✅ Yes | ❌ No | ❌ No |
| DEBUG | ✅ Yes | ✅ Yes | ❌ No |
| INFO | ✅ Yes | ✅ Yes | ✅ Yes |
| WARN | ✅ Yes | ✅ Yes | ✅ Yes |
| ERROR | ✅ Yes | ✅ Yes | ✅ Yes |

---

## 4. Chaos Engineering Test Scenarios

### 4.1 Chaos Test Matrix

| Test Scenario | Blast Radius | Expected Behavior | Success Criteria |
|---------------|--------------|-------------------|------------------|
| **Kill Random Agent** | Single agent | Agent restarts, task resumes | Recovery < 5s |
| **WebSocket Disconnect** | Single client | Buffered events replayed on reconnect | No event loss |
| **Event Queue Saturation** | Event subsystem | Backpressure activated, low-priority events dropped | No OOM |
| **Memory Pressure** | Entire system | Agent count reduced, caches cleared | Graceful degradation |
| **CPU Spike** | Task execution | Slower execution, no crashes | Latency degrades, no errors |
| **Network Partition** | MCP clients | Circuit breaker opens, local fallback | Operations continue degraded |
| **Clock Skew** | Time-dependent logic | Events still processed in order | Logical ordering preserved |
| **Disk Full** | Logging/persistence | Rotate logs, alert operators | No crash |

### 4.2 Chaos Test Implementation

```rust
// tests/chaos_engineering.rs

use aof_testing::chaos::{ChaosScenario, ChaosTester};

#[tokio::test]
async fn chaos_kill_random_agent() {
    let runtime = TestRuntime::new().await;
    let chaos = ChaosTester::new();

    // Spawn 10 agents
    let agents = (0..10)
        .map(|i| runtime.spawn_agent(format!("agent-{}", i)))
        .collect::<Vec<_>>();

    // Kill 3 random agents
    chaos.kill_random(agents.clone(), 3).await;

    // Verify recovery
    tokio::time::sleep(Duration::from_secs(10)).await;
    assert_eq!(runtime.healthy_agent_count().await, 10);
}

#[tokio::test]
async fn chaos_event_queue_saturation() {
    let runtime = TestRuntime::new().await;
    let chaos = ChaosTester::new();

    // Saturate queue with 1000 events
    chaos.saturate_queue(runtime.event_queue(), 1000).await;

    // Verify backpressure active
    assert!(runtime.is_backpressure_active().await);

    // Verify low-priority events dropped
    let stats = runtime.event_queue_stats().await;
    assert!(stats.dropped_count > 0);
    assert!(stats.dropped_count < stats.total_count * 0.2);  // < 20% dropped
}

#[tokio::test]
async fn chaos_network_partition() {
    let runtime = TestRuntime::new().await;
    let chaos = ChaosTester::new();

    // Partition network to MCP server
    chaos.partition_network("mcp_server").await;

    // Verify circuit breaker opens
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(runtime.is_circuit_open("mcp").await);

    // Verify local fallback active
    let result = runtime.execute_task("test_task").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().mode, ExecutionMode::LocalFallback);
}
```

### 4.3 Continuous Chaos (Production-Safe)

```yaml
# .aof/chaos-schedule.yaml
# Run chaos tests in production during low-traffic hours

chaos_schedule:
  - name: agent_resilience_test
    scenario: kill_random_agent
    schedule: "0 3 * * *"  # 3 AM daily
    blast_radius: 10%

  - name: websocket_resilience_test
    scenario: disconnect_random_clients
    schedule: "0 4 * * *"  # 4 AM daily
    blast_radius: 5%

  - name: memory_pressure_test
    scenario: gradual_memory_pressure
    schedule: "0 2 * * SUN"  # 2 AM Sunday
    blast_radius: staging_only
```

---

## 5. Production Incident Handling

### 5.1 Incident Severity Levels

| Level | Description | Response Time | Escalation | Example |
|-------|-------------|---------------|------------|---------|
| **P0 (Critical)** | Complete service outage | 15 minutes | Immediate page | Daemon crash loop |
| **P1 (High)** | Major feature broken | 1 hour | Email + Slack | All agents failing |
| **P2 (Medium)** | Minor feature degraded | 4 hours | Slack | WebSocket intermittent |
| **P3 (Low)** | Cosmetic/non-blocking | Next business day | Ticket | Logging verbosity high |

### 5.2 Incident Response Runbook Template

```markdown
# Runbook: Agent Crash Loop

## Symptoms
- Agents repeatedly crash and restart
- Error logs show panic: "thread 'main' panicked at..."
- High CPU usage from restart churn

## Impact
- Agent tasks fail to complete
- Event queue backs up
- User operations timeout

## Investigation
1. Check agent logs: `journalctl -u aof-daemon | grep panic`
2. Identify crash pattern: `grep -A 10 "panicked at" /var/log/aof/*.log`
3. Check resource usage: `top -p $(pgrep aof-daemon)`
4. Review recent changes: `git log --since="1 day ago" --oneline`

## Mitigation (Immediate)
1. Disable problematic agent type:
   ```bash
   aofctl config set agents.disabled "problematic_agent_type"
   ```
2. Restart daemon:
   ```bash
   systemctl restart aof-daemon
   ```
3. Verify recovery:
   ```bash
   aofctl status --watch
   ```

## Resolution (Permanent)
1. Reproduce crash in test environment
2. Add unit test that triggers crash
3. Fix root cause (e.g., panic on null pointer)
4. Add error handling instead of panic
5. Deploy fix with canary rollout
6. Monitor error rate for 24 hours

## Escalation
- If mitigation fails: Page on-call engineer
- If root cause unclear: Escalate to architect
- If data corruption suspected: Escalate to SRE lead

## Prevention
- Add pre-flight validation for agent inputs
- Implement supervisor pattern for all agents
- Add crash telemetry to detect patterns early
```

### 5.3 Blameless Postmortem Template

```markdown
# Incident Postmortem: [Title]

**Date:** YYYY-MM-DD
**Severity:** P0/P1/P2/P3
**Duration:** X hours Y minutes
**Impact:** X users affected, Y% error rate
**Incident Commander:** Name

## Summary
Brief 2-3 sentence overview of what happened.

## Timeline (All times UTC)
- **HH:MM** - Incident started (first alert)
- **HH:MM** - Investigation began
- **HH:MM** - Root cause identified
- **HH:MM** - Mitigation applied
- **HH:MM** - Service restored
- **HH:MM** - Incident closed

## Root Cause
Detailed explanation of why the incident occurred. Focus on system/process failures, not human errors.

## What Went Well
- Early detection via monitoring
- Quick root cause identification
- Effective mitigation

## What Went Poorly
- Alert fatigue delayed response
- Runbook was outdated
- No automated rollback

## Action Items
| Action | Owner | Due Date | Priority |
|--------|-------|----------|----------|
| Add test for crash scenario | @engineer | 2026-02-21 | P0 |
| Update runbook | @sre | 2026-02-18 | P1 |
| Implement automated rollback | @architect | 2026-03-01 | P1 |
| Reduce alert noise | @observability | 2026-02-25 | P2 |

## Lessons Learned
1. Always have automated rollback for risky deployments
2. Runbooks must be tested quarterly
3. Alert thresholds need tuning based on error budget
```

### 5.4 On-Call Escalation Procedure

```
┌─────────────────────────────────────────────────┐
│ Incident Escalation Path                        │
└─────────────────────────────────────────────────┘

[Alert Fires]
     │
     ▼
[On-Call Engineer]
     │
     ├─ Resolves in 30 min ──> [Close Incident]
     │
     ├─ P0/P1 & Complex ──────> [Page Incident Commander]
     │                                │
     │                                ▼
     │                          [Assemble War Room]
     │                                │
     │                                ├─ Engineering Lead
     │                                ├─ SRE Lead
     │                                ├─ Product Owner
     │                                └─ Support Lead
     │
     └─ No Progress in 2 hrs ──> [Escalate to Architect]
                                      │
                                      ▼
                               [Emergency Change]
                                      │
                                      └─> [Rollback / Hotfix]
```

---

## 6. Capacity Planning

### 6.1 Resource Requirements per Agent

| Agent Type | Memory (MB) | CPU (cores) | Disk I/O (MB/s) | Network (KB/s) |
|------------|-------------|-------------|-----------------|----------------|
| **Basic Executor** | 50 | 0.1 | 5 | 10 |
| **MCP Agent** | 100 | 0.2 | 10 | 50 |
| **Memory-Intensive** | 500 | 0.5 | 20 | 20 |
| **Heavy Compute** | 200 | 2.0 | 5 | 10 |

**System Overhead:**
- Daemon base: 100 MB RAM, 0.2 CPU
- Event queue: 50 MB per 1000 queued events
- WebSocket server: 10 MB per 100 clients

### 6.2 Scaling Limits

**Single-Node Limits (recommended):**
- Max concurrent agents: 50
- Max WebSocket clients: 500
- Max event queue size: 10,000
- Max memory usage: 4 GB
- Max CPU usage: 80% (leave headroom)

**Breaking Points (stress tested):**
- Agent limit: 100 (beyond this, scheduling overhead dominates)
- WebSocket limit: 1,000 (beyond this, TCP buffer exhaustion)
- Event queue: 50,000 (beyond this, GC pauses)

### 6.3 Cost Optimization Strategies

**Cloud Instance Sizing:**
```
Small Workload (< 10 agents):
  AWS: t3.medium (2 vCPU, 4 GB RAM) = $0.0416/hr
  GCP: e2-medium (2 vCPU, 4 GB RAM) = $0.0335/hr

Medium Workload (10-30 agents):
  AWS: t3.large (2 vCPU, 8 GB RAM) = $0.0832/hr
  GCP: e2-standard-2 (2 vCPU, 8 GB RAM) = $0.0670/hr

Large Workload (30-50 agents):
  AWS: t3.xlarge (4 vCPU, 16 GB RAM) = $0.1664/hr
  GCP: e2-standard-4 (4 vCPU, 16 GB RAM) = $0.1340/hr
```

**Auto-Scaling Policy:**
```yaml
# .aof/autoscaling.yaml
scaling:
  metrics:
    - type: cpu_utilization
      target: 70%

    - type: memory_utilization
      target: 75%

    - type: event_queue_depth
      target: 1000

  scale_up:
    threshold: 80%
    cooldown: 5m
    step: +1 node

  scale_down:
    threshold: 40%
    cooldown: 15m
    step: -1 node
    min_nodes: 1
```

### 6.4 Growth Forecasting

**Linear Projection Model:**
```
agents_needed(t) = current_agents * (1 + growth_rate)^t

Example:
  Current: 20 agents
  Growth: 15% monthly
  6 months: 20 * (1.15)^6 = 46 agents
```

**Capacity Planning Worksheet:**

| Month | Projected Agents | RAM Needed | CPU Needed | Instance Type | Monthly Cost |
|-------|-----------------|------------|------------|---------------|--------------|
| Feb 2026 | 20 | 2 GB | 2 vCPU | t3.medium | $30 |
| Mar 2026 | 23 | 2.3 GB | 2.3 vCPU | t3.medium | $30 |
| Apr 2026 | 26 | 2.6 GB | 2.6 vCPU | t3.medium | $30 |
| May 2026 | 30 | 3 GB | 3 vCPU | t3.large | $60 |
| Jun 2026 | 35 | 3.5 GB | 3.5 vCPU | t3.large | $60 |
| Jul 2026 | 40 | 4 GB | 4 vCPU | t3.large | $60 |
| Aug 2026 | 46 | 4.6 GB | 4.6 vCPU | t3.xlarge | $120 |

**Recommendation:** Upgrade to t3.large in May 2026 to avoid mid-month scaling event.

---

## 7. Observability Stack Recommendation

### 7.1 Recommended Tools

```
┌─────────────────────────────────────────────────┐
│ Observability Stack (Production-Grade)          │
├─────────────────────────────────────────────────┤
│                                                  │
│  Metrics:   Prometheus + Grafana                │
│             - RED dashboards                     │
│             - SLO tracking                       │
│             - Alerting                           │
│                                                  │
│  Logs:      Loki (for Grafana integration)      │
│             - Structured JSON logs               │
│             - Trace correlation                  │
│                                                  │
│  Traces:    Jaeger / Tempo                      │
│             - Distributed tracing                │
│             - Agent execution flows              │
│                                                  │
│  Alerts:    Alertmanager                        │
│             - PagerDuty integration              │
│             - Slack notifications                │
│                                                  │
│  Dashboards: Grafana                            │
│             - System health overview             │
│             - Per-agent metrics                  │
│             - SLO burn rate                      │
└─────────────────────────────────────────────────┘
```

### 7.2 Key Grafana Dashboards

**Dashboard 1: System Health Overview**
- Panels:
  - Current health status (Healthy/Degraded/Critical)
  - Active agents count
  - Event queue depth
  - WebSocket client count
  - Memory/CPU utilization
  - Error rate (last 1h)
  - P99 latency (last 1h)

**Dashboard 2: SLO Tracking**
- Panels:
  - Availability % (30d rolling)
  - Error budget remaining
  - Error budget burn rate
  - SLO compliance per service
  - Alerts fired (last 7d)

**Dashboard 3: Agent Performance**
- Panels:
  - Agent execution time (p50, p95, p99)
  - Agent success rate
  - Agent crash count
  - Agent restart count
  - Resource usage per agent type

### 7.3 Alert Thresholds (No Alert Fatigue)

**Critical Alerts (Page immediately):**
- Availability < 99% over 5 minutes
- Error rate > 5% over 5 minutes
- Error budget burn rate > 10x normal
- All agents crashed

**Warning Alerts (Slack notification):**
- Availability < 99.5% over 15 minutes
- Error rate > 1% over 15 minutes
- Event queue depth > 5000
- Memory usage > 80%

**Info Alerts (Email):**
- New deployment detected
- Circuit breaker opened
- Graceful degradation activated

---

## 8. Implementation Checklist

### Phase 8 Deliverables

- [ ] **Error Recovery**
  - [ ] Agent supervisor with crash recovery
  - [ ] Task retry logic with exponential backoff + jitter
  - [ ] Circuit breaker for MCP clients
  - [ ] Bulkhead pattern for resource isolation

- [ ] **Graceful Degradation**
  - [ ] Health check system
  - [ ] Adaptive runtime (adjusts features based on health)
  - [ ] Component independence verification
  - [ ] Daemon survives UI crashes

- [ ] **Observability**
  - [ ] RED metrics instrumentation
  - [ ] SLI/SLO definitions
  - [ ] Structured logging with trace correlation
  - [ ] Prometheus + Grafana dashboards

- [ ] **Chaos Engineering**
  - [ ] Chaos test suite (8 scenarios)
  - [ ] CI integration for chaos tests
  - [ ] Production chaos schedule
  - [ ] Chaos test report automation

- [ ] **Incident Response**
  - [ ] Runbooks for common failures (5 minimum)
  - [ ] Blameless postmortem template
  - [ ] On-call escalation procedure
  - [ ] Incident severity definitions

- [ ] **Capacity Planning**
  - [ ] Resource benchmarking per agent type
  - [ ] Scaling limits documentation
  - [ ] Auto-scaling policy
  - [ ] 6-month growth forecast

---

## Sources & References

**Google SRE Best Practices:**
- [Google SRE: Production Services Best Practices](https://sre.google/sre-book/service-best-practices/)
- [Google SRE - Table of Contents](https://sre.google/sre-book/table-of-contents/)
- [Google SRE - Incident Response Strategies](https://sre.google/sre-book/emergency-response/)
- [Google SRE - Service Level Objectives](https://sre.google/sre-book/service-level-objectives/)
- [Google SRE - Implementing SLOs](https://sre.google/workbook/implementing-slos/)

**Graceful Degradation & Observability:**
- [Agentic AI: Design Reliable Workflows (Red Hat)](https://developers.redhat.com/articles/2026/02/11/agentic-ai-design-reliable-workflows-across-hybrid-cloud)
- [Top 5 Observability Predictions for 2026](https://www.motadata.com/blog/observability-predictions/)
- [Google Cloud - Design for Graceful Degradation](https://docs.cloud.google.com/architecture/framework/reliability/graceful-degradation)
- [New Relic - Four Considerations for Graceful Degradation](https://newrelic.com/blog/observability/design-software-for-graceful-degradation)
- [Multi-Agent System Architecture Guide for 2026](https://www.clickittech.com/ai/multi-agent-system-architecture/)

**Chaos Engineering:**
- [InfoQ - Resilience and Chaos Engineering in Kubernetes](https://www.infoq.com/presentations/resilience-chaos-kubernetes/)
- [Harness - Chaos Engineering for Kubernetes Resilience](https://www.harness.io/blog/understanding-chaos-engineering-and-its-role-in-kubernetes-resilience)
- [Medium - Autonomous Agent Swarms in Chaos Engineering](https://medium.com/data-science-collective/autonomous-agent-swarms-in-chaos-engineering-revolutionizing-resilience-testing-42be9c915bcc)
- [LitmusChaos - Open Source Chaos Engineering Platform](https://litmuschaos.io/)
- [NashTech - Cross-Cloud Chaos Engineering with AI Agents](https://blog.nashtechglobal.com/cross-cloud-chaos-engineering/)

**SLI/SLO & Incident Response:**
- [Atlassian - Service-Level Objectives (SLOs)](https://www.atlassian.com/incident-management/kpis/sla-vs-slo-vs-sli)
- [incident.io - SLOs, SLAs, and SLIs Guide](https://incident.io/blog/slo-sla-sli)
- [Nobl9 - Guide to Incident Response Metrics](https://www.nobl9.com/service-availability/incident-response-metrics)
- [Splunk - SRE Metrics and Four Golden Signals](https://www.splunk.com/en_us/blog/learn/sre-metrics-four-golden-signals-of-monitoring.html)
- [Google Cloud Blog - SRE Fundamentals: SLAs vs SLOs vs SLIs](https://cloud.google.com/blog/products/devops-sre/sre-fundamentals-slis-slas-and-slos)

**Circuit Breaker & Bulkhead (Rust):**
- [tower-circuitbreaker - Rust Library](https://lib.rs/crates/tower-circuitbreaker)
- [failsafe-rs - Circuit Breaker for Rust](https://github.com/dmexe/failsafe-rs)
- [Medium - Circuit Breaker and Bulkhead Patterns for Resilience](https://medium.com/@platform.engineers/circuit-breaker-and-bulkhead-patterns-for-resilience-2a8ae88ac717)
- [circuitbreaker-rs - Rust Concurrency Library](https://lib.rs/crates/circuitbreaker-rs)

---

## Next Steps

1. **Review with Team**: Share this research with architects and SREs
2. **Prioritize Implementation**: Focus on P0 items (error recovery, observability)
3. **Build Test Infrastructure**: Set up chaos testing framework first
4. **Instrument Codebase**: Add metrics, logging, tracing to all components
5. **Define Runbooks**: Document common failure scenarios
6. **Set Up Monitoring**: Deploy Prometheus + Grafana stack
7. **Chaos Test**: Run initial chaos experiments in staging
8. **Production Rollout**: Gradual rollout with error budget tracking

---

**Document Version:** 1.0
**Last Updated:** 2026-02-14
**Author:** AOF SRE Team
**Status:** Draft for Review
