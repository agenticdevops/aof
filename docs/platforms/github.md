# GitHub Platform Integration

## Overview

The GitHub platform adapter provides webhook-based integration with GitHub.com and GitHub Enterprise, enabling AOF agents to respond to repository events, manage pull requests, post reviews, and interact with the GitHub API.

## Features

### Supported Events

**Pull Request Events:**
- `pull_request.opened` - New pull request created
- `pull_request.synchronize` - PR updated with new commits
- `pull_request.closed` - PR closed or merged
- `pull_request.reopened` - PR reopened
- `pull_request.edited` - PR title/description edited
- `pull_request.assigned` - PR assigned to someone
- `pull_request.review_requested` - Review requested

**Pull Request Review Events:**
- `pull_request_review.submitted` - Review submitted
- `pull_request_review.edited` - Review edited
- `pull_request_review.dismissed` - Review dismissed

**Issue Events:**
- `issues.opened` - New issue created
- `issues.closed` - Issue closed
- `issues.reopened` - Issue reopened
- `issues.edited` - Issue edited
- `issues.assigned` - Issue assigned
- `issues.labeled` - Label added to issue

**Comment Events:**
- `issue_comment.created` - Comment on issue or PR
- `issue_comment.edited` - Comment edited
- `issue_comment.deleted` - Comment deleted
- `pull_request_review_comment.created` - Inline PR comment

**Repository Events:**
- `push` - Code pushed to repository
- `create` - Branch or tag created
- `delete` - Branch or tag deleted
- `fork` - Repository forked
- `release.published` - Release published
- `star.created` - Repository starred
- `watch.started` - Repository watched

**CI/CD Events:**
- `workflow_run.completed` - GitHub Actions workflow completed
- `workflow_run.requested` - Workflow triggered
- `workflow_job.completed` - Workflow job completed
- `check_run.created` - Check run created
- `check_run.completed` - Check run completed
- `check_suite.completed` - Check suite completed

### API Methods

The GitHub platform provides the following API methods:

```rust
// Post a comment on an issue or PR
pub async fn post_comment(
    &self,
    owner: &str,
    repo: &str,
    issue_number: i64,
    body: &str,
) -> Result<i64, PlatformError>

// Post a PR review
pub async fn post_review(
    &self,
    owner: &str,
    repo: &str,
    pr_number: i64,
    body: &str,
    event: &str, // APPROVE, REQUEST_CHANGES, COMMENT
) -> Result<i64, PlatformError>

// Create or update a check run
pub async fn create_check_run(
    &self,
    owner: &str,
    repo: &str,
    head_sha: &str,
    name: &str,
    status: &str, // queued, in_progress, completed
    conclusion: Option<&str>, // success, failure, neutral, etc.
    output: Option<CheckRunOutput>,
) -> Result<i64, PlatformError>

// Add labels to an issue or PR
pub async fn add_labels(
    &self,
    owner: &str,
    repo: &str,
    issue_number: i64,
    labels: &[String],
) -> Result<(), PlatformError>

// Remove a label
pub async fn remove_label(
    &self,
    owner: &str,
    repo: &str,
    issue_number: i64,
    label: &str,
) -> Result<(), PlatformError>
```

## Configuration

### Basic Setup

Add GitHub to your daemon configuration (`config/aof/daemon.yaml`):

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: production

spec:
  server:
    port: 8080
    host: 0.0.0.0

  platforms:
    github:
      enabled: true
      token_env: GITHUB_TOKEN
      webhook_secret_env: GITHUB_WEBHOOK_SECRET
      bot_name: "aofbot"
```

### Environment Variables

```bash
# Required: Personal Access Token or GitHub App token
export GITHUB_TOKEN="ghp_your_token_here"

# Required: Webhook secret for signature verification
export GITHUB_WEBHOOK_SECRET="your_webhook_secret_here"

# Source your shell config if variables are in ~/.zshrc or ~/.bashrc
source ~/.zshrc  # or source ~/.bashrc
```

#### Creating a GitHub Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Select scopes:
   - `repo` (Full control of private repositories)
   - `write:discussion` (Read and write discussions)
4. Generate and copy the token

#### Generating Webhook Secret

```bash
openssl rand -hex 32
```

Save this for both the environment variable and GitHub webhook configuration.

### Repository Filtering

Restrict which repositories can trigger agents:

```yaml
platforms:
  github:
    enabled: true
    token_env: GITHUB_TOKEN
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
    allowed_repos:
      - "myorg/important-repo"    # Specific repo
      - "myorg/another-repo"
      - "myorg/*"                 # All repos in organization
```

### Organization Filtering

```yaml
platforms:
  github:
    enabled: true
    token_env: GITHUB_TOKEN
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
    allowed_orgs:
      - "myorg"
      - "anotherorg"
```

## Setting Up Webhooks

### 1. Configure Webhook in GitHub

1. Go to repository Settings → Webhooks → Add webhook
2. Configure:
   - **Payload URL**: `https://your-server.com/webhook/github`
   - **Content type**: `application/json`
   - **Secret**: Your `GITHUB_WEBHOOK_SECRET` value
   - **SSL verification**: Enable (required)
   - **Events**: Select events you want:
     - Pull requests
     - Pull request reviews
     - Pull request review comments
     - Issue comments
     - Push events
     - Workflow runs
3. Click "Add webhook"

### 2. Local Development with ngrok

```bash
# Start ngrok tunnel
ngrok http 8080

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
# Use https://abc123.ngrok.io/webhook/github as Payload URL
```

### 3. Verify Webhook

After adding the webhook, GitHub will send a `ping` event. Check:
- GitHub webhook page shows green checkmark
- Recent Deliveries shows HTTP 200 response
- AOF logs show webhook received

## Agent Configuration

### PR Review Agent Example

Create `agents/github-pr-reviewer.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: github-pr-reviewer
  labels:
    category: code-review
    platform: github

spec:
  model: google:gemini-2.5-flash

  system_prompt: |
    You are an expert code reviewer performing thorough pull request reviews.

    ## Review Focus Areas

    ### 1. Code Quality
    - Readability and maintainability
    - Proper error handling
    - Code organization and structure

    ### 2. Security
    - SQL injection vulnerabilities
    - XSS risks
    - Authentication/authorization issues
    - Secrets or API keys in code

    ### 3. Performance
    - Inefficient algorithms
    - Memory leaks
    - N+1 query problems

    ### 4. Best Practices
    - Design patterns
    - Language-specific idioms
    - Testing coverage

    ## Output Format

    Provide your review in markdown:

    ## 🔍 Code Review Summary

    **Overall Assessment**: [Approve ✅ / Request Changes ⚠️ / Comment 💬]

    ### ✨ Strengths
    - [Positive aspects]

    ### ⚠️ Issues Found

    #### Critical (Must Fix)
    - **[File:Line]** - [Issue description]

    #### Suggestions
    - **[File:Line]** - [Suggestion]

  tools:
    - shell

  max_iterations: 8
  temperature: 0.3
```

### Issue Triage Agent Example

Create `agents/github-issue-triager.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: github-issue-triager
  labels:
    category: triage
    platform: github

spec:
  model: google:gemini-2.5-flash

  system_prompt: |
    You are a GitHub issue triage assistant.

    For each new issue:
    1. Analyze the issue description
    2. Determine the type: bug, feature, documentation, question
    3. Assess severity: critical, high, medium, low
    4. Assign appropriate labels
    5. Suggest which team/person should handle it

  tools:
    - shell

  max_iterations: 5
  temperature: 0.3
```

## AgentFlow Integration

### PR Review Flow Example

Create `flows/github/pr-review.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: pr-review-flow
  labels:
    platform: github
    event: pull_request

spec:
  trigger:
    type: GitHub
    config:
      events:
        - pull_request.opened
        - pull_request.synchronize
      filters:
        - field: pull_request.draft
          operator: equals
          value: false

  nodes:
    - id: fetch-pr-diff
      type: Action
      action:
        type: shell
        command: |
          gh pr diff ${{ event.pull_request.number }} \
            --repo ${{ event.repository.full_name }}

    - id: review
      type: Agent
      agent: github-pr-reviewer
      input: |
        Review this pull request:

        **Title**: ${{ event.pull_request.title }}
        **Description**: ${{ event.pull_request.body }}
        **Changes**:
        ${{ nodes.fetch-pr-diff.output }}

    - id: post-review
      type: Action
      action:
        type: github_review
        owner: ${{ event.repository.owner.login }}
        repo: ${{ event.repository.name }}
        pr_number: ${{ event.pull_request.number }}
        body: ${{ nodes.review.output }}
        event: COMMENT
```

## Command Detection

GitHub supports command detection in PR/issue comments using slash commands:

```
/review - Trigger PR review
/deploy staging - Deploy to staging
/run-tests - Run test suite
```

The platform automatically detects commands that start with `/` in comments.

## Security Features

### Signature Verification

All webhooks are verified using HMAC-SHA256 signature:

```rust
// Automatic signature verification
fn verify_github_signature(&self, payload: &[u8], signature: &str) -> bool {
    // Verifies X-Hub-Signature-256 header
}
```

### Repository Filtering

Only allowed repositories can trigger agents:

```yaml
github:
  allowed_repos:
    - "trusted-org/*"
```

### Event Filtering

Control which events are processed:

```yaml
github:
  allowed_events:
    - "pull_request"
    - "issues"
```

## Advanced Features

### Status Checks

Create GitHub status checks for CI/CD:

```rust
platform.create_check_run(
    "owner",
    "repo",
    "commit-sha",
    "AOF Analysis",
    "completed",
    Some("success"),
    Some(CheckRunOutput {
        title: "Analysis Complete".to_string(),
        summary: "All checks passed".to_string(),
        text: Some("Detailed results...".to_string()),
    }),
).await?;
```

### PR Reviews

Post structured PR reviews:

```rust
platform.post_review(
    "owner",
    "repo",
    42,
    "LGTM! Great work on this PR.",
    "APPROVE", // or REQUEST_CHANGES, COMMENT
).await?;
```

### Label Management

Automatically manage labels:

```rust
// Add labels
platform.add_labels("owner", "repo", 42, &[
    "bug".to_string(),
    "high-priority".to_string(),
]).await?;

// Remove labels
platform.remove_label("owner", "repo", 42, "needs-triage").await?;
```

## Troubleshooting

### "GitHub enabled but missing webhook_secret"

**Solution**: Set the environment variable and source your shell config:

```bash
export GITHUB_WEBHOOK_SECRET="your_secret"
source ~/.zshrc  # or ~/.bashrc
./target/release/aofctl serve --config config/aof/daemon.yaml
```

### "Invalid signature" in webhook deliveries

**Causes**:
1. Webhook secret mismatch
2. Using HTTP instead of HTTPS
3. Payload modified by proxy

**Solution**:
1. Verify secrets match exactly
2. Use HTTPS URL (ngrok provides this)
3. Check proxy configuration

### "GitHub: GITHUB_TOKEN not set, API features disabled"

This is a warning, not an error. Webhooks still work, but:
- Cannot post comments
- Cannot post reviews
- Cannot update status checks

**Solution**: Set `GITHUB_TOKEN` environment variable.

### Agent not responding to events

**Check**:
1. Webhook configured for correct events
2. Agent file exists and loads successfully
3. Event passes repository/organization filters
4. Check AOF logs for errors

## Production Deployment

### GitHub App (Recommended)

For production, use a GitHub App instead of PAT:

**Benefits:**
- Better security
- Granular permissions
- Higher rate limits
- Per-installation tokens

### Rate Limiting

GitHub API has rate limits:
- PAT: 5,000 requests/hour
- GitHub App: 15,000 requests/hour per installation

Monitor rate limits in logs and implement backoff strategies.

### High Availability

For production:
1. Run multiple AOF instances behind load balancer
2. Use shared state (Redis, PostgreSQL)
3. Configure webhook redelivery
4. Set up monitoring and alerting

## Examples

### Complete PR Review Workflow

See the quickstart guide: [GITHUB_SETUP.md](../../GITHUB_SETUP.md)

### Issue Auto-Labeling

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: issue-labeler

spec:
  trigger:
    type: GitHub
    config:
      events:
        - issues.opened

  nodes:
    - id: analyze
      type: Agent
      agent: github-issue-triager
      input: |
        Analyze this issue and suggest labels:

        **Title**: ${{ event.issue.title }}
        **Body**: ${{ event.issue.body }}

    - id: apply-labels
      type: Action
      action:
        type: github_labels
        owner: ${{ event.repository.owner.login }}
        repo: ${{ event.repository.name }}
        issue_number: ${{ event.issue.number }}
        labels: ${{ nodes.analyze.output.labels }}
```

### Automated Dependency Updates

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: dependency-checker

spec:
  trigger:
    type: GitHub
    config:
      events:
        - pull_request.opened
      filters:
        - field: pull_request.user.login
          operator: equals
          value: dependabot[bot]

  nodes:
    - id: review-deps
      type: Agent
      agent: dependency-reviewer
      input: |
        Review this dependency update PR:
        ${{ event.pull_request.title }}

    - id: auto-approve
      type: Condition
      condition: ${{ nodes.review-deps.output.safe == true }}
      then:
        - type: github_review
          event: APPROVE
```

## API Reference

Complete API documentation: [GitHub API Docs](https://docs.github.com/en/rest)

AOF Platform Methods: See `crates/aof-triggers/src/platforms/github.rs`

## Support

- GitHub webhook documentation: https://docs.github.com/webhooks
- AOF documentation: https://docs.aof.sh
- Issues: https://github.com/agenticdevops/aof/issues
