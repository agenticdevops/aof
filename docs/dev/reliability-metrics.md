# Reliability Metrics - Internal Developer Documentation

## Overview

The reliability metrics system computes agent uptime percentage and success rate from CoordinationEvent history. Metrics are cached in a concurrent-safe `ReliabilityCache` and exposed via REST API for the Mission Control UI.

## Architecture

```
CoordinationEvent (Phase 1)
    |
    v
EventBroadcaster (broadcast channel)
    |
    v
ReliabilityCache (background subscriber)
    |
    +--- update_with_event() --> recompute metrics for affected agent
    |
    v
/api/agents/:id/metrics (Axum handler)
    |
    v
useAgentMetrics (React hook, polls every 5s)
    |
    v
AgentCard MetricBadge (color-coded display)
```

## Key Files

| File | Purpose |
|------|---------|
| `crates/aof-personas/src/metrics.rs` | `ReliabilityMetrics`, `ReliabilityCache`, `compute_agent_metrics()` |
| `crates/aofctl/src/api/metrics.rs` | `MetricsState`, `get_agent_metrics()` handler |
| `crates/aofctl/src/commands/serve.rs` | Cache creation, event bus subscription |
| `web-ui/src/hooks/useAgentMetrics.ts` | React polling hook |
| `web-ui/src/components/AgentCard.tsx` | Live metric display integration |

## Computation Logic

### Uptime Percentage

```
uptime_percent = (total_events - error_events) / total_events * 100
```

- Error events are those with `ActivityType::Error`
- All other event types count as "up" time
- Returns `None` if fewer than 10 events (insufficient data)

### Success Rate

```
success_rate = completed_events / total_events * 100
```

- Completed events are those with `ActivityType::Completed`
- Other non-error events (Thinking, ToolExecuting, etc.) don't count as "completed"
- Returns `None` if fewer than 10 events

### Edge Cases

| Scenario | uptime_percent | success_rate |
|----------|---------------|--------------|
| No events | None | None |
| <10 events | None | None |
| All Completed | 100% | 100% |
| All Error | 0% | 0% |
| 9 Completed + 1 Error | 90% | 90% |

## ReliabilityCache

Thread-safe cache using `Arc<RwLock<>>` for concurrent reads.

### Configuration

- **Max events:** 10,000 (FIFO eviction when exceeded)
- **Version counter:** Monotonically increasing `AtomicU64`, increments on every write
- **Concurrency:** `RwLock` allows multiple simultaneous readers

### Methods

```rust
// Create with default capacity (10,000 events)
let cache = ReliabilityCache::default_capacity();

// Update with a new event (recomputes metrics for that agent)
cache.update_with_event(&event).await?;

// Get metrics for a specific agent (cache hit or compute on miss)
let metrics = cache.get_metrics("k8s-monitor").await;

// Get current version (for X-Metrics-Version header)
let version = cache.version();

// Recompute all agents (full refresh)
cache.recompute_all().await;
```

## REST API

### GET /api/agents/:id/metrics

**Response (200):**

```json
{
  "agent_id": "k8s-monitor",
  "uptime_percent": 95.5,
  "success_rate": 92.0,
  "event_count": 50,
  "last_update": "2026-02-14T10:30:00Z",
  "last_error": null
}
```

**Headers:**
- `X-Metrics-Version`: Cache version number (monotonically increasing)

**Response (404):** Agent not found in event history.

```json
{
  "error": "No metrics found for agent: nonexistent"
}
```

## React Integration

### useAgentMetrics Hook

```tsx
const {
  uptime_percent,  // number | null
  success_rate,    // number | null
  event_count,     // number
  loading,         // boolean
  error,           // Error | null
  refetch,         // () => void
} = useAgentMetrics('k8s-monitor', 5000);
```

**Features:**
- Polls every `pollIntervalMs` (default 5000ms)
- Exponential backoff on errors (max 30s)
- Tracks `X-Metrics-Version` for change detection
- Cleans up interval on unmount
- Graceful 404 handling (null metrics, no error)

### AgentCard Integration

The AgentCard uses live metrics from `useAgentMetrics`, falling back to static `agent.uptime_percent` and `agent.success_rate` props. MetricBadge shows:

- Loading: `"Uptime ..."` (pulsing animation)
- Insufficient data: `"Uptime --"`
- Normal: `"Uptime 95.5%"` (color-coded)

### Color Coding

| Range | Color | CSS Class |
|-------|-------|-----------|
| >= 95% | Green | `text-green-600` |
| 80-94% | Yellow | `text-yellow-600` |
| 60-79% | Orange | `text-orange-600` |
| < 60% | Red | `text-red-600` |

## Testing

### Unit Tests (14)

In `crates/aof-personas/src/metrics.rs`:
- Empty events, all success, all errors, mixed
- Insufficient data threshold
- Agent ID filtering
- Last error timestamp
- Serialization round-trip
- Cache CRUD, version, eviction, concurrent reads

### Integration Tests (11)

In `crates/aof-personas/tests/metrics_computation_test.rs`:
- Full pipeline: event creation -> computation -> cache -> retrieval
- Multi-agent independence
- JSON shape validation
- 404 for missing agents

### Performance Tests (4)

In `crates/aof-personas/tests/metrics_performance_test.rs`:
- 100 events < 10ms
- 1,000 events < 10ms
- 10,000 events < 50ms
- Linear scaling verification

### UI Tests (3)

In `web-ui/src/components/__tests__/AgentCard.test.tsx`:
- Live metrics display from API mock
- 404 fallback to placeholder
- Loading animation state

## Performance Considerations

- Metric computation is O(n) in event count
- Cache recomputes only the affected agent on new events
- FIFO eviction keeps memory bounded at ~10,000 events
- RwLock allows concurrent metric reads without contention
- UI polls every 5 seconds (configurable per-hook)
