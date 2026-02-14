# Coordination Protocols

**Phase:** 7 (Coordination Protocols)
**Status:** Session Tools Foundation Complete

## What are Coordination Protocols?

Coordination protocols enable your agents to work together as a team without constant human intervention. Instead of just executing tasks in isolation, agents can:

- **Monitor each other's health** — detect when an agent becomes unresponsive
- **Report status proactively** — daily standups just like a human team
- **Communicate asynchronously** — send messages to request help or delegate work
- **Track coordination costs** — ensure coordination overhead stays reasonable

Think of it as giving your agents a "virtual office" where they can coordinate, report blockers, and escalate issues to humans when needed.

## Coordination Modes

Coordination is **opt-in per agent**. You choose how much coordination each agent participates in:

### Mode Comparison

| Mode | When to Use | Features | Token Overhead |
|------|-------------|----------|----------------|
| **Full** | Critical agents that need tight coordination | Heartbeat + Standup + Messages + Roundtables | ~30% |
| **Standard** | Most agents (default) | Heartbeat + Standup + Messages | ~20% |
| **Reduced** | Background agents with loose coordination | Heartbeat (5min) + Messages | ~10% |
| **HeartbeatOnly** | Simple health monitoring | Just heartbeat checks (1min) | ~5% |
| **Disabled** | Batch jobs, one-off tasks | No coordination | 0% |

**Default mode:** `Standard` (heartbeat + standup + messages, no roundtables)

### Configuration Example

```yaml
# agents.yaml
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    coordination_mode: full        # Critical infrastructure agent

  - id: incident-triage
    name: Incident Triage Agent
    coordination_mode: standard    # Default balance

  - id: batch-processor
    name: Nightly Batch Processor
    coordination_mode: disabled    # No coordination needed

  - id: log-analyzer
    name: Log Analysis Agent
    coordination_mode: reduced     # Background analysis
```

## Session Messages (Available Now)

Agents can send messages to each other asynchronously. Messages are **fire-and-forget** — the sender doesn't wait for a response.

### Message Types

| Type | Purpose | Example |
|------|---------|---------|
| **Announcement** | Inform other agents | "Starting cluster health check" |
| **CollaborationRequest** | Ask for help | "Need help analyzing logs for incident-001" |
| **TaskAssignment** | Delegate work | "Agent B, please analyze pod logs for default namespace" |
| **HumanEscalation** | Route to human | "SEV1 database outage — cannot auto-resolve" |

### How It Works

1. **Agent A** sends a message to **Agent B** via session tools
2. Message goes into **Agent B's inbox** (bounded queue, 100 messages max)
3. **Agent B** drains messages when ready (async, non-blocking)
4. Messages expire after **30 minutes** if not processed (configurable)

### Example: Agent Collaboration

```yaml
# Scenario: Incident response
# - Triage agent analyzes alert
# - Delegates to specialist agents
# - Specialists report findings
# - Triage escalates to human if needed

Triage Agent (receives alert):
  → Sends CollaborationRequest to Log Analyzer
  → Sends CollaborationRequest to Metrics Checker

Log Analyzer:
  → Receives message from Triage
  → Analyzes logs, finds errors
  → Sends Announcement back: "Found 50 errors in pod X"

Metrics Checker:
  → Receives message from Triage
  → Checks metrics, finds spike
  → Sends Announcement back: "CPU spike detected in node Y"

Triage Agent (receives findings):
  → Correlates findings
  → Confidence too low (< 60%)
  → Sends HumanEscalation: "Cannot determine root cause — needs human analysis"
```

## Heartbeat Monitoring (Coming in Plan 02)

Agents periodically report their health status.

### How It Works

1. **Heartbeat scheduler** sends `HeartbeatRequest` every 30 seconds
2. Agents respond with `HeartbeatResponse` (healthy, degraded, or silent)
3. If agent doesn't respond within **60 seconds** → marked as **unresponsive**
4. Unresponsive agents trigger alerts in Mission Control

### Agent Health States

- **Healthy** — Agent is responsive, normal operation
- **Degraded** — Agent is slow or partially functional (e.g., "High latency on LLM calls")
- **Unresponsive** — Agent didn't respond to heartbeat (timeout)

### Use Cases

- Detect stuck agents (infinite loop, deadlock)
- Monitor LLM provider outages (agent degraded if Anthropic API slow)
- Auto-restart agents that become unresponsive

## Daily Standups (Coming in Plan 03)

Agents report what they did, what they're doing, and any blockers — just like a human standup.

### How It Works

1. **Standup scheduler** triggers daily at configured time (e.g., 9am EST)
2. Agents respond with `StandupResponse`:
   - **What I did:** Recent accomplishments
   - **What I'm doing:** Current work
   - **Blockers:** Issues preventing progress
3. Responses are **aggregated** into a summary (LLM-generated)
4. Summary posted to **Mission Control** and messaging channels (Slack, Discord)

### Example Standup

```
Daily Standup Summary — 2026-02-14

K8s Monitor:
  ✅ What I did: Detected 3 pod crashes, restarted 2 automatically
  🔄 What I'm doing: Analyzing node resource utilization
  ⛔ Blockers: None

Incident Triage:
  ✅ What I did: Triaged 5 alerts (3 SEV3, 2 SEV4)
  🔄 What I'm doing: Investigating SEV2 database latency spike
  ⛔ Blockers: Need database credentials for prod cluster

Log Analyzer:
  ✅ What I did: Analyzed 1M log lines, found 12 error patterns
  🔄 What I'm doing: Idle (waiting for next incident)
  ⛔ Blockers: None

Summary: Team handled 5 incidents, 1 SEV2 in progress. Blocker: prod database creds.
```

## Token Metrics (Coming in Plan 04)

Coordination uses LLM tokens. We track how much is spent on coordination vs. production work.

### Overhead Target

**Goal:** <30% of tokens spent on coordination

- **Full mode:** ~30% overhead (acceptable for critical agents)
- **Standard mode:** ~20% overhead (good balance)
- **Reduced mode:** ~10% overhead (minimal coordination)

### What Counts as Coordination?

- Heartbeat responses (small — just health status)
- Standup reports (medium — structured text)
- Roundtables (large — multi-agent LLM conversations)

### Automatic Optimization

If an agent exceeds 30% coordination overhead, AOF can:
1. **Alert** — notify you in Mission Control
2. **Suggest fallback** — recommend switching to lower coordination mode
3. **Auto-adjust** — optionally reduce coordination mode automatically

## Cost Considerations

Coordination uses **Claude Haiku** for cheap operations (health checks, standup reports).

### Estimated Monthly Costs (per agent)

| Mode | Tokens/Day | Est. Cost/Month |
|------|------------|-----------------|
| **Full** | ~50,000 | ~$3/agent/month |
| **Standard** | ~30,000 | ~$2/agent/month |
| **Reduced** | ~10,000 | ~$0.50/agent/month |
| **HeartbeatOnly** | ~2,000 | ~$0.10/agent/month |
| **Disabled** | 0 | $0 |

**Note:** These are rough estimates. Actual costs depend on agent activity and LLM provider pricing.

## When to Use Each Mode

### Full Mode
- **Critical infrastructure agents** (K8s monitor, incident triage)
- **Agents that need tight coordination** (squad working on same incident)
- **When uptime is critical** (production monitoring)

### Standard Mode (Default)
- **Most agents** — good balance of coordination and efficiency
- **Daily operational agents** (log analysis, metric checks)
- **Agents that report status regularly**

### Reduced Mode
- **Background agents** (batch processors, nightly jobs)
- **Agents with loose SLAs** (weekly reports, periodic checks)
- **Cost-sensitive deployments**

### HeartbeatOnly Mode
- **Simple agents** that just need health monitoring
- **Agents without blockers** (stateless, independent work)
- **Testing/development environments**

### Disabled Mode
- **One-off tasks** (ad-hoc queries, manual interventions)
- **Batch jobs** (run once, don't need monitoring)
- **Cost-critical deployments** (every token counts)

## Configuration Reference

### Per-Agent Coordination

```yaml
agents:
  - id: my-agent
    coordination_mode: standard   # full | standard | reduced | heartbeat_only | disabled
```

### Global Coordination Settings

```yaml
# aofctl serve config
coordination:
  heartbeat_interval: 30s        # How often to check health
  heartbeat_timeout: 60s         # When to mark as unresponsive
  standup_schedule: "0 9 * * *"  # Cron: daily at 9am
  standup_timezone: "America/New_York"
  token_overhead_threshold: 0.30  # Alert if >30%
  message_queue_capacity: 100     # Max messages per agent
  message_ttl: 30m                # Message expiration
```

## Coming Features

### Phase 7 Roadmap

- ✅ **Plan 01: Session Tools** — Agent-to-agent messaging (complete)
- 🚧 **Plan 02: Heartbeat** — Health monitoring (next)
- 📅 **Plan 03: Standup** — Daily status reports
- 📅 **Plan 04: Token Metrics** — Coordination overhead tracking

### Phase 8: Roundtables

- **Multi-agent conversations** — agents discuss problems together
- **Consensus building** — vote on next steps
- **Collaborative debugging** — multiple agents analyze same issue

## FAQ

### Q: Do agents need to be running simultaneously for messages to work?

**A:** No. Messages are queued. Agent B can receive messages from Agent A even if Agent A stopped running. Messages expire after 30 minutes (configurable).

### Q: What happens if an agent's message queue fills up?

**A:** New messages are rejected with a `QueueFull` error. The sender can retry with backoff or drop the message. Default capacity is 100 messages.

### Q: Can I customize coordination schedules per agent?

**A:** Currently, heartbeat and standup schedules are global (same for all agents). Per-agent schedules are planned for Phase 8.

### Q: How do I view coordination activity in Mission Control?

**A:** Coordination events (heartbeat, standup, messages) appear in the **Squad Chat** panel and **Activity Feed**. You can filter by event type.

### Q: What LLM is used for coordination?

**A:** **Claude Haiku** for cheap operations (heartbeats, standups). Expensive operations (roundtables) use Claude Sonnet/Opus based on your config.

### Q: Can I disable coordination for specific operations?

**A:** Yes. Set `coordination_mode: disabled` per agent. Agents with coordination disabled don't participate in heartbeat, standup, or messaging.

## Next Steps

1. **Try session messaging** — configure two agents with `coordination_mode: standard` and watch them communicate
2. **Enable heartbeat** — after Plan 02 ships, monitor agent health in Mission Control
3. **Review token metrics** — after Plan 04 ships, check if coordination overhead is acceptable
4. **Optimize modes** — adjust coordination_mode per agent based on token usage

---

**Questions?** See the [Internal Developer Docs](/docs/dev/coordination-protocols.md) for implementation details.
