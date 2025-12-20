---
id: github-integration
title: GitHub Integration Reference
sidebar_label: GitHub Integration
description: Complete API reference for GitHub platform integration with webhooks, events, and actions
keywords: [github, reference, api, webhook, events, actions]
---

# GitHub Integration Reference

Complete reference for GitHub platform integration with AOF.

## Platform Support Status

| Platform | Status | Notes |
|----------|--------|-------|
| **GitHub** | ✅ Stable | Fully tested, production-ready |
| **GitLab** | 🧪 Experimental | Implemented but untested - contributions welcome |
| **Bitbucket** | 🧪 Experimental | Implemented but untested - contributions welcome |

**Important:**
- This documentation focuses on **GitHub**, which is the only fully tested and supported Git platform at this time
- GitLab and Bitbucket adapters exist in the codebase with similar API patterns, but have not been validated in real-world usage
- If you're interested in using GitLab or Bitbucket, we encourage you to test and contribute improvements to these integrations

## Overview

AOF provides deep GitHub integration through webhook-based triggers and the GitHub API, enabling agents to automate pull request reviews, issue management, CI/CD checks, and deployment workflows.

**Key Capabilities:**
- Automatic PR code review with AI analysis
- Issue triage and auto-labeling
- CI/CD check runs and status updates
- Deployment automation on merge events
- Comment posting and review submission
- Multi-repository synchronization

## Architecture

GitHub integration consists of three components:

1. **GitHub Trigger** - Receives webhook events from GitHub
2. **GitHub Platform Adapter** - Parses events and manages API calls
3. **Agents/Flows** - Process events and take actions

```
GitHub Webhook → AOF Daemon → Trigger → Agent/Flow → GitHub API
```

---

## Configuration Reference

### Daemon Configuration

The DaemonConfig enables GitHub as a platform and points to resource directories. **Command routing and filtering are defined in Triggers**, not here.

Configure GitHub platform in `daemon.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-daemon

spec:
  server:
    port: 3000
    host: "0.0.0.0"
    cors: true
    timeout_secs: 60

  # Enable platforms - just enables the webhook endpoint
  # Command routing is defined in Triggers
  platforms:
    github:
      enabled: true
      token_env: GITHUB_TOKEN           # Environment variable name
      webhook_secret_env: GITHUB_WEBHOOK_SECRET
      bot_name: aofbot                  # Optional: for @mentions

  # Resource discovery
  triggers:
    directory: ./triggers/
    watch: true

  agents:
    directory: ./agents/

  fleets:
    directory: ./fleets/

  flows:
    directory: ./flows/
    enabled: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

### Platform Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable GitHub webhook endpoint (`/webhook/github`) |
| `token_env` | string | Yes | Environment variable name for GitHub token |
| `webhook_secret_env` | string | Yes | Environment variable name for webhook secret |
| `bot_name` | string | No | Bot name for @mentions (default: "aofbot") |

> **Note**: Event filtering, repository filtering, and command routing are configured in **Trigger** files, not in DaemonConfig. This separation keeps the daemon config minimal and allows per-trigger customization.

### Trigger Configuration

Create trigger in `triggers/github-pr.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-pr-events

spec:
  type: GitHub

  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}

    # Event filters
    github_events:
      - pull_request
      - pull_request_review
      - pull_request_review_comment

    # Repository filters
    repositories:
      - "myorg/api"
      - "myorg/web"

    # Branch filters (for push events)
    branches:
      - "main"
      - "develop"
      - "release/*"

  # Route to agent or flow
  commands:
    /review:
      agent: code-reviewer
      description: "Review pull request"

    /deploy:
      flow: deploy-flow
      description: "Deploy to production"

  default_agent: github-assistant
```

---

## Multi-Repository Configuration

AOF uses a **single DaemonConfig + multiple Triggers** architecture. This allows you to:
- Use one webhook URL for all repositories
- Configure different commands per repository or repository group
- Assign different teams/agents to different repos

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  GitHub Webhook (single URL: /webhook/github)                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  DaemonConfig (global)                                          │
│  - Enables GitHub platform                                       │
│  - Points to triggers/ directory                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│ frontend.yaml    │ │ backend.yaml     │ │ infra.yaml       │
│ repos: web-*, ui │ │ repos: api-*, svc│ │ repos: terraform │
│ fleet: frontend  │ │ fleet: backend   │ │ fleet: platform  │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

### Example: Per-Team Triggers

**Directory structure:**
```
config/
  daemon.yaml           # Global config

triggers/
  github-frontend.yaml  # Frontend team repos
  github-backend.yaml   # Backend team repos
  github-infra.yaml     # Platform team repos
  github-docs.yaml      # Documentation repos
```

**Frontend trigger** (`triggers/github-frontend.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: frontend-pr-bot
  labels:
    team: frontend

spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}

    # Only handle frontend repositories
    repositories:
      - "myorg/web-app"
      - "myorg/mobile-app"
      - "myorg/ui-components"

    github_events:
      - pull_request
      - issue_comment

  commands:
    pull_request.opened:
      fleet: frontend-review-fleet

    /review:
      fleet: frontend-review-fleet

    /test:
      agent: jest-runner

    /storybook:
      agent: storybook-deployer

  default_agent: frontend-helper
```

**Backend trigger** (`triggers/github-backend.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: backend-pr-bot
  labels:
    team: backend

spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}

    repositories:
      - "myorg/api-server"
      - "myorg/data-pipeline"
      - "myorg/auth-service"

    # Restrict to backend team members
    allowed_users:
      - "alice"
      - "bob"
      - "charlie"

  commands:
    pull_request.opened:
      fleet: backend-review-fleet

    /review:
      fleet: backend-review-fleet

    /deploy staging:
      flow: deploy-backend-staging

    /deploy production:
      flow: deploy-backend-prod
      params:
        require_approval: true

  default_agent: backend-helper
```

**Infrastructure trigger** (`triggers/github-infra.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: infra-pr-bot
  labels:
    team: platform

spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}

    repositories:
      - "myorg/terraform-*"      # Wildcard pattern
      - "myorg/k8s-manifests"
      - "myorg/helm-charts"

  commands:
    pull_request.opened:
      fleet: infra-review-fleet

    /plan:
      agent: terraform-planner

    /apply:
      flow: terraform-apply-flow
      params:
        require_approval: true
        approvers: ["platform-lead", "sre-oncall"]

  default_agent: infra-helper
```

### Webhook Routing

When a webhook arrives, AOF:
1. Parses the repository from the event payload
2. Matches against `repositories` patterns in each Trigger
3. Routes to the **first matching Trigger**
4. If no match, event is ignored (logged as unhandled)

**Pattern matching:**
- `"myorg/repo"` - Exact match
- `"myorg/*"` - All repos in org
- `"myorg/prefix-*"` - Repos starting with prefix

### Multi-Organization Support

> **Current Status**: AOF currently supports multiple organizations through multiple Triggers, but requires the same webhook secret for all.
>
> **Future Enhancement**: Native multi-org support with per-org secrets and GitHub App installation is tracked in [GitHub Issue #46](https://github.com/agenticdevops/aof/issues/46).

**Current approach** - Multiple triggers, same webhook secret:

```yaml
# triggers/org1-repos.yaml
spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    repositories:
      - "org1/*"

# triggers/org2-repos.yaml
spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}  # Same secret
    repositories:
      - "org2/*"
```

**Limitation**: All orgs must use the same webhook secret. This works but isn't ideal for enterprise multi-tenant scenarios.

**Future design** (Issue #46):
```yaml
# Future: Per-org configuration
platforms:
  github:
    organizations:
      - name: org1
        token_env: ORG1_GITHUB_TOKEN
        webhook_secret_env: ORG1_WEBHOOK_SECRET
      - name: org2
        token_env: ORG2_GITHUB_TOKEN
        webhook_secret_env: ORG2_WEBHOOK_SECRET
```

---

### Environment Variables

```bash
# Required
export GITHUB_TOKEN="ghp_xxxxx"                    # Personal Access Token
export GITHUB_WEBHOOK_SECRET="random-secret"       # Webhook HMAC secret

# Optional - for GitHub App
export GITHUB_APP_ID="123456"
export GITHUB_APP_PRIVATE_KEY_PATH="/path/to/key.pem"
export GITHUB_INSTALLATION_ID="12345678"

# LLM provider
export GOOGLE_API_KEY="xxxxx"
# OR
export ANTHROPIC_API_KEY="xxxxx"
```

---

## Event Reference

### Supported Events

AOF supports all major GitHub webhook events:

| Event Type | Description | Common Actions |
|------------|-------------|----------------|
| `pull_request` | PR lifecycle events | `opened`, `synchronize`, `closed`, `reopened` |
| `pull_request_review` | PR review submitted | `submitted`, `edited`, `dismissed` |
| `pull_request_review_comment` | Comment on PR diff | `created`, `edited`, `deleted` |
| `push` | Code pushed to branch | N/A (no action field) |
| `issues` | Issue lifecycle | `opened`, `edited`, `closed`, `reopened`, `labeled` |
| `issue_comment` | Comment on issue/PR | `created`, `edited`, `deleted` |
| `workflow_run` | GitHub Actions workflow | `completed`, `requested`, `in_progress` |
| `check_run` | Check run updated | `created`, `completed`, `rerequested` |
| `check_suite` | Check suite updated | `completed`, `requested` |
| `release` | Release published | `published`, `created`, `edited` |
| `create` | Branch/tag created | N/A |
| `delete` | Branch/tag deleted | N/A |
| `fork` | Repository forked | N/A |
| `star` | Repository starred | `created`, `deleted` |
| `watch` | Repository watched | `started` |

### Event Actions

#### pull_request Actions

| Action | Description |
|--------|-------------|
| `opened` | New PR created |
| `synchronize` | PR updated with new commits |
| `reopened` | Closed PR reopened |
| `closed` | PR closed (check `merged` field) |
| `edited` | PR title/body edited |
| `labeled` / `unlabeled` | Labels changed |
| `assigned` / `unassigned` | Assignees changed |
| `review_requested` | Reviewers requested |
| `ready_for_review` | Draft converted to ready |
| `converted_to_draft` | Ready converted to draft |

#### issues Actions

| Action | Description |
|--------|-------------|
| `opened` | New issue created |
| `edited` | Issue title/body edited |
| `closed` / `reopened` | State changed |
| `labeled` / `unlabeled` | Labels changed |
| `assigned` / `unassigned` | Assignees changed |
| `milestoned` / `demilestoned` | Milestone changed |
| `pinned` / `unpinned` | Pinned status changed |
| `locked` / `unlocked` | Conversation locked/unlocked |

#### issue_comment / pull_request_review_comment Actions

| Action | Description |
|--------|-------------|
| `created` | New comment posted |
| `edited` | Comment edited |
| `deleted` | Comment deleted |

#### workflow_run Actions

| Action | Description |
|--------|-------------|
| `requested` | Workflow queued |
| `in_progress` | Workflow running |
| `completed` | Workflow finished (check `conclusion` field) |

**Workflow Conclusions:**
- `success` - All jobs passed
- `failure` - One or more jobs failed
- `cancelled` - Workflow cancelled
- `neutral` - Neutral result
- `skipped` - Workflow skipped
- `timed_out` - Workflow timed out

---

## Context Variables

When an agent is triggered by a GitHub event, these variables are available in `metadata`:

### Common Variables

| Variable | Type | Description |
|----------|------|-------------|
| `event_type` | string | Event type (pull_request, push, etc.) |
| `action` | string | Event action (opened, closed, etc.) |
| `repo_id` | integer | Repository ID |
| `repo_full_name` | string | Repository (owner/repo) |
| `repo_private` | boolean | Is repository private |
| `sender_id` | integer | User ID who triggered event |
| `sender_login` | string | Username who triggered event |

### Pull Request Variables

Available when `event_type` = `pull_request`:

| Variable | Type | Description |
|----------|------|-------------|
| `pr_number` | integer | PR number |
| `pr_title` | string | PR title |
| `pr_state` | string | open, closed |
| `pr_draft` | boolean | Is draft PR |
| `pr_base_ref` | string | Base branch name |
| `pr_head_ref` | string | Head branch name |
| `pr_head_sha` | string | Head commit SHA |
| `pr_additions` | integer | Lines added |
| `pr_deletions` | integer | Lines deleted |
| `pr_changed_files` | integer | Files changed |
| `pr_html_url` | string | PR web URL |

### Issue Variables

Available when `event_type` = `issues`:

| Variable | Type | Description |
|----------|------|-------------|
| `issue_number` | integer | Issue number |
| `issue_title` | string | Issue title |
| `issue_state` | string | open, closed |
| `issue_html_url` | string | Issue web URL |

### Push Variables

Available when `event_type` = `push`:

| Variable | Type | Description |
|----------|------|-------------|
| `ref` | string | Git ref (refs/heads/main) |
| `before_sha` | string | Previous commit SHA |
| `after_sha` | string | New commit SHA |
| `commit_count` | integer | Number of commits pushed |

### Usage in Agent Instructions

```yaml
spec:
  instructions: |
    You are reviewing PR #{{ metadata.pr_number }}.

    Repository: {{ metadata.repo_full_name }}
    Base branch: {{ metadata.pr_base_ref }}
    Head branch: {{ metadata.pr_head_ref }}
    Commit: {{ metadata.pr_head_sha }}

    Changes: +{{ metadata.pr_additions }} -{{ metadata.pr_deletions }}
    Files changed: {{ metadata.pr_changed_files }}
```

---

## API Actions

Actions agents can perform on GitHub via built-in tools.

### Post Comment

Post a comment on an issue or PR.

```yaml
# In agent instructions
tools:
  - github_post_comment

# Usage
Post a comment to PR #{{ metadata.pr_number }}:
"✅ Code review passed. Looks good!"
```

**Rust API:**
```rust
platform.post_comment("owner", "repo", 42, "✅ Review complete").await?;
```

**Parameters:**
- `owner` (string) - Repository owner
- `repo` (string) - Repository name
- `issue_number` (integer) - Issue or PR number
- `body` (string) - Comment text (supports Markdown)

**Returns:** Comment ID

### Post Review

Submit a PR review with approval/changes/comments.

```yaml
# Usage in agent
Submit a PR review with status "APPROVE" and body "LGTM! ✅"
```

**Rust API:**
```rust
platform.post_review("owner", "repo", 42, "LGTM! ✅", "APPROVE").await?;
```

**Parameters:**
- `owner` (string) - Repository owner
- `repo` (string) - Repository name
- `pr_number` (integer) - Pull request number
- `body` (string) - Review body (supports Markdown)
- `event` (string) - Review type

**Review Types:**
| Event | Description |
|-------|-------------|
| `APPROVE` | Approve the PR |
| `REQUEST_CHANGES` | Request changes before merge |
| `COMMENT` | Comment without explicit approval |

**Returns:** Review ID

### Create Check Run

Create or update a CI/CD check run.

```yaml
# Usage
Create check run "AI Code Review" for commit {{ metadata.pr_head_sha }}
Status: "in_progress"
```

**Rust API:**
```rust
use CheckRunOutput;

let output = CheckRunOutput {
    title: "Review Complete".to_string(),
    summary: "All checks passed".to_string(),
    text: Some("Detailed findings...".to_string()),
};

platform.create_check_run(
    "owner",
    "repo",
    "abc123",  // head_sha
    "AI Code Review",
    "completed",
    Some("success"),
    Some(output)
).await?;
```

**Parameters:**
- `owner` (string) - Repository owner
- `repo` (string) - Repository name
- `head_sha` (string) - Commit SHA
- `name` (string) - Check run name
- `status` (string) - queued, in_progress, completed
- `conclusion` (string, optional) - Required if status=completed
- `output` (object, optional) - Detailed output

**Status Values:**
- `queued` - Check is queued
- `in_progress` - Check is running
- `completed` - Check finished

**Conclusion Values (when status=completed):**
- `success` - Check passed
- `failure` - Check failed
- `neutral` - Neutral result
- `cancelled` - Check cancelled
- `skipped` - Check skipped
- `timed_out` - Check timed out
- `action_required` - Action required

**Returns:** Check run ID

### Add Labels

Add one or more labels to an issue or PR.

```rust
platform.add_labels("owner", "repo", 42, &["bug", "priority:high"]).await?;
```

**Parameters:**
- `owner` (string) - Repository owner
- `repo` (string) - Repository name
- `issue_number` (integer) - Issue or PR number
- `labels` (array of strings) - Label names

### Remove Label

Remove a label from an issue or PR.

```rust
platform.remove_label("owner", "repo", 42, "wip").await?;
```

**Parameters:**
- `owner` (string) - Repository owner
- `repo` (string) - Repository name
- `issue_number` (integer) - Issue or PR number
- `label` (string) - Label name

---

## Security

### Webhook Verification

AOF automatically validates webhook signatures using HMAC-SHA256.

**How it works:**
1. GitHub sends signature in `X-Hub-Signature-256` header
2. AOF computes HMAC using `webhook_secret`
3. Signatures must match or request is rejected

**Configuration:**
```yaml
spec:
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}  # Required
```

**Generate secure secret:**
```bash
openssl rand -hex 32
```

### Token Permissions

**Required permissions for Personal Access Token:**

| Permission | Access | Use Case |
|------------|--------|----------|
| Contents | Read & Write | Read code, create commits |
| Pull requests | Read & Write | Review PRs, post reviews |
| Issues | Read & Write | Manage issues, post comments |
| Checks | Read & Write | Create check runs |
| Workflows | Read & Write | Trigger workflows |
| Metadata | Read | Access repository metadata |

**For GitHub App:**

Create app with same permissions, generate private key, and install on repositories.

### Repository Filtering

Whitelist repositories for security:

```yaml
allowed_repos:
  - "myorg/repo1"           # Specific repo
  - "myorg/*"               # All repos in org
  - "owner/private-repo"    # Private repos
```

### User Filtering

Restrict who can trigger actions by **GitHub username** (login):

```yaml
# In Trigger config section
config:
  allowed_users:
    - "alice"           # GitHub username
    - "bob"             # GitHub username
    - "ci-bot"          # Bot accounts work too
    - "dependabot[bot]" # GitHub Apps use [bot] suffix
```

Events from users not in this list will be ignored.

> **Current limitation**: User filtering is based on GitHub usernames only. Team-based or role-based authorization (e.g., "all members of @myorg/sre-team") is not yet implemented but is planned for a future release.

---

## Rate Limiting

### GitHub API Limits

| Type | Limit | Reset |
|------|-------|-------|
| Authenticated (PAT) | 5,000 requests/hour | Hourly |
| GitHub App | 15,000 requests/hour | Hourly |
| Search API | 30 requests/minute | Per minute |

### Mitigation Strategies

1. **Use conditional requests** - Cache responses with ETags
2. **Batch operations** - Combine multiple changes
3. **Use GraphQL** - Fetch exactly what you need
4. **Implement backoff** - Retry with exponential backoff
5. **Use GitHub App** - Higher rate limits than PAT

**Check rate limit:**
```bash
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/rate_limit
```

**AOF handles retries automatically** when rate limit is hit.

---

## Examples

### Quick Reference

#### Auto-Review PR

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pr-review

spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    github_events:
      - pull_request

  default_agent: code-reviewer
```

```yaml
# agents/code-reviewer.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: code-reviewer

spec:
  model: google:gemini-2.5-flash

  instructions: |
    Review PR #{{ metadata.pr_number }} in {{ metadata.repo_full_name }}.

    1. Analyze the diff for security issues
    2. Check code quality and style
    3. Verify test coverage
    4. Post inline comments on issues
    5. Submit review with APPROVE or REQUEST_CHANGES

  tools:
    - github_post_comment
    - github_post_review
```

#### Auto-Label Issues

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: issue-triage

spec:
  type: GitHub
  config:
    github_events:
      - issues

  default_agent: issue-triager
```

```yaml
# agents/issue-triager.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: issue-triager

spec:
  model: google:gemini-2.5-flash

  instructions: |
    Analyze issue #{{ metadata.issue_number }} and:

    1. Add appropriate labels: bug, enhancement, documentation, etc.
    2. Classify priority: low, medium, high, critical
    3. Identify component: api, frontend, database, etc.
    4. Post a helpful comment if info is missing

  tools:
    - github_add_labels
    - github_post_comment
```

#### Deploy on Merge

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: production-deploy

spec:
  type: GitHub
  config:
    github_events:
      - pull_request
    branches:
      - main

  commands:
    /deploy:
      flow: deploy-flow
```

#### Create Check Run

```yaml
spec:
  instructions: |
    Create check run for commit {{ metadata.pr_head_sha }}:

    Name: "AI Security Scan"
    Status: "in_progress"

    # Run security analysis...

    Update check run to "completed" with conclusion "success"
    Output:
      Title: "Security Scan Passed"
      Summary: "No vulnerabilities found"
```

---

## Webhook Setup

### 1. Create Webhook in GitHub

**Repository Settings:**
1. Go to Settings → Webhooks → Add webhook
2. Configure:
   - **Payload URL**: `https://your-domain.com/webhook/github`
   - **Content type**: `application/json`
   - **Secret**: Your `GITHUB_WEBHOOK_SECRET` value
   - **SSL verification**: Enable
   - **Events**: Select events or "Send me everything"

**Organization Settings:**
1. Go to Organization Settings → Webhooks
2. Same configuration as above
3. Applies to all repos in organization

### 2. Expose Endpoint

**For production:**
Deploy with public HTTPS endpoint:
```bash
# HTTPS required
https://your-domain.com/webhook/github
```

**For local testing:**
Use a tunnel service:

```bash
# Option 1: Cloudflared (no signup)
brew install cloudflared
cloudflared tunnel --url http://localhost:3000

# Option 2: ngrok (free account)
ngrok http 3000
```

Use tunnel URL as webhook payload URL.

### 3. Verify Webhook

1. Send test event in GitHub webhook settings
2. Check "Recent Deliveries" for response
3. Verify AOF logs show received event

```bash
# Check logs
tail -f /var/log/aof/daemon.log

# Look for:
# INFO  GitHub webhook received: pull_request
# INFO  Posted comment 123456 to owner/repo#42
```

---

## Troubleshooting

### Webhook Not Triggering

**Symptoms:** GitHub webhook shows success but agent doesn't run

**Solutions:**
1. Check webhook delivery logs in GitHub
2. Verify webhook secret matches `GITHUB_WEBHOOK_SECRET`
3. Check firewall allows GitHub IPs: `140.82.112.0/20`, `185.199.108.0/22`
4. Verify daemon is running: `aofctl serve --config daemon.yaml`
5. Check logs: `RUST_LOG=debug aofctl serve`

### Signature Verification Failed

**Symptoms:** `Invalid GitHub signature` in logs

**Solutions:**
1. Verify `GITHUB_WEBHOOK_SECRET` matches webhook configuration
2. Check webhook content type is `application/json`
3. Ensure no proxy modifying request body

### Cannot Post Comments

**Symptoms:** Agent runs but comments don't appear

**Solutions:**
1. Verify `GITHUB_TOKEN` has correct permissions
2. Check token has `pull_requests:write` and `issues:write`
3. Test token manually:
   ```bash
   curl -H "Authorization: Bearer $GITHUB_TOKEN" \
     https://api.github.com/user
   ```
4. Verify `enable_comments: true` in daemon config

### Rate Limit Exceeded

**Symptoms:** `API rate limit exceeded` error

**Solutions:**
1. Check rate limit status:
   ```bash
   curl -H "Authorization: Bearer $GITHUB_TOKEN" \
     https://api.github.com/rate_limit
   ```
2. Reduce event frequency (use action filters)
3. Use GitHub App for higher limits (15,000/hour)
4. Implement request caching

### Check Run Not Updating

**Symptoms:** Check run shows "queued" forever

**Solutions:**
1. Verify token has `checks:write` permission
2. Check `head_sha` matches actual commit
3. Verify conclusion is valid: success, failure, neutral, etc.
4. Use correct check run ID for updates

---

## Best Practices

### 1. Filter Events Wisely

**❌ Too broad:**
```yaml
github_events:
  - pull_request  # All PR actions
  - issues        # All issue actions
  - push          # All pushes
```

**✅ Specific:**
```yaml
github_events:
  - pull_request
actions:
  - opened
  - synchronize
branches:
  - main
  - develop
```

### 2. Use Repository Filters

**For production:**
```yaml
allowed_repos:
  - "myorg/production-api"
  - "myorg/production-web"
```

Prevents accidental automation on test repos.

### 3. Implement Approval Gates

For destructive actions:
```yaml
spec:
  instructions: |
    Before deploying to production:

    1. Check PR has required approvals (2+)
    2. Verify all checks passed
    3. Ensure no "do-not-merge" labels
    4. Check deployment window (weekdays 9-5)
    5. Post deployment plan for human approval
```

### 4. Use Check Runs for CI/CD

Instead of posting comments, use check runs:
- Show up in PR checks section
- Block merge if failed
- Provide structured output
- Better UX than comment spam

### 5. Cache GitHub Data

Reduce API calls by caching:
```yaml
spec:
  memory: "File:./github-cache.json:1000"
```

---

## Enterprise Scaling

### Current Architecture

The AOF daemon is a **single-process server** that:
- Receives webhooks on a single HTTP endpoint
- Loads all Triggers from the configured directory
- Routes events to matching Triggers
- Executes agents/fleets/flows for each event

**Current capacity** (single instance):
- Handles ~100-500 webhook events/minute (depending on agent complexity)
- Loads 100s of Trigger files efficiently
- Suitable for small-to-medium teams (up to ~50 repositories)

### Scaling Challenges

For enterprise scale (1000s of repos, multiple orgs, high throughput), current limitations include:

| Challenge | Current State | Impact |
|-----------|--------------|--------|
| Single process | One daemon handles all events | Single point of failure |
| In-memory triggers | Triggers loaded at startup | Memory grows with trigger count |
| No queue | Synchronous webhook processing | Backpressure under load |
| Shared credentials | One token for all repos | No org isolation |

### Enterprise Deployment (Kubernetes)

> **Future Enhancement**: Native horizontal scaling with Redis/NATS message queue is tracked in [GitHub Issue #47](https://github.com/agenticdevops/aof/issues/47).

**Recommended architecture for enterprise:**

```
                    ┌─────────────────┐
                    │   Ingress/LB    │
                    │  (nginx/traefik)│
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ AOF Daemon  │ │ AOF Daemon  │ │ AOF Daemon  │
     │ (replica 1) │ │ (replica 2) │ │ (replica 3) │
     └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                   ┌─────────────────┐
                   │   Redis/NATS    │
                   │  (message queue)│
                   └────────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ AOF Worker  │ │ AOF Worker  │ │ AOF Worker  │
     │ (executor)  │ │ (executor)  │ │ (executor)  │
     └─────────────┘ └─────────────┘ └─────────────┘
```

**Current workaround** - Multiple daemon instances with trigger sharding:

```yaml
# Kubernetes Deployment - Shard by org/team
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-daemon-frontend
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: aof
          image: ghcr.io/agenticdevops/aof:latest
          args:
            - serve
            - --config=/config/daemon.yaml
          volumeMounts:
            - name: triggers
              mountPath: /triggers
              subPath: frontend  # Only frontend triggers
```

```yaml
# Separate deployment for backend team
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-daemon-backend
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: aof
          volumeMounts:
            - name: triggers
              mountPath: /triggers
              subPath: backend  # Only backend triggers
```

**Ingress routing by org:**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: aof-ingress
  annotations:
    nginx.ingress.kubernetes.io/use-regex: "true"
spec:
  rules:
    - host: aof.example.com
      http:
        paths:
          # Route by X-GitHub-Delivery header or body parsing
          - path: /webhook/github
            pathType: Prefix
            backend:
              service:
                name: aof-daemon-router
                port:
                  number: 3000
```

### Scaling Recommendations

| Scale | Repos | Events/min | Recommended Setup |
|-------|-------|------------|-------------------|
| Small | 1-50 | 1-100 | Single daemon, single node |
| Medium | 50-200 | 100-500 | 2-3 daemon replicas, load balanced |
| Large | 200-1000 | 500-2000 | Sharded by org/team, separate deployments |
| Enterprise | 1000+ | 2000+ | Message queue + worker pools (Issue #47) |

### High Availability

**Current approach:**
```yaml
# Run multiple replicas behind load balancer
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
```

**Health checks:**
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```

### Configuration Management

**GitOps with triggers in ConfigMap:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aof-triggers
data:
  github-frontend.yaml: |
    apiVersion: aof.dev/v1
    kind: Trigger
    metadata:
      name: frontend-pr-bot
    spec:
      type: GitHub
      config:
        repositories: ["myorg/web-*"]
      commands:
        /review:
          fleet: frontend-review
```

**Hot-reload triggers:**
```yaml
spec:
  triggers:
    directory: /triggers
    watch: true  # Watches for ConfigMap updates
```

### Future Roadmap

See these issues for enterprise features:

- [#45 - Team/role-based authorization](https://github.com/agenticdevops/aof/issues/45)
- [#46 - Multi-organization support](https://github.com/agenticdevops/aof/issues/46)
- [#47 - Horizontal scaling with message queue](https://github.com/agenticdevops/aof/issues/47)

---

## See Also

- [Trigger Specification](./trigger-spec.md) - Complete trigger reference
- [Agent Specification](./agent-spec.md) - Agent configuration
- [Daemon Configuration](./daemon-config.md) - Daemon setup
- [GitHub Automation Tutorial](../tutorials/github-automation.md) - Step-by-step guide
- [GitHub Integration Concepts](../concepts/github-integration.md) - High-level overview
