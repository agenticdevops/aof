# Token Budget Management

**Status:** Phase 7 Plan 04 complete
**Last Updated:** 2026-02-14

## What is Token Budget Management?

Coordination protocols (heartbeat health checks, daily standups, session messages) use LLM tokens to function. Token budget management ensures **coordination never costs more than it's worth** by tracking overhead and automatically scaling back when needed.

Think of it as a safety valve: agents coordinate when it helps productivity, but coordination shuts off automatically if it becomes a tax on the system.

## The 30% Rule

By default, coordination overhead is limited to **30% of total token usage**.

**Example:**
- Your agents complete 100 tasks/day, using 5,000,000 tokens (production work).
- Heartbeat health checks use 720,000 tokens/day.
- Overhead: 720k / 5.72M * 100 = **12.6%** ✅ (well under 30%)

If overhead exceeds 30%, protocols **automatically degrade** to reduce cost.

## How Auto-Degradation Works

When overhead exceeds the 30% threshold, AOF automatically scales back coordination through 5 levels:

```
Full → Standard → Reduced → HeartbeatOnly → Disabled
```

### Mode Levels

| Mode | Protocols Active | When Used |
|------|-----------------|-----------|
| **Full** | Heartbeat + Standup + Roundtables + Messages | Normal operation, <30% overhead |
| **Standard** | Heartbeat + Standup + Messages (no roundtables) | Light reduction, still <30% overhead |
| **Reduced** | Heartbeat at 60s + Messages (no standup) | Overhead 30-40% |
| **HeartbeatOnly** | Health checks only (60s frequency) | Overhead 40-50% |
| **Disabled** | All coordination paused | Overhead >50% or agents mostly idle |

**Auto-degradation is reversible.** When overhead drops below 20% (recovery threshold), protocols gradually re-enable.

## Why the 20% Recovery Threshold?

Without a recovery threshold, the system would "flap" between modes:

```
Overhead: 29% → 31% → 29% → 31% → ...
Mode:     Full → Standard → Full → Standard → ... (BAD)
```

With a **20% recovery, 30% degrade** hysteresis:

```
Overhead: 29% → 31% → 29% → 25% → 19% → 21%
Mode:     Full → Standard → Standard → Standard → Full → Full (STABLE)
```

Between 20-30% = **no mode change** (hysteresis zone prevents flapping).

## Real-World Examples

### Scenario 1: Normal Workload (No Degradation)

**Setup:**
- 10 agents
- 500 tasks/day (2,500,000 production tokens)
- Heartbeat: 720,000 tokens/day
- Standup: 18,000 tokens/day

**Result:**
- Overhead: 738k / 3.24M = **22.8%** ✅
- Mode: **Full** (all protocols active)
- Cost: $0.40/day (Haiku pricing)

### Scenario 2: Light Workload (Auto-Degrade Triggered)

**Setup:**
- 10 agents
- 50 tasks/day (250,000 production tokens)
- Heartbeat: 720,000 tokens/day
- Standup: 18,000 tokens/day

**Before auto-degradation:**
- Overhead: 738k / 988k = **74.7%** ❌
- Cost: $0.12/day

**After auto-degradation:**
1. Overhead 74.7% → degrade to **Standard** (disables roundtables)
2. Still 74.7% → degrade to **Reduced** (disables standup)
3. Overhead: 720k / 970k = 74.2% → degrade to **HeartbeatOnly**
4. Still high → degrade to **Disabled** (all protocols paused)
5. Final overhead: **0%** ✅
6. Final cost: $0.03/day (production only, no coordination)

**Saved $0.09/day** by auto-degrading when agents were mostly idle.

### Scenario 3: Bursty Workload (Recovery)

**Morning (idle):**
- Overhead: 80% → mode degraded to **Disabled**

**Afternoon (busy - 1000 tasks):**
- Production tokens: 5,000,000
- Overhead: 0% (coordination disabled)
- Overhead drops below 20% threshold → **recovers to HeartbeatOnly**
- Next evaluation: 720k / 5.72M = 12.6% → **recovers to Full**
- All protocols re-enabled automatically

## Configuration

### Default Settings

```yaml
spec:
  coordination:
    enabled: true
    mode: full
    token_limits:
      max_overhead_percent: 30     # Degrade when overhead > 30%
      auto_degrade: true            # Enable automatic degradation
      recovery_threshold: 20        # Recover when overhead < 20%
```

### Adjust Thresholds

**More aggressive degradation (tighter budget):**
```yaml
token_limits:
  max_overhead_percent: 20     # Degrade at 20% instead of 30%
  recovery_threshold: 10       # Recover at 10%
```

**More lenient (allow higher coordination cost):**
```yaml
token_limits:
  max_overhead_percent: 50     # Allow 50% overhead
  recovery_threshold: 40       # Recover at 40%
```

**Disable auto-degradation (manual control only):**
```yaml
token_limits:
  auto_degrade: false          # No automatic degradation
```

### Per-Agent Modes (Advanced)

Override coordination mode for specific agents:

```yaml
spec:
  coordination:
    enabled: true
    mode: full  # Global default
    per_agent:
      k8s-monitor: full          # Critical agent, always coordinate
      batch-processor: disabled  # Batch job, no coordination needed
      log-analyzer: heartbeat_only  # Health checks only, no standup
```

**Use cases:**
- **Disabled:** Batch jobs, scheduled tasks (don't need health checks)
- **HeartbeatOnly:** Background workers (health monitoring, no status reports)
- **Full:** Critical agents (always coordinate)

## Monitoring

### View Current Metrics

```bash
curl http://localhost:8080/api/coordination/metrics
```

Response:
```json
{
  "coordination_tokens": 738000,
  "production_tokens": 2500000,
  "overhead_percent": 22.8,
  "heartbeat_tokens": 720000,
  "standup_tokens": 18000,
  "current_mode": "Full",
  "window_start": "2026-02-14T00:00:00Z",
  "auto_degrade_enabled": true,
  "max_overhead_percent": 30.0
}
```

### Manual Mode Override

Force coordination mode (bypasses auto-degradation):

```bash
curl -X POST http://localhost:8080/api/coordination/mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "heartbeat_only"}'
```

Response:
```json
{
  "success": true,
  "mode": "HeartbeatOnly",
  "message": "Coordination mode updated"
}
```

**Valid modes:** `"full"`, `"standard"`, `"reduced"`, `"heartbeat_only"`, `"disabled"`

**Note:** Manual override lasts until next auto-degradation evaluation (60 seconds). For permanent override, set `auto_degrade: false` in config.

## Cost Projections

### Daily Token Cost Estimates (10 agents, Haiku pricing)

| Work Level | Tasks/Day | Production Tokens | Heartbeat | Standup | Overhead % | Cost/Day |
|------------|-----------|------------------|-----------|---------|-----------|----------|
| Idle | 0 | 0 | 720,000 | 18,000 | 100% → **Disabled** | $0.00 |
| Light | 50 | 250,000 | 720,000 | 18,000 | 74.7% → **Disabled** | $0.03 |
| Normal | 500 | 2,500,000 | 720,000 | 18,000 | 22.8% ✅ | $0.40 |
| Heavy | 2000 | 10,000,000 | 720,000 | 18,000 | 6.7% ✅ | $1.30 |

**Key Insight:** Auto-degradation saves money when agents are idle or lightly used. Heavy workloads naturally dilute coordination overhead.

### Monthly Cost Projections

| Agent Count | Tasks/Month | Mode | Cost/Month (Haiku) |
|-------------|-------------|------|-------------------|
| 5 | 15,000 | Full | $6.00 |
| 10 | 15,000 | Full | $12.00 |
| 20 | 30,000 | Full | $24.00 |
| 10 | 1,500 (light) | Reduced/Disabled | $1.50 |

**Switch to Sonnet or Opus:** Multiply by 5-10x (but production work also scales proportionally, so overhead % stays similar).

## Best Practices

### When to Disable Coordination Entirely

Set `coordination.enabled: false` for:
- **Batch jobs** (cron-triggered, no long-running daemon)
- **Single-agent setups** (no coordination needed)
- **Development/testing** (reduce noise)

### When to Use Per-Agent Modes

- **Background workers:** `heartbeat_only` (just health checks)
- **Critical agents:** `full` (always coordinate, even if costly)
- **Scheduled tasks:** `disabled` (no coordination)

### When to Adjust Thresholds

- **Tight budget:** Lower `max_overhead_percent` to 20% (more aggressive degradation)
- **High availability needs:** Raise `max_overhead_percent` to 50% (keep coordination active)
- **Flapping observed:** Widen hysteresis gap (e.g., 10% recovery, 40% degrade)

## Troubleshooting

### Problem: Mode stuck at Disabled

**Cause:** Agents are idle or doing very light work. Coordination overhead dominates.

**Solutions:**
1. Increase production workload (more tasks).
2. Disable coordination entirely (`enabled: false`).
3. Set manual mode override (`POST /api/coordination/mode` → `"full"`).
4. Increase `max_overhead_percent` threshold to 50%.

### Problem: Mode flapping between Full and Standard

**Cause:** Overhead oscillating around 30% threshold.

**Solutions:**
1. Widen hysteresis gap (lower `recovery_threshold` to 15%).
2. Use manual mode override to fix at `full` or `standard`.
3. Disable auto-degradation (`auto_degrade: false`).

### Problem: Coordination disabled when I need it

**Cause:** Auto-degradation triggered due to high overhead.

**Solutions:**
1. Check metrics: `GET /api/coordination/metrics` (see actual overhead %).
2. Increase production workload (coordination overhead will drop).
3. Disable auto-degradation (`auto_degrade: false`).
4. Raise `max_overhead_percent` to 50%.

### Problem: Coordination costs too much

**Cause:** Agents are idle, but coordination still running.

**Solutions:**
1. Enable auto-degradation (`auto_degrade: true`) — it will disable protocols when idle.
2. Reduce heartbeat frequency to 120s or 300s (`heartbeat.frequency_secs: 300`).
3. Disable standup for non-critical agents (`per_agent.batch-job: heartbeat_only`).
4. Use per-agent `disabled` mode for batch jobs.

## FAQ

### Q: Does auto-degradation affect reliability?

**A:** No. Degradation only disables coordination protocols (status reports, health checks), not core agent functionality. Agents continue executing tasks normally at all degradation levels.

Disabled coordination means:
- No heartbeat health checks (agents assumed healthy)
- No daily standup reports
- Messages still work (session tools always active)

### Q: What happens if I manually override mode?

**A:** Manual override lasts until next auto-degradation evaluation (60 seconds). To make it permanent, set `auto_degrade: false` in config.

### Q: Can I disable auto-degradation for specific agents?

**A:** Yes, use `per_agent` config:
```yaml
per_agent:
  critical-agent: full  # Never degrades
  batch-job: disabled   # Never coordinates
```

### Q: How often does auto-degradation evaluate?

**A:** Every 60 seconds. Configurable via `check_interval` (internal setting, not exposed in serve-config.yaml yet).

### Q: What's the difference between "overhead" and "cost"?

**A:**
- **Overhead %** = coordination tokens / total tokens (30% means coordination is 30% of LLM usage)
- **Cost** = total tokens × LLM pricing (includes both coordination and production)

Auto-degradation targets overhead %, but **reduces total cost** as a side effect.

### Q: Why 30% as the default threshold?

**A:** User requirement: "Aggressive token budget." 30% is a conservative limit ensuring coordination never dominates production work. Most systems run at 5-15% overhead in normal operation.

### Q: Can I track overhead in CloudWatch/Grafana?

**A:** Yes, scrape `GET /api/coordination/metrics` every minute and graph `overhead_percent`, `current_mode`, and token breakdowns. Alerts recommended at 25% (approaching threshold).

---

**See also:**
- [Coordination Protocols Overview](./coordination-protocols.md) — User guide to heartbeat, standup, and session tools
- [Heartbeat Monitoring](./heartbeat-monitoring.md) — Health check configuration and troubleshooting
- [Developer Documentation](../dev/coordination-protocols.md) — Internal architecture and testing strategy
