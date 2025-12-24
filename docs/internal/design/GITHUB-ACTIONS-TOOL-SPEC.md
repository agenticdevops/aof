# GitHub Actions Tool - Internal Design Specification

## 1. Overview

The GitHub Actions Tool provides AOF agents with the ability to interact with GitHub's CI/CD platform through the REST API. This enables agents to trigger workflows, monitor pipeline status, manage workflow runs, and access build artifacts programmatically.

### Capabilities

- **Workflow Management**: List and inspect workflow definitions
- **Workflow Execution**: Trigger workflow runs via dispatch events
- **Run Monitoring**: List, inspect, and track workflow run status
- **Run Control**: Cancel, rerun, or force-cancel workflow runs
- **Artifact Management**: List and download workflow artifacts
- **Log Access**: Download workflow run logs

### Use Cases

- **CI/CD Automation**: Trigger builds and deployments from agents
- **Pipeline Monitoring**: Track build status and pipeline health
- **Artifact Retrieval**: Download build outputs and test results
- **Workflow Orchestration**: Coordinate complex multi-workflow pipelines
- **Deployment Automation**: Trigger and monitor deployment workflows

## 2. Tool Operations

### 2.1 github_workflow_list

List all workflows in a repository.

**Purpose**: Discover available workflows and their configuration.

**Parameters**:
- `token` (string, required): GitHub Personal Access Token (PAT) or GitHub App token
- `owner` (string, required): Repository owner (username or organization)
- `repo` (string, required): Repository name
- `per_page` (integer, optional): Results per page (default: 30, max: 100)
- `page` (integer, optional): Page number (default: 1)

**Response**:
```json
{
  "workflows": [
    {
      "id": 12345,
      "name": "CI",
      "path": ".github/workflows/ci.yml",
      "state": "active",
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-15T00:00:00Z",
      "url": "https://api.github.com/repos/owner/repo/actions/workflows/12345",
      "html_url": "https://github.com/owner/repo/actions/workflows/ci.yml",
      "badge_url": "https://github.com/owner/repo/workflows/CI/badge.svg"
    }
  ],
  "total_count": 5
}
```

---

### 2.2 github_workflow_dispatch

Trigger a workflow run via dispatch event.

**Purpose**: Programmatically start a workflow execution with custom inputs.

**Prerequisites**:
- Workflow must have `workflow_dispatch` event configured
- Workflow must exist in the default branch

**Parameters**:
- `token` (string, required): GitHub PAT with `repo` scope
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `workflow_id` (string/integer, required): Workflow ID or filename (e.g., "ci.yml")
- `ref` (string, required): Git ref (branch, tag, or commit SHA)
- `inputs` (object, optional): Input parameters (max 25 key-value pairs)

**Request Example**:
```json
{
  "token": "ghp_xxxxx",
  "owner": "myorg",
  "repo": "myapp",
  "workflow_id": "deploy.yml",
  "ref": "main",
  "inputs": {
    "environment": "production",
    "version": "v1.2.3"
  }
}
```

**Response**:
```json
{
  "status": "dispatched",
  "workflow_id": "deploy.yml",
  "ref": "main",
  "message": "Workflow dispatch event created successfully"
}
```

**HTTP Status**: 204 No Content (success)

---

### 2.3 github_run_list

List workflow runs for a repository.

**Purpose**: Monitor pipeline execution history and current status.

**Parameters**:
- `token` (string, required): GitHub PAT
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `workflow_id` (string/integer, optional): Filter by workflow
- `actor` (string, optional): Filter by user who triggered the run
- `branch` (string, optional): Filter by branch
- `event` (string, optional): Filter by event type (push, pull_request, workflow_dispatch)
- `status` (string, optional): Filter by status (queued, in_progress, completed)
- `created` (string, optional): Filter by creation date (ISO 8601 or date range)
- `head_sha` (string, optional): Filter by commit SHA
- `exclude_pull_requests` (boolean, optional): Exclude PR-triggered runs
- `per_page` (integer, optional): Results per page (default: 30, max: 100)
- `page` (integer, optional): Page number (default: 1)

**Response**:
```json
{
  "workflow_runs": [
    {
      "id": 987654321,
      "name": "CI",
      "head_branch": "main",
      "head_sha": "abc123def456",
      "status": "completed",
      "conclusion": "success",
      "workflow_id": 12345,
      "run_number": 42,
      "event": "push",
      "created_at": "2025-01-23T10:00:00Z",
      "updated_at": "2025-01-23T10:15:00Z",
      "run_started_at": "2025-01-23T10:00:05Z",
      "url": "https://api.github.com/repos/owner/repo/actions/runs/987654321",
      "html_url": "https://github.com/owner/repo/actions/runs/987654321",
      "jobs_url": "https://api.github.com/repos/owner/repo/actions/runs/987654321/jobs",
      "logs_url": "https://api.github.com/repos/owner/repo/actions/runs/987654321/logs",
      "artifacts_url": "https://api.github.com/repos/owner/repo/actions/runs/987654321/artifacts"
    }
  ],
  "total_count": 150
}
```

**Status Values**:
- `queued`: Waiting to start
- `in_progress`: Currently running
- `completed`: Finished (check `conclusion`)

**Conclusion Values** (when status is completed):
- `success`: All jobs passed
- `failure`: One or more jobs failed
- `cancelled`: Run was cancelled
- `skipped`: Run was skipped
- `timed_out`: Run exceeded timeout
- `action_required`: Manual approval needed
- `neutral`: Completed with neutral status

---

### 2.4 github_run_get

Get details of a specific workflow run.

**Purpose**: Inspect detailed status, timing, and metadata of a workflow run.

**Parameters**:
- `token` (string, required): GitHub PAT
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, required): Workflow run ID
- `exclude_pull_requests` (boolean, optional): Exclude PR context

**Response**:
```json
{
  "id": 987654321,
  "name": "CI",
  "head_branch": "main",
  "head_sha": "abc123def456",
  "status": "in_progress",
  "conclusion": null,
  "workflow_id": 12345,
  "run_number": 42,
  "run_attempt": 1,
  "event": "push",
  "actor": {
    "login": "username",
    "id": 123456
  },
  "triggering_actor": {
    "login": "username",
    "id": 123456
  },
  "created_at": "2025-01-23T10:00:00Z",
  "updated_at": "2025-01-23T10:10:00Z",
  "run_started_at": "2025-01-23T10:00:05Z",
  "jobs_count": 3,
  "url": "https://api.github.com/repos/owner/repo/actions/runs/987654321",
  "html_url": "https://github.com/owner/repo/actions/runs/987654321"
}
```

---

### 2.5 github_run_cancel

Cancel a running workflow.

**Purpose**: Stop an in-progress workflow run.

**Parameters**:
- `token` (string, required): GitHub PAT with `repo` scope
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, required): Workflow run ID to cancel

**Response**:
```json
{
  "status": "cancelled",
  "run_id": 987654321,
  "message": "Workflow run cancelled successfully"
}
```

**HTTP Status**: 202 Accepted

**Note**: Cancellation is asynchronous. The run status will transition to `completed` with conclusion `cancelled`.

---

### 2.6 github_run_rerun

Rerun a failed or cancelled workflow.

**Purpose**: Retry a workflow run, optionally with debug logging.

**Parameters**:
- `token` (string, required): GitHub PAT with `repo` scope
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, required): Workflow run ID to rerun
- `enable_debug_logging` (boolean, optional): Enable step debug and runner diagnostic logging

**Variants**:
- **Rerun entire workflow**: `/actions/runs/{run_id}/rerun`
- **Rerun failed jobs only**: `/actions/runs/{run_id}/rerun-failed-jobs`

**Response**:
```json
{
  "status": "rerun_requested",
  "run_id": 987654321,
  "debug_logging": false,
  "message": "Workflow rerun initiated"
}
```

**HTTP Status**: 201 Created

---

### 2.7 github_run_force_cancel

Force cancel an unresponsive workflow.

**Purpose**: Forcefully terminate a workflow that isn't responding to normal cancellation.

**Parameters**:
- `token` (string, required): GitHub PAT with `repo` scope
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, required): Workflow run ID to force cancel

**Response**:
```json
{
  "status": "force_cancelled",
  "run_id": 987654321,
  "message": "Workflow force cancelled"
}
```

**HTTP Status**: 202 Accepted

---

### 2.8 github_artifacts_list

List artifacts for a workflow run or repository.

**Purpose**: Discover available build outputs, test results, and generated files.

**Parameters**:
- `token` (string, required): GitHub PAT
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, optional): Filter by specific workflow run
- `name` (string, optional): Filter by artifact name
- `per_page` (integer, optional): Results per page (default: 30, max: 100)
- `page` (integer, optional): Page number (default: 1)

**Response**:
```json
{
  "artifacts": [
    {
      "id": 123456789,
      "name": "build-artifacts",
      "size_in_bytes": 1024000,
      "url": "https://api.github.com/repos/owner/repo/actions/artifacts/123456789",
      "archive_download_url": "https://api.github.com/repos/owner/repo/actions/artifacts/123456789/zip",
      "expired": false,
      "created_at": "2025-01-23T10:15:00Z",
      "updated_at": "2025-01-23T10:15:00Z",
      "expires_at": "2025-04-23T10:15:00Z",
      "workflow_run": {
        "id": 987654321,
        "repository_id": 456789,
        "head_repository_id": 456789,
        "head_branch": "main",
        "head_sha": "abc123def456"
      }
    }
  ],
  "total_count": 10
}
```

**Note**: Artifacts expire after 90 days by default (configurable per repository).

---

### 2.9 github_artifacts_download

Download a workflow artifact.

**Purpose**: Retrieve artifact ZIP archive for local processing.

**Parameters**:
- `token` (string, required): GitHub PAT
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `artifact_id` (integer, required): Artifact ID
- `output_path` (string, optional): Local path to save artifact (default: current directory)

**Response**:
```json
{
  "artifact_id": 123456789,
  "name": "build-artifacts",
  "file_path": "/tmp/build-artifacts.zip",
  "size_bytes": 1024000,
  "downloaded_at": "2025-01-23T12:00:00Z"
}
```

**HTTP Status**: 302 Found (redirect to download URL) or 410 Gone (expired)

**Implementation Note**: The API returns a redirect. The tool should:
1. GET `/repos/{owner}/{repo}/actions/artifacts/{artifact_id}/zip`
2. Follow 302 redirect to actual download URL
3. Stream ZIP file to local filesystem
4. Verify file integrity if size is provided

---

### 2.10 github_run_logs

Download workflow run logs.

**Purpose**: Retrieve complete execution logs for debugging and analysis.

**Parameters**:
- `token` (string, required): GitHub PAT
- `owner` (string, required): Repository owner
- `repo` (string, required): Repository name
- `run_id` (integer, required): Workflow run ID
- `output_path` (string, optional): Local path to save logs (default: current directory)

**Response**:
```json
{
  "run_id": 987654321,
  "file_path": "/tmp/workflow-run-987654321-logs.zip",
  "size_bytes": 512000,
  "downloaded_at": "2025-01-23T12:00:00Z"
}
```

**HTTP Status**: 302 Found (redirect to download URL)

**Log Format**: ZIP archive containing separate log files for each job/step.

## 3. Configuration

### 3.1 Authentication

GitHub Actions API requires authentication via:
- **Personal Access Token (PAT)**: Classic or fine-grained
- **GitHub App Token**: For organization-level integrations

**Required Scopes**:
- `repo`: Full repository access (read/write)
- For public repositories: Read operations may work with reduced permissions

**Token Storage**:
- Store in agent configuration as environment variable
- Never hardcode tokens in agent YAML files
- Use AOF's secret management system

**Example Configuration**:
```yaml
spec:
  config:
    github_token: "${GITHUB_TOKEN}"  # From environment
```

### 3.2 Repository Identification

All operations require:
- **owner**: Repository owner (user or organization name)
- **repo**: Repository name

**Example**:
- Repository: `https://github.com/agenticdevops/aof`
- Owner: `agenticdevops`
- Repo: `aof`

### 3.3 Rate Limiting

GitHub API rate limits:
- **Authenticated requests**: 5,000 requests/hour
- **GitHub App installations**: 15,000 requests/hour

**Rate Limit Headers** (returned in responses):
- `X-RateLimit-Limit`: Maximum requests per hour
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Unix timestamp when limit resets

**Tool Behavior on Rate Limit**:
- Return error with retry-after timestamp
- Log rate limit status for monitoring
- Suggest using conditional requests with ETags

### 3.4 Pagination

List operations support pagination:
- Default: 30 items per page
- Maximum: 100 items per page
- Parameters: `per_page`, `page`

**Link Header Format**:
```
Link: <https://api.github.com/repos/owner/repo/actions/runs?page=2>; rel="next",
      <https://api.github.com/repos/owner/repo/actions/runs?page=5>; rel="last"
```

**Tool Implementation**:
- Parse `Link` header for next page
- Provide `has_next_page` boolean in response
- Return `next_page` number if available

## 4. Implementation Details

### 4.1 HTTP Client Setup

```rust
use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest;
use serde_json;

fn create_github_client(token: &str) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Bearer token authentication
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    // GitHub API version
    headers.insert(
        "X-GitHub-Api-Version",
        reqwest::header::HeaderValue::from_static("2022-11-28"),
    );

    // User agent (required by GitHub)
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("aof-github-actions-tool"),
    );

    // Accept header for JSON
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.2 Base URL

All endpoints are relative to: `https://api.github.com`

**Full URL Construction**:
```rust
let base_url = "https://api.github.com";
let endpoint = format!("{}/repos/{}/{}/actions/workflows", base_url, owner, repo);
```

### 4.3 Authentication Header

**Format**: `Authorization: Bearer <token>`

**Example**:
```
Authorization: Bearer ghp_1234567890abcdefghijklmnopqrstuvwxyz
```

### 4.4 API Version Header

GitHub recommends specifying API version:
```
X-GitHub-Api-Version: 2022-11-28
```

### 4.5 User Agent Header

**Required** by GitHub API:
```
User-Agent: aof-github-actions-tool
```

### 4.6 Accept Header

For consistent JSON responses:
```
Accept: application/vnd.github+json
```

### 4.7 Pagination Handling

```rust
async fn list_with_pagination(
    client: &reqwest::Client,
    url: &str,
    per_page: i32,
    page: i32,
) -> AofResult<serde_json::Value> {
    let params = [
        ("per_page", per_page.to_string()),
        ("page", page.to_string()),
    ];

    let response = client.get(url).query(&params).send().await
        .map_err(|e| aof_core::AofError::tool(format!("Request failed: {}", e)))?;

    // Parse Link header for pagination
    let link_header = response.headers().get("Link")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let has_next = link_header.as_ref()
        .map(|link| link.contains("rel=\"next\""))
        .unwrap_or(false);

    let body: serde_json::Value = response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to parse response: {}", e)))?;

    // Add pagination metadata
    let mut result = body.clone();
    result["pagination"] = serde_json::json!({
        "current_page": page,
        "per_page": per_page,
        "has_next_page": has_next
    });

    Ok(result)
}
```

### 4.8 Response Parsing

```rust
async fn parse_response(response: reqwest::Response) -> AofResult<ToolResult> {
    let status = response.status().as_u16();

    // Handle different status codes
    match status {
        200 | 201 => {
            let body: serde_json::Value = response.json().await
                .map_err(|e| aof_core::AofError::tool(format!("Parse error: {}", e)))?;
            Ok(ToolResult::success(body))
        },
        204 => {
            Ok(ToolResult::success(serde_json::json!({
                "message": "Operation completed successfully"
            })))
        },
        302 => {
            // Handle redirects for downloads
            let location = response.headers().get("Location")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| aof_core::AofError::tool("Missing redirect location".into()))?;

            Ok(ToolResult::success(serde_json::json!({
                "redirect_url": location
            })))
        },
        401 => {
            Ok(ToolResult::error("Authentication failed. Check token permissions.".into()))
        },
        403 => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let message = body.get("message").and_then(|m| m.as_str())
                .unwrap_or("Forbidden. Check repository permissions or rate limits.");
            Ok(ToolResult::error(message.to_string()))
        },
        404 => {
            Ok(ToolResult::error("Resource not found. Check owner, repo, and ID.".into()))
        },
        409 => {
            Ok(ToolResult::error("Conflict. Workflow may not be in valid state for this operation.".into()))
        },
        410 => {
            Ok(ToolResult::error("Resource expired or deleted.".into()))
        },
        422 => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let message = body.get("message").and_then(|m| m.as_str())
                .unwrap_or("Validation failed");
            Ok(ToolResult::error(format!("Validation error: {}", message)))
        },
        429 => {
            let retry_after = response.headers().get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown");
            Ok(ToolResult::error(format!("Rate limited. Retry after: {}", retry_after)))
        },
        _ => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            Ok(ToolResult::error(format!("GitHub API error ({}): {:?}", status, body)))
        }
    }
}
```

### 4.9 Download Handling

For artifact and log downloads:

```rust
async fn download_file(
    client: &reqwest::Client,
    url: &str,
    output_path: &str,
) -> AofResult<u64> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let response = client.get(url).send().await
        .map_err(|e| aof_core::AofError::tool(format!("Download failed: {}", e)))?;

    if response.status() != 200 && response.status() != 302 {
        return Err(aof_core::AofError::tool(
            format!("Download failed with status: {}", response.status())
        ));
    }

    // Follow redirect if needed
    let download_url = if response.status() == 302 {
        response.headers().get("Location")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| aof_core::AofError::tool("Missing redirect location".into()))?
            .to_string()
    } else {
        url.to_string()
    };

    let bytes = client.get(&download_url).send().await
        .map_err(|e| aof_core::AofError::tool(format!("Download failed: {}", e)))?
        .bytes().await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to read bytes: {}", e)))?;

    let mut file = File::create(output_path).await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create file: {}", e)))?;

    file.write_all(&bytes).await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to write file: {}", e)))?;

    Ok(bytes.len() as u64)
}
```

## 5. Tool Parameters Schema

### 5.1 github_workflow_list

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner (username or organization)"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "per_page": {
      "type": "integer",
      "description": "Results per page (max 100)",
      "default": 30,
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
  "required": ["token", "owner", "repo"]
}
```

### 5.2 github_workflow_dispatch

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token with repo scope"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "workflow_id": {
      "type": ["string", "integer"],
      "description": "Workflow ID or filename (e.g., 'deploy.yml')"
    },
    "ref": {
      "type": "string",
      "description": "Git branch, tag, or commit SHA"
    },
    "inputs": {
      "type": "object",
      "description": "Workflow input parameters (max 25)",
      "additionalProperties": {
        "type": "string"
      }
    }
  },
  "required": ["token", "owner", "repo", "workflow_id", "ref"]
}
```

### 5.3 github_run_list

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "workflow_id": {
      "type": ["string", "integer"],
      "description": "Filter by workflow ID or filename"
    },
    "actor": {
      "type": "string",
      "description": "Filter by user who triggered the run"
    },
    "branch": {
      "type": "string",
      "description": "Filter by branch name"
    },
    "event": {
      "type": "string",
      "description": "Filter by event type",
      "enum": ["push", "pull_request", "workflow_dispatch", "schedule", "release"]
    },
    "status": {
      "type": "string",
      "description": "Filter by run status",
      "enum": ["queued", "in_progress", "completed"]
    },
    "created": {
      "type": "string",
      "description": "Filter by creation date (ISO 8601 or range)"
    },
    "head_sha": {
      "type": "string",
      "description": "Filter by commit SHA"
    },
    "exclude_pull_requests": {
      "type": "boolean",
      "description": "Exclude pull request triggered runs",
      "default": false
    },
    "per_page": {
      "type": "integer",
      "description": "Results per page (max 100)",
      "default": 30,
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
  "required": ["token", "owner", "repo"]
}
```

### 5.4 github_run_get

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Workflow run ID"
    }
  },
  "required": ["token", "owner", "repo", "run_id"]
}
```

### 5.5 github_run_cancel

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token with repo scope"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Workflow run ID to cancel"
    }
  },
  "required": ["token", "owner", "repo", "run_id"]
}
```

### 5.6 github_run_rerun

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token with repo scope"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Workflow run ID to rerun"
    },
    "enable_debug_logging": {
      "type": "boolean",
      "description": "Enable debug logging for rerun",
      "default": false
    },
    "rerun_failed_jobs": {
      "type": "boolean",
      "description": "Only rerun failed jobs instead of entire workflow",
      "default": false
    }
  },
  "required": ["token", "owner", "repo", "run_id"]
}
```

### 5.7 github_run_force_cancel

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token with repo scope"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Workflow run ID to force cancel"
    }
  },
  "required": ["token", "owner", "repo", "run_id"]
}
```

### 5.8 github_artifacts_list

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Filter by workflow run ID (optional)"
    },
    "name": {
      "type": "string",
      "description": "Filter by artifact name"
    },
    "per_page": {
      "type": "integer",
      "description": "Results per page (max 100)",
      "default": 30,
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
  "required": ["token", "owner", "repo"]
}
```

### 5.9 github_artifacts_download

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "artifact_id": {
      "type": "integer",
      "description": "Artifact ID to download"
    },
    "output_path": {
      "type": "string",
      "description": "Local file path to save artifact (default: ./artifact-{id}.zip)"
    }
  },
  "required": ["token", "owner", "repo", "artifact_id"]
}
```

### 5.10 github_run_logs

```json
{
  "type": "object",
  "properties": {
    "token": {
      "type": "string",
      "description": "GitHub Personal Access Token"
    },
    "owner": {
      "type": "string",
      "description": "Repository owner"
    },
    "repo": {
      "type": "string",
      "description": "Repository name"
    },
    "run_id": {
      "type": "integer",
      "description": "Workflow run ID"
    },
    "output_path": {
      "type": "string",
      "description": "Local file path to save logs (default: ./logs-{run_id}.zip)"
    }
  },
  "required": ["token", "owner", "repo", "run_id"]
}
```

## 6. Error Handling

### 6.1 HTTP Status Codes

| Status | Meaning | Tool Response |
|--------|---------|---------------|
| 200 | Success | Return parsed JSON data |
| 201 | Created | Return success with resource info |
| 202 | Accepted | Return processing status |
| 204 | No Content | Return success message |
| 302 | Redirect | Follow redirect for downloads |
| 401 | Unauthorized | "Authentication failed. Check token." |
| 403 | Forbidden | "Permission denied. Check token scopes or rate limit." |
| 404 | Not Found | "Resource not found. Verify owner/repo/ID." |
| 409 | Conflict | "Operation conflict. Check workflow state." |
| 410 | Gone | "Resource expired or deleted." |
| 422 | Validation Error | Return detailed validation message |
| 429 | Rate Limited | "Rate limited. Retry after: {timestamp}" |
| 500+ | Server Error | "GitHub server error. Try again later." |

### 6.2 Common Error Scenarios

#### Invalid Token
```json
{
  "error": "Authentication failed. Check token permissions.",
  "status": 401,
  "suggestion": "Verify GITHUB_TOKEN has 'repo' scope"
}
```

#### Repository Not Found
```json
{
  "error": "Resource not found. Verify owner/repo.",
  "status": 404,
  "suggestion": "Check repository exists and is accessible with provided token"
}
```

#### Workflow Not Dispatchable
```json
{
  "error": "Validation error: Workflow must be configured with workflow_dispatch event",
  "status": 422,
  "suggestion": "Add 'on: workflow_dispatch:' to workflow YAML"
}
```

#### Rate Limit Exceeded
```json
{
  "error": "Rate limited. Retry after: 2025-01-23T13:00:00Z",
  "status": 429,
  "rate_limit": {
    "limit": 5000,
    "remaining": 0,
    "reset": 1706014800
  },
  "suggestion": "Wait until rate limit resets or use GitHub App token for higher limits"
}
```

#### Artifact Expired
```json
{
  "error": "Resource expired or deleted.",
  "status": 410,
  "suggestion": "Artifacts expire after 90 days. Configure retention settings in repository."
}
```

### 6.3 Network Errors

#### Connection Timeout
```json
{
  "error": "Request timeout after 60s",
  "suggestion": "Check network connectivity and GitHub status"
}
```

#### Connection Failed
```json
{
  "error": "Connection failed: DNS resolution failed",
  "suggestion": "Verify network access to api.github.com"
}
```

### 6.4 Validation Errors

#### Missing Required Parameter
```rust
if owner.is_empty() {
    return Ok(ToolResult::error(
        "Parameter 'owner' is required but was not provided".to_string()
    ));
}
```

#### Invalid Workflow ID
```json
{
  "error": "Validation error: workflow_id must be numeric ID or .yml/.yaml filename",
  "provided": "invalid-name",
  "suggestion": "Use 'deploy.yml' or workflow numeric ID"
}
```

### 6.5 Error Response Format

All errors return consistent structure:
```json
{
  "error": "Human-readable error message",
  "status": 404,
  "details": {
    "message": "Additional GitHub API error details",
    "documentation_url": "https://docs.github.com/rest/..."
  },
  "suggestion": "Actionable suggestion to fix the error"
}
```

## 7. Example Usage in Agent YAML

### 7.1 Basic CI/CD Agent

```yaml
apiVersion: v1
kind: Agent
metadata:
  name: cicd-orchestrator
  description: Manages GitHub Actions workflows and monitors pipeline status

spec:
  model:
    provider: google
    model_id: gemini-2.5-flash

  tools:
    - github_workflow_list
    - github_workflow_dispatch
    - github_run_list
    - github_run_get
    - github_run_cancel
    - github_artifacts_list

  config:
    github_token: "${GITHUB_TOKEN}"
    default_owner: "agenticdevops"
    default_repo: "aof"

  system_prompt: |
    You are a CI/CD orchestrator that manages GitHub Actions workflows.

    Your capabilities:
    - Trigger deployment workflows with environment-specific inputs
    - Monitor build and test pipeline status
    - Cancel failed or stuck workflow runs
    - List and inspect workflow artifacts

    When triggering workflows:
    - Always verify the workflow exists first using github_workflow_list
    - Use appropriate branch/tag for the ref parameter
    - Validate inputs match workflow requirements

    When monitoring:
    - Check run status every 30 seconds until completion
    - Report final conclusion (success/failure/cancelled)
    - Download artifacts on successful builds

  triggers:
    - type: interval
      schedule: "*/5 * * * *"  # Every 5 minutes
      action: |
        Check for failed workflow runs in the last hour.
        If any critical workflows failed, report details and suggest rerun.
```

### 7.2 Deployment Agent

```yaml
apiVersion: v1
kind: Agent
metadata:
  name: deployment-manager
  description: Handles production deployments via GitHub Actions

spec:
  model:
    provider: google
    model_id: gemini-2.5-flash

  tools:
    - github_workflow_dispatch
    - github_run_get
    - github_run_cancel

  config:
    github_token: "${GITHUB_TOKEN}"
    repo_owner: "myorg"
    repo_name: "production-app"
    deploy_workflow: "deploy.yml"

  system_prompt: |
    You are a deployment manager for production environments.

    Deployment Process:
    1. Verify deployment approval
    2. Trigger deploy.yml workflow with environment and version inputs
    3. Monitor deployment progress
    4. If deployment exceeds 15 minutes, cancel and rollback
    5. Report deployment status and artifacts

    Safety Rules:
    - Never deploy to production without approval
    - Always specify exact version tag (no 'latest')
    - Cancel deployments that timeout
    - Verify artifact checksums before deployment

  variables:
    allowed_environments:
      - staging
      - production
    deployment_timeout_minutes: 15
```

### 7.3 Build Monitor Agent

```yaml
apiVersion: v1
kind: Agent
metadata:
  name: build-monitor
  description: Monitors CI builds and retrieves test results

spec:
  model:
    provider: google
    model_id: gemini-2.5-flash

  tools:
    - github_run_list
    - github_run_get
    - github_artifacts_list
    - github_artifacts_download
    - github_run_logs

  config:
    github_token: "${GITHUB_TOKEN}"
    repositories:
      - owner: "myorg"
        repo: "backend-api"
      - owner: "myorg"
        repo: "frontend-app"

  system_prompt: |
    You monitor CI builds across multiple repositories.

    For each repository:
    1. List recent workflow runs (last 24 hours)
    2. Check for failures
    3. Download test result artifacts
    4. Parse test results and identify failing tests
    5. Report summary with failure details

    For failed builds:
    - Download logs for analysis
    - Identify common failure patterns
    - Suggest fixes based on error messages
    - Track failure frequency over time

  memory:
    type: persistent
    backend: sqlite
    config:
      database: "./build-history.db"

  triggers:
    - type: interval
      schedule: "0 */3 * * *"  # Every 3 hours
      action: "Analyze build health and report trends"
```

### 7.4 Release Automation Agent

```yaml
apiVersion: v1
kind: Agent
metadata:
  name: release-automator
  description: Automates release workflows and artifact publishing

spec:
  model:
    provider: google
    model_id: gemini-2.5-flash

  tools:
    - github_workflow_dispatch
    - github_run_get
    - github_artifacts_list
    - github_artifacts_download

  config:
    github_token: "${GITHUB_TOKEN}"
    owner: "agenticdevops"
    repo: "aof"
    release_workflow: "release.yml"

  system_prompt: |
    You automate the release process for AOF.

    Release Steps:
    1. Verify version tag format (e.g., v0.1.14)
    2. Trigger release.yml workflow with version input
    3. Monitor build progress (30+ minute timeout)
    4. Verify artifacts are generated:
       - Linux binary
       - macOS Intel binary
       - macOS Apple Silicon binary
       - Windows binary
       - SHA256 checksums
    5. Download artifacts for verification
    6. Publish release if all checks pass

    Quality Gates:
    - All artifacts must be present
    - Checksums must be valid
    - Binary sizes must be within expected ranges
    - Test suite must pass (100% success rate)

  variables:
    expected_artifacts:
      - "aof-linux-x86_64"
      - "aof-darwin-x86_64"
      - "aof-darwin-aarch64"
      - "aof-windows-x86_64.exe"
      - "checksums.txt"
    build_timeout_minutes: 45
```

### 7.5 Workflow Health Agent

```yaml
apiVersion: v1
kind: Agent
metadata:
  name: workflow-health
  description: Monitors workflow reliability and suggests improvements

spec:
  model:
    provider: google
    model_id: gemini-2.5-flash

  tools:
    - github_workflow_list
    - github_run_list
    - github_run_get

  config:
    github_token: "${GITHUB_TOKEN}"
    owner: "agenticdevops"
    repo: "aof"

  system_prompt: |
    You analyze workflow health and reliability metrics.

    Metrics to Track:
    - Success rate per workflow (last 7 days)
    - Average execution time
    - Frequency of runs
    - Cancellation rate
    - Timeout occurrences

    Analysis:
    1. Identify flaky workflows (inconsistent pass/fail)
    2. Detect performance regressions (increasing duration)
    3. Find underutilized workflows (rarely triggered)
    4. Spot bottlenecks (frequently queued)

    Reporting:
    - Generate weekly health reports
    - Alert on workflows below 90% success rate
    - Recommend optimizations for slow workflows
    - Suggest deprecation of unused workflows

  memory:
    type: persistent
    backend: sqlite
    config:
      database: "./workflow-metrics.db"

  triggers:
    - type: interval
      schedule: "0 9 * * 1"  # Monday 9 AM
      action: "Generate weekly workflow health report"
```

---

## 8. Implementation Checklist

### 8.1 Core Implementation

- [ ] Create `GitHubActionsTool` struct implementing `Tool` trait
- [ ] Implement `create_github_client()` helper with authentication
- [ ] Create individual tool structs for each operation
- [ ] Implement JSON schema for each tool's parameters
- [ ] Add proper timeout configuration (60s default)
- [ ] Implement comprehensive error handling

### 8.2 Tool Operations

- [ ] `github_workflow_list` - List workflows
- [ ] `github_workflow_dispatch` - Trigger workflow
- [ ] `github_run_list` - List runs with pagination
- [ ] `github_run_get` - Get run details
- [ ] `github_run_cancel` - Cancel run
- [ ] `github_run_rerun` - Rerun workflow
- [ ] `github_run_force_cancel` - Force cancel
- [ ] `github_artifacts_list` - List artifacts
- [ ] `github_artifacts_download` - Download artifact
- [ ] `github_run_logs` - Download logs

### 8.3 Supporting Features

- [ ] Pagination handling with Link header parsing
- [ ] Rate limit detection and reporting
- [ ] File download with progress tracking
- [ ] Response caching for repeated requests
- [ ] Retry logic for transient failures

### 8.4 Testing

- [ ] Unit tests for each tool operation
- [ ] Integration tests with mock GitHub API
- [ ] Error scenario tests (401, 403, 404, 429, etc.)
- [ ] Pagination tests
- [ ] Download tests with various file sizes
- [ ] Rate limit handling tests

### 8.5 Documentation

- [ ] Add module documentation with examples
- [ ] Document required token scopes
- [ ] Create user guide with agent YAML examples
- [ ] Add troubleshooting guide
- [ ] Document rate limits and best practices

### 8.6 Integration

- [ ] Register tools in `ToolRegistry`
- [ ] Add feature flag for GitHub Actions tools
- [ ] Update `Cargo.toml` dependencies
- [ ] Add example agent configurations
- [ ] Create CLI examples for testing

---

## 9. Security Considerations

### 9.1 Token Management

- **Never log tokens**: Sanitize logs to prevent token exposure
- **Environment variables**: Store tokens in `GITHUB_TOKEN` env var
- **Scope validation**: Verify token has required scopes before operations
- **Rotation**: Support token rotation without agent restart

### 9.2 Input Validation

- **Repository names**: Validate owner/repo format (alphanumeric, hyphens, underscores)
- **Workflow IDs**: Accept numeric IDs or .yml/.yaml filenames only
- **Ref validation**: Ensure refs don't contain path traversal attempts
- **Input sanitization**: Escape special characters in workflow inputs

### 9.3 Rate Limit Protection

- **Track limits**: Monitor `X-RateLimit-*` headers
- **Backoff**: Implement exponential backoff on rate limit errors
- **Caching**: Cache workflow lists and metadata to reduce API calls
- **Batch operations**: Group related API calls when possible

### 9.4 Artifact Security

- **Path validation**: Prevent directory traversal in download paths
- **Size limits**: Reject artifacts exceeding reasonable size (e.g., 1GB)
- **Checksum verification**: Validate SHA256 checksums if provided
- **Cleanup**: Remove downloaded artifacts after processing

---

## 10. Performance Optimization

### 10.1 Caching Strategy

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct ResponseCache {
    cache: HashMap<String, (serde_json::Value, Instant)>,
    ttl: Duration,
}

impl ResponseCache {
    fn new(ttl_secs: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.cache.get(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(value)
            } else {
                None
            }
        })
    }

    fn set(&mut self, key: String, value: serde_json::Value) {
        self.cache.insert(key, (value, Instant::now()));
    }
}
```

**Cacheable Operations**:
- Workflow lists (TTL: 5 minutes)
- Workflow definitions (TTL: 1 hour)
- Completed run details (TTL: 1 hour)

**Never Cache**:
- In-progress run status
- Artifact download URLs (they expire)
- Authentication responses

### 10.2 Parallel Requests

When monitoring multiple repositories:
```rust
use futures::future::join_all;

async fn get_runs_parallel(
    repos: Vec<(String, String)>,
    client: &reqwest::Client,
) -> Vec<AofResult<serde_json::Value>> {
    let futures: Vec<_> = repos.iter()
        .map(|(owner, repo)| get_runs(client, owner, repo))
        .collect();

    join_all(futures).await
}
```

### 10.3 Conditional Requests

Use ETags for efficient polling:
```rust
let mut headers = HeaderMap::new();
if let Some(etag) = last_etag {
    headers.insert("If-None-Match", HeaderValue::from_str(&etag)?);
}

let response = client.get(url).headers(headers).send().await?;

if response.status() == 304 {
    // Not modified, use cached data
    return Ok(cached_data);
}
```

---

## 11. Monitoring and Observability

### 11.1 Metrics to Track

- **API call count**: Total calls per operation type
- **Success rate**: Successful vs. failed calls
- **Response times**: P50, P95, P99 latencies
- **Rate limit utilization**: Remaining vs. total
- **Error rates**: By status code
- **Download sizes**: Artifact and log file sizes

### 11.2 Logging Best Practices

```rust
use tracing::{debug, info, warn, error};

// Start of operation
info!(
    owner = %owner,
    repo = %repo,
    workflow_id = %workflow_id,
    "Triggering workflow dispatch"
);

// API call details
debug!(
    method = "POST",
    url = %url,
    "Sending GitHub API request"
);

// Success
info!(
    run_id = run_id,
    duration_ms = start.elapsed().as_millis(),
    "Workflow dispatch successful"
);

// Errors (without exposing tokens)
error!(
    status = response.status().as_u16(),
    error = %sanitize_error(&error_msg),
    "GitHub API request failed"
);
```

### 11.3 Health Checks

Expose tool health status:
```rust
pub async fn health_check(token: &str) -> ToolResult {
    // Simple API call to verify connectivity and auth
    let client = create_github_client(token)?;
    let response = client.get("https://api.github.com/rate_limit").send().await?;

    if response.status() == 200 {
        let data: serde_json::Value = response.json().await?;
        Ok(ToolResult::success(serde_json::json!({
            "status": "healthy",
            "rate_limit": data.get("resources")
        })))
    } else {
        Ok(ToolResult::error("GitHub API unhealthy".into()))
    }
}
```

---

## 12. Future Enhancements

### 12.1 Planned Features

- **Webhook Integration**: Real-time workflow status via webhooks
- **Job-Level Control**: Cancel/rerun individual jobs within a run
- **Approval Workflows**: Approve pending deployments programmatically
- **Workflow Editing**: Modify workflow files via API
- **Secret Management**: Update repository secrets from agents
- **Environment Management**: Manage deployment environments
- **Cache Management**: Clear workflow caches

### 12.2 Advanced Capabilities

- **Workflow Templates**: Deploy workflow templates across repositories
- **Cross-Repo Orchestration**: Coordinate workflows across multiple repos
- **Dependency Management**: Track workflow dependencies and trigger chains
- **Cost Optimization**: Analyze and optimize GitHub Actions minutes usage
- **Compliance Checks**: Validate workflows against security policies

---

## Sources

- [GitHub REST API - Workflows](https://docs.github.com/en/rest/actions/workflows?apiVersion=2022-11-28)
- [GitHub REST API - Workflow Runs](https://docs.github.com/en/rest/actions/workflow-runs)
- [GitHub REST API - Artifacts](https://docs.github.com/en/rest/actions/artifacts)
- [GitHub REST API - Actions Overview](https://docs.github.com/en/rest/actions)
