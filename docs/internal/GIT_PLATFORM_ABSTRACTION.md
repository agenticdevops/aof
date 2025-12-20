# Git Platform Abstraction Design

## Overview

AOF's Git Platform abstraction provides a unified interface for integrating with multiple Git hosting platforms (GitHub, GitLab, Bitbucket). This abstraction enables:

- **Reusable agent workflows** - Write once, run on any platform
- **Consistent event handling** - Normalized webhook events across platforms
- **Unified API operations** - Platform-agnostic actions (comment, review, label)
- **Easy platform addition** - Clear adapter interface for new platforms

### Why Abstraction Matters

Without abstraction, every agent would need platform-specific logic:
```yaml
# ❌ Bad: Platform-specific agents
- name: github-pr-reviewer
  triggers:
    - type: GitHub  # Only works with GitHub

- name: gitlab-mr-reviewer  # Duplicate logic for GitLab
  triggers:
    - type: GitLab
```

With abstraction:
```yaml
# ✅ Good: Platform-agnostic agents
- name: pr-reviewer
  triggers:
    - type: GitPlatform  # Works with GitHub, GitLab, Bitbucket
      config:
        platform: GitHub | GitLab | Bitbucket
        events: [pull_request.opened]
```

## Unified Event Model

### Core Event Types

All platforms map to these normalized event types:

| Unified Event | GitHub | GitLab | Bitbucket |
|---------------|--------|--------|-----------|
| `pull_request` | Pull Request | Merge Request | Pull Request |
| `push` | Push | Push | Push |
| `issue` | Issue | Issue | Issue |
| `release` | Release | Release | N/A (use tags) |
| `tag` | Create (tag) | Tag Push | Tag |

### Event Actions

Each event type has standardized actions:

#### `pull_request` Actions
- `opened` - New PR/MR created
- `updated` - PR/MR updated (new commits, force push)
- `synchronize` - Synonym for `updated` (GitHub terminology)
- `reopened` - Closed PR/MR reopened
- `closed` - PR/MR closed without merge
- `merged` - PR/MR merged
- `review_requested` - Reviewer assigned
- `labeled` - Labels added/removed
- `approved` - PR/MR approved
- `changes_requested` - Changes requested in review

#### `push` Actions
- `created` - New branch/tag created
- `deleted` - Branch/tag deleted
- `updated` - Branch updated with commits

#### `issue` Actions
- `opened` - New issue created
- `updated` - Issue edited
- `closed` - Issue closed
- `reopened` - Issue reopened
- `labeled` - Labels changed
- `assigned` - Assignees changed

### Event Data Normalization

All platforms provide this normalized context to agents:

```rust
pub struct GitPlatformEvent {
    pub event_type: GitEventType,
    pub action: GitEventAction,
    pub repository: Repository,
    pub sender: User,
    pub pull_request: Option<PullRequest>,
    pub issue: Option<Issue>,
    pub push: Option<PushEvent>,
}

pub struct Repository {
    pub full_name: String,        // "owner/repo"
    pub clone_url: String,
    pub default_branch: String,
    pub is_private: bool,
}

pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: PullRequestState,  // open, closed, merged
    pub author: User,
    pub base_branch: String,
    pub head_branch: String,
    pub base_sha: String,
    pub head_sha: String,
    pub labels: Vec<String>,
    pub reviewers: Vec<User>,
    pub assignees: Vec<User>,
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub changed_files: Option<Vec<ChangedFile>>,
}

pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: IssueState,  // open, closed
    pub author: User,
    pub labels: Vec<String>,
    pub assignees: Vec<User>,
}

pub struct PushEvent {
    pub ref_name: String,      // "refs/heads/main"
    pub before_sha: String,
    pub after_sha: String,
    pub commits: Vec<Commit>,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
}

pub struct User {
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub struct ChangedFile {
    pub filename: String,
    pub status: FileStatus,  // added, modified, removed, renamed
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>,
}
```

## Unified Trigger Configuration Schema

### Basic Configuration

```yaml
triggers:
  - type: GitPlatform
    config:
      # Platform selection
      platform: GitHub | GitLab | Bitbucket

      # Common fields (all platforms)
      webhook_secret: ${WEBHOOK_SECRET}

      # Event filtering
      events:
        - pull_request.opened
        - pull_request.synchronize
        - push

      # Repository filtering (optional)
      repositories:
        - owner/repo1
        - owner/repo2

      # Branch filtering (optional)
      branches:
        - main
        - develop
        - release/*

      # Label filtering (optional, PR events only)
      labels:
        - review-required
        - needs-testing

      # Platform-specific configuration
      platform_config:
        # GitHub-specific
        github_app_id: ${GITHUB_APP_ID}
        github_private_key: ${GITHUB_PRIVATE_KEY}

        # GitLab-specific
        gitlab_token: ${GITLAB_TOKEN}
        gitlab_url: https://gitlab.company.com

        # Bitbucket-specific
        bitbucket_username: ${BITBUCKET_USERNAME}
        bitbucket_app_password: ${BITBUCKET_APP_PASSWORD}
        bitbucket_workspace: my-workspace
```

### Advanced Filtering

```yaml
triggers:
  - type: GitPlatform
    config:
      platform: GitHub
      events: [pull_request.opened]

      # File path filters
      file_patterns:
        include:
          - src/**/*.rs
          - tests/**/*.rs
        exclude:
          - docs/**
          - .github/**

      # Author filters
      author_filter:
        include: [bot-user, ci-user]
        exclude: [spam-user]

      # Size filters
      max_changed_files: 100
      max_additions: 1000

      # Draft PR handling
      skip_drafts: true

      # Conditional processing
      conditions:
        - field: pull_request.labels
          contains: needs-review
        - field: pull_request.base_branch
          equals: main
```

## Unified Agent Context

Agents receive normalized context regardless of platform:

```yaml
# Agent receives this context from any platform
context:
  platform: GitHub | GitLab | Bitbucket
  event_type: pull_request
  action: opened

  repository:
    full_name: owner/repo
    clone_url: https://github.com/owner/repo
    default_branch: main

  pull_request:
    number: 123
    title: "Add new feature"
    body: "Description of changes"
    state: open
    author:
      username: developer
      email: dev@example.com
    base_branch: main
    head_branch: feature/new-feature
    base_sha: abc123
    head_sha: def456
    labels: [enhancement, needs-review]
    draft: false
    changed_files:
      - filename: src/main.rs
        status: modified
        additions: 42
        deletions: 10
```

### Using Context in Agents

```yaml
spec:
  agent:
    prompt: |
      You are reviewing PR #{{pull_request.number}}: "{{pull_request.title}}"

      Repository: {{repository.full_name}}
      Author: {{pull_request.author.username}}
      Branch: {{pull_request.head_branch}} → {{pull_request.base_branch}}

      Changed files ({{pull_request.changed_files | length}}):
      {% for file in pull_request.changed_files %}
      - {{file.filename}} (+{{file.additions}} -{{file.deletions}})
      {% endfor %}

      Review the changes and provide feedback.
```

## Platform Adapter Interface

All platform adapters implement this trait:

```rust
#[async_trait]
pub trait GitPlatformAdapter: Send + Sync {
    /// Parse incoming webhook payload into normalized event
    async fn parse_webhook(
        &self,
        headers: &HeaderMap,
        body: &[u8],
    ) -> Result<GitPlatformEvent>;

    /// Verify webhook signature
    async fn verify_webhook(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        secret: &str,
    ) -> Result<bool>;

    /// Post a comment on PR/Issue
    async fn post_comment(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<Comment>;

    /// Create/update a review on PR
    async fn post_review(
        &self,
        repo: &str,
        pr_number: u64,
        review: &Review,
    ) -> Result<Review>;

    /// Add labels to PR/Issue
    async fn add_labels(
        &self,
        repo: &str,
        number: u64,
        labels: Vec<String>,
    ) -> Result<()>;

    /// Remove labels from PR/Issue
    async fn remove_labels(
        &self,
        repo: &str,
        number: u64,
        labels: Vec<String>,
    ) -> Result<()>;

    /// Create commit status/check
    async fn create_status(
        &self,
        repo: &str,
        sha: &str,
        status: &CommitStatus,
    ) -> Result<()>;

    /// Request reviewers on PR
    async fn request_reviewers(
        &self,
        repo: &str,
        pr_number: u64,
        reviewers: Vec<String>,
    ) -> Result<()>;

    /// Get PR diff/patch
    async fn get_pr_diff(
        &self,
        repo: &str,
        pr_number: u64,
    ) -> Result<String>;

    /// Get file contents at specific ref
    async fn get_file_contents(
        &self,
        repo: &str,
        path: &str,
        ref_name: &str,
    ) -> Result<String>;

    /// Merge pull request
    async fn merge_pr(
        &self,
        repo: &str,
        pr_number: u64,
        merge_method: MergeMethod,
        commit_message: Option<String>,
    ) -> Result<()>;

    /// Close pull request without merging
    async fn close_pr(
        &self,
        repo: &str,
        pr_number: u64,
    ) -> Result<()>;
}

pub struct Review {
    pub body: String,
    pub event: ReviewEvent,  // approve, request_changes, comment
    pub comments: Vec<ReviewComment>,
}

pub struct ReviewComment {
    pub path: String,
    pub line: u32,
    pub body: String,
    pub side: CommentSide,  // left (base), right (head)
}

pub struct CommitStatus {
    pub state: StatusState,  // pending, success, failure, error
    pub description: String,
    pub context: String,  // Unique identifier for this status
    pub target_url: Option<String>,
}

pub enum MergeMethod {
    Merge,      // Create merge commit
    Squash,     // Squash and merge
    Rebase,     // Rebase and merge
}
```

### Adapter Factory

```rust
pub struct GitPlatformAdapterFactory;

impl GitPlatformAdapterFactory {
    pub fn create(config: &GitPlatformConfig) -> Result<Box<dyn GitPlatformAdapter>> {
        match config.platform {
            GitPlatform::GitHub => {
                let adapter = GitHubAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
            GitPlatform::GitLab => {
                let adapter = GitLabAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
            GitPlatform::Bitbucket => {
                let adapter = BitbucketAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
        }
    }
}
```

## Implementation Status

| Platform | Status | Priority | Webhook Events | API Actions | Notes |
|----------|--------|----------|----------------|-------------|-------|
| **GitHub** | ✅ Implemented | - | ✅ All | ✅ All | Full support via GitHub App |
| **GitLab** | 🔜 Planned | P1 | 🔜 | 🔜 | Self-hosted + GitLab.com |
| **Bitbucket** | 🔜 Planned | P2 | 🔜 | 🔜 | Cloud + Server |
| **Gitea** | 📋 Backlog | P3 | - | - | Self-hosted alternative |
| **Azure DevOps** | 📋 Backlog | P4 | - | - | Enterprise |

### GitHub Implementation (Completed)

**Webhook Events Supported:**
- `pull_request` (all actions)
- `push` (all actions)
- `issue_comment` (on PRs)
- `pull_request_review`
- `pull_request_review_comment`

**API Actions Implemented:**
- ✅ Post comments
- ✅ Create reviews with inline comments
- ✅ Add/remove labels
- ✅ Create commit statuses
- ✅ Request reviewers
- ✅ Get PR diff
- ✅ Get file contents
- ✅ Merge PR
- ✅ Close PR

**Authentication:**
- GitHub App (recommended)
- Personal Access Token (fallback)

### GitLab Implementation (Planned)

**Key Differences:**
- Merge Requests instead of Pull Requests
- Different webhook payload structure
- API v4 REST + GraphQL
- Self-hosted support required
- Project-level vs repo-level webhooks

**Webhook Mapping:**
```
GitLab Event          → Unified Event
merge_request         → pull_request
push                  → push
issue                 → issue
note (on MR)          → pull_request_comment
wiki_page             → (not mapped)
```

**Configuration:**
```yaml
platform_config:
  gitlab_token: ${GITLAB_TOKEN}
  gitlab_url: https://gitlab.company.com  # Optional, defaults to gitlab.com
  api_version: v4  # Default
```

### Bitbucket Implementation (Planned)

**Key Differences:**
- Workspaces instead of organizations
- Different permission model
- API 2.0
- Cloud vs Server different APIs

**Webhook Mapping:**
```
Bitbucket Event       → Unified Event
pullrequest:created   → pull_request.opened
pullrequest:updated   → pull_request.synchronize
pullrequest:fulfilled → pull_request.merged
repo:push             → push
issue:created         → issue.opened
```

**Configuration:**
```yaml
platform_config:
  bitbucket_username: ${BITBUCKET_USERNAME}
  bitbucket_app_password: ${BITBUCKET_APP_PASSWORD}
  bitbucket_workspace: my-workspace
  bitbucket_url: https://bitbucket.company.com  # For Server
```

## Extension Guide

### Adding a New Platform

**Step 1: Define Platform Enum**

```rust
// In aof-core/src/git_platform/mod.rs
pub enum GitPlatform {
    GitHub,
    GitLab,
    Bitbucket,
    YourPlatform,  // Add here
}
```

**Step 2: Implement Adapter**

Create `aof-core/src/git_platform/adapters/your_platform.rs`:

```rust
use super::*;

pub struct YourPlatformAdapter {
    client: reqwest::Client,
    config: PlatformConfig,
}

impl YourPlatformAdapter {
    pub fn new(config: &HashMap<String, String>) -> Result<Self> {
        // Parse platform-specific config
        let api_token = config.get("your_platform_token")
            .ok_or_else(|| anyhow!("Missing your_platform_token"))?;

        Ok(Self {
            client: reqwest::Client::new(),
            config: PlatformConfig { api_token: api_token.clone() },
        })
    }
}

#[async_trait]
impl GitPlatformAdapter for YourPlatformAdapter {
    async fn parse_webhook(
        &self,
        headers: &HeaderMap,
        body: &[u8],
    ) -> Result<GitPlatformEvent> {
        // 1. Parse webhook JSON
        let payload: YourPlatformWebhook = serde_json::from_slice(body)?;

        // 2. Determine event type and action
        let event_type = match payload.event_type.as_str() {
            "pr.opened" => GitEventType::PullRequest,
            "repo.push" => GitEventType::Push,
            _ => return Err(anyhow!("Unknown event type")),
        };

        let action = match payload.action.as_str() {
            "opened" => GitEventAction::Opened,
            "updated" => GitEventAction::Synchronize,
            _ => return Err(anyhow!("Unknown action")),
        };

        // 3. Normalize to GitPlatformEvent
        Ok(GitPlatformEvent {
            event_type,
            action,
            repository: self.normalize_repository(&payload.repository),
            sender: self.normalize_user(&payload.sender),
            pull_request: self.normalize_pr(&payload.pull_request),
            issue: None,
            push: None,
        })
    }

    async fn verify_webhook(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        secret: &str,
    ) -> Result<bool> {
        // Implement platform-specific signature verification
        let signature = headers.get("X-Your-Platform-Signature")
            .ok_or_else(|| anyhow!("Missing signature header"))?
            .to_str()?;

        let expected = hmac_sha256(body, secret.as_bytes());
        Ok(signature == hex::encode(expected))
    }

    async fn post_comment(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<Comment> {
        // Call platform API to post comment
        let url = format!("https://api.yourplatform.com/repos/{}/prs/{}/comments", repo, number);

        let response = self.client
            .post(&url)
            .bearer_auth(&self.config.api_token)
            .json(&serde_json::json!({ "body": body }))
            .send()
            .await?;

        let comment: YourPlatformComment = response.json().await?;
        Ok(Comment {
            id: comment.id,
            body: comment.body,
        })
    }

    // Implement remaining trait methods...
}
```

**Step 3: Add to Factory**

```rust
// In git_platform/mod.rs
impl GitPlatformAdapterFactory {
    pub fn create(config: &GitPlatformConfig) -> Result<Box<dyn GitPlatformAdapter>> {
        match config.platform {
            GitPlatform::GitHub => Ok(Box::new(GitHubAdapter::new(&config.platform_config)?)),
            GitPlatform::GitLab => Ok(Box::new(GitLabAdapter::new(&config.platform_config)?)),
            GitPlatform::Bitbucket => Ok(Box::new(BitbucketAdapter::new(&config.platform_config)?)),
            GitPlatform::YourPlatform => Ok(Box::new(YourPlatformAdapter::new(&config.platform_config)?)),
        }
    }
}
```

**Step 4: Add Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_webhook() {
        let adapter = YourPlatformAdapter::new(&test_config()).unwrap();

        let webhook_payload = include_bytes!("../../../tests/fixtures/your_platform_pr_opened.json");
        let headers = test_headers();

        let event = adapter.parse_webhook(&headers, webhook_payload).await.unwrap();

        assert_eq!(event.event_type, GitEventType::PullRequest);
        assert_eq!(event.action, GitEventAction::Opened);
        assert_eq!(event.pull_request.unwrap().number, 123);
    }

    #[tokio::test]
    async fn test_post_comment() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("POST", "/repos/owner/repo/prs/123/comments")
            .with_status(201)
            .with_body(r#"{"id": 1, "body": "Test comment"}"#)
            .create_async()
            .await;

        let adapter = YourPlatformAdapter::new(&test_config()).unwrap();
        let comment = adapter.post_comment("owner/repo", 123, "Test comment").await.unwrap();

        assert_eq!(comment.body, "Test comment");
        mock.assert_async().await;
    }
}
```

**Step 5: Add Documentation**

Create `docs/triggers/your-platform.md`:

```markdown
# YourPlatform Trigger

Trigger agents based on YourPlatform events (PRs, pushes, issues).

## Configuration

...yaml
triggers:
  - type: GitPlatform
    config:
      platform: YourPlatform
      webhook_secret: ${WEBHOOK_SECRET}
      events:
        - pull_request.opened
        - push
      platform_config:
        your_platform_token: ${YOUR_PLATFORM_TOKEN}
        your_platform_url: https://api.yourplatform.com
...

## Webhook Setup

1. Navigate to repository settings
2. Add webhook: `https://your-daemon.com/webhooks/git-platform`
3. Set secret: `your-webhook-secret`
4. Select events: Pull Requests, Pushes

## Available Events

- `pull_request.opened`
- `pull_request.synchronize`
- `pull_request.merged`
- `push`
- `issue.opened`

## Examples

See `examples/git-platform/pr-reviewer.yaml`
```

**Step 6: Update User Documentation**

Add to `docs/concepts/triggers.md`:

```markdown
### Supported Git Platforms

| Platform | Status | Authentication |
|----------|--------|----------------|
| GitHub | ✅ | GitHub App, PAT |
| GitLab | ✅ | Token |
| Bitbucket | ✅ | App Password |
| YourPlatform | ✅ | API Token |
```

## Migration Path

### For Existing Agents

Agents using `GitHub` trigger can migrate to `GitPlatform`:

**Before (GitHub-specific):**
```yaml
triggers:
  - type: GitHub
    config:
      webhook_secret: ${WEBHOOK_SECRET}
      events: [pull_request]
```

**After (Platform-agnostic):**
```yaml
triggers:
  - type: GitPlatform
    config:
      platform: GitHub  # Can change to GitLab, Bitbucket later
      webhook_secret: ${WEBHOOK_SECRET}
      events: [pull_request.opened]
```

### Backward Compatibility

AOF maintains backward compatibility:
- `type: GitHub` continues to work (maps to `GitPlatform` with `platform: GitHub`)
- Deprecation warnings guide users to new syntax
- Automatic migration tool: `aofctl migrate triggers`

## Best Practices

### 1. Platform Detection

```yaml
# Use platform-specific logic only when necessary
spec:
  agent:
    prompt: |
      {% if platform == "GitHub" %}
      Use GitHub-flavored Markdown with task lists:
      - [ ] Item 1
      {% elif platform == "GitLab" %}
      Use GitLab Markdown with quick actions:
      /label ~needs-review
      {% endif %}
```

### 2. Graceful Degradation

```rust
// Not all platforms support all features
match adapter.create_status(repo, sha, status).await {
    Ok(_) => info!("Status created"),
    Err(e) if e.to_string().contains("not supported") => {
        // Fallback: use comment instead
        adapter.post_comment(repo, pr_number, &status_message).await?;
    }
    Err(e) => return Err(e),
}
```

### 3. Platform-Specific Features

Use `platform_config` for features unique to a platform:

```yaml
platform_config:
  # GitHub-specific: Draft PRs
  skip_drafts: true

  # GitLab-specific: Merge train
  gitlab_merge_train: true

  # Bitbucket-specific: Build status
  bitbucket_build_status: true
```

## Testing

### Integration Tests

```bash
# Test all platforms
cargo test --features github,gitlab,bitbucket

# Test specific platform
cargo test --features github
```

### Webhook Fixtures

Store real webhook payloads in `tests/fixtures/`:
- `github_pr_opened.json`
- `gitlab_mr_opened.json`
- `bitbucket_pr_created.json`

### Mock Servers

Use `mockito` to simulate platform APIs:

```rust
#[tokio::test]
async fn test_cross_platform_comment() {
    for platform in [GitPlatform::GitHub, GitPlatform::GitLab, GitPlatform::Bitbucket] {
        let adapter = create_test_adapter(platform);
        let comment = adapter.post_comment("owner/repo", 123, "Test").await.unwrap();
        assert_eq!(comment.body, "Test");
    }
}
```

## Security Considerations

### Webhook Verification

All platforms must verify webhook signatures:

```rust
async fn verify_webhook(&self, headers: &HeaderMap, body: &[u8], secret: &str) -> Result<bool> {
    // Platform-specific signature verification
    // GitHub: HMAC SHA-256 with "sha256=" prefix
    // GitLab: X-Gitlab-Token header
    // Bitbucket: No standard signature (use IP allowlist)
}
```

### Token Scopes

Document minimum required scopes per platform:

| Platform | Scope | Purpose |
|----------|-------|---------|
| GitHub | `repo` | Full repository access |
| GitHub | `pull_requests:write` | Fine-grained alternative |
| GitLab | `api` | Full API access |
| GitLab | `read_repository` | Read-only |
| Bitbucket | `pullrequest:write` | PR comments/reviews |
| Bitbucket | `repository:write` | Full access |

## Future Enhancements

### Planned Features

1. **GraphQL Support** (GitHub, GitLab)
   - More efficient queries
   - Reduced API calls
   - Better rate limit usage

2. **Webhook Replay**
   - Store webhook payloads
   - Replay for testing
   - Debug failed triggers

3. **Multi-Platform PR Sync**
   - Mirror PRs across platforms
   - Sync comments/reviews
   - Cross-platform CI/CD

4. **Platform Analytics**
   - Track webhook volumes
   - Measure API latencies
   - Optimize adapter performance

5. **Custom Adapters**
   - Plugin system for proprietary platforms
   - Community-contributed adapters
   - Dynamic adapter loading

## References

- [GitHub Webhook Events](https://docs.github.com/webhooks/event-payloads)
- [GitLab Webhook Events](https://docs.gitlab.com/ee/user/project/integrations/webhooks.html)
- [Bitbucket Webhook Events](https://support.atlassian.com/bitbucket-cloud/docs/event-payloads/)
- [AOF Trigger System](../concepts/triggers.md)
- [Agent Context](../concepts/agent-context.md)

---

**Status**: Design Document (GitHub implementation complete, GitLab/Bitbucket planned)
**Last Updated**: 2025-12-20
**Owner**: AOF Core Team
