//! GitHub Webhook adapter for AOF
//!
//! This module provides integration with GitHub's Webhook API, supporting:
//! - Push events
//! - Pull request events (opened, synchronize, closed)
//! - Issue events
//! - Workflow run events
//! - Check run events
//! - HMAC-SHA256 signature verification
//! - PR reviews and status checks
//!
//! # Design Philosophy
//!
//! This platform is designed as a **pluggable component** that can be:
//! - Enabled/disabled via Cargo feature flags
//! - Extended with custom event handlers
//! - Integrated with AgentFlow workflows
//!
//! # Example Usage
//!
//! ```yaml
//! apiVersion: aof.dev/v1
//! kind: AgentFlow
//! spec:
//!   trigger:
//!     type: GitHub
//!     config:
//!       events:
//!         - pull_request.opened
//!         - push
//!       token_env: GITHUB_TOKEN
//!       webhook_secret_env: GITHUB_WEBHOOK_SECRET
//! ```

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// GitHub platform adapter
///
/// Implements the `TriggerPlatform` trait for GitHub webhooks.
/// Supports multiple event types and provides methods for interacting
/// with the GitHub API (posting comments, reviews, status checks).
pub struct GitHubPlatform {
    config: GitHubConfig,
    client: reqwest::Client,
}

/// GitHub configuration
///
/// All configuration options for the GitHub platform.
/// Supports environment variable resolution for secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Personal Access Token or GitHub App token for API calls
    /// Can use PAT (ghp_*) or Installation token
    pub token: String,

    /// Webhook secret for HMAC-SHA256 signature verification
    /// Required for secure webhook validation
    pub webhook_secret: String,

    /// Bot/App name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// API base URL (for GitHub Enterprise support)
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Allowed repository filter (optional whitelist)
    /// Format: ["owner/repo", "owner/*", "*"]
    #[serde(default)]
    pub allowed_repos: Option<Vec<String>>,

    /// Event types to handle (optional filter)
    /// If empty, handles all events
    #[serde(default)]
    pub allowed_events: Option<Vec<String>>,

    /// User IDs allowed to trigger actions (optional whitelist)
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,

    /// Auto-approve certain operations (for trusted repos)
    #[serde(default)]
    pub auto_approve_patterns: Option<Vec<String>>,

    /// Enable posting status checks
    #[serde(default = "default_true")]
    pub enable_status_checks: bool,

    /// Enable posting PR reviews
    #[serde(default = "default_true")]
    pub enable_reviews: bool,

    /// Enable posting comments
    #[serde(default = "default_true")]
    pub enable_comments: bool,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_api_url() -> String {
    "https://api.github.com".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// WEBHOOK PAYLOAD TYPES
// ============================================================================

/// GitHub webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitHubEventType {
    Push,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    Issues,
    IssueComment,
    WorkflowRun,
    WorkflowJob,
    CheckRun,
    CheckSuite,
    Create,
    Delete,
    Fork,
    Release,
    Star,
    Watch,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for GitHubEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
            Self::PullRequest => write!(f, "pull_request"),
            Self::PullRequestReview => write!(f, "pull_request_review"),
            Self::PullRequestReviewComment => write!(f, "pull_request_review_comment"),
            Self::Issues => write!(f, "issues"),
            Self::IssueComment => write!(f, "issue_comment"),
            Self::WorkflowRun => write!(f, "workflow_run"),
            Self::WorkflowJob => write!(f, "workflow_job"),
            Self::CheckRun => write!(f, "check_run"),
            Self::CheckSuite => write!(f, "check_suite"),
            Self::Create => write!(f, "create"),
            Self::Delete => write!(f, "delete"),
            Self::Fork => write!(f, "fork"),
            Self::Release => write!(f, "release"),
            Self::Star => write!(f, "star"),
            Self::Watch => write!(f, "watch"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for GitHubEventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "push" => Self::Push,
            "pull_request" => Self::PullRequest,
            "pull_request_review" => Self::PullRequestReview,
            "pull_request_review_comment" => Self::PullRequestReviewComment,
            "issues" => Self::Issues,
            "issue_comment" => Self::IssueComment,
            "workflow_run" => Self::WorkflowRun,
            "workflow_job" => Self::WorkflowJob,
            "check_run" => Self::CheckRun,
            "check_suite" => Self::CheckSuite,
            "create" => Self::Create,
            "delete" => Self::Delete,
            "fork" => Self::Fork,
            "release" => Self::Release,
            "star" => Self::Star,
            "watch" => Self::Watch,
            _ => Self::Unknown,
        }
    }
}

/// Common repository information
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub default_branch: Option<String>,
}

/// GitHub user information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    #[serde(rename = "type")]
    pub user_type: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
}

/// Pull request information
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub body: Option<String>,
    pub user: GitHubUser,
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub merged: bool,
    #[serde(default)]
    pub mergeable: Option<bool>,
    pub base: PullRequestRef,
    pub head: PullRequestRef,
    #[serde(default)]
    pub additions: i64,
    #[serde(default)]
    pub deletions: i64,
    #[serde(default)]
    pub changed_files: i64,
}

/// Pull request ref (base or head)
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    #[serde(default)]
    pub repo: Option<Repository>,
}

/// Issue information
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub body: Option<String>,
    pub user: GitHubUser,
    pub html_url: String,
    #[serde(default)]
    pub labels: Vec<Label>,
}

/// Label information
#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Comment information
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub user: GitHubUser,
    pub html_url: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Push event commit
#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: CommitAuthor,
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(default)]
    pub modified: Vec<String>,
}

/// Commit author information
#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub username: Option<String>,
}

/// Workflow run information
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRun {
    pub id: i64,
    pub name: String,
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    #[serde(default)]
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Check run information
#[derive(Debug, Clone, Deserialize)]
pub struct CheckRun {
    pub id: i64,
    pub name: String,
    pub head_sha: String,
    pub status: String,
    #[serde(default)]
    pub conclusion: Option<String>,
    pub html_url: String,
}

// ============================================================================
// WEBHOOK PAYLOAD
// ============================================================================

/// Generic GitHub webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubWebhookPayload {
    /// Action that triggered the event
    #[serde(default)]
    pub action: Option<String>,

    /// Repository information
    #[serde(default)]
    pub repository: Option<Repository>,

    /// Sender (user who triggered the event)
    #[serde(default)]
    pub sender: Option<GitHubUser>,

    /// Pull request (for PR events)
    #[serde(default)]
    pub pull_request: Option<PullRequest>,

    /// Issue (for issue events)
    #[serde(default)]
    pub issue: Option<Issue>,

    /// Comment (for comment events)
    #[serde(default)]
    pub comment: Option<Comment>,

    /// Commits (for push events)
    #[serde(default)]
    pub commits: Option<Vec<Commit>>,

    /// Head commit (for push events)
    #[serde(default)]
    pub head_commit: Option<Commit>,

    /// Ref (for push events)
    #[serde(rename = "ref", default)]
    pub git_ref: Option<String>,

    /// Before SHA (for push events)
    #[serde(default)]
    pub before: Option<String>,

    /// After SHA (for push events)
    #[serde(default)]
    pub after: Option<String>,

    /// Workflow run (for workflow events)
    #[serde(default)]
    pub workflow_run: Option<WorkflowRun>,

    /// Check run (for check events)
    #[serde(default)]
    pub check_run: Option<CheckRun>,

    /// Installation (for GitHub App events)
    #[serde(default)]
    pub installation: Option<Installation>,
}

/// GitHub App installation information
#[derive(Debug, Clone, Deserialize)]
pub struct Installation {
    pub id: i64,
    #[serde(default)]
    pub account: Option<GitHubUser>,
}

// ============================================================================
// GITHUB API RESPONSE TYPES
// ============================================================================

/// GitHub API error response
#[derive(Debug, Deserialize)]
struct GitHubApiError {
    message: String,
    #[serde(default)]
    documentation_url: Option<String>,
}

/// Comment creation response
#[derive(Debug, Deserialize)]
struct CommentResponse {
    id: i64,
    html_url: String,
}

/// Review creation response
#[derive(Debug, Deserialize)]
struct ReviewResponse {
    id: i64,
    state: String,
    html_url: String,
}

/// Check run creation response
#[derive(Debug, Deserialize)]
struct CheckRunResponse {
    id: i64,
    name: String,
    status: String,
}

// ============================================================================
// PLATFORM IMPLEMENTATION
// ============================================================================

impl GitHubPlatform {
    /// Create new GitHub platform adapter
    ///
    /// # Errors
    /// Returns error if token or webhook_secret is empty
    pub fn new(config: GitHubConfig) -> Result<Self, PlatformError> {
        if config.token.is_empty() || config.webhook_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "GitHub token and webhook secret are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("AOF/{} ({})", env!("CARGO_PKG_VERSION"), config.bot_name))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify HMAC-SHA256 signature from GitHub webhook
    ///
    /// GitHub sends signature in format: sha256=<hex_signature>
    fn verify_github_signature(&self, payload: &[u8], signature: &str) -> bool {
        // GitHub signature format: sha256=<hex_signature>
        if !signature.starts_with("sha256=") {
            debug!("Invalid signature format - must start with sha256=");
            return false;
        }

        let provided_signature = &signature[7..];

        let mut mac = match HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                error!("HMAC setup failed: {}", e);
                return false;
            }
        };

        mac.update(payload);
        let result = mac.finalize();
        let computed_signature = hex::encode(result.into_bytes());

        if computed_signature == provided_signature {
            debug!("GitHub signature verified successfully");
            true
        } else {
            debug!(
                "Signature mismatch - computed: {}, provided: {}",
                &computed_signature[..8],
                &provided_signature[..8.min(provided_signature.len())]
            );
            false
        }
    }

    /// Check if repository is allowed
    fn is_repo_allowed(&self, repo_full_name: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_repos {
            for pattern in allowed {
                if pattern == "*" {
                    return true;
                }
                if pattern.ends_with("/*") {
                    let org = &pattern[..pattern.len() - 2];
                    if repo_full_name.starts_with(&format!("{}/", org)) {
                        return true;
                    }
                }
                if pattern == repo_full_name {
                    return true;
                }
            }
            false
        } else {
            true // All repos allowed if not configured
        }
    }

    /// Check if event type is allowed
    fn is_event_allowed(&self, event_type: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_events {
            allowed.iter().any(|e| e == event_type || e == "*")
        } else {
            true // All events allowed if not configured
        }
    }

    /// Check if user is allowed
    fn is_user_allowed(&self, username: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_users {
            allowed.contains(&username.to_string())
        } else {
            true // All users allowed if not configured
        }
    }

    /// Parse webhook payload
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<GitHubWebhookPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse GitHub webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid GitHub webhook payload: {}", e))
        })
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_url, path)
    }

    // =========================================================================
    // PUBLIC API METHODS - For use by AgentFlow nodes and handlers
    // =========================================================================

    /// Post a comment on an issue or PR
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `issue_number` - Issue or PR number
    /// * `body` - Comment body (supports Markdown)
    pub async fn post_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i64,
        body: &str,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_comments {
            return Err(PlatformError::ApiError("Comments are disabled".to_string()));
        }

        let url = self.api_url(&format!("/repos/{}/{}/issues/{}/comments", owner, repo, issue_number));

        let payload = serde_json::json!({
            "body": body
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitHubApiError = response.json().await.unwrap_or(GitHubApiError {
                message: "Unknown error".to_string(),
                documentation_url: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        let comment: CommentResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted comment {} to {}/{}#{}", comment.id, owner, repo, issue_number);
        Ok(comment.id)
    }

    /// Post a PR review
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `pr_number` - Pull request number
    /// * `body` - Review body (supports Markdown)
    /// * `event` - Review event: APPROVE, REQUEST_CHANGES, or COMMENT
    pub async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: i64,
        body: &str,
        event: &str,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_reviews {
            return Err(PlatformError::ApiError("Reviews are disabled".to_string()));
        }

        let url = self.api_url(&format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pr_number));

        let payload = serde_json::json!({
            "body": body,
            "event": event.to_uppercase()
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitHubApiError = response.json().await.unwrap_or(GitHubApiError {
                message: "Unknown error".to_string(),
                documentation_url: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        let review: ReviewResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted {} review {} to {}/{}#{}", review.state, review.id, owner, repo, pr_number);
        Ok(review.id)
    }

    /// Create or update a check run
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `head_sha` - Commit SHA to attach check to
    /// * `name` - Check run name
    /// * `status` - Status: queued, in_progress, completed
    /// * `conclusion` - Conclusion (required if status=completed): success, failure, neutral, etc.
    /// * `output` - Optional detailed output
    pub async fn create_check_run(
        &self,
        owner: &str,
        repo: &str,
        head_sha: &str,
        name: &str,
        status: &str,
        conclusion: Option<&str>,
        output: Option<CheckRunOutput>,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_status_checks {
            return Err(PlatformError::ApiError("Status checks are disabled".to_string()));
        }

        let url = self.api_url(&format!("/repos/{}/{}/check-runs", owner, repo));

        let mut payload = serde_json::json!({
            "name": name,
            "head_sha": head_sha,
            "status": status
        });

        if let Some(c) = conclusion {
            payload["conclusion"] = serde_json::json!(c);
        }

        if let Some(o) = output {
            payload["output"] = serde_json::json!({
                "title": o.title,
                "summary": o.summary,
                "text": o.text
            });
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitHubApiError = response.json().await.unwrap_or(GitHubApiError {
                message: "Unknown error".to_string(),
                documentation_url: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        let check_run: CheckRunResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Created check run {} ({}) for {}/{}", check_run.id, check_run.name, owner, repo);
        Ok(check_run.id)
    }

    /// Add labels to an issue or PR
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i64,
        labels: &[String],
    ) -> Result<(), PlatformError> {
        let url = self.api_url(&format!("/repos/{}/{}/issues/{}/labels", owner, repo, issue_number));

        let payload = serde_json::json!({
            "labels": labels
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitHubApiError = response.json().await.unwrap_or(GitHubApiError {
                message: "Unknown error".to_string(),
                documentation_url: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        info!("Added labels {:?} to {}/{}#{}", labels, owner, repo, issue_number);
        Ok(())
    }

    /// Remove a label from an issue or PR
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i64,
        label: &str,
    ) -> Result<(), PlatformError> {
        let url = self.api_url(&format!(
            "/repos/{}/{}/issues/{}/labels/{}",
            owner, repo, issue_number, label
        ));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        // 404 is acceptable (label doesn't exist)
        if !response.status().is_success() && response.status().as_u16() != 404 {
            let error: GitHubApiError = response.json().await.unwrap_or(GitHubApiError {
                message: "Unknown error".to_string(),
                documentation_url: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        info!("Removed label {} from {}/{}#{}", label, owner, repo, issue_number);
        Ok(())
    }

    /// Get the config (for external use)
    pub fn config(&self) -> &GitHubConfig {
        &self.config
    }

    /// Format response for GitHub (Markdown)
    fn format_response_text(&self, response: &TriggerResponse) -> String {
        let status_emoji = match response.status {
            crate::response::ResponseStatus::Success => "✅",
            crate::response::ResponseStatus::Error => "❌",
            crate::response::ResponseStatus::Warning => "⚠️",
            crate::response::ResponseStatus::Info => "ℹ️",
        };

        format!("{} {}", status_emoji, response.text)
    }

    /// Build TriggerMessage from parsed webhook
    fn build_trigger_message(
        &self,
        event_type: &str,
        action: Option<&str>,
        payload: &GitHubWebhookPayload,
    ) -> Result<TriggerMessage, PlatformError> {
        let repo = payload.repository.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing repository in webhook".to_string())
        })?;

        let sender = payload.sender.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing sender in webhook".to_string())
        })?;

        // Build message text based on event type
        let text = match event_type {
            "push" => {
                let ref_name = payload.git_ref.as_deref().unwrap_or("unknown");
                let commit_count = payload.commits.as_ref().map(|c| c.len()).unwrap_or(0);
                let head_msg = payload
                    .head_commit
                    .as_ref()
                    .map(|c| c.message.lines().next().unwrap_or(""))
                    .unwrap_or("");
                format!(
                    "push:{} {} commits to {} - {}",
                    action.unwrap_or("pushed"),
                    commit_count,
                    ref_name,
                    head_msg
                )
            }
            "pull_request" => {
                let pr = payload.pull_request.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing pull_request in PR event".to_string())
                })?;
                format!(
                    "pr:{}:{}:{} #{} {} - {}",
                    action.unwrap_or(""),
                    pr.base.ref_name,
                    pr.head.ref_name,
                    pr.number,
                    pr.title,
                    pr.body.as_deref().unwrap_or("").lines().next().unwrap_or("")
                )
            }
            "issues" => {
                let issue = payload.issue.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing issue in issues event".to_string())
                })?;
                format!(
                    "issue:{}:#{} {} - {}",
                    action.unwrap_or(""),
                    issue.number,
                    issue.title,
                    issue.body.as_deref().unwrap_or("").lines().next().unwrap_or("")
                )
            }
            "issue_comment" => {
                let comment = payload.comment.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing comment in comment event".to_string())
                })?;
                let issue = payload.issue.as_ref();
                let issue_num = issue.map(|i| i.number).unwrap_or(0);
                format!(
                    "comment:{}:#{} {}",
                    action.unwrap_or(""),
                    issue_num,
                    comment.body.lines().next().unwrap_or("")
                )
            }
            "workflow_run" => {
                let run = payload.workflow_run.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing workflow_run in event".to_string())
                })?;
                format!(
                    "workflow:{}:{} {} on {}",
                    action.unwrap_or(""),
                    run.name,
                    run.conclusion.as_deref().unwrap_or(&run.status),
                    run.head_branch
                )
            }
            "check_run" => {
                let check = payload.check_run.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing check_run in event".to_string())
                })?;
                format!(
                    "check:{}:{} {}",
                    action.unwrap_or(""),
                    check.name,
                    check.conclusion.as_deref().unwrap_or(&check.status)
                )
            }
            _ => format!("{}:{}", event_type, action.unwrap_or("")),
        };

        // Build channel_id from repo full name
        let channel_id = repo.full_name.clone();

        // Build user
        let trigger_user = TriggerUser {
            id: sender.id.to_string(),
            username: Some(sender.login.clone()),
            display_name: Some(sender.login.clone()),
            is_bot: sender.user_type.to_lowercase() == "bot",
        };

        // Build metadata with full event details
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), serde_json::json!(event_type));
        metadata.insert("action".to_string(), serde_json::json!(action));
        metadata.insert("repo_id".to_string(), serde_json::json!(repo.id));
        metadata.insert("repo_full_name".to_string(), serde_json::json!(repo.full_name));
        metadata.insert("repo_private".to_string(), serde_json::json!(repo.private));
        metadata.insert("sender_id".to_string(), serde_json::json!(sender.id));
        metadata.insert("sender_login".to_string(), serde_json::json!(sender.login));

        // Add PR-specific metadata
        if let Some(ref pr) = payload.pull_request {
            metadata.insert("pr_number".to_string(), serde_json::json!(pr.number));
            metadata.insert("pr_title".to_string(), serde_json::json!(pr.title));
            metadata.insert("pr_state".to_string(), serde_json::json!(pr.state));
            metadata.insert("pr_draft".to_string(), serde_json::json!(pr.draft));
            metadata.insert("pr_base_ref".to_string(), serde_json::json!(pr.base.ref_name));
            metadata.insert("pr_head_ref".to_string(), serde_json::json!(pr.head.ref_name));
            metadata.insert("pr_head_sha".to_string(), serde_json::json!(pr.head.sha));
            metadata.insert("pr_additions".to_string(), serde_json::json!(pr.additions));
            metadata.insert("pr_deletions".to_string(), serde_json::json!(pr.deletions));
            metadata.insert("pr_changed_files".to_string(), serde_json::json!(pr.changed_files));
            metadata.insert("pr_html_url".to_string(), serde_json::json!(pr.html_url));
        }

        // Add issue-specific metadata
        if let Some(ref issue) = payload.issue {
            metadata.insert("issue_number".to_string(), serde_json::json!(issue.number));
            metadata.insert("issue_title".to_string(), serde_json::json!(issue.title));
            metadata.insert("issue_state".to_string(), serde_json::json!(issue.state));
            metadata.insert("issue_html_url".to_string(), serde_json::json!(issue.html_url));
        }

        // Add push-specific metadata
        if let Some(ref git_ref) = payload.git_ref {
            metadata.insert("ref".to_string(), serde_json::json!(git_ref));
        }
        if let Some(ref before) = payload.before {
            metadata.insert("before_sha".to_string(), serde_json::json!(before));
        }
        if let Some(ref after) = payload.after {
            metadata.insert("after_sha".to_string(), serde_json::json!(after));
        }
        if let Some(ref commits) = payload.commits {
            metadata.insert("commit_count".to_string(), serde_json::json!(commits.len()));
        }

        // Build message ID from event type and unique identifiers
        let message_id = if let Some(ref pr) = payload.pull_request {
            format!("gh-{}-{}-pr-{}", repo.id, event_type, pr.id)
        } else if let Some(ref issue) = payload.issue {
            format!("gh-{}-{}-issue-{}", repo.id, event_type, issue.id)
        } else if let Some(ref after) = payload.after {
            format!("gh-{}-{}-{}", repo.id, event_type, &after[..8.min(after.len())])
        } else {
            format!("gh-{}-{}-{}", repo.id, event_type, chrono::Utc::now().timestamp_millis())
        };

        // Thread ID for PRs and issues
        let thread_id = if let Some(ref pr) = payload.pull_request {
            Some(format!("pr-{}", pr.number))
        } else if let Some(ref issue) = payload.issue {
            Some(format!("issue-{}", issue.number))
        } else {
            None
        };

        Ok(TriggerMessage {
            id: message_id,
            platform: "github".to_string(),
            channel_id,
            user: trigger_user,
            text,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id,
            reply_to: None,
        })
    }
}

/// Check run output for detailed status
#[derive(Debug, Clone)]
pub struct CheckRunOutput {
    pub title: String,
    pub summary: String,
    pub text: Option<String>,
}

// ============================================================================
// TRIGGER PLATFORM TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl TriggerPlatform for GitHubPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Get event type from header
        let event_type = headers
            .get("x-github-event")
            .ok_or_else(|| PlatformError::ParseError("Missing X-GitHub-Event header".to_string()))?;

        // Verify signature
        if let Some(signature) = headers.get("x-hub-signature-256") {
            if !self.verify_github_signature(raw, signature) {
                warn!("Invalid GitHub signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        } else {
            warn!("Missing X-Hub-Signature-256 header");
            return Err(PlatformError::InvalidSignature(
                "Missing signature header".to_string(),
            ));
        }

        // Check if event type is allowed
        if !self.is_event_allowed(event_type) {
            info!("Event type {} not allowed", event_type);
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Parse payload
        let payload = self.parse_webhook_payload(raw)?;

        // Check if repo is allowed
        if let Some(ref repo) = payload.repository {
            if !self.is_repo_allowed(&repo.full_name) {
                info!("Repository {} not allowed", repo.full_name);
                return Err(PlatformError::InvalidSignature(
                    "Repository not allowed".to_string(),
                ));
            }
        }

        // Check if user is allowed
        if let Some(ref sender) = payload.sender {
            if !self.is_user_allowed(&sender.login) {
                info!("User {} not allowed", sender.login);
                return Err(PlatformError::InvalidSignature(
                    "User not allowed".to_string(),
                ));
            }
        }

        // Build trigger message
        let action = payload.action.as_deref();
        self.build_trigger_message(event_type, action, &payload)
    }

    async fn send_response(
        &self,
        channel: &str, // format: owner/repo or owner/repo#number
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Parse channel format: owner/repo#number
        let parts: Vec<&str> = channel.splitn(2, '#').collect();
        let repo_parts: Vec<&str> = parts[0].splitn(2, '/').collect();

        if repo_parts.len() != 2 {
            return Err(PlatformError::ParseError(format!(
                "Invalid channel format: {}. Expected owner/repo or owner/repo#number",
                channel
            )));
        }

        let owner = repo_parts[0];
        let repo = repo_parts[1];
        let text = self.format_response_text(&response);

        // If we have a number (PR or issue), post a comment
        if parts.len() == 2 {
            let number: i64 = parts[1]
                .parse()
                .map_err(|_| PlatformError::ParseError("Invalid issue/PR number".to_string()))?;

            self.post_comment(owner, repo, number, &text).await?;
        } else {
            // No number - we can't post without a target
            // This might be a push event - we could create a commit comment
            info!("No issue/PR number in channel, skipping response");
        }

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "github"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_github_signature(payload, signature)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // GitHub supports conversation threads on PRs/issues
    }

    fn supports_interactive(&self) -> bool {
        true // GitHub supports reactions and workflows
    }

    fn supports_files(&self) -> bool {
        true // GitHub can attach files via releases/artifacts
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> GitHubConfig {
        GitHubConfig {
            token: "ghp_test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-aof-bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        }
    }

    #[test]
    fn test_github_platform_new() {
        let config = create_test_config();
        let platform = GitHubPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_github_platform_invalid_config() {
        let config = GitHubConfig {
            token: "".to_string(),
            webhook_secret: "".to_string(),
            bot_name: "".to_string(),
            api_url: "".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };
        let platform = GitHubPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_repo_allowed_all() {
        let config = create_test_config();
        let platform = GitHubPlatform::new(config).unwrap();

        assert!(platform.is_repo_allowed("owner/repo"));
        assert!(platform.is_repo_allowed("any/repo"));
    }

    #[test]
    fn test_repo_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_repos = Some(vec![
            "owner/specific-repo".to_string(),
            "org/*".to_string(),
        ]);

        let platform = GitHubPlatform::new(config).unwrap();

        assert!(platform.is_repo_allowed("owner/specific-repo"));
        assert!(platform.is_repo_allowed("org/any-repo"));
        assert!(!platform.is_repo_allowed("other/repo"));
    }

    #[test]
    fn test_event_allowed_all() {
        let config = create_test_config();
        let platform = GitHubPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("push"));
        assert!(platform.is_event_allowed("pull_request"));
        assert!(platform.is_event_allowed("issues"));
    }

    #[test]
    fn test_event_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_events = Some(vec!["push".to_string(), "pull_request".to_string()]);

        let platform = GitHubPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("push"));
        assert!(platform.is_event_allowed("pull_request"));
        assert!(!platform.is_event_allowed("issues"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = GitHubPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "github");
        assert_eq!(platform.bot_name(), "test-aof-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(GitHubEventType::from("push"), GitHubEventType::Push);
        assert_eq!(GitHubEventType::from("pull_request"), GitHubEventType::PullRequest);
        assert_eq!(GitHubEventType::from("issues"), GitHubEventType::Issues);
        assert_eq!(GitHubEventType::from("unknown_event"), GitHubEventType::Unknown);
    }

    #[tokio::test]
    async fn test_verify_signature_format() {
        let config = create_test_config();
        let platform = GitHubPlatform::new(config).unwrap();

        let payload = b"test payload";
        let invalid_sig = "invalid";

        let result = platform.verify_signature(payload, invalid_sig).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_parse_pr_webhook() {
        let webhook_json = r#"{
            "action": "opened",
            "pull_request": {
                "id": 123,
                "number": 42,
                "title": "Test PR",
                "state": "open",
                "draft": false,
                "merged": false,
                "html_url": "https://github.com/owner/repo/pull/42",
                "additions": 10,
                "deletions": 5,
                "changed_files": 3,
                "user": {
                    "id": 456,
                    "login": "testuser",
                    "type": "User"
                },
                "base": {
                    "ref": "main",
                    "sha": "abc123"
                },
                "head": {
                    "ref": "feature-branch",
                    "sha": "def456"
                }
            },
            "repository": {
                "id": 789,
                "name": "repo",
                "full_name": "owner/repo",
                "private": false
            },
            "sender": {
                "id": 456,
                "login": "testuser",
                "type": "User"
            }
        }"#;

        // Create test signature
        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(webhook_json.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let mut headers = HashMap::new();
        headers.insert("x-github-event".to_string(), "pull_request".to_string());
        headers.insert("x-hub-signature-256".to_string(), signature);

        let config = create_test_config();
        let platform = GitHubPlatform::new(config).unwrap();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "github");
        assert_eq!(message.channel_id, "owner/repo");
        assert!(message.text.contains("pr:opened"));
        assert_eq!(message.user.id, "456");
        assert_eq!(message.user.username, Some("testuser".to_string()));
        assert_eq!(message.metadata.get("pr_number").unwrap(), &serde_json::json!(42));
    }
}
