# CI/CD Agent Examples

This directory contains example agents for managing CI/CD pipelines across multiple platforms.

## Available Agents

| Agent | Platform | Description |
|-------|----------|-------------|
| `github-actions-ops` | GitHub Actions | Manage workflows, runs, and artifacts |
| `gitlab-ci-ops` | GitLab CI/CD | Manage pipelines, jobs, and logs |
| `argocd-ops` | ArgoCD | Manage GitOps applications and syncs |
| `flux-ops` | Flux CD | Manage kustomizations and Helm releases |

## Prerequisites

### Feature Flag

Enable the `cicd` feature in your AOF configuration:

```toml
aof-tools = { version = "0.2", features = ["cicd"] }
```

### Authentication

Each agent requires platform-specific authentication:

**GitHub Actions:**
```bash
export GITHUB_TOKEN="ghp_xxxx"
export GITHUB_OWNER="your-org"
export GITHUB_REPO="your-repo"
```

**GitLab CI:**
```bash
export GITLAB_ENDPOINT="https://gitlab.com"
export GITLAB_TOKEN="glpat-xxxx"
export GITLAB_PROJECT_ID="123"
```

**ArgoCD:**
```bash
export ARGOCD_SERVER="https://argocd.example.com"
export ARGOCD_TOKEN="xxxx"
```

**Flux CD:**
```bash
export KUBECONFIG="~/.kube/config"
export KUBE_CONTEXT="my-cluster"
```

## Usage

### GitHub Actions

```bash
# Check workflow status
aofctl run agent github-actions-ops "show recent workflow runs"

# Trigger deployment
aofctl run agent github-actions-ops "trigger deploy workflow to production"

# Debug failures
aofctl run agent github-actions-ops "why did the last CI run fail?"
```

### GitLab CI

```bash
# Monitor pipelines
aofctl run agent gitlab-ci-ops "list recent pipelines for main branch"

# Deploy with variables
aofctl run agent gitlab-ci-ops "create pipeline with VERSION=2.0.0"

# Investigate failures
aofctl run agent gitlab-ci-ops "get logs for failed job in pipeline 1234"
```

### ArgoCD

```bash
# Check sync status
aofctl run agent argocd-ops "show all out-of-sync applications"

# Sync application
aofctl run agent argocd-ops "sync my-app to revision v1.2.3"

# Review changes
aofctl run agent argocd-ops "show diff for production-api"
```

### Flux

```bash
# Check resources
aofctl run agent flux-ops "list all kustomizations and their status"

# Force reconciliation
aofctl run agent flux-ops "reconcile the apps kustomization"

# Debug issues
aofctl run agent flux-ops "get helm-controller logs"
```

## Customization

These agents can be customized by:

1. **Adding more tools**: Include additional CI/CD tools in the `tools` list
2. **Adjusting prompts**: Modify `system_prompt` for your workflow
3. **Adding variables**: Include project-specific variables in `environment`

## Related Documentation

- [CI/CD Tools Reference](../../../docs/tools/cicd.md)
- [Built-in Tools](../../../docs/tools/builtin-tools.md)
- [Agent Configuration](../../../docs/concepts/agents.md)
