//! GitLab Webhook adapter for AOF
//!
//! This module provides integration with GitLab's Webhook API, supporting:
//! - Push events
//! - Merge request events (open, update, merge, close)
//! - Issue events
//! - Note/comment events (on MRs and issues)
//! - Pipeline events
//! - Job events
//! - Tag push events
//! - Wiki page events
//! - Token-based signature verification (X-Gitlab-Token)
//! - MR approvals and labels
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
//!     type: GitLab
//!     config:
//!       events:
//!         - merge_request.opened
//!         - push
//!       token_env: GITLAB_TOKEN
//!       webhook_secret_env: GITLAB_WEBHOOK_SECRET
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// GitLab platform adapter
///
/// Implements the `TriggerPlatform` trait for GitLab webhooks.
/// Supports multiple event types and provides methods for interacting
/// with the GitLab API (posting comments, approvals, labels, pipeline status).
pub struct GitLabPlatform {
    config: GitLabConfig,
    client: reqwest::Client,
}

/// GitLab configuration
///
/// All configuration options for the GitLab platform.
/// Supports environment variable resolution for secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    /// Personal Access Token or Project Token for API calls
    /// Required scopes: api, read_api, write_repository
    pub token: String,

    /// Webhook secret token for verification
    /// GitLab uses simple token comparison (not HMAC)
    pub webhook_secret: String,

    /// Bot/App name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// API base URL (default https://gitlab.com/api/v4)
    /// For self-hosted GitLab instances, set to https://gitlab.example.com/api/v4
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Allowed project filter (optional whitelist)
    /// Format: ["group/project", "group/*", "*"]
    #[serde(default)]
    pub allowed_projects: Option<Vec<String>>,

    /// Event types to handle (optional filter)
    /// If empty, handles all events
    #[serde(default)]
    pub allowed_events: Option<Vec<String>>,

    /// User IDs allowed to trigger actions (optional whitelist)
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,

    /// Enable posting comments on MRs/issues
    #[serde(default = "default_true")]
    pub enable_comments: bool,

    /// Enable MR approvals
    #[serde(default = "default_true")]
    pub enable_approvals: bool,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_api_url() -> String {
    "https://gitlab.com/api/v4".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// WEBHOOK PAYLOAD TYPES
// ============================================================================

/// GitLab webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitLabEventType {
    Push,
    TagPush,
    MergeRequest,
    Issue,
    Note,
    Pipeline,
    Job,
    WikiPage,
    Deployment,
    Release,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for GitLabEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
            Self::TagPush => write!(f, "tag_push"),
            Self::MergeRequest => write!(f, "merge_request"),
            Self::Issue => write!(f, "issue"),
            Self::Note => write!(f, "note"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Job => write!(f, "job"),
            Self::WikiPage => write!(f, "wiki_page"),
            Self::Deployment => write!(f, "deployment"),
            Self::Release => write!(f, "release"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for GitLabEventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "push hook" | "push" => Self::Push,
            "tag push hook" | "tag_push" => Self::TagPush,
            "merge request hook" | "merge_request" => Self::MergeRequest,
            "issue hook" | "issue" => Self::Issue,
            "note hook" | "note" => Self::Note,
            "pipeline hook" | "pipeline" => Self::Pipeline,
            "job hook" | "job" => Self::Job,
            "wiki page hook" | "wiki_page" => Self::WikiPage,
            "deployment hook" | "deployment" => Self::Deployment,
            "release hook" | "release" => Self::Release,
            _ => Self::Unknown,
        }
    }
}

/// GitLab project information
#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path_with_namespace: String,
    #[serde(default)]
    pub web_url: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub default_branch: Option<String>,
}

/// GitLab user information
#[derive(Debug, Clone, Deserialize)]
pub struct GitLabUser {
    pub id: i64,
    pub username: String,
    pub name: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

/// Merge request information
#[derive(Debug, Clone, Deserialize)]
pub struct MergeRequest {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub description: Option<String>,
    pub author: GitLabUser,
    pub source_branch: String,
    pub target_branch: String,
    #[serde(default)]
    pub work_in_progress: bool,
    #[serde(default)]
    pub draft: bool,
    pub merge_status: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
    #[serde(default)]
    pub source: Option<Project>,
    #[serde(default)]
    pub target: Option<Project>,
    #[serde(default)]
    pub last_commit: Option<Commit>,
}

/// Issue information
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub description: Option<String>,
    pub author: GitLabUser,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub labels: Vec<Label>,
}

/// Label information
#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Note/comment information
#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    pub id: i64,
    pub note: String,
    pub author: GitLabUser,
    #[serde(default)]
    pub noteable_type: Option<String>,
    #[serde(default)]
    pub noteable_id: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
}

/// Commit information
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
}

/// Pipeline information
#[derive(Debug, Clone, Deserialize)]
pub struct Pipeline {
    pub id: i64,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub status: String,
    #[serde(default)]
    pub web_url: Option<String>,
}

/// Job information
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub id: i64,
    pub name: String,
    pub stage: String,
    pub status: String,
    #[serde(default)]
    pub commit: Option<Commit>,
}

// ============================================================================
// WEBHOOK PAYLOAD
// ============================================================================

/// Generic GitLab webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct GitLabWebhookPayload {
    /// Event type (maps to object_kind in GitLab)
    #[serde(default)]
    pub object_kind: Option<String>,

    /// Event name (additional field in some events)
    #[serde(default)]
    pub event_name: Option<String>,

    /// Action that triggered the event (for MR and issue events)
    #[serde(default)]
    pub action: Option<String>,

    /// Project information
    #[serde(default)]
    pub project: Option<Project>,

    /// User who triggered the event
    #[serde(default)]
    pub user: Option<GitLabUser>,

    /// Merge request (for MR events)
    #[serde(default)]
    pub object_attributes: Option<serde_json::Value>,

    /// Merge request (direct field in some events)
    #[serde(default)]
    pub merge_request: Option<MergeRequest>,

    /// Issue (for issue events)
    #[serde(default)]
    pub issue: Option<Issue>,

    /// Note/comment (for note events)
    #[serde(default)]
    pub object_attributes_note: Option<Note>,

    /// Commits (for push events)
    #[serde(default)]
    pub commits: Option<Vec<Commit>>,

    /// Total commits count (for push events)
    #[serde(default)]
    pub total_commits_count: Option<i64>,

    /// Ref (for push events)
    #[serde(rename = "ref", default)]
    pub git_ref: Option<String>,

    /// Before SHA (for push events)
    #[serde(default)]
    pub before: Option<String>,

    /// After SHA (for push events)
    #[serde(default)]
    pub after: Option<String>,

    /// Pipeline (for pipeline events)
    #[serde(default)]
    pub object_attributes_pipeline: Option<Pipeline>,

    /// Job (for job events)
    #[serde(default)]
    pub build: Option<Job>,
}

// ============================================================================
// GITLAB API RESPONSE TYPES
// ============================================================================

/// GitLab API error response
#[derive(Debug, Deserialize)]
struct GitLabApiError {
    message: String,
    #[serde(default)]
    error: Option<String>,
}

/// Note creation response
#[derive(Debug, Deserialize)]
struct NoteResponse {
    id: i64,
    #[serde(default)]
    body: Option<String>,
}

/// Approval response
#[derive(Debug, Deserialize)]
struct ApprovalResponse {
    #[serde(default)]
    approved: bool,
}

// ============================================================================
// PLATFORM IMPLEMENTATION
// ============================================================================

impl GitLabPlatform {
    /// Create new GitLab platform adapter
    ///
    /// # Errors
    /// Returns error if token or webhook_secret is empty
    pub fn new(config: GitLabConfig) -> Result<Self, PlatformError> {
        if config.token.is_empty() || config.webhook_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "GitLab token and webhook secret are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("AOF/{} ({})", env!("CARGO_PKG_VERSION"), config.bot_name))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify webhook token from GitLab
    ///
    /// GitLab uses simple token comparison via X-Gitlab-Token header
    fn verify_gitlab_token(&self, token: &str) -> bool {
        if token == self.config.webhook_secret {
            debug!("GitLab token verified successfully");
            true
        } else {
            debug!("GitLab token mismatch");
            false
        }
    }

    /// Check if project is allowed
    fn is_project_allowed(&self, project_path: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_projects {
            for pattern in allowed {
                if pattern == "*" {
                    return true;
                }
                if pattern.ends_with("/*") {
                    let group = &pattern[..pattern.len() - 2];
                    if project_path.starts_with(&format!("{}/", group)) {
                        return true;
                    }
                }
                if pattern == project_path {
                    return true;
                }
            }
            false
        } else {
            true // All projects allowed if not configured
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
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<GitLabWebhookPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse GitLab webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid GitLab webhook payload: {}", e))
        })
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_url, path)
    }

    // =========================================================================
    // PUBLIC API METHODS - For use by AgentFlow nodes and handlers
    // =========================================================================

    /// Post a comment on a merge request
    ///
    /// # Arguments
    /// * `project_id` - Project ID or URL-encoded path (e.g., "group%2Fproject")
    /// * `mr_iid` - Merge request IID (internal ID)
    /// * `body` - Comment body (supports Markdown)
    pub async fn post_comment(
        &self,
        project_id: &str,
        mr_iid: i64,
        body: &str,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_comments {
            return Err(PlatformError::ApiError("Comments are disabled".to_string()));
        }

        let url = self.api_url(&format!("/projects/{}/merge_requests/{}/notes", project_id, mr_iid));

        let payload = serde_json::json!({
            "body": body
        });

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitLabApiError = response.json().await.unwrap_or(GitLabApiError {
                message: "Unknown error".to_string(),
                error: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        let note: NoteResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted comment {} to MR !{} in project {}", note.id, mr_iid, project_id);
        Ok(note.id)
    }

    /// Post a comment on an issue
    ///
    /// # Arguments
    /// * `project_id` - Project ID or URL-encoded path
    /// * `issue_iid` - Issue IID
    /// * `body` - Comment body (supports Markdown)
    pub async fn post_issue_comment(
        &self,
        project_id: &str,
        issue_iid: i64,
        body: &str,
    ) -> Result<i64, PlatformError> {
        if !self.config.enable_comments {
            return Err(PlatformError::ApiError("Comments are disabled".to_string()));
        }

        let url = self.api_url(&format!("/projects/{}/issues/{}/notes", project_id, issue_iid));

        let payload = serde_json::json!({
            "body": body
        });

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitLabApiError = response.json().await.unwrap_or(GitLabApiError {
                message: "Unknown error".to_string(),
                error: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        let note: NoteResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted comment {} to issue #{} in project {}", note.id, issue_iid, project_id);
        Ok(note.id)
    }

    /// Approve a merge request
    ///
    /// # Arguments
    /// * `project_id` - Project ID or URL-encoded path
    /// * `mr_iid` - Merge request IID
    pub async fn approve_mr(
        &self,
        project_id: &str,
        mr_iid: i64,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_approvals {
            return Err(PlatformError::ApiError("Approvals are disabled".to_string()));
        }

        let url = self.api_url(&format!("/projects/{}/merge_requests/{}/approve", project_id, mr_iid));

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitLabApiError = response.json().await.unwrap_or(GitLabApiError {
                message: "Unknown error".to_string(),
                error: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        info!("Approved MR !{} in project {}", mr_iid, project_id);
        Ok(())
    }

    /// Add labels to a merge request
    pub async fn add_labels(
        &self,
        project_id: &str,
        mr_iid: i64,
        labels: &[String],
    ) -> Result<(), PlatformError> {
        let url = self.api_url(&format!("/projects/{}/merge_requests/{}", project_id, mr_iid));

        let labels_str = labels.join(",");
        let payload = serde_json::json!({
            "add_labels": labels_str
        });

        let response = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitLabApiError = response.json().await.unwrap_or(GitLabApiError {
                message: "Unknown error".to_string(),
                error: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        info!("Added labels {:?} to MR !{} in project {}", labels, mr_iid, project_id);
        Ok(())
    }

    /// Create a pipeline status (commit status)
    ///
    /// # Arguments
    /// * `project_id` - Project ID or URL-encoded path
    /// * `sha` - Commit SHA
    /// * `state` - Status: pending, running, success, failed, canceled
    /// * `name` - Status check name
    /// * `description` - Optional description
    pub async fn create_pipeline_status(
        &self,
        project_id: &str,
        sha: &str,
        state: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), PlatformError> {
        let url = self.api_url(&format!("/projects/{}/statuses/{}", project_id, sha));

        let mut payload = serde_json::json!({
            "state": state,
            "name": name
        });

        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: GitLabApiError = response.json().await.unwrap_or(GitLabApiError {
                message: "Unknown error".to_string(),
                error: None,
            });
            return Err(PlatformError::ApiError(error.message));
        }

        info!("Created pipeline status '{}' ({}) for commit {} in project {}", name, state, &sha[..8], project_id);
        Ok(())
    }

    /// Get the config (for external use)
    pub fn config(&self) -> &GitLabConfig {
        &self.config
    }

    /// Format response for GitLab (Markdown)
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
        payload: &GitLabWebhookPayload,
    ) -> Result<TriggerMessage, PlatformError> {
        let project = payload.project.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing project in webhook".to_string())
        })?;

        let user = payload.user.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing user in webhook".to_string())
        })?;

        // Parse object_attributes for MR and issue data
        let mr_data: Option<MergeRequest> = if let Some(ref attrs) = payload.object_attributes {
            serde_json::from_value(attrs.clone()).ok()
        } else {
            payload.merge_request.clone()
        };

        // Build message text based on event type (case-insensitive)
        let event_lower = event_type.to_lowercase();
        let text = match event_lower.as_str() {
            "push" | "push hook" => {
                let ref_name = payload.git_ref.as_deref().unwrap_or("unknown");
                let commit_count = payload.total_commits_count.unwrap_or(0);
                let head_msg = payload
                    .commits
                    .as_ref()
                    .and_then(|c| c.first())
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
            "merge_request" | "merge request hook" => {
                let mr = mr_data.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing merge_request in MR event".to_string())
                })?;
                format!(
                    "mr:{}:{}:{} !{} {} - {}",
                    action.unwrap_or(""),
                    mr.target_branch,
                    mr.source_branch,
                    mr.iid,
                    mr.title,
                    mr.description.as_deref().unwrap_or("").lines().next().unwrap_or("")
                )
            }
            "issue" | "issue hook" => {
                let issue = payload.issue.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing issue in issue event".to_string())
                })?;
                format!(
                    "issue:{}:#{} {} - {}",
                    action.unwrap_or(""),
                    issue.iid,
                    issue.title,
                    issue.description.as_deref().unwrap_or("").lines().next().unwrap_or("")
                )
            }
            "note" | "note hook" => {
                let note = payload.object_attributes_note.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing note in note event".to_string())
                })?;
                let noteable_type = note.noteable_type.as_deref().unwrap_or("unknown");
                let noteable_id = note.noteable_id.unwrap_or(0);
                format!(
                    "note:{}:{}:{} {}",
                    action.unwrap_or(""),
                    noteable_type,
                    noteable_id,
                    note.note.lines().next().unwrap_or("")
                )
            }
            "pipeline" | "pipeline hook" => {
                let pipeline = payload.object_attributes_pipeline.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing pipeline in pipeline event".to_string())
                })?;
                format!(
                    "pipeline:{}:{} {} on {}",
                    action.unwrap_or(""),
                    pipeline.status,
                    pipeline.id,
                    pipeline.ref_name
                )
            }
            "job" | "job hook" => {
                let job = payload.build.as_ref().ok_or_else(|| {
                    PlatformError::ParseError("Missing job in job event".to_string())
                })?;
                format!(
                    "job:{}:{} {} in stage {}",
                    action.unwrap_or(""),
                    job.name,
                    job.status,
                    job.stage
                )
            }
            _ => format!("{}:{}", event_type, action.unwrap_or("")),
        };

        // Build channel_id from project path
        let channel_id = project.path_with_namespace.clone();

        // Build user
        let trigger_user = TriggerUser {
            id: user.id.to_string(),
            username: Some(user.username.clone()),
            display_name: Some(user.name.clone()),
            is_bot: false, // GitLab doesn't have a direct bot field
        };

        // Build metadata with full event details
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), serde_json::json!(event_type));
        metadata.insert("action".to_string(), serde_json::json!(action));
        metadata.insert("project_id".to_string(), serde_json::json!(project.id));
        metadata.insert("project_path".to_string(), serde_json::json!(project.path_with_namespace));
        metadata.insert("user_id".to_string(), serde_json::json!(user.id));
        metadata.insert("user_username".to_string(), serde_json::json!(user.username));

        // Add MR-specific metadata
        if let Some(ref mr) = mr_data {
            metadata.insert("mr_iid".to_string(), serde_json::json!(mr.iid));
            metadata.insert("mr_title".to_string(), serde_json::json!(mr.title));
            metadata.insert("mr_state".to_string(), serde_json::json!(mr.state));
            metadata.insert("mr_draft".to_string(), serde_json::json!(mr.draft || mr.work_in_progress));
            metadata.insert("mr_source_branch".to_string(), serde_json::json!(mr.source_branch));
            metadata.insert("mr_target_branch".to_string(), serde_json::json!(mr.target_branch));
            metadata.insert("mr_merge_status".to_string(), serde_json::json!(mr.merge_status));
            if let Some(ref url) = mr.web_url {
                metadata.insert("mr_web_url".to_string(), serde_json::json!(url));
            }
        }

        // Add issue-specific metadata
        if let Some(ref issue) = payload.issue {
            metadata.insert("issue_iid".to_string(), serde_json::json!(issue.iid));
            metadata.insert("issue_title".to_string(), serde_json::json!(issue.title));
            metadata.insert("issue_state".to_string(), serde_json::json!(issue.state));
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
        if let Some(count) = payload.total_commits_count {
            metadata.insert("commit_count".to_string(), serde_json::json!(count));
        }

        // Build message ID from event type and unique identifiers
        let message_id = if let Some(ref mr) = mr_data {
            format!("gl-{}-{}-mr-{}", project.id, event_type, mr.id)
        } else if let Some(ref issue) = payload.issue {
            format!("gl-{}-{}-issue-{}", project.id, event_type, issue.id)
        } else if let Some(ref after) = payload.after {
            format!("gl-{}-{}-{}", project.id, event_type, &after[..8.min(after.len())])
        } else {
            format!("gl-{}-{}-{}", project.id, event_type, chrono::Utc::now().timestamp_millis())
        };

        // Thread ID for MRs and issues
        let thread_id = if let Some(ref mr) = mr_data {
            Some(format!("mr-{}", mr.iid))
        } else if let Some(ref issue) = payload.issue {
            Some(format!("issue-{}", issue.iid))
        } else {
            None
        };

        Ok(TriggerMessage {
            id: message_id,
            platform: "gitlab".to_string(),
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
impl TriggerPlatform for GitLabPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Get event type from header
        let event_type = headers
            .get("x-gitlab-event")
            .ok_or_else(|| PlatformError::ParseError("Missing X-Gitlab-Event header".to_string()))?;

        // Verify token
        if let Some(token) = headers.get("x-gitlab-token") {
            if !self.verify_gitlab_token(token) {
                warn!("Invalid GitLab token");
                return Err(PlatformError::InvalidSignature(
                    "Token verification failed".to_string(),
                ));
            }
        } else {
            warn!("Missing X-Gitlab-Token header");
            return Err(PlatformError::InvalidSignature(
                "Missing token header".to_string(),
            ));
        }

        // Check if event type is allowed
        if !self.is_event_allowed(event_type) {
            info!("Event type {} not allowed", event_type);
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Parse payload
        let payload = self.parse_webhook_payload(raw)?;

        // Check if project is allowed
        if let Some(ref project) = payload.project {
            if !self.is_project_allowed(&project.path_with_namespace) {
                info!("Project {} not allowed", project.path_with_namespace);
                return Err(PlatformError::InvalidSignature(
                    "Project not allowed".to_string(),
                ));
            }
        }

        // Check if user is allowed
        if let Some(ref user) = payload.user {
            if !self.is_user_allowed(&user.username) {
                info!("User {} not allowed", user.username);
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
        channel: &str, // format: group/project or group/project!iid
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Parse channel format: group/project!iid or group/project#iid
        let parts: Vec<&str> = if channel.contains('!') {
            channel.splitn(2, '!').collect()
        } else if channel.contains('#') {
            channel.splitn(2, '#').collect()
        } else {
            vec![channel]
        };

        let project_path = parts[0];
        let text = self.format_response_text(&response);

        // If we have an iid (MR or issue), post a comment
        if parts.len() == 2 {
            // URL-encode the project path
            let project_id = urlencoding::encode(project_path);

            let iid: i64 = parts[1]
                .parse()
                .map_err(|_| PlatformError::ParseError("Invalid MR/issue IID".to_string()))?;

            // Try to post as MR comment first (most common case)
            // If it fails, try as issue comment
            match self.post_comment(&project_id, iid, &text).await {
                Ok(_) => Ok(()),
                Err(_) => {
                    // Fallback to issue comment
                    self.post_issue_comment(&project_id, iid, &text).await?;
                    Ok(())
                }
            }
        } else {
            // No iid - we can't post without a target
            info!("No MR/issue IID in channel, skipping response");
            Ok(())
        }
    }

    fn platform_name(&self) -> &'static str {
        "gitlab"
    }

    async fn verify_signature(&self, _payload: &[u8], signature: &str) -> bool {
        self.verify_gitlab_token(signature)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // GitLab supports conversation threads on MRs/issues
    }

    fn supports_interactive(&self) -> bool {
        true // GitLab supports reactions and CI/CD workflows
    }

    fn supports_files(&self) -> bool {
        true // GitLab can attach files via releases/artifacts
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

    fn create_test_config() -> GitLabConfig {
        GitLabConfig {
            token: "glpat-test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-aof-bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        }
    }

    #[test]
    fn test_gitlab_platform_new() {
        let config = create_test_config();
        let platform = GitLabPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_gitlab_platform_invalid_config() {
        let config = GitLabConfig {
            token: "".to_string(),
            webhook_secret: "".to_string(),
            bot_name: "".to_string(),
            api_url: "".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };
        let platform = GitLabPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_project_allowed_all() {
        let config = create_test_config();
        let platform = GitLabPlatform::new(config).unwrap();

        assert!(platform.is_project_allowed("group/project"));
        assert!(platform.is_project_allowed("any/repo"));
    }

    #[test]
    fn test_project_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_projects = Some(vec![
            "group/specific-project".to_string(),
            "org/*".to_string(),
        ]);

        let platform = GitLabPlatform::new(config).unwrap();

        assert!(platform.is_project_allowed("group/specific-project"));
        assert!(platform.is_project_allowed("org/any-repo"));
        assert!(!platform.is_project_allowed("other/repo"));
    }

    #[test]
    fn test_event_allowed_all() {
        let config = create_test_config();
        let platform = GitLabPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("Push Hook"));
        assert!(platform.is_event_allowed("Merge Request Hook"));
        assert!(platform.is_event_allowed("Issue Hook"));
    }

    #[test]
    fn test_event_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_events = Some(vec!["Push Hook".to_string(), "Merge Request Hook".to_string()]);

        let platform = GitLabPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("Push Hook"));
        assert!(platform.is_event_allowed("Merge Request Hook"));
        assert!(!platform.is_event_allowed("Issue Hook"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = GitLabPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "gitlab");
        assert_eq!(platform.bot_name(), "test-aof-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(GitLabEventType::from("Push Hook"), GitLabEventType::Push);
        assert_eq!(GitLabEventType::from("Merge Request Hook"), GitLabEventType::MergeRequest);
        assert_eq!(GitLabEventType::from("Issue Hook"), GitLabEventType::Issue);
        assert_eq!(GitLabEventType::from("Note Hook"), GitLabEventType::Note);
        assert_eq!(GitLabEventType::from("unknown_event"), GitLabEventType::Unknown);
    }

    #[tokio::test]
    async fn test_verify_token() {
        let config = create_test_config();
        let platform = GitLabPlatform::new(config).unwrap();

        assert!(platform.verify_signature(&[], "test_secret").await);
        assert!(!platform.verify_signature(&[], "wrong_secret").await);
    }

    #[tokio::test]
    async fn test_parse_mr_webhook() {
        let webhook_json = r#"{
            "object_kind": "merge_request",
            "event_type": "merge_request",
            "object_attributes": {
                "id": 123,
                "iid": 42,
                "title": "Test MR",
                "state": "opened",
                "description": "Test description",
                "source_branch": "feature-branch",
                "target_branch": "main",
                "work_in_progress": false,
                "draft": false,
                "merge_status": "can_be_merged",
                "web_url": "https://gitlab.com/group/project/-/merge_requests/42",
                "author": {
                    "id": 456,
                    "username": "testuser",
                    "name": "Test User"
                },
                "last_commit": {
                    "id": "abc123",
                    "message": "Test commit",
                    "timestamp": "2025-01-01T00:00:00Z",
                    "author": {
                        "name": "Test User",
                        "email": "test@example.com"
                    }
                }
            },
            "project": {
                "id": 789,
                "name": "project",
                "path_with_namespace": "group/project",
                "web_url": "https://gitlab.com/group/project"
            },
            "user": {
                "id": 456,
                "username": "testuser",
                "name": "Test User"
            }
        }"#;

        let mut headers = HashMap::new();
        headers.insert("x-gitlab-event".to_string(), "Merge Request Hook".to_string());
        headers.insert("x-gitlab-token".to_string(), "test_secret".to_string());

        let config = create_test_config();
        let platform = GitLabPlatform::new(config).unwrap();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "gitlab");
        assert_eq!(message.channel_id, "group/project");
        assert!(message.text.contains("mr:"));
        assert_eq!(message.user.id, "456");
        assert_eq!(message.user.username, Some("testuser".to_string()));
        assert_eq!(message.metadata.get("mr_iid").unwrap(), &serde_json::json!(42));
    }
}
