# Bitbucket Platform Integration

## Overview

The Bitbucket platform adapter provides webhook-based integration with Bitbucket Cloud and Server/Data Center, enabling AOF agents to respond to repository events, manage pull requests, and interact with the Bitbucket API.

## Features

### Supported Events

**Pull Request Events:**
- `pullrequest:created` - New pull request opened
- `pullrequest:updated` - Pull request updated
- `pullrequest:approved` - Pull request approved
- `pullrequest:unapproved` - Approval removed
- `pullrequest:fulfilled` - Pull request merged
- `pullrequest:rejected` - Pull request declined
- `pullrequest:comment_created` - Comment added to PR
- `pullrequest:comment_updated` - Comment edited
- `pullrequest:comment_deleted` - Comment removed

**Repository Events:**
- `repo:push` - Code pushed to repository
- `repo:fork` - Repository forked
- `repo:updated` - Repository settings changed
- `repo:commit_comment_created` - Comment on commit
- `repo:commit_status_created` - Build status created
- `repo:commit_status_updated` - Build status updated

**Issue Events:**
- `issue:created` - New issue created
- `issue:updated` - Issue updated
- `issue:comment_created` - Comment on issue

**Build Events:**
- `build:status_created` - Build status created
- `build:status_updated` - Build status updated

### API Methods

The Bitbucket platform provides the following API methods:

```rust
// Post a comment on a pull request
pub async fn post_comment(
    &self,
    workspace: &str,
    repo_slug: &str,
    pr_id: i64,
    body: &str,
) -> Result<i64, PlatformError>

// Approve a pull request
pub async fn approve_pr(
    &self,
    workspace: &str,
    repo_slug: &str,
    pr_id: i64,
) -> Result<(), PlatformError>

// Remove approval from a pull request
pub async fn unapprove_pr(
    &self,
    workspace: &str,
    repo_slug: &str,
    pr_id: i64,
) -> Result<(), PlatformError>

// Add a default reviewer to a pull request
pub async fn add_default_reviewer(
    &self,
    workspace: &str,
    repo_slug: &str,
    pr_id: i64,
    reviewer_uuid: &str,
) -> Result<(), PlatformError>

// Create or update a build status
pub async fn create_build_status(
    &self,
    workspace: &str,
    repo_slug: &str,
    commit_hash: &str,
    key: &str,
    state: &str,
    name: &str,
    description: Option<&str>,
    url: Option<&str>,
) -> Result<(), PlatformError>
```

## Configuration

### Basic Configuration

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: bitbucket-pr-reviewer
spec:
  trigger:
    type: Bitbucket
    config:
      username: myusername
      app_password_env: BITBUCKET_APP_PASSWORD
      webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
      bot_name: aof-pr-bot

      # Optional filters
      allowed_repos:
        - "myworkspace/myrepo"
        - "myworkspace/*"

      allowed_events:
        - "pullrequest:created"
        - "pullrequest:updated"
        - "repo:push"

      # Feature flags
      enable_comments: true
      enable_approvals: true
      enable_build_status: true
```

### Using OAuth Token

```yaml
config:
  username: myusername
  oauth_token_env: BITBUCKET_OAUTH_TOKEN
  webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
```

### Bitbucket Server/Data Center

```yaml
config:
  username: myusername
  app_password_env: BITBUCKET_APP_PASSWORD
  webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
  api_url: "https://bitbucket.example.com/rest/api/1.0"
```

## Authentication

### App Passwords (Recommended for Cloud)

1. Go to Bitbucket Settings → App passwords
2. Create a new app password with required permissions:
   - **Repository**: Read, Write
   - **Pull requests**: Read, Write
   - **Issues**: Read, Write (if using issues)
3. Store the password in environment variable:
   ```bash
   export BITBUCKET_APP_PASSWORD="your_app_password"
   ```

### OAuth Tokens

For OAuth-based authentication, obtain an access token and configure:
```bash
export BITBUCKET_OAUTH_TOKEN="your_oauth_token"
```

## Webhook Setup

### 1. Configure Webhook in Bitbucket

1. Navigate to Repository Settings → Webhooks
2. Click "Add webhook"
3. Configure:
   - **Title**: AOF Webhook
   - **URL**: `https://your-server.com/webhooks/bitbucket`
   - **Secret**: Generate a random string (store in `BITBUCKET_WEBHOOK_SECRET`)
   - **Status**: Active

### 2. Select Events

Choose the events you want to trigger:
- Repository: Push, Fork, Updated
- Pull request: Created, Updated, Approved, Merged, Declined, Comment
- Issue: Created, Updated, Comment
- Build: Status created, Status updated

### 3. Signature Verification

Bitbucket uses HMAC-SHA256 for webhook signature verification, sent in the `X-Hub-Signature` header (format: `sha256=<hex_signature>`).

## Example Usage

### PR Review Bot

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: bitbucket-pr-reviewer
spec:
  trigger:
    type: Bitbucket
    config:
      username: reviewer-bot
      app_password_env: BITBUCKET_APP_PASSWORD
      webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
      allowed_events:
        - "pullrequest:created"
        - "pullrequest:updated"

  agents:
    - id: reviewer
      name: Code Reviewer
      model: google:gemini-2.5-flash
      tools:
        - name: analyze_code
          description: Analyze code changes in PR

  workflow:
    - step: analyze
      agent: reviewer
      input: |
        Review the pull request:
        Title: {{ trigger.pr_title }}
        Description: {{ trigger.pr_description }}
        Files changed: {{ trigger.pr_changed_files }}

    - step: comment
      action: |
        Use the Bitbucket platform to post a comment with the review:
        {{ steps.analyze.output }}
```

### Build Status Reporter

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: build-status-reporter
spec:
  trigger:
    type: Bitbucket
    config:
      username: build-bot
      app_password_env: BITBUCKET_APP_PASSWORD
      webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
      allowed_events:
        - "repo:push"

  workflow:
    - step: update_status
      action: |
        platform.create_build_status(
          workspace="{{ trigger.workspace }}",
          repo_slug="{{ trigger.repo_slug }}",
          commit_hash="{{ trigger.commit_hash }}",
          key="aof-checks",
          state="INPROGRESS",
          name="AOF Validation",
          description="Running AOF validation checks..."
        )
```

## Platform Capabilities

| Feature | Supported |
|---------|-----------|
| Threading | ✅ Yes (PR conversations) |
| Interactive Elements | ❌ No |
| File Attachments | ✅ Yes |
| Reactions | ❌ No |
| Rich Text (Markdown) | ✅ Yes |
| Approvals | ✅ Yes |

## Metadata Available in Triggers

### Pull Request Events

- `pr_id` - Pull request ID
- `pr_title` - Pull request title
- `pr_state` - Pull request state
- `pr_source_branch` - Source branch name
- `pr_dest_branch` - Destination branch name
- `pr_source_commit` - Source commit hash
- `pr_dest_commit` - Destination commit hash
- `pr_html_url` - Pull request URL

### Push Events

- `branch` - Branch name
- `commit_hash` - Commit hash
- `commit_count` - Number of commits in push

### Issue Events

- `issue_id` - Issue ID
- `issue_title` - Issue title
- `issue_state` - Issue state
- `issue_kind` - Issue kind (bug, enhancement, etc.)
- `issue_html_url` - Issue URL

### Common Metadata

- `event_type` - Event type (e.g., "pullrequest:created")
- `repo_uuid` - Repository UUID
- `repo_full_name` - Repository full name (workspace/repo)
- `repo_private` - Whether repository is private
- `actor_uuid` - User UUID who triggered the event
- `actor_display_name` - User display name

## API Rate Limits

Bitbucket Cloud rate limits:
- **Standard**: 1,000 requests/hour
- **App passwords**: Higher limits based on account type

Bitbucket Server/Data Center:
- Configurable per-installation

## Differences from GitHub

1. **Event names**: Uses colon notation (e.g., `pullrequest:created` vs `pull_request`)
2. **IDs**: Uses UUIDs for resources instead of numeric IDs
3. **Authentication**: App passwords vs Personal Access Tokens
4. **API structure**: Different API endpoint structure and pagination
5. **Build status**: Uses different state values (INPROGRESS, SUCCESSFUL, FAILED)

## Error Handling

The platform adapter handles:
- Invalid signatures (returns 401)
- Repository not in allowlist (returns 403)
- Event type not in allowlist (ignored)
- API errors with detailed error messages

## Testing

Run the test suite:
```bash
cargo test --package aof-triggers --lib bitbucket
```

All tests include:
- Configuration validation
- Repository filtering
- Event filtering
- Signature verification
- Webhook payload parsing
- Platform capabilities

## Resources

- [Bitbucket Webhooks Documentation](https://support.atlassian.com/bitbucket-cloud/docs/manage-webhooks/)
- [Bitbucket API Reference](https://developer.atlassian.com/cloud/bitbucket/rest/intro/)
- [App Passwords Guide](https://support.atlassian.com/bitbucket-cloud/docs/app-passwords/)
