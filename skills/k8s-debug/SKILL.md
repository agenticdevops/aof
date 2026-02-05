---
name: k8s-debug
description: "Kubernetes pod debugging, log analysis, and troubleshooting"
homepage: "https://docs.aof.sh/skills/k8s-debug"
metadata:
  emoji: "🐳"
  version: "1.0.0"
  author: "AOF Team"
  license: "Apache-2.0"
  requires:
    bins:
      - kubectl
    env: []
    config:
      - "~/.kube/config"
  install:
    - id: brew-kubectl
      kind: brew
      package: kubernetes-cli
      bins:
        - kubectl
    - id: apt-kubectl
      kind: apt
      package: kubectl
      bins:
        - kubectl
  tags:
    - kubernetes
    - debugging
    - pods
    - logs
    - troubleshooting
---

# Kubernetes Debug Skill

Expert guidance for debugging Kubernetes workloads, analyzing pod issues, and troubleshooting cluster problems.

## When to Use This Skill

- Pod is in CrashLoopBackOff, ImagePullBackOff, or Pending state
- Application logs show errors or unexpected behavior
- Services are not reachable or load balancing issues
- Resource constraints (CPU/memory) causing problems
- Network policies blocking traffic
- Configuration issues (ConfigMaps, Secrets)

## Quick Diagnostics

### Pod Status Overview
```bash
# Get pod status with events
kubectl get pods -o wide
kubectl describe pod <pod-name>

# Get events sorted by time
kubectl get events --sort-by='.lastTimestamp'
```

### Log Analysis
```bash
# Current logs
kubectl logs <pod-name> [-c <container>]

# Previous container logs (after crash)
kubectl logs <pod-name> --previous

# Follow logs in real-time
kubectl logs -f <pod-name>

# Logs with timestamps
kubectl logs <pod-name> --timestamps

# Last N lines
kubectl logs <pod-name> --tail=100
```

### Resource Usage
```bash
# Pod resource usage
kubectl top pods

# Node resource usage
kubectl top nodes

# Detailed resource requests/limits
kubectl get pods -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[*].resources}{"\n"}{end}'
```

## Common Issues and Solutions

### CrashLoopBackOff

**Symptoms:** Pod repeatedly crashes and restarts

**Diagnosis Steps:**
1. Check logs: `kubectl logs <pod> --previous`
2. Check events: `kubectl describe pod <pod>`
3. Verify image exists and is accessible
4. Check resource limits (OOMKilled?)
5. Verify environment variables and secrets

**Common Causes:**
- Application error on startup
- Missing dependencies or config
- Insufficient memory (OOMKilled)
- Liveness probe failing
- Missing or incorrect command/args

### ImagePullBackOff

**Symptoms:** Pod stuck trying to pull image

**Diagnosis:**
```bash
kubectl describe pod <pod> | grep -A5 "Events"
```

**Common Causes:**
- Image doesn't exist
- Wrong image tag
- Private registry without imagePullSecret
- Network issues reaching registry

**Fix:**
```bash
# Check secret exists
kubectl get secret <pull-secret>

# Test image pull manually
docker pull <image>
```

### Pending State

**Symptoms:** Pod stuck in Pending

**Diagnosis:**
```bash
kubectl describe pod <pod> | grep -A10 "Events"
```

**Common Causes:**
- Insufficient resources on nodes
- Node selector/affinity not matching
- PVC not bound
- Taints preventing scheduling

**Check Resources:**
```bash
kubectl describe nodes | grep -A5 "Allocated resources"
```

### OOMKilled

**Symptoms:** Container killed due to memory

**Diagnosis:**
```bash
kubectl describe pod <pod> | grep -i "OOMKilled"
kubectl get pod <pod> -o jsonpath='{.status.containerStatuses[*].lastState}'
```

**Solution:**
- Increase memory limits
- Fix memory leak in application
- Add horizontal pod autoscaling

## Network Debugging

### Service Connectivity
```bash
# Check service endpoints
kubectl get endpoints <service>

# Test DNS resolution
kubectl run tmp-shell --rm -i --tty --image nicolaka/netshoot -- nslookup <service>

# Test connectivity
kubectl run tmp-shell --rm -i --tty --image nicolaka/netshoot -- curl <service>:<port>
```

### Network Policies
```bash
# List network policies
kubectl get networkpolicies

# Describe policy
kubectl describe networkpolicy <policy>
```

## Interactive Debugging

### Exec into Pod
```bash
# Shell into container
kubectl exec -it <pod> -- /bin/sh

# Specific container
kubectl exec -it <pod> -c <container> -- /bin/bash
```

### Debug Container (Kubernetes 1.25+)
```bash
# Ephemeral debug container
kubectl debug -it <pod> --image=busybox --target=<container>

# Debug node
kubectl debug node/<node> -it --image=ubuntu
```

## Best Practices

1. **Always check events first** - They often reveal the root cause
2. **Use `--previous` for crash logs** - The current container may be too new
3. **Compare with working pods** - Diff configurations
4. **Check resource metrics** - CPU/memory pressure is common
5. **Verify network connectivity** - Use debug pods with network tools
6. **Check RBAC** - Service accounts may lack permissions

## Related Commands Reference

| Task | Command |
|------|---------|
| Get all resources in namespace | `kubectl get all -n <ns>` |
| Port forward to pod | `kubectl port-forward <pod> <local>:<remote>` |
| Copy files from pod | `kubectl cp <pod>:<path> <local-path>` |
| Run command in pod | `kubectl exec <pod> -- <command>` |
| Scale deployment | `kubectl scale deployment <name> --replicas=N` |
| Rollout status | `kubectl rollout status deployment/<name>` |
| Rollback | `kubectl rollout undo deployment/<name>` |
