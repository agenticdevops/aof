---
sidebar_position: 2
sidebar_label: Kubernetes
---

# Kubernetes Agents

Production-ready agents for Kubernetes operations and debugging.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [pod-doctor](#pod-doctor) | Diagnose pod issues | kubectl |
| [hpa-tuner](#hpa-tuner) | Optimize autoscaling | kubectl |
| [netpol-debugger](#netpol-debugger) | Debug network policies | kubectl |
| [yaml-linter](#yaml-linter) | Validate K8s manifests | kubectl |
| [resource-optimizer](#resource-optimizer) | Right-size resources | kubectl |

## pod-doctor

Diagnoses pod issues including CrashLoopBackOff, ImagePullBackOff, OOMKilled, and other common failures.

### Usage

```bash
# Diagnose pods in a namespace
aofctl run agent library://kubernetes/pod-doctor \
  --prompt "Debug failing pods in namespace production"

# Diagnose a specific pod
aofctl run agent library://kubernetes/pod-doctor \
  --prompt "Why is pod api-server-abc123 in CrashLoopBackOff?"
```

### Capabilities

- Analyzes pod events and status conditions
- Examines container logs for error patterns
- Checks resource constraints (CPU, memory limits)
- Identifies image pull issues
- Detects OOMKilled patterns
- Recommends fixes based on diagnosis

### Example Output

```markdown
## Pod Diagnosis: api-server-abc123

**Status**: CrashLoopBackOff
**Namespace**: production
**Node**: node-worker-01

### Root Cause
The pod is experiencing OOMKilled events. Container memory limit
(256Mi) is insufficient for the application's memory requirements.

### Evidence
- Container terminated 5 times in last 10 minutes
- Exit code: 137 (SIGKILL from OOM)
- Memory usage peaked at 254Mi before termination

### Recommendations
1. Increase memory limit to 512Mi
2. Add memory request of 384Mi
3. Consider enabling memory profiling

### Fix Command
kubectl patch deployment api-server -n production \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"limits":{"memory":"512Mi"},"requests":{"memory":"384Mi"}}}]}}}}'
```

---

## hpa-tuner

Optimizes Horizontal Pod Autoscaler configurations for efficient scaling.

### Usage

```bash
# Analyze HPA in namespace
aofctl run agent library://kubernetes/hpa-tuner \
  --prompt "Optimize HPA settings for namespace production"

# Tune specific HPA
aofctl run agent library://kubernetes/hpa-tuner \
  --prompt "Why is the api-server HPA not scaling up?"
```

### Capabilities

- Analyzes HPA metrics and scaling history
- Identifies suboptimal scaling thresholds
- Detects HPAs that never scale (over-provisioned)
- Recommends target CPU/memory utilization
- Suggests min/max replica counts
- Analyzes scaling behavior patterns

### Example Output

```markdown
## HPA Analysis: api-server

**Current Configuration**:
- Min Replicas: 2
- Max Replicas: 10
- CPU Target: 80%

**Observed Behavior** (last 7 days):
- Actual replicas: 2 (never scaled)
- Avg CPU utilization: 15%
- Peak CPU utilization: 35%

### Issues Detected
1. HPA is over-provisioned - never scales beyond minimum
2. CPU target (80%) is never reached

### Recommendations
1. Reduce min replicas to 1 (save 50% baseline cost)
2. Lower CPU target to 50% for more responsive scaling
3. Consider adding memory-based scaling

### Optimized Configuration
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-server
spec:
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 50
```
```

---

## netpol-debugger

Debugs Kubernetes NetworkPolicy issues and connectivity problems.

### Usage

```bash
# Debug connectivity between pods
aofctl run agent library://kubernetes/netpol-debugger \
  --prompt "Why can't frontend pods connect to api-server?"

# Analyze network policies
aofctl run agent library://kubernetes/netpol-debugger \
  --prompt "List all network policies affecting namespace production"
```

### Capabilities

- Traces network connectivity between pods
- Identifies blocking NetworkPolicies
- Analyzes ingress/egress rules
- Detects missing allow rules
- Recommends policy fixes
- Visualizes policy relationships

### Example Output

```markdown
## Network Policy Debug: frontend → api-server

**Source**: frontend-abc123 (namespace: web)
**Destination**: api-server-xyz789 (namespace: api, port: 8080)

### Connectivity Status: BLOCKED

### Blocking Policy
NetworkPolicy `api-server-ingress` in namespace `api` is blocking traffic.

**Policy Analysis**:
```yaml
ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          network-zone: internal
```

The source namespace `web` does not have label `network-zone: internal`.

### Resolution Options

**Option 1**: Label the source namespace
```bash
kubectl label namespace web network-zone=internal
```

**Option 2**: Update NetworkPolicy to allow from `web` namespace
```yaml
ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: web
```
```

---

## yaml-linter

Validates Kubernetes YAML manifests for correctness and best practices.

### Usage

```bash
# Validate a manifest file
aofctl run agent library://kubernetes/yaml-linter \
  --prompt "Validate deployment.yaml" \
  --file deployment.yaml

# Check for security issues
aofctl run agent library://kubernetes/yaml-linter \
  --prompt "Check security best practices in my manifests"
```

### Capabilities

- Validates YAML syntax and schema
- Checks API version compatibility
- Identifies deprecated APIs
- Security misconfigurations (privileged, hostNetwork, etc.)
- Best practice recommendations
- Resource quota compliance

### Example Output

```markdown
## Manifest Validation: deployment.yaml

**Resource**: Deployment/api-server
**API Version**: apps/v1

### Validation Results

| Check | Status | Details |
|-------|--------|---------|
| Schema Valid | PASS | Valid Deployment spec |
| API Version | PASS | apps/v1 is current |
| Resource Limits | WARN | No memory limit specified |
| Security Context | FAIL | Running as root |
| Probes | WARN | No readinessProbe defined |

### Security Issues

1. **HIGH**: Container runs as root
   ```yaml
   securityContext:
     runAsNonRoot: true
     runAsUser: 1000
   ```

2. **MEDIUM**: No security context at pod level
   ```yaml
   securityContext:
     fsGroup: 1000
     runAsNonRoot: true
   ```

### Recommendations
1. Add resource limits for memory
2. Enable readinessProbe for traffic management
3. Remove privileged: true
```

---

## resource-optimizer

Analyzes and optimizes Kubernetes resource requests and limits.

### Usage

```bash
# Analyze namespace resource usage
aofctl run agent library://kubernetes/resource-optimizer \
  --prompt "Optimize resources in namespace production"

# Right-size a specific deployment
aofctl run agent library://kubernetes/resource-optimizer \
  --prompt "Right-size the api-server deployment"
```

### Capabilities

- Analyzes actual vs requested resources
- Identifies over/under-provisioned workloads
- Calculates optimal requests and limits
- Estimates cost savings
- Recommends resource quotas
- QoS class optimization

### Example Output

```markdown
## Resource Optimization: namespace production

### Summary
- **Total Deployments**: 12
- **Over-provisioned**: 8 (67%)
- **Under-provisioned**: 2 (17%)
- **Optimal**: 2 (17%)
- **Estimated Savings**: $340/month

### Top Optimization Opportunities

| Deployment | Current CPU | Recommended | Savings |
|------------|-------------|-------------|---------|
| api-server | 2000m | 500m | $120/mo |
| worker | 1000m | 250m | $80/mo |
| cache | 500m | 200m | $40/mo |

### Detailed Recommendations

#### api-server
**Current**:
- CPU Request: 2000m, Limit: 4000m
- Memory Request: 4Gi, Limit: 8Gi

**Observed** (p95 over 7 days):
- CPU: 380m peak
- Memory: 1.2Gi peak

**Recommended**:
- CPU Request: 400m, Limit: 800m
- Memory Request: 1.5Gi, Limit: 2Gi

```yaml
resources:
  requests:
    cpu: 400m
    memory: 1.5Gi
  limits:
    cpu: 800m
    memory: 2Gi
```
```

---

## Environment Setup

All Kubernetes agents require:

```bash
# Kubernetes access
export KUBECONFIG=~/.kube/config

# Or use in-cluster config (for running inside K8s)
# No environment variable needed - uses service account
```

## Next Steps

- [Observability Agents](./observability.md) - Monitor and analyze
- [Incident Agents](./incident.md) - Respond to incidents
- [First Agent Tutorial](../tutorials/first-agent.md) - Build custom agents
