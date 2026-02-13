# Incident Response System - Developer Guide

## Overview

The Incident Response System enables AOF agents to automatically triage alerts, dispatch specialist agents, and make escalation decisions based on confidence levels and impact assessment. This system is built on the decision logging infrastructure (Phase 2, Plan 1) and provides the foundation for intelligent incident handling.

## Architecture Components

### 1. TriageAgent

**Location:** `crates/aof-runtime/src/executor/incident_triage.rs`

The TriageAgent is responsible for initial alert analysis and classification.

**Key Methods:**
- `classify_alert(&self, alert: &AlertPayload) -> Result<TriageClassification>`
  - Analyzes alert using LLM (or deterministic logic in Phase 2)
  - Returns severity (SEV1-4), confidence (0.0-1.0), category, specialists needed
  - Emits TriageClassification event

- `triage(&self, alert: &AlertPayload) -> Result<TriageResult>`
  - Orchestrates full triage workflow
  - Logs decision to DecisionLogger
  - Determines escalation need (confidence < 60%)
  - Returns TriageResult with escalation_reason

**Types:**
```rust
pub struct AlertPayload {
    pub alert_id: String,
    pub summary: String,
    pub error_rate: Option<f64>,
    pub affected_services: Vec<String>,
    pub duration_seconds: u64,
    pub affected_users: Option<u64>,
    pub logs_available: bool,
    pub metrics_available: bool,
    pub context: serde_json::Value,
}

pub struct TriageClassification {
    pub severity: String,        // "SEV1", "SEV2", "SEV3", "SEV4"
    pub confidence: f64,         // 0.0-1.0
    pub category: String,        // "api-degradation", "database-error", etc.
    pub specialists_needed: Vec<String>,
    pub reasoning: String,
}

pub struct TriageResult {
    pub incident_id: String,
    pub classification: TriageClassification,
    pub should_escalate: bool,
    pub escalation_reason: Option<String>,
}
```

### 2. Specialist Agents

Specialist agents are spawned based on triage classification. Each specialist is a separate agent with specific skills and task instructions.

**Specialists (Phase 2):**
- **log-analyzer:** Parses logs from Loki, finds error patterns
  - Skills: loki-search, shell-execute
  - Task: Find ERROR/FATAL logs, identify patterns, count occurrences

- **metric-checker:** Queries Prometheus for metrics anomalies
  - Skills: prometheus-query, shell-execute
  - Task: Compare current metrics to baseline, identify spikes

- **k8s-diagnostician:** Analyzes Kubernetes cluster state
  - Skills: k8s-debug, k8s-logs, shell-execute
  - Task: Inspect pods, events, node status, identify crashes

### 3. IncidentContextStore

**Location:** `crates/aof-runtime/src/executor/incident_triage.rs`

Provides shared context for specialists to query and store findings.

**Key Methods:**
- `store_alert_context(&self, alert: &AlertPayload)` — Stores original alert for specialists
- `store_finding(&self, agent_id: &str, finding: &str, confidence: f64)` — Specialists log findings
- `get_recent_findings(&self) -> Vec<(String, String, f64)>` — Query all findings
- `query_logs(&self, query: &str)` — Helper for log-analyzer
- `query_metrics(&self, metric_name: &str)` — Helper for metric-checker

### 4. IncidentResponseFlow

**Location:** `crates/aof-runtime/src/fleet/incident_response.rs`

Orchestrates the full incident response workflow from alert to resolution.

**Key Methods:**
- `handle_alert(&self, alert: &AlertPayload) -> Result<IncidentResponse>`
  - Entry point for alert handling
  - Runs triage, spawns specialists, synthesizes findings
  - Checks escalation triggers, escalates if needed
  - Returns IncidentResponse with status, findings, involved specialists

- `escalate(&self, trigger: &EscalationTrigger)` — Triggers escalation to human team
- `synthesize_findings(&self)` — Combines specialist findings into RCA summary

**Types:**
```rust
pub enum EscalationTrigger {
    ConfidenceLow { classification_confidence: f64 },
    TimeThreshold { minutes: u64 },
    ImpactHigh { affected_users: u64, revenue_impact: Option<String> },
    SpecialistFailed { agent_id: String, reason: String },
}

pub struct IncidentResponse {
    pub incident_id: String,
    pub severity: String,
    pub status: String,           // "investigating", "escalated", "resolved"
    pub findings: String,
    pub specialists_involved: Vec<String>,
    pub resolution_time_seconds: u64,
    pub escalations: Vec<EscalationTrigger>,
}
```

## Event Flow

```
Alert fires
  ↓
TriageAgent.triage()
  ├─ classify_alert() — LLM/logic classification
  ├─ log decision to DecisionLogger
  ├─ emit TriageClassification event
  ├─ determine escalation need
  └─ return TriageResult

IncidentResponseFlow.handle_alert()
  ├─ store alert context in IncidentContextStore
  ├─ run triage workflow
  ├─ spawn specialists
  │  └─ Each specialist pulls context from IncidentContextStore
  ├─ wait for findings
  ├─ check escalation triggers
  ├─ escalate if needed (log decision, emit EscalationTriggered)
  ├─ synthesize findings into RCA
  ├─ emit IncidentResolved event
  └─ return IncidentResponse
```

## Decision Logging Integration

All significant actions are logged to DecisionLogger:

1. **triage_classification** — When triage completes
   - Action: "classify_alert"
   - Reasoning: Triage classification reasoning
   - Confidence: Triage confidence score

2. **spawned_specialist_{type}** — When each specialist is spawned
   - Action: "spawn_specialist"
   - Reasoning: Why this specialist was chosen
   - Confidence: 0.95 (high confidence in spawn decision)

3. **specialist_finding** — When specialist reports a finding
   - Action: "specialist_finding"
   - Reasoning: The finding and its implications
   - Confidence: Specialist's confidence in the finding

4. **escalate_incident** — When escalation is triggered
   - Action: "escalate_incident"
   - Reasoning: Escalation trigger reason
   - Confidence: 0.9

## Configuration

Incident response is configured via YAML agent templates:

- `agents/triage-agent.yaml` — Triage agent instructions and tools
- `agents/log-analyzer-agent.yaml` — Log analyzer instructions and skills
- `agents/metric-checker-agent.yaml` — Metric checker instructions and skills
- `agents/k8s-diagnostician-agent.yaml` — K8s diagnostician instructions and skills

Each agent YAML includes:
- Model (provider, model name)
- Instructions (task description, output format)
- Skills (which skills to use)
- Memory configuration
- Timeout and iteration limits

## Testing

### Unit Tests

Located in `crates/aof-runtime/src/executor/incident_triage.rs` and `fleet/incident_response.rs`:

- `test_classify_alert_high_error_rate()` — Verify SEV1 classification
- `test_triage_escalation_on_low_confidence()` — Verify escalation on low confidence
- `test_incident_response_flow()` — Full end-to-end flow
- `test_escalation_trigger_low_confidence()` — Verify escalation trigger logic

### Integration Tests

`crates/aof-runtime/tests/incident_response_integration.rs`

Tests full workflow: alert → triage → specialist spawn → decision logging → events

### Manual Testing

```bash
# Build the project
cargo build --release

# Run tests
cargo test --package aof-runtime incident_response

# View decision log
cat ~/.aof/decisions.jsonl | jq '.[] | select(.action | contains("incident"))'
```

## Future Enhancements

### Phase 3 (Messaging Gateway)
- Escalation notifications to Slack, PagerDuty, email
- War room creation for critical incidents
- Real-time collaboration channels

### Phase 4 (Mission Control UI)
- Incident dashboard with live specialist status
- Finding visualization and synthesis
- Escalation approval UI

### Phase 7 (Coordination Protocols)
- Multi-incident coordination when multiple alerts fire
- Deduplication logic (is this a new incident or continuation?)
- Incident grouping by root cause

### Phase 8 (Production Readiness)
- LLM-based classification with actual Claude model
- Confidence calibration via feedback loops
- Performance optimization for high-volume alerts
- SLA tracking and response time metrics

## Troubleshooting

### Specialist Not Spawning

Check:
1. Specialist YAML exists in `agents/` directory
2. Specialist type is in `TriageClassification.specialists_needed`
3. AgentExecutor has required model configured
4. Check logs for spawn failures in decision log

### Low Confidence Escalations

Verify:
1. Alert has sufficient context (error_rate, affected_users, etc.)
2. Multiple signals align (error rate + latency + CPU)
3. Category matches known patterns (api-degradation, pod-crash, etc.)

### Finding Synthesis Issues

Check:
1. Specialists completed execution (check decision log)
2. IncidentContextStore has specialist findings stored
3. Findings have reasonable confidence levels
4. RCA synthesis prompt is accurate

## Integration Points

- **aof-core:** Uses IncidentEvent variants in CoordinationEvent
- **aof-coordination:** Uses DecisionLogger for audit trail, EventBroadcaster for events
- **aof-runtime:** Extends AgentExecutor with specialist spawning
- **aof-llm:** Phase 3+ will use for LLM-based classification
- **aofctl:** Integration point for incident commands

## Performance Characteristics

- **Triage classification:** <1s (Phase 2 deterministic)
- **Specialist spawning:** <5s per specialist
- **Finding synthesis:** <30s (depends on specialist execution time)
- **Decision logging:** <5ms per entry
- **Event emission:** Best-effort, non-blocking

## See Also

- `docs/concepts/incident-response-flow.md` — User-facing explanation
- `crates/aof-coordination/src/decision_log.rs` — Decision logging details
- `agents/*.yaml` — Agent configurations
