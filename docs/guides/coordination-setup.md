# Coordination Setup Guide

Enable agent coordination protocols (heartbeat health checks, daily standups, inter-agent messaging) in AOF daemon.

## Table of Contents

- [Getting Started](#getting-started)
- [Quick Start Config](#quick-start-config)
- [Enabling Daily Standups](#enabling-daily-standups)
- [Per-Agent Coordination Modes](#per-agent-coordination-modes)
- [Monitoring Coordination](#monitoring-coordination)
- [Mission Control Dashboard](#mission-control-dashboard)
- [Token Budget Management](#token-budget-management)
- [Full Configuration Reference](#full-configuration-reference)

## Getting Started

Enable coordination in 3 steps:

### Step 1: Add coordination section to serve-config.yaml

```yaml
# serve-config.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig

coordination:
  enabled: true
  mode: full
  heartbeat:
    frequency_secs: 60
    timeout_secs: 120
```

### Step 2: Set coordination_mode per agent (optional)

By default, all agents use the global mode (`full`). Override per agent:

```yaml
coordination:
  per_agent:
    k8s-monitor: full          # Critical agent, full coordination
    batch-processor: disabled  # Silent worker, no coordination
```

### Step 3: Restart aofctl serve

```bash
aofctl serve --config serve-config.yaml
```

That's it! Heartbeat checks run automatically every 60 seconds.

## Quick Start Config

Minimal configuration to get heartbeat working:

```yaml
# Minimal serve-config.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig

serve:
  port: 8080

coordination:
  enabled: true
  mode: full
  heartbeat:
    frequency_secs: 60
    timeout_secs: 120
```

Start daemon:

```bash
aofctl serve --config serve-config.yaml
```

Verify heartbeat is running:

```bash
curl http://localhost:8080/api/coordination/health
```

Expected response:

```json
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "healthy",
      "consecutive_misses": 0,
      "last_response_ms": 150
    }
  ],
  "heartbeat_config": {
    "frequency_secs": 60,
    "timeout_secs": 120
  }
}
```

## Enabling Daily Standups

Add standup section to coordination config:

```yaml
coordination:
  enabled: true
  mode: full

  heartbeat:
    frequency_secs: 60
    timeout_secs: 120

  standup:
    # Daily at 9:00 AM EST
    cron: "0 0 9 * * *"
    timezone: "America/New_York"

    # Enable AI summarization (optional, costs ~$0.01/standup)
    summarize: true
```

### Cron Expression Format

6-field format: `second minute hour day month day_of_week`

Examples:

| Expression | Meaning |
|------------|---------|
| `0 0 9 * * *` | 9:00 AM every day |
| `0 30 8 * * MON-FRI` | 8:30 AM weekdays only |
| `0 0 17 * * FRI` | 5:00 PM every Friday |
| `0 0 */4 * * *` | Every 4 hours |

### Timezone Support

Use IANA timezone names:

- **US**: `America/New_York`, `America/Los_Angeles`, `America/Chicago`
- **Europe**: `Europe/London`, `Europe/Paris`, `Europe/Berlin`
- **Asia**: `Asia/Tokyo`, `Asia/Singapore`, `Asia/Kolkata`

Full list: [IANA Time Zone Database](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones)

### Manual Standup Trigger

Trigger standup outside of schedule (useful for testing):

```bash
curl -X POST http://localhost:8080/api/coordination/standup/trigger
```

View latest standup:

```bash
curl http://localhost:8080/api/coordination/standup/latest
```

## Per-Agent Coordination Modes

Control coordination level per agent:

### Mode Levels

| Mode | Heartbeat | Standup | Messages | Use Case |
|------|-----------|---------|----------|----------|
| `full` | ✅ | ✅ | ✅ | Mission-critical agents |
| `standard` | ✅ | ✅ | ❌ | Normal agents |
| `reduced` | ✅ | ❌ | ❌ | Background workers |
| `heartbeat_only` | ✅ | ❌ | ❌ | Lightweight monitoring |
| `disabled` | ❌ | ❌ | ❌ | Silent/batch jobs |

### Configuration

```yaml
coordination:
  # Global default for all agents
  mode: full

  # Per-agent overrides
  per_agent:
    # Critical agents: full coordination
    k8s-monitor: full
    alert-triage: full
    incident-responder: full

    # Standard agents: heartbeat + standup
    log-analyzer: standard
    metric-checker: standard

    # Background agents: heartbeat only
    data-processor: reduced
    report-generator: reduced

    # Silent workers: no coordination
    batch-job: disabled
    ci-runner: disabled
```

### When to Use Each Mode

**Full:**
- Real-time incident response agents
- Customer-facing agents
- Agents requiring immediate attention if unresponsive

**Standard:**
- Most production agents
- Agents that benefit from daily check-ins
- Stable, long-running agents

**Reduced:**
- Background data processors
- Scheduled report generators
- High-volume batch workers

**Heartbeat Only:**
- Legacy agents without standup support
- Extremely high-volume agents (cost optimization)

**Disabled:**
- Cron-scheduled agents
- Fire-and-forget tasks
- Agents with external monitoring

## Monitoring Coordination

### REST API Endpoints

**Agent Health:**

```bash
GET /api/coordination/health
```

Response:

```json
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "healthy",
      "consecutive_misses": 0,
      "last_response_ms": 150
    },
    {
      "agent_id": "log-analyzer",
      "status": "unresponsive",
      "consecutive_misses": 3,
      "last_response_ms": null
    }
  ],
  "heartbeat_config": {
    "frequency_secs": 60,
    "timeout_secs": 120
  }
}
```

**Latest Standup:**

```bash
GET /api/coordination/standup/latest
```

Response:

```json
{
  "request_id": "abc123",
  "timestamp": "2026-02-14T09:00:00Z",
  "responses": [
    {
      "agent_id": "k8s-monitor",
      "what_i_did": "Monitored 5 pods, detected 1 crash",
      "what_im_doing": "Investigating pod restart loop",
      "blockers": []
    }
  ],
  "summary": "Team is healthy. K8s-monitor investigating pod issues. No blockers.",
  "response_count": 3
}
```

**Token Metrics:**

```bash
GET /api/coordination/metrics
```

Response:

```json
{
  "coordination_tokens": 125000,
  "production_tokens": 2500000,
  "overhead_percent": 5.0,
  "current_mode": "full",
  "heartbeat_tokens": 75000,
  "standup_tokens": 50000
}
```

**Force Mode Change (manual override):**

```bash
POST /api/coordination/mode
Content-Type: application/json

{
  "mode": "reduced"
}
```

### CLI Monitoring

**Health check:**

```bash
# Pretty-print JSON
curl -s http://localhost:8080/api/coordination/health | jq '.'

# Check if any agents unresponsive
curl -s http://localhost:8080/api/coordination/health | jq '.agents[] | select(.status == "unresponsive")'

# Count healthy agents
curl -s http://localhost:8080/api/coordination/health | jq '.agents | map(select(.status == "healthy")) | length'
```

**Standup check:**

```bash
# View latest standup summary
curl -s http://localhost:8080/api/coordination/standup/latest | jq '.summary'

# List agents with blockers
curl -s http://localhost:8080/api/coordination/standup/latest | jq '.responses[] | select(.blockers | length > 0)'
```

**Token overhead:**

```bash
# Check current overhead
curl -s http://localhost:8080/api/coordination/metrics | jq '.overhead_percent'

# Alert if >25% (approaching 30% limit)
OVERHEAD=$(curl -s http://localhost:8080/api/coordination/metrics | jq '.overhead_percent')
if (( $(echo "$OVERHEAD > 25" | bc -l) )); then
  echo "WARNING: Coordination overhead at ${OVERHEAD}%"
fi
```

## Mission Control Dashboard

View coordination in the web UI:

1. **Start daemon:**

```bash
aofctl serve --config serve-config.yaml
```

2. **Open Mission Control:**

Navigate to http://localhost:8080/ in your browser.

3. **Agent Health Panel:**

- **Green badge**: Agent is healthy
- **Red badge**: Agent is unresponsive
- **Heartbeat indicator**: Shows last response time

4. **Virtual Office Feed:**

Coordination events appear in the activity feed:

- `[Heartbeat] k8s-monitor: Healthy (150ms)`
- `[Standup] Daily standup completed (5 agents)`
- `[Alert] log-analyzer: Unresponsive (timeout after 120s)`

5. **Metrics Dashboard:**

- Coordination token usage
- Overhead percentage (vs. 30% budget)
- Current coordination mode
- Per-protocol breakdown (heartbeat, standup)

## Token Budget Management

Coordination protocols consume tokens. AOF enforces a 30% overhead budget.

### The 30% Rule

Coordination tokens must stay below 30% of total token usage:

```
overhead_percent = (coordination_tokens / total_tokens) × 100
```

If overhead exceeds 30%, auto-degradation activates.

### Auto-Degradation

When overhead > 30%, system automatically reduces coordination:

1. **Full → Standard**: Disable ad-hoc messaging
2. **Standard → Reduced**: Disable standup, keep heartbeat
3. **Reduced → HeartbeatOnly**: Minimal heartbeat
4. **HeartbeatOnly → Disabled**: Pause all coordination

### Recovery (Hysteresis)

When overhead drops below 20%, system recovers:

1. **Disabled → HeartbeatOnly**
2. **HeartbeatOnly → Reduced**
3. **Reduced → Standard**
4. **Standard → Full**

**Gap between 20-30% prevents flapping** (mode doesn't oscillate).

### Cost Projections

**10 agents, 60-second heartbeat:**

| Scenario | Heartbeat | Standup | Production | Overhead |
|----------|-----------|---------|------------|----------|
| Idle | 720k/day | 18k/day | 100k/day | 88% ⚠️ |
| Light | 720k/day | 18k/day | 1M/day | 42% ⚠️ |
| Normal | 720k/day | 18k/day | 10M/day | 7% ✅ |
| Heavy | 720k/day | 18k/day | 50M/day | 1.5% ✅ |

**Idle/light workloads trigger auto-degradation.** This is expected and healthy.

### Tuning Overhead

**Reduce coordination cost:**

1. **Increase heartbeat interval** (60s → 120s)
2. **Disable standup summarization** (`summarize: false`)
3. **Reduce agent count** (fewer agents = less coordination)
4. **Use reduced mode** for background agents

**Increase overhead budget:**

```yaml
coordination:
  token_limits:
    max_overhead_percent: 50  # Allow up to 50%
    recovery_threshold_percent: 40
```

⚠️ **Not recommended** — coordination should not dominate production work.

## Full Configuration Reference

```yaml
# Complete coordination configuration
coordination:
  # Master switch (required)
  enabled: true

  # Global default mode (required)
  mode: full  # Options: full, standard, reduced, heartbeat_only, disabled

  # Heartbeat configuration (optional, defaults shown)
  heartbeat:
    frequency_secs: 60      # How often to check (30-300 recommended)
    timeout_secs: 120       # When to mark unresponsive (2-3x frequency)

  # Standup configuration (optional, defaults shown)
  standup:
    cron: "0 0 9 * * *"           # Daily at 9 AM
    timezone: "America/New_York"  # IANA timezone
    summarize: false              # AI summarization ($0.01/standup)

  # Token limits (optional, defaults shown)
  token_limits:
    max_overhead_percent: 30           # Hard limit (coordination / total)
    auto_degrade: true                 # Automatically reduce if exceeded
    recovery_threshold_percent: 20     # Recover when overhead drops below

  # Per-agent overrides (optional)
  per_agent:
    agent-id: full  # Override global mode for specific agents
```

### Environment Variables

None. All coordination configuration is in serve-config.yaml.

### Logs

Coordination events are logged at `INFO` level:

```
INFO aof_coordination_protocols::heartbeat: Starting heartbeat scheduler (frequency: 60s, timeout: 120s)
INFO aof_coordination_protocols::heartbeat: Heartbeat tick: request_id = abc123
INFO aof_coordination_protocols::manager: Agent k8s-monitor: Healthy (150ms)
WARN aof_coordination_protocols::heartbeat: Agent log-analyzer unresponsive (timeout: 120s)
INFO aof_coordination_protocols::standup: Standup triggered: 2026-02-14T09:00:00Z
INFO aof_coordination_protocols::metrics: Token overhead: 28% (approaching threshold)
WARN aof_coordination_protocols::metrics: Auto-degrading: Full → Standard (overhead: 32%)
```

### Metrics Integration

Coordination metrics are available for Prometheus/Grafana:

- `coordination_tokens_total{protocol="heartbeat"}`
- `coordination_tokens_total{protocol="standup"}`
- `coordination_overhead_percent`
- `coordination_mode` (gauge: 0=disabled, 1=heartbeat_only, 2=reduced, 3=standard, 4=full)
- `agent_health_status{agent_id="k8s-monitor"}` (0=unresponsive, 1=healthy)

## Next Steps

- **Troubleshooting**: See [Coordination Troubleshooting Guide](coordination-troubleshooting.md)
- **Internal Docs**: See [Developer Documentation](../dev/coordination-protocols.md)
- **Concepts**: See [Coordination Protocols](../concepts/coordination-protocols.md)
- **Example Config**: See [examples/coordination-config.yaml](../../examples/coordination-config.yaml)
