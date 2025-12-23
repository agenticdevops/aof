---
sidebar_position: 3
sidebar_label: GitHub
---

# GitHub MCP Server

Interact with GitHub repositories, issues, pull requests, and more.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-github` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/github) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-github
```

## Configuration

```yaml
mcp_servers:
  - name: github
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}
```

### Required Permissions

Your GitHub token needs these scopes:
- `repo` - Full repository access
- `read:org` - Read organization data (optional)
- `read:user` - Read user data (optional)

## Tools

### create_or_update_file

Create or update a file in a repository.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `path` (string): File path
- `content` (string): File content
- `message` (string): Commit message
- `branch` (string): Branch name
- `sha` (string, optional): SHA of file being replaced (for updates)

### search_repositories

Search for repositories.

**Parameters**:
- `query` (string): Search query

**Example**:
```json
{
  "tool": "search_repositories",
  "arguments": {
    "query": "kubernetes operator language:rust"
  }
}
```

### create_repository

Create a new repository.

**Parameters**:
- `name` (string): Repository name
- `description` (string, optional): Description
- `private` (boolean, optional): Private repository

### get_file_contents

Get contents of a file or directory.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `path` (string): File path
- `branch` (string, optional): Branch name

### push_files

Push multiple files in a single commit.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `branch` (string): Branch name
- `files` (array): Files to push
- `message` (string): Commit message

### create_issue

Create a new issue.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `title` (string): Issue title
- `body` (string, optional): Issue body
- `labels` (array, optional): Labels
- `assignees` (array, optional): Assignees

### create_pull_request

Create a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `title` (string): PR title
- `body` (string, optional): PR body
- `head` (string): Head branch
- `base` (string): Base branch

### fork_repository

Fork a repository.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name

### create_branch

Create a new branch.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `branch` (string): New branch name
- `from_branch` (string, optional): Source branch

### list_commits

List commits in a repository.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `page` (number, optional): Page number
- `per_page` (number, optional): Results per page
- `sha` (string, optional): Branch/commit SHA

### list_issues

List issues in a repository.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `state` (string, optional): open, closed, all
- `labels` (string, optional): Comma-separated labels
- `sort` (string, optional): created, updated, comments
- `direction` (string, optional): asc, desc

### update_issue

Update an existing issue.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `issue_number` (number): Issue number
- `title` (string, optional): New title
- `body` (string, optional): New body
- `state` (string, optional): open, closed
- `labels` (array, optional): Labels

### add_issue_comment

Add a comment to an issue.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `issue_number` (number): Issue number
- `body` (string): Comment body

### search_code

Search code across repositories.

**Parameters**:
- `query` (string): Search query

**Example**:
```json
{
  "tool": "search_code",
  "arguments": {
    "query": "filename:Dockerfile FROM node"
  }
}
```

### search_issues

Search issues and pull requests.

**Parameters**:
- `query` (string): Search query

### search_users

Search GitHub users.

**Parameters**:
- `query` (string): Search query

### get_issue

Get details of a specific issue.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `issue_number` (number): Issue number

### get_pull_request

Get details of a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `pull_number` (number): PR number

### list_pull_requests

List pull requests.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `state` (string, optional): open, closed, all
- `head` (string, optional): Filter by head branch
- `base` (string, optional): Filter by base branch

### create_pull_request_review

Create a review on a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `pull_number` (number): PR number
- `body` (string, optional): Review body
- `event` (string): APPROVE, REQUEST_CHANGES, COMMENT
- `comments` (array, optional): Line comments

### merge_pull_request

Merge a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `pull_number` (number): PR number
- `commit_title` (string, optional): Merge commit title
- `commit_message` (string, optional): Merge commit message
- `merge_method` (string, optional): merge, squash, rebase

### get_pull_request_files

Get files changed in a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `pull_number` (number): PR number

### get_pull_request_diff

Get the diff of a pull request.

**Parameters**:
- `owner` (string): Repository owner
- `repo` (string): Repository name
- `pull_number` (number): PR number

## Use Cases

### PR Review Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: pr-reviewer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: github
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}
  system_prompt: |
    You are a code review agent. When given a PR:
    1. Use get_pull_request_diff to see changes
    2. Use get_pull_request_files to list changed files
    3. Analyze for bugs, security issues, and style
    4. Use create_pull_request_review to submit review
```

### Issue Triage Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: issue-triage
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: github
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}
  system_prompt: |
    You triage new GitHub issues:
    1. Use list_issues to get new issues
    2. Analyze each issue for priority and category
    3. Use update_issue to add labels
    4. Use add_issue_comment to acknowledge
```

### Repository Analyzer

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: repo-analyzer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: github
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}
  system_prompt: |
    You analyze GitHub repositories for:
    - Code quality patterns
    - Documentation completeness
    - CI/CD configuration
    - Security practices

    Use get_file_contents to read key files like
    README.md, .github/workflows/, Dockerfile, etc.
```

## Rate Limits

GitHub API has rate limits:
- **Authenticated**: 5,000 requests/hour
- **Search API**: 30 requests/minute

The server handles rate limiting automatically with retries.

## Troubleshooting

### Authentication Failed

Verify your token:
```bash
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user
```

### Permission Denied

Check token scopes at: https://github.com/settings/tokens

### Rate Limited

Wait for rate limit reset or use a token with higher limits.
