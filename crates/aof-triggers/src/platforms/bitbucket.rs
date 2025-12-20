//! Bitbucket Webhook adapter for AOF
//!
//! This module provides integration with Bitbucket's Webhook API, supporting:
//! - Push events
//! - Pull request events (created, updated, merged, declined)
//! - Issue events
//! - Build status events
//! - PR comment events
//! - HMAC-SHA256 signature verification
//! - PR approvals and comments
//!
//! # Design Philosophy
//!
//! This platform is designed as a **pluggable component** that can be:
//! - Enabled/disabled via Cargo feature flags
//! - Extended with custom event handlers
//! - Integrated with AgentFlow workflows
//! - Used with both Bitbucket Cloud and Server/Data Center
//!
//! # Example Usage
//!
//! ```yaml
//! apiVersion: aof.dev/v1
//! kind: AgentFlow
//! spec:
//!   trigger:
//!     type: Bitbucket
//!     config:
//!       events:
//!         - pullrequest:created
//!         - repo:push
//!       username: myuser
//!       app_password_env: BITBUCKET_APP_PASSWORD
//!       webhook_secret_env: BITBUCKET_WEBHOOK_SECRET
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

/// Bitbucket platform adapter
///
/// Implements the `TriggerPlatform` trait for Bitbucket webhooks.
/// Supports multiple event types and provides methods for interacting
/// with the Bitbucket API (posting comments, approvals, build statuses).
pub struct BitbucketPlatform {
    config: BitbucketConfig,
    client: reqwest::Client,
}

/// Bitbucket configuration
///
/// All configuration options for the Bitbucket platform.
/// Supports environment variable resolution for secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketConfig {
    /// Bitbucket username for API authentication
    pub username: String,

    /// App Password or OAuth token for API calls
    /// App passwords recommended for Cloud: https://bitbucket.org/account/settings/app-passwords/
    #[serde(default)]
    pub app_password: Option<String>,

    /// OAuth token (alternative to app_password)
    #[serde(default)]
    pub oauth_token: Option<String>,

    /// Webhook secret for HMAC-SHA256 signature verification
    /// Required for secure webhook validation
    pub webhook_secret: String,

    /// Bot/App name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// API base URL
    /// - Cloud: https://api.bitbucket.org/2.0 (default)
    /// - Server/DC: https://bitbucket.example.com/rest/api/1.0
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Allowed repository filter (optional whitelist)
    /// Format: ["workspace/repo", "workspace/*", "*"]
    #[serde(default)]
    pub allowed_repos: Option<Vec<String>>,

    /// Event types to handle (optional filter)
    /// If empty, handles all events
    #[serde(default)]
    pub allowed_events: Option<Vec<String>>,

    /// User UUIDs allowed to trigger actions (optional whitelist)
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,

    /// Enable posting PR comments
    #[serde(default = "default_true")]
    pub enable_comments: bool,

    /// Enable PR approvals
    #[serde(default = "default_true")]
    pub enable_approvals: bool,

    /// Enable build status updates
    #[serde(default = "default_true")]
    pub enable_build_status: bool,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_api_url() -> String {
    "https://api.bitbucket.org/2.0".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// WEBHOOK PAYLOAD TYPES
// ============================================================================

/// Bitbucket webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BitbucketEventType {
    /// Pull request created
    #[serde(rename = "pullrequest:created")]
    PullRequestCreated,

    /// Pull request updated
    #[serde(rename = "pullrequest:updated")]
    PullRequestUpdated,

    /// Pull request approved
    #[serde(rename = "pullrequest:approved")]
    PullRequestApproved,

    /// Pull request unapproved
    #[serde(rename = "pullrequest:unapproved")]
    PullRequestUnapproved,

    /// Pull request merged
    #[serde(rename = "pullrequest:fulfilled")]
    PullRequestMerged,

    /// Pull request declined
    #[serde(rename = "pullrequest:rejected")]
    PullRequestDeclined,

    /// Pull request comment created
    #[serde(rename = "pullrequest:comment_created")]
    PullRequestCommentCreated,

    /// Pull request comment updated
    #[serde(rename = "pullrequest:comment_updated")]
    PullRequestCommentUpdated,

    /// Pull request comment deleted
    #[serde(rename = "pullrequest:comment_deleted")]
    PullRequestCommentDeleted,

    /// Repository push
    #[serde(rename = "repo:push")]
    RepoPush,

    /// Repository fork
    #[serde(rename = "repo:fork")]
    RepoFork,

    /// Repository updated
    #[serde(rename = "repo:updated")]
    RepoUpdated,

    /// Repository commit comment created
    #[serde(rename = "repo:commit_comment_created")]
    RepoCommitCommentCreated,

    /// Repository commit status created
    #[serde(rename = "repo:commit_status_created")]
    RepoCommitStatusCreated,

    /// Repository commit status updated
    #[serde(rename = "repo:commit_status_updated")]
    RepoCommitStatusUpdated,

    /// Issue created
    #[serde(rename = "issue:created")]
    IssueCreated,

    /// Issue updated
    #[serde(rename = "issue:updated")]
    IssueUpdated,

    /// Issue comment created
    #[serde(rename = "issue:comment_created")]
    IssueCommentCreated,

    /// Build status created
    #[serde(rename = "build:status_created")]
    BuildStatusCreated,

    /// Build status updated
    #[serde(rename = "build:status_updated")]
    BuildStatusUpdated,

    /// Unknown event type
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for BitbucketEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PullRequestCreated => write!(f, "pullrequest:created"),
            Self::PullRequestUpdated => write!(f, "pullrequest:updated"),
            Self::PullRequestApproved => write!(f, "pullrequest:approved"),
            Self::PullRequestUnapproved => write!(f, "pullrequest:unapproved"),
            Self::PullRequestMerged => write!(f, "pullrequest:fulfilled"),
            Self::PullRequestDeclined => write!(f, "pullrequest:rejected"),
            Self::PullRequestCommentCreated => write!(f, "pullrequest:comment_created"),
            Self::PullRequestCommentUpdated => write!(f, "pullrequest:comment_updated"),
            Self::PullRequestCommentDeleted => write!(f, "pullrequest:comment_deleted"),
            Self::RepoPush => write!(f, "repo:push"),
            Self::RepoFork => write!(f, "repo:fork"),
            Self::RepoUpdated => write!(f, "repo:updated"),
            Self::RepoCommitCommentCreated => write!(f, "repo:commit_comment_created"),
            Self::RepoCommitStatusCreated => write!(f, "repo:commit_status_created"),
            Self::RepoCommitStatusUpdated => write!(f, "repo:commit_status_updated"),
            Self::IssueCreated => write!(f, "issue:created"),
            Self::IssueUpdated => write!(f, "issue:updated"),
            Self::IssueCommentCreated => write!(f, "issue:comment_created"),
            Self::BuildStatusCreated => write!(f, "build:status_created"),
            Self::BuildStatusUpdated => write!(f, "build:status_updated"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for BitbucketEventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pullrequest:created" => Self::PullRequestCreated,
            "pullrequest:updated" => Self::PullRequestUpdated,
            "pullrequest:approved" => Self::PullRequestApproved,
            "pullrequest:unapproved" => Self::PullRequestUnapproved,
            "pullrequest:fulfilled" => Self::PullRequestMerged,
            "pullrequest:rejected" => Self::PullRequestDeclined,
            "pullrequest:comment_created" => Self::PullRequestCommentCreated,
            "pullrequest:comment_updated" => Self::PullRequestCommentUpdated,
            "pullrequest:comment_deleted" => Self::PullRequestCommentDeleted,
            "repo:push" => Self::RepoPush,
            "repo:fork" => Self::RepoFork,
            "repo:updated" => Self::RepoUpdated,
            "repo:commit_comment_created" => Self::RepoCommitCommentCreated,
            "repo:commit_status_created" => Self::RepoCommitStatusCreated,
            "repo:commit_status_updated" => Self::RepoCommitStatusUpdated,
            "issue:created" => Self::IssueCreated,
            "issue:updated" => Self::IssueUpdated,
            "issue:comment_created" => Self::IssueCommentCreated,
            "build:status_created" => Self::BuildStatusCreated,
            "build:status_updated" => Self::BuildStatusUpdated,
            _ => Self::Unknown,
        }
    }
}

/// Common repository information
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub uuid: String,
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub links: Option<RepositoryLinks>,
}

/// Repository links
#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryLinks {
    #[serde(default)]
    pub html: Option<Link>,
    #[serde(default)]
    pub clone: Option<Vec<CloneLink>>,
}

/// Generic link object
#[derive(Debug, Clone, Deserialize)]
pub struct Link {
    pub href: String,
}

/// Clone link with name
#[derive(Debug, Clone, Deserialize)]
pub struct CloneLink {
    pub name: String,
    pub href: String,
}

/// Bitbucket user/actor information
#[derive(Debug, Clone, Deserialize)]
pub struct Actor {
    pub uuid: String,
    pub display_name: String,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(rename = "type")]
    pub actor_type: String,
    #[serde(default)]
    pub links: Option<ActorLinks>,
}

/// Actor links
#[derive(Debug, Clone, Deserialize)]
pub struct ActorLinks {
    #[serde(default)]
    pub html: Option<Link>,
    #[serde(default)]
    pub avatar: Option<Link>,
}

/// Pull request information
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub state: String,
    pub author: Actor,
    pub source: PullRequestBranch,
    pub destination: PullRequestBranch,
    #[serde(default)]
    pub merge_commit: Option<Commit>,
    #[serde(default)]
    pub close_source_branch: bool,
    #[serde(default)]
    pub closed_by: Option<Actor>,
    pub created_on: String,
    pub updated_on: String,
    #[serde(default)]
    pub links: Option<PullRequestLinks>,
}

/// Pull request branch info
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestBranch {
    pub branch: Branch,
    pub commit: Commit,
    pub repository: Repository,
}

/// Branch information
#[derive(Debug, Clone, Deserialize)]
pub struct Branch {
    pub name: String,
}

/// Commit information
#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub hash: String,
    #[serde(rename = "type")]
    pub commit_type: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub author: Option<CommitAuthor>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub links: Option<CommitLinks>,
}

/// Commit author
#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub user: Option<Actor>,
}

/// Commit links
#[derive(Debug, Clone, Deserialize)]
pub struct CommitLinks {
    #[serde(default)]
    pub html: Option<Link>,
}

/// Pull request links
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestLinks {
    #[serde(default)]
    pub html: Option<Link>,
    #[serde(default)]
    pub comments: Option<Link>,
    #[serde(default)]
    pub commits: Option<Link>,
    #[serde(default)]
    pub approve: Option<Link>,
}

/// Issue information
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub content: Option<IssueContent>,
    pub state: String,
    pub kind: String,
    pub priority: String,
    pub reporter: Actor,
    #[serde(default)]
    pub assignee: Option<Actor>,
    pub created_on: String,
    pub updated_on: String,
    #[serde(default)]
    pub links: Option<IssueLinks>,
}

/// Issue content
#[derive(Debug, Clone, Deserialize)]
pub struct IssueContent {
    pub raw: String,
    #[serde(default)]
    pub markup: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
}

/// Issue links
#[derive(Debug, Clone, Deserialize)]
pub struct IssueLinks {
    #[serde(default)]
    pub html: Option<Link>,
}

/// Comment information
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub content: CommentContent,
    pub user: Actor,
    pub created_on: String,
    #[serde(default)]
    pub updated_on: Option<String>,
    #[serde(default)]
    pub links: Option<CommentLinks>,
}

/// Comment content
#[derive(Debug, Clone, Deserialize)]
pub struct CommentContent {
    pub raw: String,
    #[serde(default)]
    pub markup: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
}

/// Comment links
#[derive(Debug, Clone, Deserialize)]
pub struct CommentLinks {
    #[serde(default)]
    pub html: Option<Link>,
}

/// Push event information
#[derive(Debug, Clone, Deserialize)]
pub struct Push {
    pub changes: Vec<PushChange>,
}

/// Push change information
#[derive(Debug, Clone, Deserialize)]
pub struct PushChange {
    #[serde(default)]
    pub new: Option<PushTarget>,
    #[serde(default)]
    pub old: Option<PushTarget>,
    #[serde(default)]
    pub created: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub forced: bool,
    #[serde(default)]
    pub commits: Option<Vec<Commit>>,
}

/// Push target (branch or tag)
#[derive(Debug, Clone, Deserialize)]
pub struct PushTarget {
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub target: Commit,
}

/// Build status information
#[derive(Debug, Clone, Deserialize)]
pub struct BuildStatus {
    pub key: String,
    pub state: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    pub commit: Commit,
    pub created_on: String,
    pub updated_on: String,
}

// ============================================================================
// WEBHOOK PAYLOAD
// ============================================================================

/// Generic Bitbucket webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct BitbucketWebhookPayload {
    /// Repository information
    #[serde(default)]
    pub repository: Option<Repository>,

    /// Actor (user who triggered the event)
    #[serde(default)]
    pub actor: Option<Actor>,

    /// Pull request (for PR events)
    #[serde(default)]
    pub pullrequest: Option<PullRequest>,

    /// Issue (for issue events)
    #[serde(default)]
    pub issue: Option<Issue>,

    /// Comment (for comment events)
    #[serde(default)]
    pub comment: Option<Comment>,

    /// Push information (for push events)
    #[serde(default)]
    pub push: Option<Push>,

    /// Commit status (for build status events)
    #[serde(default)]
    pub commit_status: Option<BuildStatus>,
}

// ============================================================================
// BITBUCKET API RESPONSE TYPES
// ============================================================================

/// Bitbucket API error response
#[derive(Debug, Deserialize)]
struct BitbucketApiError {
    #[serde(default)]
    error: Option<ErrorDetail>,
}

/// Error detail
#[derive(Debug, Deserialize)]
struct ErrorDetail {
    message: String,
}

/// Comment creation response
#[derive(Debug, Deserialize)]
struct CommentResponse {
    id: i64,
    #[serde(default)]
    links: Option<CommentLinks>,
}

/// Approval response
#[derive(Debug, Deserialize)]
struct ApprovalResponse {
    approved: bool,
    user: Actor,
}

/// Build status response
#[derive(Debug, Deserialize)]
struct BuildStatusResponse {
    key: String,
    state: String,
    #[serde(default)]
    url: Option<String>,
}

// ============================================================================
// PLATFORM IMPLEMENTATION
// ============================================================================

impl BitbucketPlatform {
    /// Create new Bitbucket platform adapter
    ///
    /// # Errors
    /// Returns error if authentication credentials or webhook_secret is missing
    pub fn new(config: BitbucketConfig) -> Result<Self, PlatformError> {
        if config.app_password.is_none() && config.oauth_token.is_none() {
            return Err(PlatformError::ParseError(
                "Either app_password or oauth_token is required".to_string(),
            ));
        }

        if config.webhook_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "Bitbucket webhook secret is required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("AOF/{} ({})", env!("CARGO_PKG_VERSION"), config.bot_name))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify HMAC-SHA256 signature from Bitbucket webhook
    ///
    /// Bitbucket sends signature in X-Hub-Signature header (same format as GitHub)
    fn verify_bitbucket_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Bitbucket signature format: sha256=<hex_signature>
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
            debug!("Bitbucket signature verified successfully");
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
                    let workspace = &pattern[..pattern.len() - 2];
                    if repo_full_name.starts_with(&format!("{}/", workspace)) {
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
    fn is_user_allowed(&self, user_uuid: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_users {
            allowed.contains(&user_uuid.to_string())
        } else {
            true // All users allowed if not configured
        }
    }

    /// Parse webhook payload
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<BitbucketWebhookPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse Bitbucket webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid Bitbucket webhook payload: {}", e))
        })
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_url, path)
    }

    /// Get authorization header
    fn auth_header(&self) -> String {
        if let Some(ref token) = self.config.oauth_token {
            format!("Bearer {}", token)
        } else if let Some(ref password) = self.config.app_password {
            use base64::Engine;
            let credentials = format!("{}:{}", self.config.username, password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
            format!("Basic {}", encoded)
        } else {
            String::new()
        }
    }

    // =========================================================================
    // PUBLIC API METHODS - For use by AgentFlow nodes and handlers
    // =========================================================================

    /// Post a comment on a pull request
    ///
    /// # Arguments
    /// * `workspace` - Repository workspace/owner
    /// * `repo_slug` - Repository slug/name
    /// * `pr_id` - Pull request ID
    /// * `body` - Comment body (supports Markdown)
    pub async fn post_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: i64,
        body: &str,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_comments {
            return Err(PlatformError::ApiError("Comments are disabled".to_string()));
        }

        let url = self.api_url(&format!(
            "/repositories/{}/{}/pullrequests/{}/comments",
            workspace, repo_slug, pr_id
        ));

        let payload = serde_json::json!({
            "content": {
                "raw": body
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: BitbucketApiError = response.json().await.unwrap_or(BitbucketApiError {
                error: Some(ErrorDetail {
                    message: "Unknown error".to_string(),
                }),
            });
            return Err(PlatformError::ApiError(
                error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let comment: CommentResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted comment {} to {}/{}#{}", comment.id, workspace, repo_slug, pr_id);
        Ok(comment.id)
    }

    /// Approve a pull request
    ///
    /// # Arguments
    /// * `workspace` - Repository workspace/owner
    /// * `repo_slug` - Repository slug/name
    /// * `pr_id` - Pull request ID
    pub async fn approve_pr(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: i64,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_approvals {
            return Err(PlatformError::ApiError("Approvals are disabled".to_string()));
        }

        let url = self.api_url(&format!(
            "/repositories/{}/{}/pullrequests/{}/approve",
            workspace, repo_slug, pr_id
        ));

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: BitbucketApiError = response.json().await.unwrap_or(BitbucketApiError {
                error: Some(ErrorDetail {
                    message: "Unknown error".to_string(),
                }),
            });
            return Err(PlatformError::ApiError(
                error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        info!("Approved PR {}/{}#{}", workspace, repo_slug, pr_id);
        Ok(())
    }

    /// Remove approval from a pull request
    ///
    /// # Arguments
    /// * `workspace` - Repository workspace/owner
    /// * `repo_slug` - Repository slug/name
    /// * `pr_id` - Pull request ID
    pub async fn unapprove_pr(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: i64,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_approvals {
            return Err(PlatformError::ApiError("Approvals are disabled".to_string()));
        }

        let url = self.api_url(&format!(
            "/repositories/{}/{}/pullrequests/{}/approve",
            workspace, repo_slug, pr_id
        ));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            let error: BitbucketApiError = response.json().await.unwrap_or(BitbucketApiError {
                error: Some(ErrorDetail {
                    message: "Unknown error".to_string(),
                }),
            });
            return Err(PlatformError::ApiError(
                error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        info!("Removed approval from PR {}/{}#{}", workspace, repo_slug, pr_id);
        Ok(())
    }

    /// Add a default reviewer to a pull request
    ///
    /// # Arguments
    /// * `workspace` - Repository workspace/owner
    /// * `repo_slug` - Repository slug/name
    /// * `pr_id` - Pull request ID
    /// * `reviewer_uuid` - UUID of the reviewer
    pub async fn add_default_reviewer(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: i64,
        reviewer_uuid: &str,
    ) -> Result<(), PlatformError> {
        let url = self.api_url(&format!(
            "/repositories/{}/{}/pullrequests/{}/default-reviewers/{}",
            workspace, repo_slug, pr_id, reviewer_uuid
        ));

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: BitbucketApiError = response.json().await.unwrap_or(BitbucketApiError {
                error: Some(ErrorDetail {
                    message: "Unknown error".to_string(),
                }),
            });
            return Err(PlatformError::ApiError(
                error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        info!("Added reviewer {} to PR {}/{}#{}", reviewer_uuid, workspace, repo_slug, pr_id);
        Ok(())
    }

    /// Create or update a build status
    ///
    /// # Arguments
    /// * `workspace` - Repository workspace/owner
    /// * `repo_slug` - Repository slug/name
    /// * `commit_hash` - Commit hash to attach status to
    /// * `key` - Unique key for this build status
    /// * `state` - State: INPROGRESS, SUCCESSFUL, FAILED
    /// * `name` - Build name
    /// * `description` - Optional description
    /// * `url` - Optional URL to build results
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
    ) -> Result<(), PlatformError> {
        if !self.config.enable_build_status {
            return Err(PlatformError::ApiError("Build status is disabled".to_string()));
        }

        let api_url = self.api_url(&format!(
            "/repositories/{}/{}/commit/{}/statuses/build",
            workspace, repo_slug, commit_hash
        ));

        let mut payload = serde_json::json!({
            "key": key,
            "state": state.to_uppercase(),
            "name": name
        });

        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }

        if let Some(u) = url {
            payload["url"] = serde_json::json!(u);
        }

        let response = self
            .client
            .post(&api_url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: BitbucketApiError = response.json().await.unwrap_or(BitbucketApiError {
                error: Some(ErrorDetail {
                    message: "Unknown error".to_string(),
                }),
            });
            return Err(PlatformError::ApiError(
                error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        info!("Created build status {} ({}) for {}/{}", key, state, workspace, repo_slug);
        Ok(())
    }

    /// Get the config (for external use)
    pub fn config(&self) -> &BitbucketConfig {
        &self.config
    }

    /// Format response for Bitbucket (Markdown)
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
        payload: &BitbucketWebhookPayload,
    ) -> Result<TriggerMessage, PlatformError> {
        let repo = payload.repository.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing repository in webhook".to_string())
        })?;

        let actor = payload.actor.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing actor in webhook".to_string())
        })?;

        // Build message text based on event type
        let text = match event_type {
            "repo:push" => {
                if let Some(ref push) = payload.push {
                    let change = push.changes.first();
                    let branch_name = change
                        .and_then(|c| c.new.as_ref())
                        .map(|n| n.name.as_str())
                        .unwrap_or("unknown");
                    let commit_count = change.and_then(|c| c.commits.as_ref()).map(|cs| cs.len()).unwrap_or(0);
                    format!("push:{} commits to {}", commit_count, branch_name)
                } else {
                    "push:unknown".to_string()
                }
            }
            "pullrequest:created" | "pullrequest:updated" | "pullrequest:fulfilled" | "pullrequest:rejected" => {
                let pr = payload.pullrequest.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing pullrequest in PR event".to_string())
                })?;
                let action = event_type.split(':').nth(1).unwrap_or("");
                format!(
                    "pr:{}:{}:{} #{} {} - {}",
                    action,
                    pr.destination.branch.name,
                    pr.source.branch.name,
                    pr.id,
                    pr.title,
                    pr.description.as_deref().unwrap_or("").lines().next().unwrap_or("")
                )
            }
            "issue:created" | "issue:updated" => {
                let issue = payload.issue.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing issue in issue event".to_string())
                })?;
                let action = event_type.split(':').nth(1).unwrap_or("");
                format!(
                    "issue:{}:#{} {} - {}",
                    action,
                    issue.id,
                    issue.title,
                    issue.content.as_ref().map(|c| c.raw.lines().next().unwrap_or("")).unwrap_or("")
                )
            }
            "pullrequest:comment_created" | "pullrequest:comment_updated" => {
                let comment = payload.comment.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing comment in comment event".to_string())
                })?;
                let pr = payload.pullrequest.as_ref();
                let pr_id = pr.map(|p| p.id).unwrap_or(0);
                let action = event_type.split(':').nth(1).unwrap_or("");
                format!(
                    "comment:{}:#{} {}",
                    action,
                    pr_id,
                    comment.content.raw.lines().next().unwrap_or("")
                )
            }
            _ => event_type.to_string(),
        };

        // Build channel_id from repo full name
        let channel_id = repo.full_name.clone();

        // Build user
        let trigger_user = TriggerUser {
            id: actor.uuid.clone(),
            username: actor.nickname.clone(),
            display_name: Some(actor.display_name.clone()),
            is_bot: actor.actor_type.to_lowercase() == "bot",
        };

        // Build metadata with full event details
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), serde_json::json!(event_type));
        metadata.insert("repo_uuid".to_string(), serde_json::json!(repo.uuid));
        metadata.insert("repo_full_name".to_string(), serde_json::json!(repo.full_name));
        metadata.insert("repo_private".to_string(), serde_json::json!(repo.is_private));
        metadata.insert("actor_uuid".to_string(), serde_json::json!(actor.uuid));
        metadata.insert("actor_display_name".to_string(), serde_json::json!(actor.display_name));

        // Add PR-specific metadata
        if let Some(ref pr) = payload.pullrequest {
            metadata.insert("pr_id".to_string(), serde_json::json!(pr.id));
            metadata.insert("pr_title".to_string(), serde_json::json!(pr.title));
            metadata.insert("pr_state".to_string(), serde_json::json!(pr.state));
            metadata.insert("pr_source_branch".to_string(), serde_json::json!(pr.source.branch.name));
            metadata.insert("pr_dest_branch".to_string(), serde_json::json!(pr.destination.branch.name));
            metadata.insert("pr_source_commit".to_string(), serde_json::json!(pr.source.commit.hash));
            metadata.insert("pr_dest_commit".to_string(), serde_json::json!(pr.destination.commit.hash));
            if let Some(ref links) = pr.links {
                if let Some(ref html) = links.html {
                    metadata.insert("pr_html_url".to_string(), serde_json::json!(html.href));
                }
            }
        }

        // Add issue-specific metadata
        if let Some(ref issue) = payload.issue {
            metadata.insert("issue_id".to_string(), serde_json::json!(issue.id));
            metadata.insert("issue_title".to_string(), serde_json::json!(issue.title));
            metadata.insert("issue_state".to_string(), serde_json::json!(issue.state));
            metadata.insert("issue_kind".to_string(), serde_json::json!(issue.kind));
            if let Some(ref links) = issue.links {
                if let Some(ref html) = links.html {
                    metadata.insert("issue_html_url".to_string(), serde_json::json!(html.href));
                }
            }
        }

        // Add push-specific metadata
        if let Some(ref push) = payload.push {
            if let Some(change) = push.changes.first() {
                if let Some(ref new) = change.new {
                    metadata.insert("branch".to_string(), serde_json::json!(new.name));
                    metadata.insert("commit_hash".to_string(), serde_json::json!(new.target.hash));
                }
                if let Some(ref commits) = change.commits {
                    metadata.insert("commit_count".to_string(), serde_json::json!(commits.len()));
                }
            }
        }

        // Build message ID from event type and unique identifiers
        let message_id = if let Some(ref pr) = payload.pullrequest {
            format!("bb-{}-{}-pr-{}", repo.uuid, event_type, pr.id)
        } else if let Some(ref issue) = payload.issue {
            format!("bb-{}-{}-issue-{}", repo.uuid, event_type, issue.id)
        } else if let Some(ref push) = payload.push {
            let hash = push.changes.first()
                .and_then(|c| c.new.as_ref())
                .map(|n| &n.target.hash[..8.min(n.target.hash.len())])
                .unwrap_or("unknown");
            format!("bb-{}-{}-{}", repo.uuid, event_type, hash)
        } else {
            format!("bb-{}-{}-{}", repo.uuid, event_type, chrono::Utc::now().timestamp_millis())
        };

        // Thread ID for PRs and issues
        let thread_id = if let Some(ref pr) = payload.pullrequest {
            Some(format!("pr-{}", pr.id))
        } else if let Some(ref issue) = payload.issue {
            Some(format!("issue-{}", issue.id))
        } else {
            None
        };

        Ok(TriggerMessage {
            id: message_id,
            platform: "bitbucket".to_string(),
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

// ============================================================================
// TRIGGER PLATFORM TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl TriggerPlatform for BitbucketPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Get event type from header
        let event_type = headers
            .get("x-event-key")
            .ok_or_else(|| PlatformError::ParseError("Missing X-Event-Key header".to_string()))?;

        // Verify signature
        if let Some(signature) = headers.get("x-hub-signature") {
            if !self.verify_bitbucket_signature(raw, signature) {
                warn!("Invalid Bitbucket signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        } else {
            warn!("Missing X-Hub-Signature header");
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
        if let Some(ref actor) = payload.actor {
            if !self.is_user_allowed(&actor.uuid) {
                info!("User {} not allowed", actor.display_name);
                return Err(PlatformError::InvalidSignature(
                    "User not allowed".to_string(),
                ));
            }
        }

        // Build trigger message
        self.build_trigger_message(event_type, &payload)
    }

    async fn send_response(
        &self,
        channel: &str, // format: workspace/repo or workspace/repo#number
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Parse channel format: workspace/repo#pr_id
        let parts: Vec<&str> = channel.splitn(2, '#').collect();
        let repo_parts: Vec<&str> = parts[0].splitn(2, '/').collect();

        if repo_parts.len() != 2 {
            return Err(PlatformError::ParseError(format!(
                "Invalid channel format: {}. Expected workspace/repo or workspace/repo#pr_id",
                channel
            )));
        }

        let workspace = repo_parts[0];
        let repo_slug = repo_parts[1];
        let text = self.format_response_text(&response);

        // If we have a PR number, post a comment
        if parts.len() == 2 {
            let pr_id: i64 = parts[1]
                .parse()
                .map_err(|_| PlatformError::ParseError("Invalid PR ID".to_string()))?;

            self.post_comment(workspace, repo_slug, pr_id, &text).await?;
        } else {
            // No PR number - we can't post without a target
            info!("No PR ID in channel, skipping response");
        }

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "bitbucket"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_bitbucket_signature(payload, signature)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Bitbucket supports conversation threads on PRs
    }

    fn supports_interactive(&self) -> bool {
        false // Bitbucket doesn't support interactive elements in comments
    }

    fn supports_files(&self) -> bool {
        true // Bitbucket can attach files
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

    fn create_test_config() -> BitbucketConfig {
        BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("test_password".to_string()),
            oauth_token: None,
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-aof-bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        }
    }

    #[test]
    fn test_bitbucket_platform_new() {
        let config = create_test_config();
        let platform = BitbucketPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_bitbucket_platform_invalid_config() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: None,
            oauth_token: None,
            webhook_secret: "".to_string(),
            bot_name: "".to_string(),
            api_url: "".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };
        let platform = BitbucketPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_repo_allowed_all() {
        let config = create_test_config();
        let platform = BitbucketPlatform::new(config).unwrap();

        assert!(platform.is_repo_allowed("workspace/repo"));
        assert!(platform.is_repo_allowed("any/repo"));
    }

    #[test]
    fn test_repo_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_repos = Some(vec![
            "workspace/specific-repo".to_string(),
            "myworkspace/*".to_string(),
        ]);

        let platform = BitbucketPlatform::new(config).unwrap();

        assert!(platform.is_repo_allowed("workspace/specific-repo"));
        assert!(platform.is_repo_allowed("myworkspace/any-repo"));
        assert!(!platform.is_repo_allowed("other/repo"));
    }

    #[test]
    fn test_event_allowed_all() {
        let config = create_test_config();
        let platform = BitbucketPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("repo:push"));
        assert!(platform.is_event_allowed("pullrequest:created"));
        assert!(platform.is_event_allowed("issue:created"));
    }

    #[test]
    fn test_event_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_events = Some(vec![
            "repo:push".to_string(),
            "pullrequest:created".to_string(),
        ]);

        let platform = BitbucketPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("repo:push"));
        assert!(platform.is_event_allowed("pullrequest:created"));
        assert!(!platform.is_event_allowed("issue:created"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = BitbucketPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "bitbucket");
        assert_eq!(platform.bot_name(), "test-aof-bot");
        assert!(platform.supports_threading());
        assert!(!platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(
            BitbucketEventType::from("repo:push"),
            BitbucketEventType::RepoPush
        );
        assert_eq!(
            BitbucketEventType::from("pullrequest:created"),
            BitbucketEventType::PullRequestCreated
        );
        assert_eq!(
            BitbucketEventType::from("issue:created"),
            BitbucketEventType::IssueCreated
        );
        assert_eq!(
            BitbucketEventType::from("unknown_event"),
            BitbucketEventType::Unknown
        );
    }

    #[tokio::test]
    async fn test_verify_signature_format() {
        let config = create_test_config();
        let platform = BitbucketPlatform::new(config).unwrap();

        let payload = b"test payload";
        let invalid_sig = "invalid";

        let result = platform.verify_signature(payload, invalid_sig).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_parse_pr_webhook() {
        let webhook_json = r#"{
            "repository": {
                "uuid": "{abc-123}",
                "name": "repo",
                "full_name": "workspace/repo",
                "is_private": false
            },
            "actor": {
                "uuid": "{user-456}",
                "display_name": "Test User",
                "nickname": "testuser",
                "type": "User"
            },
            "pullrequest": {
                "id": 42,
                "title": "Test PR",
                "description": "Test description",
                "state": "OPEN",
                "author": {
                    "uuid": "{user-456}",
                    "display_name": "Test User",
                    "type": "User"
                },
                "source": {
                    "branch": {
                        "name": "feature-branch"
                    },
                    "commit": {
                        "hash": "abc123",
                        "type": "commit"
                    },
                    "repository": {
                        "uuid": "{abc-123}",
                        "name": "repo",
                        "full_name": "workspace/repo",
                        "is_private": false
                    }
                },
                "destination": {
                    "branch": {
                        "name": "main"
                    },
                    "commit": {
                        "hash": "def456",
                        "type": "commit"
                    },
                    "repository": {
                        "uuid": "{abc-123}",
                        "name": "repo",
                        "full_name": "workspace/repo",
                        "is_private": false
                    }
                },
                "close_source_branch": false,
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z"
            }
        }"#;

        // Create test signature
        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(webhook_json.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let mut headers = HashMap::new();
        headers.insert("x-event-key".to_string(), "pullrequest:created".to_string());
        headers.insert("x-hub-signature".to_string(), signature);

        let config = create_test_config();
        let platform = BitbucketPlatform::new(config).unwrap();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "bitbucket");
        assert_eq!(message.channel_id, "workspace/repo");
        assert!(message.text.contains("pr:created"));
        assert_eq!(message.user.id, "{user-456}");
        assert_eq!(message.user.display_name, Some("Test User".to_string()));
        assert_eq!(message.metadata.get("pr_id").unwrap(), &serde_json::json!(42));
    }
}
