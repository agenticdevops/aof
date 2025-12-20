# Horizontal Scaling with Message Queue

**GitHub Issue**: [#47](https://github.com/agenticdevops/aof/issues/47)
**Status**: Planned
**Priority**: Medium

## Summary

Enable enterprise-scale deployments (1000s of repositories, high webhook throughput) through horizontal scaling with a message queue architecture separating webhook ingestion from event processing.

## Current Architecture

```
GitHub Webhook → AOF Daemon (single process) → Execute Agent/Fleet/Flow
```

**Limitations:**
- Single process handles all events
- Synchronous webhook processing
- No built-in queue for backpressure handling
- Memory grows with trigger count
- Single point of failure

**Current Capacity:**
- ~100-500 webhook events/minute (single instance)
- Suitable for up to ~50 repositories

## Proposed Architecture

```
                    ┌─────────────────┐
                    │   Ingress/LB    │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ AOF Gateway │ │ AOF Gateway │ │ AOF Gateway │
     │ (stateless) │ │ (stateless) │ │ (stateless) │
     └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                   ┌─────────────────┐
                   │   Redis/NATS    │
                   │  (message queue)│
                   └────────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ AOF Worker  │ │ AOF Worker  │ │ AOF Worker  │
     │ (executor)  │ │ (executor)  │ │ (executor)  │
     └─────────────┘ └─────────────┘ └─────────────┘
```

## Components

### AOF Gateway (stateless)
- Receives webhooks
- Validates signatures
- Matches to trigger
- Publishes to queue
- Returns 202 Accepted immediately

### Message Queue
Options:
- **Redis Streams**: Simple, widely deployed, good for moderate scale
- **NATS JetStream**: High performance, built for messaging, better for high scale

Features needed:
- Durable message storage
- Consumer groups for load distribution
- Dead letter queue for failed events
- Retry with exponential backoff

### AOF Worker (stateless)
- Consumes from queue
- Executes agents/fleets/flows
- Reports results back
- Horizontally scalable

## Configuration

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-gateway

spec:
  mode: gateway  # gateway | worker | standalone (default)

  queue:
    type: redis  # redis | nats
    url: redis://redis-cluster:6379

    # Queue settings
    stream: aof-events
    consumer_group: aof-workers
    max_retries: 3
    retry_delay_ms: 1000
    dead_letter_queue: aof-dlq

  # Gateway-specific settings
  gateway:
    ack_timeout_ms: 5000  # Return 202 within 5s

  # Worker-specific settings
  worker:
    concurrency: 10  # Parallel event processing
    prefetch: 5      # Events to prefetch
```

## Implementation Plan

### Phase 1: Queue Abstraction
- [ ] Define `MessageQueue` trait
- [ ] Implement Redis Streams backend
- [ ] Implement NATS JetStream backend
- [ ] Add queue configuration to DaemonConfig

```rust
#[async_trait]
pub trait MessageQueue: Send + Sync {
    async fn publish(&self, event: WebhookEvent) -> Result<MessageId>;
    async fn consume(&self) -> Result<Option<Message>>;
    async fn ack(&self, id: MessageId) -> Result<()>;
    async fn nack(&self, id: MessageId) -> Result<()>;
    async fn dead_letter(&self, id: MessageId, reason: &str) -> Result<()>;
}
```

### Phase 2: Gateway Mode
- [ ] Add `mode: gateway` option
- [ ] Separate webhook handling from execution
- [ ] Publish events to queue
- [ ] Return 202 Accepted immediately

### Phase 3: Worker Mode
- [ ] Add `mode: worker` option
- [ ] Consume from queue
- [ ] Execute agents/fleets/flows
- [ ] Handle failures and retries

### Phase 4: Observability
- [ ] Queue depth metrics (Prometheus)
- [ ] Processing latency metrics
- [ ] Dead letter queue alerting
- [ ] Distributed tracing (OpenTelemetry)

### Phase 5: Advanced Features
- [ ] Priority queues (critical events first)
- [ ] Rate limiting per org/repo
- [ ] Event deduplication
- [ ] Graceful shutdown with drain

## Kubernetes Deployment

```yaml
# Gateway Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-gateway
spec:
  replicas: 3
  template:
    spec:
      containers:
        - name: aof
          args: [serve, --mode=gateway]
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
---
# Worker Deployment (auto-scaling)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-worker
spec:
  replicas: 5
  template:
    spec:
      containers:
        - name: aof
          args: [serve, --mode=worker]
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
---
# HPA for workers based on queue depth
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: aof-worker-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: aof-worker
  minReplicas: 2
  maxReplicas: 20
  metrics:
    - type: External
      external:
        metric:
          name: redis_stream_pending_messages
        target:
          type: AverageValue
          averageValue: 100
```

## Scaling Recommendations

| Scale | Repos | Events/min | Architecture |
|-------|-------|------------|--------------|
| Small | 1-50 | 1-100 | Single daemon (standalone) |
| Medium | 50-200 | 100-500 | 2-3 replicas, load balanced |
| Large | 200-1000 | 500-2000 | Gateway + Redis + 5-10 workers |
| Enterprise | 1000+ | 2000+ | Gateway + NATS + 20+ workers |

## Benefits

- **Scalability**: Add workers to handle more load
- **Reliability**: Events persisted in queue, survive restarts
- **Backpressure**: Queue absorbs traffic spikes
- **Isolation**: Workers can be specialized (frontend, backend, infra)
- **Observability**: Queue metrics for capacity planning
- **Cost Efficiency**: Scale workers based on demand

## Message Format

```json
{
  "id": "msg-uuid",
  "timestamp": "2024-01-15T10:30:00Z",
  "platform": "github",
  "org": "myorg",
  "repo": "myrepo",
  "event_type": "pull_request",
  "action": "opened",
  "trigger_name": "frontend-pr-bot",
  "payload": { ... },
  "metadata": {
    "delivery_id": "github-delivery-uuid",
    "attempt": 1,
    "max_retries": 3
  }
}
```

## Error Handling

```
Event Processing
      │
      ▼
┌─────────────────┐
│ Execute agent   │
└────────┬────────┘
         │
    ┌────┴────┐
    │ Success │──────────────▶ ACK message
    └────┬────┘
         │
    ┌────┴────┐
    │ Failure │
    └────┬────┘
         │
         ▼
┌─────────────────┐
│ Retry < max?    │──── Yes ───▶ NACK with delay
└────────┬────────┘
         │ No
         ▼
┌─────────────────┐
│ Move to DLQ     │
│ Alert ops team  │
└─────────────────┘
```

## Dependencies

- Redis 7.0+ (for Streams with consumer groups) OR
- NATS 2.9+ (for JetStream)

## Related Issues

- #45 - Team/role-based authorization
- #46 - Multi-organization support

## Acceptance Criteria

- [ ] MessageQueue trait with Redis and NATS backends
- [ ] Gateway mode for webhook ingestion
- [ ] Worker mode for event processing
- [ ] Dead letter queue for failed events
- [ ] Retry with exponential backoff
- [ ] Prometheus metrics for queue depth
- [ ] Kubernetes manifests for horizontal deployment
- [ ] Helm chart with scaling options
- [ ] Documentation for enterprise deployment
- [ ] Benchmark showing 10x throughput improvement
- [ ] Graceful shutdown with message drain
