---
name: argocd-deploy
description: "ArgoCD deployment and rollback operations"
homepage: "https://docs.aof.sh/skills/argocd-deploy"
metadata:
  emoji: "🚀"
  version: "1.0.0"
  requires:
    bins: ["argocd", "kubectl"]
    env: []
    config: ["~/.kube/config"]
  tags: ["argocd", "deployment", "gitops"]
---

# ArgoCD Deploy Skill

Manage ArgoCD applications for continuous deployment and GitOps workflows.

## When to Use This Skill

- Deploying new versions
- Syncing application state
- Rolling back deployments
- Checking deployment status
- Managing environments

## Steps

1. **List applications** — `argocd app list`
2. **Sync application** — `argocd app sync app-name`
3. **Check status** — `argocd app get app-name`
4. **Rollback** — `argocd app rollback app-name revision`
5. **Check history** — `argocd app history app-name`
