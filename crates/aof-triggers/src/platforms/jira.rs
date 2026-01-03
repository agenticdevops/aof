//! Jira Webhook adapter for AOF
//!
//! This module provides integration with Jira's Webhook API, supporting:
//! - Issue events (created, updated, deleted)
//! - Comment events (created, updated, deleted)
//! - Sprint events (started, closed)
//! - Worklog events
//! - HMAC-SHA256 signature verification
//! - Custom field support
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
//!     type: Jira
//!     config:
//!       base_url: https://your-domain.atlassian.net
//!       email: your-email@example.com
//!       api_token_env: JIRA_API_TOKEN
//!       webhook_secret_env: JIRA_WEBHOOK_SECRET
//!       allowed_projects:
//!         - PROJECT1
//!         - PROJECT2
//! ```

use async_trait::async_trait;
use base64::Engine;
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

/// Jira platform adapter
///
/// Implements the `TriggerPlatform` trait for Jira webhooks.
/// Supports multiple event types and provides methods for interacting
/// with the Jira API (posting comments, updating issues, transitions).
pub struct JiraPlatform {
    config: JiraConfig,
    client: reqwest::Client,
}

/// Jira configuration
///
/// All configuration options for the Jira platform.
/// Supports environment variable resolution for secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Jira instance base URL (e.g., https://your-domain.atlassian.net)
    pub base_url: String,

    /// Email address for API authentication
    pub email: String,

    /// API token for Jira Cloud
    /// Generate from: https://id.atlassian.com/manage-profile/security/api-tokens
    pub api_token: String,

    /// Webhook secret for HMAC-SHA256 signature verification
    pub webhook_secret: String,

    /// Bot/App name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Allowed project keys filter (optional whitelist)
    /// Format: ["PROJECT1", "PROJECT2", "*"]
    #[serde(default)]
    pub allowed_projects: Option<Vec<String>>,

    /// Event types to handle (optional filter)
    /// If empty, handles all events
    #[serde(default)]
    pub allowed_events: Option<Vec<String>>,

    /// User account IDs allowed to trigger actions (optional whitelist)
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,

    /// Enable posting comments
    #[serde(default = "default_true")]
    pub enable_comments: bool,

    /// Enable updating issues
    #[serde(default = "default_true")]
    pub enable_updates: bool,

    /// Enable issue transitions
    #[serde(default = "default_true")]
    pub enable_transitions: bool,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// WEBHOOK PAYLOAD TYPES
// ============================================================================

/// Jira webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JiraEventType {
    #[serde(rename = "jira:issue_created")]
    IssueCreated,
    #[serde(rename = "jira:issue_updated")]
    IssueUpdated,
    #[serde(rename = "jira:issue_deleted")]
    IssueDeleted,
    #[serde(rename = "comment_created")]
    CommentCreated,
    #[serde(rename = "comment_updated")]
    CommentUpdated,
    #[serde(rename = "comment_deleted")]
    CommentDeleted,
    #[serde(rename = "sprint_started")]
    SprintStarted,
    #[serde(rename = "sprint_closed")]
    SprintClosed,
    #[serde(rename = "worklog_created")]
    WorklogCreated,
    #[serde(rename = "worklog_updated")]
    WorklogUpdated,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for JiraEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IssueCreated => write!(f, "jira:issue_created"),
            Self::IssueUpdated => write!(f, "jira:issue_updated"),
            Self::IssueDeleted => write!(f, "jira:issue_deleted"),
            Self::CommentCreated => write!(f, "comment_created"),
            Self::CommentUpdated => write!(f, "comment_updated"),
            Self::CommentDeleted => write!(f, "comment_deleted"),
            Self::SprintStarted => write!(f, "sprint_started"),
            Self::SprintClosed => write!(f, "sprint_closed"),
            Self::WorklogCreated => write!(f, "worklog_created"),
            Self::WorklogUpdated => write!(f, "worklog_updated"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for JiraEventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jira:issue_created" => Self::IssueCreated,
            "jira:issue_updated" => Self::IssueUpdated,
            "jira:issue_deleted" => Self::IssueDeleted,
            "comment_created" => Self::CommentCreated,
            "comment_updated" => Self::CommentUpdated,
            "comment_deleted" => Self::CommentDeleted,
            "sprint_started" => Self::SprintStarted,
            "sprint_closed" => Self::SprintClosed,
            "worklog_created" => Self::WorklogCreated,
            "worklog_updated" => Self::WorklogUpdated,
            _ => Self::Unknown,
        }
    }
}

/// Jira user information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUser {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub account_type: Option<String>,
}

/// Jira issue fields
#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    pub issuetype: JiraIssueType,
    pub project: JiraProject,
    #[serde(default)]
    pub status: Option<JiraStatus>,
    #[serde(default)]
    pub priority: Option<JiraPriority>,
    #[serde(default)]
    pub assignee: Option<JiraUser>,
    #[serde(default)]
    pub reporter: Option<JiraUser>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

/// Jira issue type
#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssueType {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Jira project information
#[derive(Debug, Clone, Deserialize)]
pub struct JiraProject {
    #[serde(default)]
    pub id: Option<String>,
    pub key: String,
    pub name: String,
}

/// Jira status
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraStatus {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub status_category: Option<JiraStatusCategory>,
}

/// Jira status category
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraStatusCategory {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub key: Option<String>,
    pub name: String,
}

/// Jira priority
#[derive(Debug, Clone, Deserialize)]
pub struct JiraPriority {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
}

/// Jira issue information
#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self", default)]
    pub self_url: Option<String>,
    pub fields: JiraIssueFields,
}

/// Jira comment information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "self", default)]
    pub self_url: Option<String>,
    pub body: String,
    #[serde(default)]
    pub author: Option<JiraUser>,
    #[serde(default)]
    pub update_author: Option<JiraUser>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

/// Jira sprint information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSprint {
    pub id: i64,
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub complete_date: Option<String>,
}

/// Jira changelog item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelogItem {
    pub field: String,
    pub fieldtype: String,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub from_string: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub to_string: Option<String>,
}

/// Jira changelog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraChangelog {
    pub id: String,
    #[serde(default)]
    pub items: Vec<JiraChangelogItem>,
}

/// Jira webhook payload
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWebhookPayload {
    /// Webhook event timestamp
    pub timestamp: i64,

    /// Event type
    pub webhook_event: String,

    /// Issue information (for issue events)
    #[serde(default)]
    pub issue: Option<JiraIssue>,

    /// User who triggered the event
    #[serde(default)]
    pub user: Option<JiraUser>,

    /// Changelog (for update events)
    #[serde(default)]
    pub changelog: Option<JiraChangelog>,

    /// Comment (for comment events)
    #[serde(default)]
    pub comment: Option<JiraComment>,

    /// Sprint (for sprint events)
    #[serde(default)]
    pub sprint: Option<JiraSprint>,
}

// ============================================================================
// JIRA API RESPONSE TYPES
// ============================================================================

/// Jira API error response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraApiError {
    #[serde(default)]
    error_messages: Vec<String>,
    #[serde(default)]
    errors: HashMap<String, String>,
}

/// Comment creation response
#[derive(Debug, Deserialize)]
struct CommentResponse {
    id: String,
    #[serde(rename = "self")]
    self_url: String,
}

/// Transition response
#[derive(Debug, Deserialize)]
struct TransitionResponse {
    #[serde(default)]
    transitions: Vec<JiraTransition>,
}

/// Available transition
#[derive(Debug, Deserialize)]
struct JiraTransition {
    id: String,
    name: String,
}

// ============================================================================
// PLATFORM IMPLEMENTATION
// ============================================================================

impl JiraPlatform {
    /// Create new Jira platform adapter
    ///
    /// # Errors
    /// Returns error if required config fields are empty
    pub fn new(config: JiraConfig) -> Result<Self, PlatformError> {
        if config.base_url.is_empty()
            || config.email.is_empty()
            || config.api_token.is_empty()
            || config.webhook_secret.is_empty()
        {
            return Err(PlatformError::ParseError(
                "Jira base_url, email, api_token, and webhook_secret are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("AOF/{} ({})", env!("CARGO_PKG_VERSION"), config.bot_name))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify signature from Jira webhook
    /// Supports multiple modes:
    /// 1. HMAC-SHA256 signature (prefixed with "sha256=" or raw hex)
    /// 2. Static shared secret (direct comparison for Jira Automation)
    fn verify_jira_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Strip common prefixes like "sha256=" or "sha1=" if present
        let provided_signature = signature
            .strip_prefix("sha256=")
            .or_else(|| signature.strip_prefix("sha1="))
            .unwrap_or(signature);

        // Mode 1: Direct secret comparison (for Jira Automation static secrets)
        // Jira Automation sends the secret value directly in the header
        if provided_signature == self.config.webhook_secret {
            debug!("Jira signature verified via direct secret match");
            return true;
        }

        // Mode 2: HMAC-SHA256 verification (for computed signatures)
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
            debug!("Jira signature verified via HMAC-SHA256");
            true
        } else {
            debug!(
                "Signature mismatch - computed HMAC: {}..., provided: {}...",
                &computed_signature[..8.min(computed_signature.len())],
                &provided_signature[..8.min(provided_signature.len())]
            );
            false
        }
    }

    /// Check if project is allowed
    fn is_project_allowed(&self, project_key: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_projects {
            allowed.iter().any(|p| p == "*" || p == project_key)
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
    fn is_user_allowed(&self, account_id: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_users {
            allowed.contains(&account_id.to_string())
        } else {
            true // All users allowed if not configured
        }
    }

    /// Parse webhook payload
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<JiraWebhookPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse Jira webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid Jira webhook payload: {}", e))
        })
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}/rest/api/3{}", self.config.base_url, path)
    }

    /// Get basic auth header value
    fn basic_auth(&self) -> String {
        let credentials = format!("{}:{}", self.config.email, self.config.api_token);
        format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(credentials))
    }

    // =========================================================================
    // PUBLIC API METHODS - For use by AgentFlow nodes and handlers
    // =========================================================================

    /// Post a comment on an issue
    ///
    /// # Arguments
    /// * `issue_key` - Issue key (e.g., PROJECT-123)
    /// * `body` - Comment body (supports Jira text format)
    pub async fn post_comment(&self, issue_key: &str, body: &str) -> Result<String, PlatformError> {
        if !self.config.enable_comments {
            return Err(PlatformError::ApiError("Comments are disabled".to_string()));
        }

        let url = self.api_url(&format!("/issue/{}/comment", issue_key));

        let payload = serde_json::json!({
            "body": body
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: JiraApiError = response.json().await.unwrap_or(JiraApiError {
                error_messages: vec!["Unknown error".to_string()],
                errors: HashMap::new(),
            });
            let msg = error.error_messages.join(", ");
            return Err(PlatformError::ApiError(msg));
        }

        let comment: CommentResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        info!("Posted comment {} to issue {}", comment.id, issue_key);
        Ok(comment.id)
    }

    /// Update issue fields
    ///
    /// # Arguments
    /// * `issue_key` - Issue key (e.g., PROJECT-123)
    /// * `fields` - JSON object with fields to update
    pub async fn update_issue(
        &self,
        issue_key: &str,
        fields: serde_json::Value,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_updates {
            return Err(PlatformError::ApiError("Updates are disabled".to_string()));
        }

        let url = self.api_url(&format!("/issue/{}", issue_key));

        let payload = serde_json::json!({
            "fields": fields
        });

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: JiraApiError = response.json().await.unwrap_or(JiraApiError {
                error_messages: vec!["Unknown error".to_string()],
                errors: HashMap::new(),
            });
            let msg = error.error_messages.join(", ");
            return Err(PlatformError::ApiError(msg));
        }

        info!("Updated issue {}", issue_key);
        Ok(())
    }

    /// Transition an issue to a new status
    ///
    /// # Arguments
    /// * `issue_key` - Issue key (e.g., PROJECT-123)
    /// * `transition_id` - Transition ID to execute
    pub async fn transition_issue(&self, issue_key: &str, transition_id: &str) -> Result<(), PlatformError> {
        if !self.config.enable_transitions {
            return Err(PlatformError::ApiError("Transitions are disabled".to_string()));
        }

        let url = self.api_url(&format!("/issue/{}/transitions", issue_key));

        let payload = serde_json::json!({
            "transition": {
                "id": transition_id
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: JiraApiError = response.json().await.unwrap_or(JiraApiError {
                error_messages: vec!["Unknown error".to_string()],
                errors: HashMap::new(),
            });
            let msg = error.error_messages.join(", ");
            return Err(PlatformError::ApiError(msg));
        }

        info!("Transitioned issue {} to {}", issue_key, transition_id);
        Ok(())
    }

    /// Add labels to an issue
    ///
    /// # Arguments
    /// * `issue_key` - Issue key (e.g., PROJECT-123)
    /// * `labels` - Labels to add
    pub async fn add_labels(&self, issue_key: &str, labels: &[String]) -> Result<(), PlatformError> {
        if !self.config.enable_updates {
            return Err(PlatformError::ApiError("Updates are disabled".to_string()));
        }

        let url = self.api_url(&format!("/issue/{}", issue_key));

        let update_ops: Vec<_> = labels
            .iter()
            .map(|label| {
                serde_json::json!({
                    "add": label
                })
            })
            .collect();

        let payload = serde_json::json!({
            "update": {
                "labels": update_ops
            }
        });

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: JiraApiError = response.json().await.unwrap_or(JiraApiError {
                error_messages: vec!["Unknown error".to_string()],
                errors: HashMap::new(),
            });
            let msg = error.error_messages.join(", ");
            return Err(PlatformError::ApiError(msg));
        }

        info!("Added labels {:?} to issue {}", labels, issue_key);
        Ok(())
    }

    /// Assign issue to a user
    ///
    /// # Arguments
    /// * `issue_key` - Issue key (e.g., PROJECT-123)
    /// * `account_id` - User account ID (null or "-1" to unassign)
    pub async fn assign_issue(&self, issue_key: &str, account_id: &str) -> Result<(), PlatformError> {
        if !self.config.enable_updates {
            return Err(PlatformError::ApiError("Updates are disabled".to_string()));
        }

        let assignee_value = if account_id == "-1" || account_id.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::json!({
                "accountId": account_id
            })
        };

        self.update_issue(
            issue_key,
            serde_json::json!({
                "assignee": assignee_value
            }),
        )
        .await
    }

    /// Get the config (for external use)
    pub fn config(&self) -> &JiraConfig {
        &self.config
    }

    /// Format response for Jira (plain text)
    fn format_response_text(&self, response: &TriggerResponse) -> String {
        let status_text = match response.status {
            crate::response::ResponseStatus::Success => "SUCCESS",
            crate::response::ResponseStatus::Error => "ERROR",
            crate::response::ResponseStatus::Warning => "WARNING",
            crate::response::ResponseStatus::Info => "INFO",
        };

        format!("[{}] {}", status_text, response.text)
    }

    /// Build TriggerMessage from parsed webhook
    fn build_trigger_message(
        &self,
        event_type: &str,
        payload: &JiraWebhookPayload,
    ) -> Result<TriggerMessage, PlatformError> {
        let issue = payload.issue.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing issue in webhook".to_string())
        })?;

        let user = payload.user.as_ref().ok_or_else(|| {
            PlatformError::ParseError("Missing user in webhook".to_string())
        })?;

        // Build message text based on event type
        let text = match event_type {
            "jira:issue_created" => {
                format!(
                    "issue:created:{} {} - {}",
                    issue.key,
                    issue.fields.summary,
                    issue.fields.description.as_deref().unwrap_or("")
                )
            }
            "jira:issue_updated" => {
                let changes = if let Some(ref changelog) = payload.changelog {
                    changelog
                        .items
                        .iter()
                        .map(|item| format!("{}: {} -> {}", item.field, item.from_string.as_deref().unwrap_or(""), item.to_string.as_deref().unwrap_or("")))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    "updated".to_string()
                };
                format!("issue:updated:{} {}", issue.key, changes)
            }
            "jira:issue_deleted" => {
                format!("issue:deleted:{}", issue.key)
            }
            "comment_created" => {
                let comment = payload.comment.as_ref().map(|c| c.body.as_str()).unwrap_or("");
                format!("comment:created:{} {}", issue.key, comment)
            }
            "comment_updated" => {
                let comment = payload.comment.as_ref().map(|c| c.body.as_str()).unwrap_or("");
                format!("comment:updated:{} {}", issue.key, comment)
            }
            "comment_deleted" => {
                format!("comment:deleted:{}", issue.key)
            }
            "sprint_started" => {
                let sprint = payload.sprint.as_ref().map(|s| s.name.as_str()).unwrap_or("");
                format!("sprint:started:{}", sprint)
            }
            "sprint_closed" => {
                let sprint = payload.sprint.as_ref().map(|s| s.name.as_str()).unwrap_or("");
                format!("sprint:closed:{}", sprint)
            }
            _ => format!("{}:{}", event_type, issue.key),
        };

        // Channel ID is the project key
        let channel_id = issue.fields.project.key.clone();

        // Build user
        let trigger_user = TriggerUser {
            id: user.account_id.clone().unwrap_or_default(),
            username: user.email_address.clone(),
            display_name: user.display_name.clone(),
            is_bot: user.account_type.as_deref() == Some("app"),
        };

        // Build metadata with full event details
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), serde_json::json!(event_type));
        metadata.insert("issue_id".to_string(), serde_json::json!(issue.id));
        metadata.insert("issue_key".to_string(), serde_json::json!(issue.key));
        metadata.insert("issue_type".to_string(), serde_json::json!(issue.fields.issuetype.name));
        metadata.insert("project_id".to_string(), serde_json::json!(issue.fields.project.id));
        metadata.insert("project_key".to_string(), serde_json::json!(issue.fields.project.key));
        metadata.insert("summary".to_string(), serde_json::json!(issue.fields.summary));

        if let Some(ref status) = issue.fields.status {
            metadata.insert("status".to_string(), serde_json::json!(status.name));
        }

        if let Some(ref priority) = issue.fields.priority {
            metadata.insert("priority".to_string(), serde_json::json!(priority.name));
        }

        if let Some(ref assignee) = issue.fields.assignee {
            metadata.insert("assignee".to_string(), serde_json::json!(assignee.account_id));
        }

        metadata.insert("labels".to_string(), serde_json::json!(issue.fields.labels));

        // Add changelog if present
        if let Some(ref changelog) = payload.changelog {
            metadata.insert("changelog".to_string(), serde_json::to_value(changelog).unwrap_or_default());
        }

        // Message ID from issue and timestamp
        let message_id = format!("jira-{}-{}-{}", issue.id, event_type, payload.timestamp);

        // Thread ID from issue key
        let thread_id = Some(issue.key.clone());

        Ok(TriggerMessage {
            id: message_id,
            platform: "jira".to_string(),
            channel_id,
            user: trigger_user,
            text,
            timestamp: chrono::DateTime::from_timestamp(payload.timestamp / 1000, 0).unwrap_or_else(chrono::Utc::now),
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
impl TriggerPlatform for JiraPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Log raw payload for debugging
        if let Ok(raw_str) = std::str::from_utf8(raw) {
            debug!("Jira webhook raw payload ({} bytes): {}", raw.len(),
                   if raw_str.len() > 500 { &raw_str[..500] } else { raw_str });
        } else {
            debug!("Jira webhook raw payload ({} bytes): <binary>", raw.len());
        }

        // Verify signature if present
        if let Some(signature) = headers.get("x-hub-signature") {
            if !self.verify_jira_signature(raw, signature) {
                warn!("Invalid Jira signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        // Parse payload
        let payload = self.parse_webhook_payload(raw)?;

        // Check if event type is allowed
        if !self.is_event_allowed(&payload.webhook_event) {
            info!("Event type {} not allowed", payload.webhook_event);
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Check if project is allowed
        if let Some(ref issue) = payload.issue {
            if !self.is_project_allowed(&issue.fields.project.key) {
                info!("Project {} not allowed", issue.fields.project.key);
                return Err(PlatformError::InvalidSignature(
                    "Project not allowed".to_string(),
                ));
            }
        }

        // Check if user is allowed
        if let Some(ref user) = payload.user {
            if let Some(ref account_id) = user.account_id {
                if !self.is_user_allowed(account_id) {
                    info!("User {} not allowed", account_id);
                    return Err(PlatformError::InvalidSignature(
                        "User not allowed".to_string(),
                    ));
                }
            }
        }

        // Build trigger message
        self.build_trigger_message(&payload.webhook_event, &payload)
    }

    async fn send_response(
        &self,
        channel: &str, // format: PROJECT-123
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        let text = self.format_response_text(&response);
        self.post_comment(channel, &text).await?;
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "jira"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_jira_signature(payload, signature)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Jira supports conversation threads via comments
    }

    fn supports_interactive(&self) -> bool {
        true // Jira supports transitions and workflows
    }

    fn supports_files(&self) -> bool {
        true // Jira supports attachments
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

    fn create_test_config() -> JiraConfig {
        JiraConfig {
            base_url: "https://test.atlassian.net".to_string(),
            email: "test@example.com".to_string(),
            api_token: "test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-aof-bot".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_updates: true,
            enable_transitions: true,
        }
    }

    #[test]
    fn test_jira_platform_new() {
        let config = create_test_config();
        let platform = JiraPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_jira_platform_invalid_config() {
        let config = JiraConfig {
            base_url: "".to_string(),
            email: "".to_string(),
            api_token: "".to_string(),
            webhook_secret: "".to_string(),
            bot_name: "".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_updates: true,
            enable_transitions: true,
        };
        let platform = JiraPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_project_allowed_all() {
        let config = create_test_config();
        let platform = JiraPlatform::new(config).unwrap();

        assert!(platform.is_project_allowed("PROJECT1"));
        assert!(platform.is_project_allowed("PROJECT2"));
    }

    #[test]
    fn test_project_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_projects = Some(vec!["PROJECT1".to_string(), "PROJECT2".to_string()]);

        let platform = JiraPlatform::new(config).unwrap();

        assert!(platform.is_project_allowed("PROJECT1"));
        assert!(platform.is_project_allowed("PROJECT2"));
        assert!(!platform.is_project_allowed("PROJECT3"));
    }

    #[test]
    fn test_event_allowed_all() {
        let config = create_test_config();
        let platform = JiraPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("jira:issue_created"));
        assert!(platform.is_event_allowed("jira:issue_updated"));
        assert!(platform.is_event_allowed("comment_created"));
    }

    #[test]
    fn test_event_allowed_whitelist() {
        let mut config = create_test_config();
        config.allowed_events = Some(vec!["jira:issue_created".to_string(), "jira:issue_updated".to_string()]);

        let platform = JiraPlatform::new(config).unwrap();

        assert!(platform.is_event_allowed("jira:issue_created"));
        assert!(platform.is_event_allowed("jira:issue_updated"));
        assert!(!platform.is_event_allowed("comment_created"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = JiraPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "jira");
        assert_eq!(platform.bot_name(), "test-aof-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(JiraEventType::from("jira:issue_created"), JiraEventType::IssueCreated);
        assert_eq!(JiraEventType::from("jira:issue_updated"), JiraEventType::IssueUpdated);
        assert_eq!(JiraEventType::from("comment_created"), JiraEventType::CommentCreated);
        assert_eq!(JiraEventType::from("sprint_started"), JiraEventType::SprintStarted);
        assert_eq!(JiraEventType::from("unknown_event"), JiraEventType::Unknown);
    }

    #[tokio::test]
    async fn test_parse_issue_webhook() {
        let webhook_json = r#"{
            "timestamp": 1640000000000,
            "webhookEvent": "jira:issue_created",
            "issue": {
                "id": "10001",
                "key": "PROJECT-123",
                "self": "https://test.atlassian.net/rest/api/3/issue/10001",
                "fields": {
                    "summary": "Test Issue",
                    "description": "This is a test issue",
                    "issuetype": {
                        "id": "10001",
                        "name": "Task"
                    },
                    "project": {
                        "id": "10000",
                        "key": "PROJECT",
                        "name": "Test Project"
                    },
                    "status": {
                        "id": "1",
                        "name": "To Do"
                    },
                    "priority": {
                        "id": "3",
                        "name": "Medium"
                    },
                    "labels": ["test", "automation"]
                }
            },
            "user": {
                "accountId": "557058:12345678-1234-1234-1234-123456789012",
                "displayName": "Test User",
                "emailAddress": "test@example.com",
                "active": true,
                "accountType": "atlassian"
            }
        }"#;

        // Create test signature
        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(webhook_json.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let mut headers = HashMap::new();
        headers.insert("x-hub-signature".to_string(), signature);

        let config = create_test_config();
        let platform = JiraPlatform::new(config).unwrap();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "jira");
        assert_eq!(message.channel_id, "PROJECT");
        assert!(message.text.contains("issue:created:PROJECT-123"));
        assert_eq!(message.user.id, "557058:12345678-1234-1234-1234-123456789012");
        assert_eq!(message.user.display_name, Some("Test User".to_string()));
        assert_eq!(message.metadata.get("issue_key").unwrap(), &serde_json::json!("PROJECT-123"));
    }
}
