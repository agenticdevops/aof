# Heartbeat Monitoring - Keeping Agents Alive

**Status:** Available in v0.4.0-beta
**Last Updated:** 2026-02-14
**Category:** Coordination Protocols

## What is Heartbeat Monitoring?

Heartbeat monitoring automatically checks if your agents are alive and responsive. Every 60 seconds, AOF sends a lightweight "Are you alive?" check to each agent. If an agent doesn't respond within 120 seconds, it's marked as **unresponsive** and you'll see an alert in Mission Control.

This is like a **wellness check for your AI team** — you always know which agents are healthy and which need attention.

## How It Works

```
┌─────────────────────────────────────────────────┐
│  Every 60 seconds:                              │
│                                                 │
│  AOF ──> "Are you alive?" ──> All Agents        │
│                                                 │
│  Agents respond within 120s ───> "I'm healthy" │
│                                                 │
│  No response after 120s? ───> Alert: Agent X   │
│                                  is unresponsive│
└─────────────────────────────────────────────────┘
```

**Simple:** AOF tracks agent health automatically. No manual checks needed.

## Agent Health Statuses

| Status | Meaning | What to Do |
|--------|---------|------------|
| **Healthy** | Agent responded to heartbeat | Nothing - agent is working fine |
| **Unresponsive** | Agent missed heartbeat timeout (120s) | Check agent logs, restart if needed |
| **Degraded** | Reserved for future use (slow responses) | Coming in future versions |

## Configuration

### Enable Heartbeat in serve-config.yaml

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: my-daemon
spec:
  coordination:
    enabled: true           # Turn on coordination protocols
    mode: full              # full, standard, reduced, heartbeat_only, or disabled
    heartbeat:
      frequency_secs: 60    # Check every 60 seconds (default)
      timeout_secs: 120     # Mark unresponsive after 120 seconds (default, must be >= frequency)
```

### Coordination Modes

Choose how much coordination you want per agent:

| Mode | Heartbeat | Standup | Roundtables | Use Case |
|------|-----------|---------|-------------|----------|
| **full** | ✅ | ✅ | ✅ | Production agents needing full coordination |
| **standard** | ✅ | ✅ | ❌ | Most agents (no roundtables) |
| **reduced** | ✅ (slower) | ❌ | ❌ | Low-priority agents, minimal overhead |
| **heartbeat_only** | ✅ | ❌ | ❌ | Just health checks, no status reports |
| **disabled** | ❌ | ❌ | ❌ | Zero coordination overhead (batch jobs) |

**Default:** `full` if not specified.

### Per-Agent Opt-In (Future)

In future versions, you'll be able to set coordination mode per agent in AGENTS.md:

```yaml
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    coordination_mode: full       # Always check this one

  - id: batch-processor
    name: Batch Report Generator
    coordination_mode: disabled   # Don't check - runs on schedule, not always online
```

For now, all agents use the global `spec.coordination.mode` setting.

## Viewing Health Status

### Via REST API

```bash
# Get current health status for all agents
curl http://localhost:8080/api/coordination/health | jq
```

**Response:**
```json
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "Healthy",
      "last_heartbeat": "2026-02-14T10:30:00Z",
      "consecutive_misses": 0,
      "last_response_ms": 1200
    },
    {
      "agent_id": "log-analyzer",
      "status": "Unresponsive",
      "last_heartbeat": "2026-02-14T10:28:30Z",
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

### Via Mission Control UI (Coming Soon)

Mission Control will show agent health cards with live status indicators:
- 🟢 Green: Healthy
- 🔴 Red: Unresponsive
- 🟡 Yellow: Degraded (future)

You'll also see heartbeat timeout alerts in the event stream.

## Cost

Heartbeat is **super cheap** because it uses the lightest Claude model (Haiku) and a static prompt:

```
Cost Calculation (10 agents, 60s frequency):
- Heartbeats per day: 10 agents × 1,440 minutes = 14,400 checks
- Tokens per check: ~50 (just "Are you alive?" - no context loaded)
- Total tokens/day: 720,000
- Cost (Haiku pricing): ~$0.01/day
```

**For 10 agents, heartbeat monitoring costs about 1 penny per day.**

Compare to 30-second frequency:
- Tokens/day: 1,440,000 (2x)
- Cost: ~$0.02/day

AOF defaults to 60s to balance responsiveness with cost.

## Why Super-Lightweight?

Heartbeat is designed to be **token-efficient**:

| What Heartbeat DOESN'T Load | Why |
|------------------------------|-----|
| ❌ AGENTS.md | No agent config loaded |
| ❌ SOUL.md | No personality loaded |
| ❌ Agent memories | No conversation history |
| ❌ Skills | No skill definitions |

**Heartbeat prompt:**
```
System: Are you alive?
Agent: Yes, I am alive and ready to work.
```

That's it. ~50 tokens total.

This keeps coordination overhead **under 5% of total tokens** (well below the 30% target).

## Troubleshooting

### Agent Marked Unresponsive (False Positive)

**Symptom:** Agent is actually working, but shows as unresponsive.

**Possible Causes:**
1. **Timeout too aggressive** - 120s might be too short if agent is busy with long-running task
2. **Network delay** - Check network latency between agents and daemon
3. **High CPU load** - Agent process starved, can't respond in time

**Fix:**
```yaml
# Increase timeout (must be >= frequency)
heartbeat:
  frequency_secs: 60
  timeout_secs: 180    # Give agents more time (3x frequency)
```

Or switch to `reduced` mode for lower-frequency checks.

### Agent Stuck, Not Actually Responsive

**Symptom:** Agent responds to heartbeat but doesn't do actual work.

**Diagnosis:** Heartbeat only checks liveness, not functional health. Agent might be alive but deadlocked or stuck in a bad state.

**Future Enhancement:** Plan 04 (Token Metrics) will track production work vs. coordination overhead. An agent that heartbeats but never does work will show 0% production tokens.

### Too Many Heartbeat Events in WebSocket

**Symptom:** WebSocket stream flooded with HeartbeatRequest/Response events.

**Fix:** Filter coordination events on the client side:
```javascript
// React Mission Control
const productionEvents = events.filter(e =>
  !e.coordination_activity ||
  e.coordination_activity.type === "HeartbeatTimeout" // Only show alerts
);
```

Or reduce frequency:
```yaml
heartbeat:
  frequency_secs: 300   # Check every 5 minutes instead of 1
  timeout_secs: 600
```

### Coordination Disabled But Seeing Errors

**Symptom:** Coordination logs show errors even though `enabled: false`.

**Diagnosis:** Coordination config missing or malformed.

**Fix:**
```yaml
spec:
  coordination:
    enabled: false   # Explicitly disable
```

Or remove the `coordination` section entirely (defaults to disabled).

## Best Practices

### Production Deployments

- **Use `full` or `standard` mode** for always-on agents
- **Use `heartbeat_only` mode** for agents that don't need status reports
- **Use `disabled` mode** for batch jobs or agents that run on schedule

### Development/Testing

- **Use `disabled` mode** to reduce noise during local testing
- **Enable coordination** only when testing Mission Control UI or multi-agent coordination

### Cost Optimization

- **60s frequency** is usually sufficient (agents can recover within 2 minutes)
- **120s timeout** allows for 1 missed heartbeat before alert (reduces false positives)
- **`reduced` mode** for low-priority agents cuts heartbeat frequency in half
- **`disabled` mode** for batch agents eliminates coordination overhead entirely

## What's Next?

Heartbeat is part of a larger **Coordination Protocols** suite:

- ✅ **Heartbeat** (Plan 02) - You are here
- 🚧 **Standup Reports** (Plan 03) - Daily status summaries ("What I did, what I'm doing, blockers")
- 🚧 **Token Metrics** (Plan 04) - Track coordination overhead, alert if >30%
- 🚧 **Roundtables** (Plan 05) - Multi-agent discussion for complex decisions

All protocols are **opt-in via CoordinationMode** — you choose the level of coordination vs. token cost trade-off that works for your use case.

---

**Questions?** See [Coordination Protocols Concept Doc](./coordination-protocols.md) or [Internal Developer Docs](../dev/coordination-protocols.md).
