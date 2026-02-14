# Decision Logging Architecture

## Overview

Decision logging is the audit trail and communication channel for agent actions. Every significant decision an agent makes is recorded with reasoning, confidence level, and metadata for visibility across the fleet.

**Key Purpose:** Enable decision transparency for auditing, learning, and inter-agent coordination.

## Architecture

### DecisionLogEntry Type (aof-core)

Located in: `crates/aof-core/src/coordination.rs`

```rust
pub struct DecisionLogEntry {
    pub event_id: String,           // UUID for this decision
    pub agent_id: String,           // Which agent made the decision
    pub timestamp: DateTime<Utc>,   // When it was made
    pub action: String,             // "restart_pod", "search_logs", etc.
    pub reasoning: String,          // Why this action was taken
    pub confidence: f64,            // 0.0-1.0 confidence level
    pub tags: Vec<String>,          // Searchability: "incident", "kubernetes", etc.
    pub related: Vec<String>,       // Links to related decision IDs (threading)
    pub metadata: serde_json::Value,// Action-specific context
}
```

### DecisionLogger (aof-coordination)

Located in: `crates/aof-coordination/src/decision_log.rs`

**Responsibilities:**
- Append decisions to JSON Lines file (immutable)
- Emit decisions to EventBroadcaster for real-time streaming
- Load recent decisions from file

**Key Methods:**
```rust
pub async fn log(&self, entry: DecisionLogEntry) -> Result<()>
pub async fn load_recent(&self, limit: usize) -> Result<Vec<DecisionLogEntry>>
```

**File Format:**
```
~/.aof/decisions.jsonl
```

Each line is a complete JSON-encoded DecisionLogEntry. This format:
- Enables streaming ingestion (parse line-by-line)
- Works with standard Unix tools (grep, tail, etc.)
- Survives daemon crashes (append-only)
- Scales to millions of entries without indexing overhead

### DecisionSearch (aof-coordination)

**Search Types:**

1. **Structured Query:**
   - Syntax: `agent=ops-bot AND confidence>0.8 AND tags:incident`
   - Fast, precise, no LLM cost
   - Parsed and evaluated locally

2. **Semantic Query (Phase 2 Fallback):**
   - Natural language: "What happened with pod crashes?"
   - Falls back to tag-based matching in Phase 2
   - Future: Vector embeddings for semantic similarity

## Integration Points

### 1. AgentExecutor Integration

Location: `crates/aof-runtime/src/executor/agent_executor.rs`

AgentExecutor logs decisions at 6 lifecycle points:

```rust
// 1. Agent starts
log_decision("agent_started", "Processing request: ...", 0.95, ...)

// 2. Tool execution success
log_decision("tool_executed", "Executed kubectl successfully", 0.9, ...)

// 3. Tool execution failure
log_decision("tool_failed", "Tool kubectl failed: ...", 0.5, ...)

// 4. Error occurs (max iterations, etc.)
log_decision("error_occurred", "Exceeded max iterations", 0.0, ...)

// 5. Agent completes successfully
log_decision("agent_completed", "Task completed with result: ...", 0.95, ...)
```

**Usage:**
```rust
let executor = AgentExecutor::new(...)
    .with_decision_logger(logger.clone());
```

### 2. aofctl serve Integration

Location: `crates/aofctl/src/commands/serve.rs`

The serve command initializes DecisionLogger at startup:

```rust
let decision_logger = if config.spec.decision_log.enabled {
    let decision_log_path = config.spec.decision_log.path.clone().unwrap_or_else(|| {
        ~/.aof/decisions.jsonl
    });
    Arc::new(DecisionLogger::new(decision_log_path, event_bus.clone()))
} else {
    None
};
```

**Configuration:**
```yaml
spec:
  decision_log:
    enabled: true
    path: /var/log/aof/decisions.jsonl
```

## Example Decision Entry

```json
{
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_id": "triage-bot",
  "timestamp": "2024-12-20T14:30:00Z",
  "action": "classify_alert",
  "reasoning": "Payment API 5xx rate > 10% indicates service degradation",
  "confidence": 0.85,
  "tags": ["incident", "api", "sev2", "payment"],
  "related": [],
  "metadata": {
    "alert_id": "ALT-001",
    "severity": "SEV2",
    "threshold_value": 12.5,
    "threshold_limit": 10.0
  }
}
```

## Querying Decisions

### CLI Example (Future)
```bash
aofctl decisions search "agent=ops-bot AND confidence>0.8"
aofctl decisions search "what happened with pods?"
aofctl decisions recent --limit 20
```

### Programmatic Access
```rust
let search = DecisionSearch::new(path);
let results = search.search("agent=triage AND action=classify").await?;
```

## Future Enhancements

### Phase 3+
- Elasticsearch indexing for multi-billion-entry logs
- Grafana visualization dashboard
- Postmortem generation from decision threads
- Decision replay/time-travel debugging

### Phase 8 (Production Readiness)
- ML-based anomaly detection on confidence levels
- Automatic escalation rules based on decision patterns
- Knowledge base integration (postmortems, learnings)
- GDPR-compliant archival and retention policies

## Troubleshooting

### Decisions Not Logging

1. Check if DecisionLogger was initialized:
   ```bash
   grep "Decision logger" aofctl output
   ```

2. Check file permissions:
   ```bash
   ls -la ~/.aof/decisions.jsonl
   ```

3. Enable debug logging:
   ```bash
   RUST_LOG=debug aofctl serve
   ```

### Malformed Entries

DecisionLogger skips malformed JSON lines with warnings:
```
WARN: Skipping malformed decision log line: ...
```

Check the log file for syntax errors:
```bash
jq '.' ~/.aof/decisions.jsonl
```

### Performance Issues

If logging is slow:
1. Check disk I/O: `iostat 1`
2. Consider moving log file to faster disk
3. Implement log rotation (future enhancement)
