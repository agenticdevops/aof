# Deploying AOF on Kubernetes

Quick guide for deploying AOF daemon on Kubernetes with high availability.

## Quick Deploy

```bash
# Clone repository
git clone https://github.com/agenticdevops/aof
cd aof

# Create namespace and deploy
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secrets.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/statefulset.yaml

# Verify deployment
kubectl get pods -n aof-system
kubectl logs -n aof-system aof-daemon-0 -f
```

## Configuration

### 1. Create Secrets

```bash
kubectl create secret generic aof-secrets \
  --from-literal=anthropic-api-key=sk-ant-your-key \
  --from-literal=slack-bot-token=xoxb-your-token \
  --from-literal=slack-signing-secret=your-secret \
  --namespace=aof-system
```

### 2. Customize ConfigMap

Edit `k8s/configmap.yaml` to adjust:
- `max_concurrent_tasks`
- `task_timeout_secs`
- Coordination mode (full, standard, reduced, heartbeat_only)

Apply changes:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl rollout restart statefulset/aof-daemon -n aof-system
```

### 3. Scale Deployment

```bash
kubectl scale statefulset/aof-daemon --replicas=3 -n aof-system
```

## Monitoring

### Health Checks

Liveness and readiness probes are pre-configured in StatefulSet:

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

### Prometheus Integration

StatefulSet includes Prometheus annotations:

```yaml
annotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "8080"
  prometheus.io/path: "/metrics"
```

With Prometheus Operator:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: aof-daemon
  namespace: aof-system
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: aof
  endpoints:
  - port: http
    path: /metrics
    interval: 15s
```

### View Logs

```bash
kubectl logs -n aof-system aof-daemon-0 -f
kubectl logs -n aof-system aof-daemon-0 --since=1h | jq
```

## Storage

StatefulSet uses persistent volumes for:
- **Data**: 10Gi (sessions, decisions)
- **Checkpoints**: 5Gi (workflow checkpoints)

Adjust in `k8s/statefulset.yaml`:

```yaml
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 20Gi  # Increase
```

## Upgrading

### Rolling Update

```bash
# Update image
kubectl set image statefulset/aof-daemon \
  aof=aof:v0.4.1 \
  -n aof-system

# Monitor rollout
kubectl rollout status statefulset/aof-daemon -n aof-system
```

### Rollback

```bash
kubectl rollout undo statefulset/aof-daemon -n aof-system
```

## Troubleshooting

### Pod Not Starting

```bash
kubectl describe pod aof-daemon-0 -n aof-system
kubectl logs aof-daemon-0 -n aof-system --previous
```

### Health Check Failing

```bash
kubectl exec -it aof-daemon-0 -n aof-system -- curl http://localhost:8080/health
kubectl exec -it aof-daemon-0 -n aof-system -- curl http://localhost:8080/ready
```

### Persistent Volume Issues

```bash
kubectl get pvc -n aof-system
kubectl describe pvc data-aof-daemon-0 -n aof-system
```

## Production Recommendations

1. **Resource Limits**: Adjust based on workload
   - Requests: 512Mi/500m (minimum)
   - Limits: 2Gi/2000m (prevents noisy neighbor)

2. **Replicas**: Start with 1, scale to 3+ for HA

3. **Pod Disruption Budget**:

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: aof-daemon
  namespace: aof-system
spec:
  minAvailable: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: aof
```

4. **Network Policy**: Restrict ingress to authorized clients

5. **Backup**: Use Velero or similar for PVC backups

## Next Steps

- [Configure Ingress](./ingress.md)
- [Set up Monitoring](./monitoring.md)
- [Backup and Restore](./backup.md)
