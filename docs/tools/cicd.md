# CI/CD Tools

AOF provides comprehensive CI/CD integration tools for GitHub Actions, GitLab CI, ArgoCD, and Flux. These tools enable agents to monitor, trigger, and manage CI/CD pipelines programmatically.

## Feature Flag

All CI/CD tools require the `cicd` feature flag:

```toml
# Cargo.toml
aof-tools = { version = "0.2", features = ["cicd"] }
```

## Available Tools

| Tool | Platform | Description |
|------|----------|-------------|
| `github_workflow_list` | GitHub Actions | List workflows in a repository |
| `github_workflow_dispatch` | GitHub Actions | Trigger workflow run |
| `github_run_list` | GitHub Actions | List workflow runs |
| `github_run_get` | GitHub Actions | Get run details |
| `github_run_cancel` | GitHub Actions | Cancel running workflow |
| `github_run_rerun` | GitHub Actions | Rerun failed workflow |
| `gitlab_pipeline_list` | GitLab CI | List project pipelines |
| `gitlab_pipeline_get` | GitLab CI | Get pipeline details |
| `gitlab_pipeline_create` | GitLab CI | Trigger new pipeline |
| `gitlab_pipeline_cancel` | GitLab CI | Cancel running pipeline |
| `gitlab_pipeline_retry` | GitLab CI | Retry failed pipeline |
| `gitlab_job_list` | GitLab CI | List jobs in pipeline |
| `gitlab_job_get` | GitLab CI | Get job details |
| `gitlab_job_log` | GitLab CI | Get job logs |
| `argocd_app_list` | ArgoCD | List applications |
| `argocd_app_get` | ArgoCD | Get application details |
| `argocd_app_sync` | ArgoCD | Sync application |
| `argocd_app_refresh` | ArgoCD | Refresh application |
| `argocd_app_history` | ArgoCD | Get sync history |
| `argocd_app_diff` | ArgoCD | Get application diff |
| `flux_kustomization_list` | Flux | List kustomizations |
| `flux_kustomization_get` | Flux | Get kustomization details |
| `flux_helmrelease_list` | Flux | List Helm releases |
| `flux_helmrelease_get` | Flux | Get Helm release details |
| `flux_reconcile` | Flux | Trigger reconciliation |
| `flux_suspend` | Flux | Suspend resource |
| `flux_resume` | Flux | Resume resource |
| `flux_logs` | Flux | Get controller logs |

---

## GitHub Actions

### Authentication

Requires a GitHub Personal Access Token (PAT) with `repo` scope:

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
```

### github_workflow_list

List all workflows in a repository.

**Parameters:**
- `token` (required): GitHub PAT
- `owner` (required): Repository owner
- `repo` (required): Repository name
- `per_page`: Results per page (default: 30, max: 100)
- `page`: Page number (default: 1)

**Example Agent:**

```yaml
name: github-workflow-monitor
spec:
  llm:
    provider: google
    model: gemini-2.5-flash
  tools:
    - github_workflow_list
    - github_run_list
  system_prompt: |
    Monitor GitHub Actions workflows.
    Token: Use $GITHUB_TOKEN environment variable.
```

### github_workflow_dispatch

Trigger a workflow run with optional inputs.

**Parameters:**
- `token` (required): GitHub PAT
- `owner` (required): Repository owner
- `repo` (required): Repository name
- `workflow_id` (required): Workflow ID or filename
- `ref` (required): Branch or tag to run on
- `inputs`: Workflow input values (object)

**Example:**
```
Trigger the deploy workflow on main branch with environment=production
```

### github_run_list

List workflow runs with filtering.

**Parameters:**
- `token`, `owner`, `repo` (required)
- `workflow_id`: Filter by workflow
- `branch`: Filter by branch
- `status`: Filter by status (queued, in_progress, completed)
- `event`: Filter by trigger event (push, pull_request, etc.)
- `per_page`, `page`: Pagination

### github_run_get

Get detailed information about a specific run.

**Parameters:**
- `token`, `owner`, `repo` (required)
- `run_id` (required): Workflow run ID

### github_run_cancel

Cancel a running workflow.

**Parameters:**
- `token`, `owner`, `repo` (required)
- `run_id` (required): Run ID to cancel

### github_run_rerun

Rerun a failed workflow run.

**Parameters:**
- `token`, `owner`, `repo` (required)
- `run_id` (required): Run ID to rerun

---

## GitLab CI

### Authentication

Requires a GitLab Personal Access Token or OAuth2 token:

```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
```

Token scopes required:
- `api`: Full API access
- `read_api`: Read-only access (for monitoring agents)

### gitlab_pipeline_list

List pipelines for a project.

**Parameters:**
- `endpoint` (required): GitLab URL (e.g., `https://gitlab.com`)
- `project_id` (required): Project ID or path (e.g., `123` or `group/project`)
- `token` (required): GitLab token
- `scope`: Filter by scope (running, pending, finished, branches, tags)
- `status`: Filter by status (created, pending, running, success, failed, canceled)
- `ref`: Filter by branch/tag name
- `sha`: Filter by commit SHA
- `username`: Filter by user who triggered
- `updated_after`, `updated_before`: Time filters (ISO 8601)
- `order_by`: Order by field (id, status, ref, updated_at, user_id)
- `sort`: Sort direction (asc, desc)
- `per_page`, `page`: Pagination

**Example Agent:**

```yaml
name: gitlab-pipeline-monitor
spec:
  llm:
    provider: google
    model: gemini-2.5-flash
  tools:
    - gitlab_pipeline_list
    - gitlab_pipeline_get
    - gitlab_job_list
    - gitlab_job_log
  system_prompt: |
    Monitor GitLab CI/CD pipelines.

    Configuration:
    - Endpoint: https://gitlab.com
    - Project: group/my-project
    - Token: Use $GITLAB_TOKEN

    When checking pipelines:
    1. List recent pipelines
    2. Get details for failed pipelines
    3. Retrieve job logs for debugging
```

### gitlab_pipeline_get

Get detailed pipeline information.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `pipeline_id` (required): Pipeline ID

### gitlab_pipeline_create

Trigger a new pipeline.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `ref` (required): Branch, tag, or commit SHA
- `variables`: Array of CI/CD variables
  ```json
  [
    {"key": "DEPLOY_ENV", "value": "production"},
    {"key": "VERSION", "value": "1.2.3"}
  ]
  ```

**Example:**
```
Trigger a deployment to production with VERSION=2.5.0
```

### gitlab_pipeline_cancel

Cancel a running pipeline.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `pipeline_id` (required): Pipeline ID to cancel

### gitlab_pipeline_retry

Retry a failed pipeline.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `pipeline_id` (required): Pipeline ID to retry

### gitlab_job_list

List jobs in a pipeline.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `pipeline_id` (required): Pipeline ID
- `scope`: Filter by status (array of: created, pending, running, failed, success, canceled, skipped, manual)
- `include_retried`: Include retried jobs (default: false)
- `per_page`, `page`: Pagination

### gitlab_job_get

Get job details.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `job_id` (required): Job ID

### gitlab_job_log

Get job execution logs.

**Parameters:**
- `endpoint`, `project_id`, `token` (required)
- `job_id` (required): Job ID

---

## ArgoCD

### Authentication

Requires ArgoCD server URL and authentication token:

```bash
export ARGOCD_SERVER="https://argocd.example.com"
export ARGOCD_TOKEN="xxxxxxxxxxxxxxxxxxxx"
```

### argocd_app_list

List all ArgoCD applications.

**Parameters:**
- `server` (required): ArgoCD server URL
- `token` (required): ArgoCD auth token
- `project`: Filter by project
- `namespace`: Filter by namespace
- `selector`: Label selector (e.g., `app=myapp`)

**Example Agent:**

```yaml
name: argocd-manager
spec:
  llm:
    provider: google
    model: gemini-2.5-flash
  tools:
    - argocd_app_list
    - argocd_app_get
    - argocd_app_sync
    - argocd_app_history
  system_prompt: |
    Manage ArgoCD applications.

    Configuration:
    - Server: Use $ARGOCD_SERVER
    - Token: Use $ARGOCD_TOKEN

    When syncing applications:
    1. Check current sync status
    2. Review any drift or differences
    3. Trigger sync if needed
    4. Monitor sync progress
```

### argocd_app_get

Get application details including sync status and health.

**Parameters:**
- `server`, `token` (required)
- `name` (required): Application name
- `namespace`: Application namespace (default: argocd)

### argocd_app_sync

Trigger application sync.

**Parameters:**
- `server`, `token` (required)
- `name` (required): Application name
- `namespace`: Application namespace
- `revision`: Target revision (branch, tag, or commit)
- `prune`: Remove resources not in Git (default: false)
- `dry_run`: Preview changes without applying (default: false)
- `force`: Force sync even if already synced (default: false)

### argocd_app_refresh

Force application manifest refresh.

**Parameters:**
- `server`, `token` (required)
- `name` (required): Application name
- `namespace`: Application namespace
- `hard_refresh`: Hard refresh (re-clone repo) (default: false)

### argocd_app_history

Get application sync history.

**Parameters:**
- `server`, `token` (required)
- `name` (required): Application name
- `namespace`: Application namespace

### argocd_app_diff

Get difference between live state and desired state.

**Parameters:**
- `server`, `token` (required)
- `name` (required): Application name
- `namespace`: Application namespace
- `revision`: Compare against specific revision

---

## Flux

### Prerequisites

Requires `kubectl` access to a cluster with Flux installed.

### flux_kustomization_list

List Flux Kustomization resources.

**Parameters:**
- `namespace`: Filter by namespace (default: all namespaces)
- `kubeconfig`: Path to kubeconfig file
- `context`: Kubernetes context

**Example Agent:**

```yaml
name: flux-manager
spec:
  llm:
    provider: google
    model: gemini-2.5-flash
  tools:
    - flux_kustomization_list
    - flux_kustomization_get
    - flux_helmrelease_list
    - flux_reconcile
    - flux_suspend
    - flux_resume
  system_prompt: |
    Manage Flux GitOps resources.

    When troubleshooting:
    1. List kustomizations and helm releases
    2. Check reconciliation status
    3. Trigger reconciliation if needed
    4. Review controller logs for errors
```

### flux_kustomization_get

Get Kustomization details.

**Parameters:**
- `name` (required): Kustomization name
- `namespace` (required): Kustomization namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_helmrelease_list

List Flux HelmRelease resources.

**Parameters:**
- `namespace`: Filter by namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_helmrelease_get

Get HelmRelease details.

**Parameters:**
- `name` (required): HelmRelease name
- `namespace` (required): HelmRelease namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_reconcile

Trigger reconciliation of a Flux resource.

**Parameters:**
- `kind` (required): Resource kind (kustomization, helmrelease, source)
- `name` (required): Resource name
- `namespace` (required): Resource namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_suspend

Suspend a Flux resource.

**Parameters:**
- `kind` (required): Resource kind
- `name` (required): Resource name
- `namespace` (required): Resource namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_resume

Resume a suspended Flux resource.

**Parameters:**
- `kind` (required): Resource kind
- `name` (required): Resource name
- `namespace` (required): Resource namespace
- `kubeconfig`, `context`: Kubernetes connection

### flux_logs

Get Flux controller logs.

**Parameters:**
- `controller`: Controller name (source-controller, kustomize-controller, helm-controller)
- `namespace`: Controller namespace (default: flux-system)
- `lines`: Number of log lines (default: 100)
- `kubeconfig`, `context`: Kubernetes connection

---

## Example: Multi-Platform CI/CD Agent

```yaml
name: universal-cicd-agent
description: Monitor and manage CI/CD across multiple platforms

spec:
  llm:
    provider: google
    model: gemini-2.5-flash
    temperature: 0.2

  tools:
    # GitHub Actions
    - github_workflow_list
    - github_run_list
    - github_run_get
    - github_run_cancel
    - github_run_rerun
    # GitLab CI
    - gitlab_pipeline_list
    - gitlab_pipeline_get
    - gitlab_pipeline_create
    - gitlab_job_log
    # ArgoCD
    - argocd_app_list
    - argocd_app_sync
    - argocd_app_get

  memory:
    type: in_memory
    max_messages: 100

  system_prompt: |
    You are a universal CI/CD management agent capable of working with:
    - GitHub Actions
    - GitLab CI/CD
    - ArgoCD

    Available credentials (from environment):
    - GitHub: $GITHUB_TOKEN
    - GitLab: $GITLAB_TOKEN (endpoint: https://gitlab.com)
    - ArgoCD: $ARGOCD_TOKEN (server: $ARGOCD_SERVER)

    When managing CI/CD:
    1. Identify the platform from context
    2. Use appropriate tools for the platform
    3. Monitor pipeline/workflow status
    4. Report on failures with root cause analysis
    5. Trigger deployments when requested

    Always confirm before:
    - Canceling running pipelines
    - Triggering production deployments
    - Force syncing ArgoCD applications
```

## Best Practices

### Security

1. **Store tokens securely**: Use environment variables or secret managers
2. **Least privilege**: Use tokens with minimal required scopes
3. **Rotate regularly**: Refresh access tokens periodically
4. **Audit access**: Monitor agent actions in CI/CD platforms

### Rate Limiting

- **GitHub**: 5,000 requests/hour (authenticated)
- **GitLab**: 300 requests/minute
- **ArgoCD**: Configurable per-instance

Implement exponential backoff for rate limit errors.

### Error Handling

All tools return structured responses with error information:

```json
{
  "success": false,
  "error": "Authentication failed. Check token permissions."
}
```

Common error scenarios:
- `401`: Invalid or expired token
- `403`: Insufficient permissions
- `404`: Resource not found
- `409`: Conflict (e.g., pipeline state)
- `429`: Rate limited
