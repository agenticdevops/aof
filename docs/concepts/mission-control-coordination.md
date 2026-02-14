# Mission Control: Coordination Dashboard

**Status:** Phase 7 Plan 05 complete
**Last Updated:** 2026-02-14

## Overview

The **Coordination Dashboard** in Mission Control provides real-time visibility into how your agents coordinate, report status, and manage token overhead. Think of it as a "team dashboard" for your AI workforce - you can see who's online, what everyone is working on, and how much coordination is costing.

## Accessing the Dashboard

1. Start the AOF daemon with coordination enabled:
   ```bash
   aofctl serve --config serve-config.yaml
   ```

2. Open Mission Control in your browser:
   ```
   http://localhost:8080
   ```

3. Navigate to the **Coordination** page from the sidebar.

## Dashboard Sections

### 1. Coordination Status Bar (Top)

The status bar shows the current coordination mode and token overhead at a glance.

**Coordination Mode Badge:**
- **Full** (green) - All protocols enabled (heartbeat + standup + messages)
- **Standard** (blue) - Default balance (heartbeat + standup)
- **Reduced** (yellow) - Minimal coordination (heartbeat at 5-minute intervals)
- **HeartbeatOnly** (orange) - Only heartbeat protocol active
- **Disabled** (red) - All coordination paused

**Token Overhead Gauge:**
- Visual bar showing what percentage of total tokens are spent on coordination
- Threshold line at 30% (the budget limit)
- Color-coded:
  - Green: < 20% (healthy)
  - Yellow: 20-30% (approaching threshold)
  - Red: > 30% (auto-degradation active)

**Token Breakdown:**
- Heartbeat: Tokens spent on health checks
- Standup: Tokens spent on daily status reports
- Production: Tokens spent on actual agent work

**Auto-Degrade Indicator:**
- Shows whether automatic mode degradation is enabled
- When enabled, the system automatically scales back coordination when overhead exceeds 30%

**Manual Mode Override:**
- Click the mode badge to open a dropdown
- Select a different mode to manually override
- Useful for forcing reduced coordination during high-cost periods

### 2. Heartbeat Dashboard (Left Panel)

Shows the health status of all agents with coordination enabled.

**Summary Bar:**
- **X/Y healthy** - Number of healthy agents out of total
- **N degraded** - Agents with slow responses (future feature)
- **M unresponsive** - Agents that missed their heartbeat timeout
- **Last check** - How long ago the most recent heartbeat was received

**Agent Health Cards:**

Each card shows:
- **Status dot** (green/yellow/red) with pulsing animation for unresponsive agents
- **Agent name/ID**
- **Status text** (Healthy, Degraded, Unresponsive)
- **Last heartbeat time** (relative: "5s ago", "2m ago")
- **Response latency** (in milliseconds, if available)
- **Consecutive misses** (shown in red if > 0)
- **Degraded reason** (if status is Degraded)

**Understanding Status:**
- **Healthy** - Agent responded to heartbeat within timeout (default: 120 seconds)
- **Degraded** - Reserved for future use (slow responses, partial failures)
- **Unresponsive** - Agent failed to respond to heartbeat timeout

**Real-time Updates:**
- Health cards update automatically via WebSocket
- No manual refresh needed
- Red pulsing dot draws attention to unresponsive agents

### 3. Standup Feed (Right Panel)

Displays daily standup results in chronological order.

**Header:**
- **Date and time** of the standup (e.g., "Friday, February 14, 2026 - 9:00 AM")
- **Trigger Standup Now** button for manual triggers
- **AI-generated summary** (when summarization is enabled)

**Agent Response Cards:**

Each card contains:
- **Agent avatar and name**
- **Timestamp** of the response
- **Token count** badge (shows how many tokens this response used)
- **Expand/collapse button** to show/hide details

**Expanded Response:**
- **DID section** (green) - What the agent completed since last standup
- **DOING section** (blue) - What the agent is currently working on
- **BLOCKERS section** (red or gray) - Any impediments or "No blockers"

**Empty State:**
- If no standup results exist, shows a message and trigger button
- "No standup results yet. Trigger a standup to see what your agents are working on."

**Real-time Updates:**
- As standup responses arrive via WebSocket, they appear progressively
- Summary updates after all agents have responded

## Example Scenarios

### Scenario 1: Morning Standup Review

**You open Mission Control at 9:30 AM:**
1. Coordination Status shows **Standard** mode with **12% overhead** (green)
2. Heartbeat Dashboard shows **5/6 agents healthy**, 1 unresponsive (red pulsing dot)
3. Standup Feed shows today's standup triggered at 9:00 AM
4. You expand the k8s-monitor response:
   - **DID:** "Resolved 3 pod restart incidents in production namespace"
   - **DOING:** "Investigating slow response times in api-gateway"
   - **BLOCKERS:** "Waiting for access to Grafana metrics"
5. You notice the log-analyzer agent is unresponsive:
   - Last heartbeat: "10m ago"
   - Consecutive misses: 3
   - You investigate and restart the agent

### Scenario 2: High Token Usage Alert

**You notice overhead is 28% (yellow):**
1. Coordination Status gauge shows yellow bar approaching 30% threshold
2. Token breakdown shows:
   - Heartbeat: 14K
   - Standup: 400
   - Production: 35K
3. You realize agents are mostly idle (low production tokens)
4. You click the **Standard** mode badge and select **Reduced** to lower coordination frequency
5. Overhead drops to 15% (green)
6. You plan to re-enable Standard mode when production work increases

### Scenario 3: Agent Goes Unresponsive

**An agent stops responding:**
1. HeartbeatDashboard shows "alert-manager" card with red pulsing dot
2. Status: **Unresponsive**
3. Last heartbeat: "5m ago"
4. Consecutive misses: 2
5. You check logs and find the agent crashed
6. After restarting, the card turns green within 60 seconds (next heartbeat)

## Configuration

Enable coordination in your `serve-config.yaml`:

```yaml
spec:
  coordination:
    enabled: true
    mode: standard  # or: full, reduced, heartbeat_only, disabled

    heartbeat:
      frequency_secs: 60   # How often to check agent health
      timeout_secs: 120    # When to mark unresponsive

    standup:
      schedule_cron: "0 9 * * *"  # Daily at 9 AM (local time)
      timezone: "America/New_York"
      summarize: true              # Enable AI summarization

    auto_degrade:
      enabled: true               # Automatically reduce coordination when overhead > 30%
      max_overhead_percent: 30    # Threshold for degradation
      recovery_threshold: 20      # Threshold for recovery (hysteresis)
```

## Coordination Modes Explained

### Full Mode
- **Heartbeat:** Every 30 seconds
- **Standup:** Daily
- **Session Messages:** Enabled
- **Overhead:** ~30%
- **Use Case:** Mission-critical systems where maximum visibility is needed

### Standard Mode (Default)
- **Heartbeat:** Every 60 seconds
- **Standup:** Daily
- **Session Messages:** Enabled
- **Overhead:** ~20%
- **Use Case:** Most production deployments (balanced visibility and cost)

### Reduced Mode
- **Heartbeat:** Every 5 minutes
- **Standup:** Disabled
- **Session Messages:** Enabled
- **Overhead:** ~10%
- **Use Case:** Cost-sensitive deployments, batch processing

### HeartbeatOnly Mode
- **Heartbeat:** Every 1 minute
- **Standup:** Disabled
- **Session Messages:** Disabled
- **Overhead:** ~5%
- **Use Case:** Minimal health monitoring only

### Disabled Mode
- **All protocols:** Disabled
- **Overhead:** 0%
- **Use Case:** Single-agent systems, development environments

## Understanding Token Overhead

**What is coordination overhead?**
Coordination overhead is the percentage of total LLM tokens spent on coordination protocols (heartbeat, standup, messages) versus production work (actual agent tasks).

**Formula:**
```
overhead_percent = (coordination_tokens / total_tokens) * 100
```

**Example:**
- Coordination tokens: 15,000 (heartbeat + standup)
- Production tokens: 100,000 (agent tasks)
- Total tokens: 115,000
- Overhead: 15,000 / 115,000 = **13%** ✅ (healthy)

**30% Budget:**
The 30% threshold ensures coordination never dominates production work. If overhead exceeds 30%, auto-degradation kicks in to reduce coordination frequency.

**Hysteresis (20-30% range):**
To prevent mode flapping, the system only degrades when overhead exceeds 30% and only recovers when it drops below 20%. Between 20-30% is the "hysteresis zone" where no mode change occurs.

## Auto-Degradation in Action

**Scenario:** Agents are mostly idle, overhead climbs to 35%

1. **Standard mode** at 35% overhead → **auto-degrades to Reduced**
   - Standup disabled
   - Heartbeat frequency reduced to 5 minutes
   - Overhead drops to 18%

2. **Reduced mode** at 18% overhead → **no change** (hysteresis zone 20-30%)
   - System remains in Reduced mode

3. Production work increases, overhead drops to 12%

4. **Reduced mode** at 12% overhead → **recovers to Standard**
   - Standup re-enabled
   - Heartbeat frequency restored to 60 seconds

## Troubleshooting

### Dashboard shows "Coordination not enabled"

**Cause:** Coordination is disabled in config or daemon not started with coordination.

**Solution:**
1. Edit `serve-config.yaml` and set `coordination.enabled: true`
2. Restart daemon: `aofctl serve --config serve-config.yaml`

### Agent shows as Unresponsive

**Cause:** Agent crashed, hung, or network issue preventing heartbeat response.

**Solution:**
1. Check agent logs for crashes or errors
2. Verify agent is running and reachable
3. Restart agent if crashed
4. If network issue, investigate connectivity

### Overhead gauge is red (>30%)

**Cause:** Agents are mostly idle (low production tokens) or coordination frequency too high.

**Solutions:**
1. **Manual override:** Switch to Reduced or HeartbeatOnly mode
2. **Reduce heartbeat frequency:** Change `heartbeat.frequency_secs` to 120 or 300
3. **Disable standup:** Remove or disable `standup.schedule_cron`
4. **Wait for production work:** Auto-degradation will kick in automatically

### Standup Feed shows "No standup results yet"

**Cause:** Standup not triggered yet (scheduled time hasn't arrived) or standup disabled.

**Solution:**
1. Click **Trigger Standup Now** button to manually trigger
2. Check `standup.schedule_cron` in config (e.g., `"0 9 * * *"` = daily at 9 AM)
3. Verify standup not disabled in Reduced/HeartbeatOnly mode

### Mode badge stuck at Disabled

**Cause:** Overhead was extremely high (>30% even after degradation) or coordination manually disabled.

**Solution:**
1. Check if agents are doing production work (if idle, coordination overhead will always be 100%)
2. Increase production work or manually override to HeartbeatOnly
3. If coordination not needed, leave in Disabled mode

## Best Practices

1. **Start with Standard mode** - Balanced visibility and cost for most use cases
2. **Monitor overhead gauge** - Keep an eye on the yellow/red threshold
3. **Use manual override sparingly** - Let auto-degradation handle most cases
4. **Review standup results daily** - Catch blockers and misalignment early
5. **Investigate unresponsive agents quickly** - Red pulsing dot means something is wrong
6. **Adjust heartbeat frequency for your needs** - 60s is a good default, but 300s works for non-critical systems
7. **Enable AI summarization** - Makes standup results easier to digest
8. **Disable coordination for batch agents** - If an agent runs infrequently, set mode to Disabled per-agent

## Learn More

- [Coordination Protocols Architecture](../dev/coordination-protocols.md) - Internal developer documentation
- [Configuration Reference](../reference/serve-config.md) - All coordination config options
- [Troubleshooting Guide](../troubleshooting/coordination.md) - Common issues and solutions
- [Token Efficiency](../concepts/token-efficiency.md) - Understanding and optimizing token usage

---

**Next Steps:**
- Explore the [Agent Personas](./agent-personas.md) concept
- Learn about [Session Tools](./session-tools.md) for agent-to-agent messaging
- Read the [Mission Control Overview](./mission-control.md) for the full UI tour
