# RuVector Ops Guide for AOF

## For Operations Engineers

This guide provides practical, operations-focused guidance for deploying and managing RuVector as the vector store for AOF's Agentic RAG system.

## Quick Start

### 1. Deploy with Helm

```bash
# Add RuVector Helm repo
helm repo add ruvector https://charts.ruvector.io
helm repo update

# Install RuVector cluster
helm install ruvector ruvector/ruvector \
  --namespace agentic-ops \
  --create-namespace \
  --set cluster.size=3 \
  --set persistence.enabled=true \
  --set persistence.size=100Gi \
  --set gnn.enabled=true \
  --set monitoring.enabled=true

# Verify deployment
kubectl get pods -n agentic-ops
# Expected:
# ruvector-0   2/2     Running   0          2m
# ruvector-1   2/2     Running   0          2m
# ruvector-2   2/2     Running   0          2m
```

### 2. Apply Memory CRD

```bash
# Create RuVector memory backend
kubectl apply -f config/memory-ruvector.yaml

# Check status
kubectl get memory -n agentic-ops
# NAME                   BACKEND    STATUS    AGE
# ruvector-rag-memory   RuVector   Ready     1m
```

### 3. Verify Health

```bash
# Check cluster health
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status

# Expected output:
# Cluster Status: Healthy
# Nodes: 3/3 online
# Leader: ruvector-1
# Index Size: 1.2M vectors
# Query Latency (P99): 89ms
# Learning Accuracy: 92%
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────────────────────────────────────────────┐     │
│  │              AOF Agent Pods                         │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │     │
│  │  │K8s Ops   │  │Security  │  │Monitor   │         │     │
│  │  │Agent     │  │Agent     │  │Agent     │         │     │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘         │     │
│  │       │             │             │                 │     │
│  │       └─────────────┴─────────────┘                │     │
│  │                     │                               │     │
│  └─────────────────────┼───────────────────────────────┘     │
│                        │                                      │
│  ┌─────────────────────▼──────────────────────────────┐     │
│  │         RuVector StatefulSet (3 replicas)          │     │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐   │     │
│  │  │ruvector-0  │  │ruvector-1  │  │ruvector-2  │   │     │
│  │  │(Follower)  │  │(Leader)    │  │(Follower)  │   │     │
│  │  │            │  │            │  │            │   │     │
│  │  │Vector Store│  │Vector Store│  │Vector Store│   │     │
│  │  │Graph DB    │  │Graph DB    │  │Graph DB    │   │     │
│  │  │SONA Engine │  │SONA Engine │  │SONA Engine │   │     │
│  │  └────────────┘  └────────────┘  └────────────┘   │     │
│  │         │               │               │           │     │
│  │         └───────────────┴───────────────┘           │     │
│  │                         │                            │     │
│  │  ┌──────────────────────▼─────────────────────┐    │     │
│  │  │     Persistent Volumes (100Gi each)        │    │     │
│  │  └────────────────────────────────────────────┘    │     │
│  └───────────────────────────────────────────────────┘      │
│                                                               │
│  ┌───────────────────────────────────────────────────┐      │
│  │              Monitoring Stack                      │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │      │
│  │  │Prometheus│  │Grafana   │  │Alert Mgr │        │      │
│  │  └──────────┘  └──────────┘  └──────────┘        │      │
│  └───────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Configuration Management

### Environment-Based Configs

**Development:**
```yaml
# config/dev-ruvector.yaml
spec:
  backend:
    ruvector:
      mode: Embedded
      embedded:
        memoryLimit: 2Gi
      gnn:
        enabled: false  # Disable learning in dev
      compression:
        strategy: Fixed
        fixed:
          encoding: f32  # No compression
```

**Staging:**
```yaml
# config/staging-ruvector.yaml
spec:
  backend:
    ruvector:
      mode: Cluster
      distributed:
        replication:
          factor: 2
      gnn:
        enabled: true
        adaptationInterval: 500
      compression:
        strategy: Adaptive
```

**Production:**
```yaml
# config/prod-ruvector.yaml
spec:
  backend:
    ruvector:
      mode: Distributed
      distributed:
        replication:
          factor: 3
          minSyncReplicas: 2
        sharding:
          enabled: true
          shardCount: 8
      gnn:
        enabled: true
        adaptationInterval: 100
      compression:
        strategy: Adaptive
      backup:
        enabled: true
        schedule: "0 0 * * *"
```

### Apply Configuration

```bash
# Deploy to environment
kubectl apply -f config/prod-ruvector.yaml -n agentic-ops

# Verify configuration
kubectl get memory ruvector-rag-memory -n agentic-ops -o yaml

# Check logs
kubectl logs -n agentic-ops ruvector-0 -c ruvector
```

## Monitoring and Alerting

### Grafana Dashboard

```bash
# Import RuVector dashboard
curl https://charts.ruvector.io/dashboards/overview.json | \
  curl -X POST http://grafana/api/dashboards/db \
    -H "Content-Type: application/json" \
    -d @-
```

**Key Metrics:**
- Query latency (P50, P99)
- Queries per second (QPS)
- Index memory usage
- Learning accuracy
- Replication lag
- Error rate

### Prometheus Alerts

```yaml
# alerts/ruvector-alerts.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: ruvector-alerts
  namespace: monitoring
data:
  ruvector.rules: |
    groups:
    - name: ruvector
      interval: 30s
      rules:

      # High query latency
      - alert: RuVectorHighLatency
        expr: ruvector_query_latency_p99 > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "RuVector query latency high"
          description: "P99 latency is {{ $value }}ms"

      # High memory usage
      - alert: RuVectorHighMemory
        expr: ruvector_index_memory_bytes / ruvector_memory_limit_bytes > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "RuVector memory usage high"
          description: "Memory usage at {{ $value | humanizePercentage }}"

      # Replication lag
      - alert: RuVectorReplicationLag
        expr: ruvector_replication_lag_ms > 1000
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "RuVector replication lag high"
          description: "Lag is {{ $value }}ms"

      # Learning degradation
      - alert: RuVectorLearningAccuracyDrop
        expr: rate(ruvector_learning_accuracy[5m]) < 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "RuVector learning accuracy dropped"
          description: "Accuracy is {{ $value | humanizePercentage }}"
```

### Log Analysis

```bash
# Check for errors
kubectl logs -n agentic-ops -l app=ruvector \
  --since=1h | grep -i error

# Monitor query performance
kubectl logs -n agentic-ops ruvector-0 -c ruvector \
  --since=10m | grep "query_latency"

# Check SONA adaptation
kubectl logs -n agentic-ops ruvector-0 -c ruvector \
  --since=1h | grep "adaptation"
```

## Backup and Recovery

### Automated Backups

```yaml
spec:
  backup:
    enabled: true
    schedule: "0 0 * * *"  # Daily at midnight
    retention: 30d
    compression: zstd
    destination:
      type: S3
      s3:
        bucket: aof-ruvector-backups
        prefix: production/
        region: us-east-1
```

### Manual Backup

```bash
# Trigger manual backup
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli backup create \
    --name manual-backup-$(date +%Y%m%d) \
    --destination s3://aof-ruvector-backups/manual/

# List backups
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli backup list

# Expected output:
# NAME                       SIZE     DATE
# manual-backup-20251209    12.5GB   2025-12-09 10:00:00
# auto-backup-20251208      11.8GB   2025-12-08 00:00:00
# auto-backup-20251207      11.2GB   2025-12-07 00:00:00
```

### Restore from Backup

```bash
# Stop cluster (safe mode)
kubectl scale statefulset ruvector -n agentic-ops --replicas=0

# Restore backup
kubectl run -n agentic-ops restore-job \
  --image=ruvector/cli:latest \
  --restart=Never \
  --command -- ruvector-cli backup restore \
    --name auto-backup-20251208 \
    --destination /data/ruvector

# Wait for completion
kubectl wait -n agentic-ops --for=condition=complete job/restore-job --timeout=30m

# Restart cluster
kubectl scale statefulset ruvector -n agentic-ops --replicas=3

# Verify data
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status
```

## Scaling

### Vertical Scaling (Increase Resources)

```bash
# Increase memory limit
kubectl patch statefulset ruvector -n agentic-ops \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"ruvector","resources":{"limits":{"memory":"32Gi"}}}]}}}}'

# Increase CPU
kubectl patch statefulset ruvector -n agentic-ops \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"ruvector","resources":{"limits":{"cpu":"8"}}}]}}}}'

# Rolling restart
kubectl rollout restart statefulset ruvector -n agentic-ops
kubectl rollout status statefulset ruvector -n agentic-ops
```

### Horizontal Scaling (Add Nodes)

```bash
# Scale to 5 nodes
kubectl scale statefulset ruvector -n agentic-ops --replicas=5

# Monitor scaling
kubectl get pods -n agentic-ops -w

# Verify cluster
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status
# Expected:
# Nodes: 5/5 online
# Rebalancing: In progress (60%)
```

### Auto-Scaling

```yaml
spec:
  operations:
    autoscaling:
      enabled: true
      minReplicas: 3
      maxReplicas: 10
      metrics:
        - type: QueryLatency
          target: 50ms
        - type: MemoryUsage
          target: 70%
        - type: QPS
          target: 10000
```

## Performance Tuning

### Query Latency Optimization

**Problem:** P99 latency > 100ms

**Solutions:**

1. **Reduce efSearch:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set hnsw.efSearch 50
   ```

2. **Enable query cache:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set graph.cypher.enableQueryCache true
   ```

3. **Increase parallelism:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set performance.queryParallelism 16
   ```

### Memory Optimization

**Problem:** Memory usage > 80%

**Solutions:**

1. **Enable compression:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set compression.strategy Adaptive
   ```

2. **Reduce cache size:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set performance.memory.cacheSize 2Gi
   ```

3. **Trigger compaction:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli maintenance compact
   ```

### Index Build Optimization

**Problem:** Slow index build

**Solutions:**

1. **Increase build parallelism:**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set hnsw.buildParallelism 8
   ```

2. **Use batch inserts:**
   ```bash
   # In application code
   backend.batch_insert(vectors, 1000).await?;
   ```

3. **Reduce efConstruction (trade quality):**
   ```bash
   kubectl exec -n agentic-ops ruvector-0 -- \
     ruvector-cli config set hnsw.efConstruction 100
   ```

## Troubleshooting

### Common Issues

#### 1. Pod Stuck in Pending

```bash
# Check events
kubectl describe pod ruvector-0 -n agentic-ops

# Common causes:
# - Insufficient resources
# - PVC not bound
# - Node selector mismatch

# Solution: Check node resources
kubectl describe nodes | grep -A 5 "Allocated resources"
```

#### 2. High Replication Lag

```bash
# Check cluster status
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status --detailed

# Check network latency between nodes
kubectl exec -n agentic-ops ruvector-0 -- \
  ping ruvector-1.ruvector-headless

# Solution: Check network policies
kubectl get networkpolicies -n agentic-ops
```

#### 3. Learning Not Improving

```bash
# Check feedback volume
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli learning status

# Expected output:
# Total Queries: 5000+
# Patterns Collected: 500+
# Adaptations Applied: 10

# If patterns < 50:
# - Verify feedback is being recorded
# - Check application logs
# - Ensure implicit feedback is enabled
```

#### 4. Query Errors

```bash
# Check logs
kubectl logs -n agentic-ops ruvector-0 --tail=100 | grep ERROR

# Common errors:
# - "dimension mismatch": Check embedding model
# - "index not ready": Wait for index build
# - "out of memory": Increase memory limit

# Verify index
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli index status
```

### Emergency Procedures

#### Cluster Failure

```bash
# 1. Check all pods
kubectl get pods -n agentic-ops

# 2. If quorum lost (< 2 nodes):
# Force restart leader
kubectl delete pod ruvector-1 -n agentic-ops --force --grace-period=0

# 3. Wait for new leader election
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster wait-for-leader --timeout=60s

# 4. Verify cluster health
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status
```

#### Data Corruption

```bash
# 1. Identify corrupted node
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli maintenance check

# 2. Remove from cluster
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster remove-node ruvector-2

# 3. Delete pod and PVC
kubectl delete pod ruvector-2 -n agentic-ops
kubectl delete pvc data-ruvector-2 -n agentic-ops

# 4. Scale back up
kubectl scale statefulset ruvector -n agentic-ops --replicas=3

# 5. Verify recovery
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli cluster status
```

## Security

### Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: ruvector-netpol
  namespace: agentic-ops
spec:
  podSelector:
    matchLabels:
      app: ruvector
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          aof.dev/component: agent
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - podSelector:
        matchLabels:
          app: ruvector
    ports:
    - protocol: TCP
      port: 7946  # Raft
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: ruvector
    ports:
    - protocol: TCP
      port: 7946
```

### TLS Configuration

```bash
# Generate certificates
kubectl create secret tls ruvector-tls \
  --cert=ruvector.crt \
  --key=ruvector.key \
  -n agentic-ops

# Enable TLS in config
kubectl patch memory ruvector-rag-memory -n agentic-ops \
  --type=merge \
  -p '{"spec":{"security":{"encryption":{"inTransit":true}}}}'
```

### RBAC

```yaml
# rbac.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: ruvector-operator
  namespace: agentic-ops
rules:
- apiGroups: [""]
  resources: ["pods", "services", "endpoints"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["statefulsets"]
  verbs: ["get", "list", "watch", "update", "patch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: ruvector-operator-binding
  namespace: agentic-ops
subjects:
- kind: ServiceAccount
  name: ruvector
  namespace: agentic-ops
roleRef:
  kind: Role
  name: ruvector-operator
  apiGroup: rbac.authorization.k8s.io
```

## Cost Optimization

### Compression Strategy

```yaml
# Maximize compression for cost savings
compression:
  strategy: Adaptive
  tiers:
    hot:      # 7 days
      encoding: f16  # 2x savings
    warm:     # 30 days
      encoding: PQ8  # 4x savings
    cold:     # 90 days
      encoding: PQ4  # 8x savings
    archive:  # 365 days
      encoding: PQ4  # 8x savings
```

**Estimated Savings:**
- 1M vectors (1536 dims, f32) = 6GB raw
- With adaptive compression: ~1.5GB average (4x savings)
- Storage cost reduction: 75%

### Resource Right-Sizing

```bash
# Monitor actual usage
kubectl top pods -n agentic-ops

# Adjust requests/limits
kubectl patch statefulset ruvector -n agentic-ops \
  -p '{"spec":{"template":{"spec":{"containers":[{
    "name":"ruvector",
    "resources":{
      "requests":{"cpu":"1","memory":"4Gi"},
      "limits":{"cpu":"4","memory":"16Gi"}
    }
  }]}}}}'
```

## Maintenance

### Routine Tasks

**Daily:**
- Check cluster health
- Review error logs
- Monitor query latency
- Verify backups

**Weekly:**
- Review learning metrics
- Optimize compression tiers
- Check index fragmentation
- Update knowledge base

**Monthly:**
- Performance review
- Cost analysis
- Capacity planning
- Security audit

### Maintenance Window

```bash
# 1. Enable maintenance mode
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli maintenance enable

# 2. Perform maintenance tasks
# - Index optimization
# - Compaction
# - Backup verification

# 3. Disable maintenance mode
kubectl exec -n agentic-ops ruvector-0 -- \
  ruvector-cli maintenance disable
```

## Support

- **Documentation**: https://docs.ruvector.io
- **GitHub Issues**: https://github.com/ruvnet/ruvector/issues
- **Slack Channel**: #ruvector-ops
- **Email**: ops-support@ruvector.io

## Runbook

See [RUNBOOK.md](./RUNBOOK.md) for detailed incident response procedures.
