---
sidebar_position: 10
sidebar_label: GitLab
---

# GitLab MCP Server

Interact with GitLab repositories, merge requests, and CI/CD.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-gitlab` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/gitlab) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-gitlab
```

## Configuration

```yaml
mcp_servers:
  - name: gitlab
    command: npx
    args: ["-y", "@modelcontextprotocol/server-gitlab"]
    env:
      GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
      GITLAB_API_URL: https://gitlab.com/api/v4
```

### Self-Hosted GitLab

```yaml
mcp_servers:
  - name: gitlab
    command: npx
    args: ["-y", "@modelcontextprotocol/server-gitlab"]
    env:
      GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
      GITLAB_API_URL: https://gitlab.mycompany.com/api/v4
```

### Required Token Scopes

- `api` - Full API access
- `read_repository` - Read repository content
- `write_repository` - Push code (optional)

## Tools

### create_or_update_file

Create or update a file in a repository.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `file_path` (string, required): Path to file
- `content` (string, required): File content
- `commit_message` (string, required): Commit message
- `branch` (string, required): Branch name
- `previous_path` (string, optional): For renaming files

### search_repositories

Search for projects.

**Parameters**:
- `query` (string, required): Search query

**Example**:
```json
{
  "tool": "search_repositories",
  "arguments": {
    "query": "kubernetes operator"
  }
}
```

### get_file_contents

Get file or directory contents.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `file_path` (string, required): Path to file
- `ref` (string, optional): Branch or commit

### create_issue

Create a new issue.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `title` (string, required): Issue title
- `description` (string, optional): Issue description
- `labels` (array, optional): Labels to apply
- `assignee_ids` (array, optional): Assignee IDs

### list_issues

List project issues.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `state` (string, optional): opened, closed, all
- `labels` (string, optional): Comma-separated labels

### create_merge_request

Create a merge request.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `title` (string, required): MR title
- `description` (string, optional): MR description
- `source_branch` (string, required): Source branch
- `target_branch` (string, required): Target branch

### list_merge_requests

List merge requests.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `state` (string, optional): opened, closed, merged, all

### get_merge_request

Get merge request details.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `merge_request_iid` (number, required): MR IID

### get_merge_request_diffs

Get merge request changes.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `merge_request_iid` (number, required): MR IID

### create_merge_request_note

Add a comment to a merge request.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `merge_request_iid` (number, required): MR IID
- `body` (string, required): Comment body

### fork_repository

Fork a project.

**Parameters**:
- `project_id` (string, required): Project ID or path

### create_branch

Create a new branch.

**Parameters**:
- `project_id` (string, required): Project ID or path
- `branch` (string, required): New branch name
- `ref` (string, optional): Source ref (default: main)

## Use Cases

### MR Review Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: mr-reviewer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: gitlab
      command: npx
      args: ["-y", "@modelcontextprotocol/server-gitlab"]
      env:
        GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
  system_prompt: |
    You review GitLab merge requests:
    1. Get MR details and diff
    2. Analyze code changes
    3. Check for bugs and security issues
    4. Add review comments
    5. Suggest improvements
```

### Issue Manager Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: issue-manager
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: gitlab
      command: npx
      args: ["-y", "@modelcontextprotocol/server-gitlab"]
      env:
        GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
  system_prompt: |
    You manage GitLab issues:
    - Create issues from user requests
    - Add appropriate labels
    - Assign to team members
    - Link related issues
    - Track progress
```

### CI/CD Monitor Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: cicd-monitor
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: gitlab
      command: npx
      args: ["-y", "@modelcontextprotocol/server-gitlab"]
      env:
        GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
  system_prompt: |
    You monitor GitLab CI/CD pipelines:
    - Check pipeline status
    - Identify failing jobs
    - Analyze error logs
    - Suggest fixes
    - Retry failed jobs when appropriate
```

### Code Migration Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: code-migrator
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: gitlab
      command: npx
      args: ["-y", "@modelcontextprotocol/server-gitlab"]
      env:
        GITLAB_PERSONAL_ACCESS_TOKEN: ${GITLAB_TOKEN}
  system_prompt: |
    You assist with code migrations:
    - Fork repositories
    - Create migration branches
    - Update dependencies
    - Create merge requests
    - Document changes
```

## Project ID Formats

GitLab accepts project IDs in multiple formats:

```yaml
# Numeric ID
project_id: "12345"

# URL-encoded path
project_id: "mygroup%2Fmyproject"

# Path with namespace
project_id: "mygroup/myproject"
```

## Merge Request Workflow

```javascript
// 1. Create branch
create_branch({
  project_id: "mygroup/myproject",
  branch: "feature/new-feature",
  ref: "main"
})

// 2. Create/update files
create_or_update_file({
  project_id: "mygroup/myproject",
  file_path: "src/feature.js",
  content: "// New feature code",
  commit_message: "Add new feature",
  branch: "feature/new-feature"
})

// 3. Create merge request
create_merge_request({
  project_id: "mygroup/myproject",
  title: "Add new feature",
  source_branch: "feature/new-feature",
  target_branch: "main",
  description: "This MR adds..."
})
```

## Rate Limits

GitLab API rate limits:
- **Authenticated**: 2,000 requests/minute
- **Search**: 10 requests/minute

## Troubleshooting

### 401 Unauthorized

Check your token:
```bash
curl -H "PRIVATE-TOKEN: YOUR_TOKEN" \
  "https://gitlab.com/api/v4/user"
```

### 404 Not Found

Verify project access:
- Check project visibility
- Verify token has access to project
- Use correct project ID format

### 403 Forbidden

Token may lack required scopes:
- Go to Settings > Access Tokens
- Regenerate with required scopes

### Self-Hosted SSL Issues

For self-signed certificates:
```yaml
env:
  NODE_TLS_REJECT_UNAUTHORIZED: "0"  # Not recommended for production
```
