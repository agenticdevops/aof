# Incident Response Flow - Concepts

## What is Incident Response?

Incident Response is AOF's intelligent system for handling operational alerts. When something goes wrong in your infrastructure, instead of you manually investigating, AOF's agents automatically:

1. **Triage** the alert to understand severity
2. **Classify** it by type (API degradation, database error, pod crash, etc.)
3. **Dispatch** specialist agents to investigate independently
4. **Synthesize** findings into a Root Cause Analysis (RCA)
5. **Escalate** to humans if confidence is low or impact is high

All decisions are logged to a shared audit trail so you can see exactly what each agent decided and why.

## The Workflow

```
┌──────────────────────┐
│   Alert Fires        │
│ Error rate > 10%     │
│ Service: payment-api │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────────┐
│ Triage Agent Analyzes    │
│ • Severity: SEV2 (high)  │
│ • Confidence: 75%        │
│ • Category: api-degrad   │
│ • Needs: logs + metrics  │
└──────────┬───────────────┘
           │
           ▼
    ┌──────┴──────┐
    ▼             ▼
┌─────────────┐ ┌──────────────┐
│Log Analyzer │ │Metric Checker│
│ • Finds     │ │ • Compares   │
│   error     │ │   current vs │
│   patterns  │ │   baseline   │
│ • Reports  │ │ • Reports    │
│   findings │ │   anomalies  │
└──────┬──────┘ └──────┬───────┘
       │                │
       └────────┬───────┘
                ▼
      ┌──────────────────────┐
      │ Synthesis (RCA)      │
      │ "Likely cause: Pod   │
      │  restarted due to    │
      │  OOM killer"         │
      └──────────┬───────────┘
                 │
         ┌───────▼────────┐
         │ Confidence     │
         │ > 60%?         │
         └───────┬────────┘
            Yes  │  No
                 │   └─────────────┐
                 ▼                 ▼
          ┌──────────────┐  ┌────────────────┐
          │ Resolved     │  │ Escalate to    │
          │ (Findings    │  │ Human Team     │
          │  logged)     │  │ (Low confidence)
          └──────────────┘  └────────────────┘
```

## Key Concepts

### Triage Agent

The **Triage Agent** is the first responder. It quickly analyzes the incoming alert and decides:

1. **Severity:** How bad is this?
   - SEV1 (Critical): Service completely down, no workarounds
   - SEV2 (High): Major functionality impaired, users affected
   - SEV3 (Medium): Minor functionality impaired
   - SEV4 (Low): Non-critical issue or warning

2. **Confidence:** How sure are we about this classification?
   - 0.0 = Complete guess
   - 0.5 = Moderately sure
   - 1.0 = Extremely confident

3. **Category:** What type of problem?
   - api-degradation: API returning errors or latency
   - database-error: Database connection/query failures
   - pod-crash: Kubernetes pod crashing/restarting
   - network-issue: Network connectivity problems
   - resource-exhaustion: CPU, memory, or disk full
   - Other

4. **Specialists Needed:** Which agents should investigate?
   - log-analyzer: Dig through logs for error patterns
   - metric-checker: Check metrics for anomalies
   - k8s-diagnostician: Inspect Kubernetes state

### Specialist Agents

Specialist agents work independently, each focusing on their domain:

- **Log Analyzer**
  - Searches logs from the last 30 minutes
  - Finds repeated ERROR/FATAL messages
  - Identifies stack traces and patterns
  - Reports findings with confidence levels

- **Metric Checker**
  - Queries Prometheus for key metrics
  - Compares current values to 24-hour baseline
  - Identifies anomalies (spikes, drops, threshold violations)
  - Reports metrics that deviate from baseline

- **Kubernetes Diagnostician**
  - Lists pods, checks for CrashLoopBackOff
  - Inspects pod events and descriptions
  - Checks node status and resource usage
  - Identifies DNS failures or mount issues

All specialists have access to:
- Original alert data (summary, affected services, duration, etc.)
- Shared context store (other specialists' findings)
- Their specialized skills (kubectl, curl, grep, etc.)

### Context Pull Model

Instead of Triage pushing data to specialists, specialists **pull** what they need:

```
Triage Agent stores:
├─ Alert summary
├─ Error rate
├─ Affected services
└─ Timestamps

Specialists query context:
├─ Log Analyzer: "What services are affected?"
├─ Metric Checker: "What time range should I check?"
└─ K8s Diagnostician: "What services failed?"
```

This gives specialists independence: they can discover their own clues, prioritize their investigation, and report findings without waiting for Triage to tell them what to do.

### Escalation Triggers

Even if specialists find something, escalation happens when:

1. **Low Confidence** (< 60%)
   - Triage wasn't sure what type of incident this is
   - Specialists need human judgment to interpret findings

2. **Time Threshold** (> 30 minutes)
   - Alert has been ongoing for 30+ minutes
   - Escalate to team lead
   - After 1 hour, escalate to manager

3. **High Impact** (> 10,000 affected users)
   - Large number of users impacted
   - Escalate to executive team
   - Requires immediate human attention

4. **Specialist Failed**
   - A specialist couldn't complete investigation
   - Need human to manually diagnose

5. **SEV1 Always**
   - Critical incidents always escalate immediately
   - No waiting for analysis

### Decision Log

Every decision is recorded:

```json
{
  "event_id": "a1b2c3d4-...",
  "agent_id": "triage-agent",
  "timestamp": "2026-02-13T09:30:45Z",
  "action": "classify_alert",
  "reasoning": "High error rate (15%) on payment service suggests API degradation",
  "confidence": 0.85,
  "tags": ["incident", "sev2", "api-degradation", "payment"],
  "metadata": {
    "severity": "SEV2",
    "category": "api-degradation",
    "specialists_needed": ["log-analyzer", "metric-checker"]
  }
}
```

The log serves as:
- **Audit trail:** See every decision and why
- **Context for learning:** Pattern match against past incidents
- **Communication:** Humans can see what agents were thinking
- **Training data:** Future models can learn from past decisions

## Example Incident

**Alert:** "Payment API error rate 15%, 500 affected users"

**Triage Agent Response:**
```
SEVERITY: SEV2
CONFIDENCE: 0.85
CATEGORY: api-degradation
SPECIALISTS: log-analyzer, metric-checker
REASONING: High error rate on critical service indicates degradation, not complete outage.
```

Decision logged: `action=classify_alert, severity=SEV2, confidence=0.85`

**Spawned Specialists:**
- log-analyzer
- metric-checker

**Log Analyzer Findings:**
```
ERROR PATTERN: "Connection refused" on paymentdb.internal
OCCURRENCES: 1,247 in last 5 minutes
LIKELY CAUSE: Database connection pool exhaustion or database service down
CONFIDENCE: 0.92
```

Decision logged: `action=specialist_finding, agent=log-analyzer, confidence=0.92`

**Metric Checker Findings:**
```
METRIC: payments_latency_p99
VALUE: 8,500ms
BASELINE: 200ms
CHANGE: 4,150% (massive spike!)

METRIC: db_connections_active
VALUE: 500
BASELINE: 50
CHANGE: 900% (all connections in use)
```

Decision logged: `action=specialist_finding, agent=metric-checker, confidence=0.88`

**RCA Synthesis:**
"Likely cause: Database connection pool exhausted due to slow queries. All 500 connections are held by slow transactions. New payment requests wait in queue until timeout. Recommendation: Kill slow transactions, increase connection pool, investigate slow query root cause."

**Escalation Decision:**
- Confidence: 0.88 (high enough, no escalation)
- Time: 5 minutes (under 30m threshold)
- Impact: 500 users (under 10k threshold)
- Severity: SEV2 (not SEV1)

**Result:** Incident marked "investigating", specialists' findings logged. Humans can review decision log and use recommendations to resolve.

---

## Key Principles

### 1. Transparency
Every decision is logged with reasoning and confidence. You can always understand why an agent made a choice.

### 2. Specialist Independence
Specialists don't wait for Triage to tell them what to investigate. They pull context, investigate independently, and report findings.

### 3. Confidence-Driven
Escalation is driven by confidence, not by rules. If we're unsure, we ask humans. If we're sure, we handle it.

### 4. Auditability
All decisions create a searchable audit trail. Find patterns, learn from past incidents, improve future responses.

### 5. Fault Tolerant
If a specialist fails (skill not available, timeout, etc.), investigation continues with remaining specialists. No single point of failure.

## Related Documentation

- **For Developers:** See `docs/dev/incident-response.md` for architecture, code locations, testing
- **Agent Templates:** See `agents/triage-agent.yaml`, `agents/log-analyzer-agent.yaml`, etc.
- **Decision Logging:** See `docs/dev/decision-logging.md` for how decisions are stored and searched
- **Skills Platform:** See `docs/dev/skills-platform.md` for available skills

## What's Next?

**Phase 3 (Messaging Gateway):**
- Escalations notify your team on Slack, PagerDuty, email
- War rooms auto-created for critical incidents
- Live collaboration with agents

**Phase 4 (Mission Control UI):**
- Dashboard showing live incident status
- Visualization of specialist findings
- Ability to interrupt or redirect agents

**Phase 7 (Coordination):**
- Multiple incidents coordinated automatically
- Deduplication (is this a new incident or continuation?)
- Incident grouping by root cause

**Phase 8 (Production Readiness):**
- Real LLM-based classification (not deterministic)
- Confidence tuning via feedback loops
- Load testing and optimization
