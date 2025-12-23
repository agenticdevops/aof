# GitLab CI Tool - Internal Design Specification

## 1. Overview

### 1.1 Purpose

The GitLab CI Tool provides programmatic access to GitLab's built-in CI/CD system, enabling agents to:
- Monitor pipeline execution status
- Trigger new pipeline runs
- Cancel or retry pipelines
- Retrieve job logs and details
- Automate CI/CD workflows

### 1.2 GitLab CI/CD Capabilities

GitLab CI/CD is an integrated continuous integration and deployment system that:
- Executes pipelines defined in `.gitlab-ci.yml` files
- Organizes work into stages and jobs
- Supports manual, scheduled, and trigger-based pipeline runs
- Provides detailed job logs and artifacts
- Integrates with GitLab merge requests and issues

### 1.3 API Foundation

The tool uses GitLab's REST API v4:
- Base URL: `https://gitlab.com/api/v4` (GitLab.com) or `https://gitlab.example.com/api/v4` (self-hosted)
- Authentication: Private Token or OAuth2 Token
- Rate Limiting: 300 requests per minute (GitLab.com), configurable for self-hosted
- Pagination: Link header-based pagination for list operations

### 1.4 Feature Flag

- **Feature**: `cicd`
- **Rationale**: Groups CI/CD integration tools together
- **Dependencies**: None (standalone HTTP client)

---

## 2. Tool Operations

### 2.1 gitlab_pipeline_list

**Purpose**: List pipelines for a project with optional filtering.

**GitLab API Endpoint**: `GET /projects/:id/pipelines`

**Use Cases**:
- Monitor recent pipeline activity
- Find pipelines by status or ref
- Audit pipeline history
- Dashboard integration

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx",
  "scope": "finished",
  "status": "success",
  "ref": "main",
  "sha": "a1b2c3d4",
  "username": "deploy-bot",
  "updated_after": "2024-01-01T00:00:00Z",
  "updated_before": "2024-12-31T23:59:59Z",
  "order_by": "id",
  "sort": "desc",
  "per_page": 20,
  "page": 1
}
```

**Query Parameters**:
- `scope`: Filter by scope (running, pending, finished, branches, tags)
- `status`: Filter by status (created, waiting_for_resource, preparing, pending, running, success, failed, canceled, skipped, manual, scheduled)
- `ref`: Filter by branch/tag name
- `sha`: Filter by specific commit SHA
- `username`: Filter by user who triggered pipeline
- `updated_after`: Return pipelines updated after this time
- `updated_before`: Return pipelines updated before this time
- `order_by`: Order by id, status, ref, updated_at, user_id (default: id)
- `sort`: Sort direction (asc, desc)
- `per_page`: Results per page (max 100)
- `page`: Page number

**Response**:
```json
{
  "pipelines": [
    {
      "id": 1001,
      "iid": 15,
      "project_id": 123,
      "status": "success",
      "ref": "main",
      "sha": "a1b2c3d4e5f6",
      "before_sha": "b2c3d4e5f6a1",
      "tag": false,
      "yaml_errors": null,
      "user": {
        "id": 42,
        "username": "deploy-bot",
        "name": "Deploy Bot"
      },
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:45:00Z",
      "started_at": "2024-01-15T10:31:00Z",
      "finished_at": "2024-01-15T10:45:00Z",
      "committed_at": "2024-01-15T10:29:00Z",
      "duration": 840,
      "queued_duration": 15,
      "coverage": "95.2",
      "web_url": "https://gitlab.example.com/project/pipelines/1001"
    }
  ],
  "count": 1,
  "total_duration_seconds": 840
}
```

---

### 2.2 gitlab_pipeline_get

**Purpose**: Get detailed information about a specific pipeline.

**GitLab API Endpoint**: `GET /projects/:id/pipelines/:pipeline_id`

**Use Cases**:
- Check pipeline execution status
- Monitor pipeline progress
- Retrieve detailed pipeline metadata
- Integration with monitoring systems

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "pipeline_id": "1001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx"
}
```

**Response**:
```json
{
  "id": 1001,
  "iid": 15,
  "project_id": 123,
  "status": "running",
  "ref": "main",
  "sha": "a1b2c3d4e5f6",
  "before_sha": "b2c3d4e5f6a1",
  "tag": false,
  "yaml_errors": null,
  "user": {
    "id": 42,
    "username": "deploy-bot",
    "name": "Deploy Bot",
    "avatar_url": "https://gitlab.example.com/uploads/user/avatar/42/avatar.png"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:35:00Z",
  "started_at": "2024-01-15T10:31:00Z",
  "finished_at": null,
  "committed_at": "2024-01-15T10:29:00Z",
  "duration": null,
  "queued_duration": 15,
  "coverage": null,
  "web_url": "https://gitlab.example.com/project/pipelines/1001",
  "detailed_status": {
    "icon": "status_running",
    "text": "running",
    "label": "running",
    "group": "running"
  }
}
```

---

### 2.3 gitlab_pipeline_create

**Purpose**: Trigger a new pipeline for a project.

**GitLab API Endpoint**: `POST /projects/:id/pipeline`

**Use Cases**:
- Automated deployment triggers
- Manual pipeline execution from agents
- Integration testing workflows
- Custom CI/CD automation

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx",
  "ref": "main",
  "variables": [
    {
      "key": "DEPLOY_ENV",
      "value": "production",
      "variable_type": "env_var"
    },
    {
      "key": "VERSION",
      "value": "1.2.3",
      "variable_type": "env_var"
    }
  ]
}
```

**Request Body**:
- `ref`: Branch, tag, or commit SHA to run pipeline for (required)
- `variables`: Array of CI/CD variables to pass to pipeline
  - `key`: Variable name
  - `value`: Variable value
  - `variable_type`: Type of variable (env_var or file)

**Response**:
```json
{
  "pipeline_id": 1002,
  "status": "created",
  "ref": "main",
  "sha": "a1b2c3d4e5f6",
  "web_url": "https://gitlab.example.com/project/pipelines/1002",
  "created_at": "2024-01-15T11:00:00Z"
}
```

---

### 2.4 gitlab_pipeline_cancel

**Purpose**: Cancel a running or pending pipeline.

**GitLab API Endpoint**: `POST /projects/:id/pipelines/:pipeline_id/cancel`

**Use Cases**:
- Stop failed deployments
- Cancel outdated pipeline runs
- Emergency pipeline termination
- Resource management

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "pipeline_id": "1001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx"
}
```

**Response**:
```json
{
  "pipeline_id": 1001,
  "status": "canceled",
  "ref": "main",
  "sha": "a1b2c3d4e5f6",
  "web_url": "https://gitlab.example.com/project/pipelines/1001",
  "canceled_at": "2024-01-15T10:40:00Z"
}
```

---

### 2.5 gitlab_pipeline_retry

**Purpose**: Retry a failed pipeline.

**GitLab API Endpoint**: `POST /projects/:id/pipelines/:pipeline_id/retry`

**Use Cases**:
- Retry transient failures
- Resume after infrastructure issues
- Automatic retry logic in agents
- Recovery workflows

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "pipeline_id": "1001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx"
}
```

**Response**:
```json
{
  "pipeline_id": 1003,
  "status": "pending",
  "ref": "main",
  "sha": "a1b2c3d4e5f6",
  "web_url": "https://gitlab.example.com/project/pipelines/1003",
  "created_at": "2024-01-15T10:50:00Z",
  "retried_from": 1001
}
```

---

### 2.6 gitlab_job_list

**Purpose**: List jobs in a pipeline.

**GitLab API Endpoint**: `GET /projects/:id/pipelines/:pipeline_id/jobs`

**Use Cases**:
- Monitor individual job status
- Find failed jobs for debugging
- Track job execution order
- Performance analysis

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "pipeline_id": "1001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx",
  "scope": ["failed", "success"],
  "include_retried": false,
  "per_page": 20,
  "page": 1
}
```

**Query Parameters**:
- `scope`: Filter by status (array of created, pending, running, failed, success, canceled, skipped, manual)
- `include_retried`: Include retried jobs (default: false)
- `per_page`: Results per page (max 100)
- `page`: Page number

**Response**:
```json
{
  "jobs": [
    {
      "id": 5001,
      "status": "success",
      "stage": "build",
      "name": "compile",
      "ref": "main",
      "tag": false,
      "coverage": "98.5",
      "created_at": "2024-01-15T10:31:00Z",
      "started_at": "2024-01-15T10:32:00Z",
      "finished_at": "2024-01-15T10:37:00Z",
      "duration": 300,
      "queued_duration": 15,
      "user": {
        "id": 42,
        "username": "deploy-bot"
      },
      "commit": {
        "id": "a1b2c3d4e5f6",
        "short_id": "a1b2c3d4",
        "title": "Add new feature"
      },
      "pipeline": {
        "id": 1001,
        "ref": "main",
        "sha": "a1b2c3d4e5f6",
        "status": "running"
      },
      "web_url": "https://gitlab.example.com/project/-/jobs/5001",
      "artifacts": [
        {
          "file_type": "archive",
          "size": 1024000,
          "filename": "artifacts.zip"
        }
      ]
    }
  ],
  "count": 1
}
```

---

### 2.7 gitlab_job_get

**Purpose**: Get detailed information about a specific job.

**GitLab API Endpoint**: `GET /projects/:id/jobs/:job_id`

**Use Cases**:
- Check job execution details
- Monitor job progress
- Retrieve job artifacts information
- Debugging job failures

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "job_id": "5001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx"
}
```

**Response**:
```json
{
  "id": 5001,
  "status": "success",
  "stage": "build",
  "name": "compile",
  "ref": "main",
  "tag": false,
  "coverage": "98.5",
  "allow_failure": false,
  "created_at": "2024-01-15T10:31:00Z",
  "started_at": "2024-01-15T10:32:00Z",
  "finished_at": "2024-01-15T10:37:00Z",
  "duration": 300,
  "queued_duration": 15,
  "user": {
    "id": 42,
    "username": "deploy-bot",
    "name": "Deploy Bot"
  },
  "commit": {
    "id": "a1b2c3d4e5f6",
    "short_id": "a1b2c3d4",
    "title": "Add new feature",
    "author_name": "John Doe",
    "author_email": "john@example.com",
    "created_at": "2024-01-15T10:29:00Z"
  },
  "pipeline": {
    "id": 1001,
    "ref": "main",
    "sha": "a1b2c3d4e5f6",
    "status": "running"
  },
  "web_url": "https://gitlab.example.com/project/-/jobs/5001",
  "artifacts": [
    {
      "file_type": "archive",
      "size": 1024000,
      "filename": "artifacts.zip"
    }
  ],
  "runner": {
    "id": 10,
    "description": "Production Runner",
    "active": true,
    "is_shared": false
  }
}
```

---

### 2.8 gitlab_job_log

**Purpose**: Retrieve job execution logs.

**GitLab API Endpoint**: `GET /projects/:id/jobs/:job_id/trace`

**Use Cases**:
- Debug job failures
- Monitor job output
- Log aggregation
- Automated error detection

**Parameters**:
```json
{
  "endpoint": "https://gitlab.example.com",
  "project_id": "123",
  "job_id": "5001",
  "token": "glpat-xxxxxxxxxxxxxxxxxxxx"
}
```

**Response**:
```json
{
  "job_id": 5001,
  "job_name": "compile",
  "status": "success",
  "log": "Running with gitlab-runner 16.5.0\n  on production-runner xyz123\nPreparing the \"docker\" executor\n  Using Docker executor with image node:18 ...\nPulling docker image node:18 ...\nUsing docker image sha256:abc123... for node:18 with digest node@sha256:def456...\nPreparing environment\n  Running on runner-xyz123-project-123-concurrent-0 via runner-host...\nGetting source from Git repository\n  Fetching changes with git depth set to 20...\n  Initialized empty Git repository in /builds/project/.git/\n  Created fresh repository.\n  Checking out a1b2c3d4 as main...\n  Skipping Git submodules setup\nExecuting \"step_script\" stage of the job script\n$ npm ci\nadded 245 packages in 15s\n$ npm run build\n> project@1.0.0 build\n> tsc\nBuild completed successfully\nJob succeeded",
  "log_lines": 18,
  "truncated": false
}
```

---

## 3. Configuration

### 3.1 GitLab Connection

**Required Configuration**:
```yaml
cicd:
  type: gitlab
  endpoint: https://gitlab.example.com
  project_id: "123"
  token: glpat-xxxxxxxxxxxxxxxxxxxx
```

**Configuration Fields**:
- `endpoint`: GitLab instance URL (GitLab.com or self-hosted)
- `project_id`: Project ID or path (e.g., "123" or "group/project")
- `token`: Private token or OAuth2 token

### 3.2 Authentication Methods

**Private Token (Recommended)**:
- Create in GitLab: User Settings → Access Tokens
- Scopes required: `api`, `read_api`, `write_repository` (for pipeline creation)
- Set expiration date
- Store securely in environment variables

**OAuth2 Token**:
- Use GitLab OAuth2 application
- Scopes: `api`, `read_api`, `write_repository`
- Refresh token handling required

**Project Access Token**:
- Create in Project Settings → Access Tokens
- Scopes: `api`, `read_api`, `write_repository`
- Limited to specific project

### 3.3 Project Identification

**By Project ID**:
```yaml
project_id: "123"
```

**By Project Path**:
```yaml
project_id: "group/subgroup/project"
```

**URL Encoding**:
- Path must be URL-encoded: `group%2Fsubgroup%2Fproject`
- Tool handles encoding automatically

### 3.4 Rate Limiting

**GitLab.com**:
- 300 requests per minute per user
- 300 requests per minute per project

**Self-Hosted**:
- Configurable in admin settings
- Default: no limit (can be enabled)

**Rate Limit Headers**:
- `RateLimit-Limit`: Maximum requests per minute
- `RateLimit-Remaining`: Remaining requests
- `RateLimit-Reset`: Unix timestamp of reset

---

## 4. Implementation Details

### 4.1 HTTP Client Setup

```rust
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};

fn create_gitlab_client(token: &str) -> Result<Client, AofError> {
    let mut headers = HeaderMap::new();

    // Private-Token authentication
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value)
            .map_err(|e| AofError::tool(format!("Invalid token: {}", e)))?
    );

    // Content type for JSON requests
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/json")
    );

    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.2 Authentication Header

**Private Token**:
```rust
// Legacy format (still supported)
headers.insert("PRIVATE-TOKEN", HeaderValue::from_str(token)?);

// OAuth2/Bearer format (recommended)
headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);
```

### 4.3 Pagination Handling

GitLab uses Link header-based pagination:

```rust
fn parse_pagination_links(headers: &HeaderMap) -> Option<PaginationInfo> {
    let link_header = headers.get("Link")?.to_str().ok()?;

    let mut next_page = None;
    let mut total_pages = None;
    let mut total_count = None;

    // Parse Link header: <url>; rel="next", <url>; rel="last"
    for link in link_header.split(',') {
        if link.contains("rel=\"next\"") {
            // Extract page number from URL
            next_page = extract_page_number(link);
        }
        if link.contains("rel=\"last\"") {
            total_pages = extract_page_number(link);
        }
    }

    // Total count from X-Total header
    if let Some(total) = headers.get("X-Total") {
        total_count = total.to_str().ok()?.parse().ok();
    }

    Some(PaginationInfo {
        next_page,
        total_pages,
        total_count,
    })
}

struct PaginationInfo {
    next_page: Option<u32>,
    total_pages: Option<u32>,
    total_count: Option<u64>,
}
```

**Response Headers**:
- `Link`: Pagination links (next, prev, first, last)
- `X-Total`: Total number of items
- `X-Total-Pages`: Total number of pages
- `X-Per-Page`: Items per page
- `X-Page`: Current page number
- `X-Next-Page`: Next page number
- `X-Prev-Page`: Previous page number

### 4.4 Response Parsing

```rust
async fn execute_gitlab_request<T: DeserializeOwned>(
    client: &Client,
    url: &str,
) -> AofResult<T> {
    let response = client.get(url).send().await
        .map_err(|e| {
            if e.is_timeout() {
                AofError::tool("Request timeout".to_string())
            } else if e.is_connect() {
                AofError::tool(format!("Connection failed: {}. Check endpoint URL.", e))
            } else {
                AofError::tool(format!("GitLab request failed: {}", e))
            }
        })?;

    let status = response.status().as_u16();

    // Handle status codes
    match status {
        200 | 201 => {
            response.json::<T>().await
                .map_err(|e| AofError::tool(format!("Failed to parse response: {}", e)))
        }
        401 => Err(AofError::tool(
            "Authentication failed. Check token and permissions.".to_string()
        )),
        403 => Err(AofError::tool(
            "Access forbidden. Check token scopes and project permissions.".to_string()
        )),
        404 => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            Err(AofError::tool(format!(
                "Resource not found: {}",
                body.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
            )))
        }
        409 => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            Err(AofError::tool(format!(
                "Conflict: {}",
                body.get("message").and_then(|m| m.as_str()).unwrap_or("Resource conflict")
            )))
        }
        429 => {
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("60");
            Err(AofError::tool(format!(
                "Rate limited. Retry after {} seconds",
                retry_after
            )))
        }
        500..=599 => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            Err(AofError::tool(format!(
                "GitLab server error ({}): {:?}",
                status,
                body.get("message")
            )))
        }
        _ => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            Err(AofError::tool(format!(
                "GitLab returned status {}: {:?}",
                status,
                body.get("message")
            )))
        }
    }
}
```

### 4.5 Project ID URL Encoding

```rust
fn encode_project_id(project_id: &str) -> String {
    // URL encode project path (e.g., "group/project" -> "group%2Fproject")
    project_id.replace("/", "%2F")
}

fn build_project_url(endpoint: &str, project_id: &str, path: &str) -> String {
    let encoded_id = encode_project_id(project_id);
    format!(
        "{}/api/v4/projects/{}/{}",
        endpoint.trim_end_matches('/'),
        encoded_id,
        path.trim_start_matches('/')
    )
}
```

### 4.6 Variable Handling

```rust
#[derive(Debug, Serialize)]
struct PipelineVariable {
    key: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variable_type: Option<String>, // "env_var" or "file"
}

fn create_pipeline_variables(
    vars: Vec<serde_json::Value>
) -> Result<Vec<PipelineVariable>, AofError> {
    vars.into_iter()
        .map(|v| {
            let key = v.get("key")
                .and_then(|k| k.as_str())
                .ok_or_else(|| AofError::tool("Variable missing 'key'".to_string()))?
                .to_string();

            let value = v.get("value")
                .and_then(|val| val.as_str())
                .ok_or_else(|| AofError::tool("Variable missing 'value'".to_string()))?
                .to_string();

            let variable_type = v.get("variable_type")
                .and_then(|t| t.as_str())
                .map(String::from);

            Ok(PipelineVariable {
                key,
                value,
                variable_type,
            })
        })
        .collect()
}
```

---

## 5. Tool Parameters Schema

### 5.1 gitlab_pipeline_list

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL (e.g., https://gitlab.com or https://gitlab.example.com)"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path (e.g., '123' or 'group/project')"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    },
    "scope": {
      "type": "string",
      "description": "Filter by scope",
      "enum": ["running", "pending", "finished", "branches", "tags"]
    },
    "status": {
      "type": "string",
      "description": "Filter by status",
      "enum": ["created", "waiting_for_resource", "preparing", "pending", "running", "success", "failed", "canceled", "skipped", "manual", "scheduled"]
    },
    "ref": {
      "type": "string",
      "description": "Filter by branch or tag name"
    },
    "sha": {
      "type": "string",
      "description": "Filter by commit SHA"
    },
    "username": {
      "type": "string",
      "description": "Filter by user who triggered pipeline"
    },
    "updated_after": {
      "type": "string",
      "description": "Return pipelines updated after this time (ISO 8601 format)"
    },
    "updated_before": {
      "type": "string",
      "description": "Return pipelines updated before this time (ISO 8601 format)"
    },
    "order_by": {
      "type": "string",
      "description": "Order results by field",
      "enum": ["id", "status", "ref", "updated_at", "user_id"],
      "default": "id"
    },
    "sort": {
      "type": "string",
      "description": "Sort direction",
      "enum": ["asc", "desc"],
      "default": "desc"
    },
    "per_page": {
      "type": "integer",
      "description": "Results per page (max 100)",
      "default": 20,
      "minimum": 1,
      "maximum": 100
    },
    "page": {
      "type": "integer",
      "description": "Page number",
      "default": 1,
      "minimum": 1
    }
  },
  "required": ["endpoint", "project_id", "token"]
}
```

### 5.2 gitlab_pipeline_get

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "pipeline_id": {
      "type": "string",
      "description": "Pipeline ID"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    }
  },
  "required": ["endpoint", "project_id", "pipeline_id", "token"]
}
```

### 5.3 gitlab_pipeline_create

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    },
    "ref": {
      "type": "string",
      "description": "Branch, tag, or commit SHA to run pipeline for"
    },
    "variables": {
      "type": "array",
      "description": "CI/CD variables to pass to pipeline",
      "items": {
        "type": "object",
        "properties": {
          "key": {
            "type": "string",
            "description": "Variable name"
          },
          "value": {
            "type": "string",
            "description": "Variable value"
          },
          "variable_type": {
            "type": "string",
            "description": "Variable type",
            "enum": ["env_var", "file"],
            "default": "env_var"
          }
        },
        "required": ["key", "value"]
      }
    }
  },
  "required": ["endpoint", "project_id", "token", "ref"]
}
```

### 5.4 gitlab_pipeline_cancel

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "pipeline_id": {
      "type": "string",
      "description": "Pipeline ID to cancel"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    }
  },
  "required": ["endpoint", "project_id", "pipeline_id", "token"]
}
```

### 5.5 gitlab_pipeline_retry

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "pipeline_id": {
      "type": "string",
      "description": "Pipeline ID to retry"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    }
  },
  "required": ["endpoint", "project_id", "pipeline_id", "token"]
}
```

### 5.6 gitlab_job_list

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "pipeline_id": {
      "type": "string",
      "description": "Pipeline ID"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    },
    "scope": {
      "type": "array",
      "description": "Filter by job status",
      "items": {
        "type": "string",
        "enum": ["created", "pending", "running", "failed", "success", "canceled", "skipped", "manual"]
      }
    },
    "include_retried": {
      "type": "boolean",
      "description": "Include retried jobs",
      "default": false
    },
    "per_page": {
      "type": "integer",
      "description": "Results per page (max 100)",
      "default": 20,
      "minimum": 1,
      "maximum": 100
    },
    "page": {
      "type": "integer",
      "description": "Page number",
      "default": 1,
      "minimum": 1
    }
  },
  "required": ["endpoint", "project_id", "pipeline_id", "token"]
}
```

### 5.7 gitlab_job_get

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "job_id": {
      "type": "string",
      "description": "Job ID"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    }
  },
  "required": ["endpoint", "project_id", "job_id", "token"]
}
```

### 5.8 gitlab_job_log

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "GitLab instance URL"
    },
    "project_id": {
      "type": "string",
      "description": "Project ID or path"
    },
    "job_id": {
      "type": "string",
      "description": "Job ID"
    },
    "token": {
      "type": "string",
      "description": "GitLab private token or OAuth2 token"
    }
  },
  "required": ["endpoint", "project_id", "job_id", "token"]
}
```

---

## 6. Error Handling

### 6.1 HTTP Status Codes

| Status | Error Type | Handling |
|--------|------------|----------|
| 401 | Authentication failed | Check token validity and scopes |
| 403 | Permission denied | Verify token has required permissions |
| 404 | Resource not found | Validate project ID and pipeline/job ID |
| 409 | Conflict | Pipeline already exists or cannot be modified |
| 422 | Unprocessable entity | Invalid parameters (e.g., invalid ref) |
| 429 | Rate limit exceeded | Implement exponential backoff with Retry-After header |
| 500-599 | Server error | Retry with exponential backoff |

### 6.2 Error Response Format

```json
{
  "message": "404 Project Not Found",
  "error": "Not Found"
}
```

### 6.3 Common Error Scenarios

**Invalid Token**:
```rust
if status == 401 {
    return Ok(ToolResult::error(
        "Authentication failed. Check token validity and scopes (api, read_api required).".to_string()
    ));
}
```

**Project Not Found**:
```rust
if status == 404 {
    let body: serde_json::Value = response.json().await?;
    let message = body.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Resource not found");

    return Ok(ToolResult::error(format!(
        "Project not found: {}. Check project_id (use numeric ID or URL-encoded path)",
        message
    )));
}
```

**Pipeline Cannot Be Retried**:
```rust
if status == 409 {
    return Ok(ToolResult::error(
        "Pipeline cannot be retried. Only failed pipelines can be retried.".to_string()
    ));
}
```

**Invalid Reference**:
```rust
if status == 422 {
    let body: serde_json::Value = response.json().await?;
    let message = body.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Invalid parameters");

    return Ok(ToolResult::error(format!(
        "Invalid reference: {}. Check that branch/tag/commit exists.",
        message
    )));
}
```

**Rate Limiting**:
```rust
if status == 429 {
    let retry_after = response.headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("60");

    let remaining = response.headers()
        .get("RateLimit-Remaining")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0");

    return Ok(ToolResult::error(format!(
        "Rate limit exceeded. {} requests remaining. Retry after {} seconds.",
        remaining, retry_after
    )));
}
```

### 6.4 Network Error Handling

```rust
match client.get(&url).send().await {
    Ok(response) => {
        // Process response
    }
    Err(e) => {
        if e.is_timeout() {
            return Ok(ToolResult::error(
                "Request timeout. GitLab may be slow or unreachable.".to_string()
            ));
        } else if e.is_connect() {
            return Ok(ToolResult::error(format!(
                "Connection failed: {}. Check endpoint URL and network connectivity.",
                e
            )));
        } else if e.is_request() {
            return Ok(ToolResult::error(format!(
                "Request failed: {}. Check request parameters.",
                e
            )));
        } else {
            return Ok(ToolResult::error(format!(
                "GitLab request failed: {}",
                e
            )));
        }
    }
}
```

### 6.5 Validation

**Pre-request Validation**:
```rust
fn validate_project_id(project_id: &str) -> AofResult<()> {
    if project_id.is_empty() {
        return Err(AofError::tool("project_id cannot be empty".to_string()));
    }
    Ok(())
}

fn validate_endpoint(endpoint: &str) -> AofResult<()> {
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(AofError::tool(
            "endpoint must start with http:// or https://".to_string()
        ));
    }
    Ok(())
}

fn validate_pipeline_id(pipeline_id: &str) -> AofResult<()> {
    if pipeline_id.parse::<u64>().is_err() {
        return Err(AofError::tool(
            "pipeline_id must be a valid number".to_string()
        ));
    }
    Ok(())
}
```

---

## 7. Example Usage in Agent YAML

### 7.1 Basic Pipeline Monitoring

```yaml
name: gitlab-pipeline-monitor
description: Monitor GitLab pipeline execution

spec:
  llm:
    provider: google
    model: gemini-2.0-flash-exp
    temperature: 0.3

  tools:
    - gitlab_pipeline_list
    - gitlab_pipeline_get
    - gitlab_job_list

  memory:
    type: in_memory
    max_messages: 50

  system_prompt: |
    You are a GitLab CI/CD monitoring agent.

    Your responsibilities:
    - Monitor pipeline execution status
    - Report on pipeline success/failure
    - Identify failed jobs
    - Track pipeline duration trends

    Configuration:
    - GitLab endpoint: https://gitlab.example.com
    - Project: group/my-project
    - Token: Use $GITLAB_TOKEN environment variable

    When checking pipelines:
    1. List recent pipelines with gitlab_pipeline_list
    2. Get details for running/failed pipelines with gitlab_pipeline_get
    3. List jobs for failed pipelines with gitlab_job_list
    4. Report findings clearly
```

**Example Interaction**:
```
User: Check the status of recent pipelines

Agent: I'll check recent pipelines for the project.

[Uses gitlab_pipeline_list with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  token: $GITLAB_TOKEN
  per_page: 10
  sort: desc
]

Recent pipeline summary:
- Pipeline #1523 (main): SUCCESS - Duration: 12m 34s
- Pipeline #1522 (feature-x): FAILED - Duration: 8m 15s
- Pipeline #1521 (main): SUCCESS - Duration: 11m 58s

Pipeline #1522 failed. Let me get more details.

[Uses gitlab_job_list with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  pipeline_id: 1522
  token: $GITLAB_TOKEN
  scope: ["failed"]
]

Failed jobs in pipeline #1522:
- Job: test-backend (stage: test)
  - Duration: 2m 45s
  - Failed at: 2024-01-15 10:45:00 UTC
```

### 7.2 Automated Deployment Agent

```yaml
name: gitlab-deploy-agent
description: Automated deployment trigger and monitoring

spec:
  llm:
    provider: google
    model: gemini-2.0-flash-exp
    temperature: 0.2

  tools:
    - gitlab_pipeline_create
    - gitlab_pipeline_get
    - gitlab_job_list
    - gitlab_job_log

  memory:
    type: in_memory
    max_messages: 100

  system_prompt: |
    You are a deployment automation agent for GitLab CI/CD.

    Your capabilities:
    - Trigger deployment pipelines
    - Monitor deployment progress
    - Retrieve logs for failed deployments
    - Report deployment status

    Deployment workflow:
    1. Trigger pipeline with required variables
    2. Monitor pipeline status
    3. Check job logs if any failures occur
    4. Report final deployment status

    Configuration:
    - GitLab endpoint: https://gitlab.example.com
    - Project: ops/production-app
    - Token: Use $GITLAB_TOKEN environment variable

    Always confirm deployment parameters before triggering.
```

**Example Interaction**:
```
User: Deploy version 2.5.0 to production

Agent: I'll trigger a production deployment for version 2.5.0.

Deployment parameters:
- Environment: production
- Version: 2.5.0
- Branch: main

Proceed with deployment? (yes/no)

User: yes

Agent: Triggering deployment pipeline...

[Uses gitlab_pipeline_create with:
  endpoint: https://gitlab.example.com
  project_id: ops/production-app
  token: $GITLAB_TOKEN
  ref: main
  variables:
    - key: DEPLOY_ENV
      value: production
    - key: VERSION
      value: 2.5.0
    - key: DEPLOYMENT_TYPE
      value: rolling
]

Pipeline #2045 created successfully.
Status: running
URL: https://gitlab.example.com/ops/production-app/pipelines/2045

Monitoring deployment progress...

[Polls gitlab_pipeline_get every 30 seconds]

Pipeline #2045 - Status: success
- Duration: 15m 23s
- Started: 2024-01-15 11:00:00 UTC
- Finished: 2024-01-15 11:15:23 UTC

Deployment to production completed successfully!
Version 2.5.0 is now live.
```

### 7.3 Pipeline Failure Investigation

```yaml
name: gitlab-failure-investigator
description: Investigate and analyze pipeline failures

spec:
  llm:
    provider: google
    model: gemini-2.0-flash-exp
    temperature: 0.4

  tools:
    - gitlab_pipeline_get
    - gitlab_job_list
    - gitlab_job_log
    - gitlab_pipeline_retry

  memory:
    type: in_memory
    max_messages: 150

  system_prompt: |
    You are a CI/CD failure investigation agent.

    When investigating pipeline failures:
    1. Get pipeline details
    2. List all jobs in the pipeline
    3. Retrieve logs for failed jobs
    4. Analyze error patterns
    5. Suggest fixes or retry if transient

    Look for common issues:
    - Dependency installation failures
    - Test failures
    - Timeout errors
    - Resource constraints
    - Network issues

    Configuration:
    - GitLab endpoint: https://gitlab.example.com
    - Token: Use $GITLAB_TOKEN environment variable

    Provide clear analysis and actionable recommendations.
```

**Example Interaction**:
```
User: Investigate why pipeline 1522 failed

Agent: I'll investigate pipeline #1522.

[Uses gitlab_pipeline_get with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  pipeline_id: 1522
  token: $GITLAB_TOKEN
]

Pipeline #1522 Details:
- Branch: feature-x
- Status: failed
- Duration: 8m 15s
- Triggered by: john.doe

[Uses gitlab_job_list with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  pipeline_id: 1522
  token: $GITLAB_TOKEN
]

Jobs:
- build: success (2m 30s)
- test-frontend: success (3m 15s)
- test-backend: failed (2m 45s)
- deploy: skipped

Failed job: test-backend

[Uses gitlab_job_log with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  job_id: 5234
  token: $GITLAB_TOKEN
]

Log analysis for test-backend:
```
npm test

> backend@1.0.0 test
> jest --coverage

FAIL src/api/users.test.js
  ● User API › POST /users › should create new user

    expect(received).toBe(expected)

    Expected: 201
    Received: 500

    at Object.<anonymous> (src/api/users.test.js:45:28)

Test Suites: 1 failed, 5 passed, 6 total
Tests:       1 failed, 32 passed, 33 total
```

Analysis:
The test-backend job failed due to a failing test in the User API.
The test expects a 201 status code but receives 500 (internal server error).

Issue: The POST /users endpoint is returning a server error instead of
successfully creating a user.

This appears to be a legitimate code issue, not a transient failure.

Recommendation:
1. Check the users.test.js file for test changes
2. Verify the POST /users endpoint implementation
3. Check database connection/migrations
4. Review recent commits on the feature-x branch

This pipeline should NOT be retried until the code issue is fixed.
```

### 7.4 CI/CD Dashboard Agent

```yaml
name: gitlab-dashboard
description: Provide CI/CD metrics and insights

spec:
  llm:
    provider: google
    model: gemini-2.0-flash-exp
    temperature: 0.3

  tools:
    - gitlab_pipeline_list
    - gitlab_pipeline_get

  memory:
    type: in_memory
    max_messages: 100

  system_prompt: |
    You are a CI/CD metrics and insights agent.

    Provide dashboard-style summaries:
    - Pipeline success rates
    - Average pipeline duration
    - Recent failures and trends
    - Branch-specific metrics

    When generating reports:
    1. Fetch recent pipelines
    2. Calculate statistics
    3. Identify patterns
    4. Present in clear, visual format

    Configuration:
    - GitLab endpoint: https://gitlab.example.com
    - Project: Use project_id from context
    - Token: Use $GITLAB_TOKEN environment variable
```

**Example Interaction**:
```
User: Show me the CI/CD dashboard for the last 24 hours

Agent: Generating CI/CD dashboard for the last 24 hours...

[Uses gitlab_pipeline_list with:
  endpoint: https://gitlab.example.com
  project_id: group/my-project
  token: $GITLAB_TOKEN
  updated_after: 2024-01-14T11:00:00Z
  per_page: 100
]

═══════════════════════════════════════════════════════
           CI/CD DASHBOARD - Last 24 Hours
═══════════════════════════════════════════════════════

Pipeline Summary:
  Total Pipelines: 47
  ✓ Success: 39 (83.0%)
  ✗ Failed: 6 (12.8%)
  ⦿ Running: 2 (4.2%)

Performance Metrics:
  Average Duration: 11m 34s
  Fastest: 8m 12s (pipeline #1534)
  Slowest: 18m 45s (pipeline #1501)

Branch Activity:
  main: 15 pipelines (93.3% success)
  develop: 12 pipelines (91.7% success)
  feature branches: 20 pipelines (70.0% success)

Recent Failures:
  1. Pipeline #1522 (feature-x) - test-backend job failed
  2. Pipeline #1515 (feature-y) - timeout in integration tests
  3. Pipeline #1508 (hotfix-db) - deployment validation failed

Trends:
  ↑ Success rate improved by 5% vs. previous 24h
  ↓ Average duration reduced by 2m 15s
  → Failure rate stable at ~13%

Recommendations:
  - Investigate recurring test-backend failures
  - Optimize integration test timeout settings
  - Review deployment validation for hotfix branches
═══════════════════════════════════════════════════════
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project_id() {
        assert_eq!(encode_project_id("123"), "123");
        assert_eq!(encode_project_id("group/project"), "group%2Fproject");
        assert_eq!(
            encode_project_id("group/subgroup/project"),
            "group%2Fsubgroup%2Fproject"
        );
    }

    #[test]
    fn test_build_project_url() {
        let url = build_project_url(
            "https://gitlab.com",
            "group/project",
            "pipelines"
        );
        assert_eq!(
            url,
            "https://gitlab.com/api/v4/projects/group%2Fproject/pipelines"
        );
    }

    #[test]
    fn test_pipeline_variable_creation() {
        let vars = vec![
            serde_json::json!({
                "key": "ENV",
                "value": "production",
                "variable_type": "env_var"
            })
        ];

        let result = create_pipeline_variables(vars).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "ENV");
        assert_eq!(result[0].value, "production");
        assert_eq!(result[0].variable_type, Some("env_var".to_string()));
    }
}
```

### 8.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    // Requires GITLAB_TOKEN and GITLAB_PROJECT_ID environment variables
    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_list_pipelines() {
        let token = std::env::var("GITLAB_TOKEN").unwrap();
        let project_id = std::env::var("GITLAB_PROJECT_ID").unwrap();

        let tool = GitlabPipelineListTool::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": "https://gitlab.com",
            "project_id": project_id,
            "token": token,
            "per_page": 5
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_and_cancel_pipeline() {
        let token = std::env::var("GITLAB_TOKEN").unwrap();
        let project_id = std::env::var("GITLAB_PROJECT_ID").unwrap();

        // Create pipeline
        let create_tool = GitlabPipelineCreateTool::new();
        let create_input = ToolInput::new(serde_json::json!({
            "endpoint": "https://gitlab.com",
            "project_id": project_id,
            "token": token,
            "ref": "main",
            "variables": [
                {
                    "key": "TEST_MODE",
                    "value": "true"
                }
            ]
        }));

        let create_result = create_tool.execute(create_input).await.unwrap();
        assert!(create_result.is_success());

        let pipeline_id = create_result.data()
            .get("pipeline_id")
            .unwrap()
            .as_u64()
            .unwrap();

        // Cancel pipeline
        let cancel_tool = GitlabPipelineCancelTool::new();
        let cancel_input = ToolInput::new(serde_json::json!({
            "endpoint": "https://gitlab.com",
            "project_id": project_id,
            "token": token,
            "pipeline_id": pipeline_id.to_string()
        }));

        let cancel_result = cancel_tool.execute(cancel_input).await.unwrap();
        assert!(cancel_result.is_success());
    }
}
```

### 8.3 Mock Testing

```rust
#[cfg(test)]
mod mock_tests {
    use mockito::{Server, ServerGuard};

    async fn setup_mock_server() -> ServerGuard {
        Server::new_async().await
    }

    #[tokio::test]
    async fn test_pipeline_list_mock() {
        let mut server = setup_mock_server().await;

        let mock = server.mock("GET", "/api/v4/projects/123/pipelines")
            .match_header("authorization", "Bearer test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "id": 1001,
                    "status": "success",
                    "ref": "main"
                }
            ]"#)
            .create_async()
            .await;

        let tool = GitlabPipelineListTool::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": server.url(),
            "project_id": "123",
            "token": "test-token"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.is_success());

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let mut server = setup_mock_server().await;

        let mock = server.mock("GET", "/api/v4/projects/123/pipelines")
            .with_status(401)
            .with_body(r#"{"message": "401 Unauthorized"}"#)
            .create_async()
            .await;

        let tool = GitlabPipelineListTool::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": server.url(),
            "project_id": "123",
            "token": "invalid-token"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.is_error());
        assert!(result.error().unwrap().contains("Authentication failed"));

        mock.assert_async().await;
    }
}
```

---

## 9. Performance Considerations

### 9.1 Caching Strategies

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct GitlabCache {
    pipeline_cache: HashMap<String, (serde_json::Value, Instant)>,
    cache_ttl: Duration,
}

impl GitlabCache {
    fn new() -> Self {
        Self {
            pipeline_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(30), // 30 second cache
        }
    }

    fn get_pipeline(&self, pipeline_id: &str) -> Option<&serde_json::Value> {
        self.pipeline_cache.get(pipeline_id).and_then(|(value, time)| {
            if time.elapsed() < self.cache_ttl {
                Some(value)
            } else {
                None
            }
        })
    }

    fn set_pipeline(&mut self, pipeline_id: String, value: serde_json::Value) {
        self.pipeline_cache.insert(pipeline_id, (value, Instant::now()));
    }
}
```

### 9.2 Rate Limit Handling

```rust
use tokio::time::sleep;

struct RateLimiter {
    requests_remaining: u32,
    reset_time: Instant,
}

impl RateLimiter {
    async fn check_and_wait(&mut self, headers: &HeaderMap) {
        if let Some(remaining) = headers.get("RateLimit-Remaining") {
            if let Ok(count) = remaining.to_str().unwrap_or("0").parse::<u32>() {
                self.requests_remaining = count;

                if count < 10 {
                    // Low on requests, slow down
                    sleep(Duration::from_secs(2)).await;
                }

                if count == 0 {
                    // Out of requests, wait for reset
                    if let Some(reset) = headers.get("RateLimit-Reset") {
                        if let Ok(timestamp) = reset.to_str().unwrap_or("0").parse::<u64>() {
                            let wait_time = timestamp.saturating_sub(
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );
                            sleep(Duration::from_secs(wait_time + 1)).await;
                        }
                    }
                }
            }
        }
    }
}
```

### 9.3 Batch Operations

```rust
async fn list_all_pipelines(
    client: &Client,
    endpoint: &str,
    project_id: &str,
) -> AofResult<Vec<serde_json::Value>> {
    let mut all_pipelines = Vec::new();
    let mut page = 1;
    let per_page = 100; // Max allowed

    loop {
        let url = build_project_url(endpoint, project_id, "pipelines");
        let response = client
            .get(&url)
            .query(&[("per_page", per_page), ("page", page)])
            .send()
            .await?;

        // Check if this is the last page
        let has_next = response.headers()
            .get("Link")
            .and_then(|h| h.to_str().ok())
            .map(|link| link.contains("rel=\"next\""))
            .unwrap_or(false);

        let pipelines: Vec<serde_json::Value> = response.json().await?;

        if pipelines.is_empty() {
            break;
        }

        all_pipelines.extend(pipelines);

        if !has_next {
            break;
        }

        page += 1;
    }

    Ok(all_pipelines)
}
```

---

## 10. Security Considerations

### 10.1 Token Storage

**Environment Variables (Recommended)**:
```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
export GITLAB_PROJECT_ID="123"
```

**Agent YAML**:
```yaml
spec:
  tools:
    - gitlab_pipeline_list

  # Token from environment
  env:
    GITLAB_TOKEN: ${GITLAB_TOKEN}
    GITLAB_PROJECT_ID: ${GITLAB_PROJECT_ID}
```

**Secrets Management**:
- Use HashiCorp Vault
- Use AWS Secrets Manager
- Use Azure Key Vault
- Never commit tokens to git

### 10.2 Token Scopes

**Minimum Required Scopes**:
- `api`: Full API access
- `read_api`: Read-only API access (for monitoring agents)
- `write_repository`: Required for triggering pipelines

**Recommended Practice**:
- Use project access tokens instead of personal access tokens
- Set token expiration dates
- Rotate tokens regularly
- Use separate tokens for different agents

### 10.3 Input Validation

```rust
fn sanitize_ref(ref_name: &str) -> AofResult<String> {
    // Prevent command injection
    if ref_name.contains("&&") || ref_name.contains(";") {
        return Err(AofError::tool("Invalid ref name".to_string()));
    }

    // Validate ref format
    let valid_chars = ref_name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.'
    });

    if !valid_chars {
        return Err(AofError::tool("Ref contains invalid characters".to_string()));
    }

    Ok(ref_name.to_string())
}
```

### 10.4 HTTPS Enforcement

```rust
fn validate_endpoint(endpoint: &str) -> AofResult<()> {
    if endpoint.starts_with("http://") {
        return Err(AofError::tool(
            "HTTP endpoints are not secure. Use HTTPS.".to_string()
        ));
    }

    if !endpoint.starts_with("https://") {
        return Err(AofError::tool(
            "Endpoint must start with https://".to_string()
        ));
    }

    Ok(())
}
```

---

## 11. Monitoring and Observability

### 11.1 Logging

```rust
use tracing::{debug, info, warn, error};

#[async_trait]
impl Tool for GitlabPipelineCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let project_id: String = input.get_arg("project_id")?;
        let ref_name: String = input.get_arg("ref")?;

        info!(
            project_id = %project_id,
            ref_name = %ref_name,
            "Creating GitLab pipeline"
        );

        // Execute request
        match client.post(&url).json(&payload).send().await {
            Ok(response) => {
                let status = response.status();
                debug!(status = %status, "Received response from GitLab");

                if status.is_success() {
                    info!(
                        project_id = %project_id,
                        ref_name = %ref_name,
                        "Pipeline created successfully"
                    );
                } else {
                    warn!(
                        project_id = %project_id,
                        status = %status,
                        "Pipeline creation failed"
                    );
                }
            }
            Err(e) => {
                error!(
                    project_id = %project_id,
                    error = %e,
                    "Pipeline creation request failed"
                );
            }
        }
    }
}
```

### 11.2 Metrics Collection

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct GitlabMetrics {
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    rate_limited: AtomicU64,
}

impl GitlabMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            rate_limited: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rate_limit(&self) {
        self.rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> GitlabStats {
        GitlabStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub struct GitlabStats {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub rate_limited: u64,
}
```

---

## 12. Future Enhancements

### 12.1 Planned Features

1. **Job Artifact Download**
   - `gitlab_artifact_download`: Download job artifacts
   - Support for specific artifact paths
   - Artifact metadata retrieval

2. **Pipeline Variables Management**
   - `gitlab_variable_list`: List project/group variables
   - `gitlab_variable_create`: Create CI/CD variables
   - `gitlab_variable_update`: Update existing variables

3. **Merge Request Integration**
   - `gitlab_mr_pipeline_list`: List MR pipelines
   - `gitlab_mr_pipeline_trigger`: Trigger MR pipeline

4. **Job Control**
   - `gitlab_job_retry`: Retry specific job
   - `gitlab_job_cancel`: Cancel running job
   - `gitlab_job_play`: Trigger manual job

5. **Pipeline Scheduling**
   - `gitlab_schedule_list`: List pipeline schedules
   - `gitlab_schedule_create`: Create new schedule
   - `gitlab_schedule_trigger`: Run scheduled pipeline

### 12.2 Advanced Features

1. **Webhook Integration**
   - Register webhook for pipeline events
   - Real-time pipeline status updates
   - Event filtering

2. **Pipeline Analytics**
   - Historical pipeline metrics
   - Success rate calculations
   - Duration trend analysis

3. **Multi-Project Support**
   - Batch operations across projects
   - Cross-project dependency tracking
   - Monorepo pipeline coordination

---

## 13. References

### 13.1 GitLab API Documentation
- [GitLab API Overview](https://docs.gitlab.com/ee/api/api_resources.html)
- [Pipelines API](https://docs.gitlab.com/ee/api/pipelines.html)
- [Jobs API](https://docs.gitlab.com/ee/api/jobs.html)
- [Authentication](https://docs.gitlab.com/ee/api/index.html#authentication)

### 13.2 Related Tools
- **Grafana Tool**: `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/grafana.rs`
- **GitHub Actions Tool**: (Future implementation)
- **Jenkins Tool**: (Future implementation)

### 13.3 Implementation Pattern
- **Tool Trait**: `/Users/gshah/work/opsflow-sh/aof/crates/aof-core/src/tool/mod.rs`
- **HTTP Client Pattern**: See Grafana tool for reference
- **Error Handling**: Follow existing AOF error patterns
