# Resource Locking Architecture

## Overview

Resource locking prevents destructive operation collisions by serializing access to shared resources through distributed locks. This document describes the architecture, implementation, and operational characteristics of AOF's resource locking system.

## Problem Statement

In a fleet of autonomous agents, multiple agents might attempt destructive operations (pod deletion, scaling, restarts) on the same resource simultaneously, causing:
- Race conditions (both agents delete the same pod)
- Inconsistent state (one agent's action undoes another's)
- Cascading failures (replica storm from scaled pods being deleted)

Resource locking ensures destructive operations on a given resource are serialized: Agent A acquires lock, performs operation, releases lock, then Agent B acquires lock.

## Architecture

### Lock Storage Backends

#### Redis Backend (Preferred)

Uses Redis atomic operations for distributed locking:

```
SET aof:lock:pod:prod/api-001 agent-001 NX EX 30
```

- **NX:** Only set if key doesn't exist (atomic test-and-set)
- **EX:** Expire after 30 seconds (auto-release on crash)
- **Ownership verification:** Lua scripts ensure only lock owner can extend/release

Example Lua script for release:
```lua
if redis.call("GET", KEYS[1]) == ARGV[1] then
    return redis.call("DEL", KEYS[1])
else
    return 0
end
```

This prevents accidental release of locks owned by other agents.

#### File-Based Fallback

For development/testing without Redis:

```
~/.aof/locks/pod:prod:api-001.lock
```

Content: `agent-001:1706234567:30`
- `agent-001` — Lock owner
- `1706234567` — Timestamp when lock acquired
- `30` — TTL in seconds

Expiry checked via timestamp comparison:
```rust
expired = now > timestamp + ttl
```

### Lock Configuration

```yaml
locking:
  enabled: true
  backend: redis              # or "file"
  redis_url: redis://localhost:6379
  ttl_seconds: 30             # Auto-expire after 30s
  timeout_seconds: 60         # acquire_with_wait timeout
  lock_dir: /tmp/aof-locks    # File backend fallback
```

### Lock Key Format

```
aof:lock:{resource_type}:{resource_id}
```

Examples:
- `aof:lock:pod:default/payment-api-001`
- `aof:lock:deployment:prod/web`
- `aof:lock:database:postgres-primary`

Granular per-resource locking allows independent operations:
- Agent A locks `pod:default/api-001` and deletes it
- Agent B locks `pod:default/api-002` and restarts it simultaneously (no collision)

## Integration

### ToolExecutor Integration

ToolExecutor checks if operation is destructive before acquiring lock:

```rust
pub async fn execute(&self, tool_name: &str, input: &ToolInput) -> Result<ToolResult> {
    // 1. Determine if destructive
    let is_destructive = self.is_destructive(tool_name, args)?;

    // 2. Acquire lock if needed
    if is_destructive {
        let lock = self.lock_manager.acquire_with_wait().await?;
        // Lock acquired - operation is serialized
    }

    // 3. Execute tool (lock auto-released via RAII guard on drop)
    // 4. Return result
}
```

### AgentExecutor Integration

AgentExecutor logs lock acquisitions to decision log:

```rust
lock_manager.acquire_with_wait().await?;
decision_logger.log_decision(DecisionLogEntry {
    action: "lock_acquired",
    metadata: {"resource": "pod:prod/api-001"},
    confidence: 0.95,
    ...
})?;
```

## Operational Characteristics

### TTL and Auto-Expiry

Locks expire after 30 seconds (configurable):

| Scenario | Outcome |
|----------|---------|
| Agent completes in 10s | Lock released explicitly, no waiting |
| Agent crashes at 15s | Lock auto-expires at 30s, other agents acquire |
| Agent operation takes 45s | Must renew lock: `lock.extend()` every 25s |

### Lock Conflict Behavior

When Agent B attempts to acquire a locked resource:

```
Agent A: acquire() → true (owns lock)
Agent B: acquire() → false (already locked)
Agent B: acquire_with_wait() → blocks, retries every 100ms
Agent A: release() → lock freed
Agent B: acquire() → true (acquires released lock)
```

Timeout prevents indefinite blocking:
```rust
acquired = lock.acquire_with_wait(Duration::from_secs(60)).await?;
if !acquired {
    return Err(AofError::lock_timeout(...));
}
```

### Resource Granularity

Locks are per-resource, enabling parallel operations on different resources:

```rust
// All three execute in parallel (different resources)
task1: lock("pod:prod/api-001")     → delete pod
task2: lock("pod:prod/api-002")     → restart pod
task3: lock("deployment:prod/web")  → scale deployment
```

But operations on same resource serialize:

```rust
task1: lock("pod:prod/api-001") → acquires lock, holds for 5s
task2: lock("pod:prod/api-001") → blocks, waits for task1 to release
```

## Configuration

### Environment Variables

```bash
# Override Redis URL
export REDIS_URL=redis://redis.default.svc.cluster.local:6379

# Override TTL
export AOF_LOCK_TTL=45

# Disable locking
export AOF_LOCKING_ENABLED=false

# Use file backend
export AOF_LOCK_BACKEND=file
export AOF_LOCK_DIR=/var/aof/locks
```

### CLI Flags

```bash
aofctl serve \
  --locking-backend redis \
  --redis-url redis://localhost:6379 \
  --lock-ttl 30
```

### YAML Configuration

```yaml
apiVersion: aof.dev/v1
kind: ServeConfig
metadata:
  name: default
spec:
  locking:
    enabled: true
    backend: redis
    redis_url: redis://redis:6379
    ttl_seconds: 30
    timeout_seconds: 60
    lock_dir: /tmp/aof-locks
```

## Monitoring

### Decision Log Entries

Each lock acquisition/release is logged:

```json
{
  "agent_id": "incident-handler-001",
  "action": "lock_acquired",
  "resource": "pod:prod/api-001",
  "timestamp": "2026-02-13T10:23:45.123Z",
  "confidence": 0.95,
  "metadata": {
    "tool": "kubectl",
    "operation": "delete pod",
    "ttl_seconds": 30
  }
}
```

### Querying Lock History

Structured search for lock patterns:

```bash
# Find all delete operations that acquired locks
aof query decision-log "action=lock_acquired AND tool=kubectl AND operation=delete"

# Find locks held by specific agent
aof query decision-log "agent_id=incident-handler-001 AND action=lock_acquired"

# Find lock timeouts
aof query decision-log "action=lock_timeout"
```

## Troubleshooting

### Lock Timeouts

**Symptom:** `Lock timeout: could not acquire lock for pod:prod/api-001 within timeout`

**Causes:**
1. Previous agent crashed with lock held → wait for TTL expiry (30s)
2. Previous operation taking longer than timeout (60s) → increase timeout
3. Redis unavailable → falls back to file-based locking (slower)

**Solutions:**
- Increase timeout: `--lock-timeout 120`
- Increase TTL: `--lock-ttl 60`
- Ensure Redis is running: `redis-cli ping`
- Check lock ownership: `aof query decision-log "action=lock_acquired AND resource=..."`

### Ownership Errors

**Symptom:** `Lock ownership error: agent-002 does not own lock for pod:prod/api-001`

**Cause:** Agent attempted to release lock it doesn't own (should not happen in normal operation)

**Debug:** Check lock history for owner
```bash
aof query decision-log "resource=pod:prod/api-001 AND action=lock_acquired" | tail -1
```

### Stale Locks

**Symptom:** Lock exists but no agent performing operation

**Cause:** Agent crashed before releasing lock (normal case — TTL will handle)

**Manual cleanup (if needed):**
```bash
# Redis backend
redis-cli DEL aof:lock:pod:prod/api-001

# File backend
rm ~/.aof/locks/pod:prod:api-001.lock
```

## Performance

### Latency Impact

- **Lock acquisition:** <5ms (Redis) or <10ms (file-based)
- **Lock release:** <5ms (Lua script validates ownership)
- **Lock extension:** <5ms (refreshes TTL)
- **Lock wait (per iteration):** 100ms sleep + <5ms check

Total overhead for destructive operation:
- **Successful acquire:** <10ms
- **Wait and acquire (10 agents):** ~1-2 seconds

### Scalability

- **Redis backend:** Linear with agent count (each acquire is atomic operation)
- **File backend:** Linear with agent count (file I/O relatively fast)
- **Lock granularity:** Scales with number of unique resources

Testing shows system handles 50+ concurrent lock requests across 20+ resources without performance degradation.

## Future Enhancements

### Phase 3: Advanced Locking
- Distributed deadlock detection (for multi-resource operations)
- Adaptive TTL based on operation type
- Lock priority levels (critical operations get priority)

### Phase 8: Production Hardening
- Elasticsearch-based lock history for long-term analysis
- Grafana dashboards for lock contention monitoring
- Lock hold time SLO tracking and alerting

## See Also

- [Decision Logging Architecture](/docs/dev/decision-logging.md)
- [Sandbox Isolation](/docs/dev/sandbox-isolation.md)
- [ToolExecutor Integration](/docs/dev/tool-executor.md)
