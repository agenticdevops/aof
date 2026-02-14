---
sidebar_position: 4
title: Bundled Skills
description: Documentation for skills included with AOF
---

# Bundled Skills

AOF ships with a set of essential ops skills. These provide a foundation for common operations and serve as examples for writing your own skills.

## Overview

| Skill | Description | Requirements |
|-------|-------------|--------------|
| [k8s-debug](#k8s-debug) | Kubernetes pod debugging | `kubectl`, `~/.kube/config` |
| [prometheus-query](#prometheus-query) | PromQL queries and alerting | `curl` or `promtool` |
| [argocd-sync](#argocd-sync) | ArgoCD application management | `argocd`, `kubectl` |
| [loki-search](#loki-search) | LogQL queries and log analysis | `logcli` or `curl` |
| [incident-diagnose](#incident-diagnose) | Systematic incident triage | None (always loaded) |

---

## k8s-debug

**Purpose**: Expert guidance for debugging Kubernetes workloads, analyzing pod issues, and troubleshooting cluster problems.

### When to Use

- Pod is in CrashLoopBackOff, ImagePullBackOff, or Pending state
- Application logs show errors or unexpected behavior
- Services are not reachable
- Resource constraints causing issues

### Requirements

- `kubectl` binary in PATH
- `~/.kube/config` exists with cluster access

### Key Capabilities

- **Pod Status Analysis**: Diagnose pod states and events
- **Log Analysis**: Retrieve and analyze container logs
- **Resource Debugging**: Check CPU/memory usage
- **Network Troubleshooting**: Service connectivity checks
- **Interactive Debugging**: Exec into pods, ephemeral containers

### Quick Reference

```bash
# Pod diagnostics
kubectl get pods -o wide
kubectl describe pod <pod-name>
kubectl logs <pod-name> --previous

# Resource usage
kubectl top pods
kubectl top nodes

# Interactive debugging
kubectl exec -it <pod> -- /bin/sh
kubectl debug -it <pod> --image=busybox
```

---

## prometheus-query

**Purpose**: Expert guidance for writing PromQL queries, analyzing metrics, and troubleshooting alerting rules.

### When to Use

- Building PromQL queries for dashboards or alerts
- Investigating metric anomalies
- Debugging alerting rules
- Capacity planning with historical data

### Requirements

- `promtool` OR `curl` available

### Key Capabilities

- **PromQL Patterns**: Rate, increase, histogram quantiles
- **Aggregation**: Sum, avg, topk by labels
- **Operational Queries**: Error rates, latency, resource usage
- **Alert Rules**: Writing and debugging alert expressions

### Quick Reference

```promql
# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100

# P95 latency
histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))

# Pod restarts
increase(kube_pod_container_status_restarts_total[1h])
```

---

## argocd-sync

**Purpose**: Expert guidance for ArgoCD application management, sync operations, and GitOps troubleshooting.

### When to Use

- Syncing applications to desired state
- Investigating sync failures
- Rolling back deployments
- Managing application configuration

### Requirements

- `argocd` CLI in PATH
- `kubectl` for cluster operations

### Key Capabilities

- **Sync Operations**: Sync, prune, force sync
- **Status Analysis**: Health and sync status interpretation
- **Rollback**: Application history and rollback
- **Troubleshooting**: Sync failures, drift detection

### Quick Reference

```bash
# Application status
argocd app list
argocd app get <app-name>

# Sync operations
argocd app sync <app-name>
argocd app sync <app-name> --prune

# Rollback
argocd app history <app-name>
argocd app rollback <app-name> <revision>

# Diff
argocd app diff <app-name>
```

---

## loki-search

**Purpose**: Expert guidance for querying logs with Loki, writing LogQL queries, and analyzing log patterns.

### When to Use

- Searching logs for errors or specific events
- Correlating logs across services
- Building log-based alerts
- Investigating incidents with log data

### Requirements

- `logcli` OR `curl` available

### Key Capabilities

- **LogQL Queries**: Stream selectors, filters, parsers
- **Log Metrics**: count_over_time, rate from logs
- **Pattern Matching**: Regex and line filters
- **Aggregation**: Sum, quantile from extracted values

### Quick Reference

```logql
# Find errors
{namespace="production"} |= "error"

# JSON parsing with filter
{job="api"} | json | level="error"

# Error count by service
sum by (service) (count_over_time({namespace="prod"} | json | level="error" [5m]))

# P99 latency from logs
quantile_over_time(0.99, {job="api"} | json | unwrap response_time [5m]) by (endpoint)
```

```bash
# LogCLI usage
logcli query '{job="api"}' --from="1h"
logcli query '{job="api"}' --tail
```

---

## incident-diagnose

**Purpose**: Systematic methodology for diagnosing production incidents, performing root cause analysis, and efficient triage.

### When to Use

- Production incident has been declared
- Customer-impacting issues reported
- Alerts firing requiring investigation
- Post-incident analysis needed

### Requirements

None - this skill is marked `always: true` and loads regardless of available tools.

### Key Capabilities

- **Triage Framework**: Impact assessment, severity classification
- **Diagnosis Workflows**: High error rate, latency, outages
- **Root Cause Analysis**: 5 Whys, timeline reconstruction
- **Communication Templates**: Status updates, escalation requests
- **Post-Incident**: Checklist, post-mortem template

### Severity Classification

| Severity | Criteria | Response |
|----------|----------|----------|
| **SEV1** | Complete outage, data loss, security breach | All hands, exec notification |
| **SEV2** | Major feature broken, significant user impact | Team mobilization |
| **SEV3** | Partial degradation, workaround available | On-call investigation |
| **SEV4** | Minor issue, no immediate user impact | Normal ticket workflow |

### Incident Checklist

- [ ] Acknowledge incident
- [ ] Assess impact and severity
- [ ] Start incident channel/bridge
- [ ] Assign roles (IC, Comms, Technical)
- [ ] Form initial hypothesis
- [ ] Gather data to confirm/refute
- [ ] Implement mitigation
- [ ] Verify resolution
- [ ] Communicate resolution
- [ ] Document for post-mortem

---

## Using Bundled Skills

### List Available Skills

```bash
aofctl skills list
```

### Check Requirements

```bash
aofctl skills check k8s-debug
```

### View Full Content

```bash
aofctl skills show prometheus-query
```

### Override with Workspace Skills

Create a skill with the same name in `.claude/skills/` to override the bundled version:

```
.claude/skills/
└── k8s-debug/
    └── SKILL.md    # Your customized version
```

Workspace skills take precedence over bundled skills.

---

## Extending Bundled Skills

### Adding Company-Specific Context

Create a wrapper skill that references the bundled skill:

```markdown
---
name: k8s-debug-acme
description: "ACME Corp Kubernetes debugging procedures"
metadata:
  requires:
    bins: ["kubectl"]
  tags: ["kubernetes", "acme"]
---

# ACME Kubernetes Debugging

Follow the standard k8s-debug procedures with these ACME-specific additions:

## ACME Cluster Access
```bash
# Get cluster credentials
gcloud container clusters get-credentials acme-prod --zone us-central1-a
```

## ACME-Specific Namespaces
- `acme-api` - Core API services
- `acme-workers` - Background job processors
- `acme-data` - Database proxies

## Escalation
If issue persists after 15 minutes, page the SRE team:
```bash
pd trigger --service-id ACME_SRE --message "K8s issue: <description>"
```
```
