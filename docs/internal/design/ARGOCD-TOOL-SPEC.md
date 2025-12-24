# ArgoCD Tool Specification

## Overview

ArgoCD is a declarative GitOps continuous delivery tool for Kubernetes. The ArgoCD Tool provides programmatic access to ArgoCD's REST API for managing applications, synchronization, rollbacks, and monitoring deployment status.

### Key Capabilities

- Application management (list, get details, health status)
- Application synchronization (manual sync, refresh)
- Rollback to previous deployment revisions
- Sync history and deployment tracking
- Diff between desired (Git) and live (cluster) state
- Project-based filtering and access control

### API Reference

- **Official API Documentation**: https://argo-cd.readthedocs.io/en/stable/developer-guide/api-docs/
- **Swagger UI**: `https://<argocd-server>/swagger-ui`
- **Authentication**: JWT-based Bearer tokens
- **Base Endpoint**: `/api/v1/applications`

## Tool Operations

### 1. argocd_app_list

List all applications across all projects or within a specific project.

**Use Case**: Discovery, monitoring dashboards, CI/CD orchestration

**API Endpoint**: `GET /api/v1/applications`

**Parameters**:
- `endpoint` (required): ArgoCD server URL (e.g., `https://argocd.example.com`)
- `token` (required): JWT authentication token
- `project` (optional): Filter by project name
- `selector` (optional): Label selector (e.g., `app=nginx,env=prod`)
- `repo` (optional): Filter by repository URL

**Response**:
```json
{
  "applications": [
    {
      "name": "my-app",
      "project": "default",
      "namespace": "production",
      "server": "https://kubernetes.default.svc",
      "source": {
        "repoURL": "https://github.com/org/repo",
        "path": "manifests/app",
        "targetRevision": "main"
      },
      "status": {
        "sync": {
          "status": "Synced",
          "revision": "abc123def456"
        },
        "health": {
          "status": "Healthy"
        }
      }
    }
  ],
  "count": 1
}
```

### 2. argocd_app_get

Get detailed information about a specific application including health, sync status, resources, and conditions.

**Use Case**: Status checks, debugging, detailed analysis

**API Endpoint**: `GET /api/v1/applications/{name}`

**Parameters**:
- `endpoint` (required): ArgoCD server URL
- `token` (required): JWT authentication token
- `app_name` (required): Application name
- `project` (optional): Project name for validation
- `refresh` (optional): Refresh mode (`normal`, `hard`). Default: none

**Refresh Modes**:
- `normal`: Refresh application data from Kubernetes
- `hard`: Force refresh from Git repository

**Response**:
```json
{
  "metadata": {
    "name": "my-app",
    "namespace": "argocd",
    "creationTimestamp": "2024-01-15T10:30:00Z"
  },
  "spec": {
    "source": {
      "repoURL": "https://github.com/org/repo",
      "path": "manifests/app",
      "targetRevision": "main"
    },
    "destination": {
      "server": "https://kubernetes.default.svc",
      "namespace": "production"
    },
    "syncPolicy": {
      "automated": {
        "prune": true,
        "selfHeal": true
      }
    }
  },
  "status": {
    "sync": {
      "status": "Synced",
      "comparedTo": {
        "source": {
          "repoURL": "https://github.com/org/repo",
          "targetRevision": "abc123def456"
        }
      },
      "revision": "abc123def456"
    },
    "health": {
      "status": "Healthy"
    },
    "operationState": {
      "phase": "Succeeded",
      "finishedAt": "2024-01-15T10:35:00Z"
    },
    "resources": [
      {
        "kind": "Deployment",
        "name": "my-app",
        "namespace": "production",
        "status": "Synced",
        "health": {
          "status": "Healthy"
        }
      }
    ]
  }
}
```

### 3. argocd_app_sync

Trigger manual synchronization of an application to bring the live state in line with the desired state from Git.

**Use Case**: Manual deployments, CI/CD integration, emergency updates

**API Endpoint**: `POST /api/v1/applications/{name}/sync`

**Parameters**:
- `endpoint` (required): ArgoCD server URL
- `token` (required): JWT authentication token
- `app_name` (required): Application name
- `revision` (optional): Specific Git revision to sync (commit SHA, branch, tag). Default: HEAD
- `prune` (optional): Remove resources not in Git. Default: false
- `dry_run` (optional): Preview sync without applying changes. Default: false
- `resources` (optional): Array of specific resources to sync (selective sync)
- `sync_options` (optional): Array of sync options (e.g., `["Validate=false", "CreateNamespace=true"]`)
- `strategy` (optional): Sync strategy object with hook options

**Resource Selective Sync**:
```json
{
  "resources": [
    {
      "group": "apps",
      "kind": "Deployment",
      "name": "nginx",
      "namespace": "default"
    }
  ]
}
```

**Sync Options**:
- `Validate=false`: Skip kubectl validation
- `CreateNamespace=true`: Auto-create namespace
- `PrunePropagationPolicy=foreground`: Set deletion propagation
- `PruneLast=true`: Prune resources last

**Response**:
```json
{
  "operation": {
    "sync": {
      "revision": "abc123def456",
      "prune": false,
      "dryRun": false
    }
  },
  "status": "Running",
  "phase": "Running",
  "message": "Sync operation started",
  "startedAt": "2024-01-15T11:00:00Z"
}
```

**Important Notes**:
- Applications with automated sync enabled can still be manually synced
- Manual sync does not disable auto-sync policy
- Dry run mode returns preview without applying changes

### 4. argocd_app_rollback

Rollback application to a previous deployment revision.

**Use Case**: Disaster recovery, reverting failed deployments, emergency rollback

**API Endpoint**: `POST /api/v1/applications/{name}/rollback`

**Parameters**:
- `endpoint` (required): ArgoCD server URL
- `token` (required): JWT authentication token
- `app_name` (required): Application name
- `revision` (required): Deployment ID to rollback to (from sync history)
- `prune` (optional): Remove resources not in target revision. Default: false
- `dry_run` (optional): Preview rollback without applying. Default: false

**Response**:
```json
{
  "operation": {
    "sync": {
      "revision": "previous-commit-sha",
      "prune": false,
      "dryRun": false
    }
  },
  "status": "Running",
  "phase": "Running",
  "message": "Rollback to revision 5 started",
  "startedAt": "2024-01-15T11:30:00Z"
}
```

**Constraints**:
- **Automated sync must be disabled**: Rollback is not available for applications with `syncPolicy.automated` enabled
- ArgoCD retains last 10 deployment revisions by default (configurable)
- Rollback creates a new sync operation with the target revision

**Error Handling**:
- If auto-sync is enabled, return clear error: "Rollback unavailable: automated sync is enabled. Disable auto-sync first."

### 5. argocd_app_history

Retrieve synchronization history including all past deployments, revisions, and outcomes.

**Use Case**: Audit trails, deployment analytics, rollback candidate identification

**API Endpoint**: `GET /api/v1/applications/{name}`

**Parameters**:
- `endpoint` (required): ArgoCD server URL
- `token` (required): JWT authentication token
- `app_name` (required): Application name

**Response**:
```json
{
  "history": [
    {
      "id": 5,
      "revision": "abc123def456",
      "deployedAt": "2024-01-15T10:35:00Z",
      "source": {
        "repoURL": "https://github.com/org/repo",
        "path": "manifests/app",
        "targetRevision": "main"
      },
      "deployStartedAt": "2024-01-15T10:30:00Z"
    },
    {
      "id": 4,
      "revision": "previous-sha",
      "deployedAt": "2024-01-14T15:20:00Z",
      "source": {
        "repoURL": "https://github.com/org/repo",
        "path": "manifests/app",
        "targetRevision": "main"
      },
      "deployStartedAt": "2024-01-14T15:15:00Z"
    }
  ]
}
```

**Notes**:
- History is extracted from `status.history` array in application object
- Default retention is 10 revisions (configurable via `--app-resync` flag)
- Each entry includes deployment ID, Git revision, timestamps

### 6. argocd_app_diff

Show differences between desired state (Git) and live state (Kubernetes cluster).

**Use Case**: Pre-sync validation, change impact analysis, drift detection

**API Endpoint**: `GET /api/v1/applications/{name}/manifests`

**Parameters**:
- `endpoint` (required): ArgoCD server URL
- `token` (required): JWT authentication token
- `app_name` (required): Application name
- `revision` (optional): Compare against specific Git revision. Default: HEAD

**Response**:
```json
{
  "manifests": [
    {
      "diff": {
        "added": [
          {
            "kind": "ConfigMap",
            "name": "new-config",
            "namespace": "production"
          }
        ],
        "modified": [
          {
            "kind": "Deployment",
            "name": "my-app",
            "namespace": "production",
            "changes": [
              {
                "path": "spec.replicas",
                "before": "3",
                "after": "5"
              },
              {
                "path": "spec.template.spec.containers[0].image",
                "before": "nginx:1.19",
                "after": "nginx:1.21"
              }
            ]
          }
        ],
        "deleted": []
      }
    }
  ],
  "summary": {
    "added": 1,
    "modified": 1,
    "deleted": 0
  }
}
```

**Implementation Note**:
- ArgoCD API returns full manifests; diff calculation may be done client-side
- Use `/api/v1/applications/{name}/resource-tree` for live state
- Compare against `/api/v1/applications/{name}/manifests` for desired state

## Configuration

### Authentication Methods

ArgoCD exclusively uses **JWT-based authentication**. Tokens must be included in the `Authorization` header as Bearer tokens.

**Token Types**:

1. **Local Admin Token** (24-hour expiry):
```bash
# Obtain token via session endpoint
POST /api/v1/session
{
  "username": "admin",
  "password": "password"
}
# Returns: {"token": "eyJhbGc..."}
```

2. **Service Account Token** (configurable expiry):
```bash
# Generate via CLI
argocd account generate-token --account <account-name> --id <token-id>

# Or via API
POST /api/v1/projects/{project}/roles/{role}/token
```

3. **SSO/OIDC Token** (handled by identity provider):
- OAuth2 login flow through configured OIDC provider
- Expiration managed by IDP

### Tool Configuration Schema

```yaml
tools:
  - name: argocd
    type: http
    config:
      endpoint: "https://argocd.example.com"
      token: "${ARGOCD_TOKEN}"  # Environment variable
      # Optional: Default project filter
      default_project: "production"
      # Optional: TLS verification
      verify_tls: true
      # Optional: Request timeout
      timeout_secs: 30
```

### Environment Variables

- `ARGOCD_SERVER`: ArgoCD server URL
- `ARGOCD_TOKEN`: JWT authentication token
- `ARGOCD_INSECURE`: Skip TLS verification (not recommended)

## Implementation Details

### HTTP Client Setup

```rust
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

fn create_argocd_client(token: &str, verify_tls: bool) -> Result<Client, AofError> {
    let mut headers = HeaderMap::new();

    // Bearer token authentication
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value)
            .map_err(|e| AofError::tool(format!("Invalid token: {}", e)))?
    );

    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(!verify_tls)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### Authentication Handler

```rust
async fn authenticate(
    endpoint: &str,
    username: &str,
    password: &str
) -> AofResult<String> {
    let client = Client::new();
    let url = format!("{}/api/v1/session", endpoint.trim_end_matches('/'));

    let payload = serde_json::json!({
        "username": username,
        "password": password
    });

    let response = client.post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AofError::tool(format!("Authentication failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AofError::tool("Invalid credentials"));
    }

    let body: serde_json::Value = response.json().await?;
    let token = body.get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| AofError::tool("No token in response"))?;

    Ok(token.to_string())
}
```

### Response Parsing

```rust
#[derive(Debug, Deserialize)]
struct ArgoApplication {
    metadata: ApplicationMetadata,
    spec: ApplicationSpec,
    status: ApplicationStatus,
}

#[derive(Debug, Deserialize)]
struct ApplicationStatus {
    sync: SyncStatus,
    health: HealthStatus,
    #[serde(default)]
    history: Vec<RevisionHistory>,
    #[serde(default)]
    resources: Vec<ResourceStatus>,
}

#[derive(Debug, Deserialize)]
struct SyncStatus {
    status: String,  // "Synced", "OutOfSync", "Unknown"
    revision: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HealthStatus {
    status: String,  // "Healthy", "Progressing", "Degraded", "Suspended", "Missing", "Unknown"
}

#[derive(Debug, Deserialize)]
struct RevisionHistory {
    id: i64,
    revision: String,
    #[serde(rename = "deployedAt")]
    deployed_at: String,
    source: ApplicationSource,
}
```

### Error Handling Strategy

**HTTP Status Codes**:
- `200`: Success
- `401`: Authentication failure - invalid or expired token
- `403`: Authorization failure - insufficient permissions or project mismatch
- `404`: Application not found
- `409`: Conflict - another operation in progress
- `500`: Server error

**Error Response Format**:
```rust
fn handle_argocd_error(status: u16, body: &serde_json::Value) -> ToolResult {
    match status {
        401 => ToolResult::error("Authentication failed. Check token validity."),
        403 => {
            let msg = body.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Permission denied");
            ToolResult::error(format!("Authorization failed: {}", msg))
        }
        404 => ToolResult::error("Application not found. Check app name and project."),
        409 => ToolResult::error("Another operation is in progress. Wait and retry."),
        429 => ToolResult::error("Rate limited. Retry after cooldown period."),
        500..=599 => {
            let msg = body.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Server error");
            ToolResult::error(format!("ArgoCD server error: {}", msg))
        }
        _ => ToolResult::error(format!("Unexpected status {}: {:?}", status, body))
    }
}
```

**Retry Logic**:
- Retry on 5xx errors (max 3 attempts with exponential backoff)
- Do NOT retry on 4xx errors (client errors)
- Handle rate limiting (429) with backoff

**Timeout Handling**:
```rust
let response = tokio::time::timeout(
    Duration::from_secs(config.timeout_secs),
    client.get(&url).send()
)
.await
.map_err(|_| AofError::tool("Request timeout"))?
.map_err(|e| {
    if e.is_timeout() {
        AofError::tool("Request timeout")
    } else if e.is_connect() {
        AofError::tool(format!("Connection failed: {}. Check endpoint URL.", e))
    } else {
        AofError::tool(format!("Request failed: {}", e))
    }
})?;
```

## Tool Parameters Schema

### argocd_app_list

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL (e.g., https://argocd.example.com)"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "project": {
      "type": "string",
      "description": "Filter by project name"
    },
    "selector": {
      "type": "string",
      "description": "Label selector (e.g., 'app=nginx,env=prod')"
    },
    "repo": {
      "type": "string",
      "description": "Filter by repository URL"
    }
  },
  "required": ["endpoint", "token"]
}
```

### argocd_app_get

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "app_name": {
      "type": "string",
      "description": "Application name"
    },
    "project": {
      "type": "string",
      "description": "Project name for validation"
    },
    "refresh": {
      "type": "string",
      "enum": ["normal", "hard"],
      "description": "Refresh mode: 'normal' (from K8s) or 'hard' (from Git)"
    }
  },
  "required": ["endpoint", "token", "app_name"]
}
```

### argocd_app_sync

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "app_name": {
      "type": "string",
      "description": "Application name"
    },
    "revision": {
      "type": "string",
      "description": "Git revision to sync (commit SHA, branch, tag)"
    },
    "prune": {
      "type": "boolean",
      "description": "Remove resources not in Git",
      "default": false
    },
    "dry_run": {
      "type": "boolean",
      "description": "Preview sync without applying changes",
      "default": false
    },
    "resources": {
      "type": "array",
      "description": "Specific resources to sync (selective sync)",
      "items": {
        "type": "object",
        "properties": {
          "group": {"type": "string"},
          "kind": {"type": "string"},
          "name": {"type": "string"},
          "namespace": {"type": "string"}
        },
        "required": ["kind", "name"]
      }
    },
    "sync_options": {
      "type": "array",
      "description": "Sync options (e.g., ['Validate=false', 'CreateNamespace=true'])",
      "items": {
        "type": "string"
      }
    }
  },
  "required": ["endpoint", "token", "app_name"]
}
```

### argocd_app_rollback

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "app_name": {
      "type": "string",
      "description": "Application name"
    },
    "revision": {
      "type": "string",
      "description": "Deployment ID to rollback to (from sync history)"
    },
    "prune": {
      "type": "boolean",
      "description": "Remove resources not in target revision",
      "default": false
    },
    "dry_run": {
      "type": "boolean",
      "description": "Preview rollback without applying",
      "default": false
    }
  },
  "required": ["endpoint", "token", "app_name", "revision"]
}
```

### argocd_app_history

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "app_name": {
      "type": "string",
      "description": "Application name"
    }
  },
  "required": ["endpoint", "token", "app_name"]
}
```

### argocd_app_diff

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "ArgoCD server URL"
    },
    "token": {
      "type": "string",
      "description": "JWT authentication token"
    },
    "app_name": {
      "type": "string",
      "description": "Application name"
    },
    "revision": {
      "type": "string",
      "description": "Git revision to compare against"
    }
  },
  "required": ["endpoint", "token", "app_name"]
}
```

## Error Handling

### Common Error Scenarios

**1. Authentication Errors (401)**
```rust
if status == 401 {
    return Ok(ToolResult::error(
        "Authentication failed. Token may be invalid or expired. \
         Generate a new token using: argocd account generate-token"
    ));
}
```

**2. Authorization Errors (403)**
```rust
if status == 403 {
    return Ok(ToolResult::error(
        format!(
            "Authorization failed: {}. \
             Check that the token has permissions for project '{}' and application '{}'.",
            error_msg, project, app_name
        )
    ));
}
```

**3. Application Not Found (404)**
```rust
if status == 404 {
    return Ok(ToolResult::error(
        format!(
            "Application '{}' not found. Verify the application name and project. \
             List available apps: argocd app list",
            app_name
        )
    ));
}
```

**4. Operation Conflict (409)**
```rust
if status == 409 {
    return Ok(ToolResult::error(
        "Another operation is in progress for this application. \
         Wait for the current operation to complete and retry."
    ));
}
```

**5. Auto-Sync Rollback Constraint**
```rust
// Check before attempting rollback
if app.spec.sync_policy.automated.is_some() {
    return Ok(ToolResult::error(
        "Rollback unavailable: automated sync is enabled. \
         Disable auto-sync first using: argocd app set <app> --sync-policy none"
    ));
}
```

**6. Network Errors**
```rust
.map_err(|e| {
    if e.is_timeout() {
        AofError::tool("Request timeout. ArgoCD server may be slow or unreachable.")
    } else if e.is_connect() {
        AofError::tool(format!(
            "Connection failed: {}. \
             Verify endpoint URL and network connectivity.",
            e
        ))
    } else {
        AofError::tool(format!("Request failed: {}", e))
    }
})
```

### Validation Errors

**Pre-execution validation**:
```rust
fn validate_input(&self, input: &ToolInput) -> AofResult<()> {
    let endpoint: String = input.get_arg("endpoint")?;

    // Validate URL format
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(AofError::tool(
            "Endpoint must be a valid URL (http:// or https://)"
        ));
    }

    let token: String = input.get_arg("token")?;

    // Validate token format (basic JWT check)
    if !token.contains('.') || token.len() < 50 {
        return Err(AofError::tool(
            "Token appears invalid. JWT tokens should be in format: header.payload.signature"
        ));
    }

    Ok(())
}
```

## Example Usage in Agent YAML

### Basic Application Monitoring Agent

```yaml
name: argocd-monitor
description: Monitor ArgoCD application health and sync status

spec:
  llm:
    provider: google
    model: google:gemini-2.5-flash

  tools:
    - name: argocd_app_list
    - name: argocd_app_get
    - name: argocd_app_history

  system_prompt: |
    You are an ArgoCD monitoring agent. Monitor application health and sync status.

    Workflow:
    1. List all applications in the production project
    2. Check health status of each application
    3. Report any applications that are:
       - OutOfSync
       - Degraded or Unhealthy
       - Have failed sync operations
    4. Provide deployment history for problematic apps

    For each issue found, provide:
    - Application name and project
    - Current sync status and health
    - Last successful deployment time
    - Recent sync history (last 3 deployments)

  triggers:
    - type: cron
      schedule: "*/5 * * * *"  # Every 5 minutes
      config:
        endpoint: "${ARGOCD_SERVER}"
        token: "${ARGOCD_TOKEN}"
        project: "production"
```

### Deployment Agent with Approval

```yaml
name: argocd-deployer
description: Deploy applications with approval workflow

spec:
  llm:
    provider: google
    model: google:gemini-2.5-flash

  tools:
    - name: argocd_app_get
    - name: argocd_app_diff
    - name: argocd_app_sync

  system_prompt: |
    You are an ArgoCD deployment agent with approval workflow.

    Before deployment:
    1. Get current application status
    2. Show diff between current and desired state
    3. **WAIT FOR USER APPROVAL** before proceeding
    4. If approved, trigger sync operation
    5. Monitor sync progress
    6. Report final deployment status

    CRITICAL: Never deploy without explicit user approval.
    Present clear diff and ask: "Approve deployment? (yes/no)"

  memory:
    - type: conversation
      max_messages: 50
```

### Rollback Agent

```yaml
name: argocd-rollback
description: Emergency rollback to previous deployment

spec:
  llm:
    provider: google
    model: google:gemini-2.5-flash

  tools:
    - name: argocd_app_get
    - name: argocd_app_history
    - name: argocd_app_rollback

  system_prompt: |
    You are an emergency rollback agent for ArgoCD applications.

    Rollback procedure:
    1. Get current application status
    2. Retrieve deployment history
    3. Identify the last healthy deployment
    4. Verify auto-sync is disabled (required for rollback)
    5. Execute rollback to previous revision
    6. Monitor rollback operation
    7. Verify application health after rollback

    SAFETY CHECKS:
    - Confirm auto-sync is disabled before rollback
    - Validate target revision exists in history
    - Warn if rollback target is >3 versions old

    Report:
    - Previous revision details
    - Rollback operation status
    - Post-rollback health check

  triggers:
    - type: webhook
      config:
        endpoint: "/rollback"
        method: POST
```

### Sync Status Reporter

```yaml
name: argocd-reporter
description: Generate ArgoCD deployment reports

spec:
  llm:
    provider: google
    model: google:gemini-2.5-flash

  tools:
    - name: argocd_app_list
    - name: argocd_app_get
    - name: argocd_app_history

  system_prompt: |
    Generate comprehensive ArgoCD deployment reports.

    Report sections:
    1. **Overview**
       - Total applications
       - Sync status breakdown (Synced/OutOfSync)
       - Health status breakdown (Healthy/Degraded/Progressing)

    2. **Per-Application Details**
       - Application name and project
       - Current Git revision
       - Sync status and health
       - Last deployment time
       - Auto-sync enabled/disabled

    3. **Recent Activity**
       - Deployments in last 24 hours
       - Failed syncs (if any)
       - Rollback operations

    4. **Alerts**
       - Applications requiring attention
       - Long-running sync operations (>10 minutes)
       - Health degradation

    Format: Markdown with tables and emoji indicators (✅ Healthy, ⚠️ Warning, ❌ Error)

  triggers:
    - type: cron
      schedule: "0 9 * * *"  # Daily at 9 AM
      config:
        endpoint: "${ARGOCD_SERVER}"
        token: "${ARGOCD_TOKEN}"
```

### Multi-Project CI/CD Integration

```yaml
name: argocd-cicd
description: CI/CD integration for multi-project deployments

spec:
  llm:
    provider: google
    model: google:gemini-2.5-flash

  tools:
    - name: argocd_app_list
    - name: argocd_app_get
    - name: argocd_app_sync
    - name: argocd_app_diff

  system_prompt: |
    You are a CI/CD orchestration agent for ArgoCD.

    Deployment pipeline:
    1. Receive deployment request with application list
    2. For each application:
       a. Verify application exists in correct project
       b. Show diff for changes
       c. Trigger sync to specific Git revision
       d. Wait for sync completion
       e. Verify health status
       f. Move to next application or abort on failure

    3. Report overall deployment status

    Rollback on failure:
    - If any deployment fails health check, rollback all deployed apps
    - Provide detailed failure analysis

    Configuration:
    - Sequential deployment (one at a time)
    - 5-minute timeout per application
    - Automatic health validation

  triggers:
    - type: webhook
      config:
        endpoint: "/deploy"
        method: POST
        payload_schema:
          applications:
            - name: "app1"
              project: "production"
              revision: "v1.2.3"
            - name: "app2"
              project: "production"
              revision: "v1.2.3"
```

## Testing Checklist

- [ ] Authentication with valid token succeeds
- [ ] Authentication with invalid token returns 401
- [ ] Authentication with expired token returns 401
- [ ] List applications without project filter
- [ ] List applications with project filter
- [ ] Get application details for existing app
- [ ] Get application returns 404 for non-existent app
- [ ] Get application with refresh=normal
- [ ] Get application with refresh=hard
- [ ] Sync application with default options
- [ ] Sync application with specific revision
- [ ] Sync application with prune enabled
- [ ] Sync application with dry_run
- [ ] Sync application with selective resources
- [ ] Sync application with sync_options
- [ ] Sync returns 409 when operation in progress
- [ ] Rollback with auto-sync disabled succeeds
- [ ] Rollback with auto-sync enabled returns error
- [ ] Rollback to valid revision
- [ ] Rollback with dry_run
- [ ] History retrieval for application with multiple deployments
- [ ] History handles empty history gracefully
- [ ] Diff calculation between states
- [ ] Diff with specific revision
- [ ] Connection timeout handling
- [ ] Request timeout handling
- [ ] TLS verification (enabled/disabled)
- [ ] Rate limiting (429) handling
- [ ] Server error (5xx) handling with retry
- [ ] Large response handling (many applications)
- [ ] Concurrent operation handling

## References

- [ArgoCD API Documentation](https://argo-cd.readthedocs.io/en/stable/developer-guide/api-docs/)
- [ArgoCD Swagger UI](https://cd.apps.argoproj.io/swagger-ui)
- [Authentication and Authorization](https://argo-cd.readthedocs.io/en/stable/developer-guide/architecture/authz-authn/)
- [Automated Sync Policy](https://argo-cd.readthedocs.io/en/stable/user-guide/auto_sync/)
- [Sync Options](https://argo-cd.readthedocs.io/en/stable/user-guide/sync-options/)
- [Security Overview](https://argo-cd.readthedocs.io/en/stable/operator-manual/security/)
- [Creating Service Accounts](https://www.arthurkoziel.com/creating-argo-cd-service-account/)

## Notes

### Design Decisions

1. **JWT-Only Authentication**: Following ArgoCD's security model, only JWT Bearer tokens are supported. Username/password is only used to obtain initial token.

2. **Project-Based Filtering**: All operations support optional project parameter for multi-tenancy and access control.

3. **Auto-Sync Constraint**: Rollback operations explicitly check and fail if automated sync is enabled, providing clear error message to users.

4. **Diff Implementation**: Diff operations may require client-side calculation by comparing manifests API response with resource-tree API response.

5. **History Retention**: Default 10-revision history matches ArgoCD defaults but can be configured server-side.

6. **Timeout Strategy**: 30-second default timeout is suitable for most operations; sync/rollback may need longer timeouts for large applications.

7. **Error Messages**: All errors include actionable guidance (e.g., CLI commands to fix issues) following ArgoCD best practices.

### Future Enhancements

- Application creation/deletion (requires additional API endpoints)
- Project management operations
- Resource-level operations (get pod logs, restart deployment)
- Webhook configuration
- Multi-source application support
- Application set integration
- GitOps notification integration
- Metrics and observability integration

---

**Specification Version**: 1.0
**Last Updated**: 2025-12-23
**Status**: Draft - Ready for Implementation
