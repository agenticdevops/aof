# Resource Collision Prevention

## The Problem

Imagine two agents running incident response simultaneously, both trying to solve the same pod crash:

```timeline
10:00:00 Agent A: "Pod api-001 is down, restarting it..."
10:00:02 Agent B: (also notices) "Pod api-001 is down, deleting it for fresh restart..."
10:00:04 Agent A: kubectl restart pod api-001
10:00:05 Agent B: kubectl delete pod api-001
10:00:06 Result: Pod is deleted just as it's restarting → creates a new pod immediately
```

Both agents succeeded (no errors), but:
- Agent A spent 4 seconds restarting a pod that was deleted
- Pod churn caused unnecessary cluster load
- Incident response timeline is confused (which action actually solved it?)

This is a **resource collision** — two agents operating on the same resource simultaneously.

## The Solution: Resource Locking

AOF prevents collisions by **serializing destructive operations** on the same resource:

```timeline
10:00:00 Agent A: Trying to restart pod api-001
10:00:01 Agent A: LOCK pod:prod/api-001 ✓ (acquired)
10:00:02 Agent B: Trying to delete pod api-001
10:00:03 Agent B: LOCK pod:prod/api-001 ✗ (locked by A, waiting...)
10:00:04 Agent A: kubectl restart pod api-001
10:00:05 Agent A: UNLOCK pod:prod/api-001
10:00:06 Agent B: LOCK pod:prod/api-001 ✓ (acquired)
10:00:07 Agent B: kubectl delete pod api-001
10:00:08 Agent B: UNLOCK pod:prod/api-001
```

Now the operations happen in sequence, with clear cause-and-effect.

## How It Works

### Lock Acquisition

When an agent performs a **destructive operation** (delete, restart, scale), AOF automatically:

1. **Computes a lock key** based on resource type and ID
   - Kubernetes pod: `pod:production/api-001`
   - Deployment: `deployment:prod/web`
   - Database: `database:postgres-primary`

2. **Acquires a lock** (typically via Redis)
   ```
   SET aof:lock:pod:production/api-001 agent-id NX EX 30
   ```
   - **NX:** Only succeeds if no one holds the lock
   - **EX 30:** Auto-release lock after 30 seconds (if agent crashes)

3. **Performs the operation** while holding the lock
   ```
   kubectl delete pod api-001
   ```

4. **Releases the lock**
   ```
   DEL aof:lock:pod:production/api-001
   ```

### Lock Wait and Timeout

If a lock is already held (another agent is working on the resource):

1. Agent waits up to 60 seconds for lock to become available
2. While waiting, retries every 100ms to acquire the lock
3. If timeout expires, returns error (other agent was taking too long)

Example:
```bash
Agent A holds lock for 5 seconds → Agent B waits 5 seconds → Agent B acquires lock

Agent C holds lock, crashes, lock expires after 30s → Agent D waits 30s → Agent D acquires lock
```

### Read Operations (No Locking)

Safe, read-only operations **skip locking entirely** for performance:

```
kubectl get pods           ✓ No lock needed
kubectl logs pod-001       ✓ No lock needed
kubectl top pods           ✓ No lock needed
prometheus query metric    ✓ No lock needed
```

These operations can run in parallel without contention.

## Lock Granularity

Locks are **per-resource**, enabling parallelism across resources:

```timeline
Agent A: LOCK pod:prod/api-001     → perform operation
Agent B: LOCK deployment:prod/web  → perform operation (PARALLEL, different resource)
Agent C: LOCK pod:prod/api-002     → perform operation (PARALLEL, different resource)

Agent D: LOCK pod:prod/api-001     → WAIT (same resource as Agent A)
```

The result: Your fleet can operate on different resources simultaneously, but can't collide on the same resource.

## Auto-Expiry (Safety Net)

Locks have a **30-second TTL** (time-to-live):

| Scenario | Result |
|----------|--------|
| Agent completes operation in 5s | Lock released explicitly (immediate) |
| Agent crashes | Lock auto-expires after 30s (other agents unblocked) |
| Long operation (>30s) | Agent must extend lock by re-acquiring (automatic in AOF) |

This ensures **no permanent deadlocks**. Even if an agent crashes, other agents will resume after 30 seconds.

## Configuration

### Enable/Disable Locking

```yaml
apiVersion: aof.dev/v1
kind: ServeConfig
spec:
  locking:
    enabled: true                    # Default: true (enabled)
    backend: redis                   # or "file" for development
    redis_url: redis://localhost:6379
    ttl_seconds: 30                  # Lock expires after 30s
    timeout_seconds: 60              # Wait up to 60s for lock
```

### Testing Without Redis

For local development, use file-based locking (no Redis required):

```yaml
locking:
  enabled: true
  backend: file
  lock_dir: /tmp/aof-locks
```

Files created at `/tmp/aof-locks/pod:prod:api-001.lock` with format:
```
agent-id:timestamp:ttl
```

## Observability

Every lock acquisition is logged to the decision log:

```json
{
  "agent_id": "incident-handler-001",
  "action": "lock_acquired",
  "resource": "pod:prod/api-001",
  "timestamp": "2026-02-13T10:23:45Z",
  "confidence": 0.95,
  "metadata": {
    "tool": "kubectl",
    "operation": "delete",
    "ttl_seconds": 30
  }
}
```

Query lock history:
```bash
# Find all delete operations
aof query "action=lock_acquired AND tool=kubectl AND operation=delete"

# Find operations on specific resource
aof query "action=lock_acquired AND resource=pod:prod/api-001"

# Find lock timeouts
aof query "action=lock_timeout"
```

## Best Practices

### 1. Resource Naming Consistency

Use consistent names for resources to ensure proper locking:

✓ **Good:**
- `pod:production/api-001` (environment:namespace/pod-name)
- `deployment:prod/web` (consistent naming)

✗ **Bad:**
- `api-001` (ambiguous, missing resource type)
- `prod-api-001-pod` (inconsistent format)

### 2. Monitor Lock Contention

High contention = many agents waiting for locks:

```bash
# High contention queries
aof query "action=lock_timeout"  # Timeout errors
aof query "action=lock_acquired" | count by resource
```

If specific resources see high contention:
- Split resource into smaller independent pieces
- Increase TTL so operations complete faster
- Consider async operations instead of blocking

### 3. Handle Lock Timeout Gracefully

Agents should handle lock timeouts as transient errors:

```rust
match lock_manager.acquire_with_wait().await {
    Ok(true) => {
        // Perform operation
    }
    Ok(false) => {
        // Timeout - other agent is working on resource
        return Err("Resource locked, please retry");
    }
    Err(e) => {
        // Lock system error (Redis down, etc)
        // Fallback to host execution without lock
    }
}
```

## Troubleshooting

### Locks Not Working

**Symptom:** Two agents are deleting the same pod simultaneously

**Diagnosis:**
1. Is locking enabled? `grep enabled crates/aofctl/src/config.yaml`
2. Is Redis running? `redis-cli ping`
3. Is tool recognized as destructive? Check risk policy

**Fix:**
```bash
# Enable locking
aofctl serve --enable-locking

# Verify Redis
redis-cli ping
# Output: PONG

# Check tool is destructive
grep delete crates/aof-runtime/src/executor/risk_policy.rs
```

### Lock Timeouts

**Symptom:** `Lock timeout: could not acquire lock for pod:prod/api-001`

**Causes:**
1. Another agent is running long operation (>60 seconds)
2. Agent crashed and lock hasn't expired yet (waits 30s)
3. Redis is very slow

**Solutions:**
- Increase timeout: `aofctl serve --lock-timeout 120`
- Increase TTL: `aofctl serve --lock-ttl 60`
- Optimize slow tools
- Scale Redis horizontally if under load

### Deadlocks

**Symptom:** Agent A waits for resource, Agent B waits for same resource forever

**Prevention:** AOF prevents this via timeouts
- Agent A holds lock, Operation takes >60s → Timeout expires
- Agent B waiting on Agent A → Unblocks after 60s

If you see persistent deadlocks:
1. Increase timeout/TTL to match operation time
2. Check logs for long-running operations
3. Split operation into smaller steps

## Related Topics

- [Sandbox Isolation](/docs/concepts/sandbox-security.md) — Running tools safely
- [Decision Logging](/docs/concepts/decision-logging.md) — Audit trail of all operations
- [Resource Locks (Technical)](/docs/dev/resource-locking.md) — Deep dive into implementation
