# Coordination Troubleshooting Guide

Solutions for common coordination protocol issues.

## Table of Contents

1. [Agent Shows as Unresponsive](#agent-shows-as-unresponsive)
2. [Standup Not Triggering](#standup-not-triggering)
3. [Token Overhead Too High](#token-overhead-too-high)
4. [Auto-Degradation Keeps Activating](#auto-degradation-keeps-activating)
5. [Messages Not Delivered Between Agents](#messages-not-delivered-between-agents)
6. [WebSocket Not Receiving Coordination Events](#websocket-not-receiving-coordination-events)
7. [False Positive Heartbeat Timeouts](#false-positive-heartbeat-timeouts)
8. [Coordination Costs Unexpectedly High](#coordination-costs-unexpectedly-high)

---

## Agent Shows as Unresponsive

**Symptom:**

```bash
curl http://localhost:8080/api/coordination/health
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "unresponsive",
      "consecutive_misses": 3
    }
  ]
}
```

### Common Causes

1. **Agent crashed or stopped**
2. **LLM API key missing/invalid**
3. **Coordination mode disabled for this agent**
4. **Agent is extremely slow to respond (>120s)**
5. **Network issues between agent and LLM**

### Diagnostic Steps

**1. Check if agent process is running:**

```bash
# Check daemon logs for agent lifecycle
aofctl logs | grep "k8s-monitor"

# Expected:
# [INFO] Agent k8s-monitor started
# [INFO] Agent k8s-monitor: Healthy (150ms)
```

**2. Check agent coordination mode:**

```bash
# Verify agent is not disabled
cat serve-config.yaml | grep -A 10 "per_agent" | grep "k8s-monitor"

# If shows:
# k8s-monitor: disabled
# Then coordination is intentionally off for this agent
```

**3. Check LLM API key:**

```bash
# Verify environment variable is set
echo $ANTHROPIC_API_KEY

# Test LLM connectivity manually
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{
    "model": "claude-haiku-20240307",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "test"}]
  }'
```

**4. Check logs for heartbeat responses:**

```bash
# Filter for heartbeat events
aofctl logs | grep "Heartbeat"

# Healthy agent shows:
# [DEBUG] Heartbeat tick: request_id = abc123
# [INFO] Agent k8s-monitor: Healthy (150ms)

# Unresponsive agent shows:
# [DEBUG] Heartbeat tick: request_id = abc123
# [WARN] Agent k8s-monitor unresponsive (timeout: 120s)
```

### Solutions

**Agent crashed:**
```bash
# Restart daemon
aofctl serve --config serve-config.yaml
```

**LLM API key missing:**
```bash
# Set environment variable
export ANTHROPIC_API_KEY="your-key-here"
aofctl serve --config serve-config.yaml
```

**Coordination disabled:**
```yaml
# In serve-config.yaml, change:
coordination:
  per_agent:
    k8s-monitor: full  # Enable coordination
```

**Agent too slow:**
```yaml
# Increase timeout in serve-config.yaml
coordination:
  heartbeat:
    timeout_secs: 300  # 5 minutes instead of 120s
```

---

## Standup Not Triggering

**Symptom:**

Standup should trigger daily at 9 AM, but nothing happens.

### Common Causes

1. **Cron expression is incorrect**
2. **Timezone mismatch**
3. **Standup disabled in config**
4. **No agents with coordination mode that includes standup**

### Diagnostic Steps

**1. Check cron expression:**

```bash
# View current config
cat serve-config.yaml | grep -A 5 "standup"

# Expected:
# standup:
#   cron: "0 0 9 * * *"
#   timezone: "America/New_York"
```

**2. Test cron expression:**

Use online cron tester (e.g., crontab.guru) but note AOF uses **6-field format** (with seconds):

| Field | Value | Meaning |
|-------|-------|---------|
| 1 | `0` | Second (0-59) |
| 2 | `0` | Minute (0-59) |
| 3 | `9` | Hour (0-23) |
| 4 | `*` | Day of month |
| 5 | `*` | Month |
| 6 | `*` | Day of week |

**3. Check timezone:**

```bash
# Verify timezone is valid
cat serve-config.yaml | grep "timezone"

# Common mistake: Using abbreviations instead of full IANA names
# ❌ Bad: timezone: "EST"
# ✅ Good: timezone: "America/New_York"
```

**4. Check agent coordination modes:**

```bash
# Verify at least one agent has Full or Standard mode
cat serve-config.yaml | grep -A 20 "per_agent"

# Standup runs for:
# - full: ✅
# - standard: ✅
# - reduced: ❌
# - heartbeat_only: ❌
# - disabled: ❌
```

**5. Manual trigger test:**

```bash
# Trigger standup manually to test if it works
curl -X POST http://localhost:8080/api/coordination/standup/trigger

# View result
curl http://localhost:8080/api/coordination/standup/latest
```

### Solutions

**Cron expression fix:**

```yaml
# Common fixes:
standup:
  cron: "0 0 9 * * *"        # 9:00 AM daily
  cron: "0 30 8 * * MON-FRI" # 8:30 AM weekdays only
  cron: "0 0 */4 * * *"      # Every 4 hours
```

**Timezone fix:**

```yaml
# Use full IANA timezone names
standup:
  timezone: "America/New_York"     # Not "EST"
  timezone: "America/Los_Angeles"  # Not "PST"
  timezone: "Europe/London"        # Not "GMT"
```

**Enable standup for agents:**

```yaml
coordination:
  mode: full  # Global default includes standup

  per_agent:
    k8s-monitor: full     # ✅ Will participate in standup
    log-analyzer: standard # ✅ Will participate in standup
    batch-job: reduced    # ❌ Will NOT participate
```

**Restart daemon after config changes:**

```bash
# Kill existing daemon
pkill aofctl

# Restart with new config
aofctl serve --config serve-config.yaml
```

---

## Token Overhead Too High

**Symptom:**

```bash
curl http://localhost:8080/api/coordination/metrics
{
  "overhead_percent": 45.0,
  "current_mode": "reduced"
}
```

Auto-degradation activated (mode changed from `full` to `reduced`).

### Common Causes

1. **Light production workload** (low denominator in overhead calculation)
2. **Too many agents** (each agent adds coordination cost)
3. **High-frequency heartbeat** (30s instead of 60s)
4. **Standup summarization enabled** (adds 500-1000 tokens/standup)

### Diagnostic Steps

**1. Check token breakdown:**

```bash
curl http://localhost:8080/api/coordination/metrics | jq '.'
{
  "coordination_tokens": 500000,
  "production_tokens": 1000000,
  "overhead_percent": 50.0,
  "heartbeat_tokens": 400000,
  "standup_tokens": 100000
}
```

**2. Calculate daily heartbeat cost:**

```
heartbeat_tokens_per_day = agent_count × (1440 minutes / frequency_minutes) × 50 tokens

# Example: 10 agents, 60-second frequency
= 10 × (1440 / 1) × 50
= 720,000 tokens/day
```

**3. Check production workload:**

```bash
# If production tokens are low, overhead appears high
# This is expected for idle/test systems
```

### Solutions

**1. Increase heartbeat interval:**

```yaml
coordination:
  heartbeat:
    frequency_secs: 120  # 2 minutes instead of 60s
    timeout_secs: 300    # 5 minutes (2.5x frequency)
```

Cost reduction: **50%** (half the heartbeat checks).

**2. Disable standup summarization:**

```yaml
coordination:
  standup:
    summarize: false  # Save 500-1000 tokens/standup
```

Cost reduction: **~5-10% daily** (depends on agent count).

**3. Reduce agent count:**

If you have 20 agents but only 5 are active, consider:

```yaml
coordination:
  per_agent:
    # Active agents: full coordination
    k8s-monitor: full
    alert-triage: full

    # Inactive agents: disable coordination
    unused-agent-1: disabled
    unused-agent-2: disabled
```

**4. Use reduced mode for background agents:**

```yaml
coordination:
  per_agent:
    # Mission-critical: full coordination
    k8s-monitor: full

    # Background workers: heartbeat only
    data-processor: reduced
    report-generator: reduced
```

**5. Increase overhead budget (not recommended):**

```yaml
coordination:
  token_limits:
    max_overhead_percent: 50  # Allow up to 50%
    recovery_threshold_percent: 40
```

⚠️ **Only use this if coordination is more valuable than production work.**

---

## Auto-Degradation Keeps Activating

**Symptom:**

Mode keeps changing: `full` → `reduced` → `full` → `reduced` (flapping).

### Common Cause

**Hysteresis gap too narrow** or **workload oscillating around threshold**.

### Diagnostic Steps

**1. Check current thresholds:**

```bash
cat serve-config.yaml | grep -A 5 "token_limits"

# Expected:
# token_limits:
#   max_overhead_percent: 30
#   recovery_threshold_percent: 20
```

**2. Monitor overhead over time:**

```bash
# Poll metrics every 60s
watch -n 60 'curl -s http://localhost:8080/api/coordination/metrics | jq ".overhead_percent, .current_mode"'

# If values oscillate:
# 32%, "reduced"
# 18%, "standard"
# 31%, "reduced"
# Then hysteresis is insufficient
```

### Solutions

**1. Widen hysteresis gap:**

```yaml
coordination:
  token_limits:
    max_overhead_percent: 30
    recovery_threshold_percent: 15  # Wider gap (30% - 15% = 15%)
```

Prevents mode changes between 15-30% overhead.

**2. Disable auto-degradation:**

```yaml
coordination:
  token_limits:
    auto_degrade: false  # Manual mode control only
```

⚠️ **Coordination can exceed 30% budget** with this setting.

**3. Force mode manually:**

```bash
# Force reduced mode permanently
curl -X POST http://localhost:8080/api/coordination/mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "reduced"}'
```

**4. Increase production workload:**

If overhead is high because production work is low, run more production tasks:

```bash
# Schedule more agent tasks
aofctl agent run k8s-monitor --task "analyze-pods"
```

---

## Messages Not Delivered Between Agents

**Symptom:**

Agent A sends message to Agent B, but Agent B never receives it.

### Common Causes

1. **Agent B not registered in session tools**
2. **Message queue full** (100-message capacity exceeded)
3. **Message TTL expired** (default 30 minutes)
4. **Agent B crashed before draining messages**

### Diagnostic Steps

**1. Check agent registration:**

```bash
# Verify both agents registered
aofctl logs | grep "register_agent"

# Expected:
# [INFO] Registered agent: agent-a (mode: full)
# [INFO] Registered agent: agent-b (mode: full)
```

**2. Check message queue status:**

```bash
# Check for QueueFull errors
aofctl logs | grep "QueueFull"

# If found:
# [WARN] Message queue full for agent-b (capacity: 100)
```

**3. Check message TTL:**

```bash
# Messages expire after 30 minutes (default)
# If Agent B is offline for >30 min, messages dropped
```

### Solutions

**1. Ensure agent registration:**

```rust
// In agent code, register on startup:
coordination_manager.register_agent("agent-b", CoordinationMode::Full).await?;
```

**2. Increase queue capacity:**

```rust
// In daemon initialization:
let session_tools = SessionTools::new(
    200,  // Increase from 100 to 200
    Duration::from_secs(30 * 60),
);
```

**3. Drain messages more frequently:**

```rust
// In agent code:
let messages = session_tools.drain_messages("agent-b").await;
for message in messages {
    process_message(message).await?;
}
```

**4. Increase TTL for long-running tasks:**

```rust
// When sending messages:
let message = SessionMessage::new(
    "agent-a",
    "agent-b",
    MessageType::TaskAssignment,
    "Analyze logs",
    Duration::from_secs(2 * 60 * 60),  // 2 hours instead of 30 min
);
```

---

## WebSocket Not Receiving Coordination Events

**Symptom:**

Mission Control UI doesn't show heartbeat events or agent health updates.

### Common Causes

1. **WebSocket connection not established**
2. **Event types filtered** (UI only subscribes to specific events)
3. **Mission Control UI version mismatch**
4. **Browser caching old JavaScript**

### Diagnostic Steps

**1. Check WebSocket connection:**

```javascript
// Open browser console (F12)
// Look for WebSocket connection
ws://localhost:8080/ws

// Should see:
WebSocket connection established
```

**2. Check event flow:**

```bash
# Subscribe to events via CLI
websocat ws://localhost:8080/ws

# Should see JSON events:
{"activity": {"type": "info"}, "coordination_activity": {"type": "HeartbeatRequest"}}
```

**3. Check browser console for errors:**

```javascript
// Common errors:
WebSocket connection failed: net::ERR_CONNECTION_REFUSED
  → Daemon not running

CORS error
  → Wrong origin

Event format mismatch
  → UI version doesn't match daemon version
```

### Solutions

**1. Ensure daemon is running:**

```bash
aofctl serve --config serve-config.yaml
```

**2. Hard refresh browser:**

```
Ctrl+F5 (Windows/Linux)
Cmd+Shift+R (Mac)
```

Clears cached JavaScript.

**3. Check Mission Control version:**

```bash
# Verify UI and daemon versions match
aofctl --version

# Update UI:
cd mission-control-ui
npm install
npm run build
```

**4. Test WebSocket manually:**

```bash
# Install websocat
brew install websocat  # Mac
apt install websocat   # Linux

# Connect to WebSocket
websocat ws://localhost:8080/ws

# Should receive events in real-time
```

---

## False Positive Heartbeat Timeouts

**Symptom:**

Agent marked unresponsive even though it's running and healthy.

**Log shows:**

```
[WARN] Agent k8s-monitor unresponsive (timeout: 120s)
```

But agent is active and processing tasks.

### Common Causes

1. **Timeout too aggressive for slow agents**
2. **LLM API latency spikes** (network delays)
3. **Agent overloaded** (processing heavy task during heartbeat)

### Diagnostic Steps

**1. Check response times:**

```bash
curl http://localhost:8080/api/coordination/health | jq '.agents[] | select(.agent_id == "k8s-monitor")'
{
  "agent_id": "k8s-monitor",
  "status": "healthy",
  "last_response_ms": 3500  # 3.5 seconds (fast)
}
```

**2. Monitor response time over time:**

```bash
# Poll every 60s
watch -n 60 'curl -s http://localhost:8080/api/coordination/health | jq ".agents[] | select(.agent_id == \"k8s-monitor\") | .last_response_ms"'

# If values spike:
# 150ms
# 200ms
# 8000ms  ← Spike!
# 150ms
```

**3. Check concurrent task execution:**

```bash
# If agent is running heavy task during heartbeat, response may be delayed
aofctl logs | grep "k8s-monitor" | grep -E "tool_executing|task_started"
```

### Solutions

**1. Increase timeout for specific agent:**

Currently not supported per-agent. Increase globally:

```yaml
coordination:
  heartbeat:
    timeout_secs: 300  # 5 minutes instead of 120s
```

**2. Reduce heartbeat frequency:**

```yaml
coordination:
  heartbeat:
    frequency_secs: 120  # 2 minutes instead of 60s
    timeout_secs: 300    # 5 minutes
```

**3. Investigate LLM latency:**

```bash
# Test LLM API directly
time curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{
    "model": "claude-haiku-20240307",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "Are you alive?"}]
  }'

# Should complete in <2 seconds
# If >10 seconds, network/API issue
```

**4. Profile agent performance:**

If agent is consistently slow, optimize agent code or reduce task load.

---

## Coordination Costs Unexpectedly High

**Symptom:**

Monthly LLM bill higher than expected, coordination identified as culprit.

### Diagnostic Steps

**1. Get token breakdown:**

```bash
curl http://localhost:8080/api/coordination/metrics | jq '.'
{
  "coordination_tokens": 5000000,
  "production_tokens": 15000000,
  "heartbeat_tokens": 4000000,
  "standup_tokens": 1000000
}
```

**2. Calculate daily cost:**

```
# Heartbeat cost
heartbeat_cost = (heartbeat_tokens / 1M) × $0.25 (Haiku input price)

# Example: 4M tokens over 30 days
= (4M / 1M) × $0.25
= $1.00 for heartbeat

# Standup cost (with summarization)
standup_cost = (standup_tokens / 1M) × $3.00 (Sonnet input price)
= (1M / 1M) × $3.00
= $3.00 for standup
```

**3. Identify most expensive protocol:**

```bash
# If heartbeat_tokens >> standup_tokens:
#   → Reduce heartbeat frequency
# If standup_tokens >> heartbeat_tokens:
#   → Disable summarization or reduce standup frequency
```

### Solutions

**1. Reduce heartbeat frequency:**

```yaml
coordination:
  heartbeat:
    frequency_secs: 300  # 5 minutes instead of 60s
```

Cost reduction: **~80%** (1/5 the checks).

**2. Disable standup summarization:**

```yaml
coordination:
  standup:
    summarize: false
```

Cost reduction: **~95% of standup cost** (Haiku vs. Sonnet).

**3. Reduce agent count:**

```yaml
coordination:
  per_agent:
    # Only coordinate mission-critical agents
    k8s-monitor: full
    alert-triage: full

    # Disable for batch/background agents
    report-generator: disabled
    data-processor: disabled
```

**4. Switch to reduced mode globally:**

```yaml
coordination:
  mode: reduced  # Heartbeat only, no standup
```

**5. Monitor costs with alerts:**

```bash
# Set up daily cost alert
DAILY_COORD_TOKENS=$(curl -s http://localhost:8080/api/coordination/metrics | jq '.coordination_tokens')
DAILY_COST=$(echo "scale=2; $DAILY_COORD_TOKENS / 1000000 * 0.25" | bc)

if (( $(echo "$DAILY_COST > 5.00" | bc -l) )); then
  echo "ALERT: Coordination costs $${DAILY_COST}/day"
fi
```

---

## Need More Help?

- **Setup Guide**: [Coordination Setup](coordination-setup.md)
- **Developer Docs**: [Internal Documentation](../dev/coordination-protocols.md)
- **Concepts**: [Coordination Protocols Overview](../concepts/coordination-protocols.md)
- **GitHub Issues**: [Report a Bug](https://github.com/agenticdevops/aof/issues)
