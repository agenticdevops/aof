---
name: argocd-sync
description: "ArgoCD application management, sync operations, and GitOps troubleshooting"
homepage: "https://docs.aof.sh/skills/argocd-sync"
metadata:
  emoji: "🔄"
  version: "1.0.0"
  author: "AOF Team"
  license: "Apache-2.0"
  requires:
    bins:
      - argocd
    any_bins:
      - kubectl
  install:
    - id: brew-argocd
      kind: brew
      package: argocd
      bins:
        - argocd
  tags:
    - argocd
    - gitops
    - deployments
    - sync
    - kubernetes
---

# ArgoCD Sync Skill

Expert guidance for ArgoCD application management, sync operations, rollbacks, and GitOps troubleshooting.

## When to Use This Skill

- Syncing applications to desired state
- Investigating sync failures
- Rolling back deployments
- Managing application configuration
- Troubleshooting health status issues
- Handling drift between Git and cluster

## Quick Commands

### Application Status

```bash
# List all applications
argocd app list

# Get application details
argocd app get <app-name>

# Get sync status
argocd app get <app-name> -o json | jq '.status.sync'

# Get health status
argocd app get <app-name> -o json | jq '.status.health'
```

### Sync Operations

```bash
# Sync application to Git
argocd app sync <app-name>

# Sync with prune (delete resources not in Git)
argocd app sync <app-name> --prune

# Sync specific resources only
argocd app sync <app-name> --resource apps:Deployment:my-deploy

# Force sync (bypass hooks)
argocd app sync <app-name> --force

# Dry-run sync
argocd app sync <app-name> --dry-run
```

### Rollback

```bash
# List history
argocd app history <app-name>

# Rollback to previous version
argocd app rollback <app-name>

# Rollback to specific revision
argocd app rollback <app-name> <revision>
```

## Sync Status Explained

| Status | Description |
|--------|-------------|
| `Synced` | Application state matches Git |
| `OutOfSync` | Live state differs from Git |
| `Unknown` | Cannot determine sync status |

### Health Status

| Status | Description |
|--------|-------------|
| `Healthy` | All resources healthy |
| `Progressing` | Resources updating |
| `Degraded` | One or more resources unhealthy |
| `Suspended` | Resources suspended (e.g., paused rollout) |
| `Missing` | Resource exists in Git but not cluster |
| `Unknown` | Health cannot be determined |

## Troubleshooting Sync Issues

### Application Out of Sync

**Diagnosis:**
```bash
# See what's different
argocd app diff <app-name>

# Detailed diff
argocd app diff <app-name> --local <path-to-manifests>
```

**Common Causes:**
- Manual changes to cluster (drift)
- Resource was modified by another controller
- Helm values overrides not matching

**Solutions:**
```bash
# Sync to restore Git state
argocd app sync <app-name>

# If resources should be deleted
argocd app sync <app-name> --prune
```

### Sync Failed

**Diagnosis:**
```bash
# Check sync result
argocd app get <app-name> -o json | jq '.status.operationState'

# Check events
kubectl get events -n argocd --sort-by='.lastTimestamp' | grep <app-name>
```

**Common Causes:**
1. **Invalid manifests** - YAML syntax errors
2. **Resource validation failed** - CRD not installed, schema mismatch
3. **Permission denied** - RBAC issues
4. **Namespace doesn't exist**
5. **Resource already exists** - Not managed by ArgoCD

**Solutions:**
```bash
# Validate manifests locally
kubectl apply --dry-run=client -f <manifest>

# Check ArgoCD has permissions
kubectl auth can-i create deployments --as=system:serviceaccount:argocd:argocd-application-controller -n <namespace>
```

### Application Degraded

**Diagnosis:**
```bash
# Get resource health details
argocd app resources <app-name>

# Check specific resource
argocd app get <app-name> --resource <kind>:<name>
```

**Common Causes:**
- Pod not ready (probe failing)
- Deployment replicas mismatch
- PVC not bound
- Service endpoints not ready

### Stuck in Progressing

**Diagnosis:**
```bash
# Check what's still progressing
argocd app get <app-name> -o json | jq '.status.resources[] | select(.health.status=="Progressing")'
```

**Common Causes:**
- Deployment stuck waiting for pods
- HPA scaling in progress
- PDB blocking rollout

**Solution:**
```bash
# Check underlying pods
kubectl get pods -n <namespace> -l app=<app>

# Check rollout status
kubectl rollout status deployment/<name> -n <namespace>
```

## Application Management

### Create Application

```bash
# From Git repository
argocd app create <app-name> \
  --repo https://github.com/org/repo.git \
  --path <path> \
  --dest-server https://kubernetes.default.svc \
  --dest-namespace <namespace>

# With Helm
argocd app create <app-name> \
  --repo https://charts.example.com \
  --helm-chart <chart-name> \
  --revision <version> \
  --dest-server https://kubernetes.default.svc \
  --dest-namespace <namespace>
```

### Update Application

```bash
# Update source revision
argocd app set <app-name> --revision <branch/tag>

# Update Helm values
argocd app set <app-name> --helm-set key=value

# Update from values file
argocd app set <app-name> --values-literal-file values.yaml
```

### Delete Application

```bash
# Delete app (keep resources)
argocd app delete <app-name>

# Delete app and resources
argocd app delete <app-name> --cascade
```

## Sync Policies

### Auto-Sync

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
spec:
  syncPolicy:
    automated:
      prune: true      # Delete resources not in Git
      selfHeal: true   # Revert manual changes
      allowEmpty: false # Don't sync if no resources
```

### Sync Options

```yaml
spec:
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
      - PrunePropagationPolicy=foreground
      - PruneLast=true
      - ApplyOutOfSyncOnly=true
      - ServerSideApply=true
```

## Hooks and Waves

### Sync Hooks

```yaml
metadata:
  annotations:
    argocd.argoproj.io/hook: PreSync    # Run before sync
    argocd.argoproj.io/hook: PostSync   # Run after sync
    argocd.argoproj.io/hook: SyncFail   # Run on failure
    argocd.argoproj.io/hook-delete-policy: HookSucceeded
```

### Sync Waves

```yaml
metadata:
  annotations:
    argocd.argoproj.io/sync-wave: "-1"  # Run first (lower = earlier)
```

## Best Practices

1. **Use sync waves** for dependencies (CRDs before CRs)
2. **Enable auto-sync** with selfHeal for GitOps compliance
3. **Use prune cautiously** - can delete unintended resources
4. **Set resource tracking** appropriately (annotation vs label)
5. **Use Projects** for RBAC and source restrictions
6. **Monitor sync status** in dashboards/alerts

## CLI Authentication

```bash
# Login to ArgoCD
argocd login <argocd-server> --username admin --password <password>

# Login with SSO
argocd login <argocd-server> --sso

# Use port-forward
kubectl port-forward svc/argocd-server -n argocd 8080:443
argocd login localhost:8080 --insecure
```

## Useful Commands Reference

| Task | Command |
|------|---------|
| List apps | `argocd app list` |
| Sync app | `argocd app sync <app>` |
| Diff app | `argocd app diff <app>` |
| Get app | `argocd app get <app>` |
| Rollback | `argocd app rollback <app> <rev>` |
| History | `argocd app history <app>` |
| Terminate sync | `argocd app terminate-op <app>` |
| Refresh | `argocd app get <app> --refresh` |
| Hard refresh | `argocd app get <app> --hard-refresh` |
