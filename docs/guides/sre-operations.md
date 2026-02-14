# SRE Operations Guide

This guide covers operational procedures for running AOF in production.

## Monitoring

### Key Metrics to Track

**Agent Health:**
- `aof_agents_active` - Current number of running agents
- `aof_agent_executions_total` - Agent execution count (by status)
- `aof_agent_execution_duration_seconds` - Execution latency
- `aof_agent_restarts_total` - Crash/restart count
- `aof_agent_failures_total` - Permanent failures

**System Health:**
- System health state (Healthy/Degraded/Critical)
- Bulkhead utilization (0.0-1.0)
- Circuit breaker states
- Memory/CPU usage

**Events & WebSockets:**
- `aof_events_emitted_total` - Event volume
- `aof_event_broadcast_latency_seconds` - Broadcast latency
- `aof_websocket_clients` - Active connections

### Prometheus Queries

**Agent success rate (last 1h):**
```promql
rate(aof_agent_executions_total{status="success"}[1h]) /
rate(aof_agent_executions_total[1h]) * 100
```

**99th percentile agent latency:**
```promql
histogram_quantile(0.99, rate(aof_agent_execution_duration_seconds_bucket[5m]))
```

**Bulkhead utilization:**
```promql
aof_agents_active / 20  # assuming max_concurrent_agents=20
```

## Understanding SLOs and Error Budgets

### What is an Error Budget?

An error budget is the maximum allowed unreliability. If your availability SLO is 99.9%, your error budget is 0.1% (43.2 minutes per month).

**Key concepts:**
- **SLO** (Service Level Objective): Internal target (e.g., 99.9% availability)
- **SLI** (Service Level Indicator): Actual measured value
- **Error Budget**: How much we can fail and still meet SLO

### Burn Rate Alerts

Burn rate = how fast you're consuming your error budget.

- **1x burn rate**: On track to exactly hit SLO
- **2x burn rate**: Consuming budget twice as fast (alert!)
- **14.4x burn rate for 1 hour**: Consumes 1% of monthly budget

**Example:** If you have a 30-day error budget of 43.2 minutes:
- 2x burn rate alert fires if error rate sustained for 12 hours
- 14.4x burn rate alert fires if error rate sustained for 1 hour

## System Health States

### Healthy
- All metrics below warning thresholds
- Normal operation
- No degradation

### Degraded
- One or more components above warning thresholds
- Automatic actions:
  - Clear non-essential caches
  - Drop low-priority events
  - Reject new agent spawns if capacity high
- Service continues with reduced capacity

### Critical
- Metrics above critical thresholds (e.g., > 90% memory)
- Automatic actions:
  - Disable metrics collection
  - Disable event persistence
  - Keep only core agent execution

**Recovery:** System auto-recovers to Healthy when metrics drop below thresholds.

## Chaos Testing in Staging

Run chaos tests before major releases:

```bash
# Agent crash scenarios
cargo test --test chaos_agent_crash

# Resource exhaustion
cargo test --test chaos_resource_exhaustion

# Network/circuit breaker
cargo test --test chaos_network_partition
```

**What to verify:**
- Agents restart after crashes
- Circuit breakers open after threshold failures
- Bulkhead prevents resource exhaustion
- System recovers gracefully

## Incident Response Procedure

### 1. Detection
- Alert fires or user report received
- Check metrics dashboard
- Assess severity (P0/P1/P2/P3)

### 2. Initial Response
- Acknowledge alert
- Notify team if P0/P1
- Check runbooks for matching scenario

### 3. Mitigation
- Apply immediate fix (restart, reduce load, etc.)
- Document actions taken
- Monitor recovery

### 4. Resolution
- Implement permanent fix
- Verify resolution
- Close incident

### 5. Postmortem
- Fill out postmortem template (see docs/templates/postmortem.md)
- Schedule review within 2 business days
- Create action items

## Postmortem Process

**Blameless culture:** Focus on systems, not individuals.

**Required for:**
- P0/P1 incidents
- SLO breaches
- Customer-impacting issues

**Template:** `docs/templates/postmortem.md`

**Review meeting:**
- Share timeline and root cause
- Discuss what went well/poorly
- Assign action items with owners and dates
- Schedule follow-ups

## On-Call Best Practices

**Before your shift:**
- Review recent incidents
- Check runbook coverage
- Test access to systems

**During incidents:**
- Follow runbooks first
- Document everything
- Communicate status updates
- Escalate early if uncertain

**After resolution:**
- Update runbooks if gaps found
- File bugs for action items
- Hand off to next on-call

## Alert Threshold Tuning

**Avoid alert fatigue:**
- Set thresholds based on actual SLO impact
- Use burn rate alerts, not absolute thresholds
- Require actionable response for every alert

**Good alert:**
- Indicates SLO breach or imminent breach
- Clear mitigation steps
- Fires before user impact

**Bad alert:**
- Noisy (frequent false positives)
- No clear action
- Already resolved by auto-recovery

## Capacity Planning

**Resource per agent (approximate):**
- **Memory:** 10-50MB per agent
- **CPU:** 0.1-0.5 cores per agent (depends on tool calls)
- **Network:** Minimal unless heavy MCP usage

**Scaling guidance:**
- **Vertical:** Increase memory/CPU up to 16GB / 8 cores
- **Horizontal:** Run multiple AOF daemons with load balancer
- **Bulkhead limits:** Set to 60-80% of max capacity for buffer

**Monitoring scaling needs:**
- Bulkhead utilization consistently > 80%
- Agent execution latency increasing
- Memory pressure frequent

## Resources

- **Runbooks:** `docs/runbooks/`
- **SLO Definitions:** `config/slo-definitions.yaml`
- **Metrics Endpoint:** `http://localhost:8080/metrics`
- **Health Endpoint:** `http://localhost:8080/health`
