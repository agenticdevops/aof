//! Integration tests for Jira platform adapter
//!
//! This test suite validates the webhook integration for Jira, covering:
//! - Configuration validation
//! - Event type parsing and handling (issues, comments, sprints)
//! - Project filtering (exact, wildcard, all)
//! - Event filtering
//! - User filtering
//! - Webhook payload parsing (issue created/updated, comment created, sprint events)
//! - Signature verification (HMAC-SHA256)
//! - Platform capabilities
//! - TriggerMessage construction

use aof_triggers::platforms::{JiraConfig, JiraPlatform, TriggerPlatform};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a valid JiraConfig for testing
fn create_test_config() -> JiraConfig {
    JiraConfig {
        api_url: "https://example.atlassian.net".to_string(),
        email: "bot@example.com".to_string(),
        api_token: "test_api_token_1234567890".to_string(),
        webhook_secret: "test_webhook_secret".to_string(),
        bot_name: "jira-bot".to_string(),
        allowed_projects: None,
        allowed_events: None,
        allowed_users: None,
        enable_comments: true,
        enable_transitions: true,
    }
}

/// Create test webhook headers
fn create_test_headers(event_type: &str, signature: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("x-atlassian-webhook-identifier".to_string(), event_type.to_string());
    headers.insert("x-hub-signature".to_string(), signature.to_string());
    headers
}

/// Generate HMAC-SHA256 signature for test payloads
fn compute_test_signature(payload: &[u8], secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Sample Jira issue created webhook payload
fn sample_issue_created_payload() -> Vec<u8> {
    r#"{
        "timestamp": 1640000000000,
        "webhookEvent": "jira:issue_created",
        "issue_event_type_name": "issue_created",
        "user": {
            "self": "https://example.atlassian.net/rest/api/2/user?accountId=123",
            "accountId": "123",
            "displayName": "John Doe",
            "emailAddress": "john@example.com"
        },
        "issue": {
            "id": "10001",
            "key": "PROJ-123",
            "self": "https://example.atlassian.net/rest/api/2/issue/10001",
            "fields": {
                "summary": "Fix critical bug in authentication",
                "description": "Users are unable to login with OAuth",
                "issuetype": {
                    "id": "1",
                    "name": "Bug",
                    "subtask": false
                },
                "project": {
                    "id": "10000",
                    "key": "PROJ",
                    "name": "Project Name"
                },
                "priority": {
                    "name": "High",
                    "id": "2"
                },
                "status": {
                    "name": "To Do",
                    "id": "1",
                    "statusCategory": {
                        "key": "new"
                    }
                },
                "assignee": null,
                "reporter": {
                    "accountId": "123",
                    "displayName": "John Doe"
                },
                "created": "2025-01-01T12:00:00.000+0000",
                "updated": "2025-01-01T12:00:00.000+0000"
            }
        }
    }"#.as_bytes().to_vec()
}

/// Sample Jira issue updated webhook payload
fn sample_issue_updated_payload() -> Vec<u8> {
    r#"{
        "timestamp": 1640000000000,
        "webhookEvent": "jira:issue_updated",
        "issue_event_type_name": "issue_updated",
        "user": {
            "accountId": "456",
            "displayName": "Jane Smith",
            "emailAddress": "jane@example.com"
        },
        "issue": {
            "id": "10001",
            "key": "PROJ-123",
            "fields": {
                "summary": "Fix critical bug in authentication",
                "description": "Users are unable to login with OAuth - investigating",
                "issuetype": {
                    "name": "Bug"
                },
                "project": {
                    "key": "PROJ",
                    "name": "Project Name"
                },
                "priority": {
                    "name": "High"
                },
                "status": {
                    "name": "In Progress"
                },
                "assignee": {
                    "accountId": "456",
                    "displayName": "Jane Smith"
                }
            }
        },
        "changelog": {
            "items": [
                {
                    "field": "status",
                    "fieldtype": "jira",
                    "from": "1",
                    "fromString": "To Do",
                    "to": "3",
                    "toString": "In Progress"
                }
            ]
        }
    }"#.as_bytes().to_vec()
}

/// Sample Jira comment created webhook payload
fn sample_comment_created_payload() -> Vec<u8> {
    r#"{
        "timestamp": 1640000000000,
        "webhookEvent": "comment_created",
        "comment": {
            "self": "https://example.atlassian.net/rest/api/2/issue/10001/comment/10200",
            "id": "10200",
            "author": {
                "accountId": "789",
                "displayName": "Bob Wilson",
                "emailAddress": "bob@example.com"
            },
            "body": "I've investigated this issue and found the root cause. Working on a fix now.",
            "created": "2025-01-01T13:00:00.000+0000",
            "updated": "2025-01-01T13:00:00.000+0000"
        },
        "issue": {
            "id": "10001",
            "key": "PROJ-123",
            "fields": {
                "summary": "Fix critical bug in authentication",
                "project": {
                    "key": "PROJ",
                    "name": "Project Name"
                }
            }
        }
    }"#.as_bytes().to_vec()
}

/// Sample Jira sprint started webhook payload
fn sample_sprint_started_payload() -> Vec<u8> {
    r#"{
        "timestamp": 1640000000000,
        "webhookEvent": "sprint_started",
        "sprint": {
            "id": 10,
            "self": "https://example.atlassian.net/rest/agile/1.0/sprint/10",
            "state": "active",
            "name": "Sprint 42",
            "startDate": "2025-01-01T09:00:00.000Z",
            "endDate": "2025-01-15T17:00:00.000Z",
            "originBoardId": 5,
            "goal": "Complete authentication improvements and bug fixes"
        },
        "board": {
            "id": 5,
            "name": "Project Board",
            "type": "scrum"
        }
    }"#.as_bytes().to_vec()
}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_jira_config_creation() {
    let config = create_test_config();
    assert_eq!(config.api_url, "https://example.atlassian.net");
    assert_eq!(config.email, "bot@example.com");
    assert_eq!(config.bot_name, "jira-bot");
    assert!(config.enable_comments);
    assert!(config.enable_transitions);
}

#[test]
fn test_jira_platform_new_success() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config);
    assert!(platform.is_ok(), "Valid config should create platform");
}

#[test]
fn test_jira_platform_new_missing_config() {
    let invalid_config = JiraConfig {
        api_url: "".to_string(),
        email: "bot@example.com".to_string(),
        api_token: "token".to_string(),
        webhook_secret: "secret".to_string(),
        bot_name: "bot".to_string(),
        allowed_projects: None,
        allowed_events: None,
        allowed_users: None,
        enable_comments: true,
        enable_transitions: true,
    };

    let result = JiraPlatform::new(invalid_config);
    assert!(result.is_err(), "Empty API URL should fail");
}

#[test]
fn test_jira_config_empty_token() {
    let invalid_config = JiraConfig {
        api_url: "https://example.atlassian.net".to_string(),
        email: "bot@example.com".to_string(),
        api_token: "".to_string(),
        webhook_secret: "secret".to_string(),
        bot_name: "bot".to_string(),
        allowed_projects: None,
        allowed_events: None,
        allowed_users: None,
        enable_comments: true,
        enable_transitions: true,
    };

    let result = JiraPlatform::new(invalid_config);
    assert!(result.is_err(), "Empty API token should fail");
}

#[test]
fn test_jira_config_empty_webhook_secret() {
    let invalid_config = JiraConfig {
        api_url: "https://example.atlassian.net".to_string(),
        email: "bot@example.com".to_string(),
        api_token: "token".to_string(),
        webhook_secret: "".to_string(),
        bot_name: "bot".to_string(),
        allowed_projects: None,
        allowed_events: None,
        allowed_users: None,
        enable_comments: true,
        enable_transitions: true,
    };

    let result = JiraPlatform::new(invalid_config);
    assert!(result.is_err(), "Empty webhook secret should fail");
}

// ============================================================================
// PROJECT FILTERING TESTS
// ============================================================================

#[test]
fn test_project_filter_all_allowed() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();

    // When no filter is set, all projects should be allowed
    assert!(platform.config().allowed_projects.is_none());
}

#[test]
fn test_project_filter_whitelist() {
    let mut config = create_test_config();
    config.allowed_projects = Some(vec![
        "PROJ".to_string(),
        "TEAM".to_string(),
    ]);

    let platform = JiraPlatform::new(config).unwrap();
    let allowed = platform.config().allowed_projects.as_ref().unwrap();
    assert!(allowed.contains(&"PROJ".to_string()));
    assert!(allowed.contains(&"TEAM".to_string()));
    assert_eq!(allowed.len(), 2);
}

#[test]
fn test_project_filter_wildcard() {
    let mut config = create_test_config();
    config.allowed_projects = Some(vec!["PROJ-*".to_string()]);

    let platform = JiraPlatform::new(config).unwrap();
    assert!(platform.config().allowed_projects.is_some());
}

// ============================================================================
// EVENT FILTERING TESTS
// ============================================================================

#[test]
fn test_event_filter_all_allowed() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();

    assert!(platform.config().allowed_events.is_none());
}

#[test]
fn test_event_filter_whitelist() {
    let mut config = create_test_config();
    config.allowed_events = Some(vec![
        "jira:issue_created".to_string(),
        "jira:issue_updated".to_string(),
        "comment_created".to_string(),
    ]);

    let platform = JiraPlatform::new(config).unwrap();
    let allowed = platform.config().allowed_events.as_ref().unwrap();
    assert!(allowed.contains(&"jira:issue_created".to_string()));
    assert!(allowed.contains(&"jira:issue_updated".to_string()));
    assert!(allowed.contains(&"comment_created".to_string()));
    assert_eq!(allowed.len(), 3);
}

// ============================================================================
// USER FILTERING TESTS
// ============================================================================

#[test]
fn test_user_filter() {
    let mut config = create_test_config();
    config.allowed_users = Some(vec![
        "123".to_string(),
        "456".to_string(),
    ]);

    let platform = JiraPlatform::new(config).unwrap();
    let allowed = platform.config().allowed_users.as_ref().unwrap();
    assert!(allowed.contains(&"123".to_string()));
    assert!(allowed.contains(&"456".to_string()));
    assert_eq!(allowed.len(), 2);
}

// ============================================================================
// WEBHOOK PARSING TESTS
// ============================================================================

#[tokio::test]
async fn test_parse_issue_created_webhook() {
    let payload = sample_issue_created_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("jira:issue_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok(), "Should parse issue created webhook successfully");
    let message = result.unwrap();
    assert_eq!(message.platform, "jira");
    assert_eq!(message.channel_id, "PROJ");
    assert!(message.text.contains("issue:created"));
    assert!(message.text.contains("PROJ-123"));
    assert!(message.text.contains("Fix critical bug in authentication"));
    assert_eq!(message.user.id, "123");
    assert_eq!(message.user.username, Some("John Doe".to_string()));
    assert_eq!(message.thread_id, Some("issue-PROJ-123".to_string()));

    // Check metadata
    assert_eq!(message.metadata.get("issue_key").unwrap(), &serde_json::json!("PROJ-123"));
    assert_eq!(message.metadata.get("issue_type").unwrap(), &serde_json::json!("Bug"));
    assert_eq!(message.metadata.get("priority").unwrap(), &serde_json::json!("High"));
}

#[tokio::test]
async fn test_parse_issue_updated_webhook() {
    let payload = sample_issue_updated_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("jira:issue_updated", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok(), "Should parse issue updated webhook successfully");
    let message = result.unwrap();
    assert_eq!(message.platform, "jira");
    assert!(message.text.contains("issue:updated"));
    assert!(message.text.contains("PROJ-123"));
    assert_eq!(message.thread_id, Some("issue-PROJ-123".to_string()));

    // Check changelog metadata
    assert!(message.metadata.contains_key("changelog"));
}

#[tokio::test]
async fn test_parse_comment_created_webhook() {
    let payload = sample_comment_created_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("comment_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok(), "Should parse comment created webhook successfully");
    let message = result.unwrap();
    assert_eq!(message.platform, "jira");
    assert_eq!(message.channel_id, "PROJ");
    assert!(message.text.contains("comment:created"));
    assert!(message.text.contains("PROJ-123"));
    assert!(message.text.contains("I've investigated this issue"));
    assert_eq!(message.user.id, "789");
    assert_eq!(message.user.username, Some("Bob Wilson".to_string()));
    assert_eq!(message.thread_id, Some("issue-PROJ-123".to_string()));

    // Check comment metadata
    assert_eq!(message.metadata.get("comment_id").unwrap(), &serde_json::json!("10200"));
}

#[tokio::test]
async fn test_parse_sprint_started_webhook() {
    let payload = sample_sprint_started_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("sprint_started", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok(), "Should parse sprint started webhook successfully");
    let message = result.unwrap();
    assert_eq!(message.platform, "jira");
    assert!(message.text.contains("sprint:started"));
    assert!(message.text.contains("Sprint 42"));

    // Check sprint metadata
    assert_eq!(message.metadata.get("sprint_id").unwrap(), &serde_json::json!(10));
    assert_eq!(message.metadata.get("sprint_name").unwrap(), &serde_json::json!("Sprint 42"));
    assert!(message.metadata.contains_key("sprint_goal"));
}

#[tokio::test]
async fn test_parse_webhook_missing_signature() {
    let payload = sample_issue_created_payload();
    let mut headers = HashMap::new();
    headers.insert("x-atlassian-webhook-identifier".to_string(), "jira:issue_created".to_string());
    // Missing x-hub-signature header

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_err(), "Missing signature should fail verification");
}

#[tokio::test]
async fn test_parse_webhook_invalid_signature() {
    let payload = sample_issue_created_payload();
    let headers = create_test_headers("jira:issue_created", "sha256=invalid_signature");

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_err(), "Invalid signature should fail verification");
}

#[tokio::test]
async fn test_parse_webhook_unknown_event() {
    let payload = b"{}";
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(payload, secret);
    let headers = create_test_headers("unknown_event_type", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(payload, &headers).await;

    // Should either error or return with unknown event type
    // Exact behavior depends on implementation
    assert!(result.is_err() || result.unwrap().metadata.contains_key("unknown_event"));
}

#[tokio::test]
async fn test_parse_malformed_json() {
    let payload = b"{ invalid json";
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(payload, secret);
    let headers = create_test_headers("jira:issue_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(payload, &headers).await;

    assert!(result.is_err(), "Malformed JSON should fail");
}

// ============================================================================
// SIGNATURE VERIFICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_verify_signature_valid() {
    let payload = b"test payload data";
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(payload, secret);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.verify_signature(payload, &signature).await;

    assert!(result, "Valid signature should verify successfully");
}

#[tokio::test]
async fn test_verify_signature_invalid() {
    let payload = b"test payload data";
    let invalid_signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.verify_signature(payload, invalid_signature).await;

    assert!(!result, "Invalid signature should fail verification");
}

#[tokio::test]
async fn test_verify_signature_wrong_format() {
    let payload = b"test payload data";
    let wrong_format = "invalid_format_signature";

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.verify_signature(payload, wrong_format).await;

    assert!(!result, "Signature without sha256= prefix should fail");
}

#[tokio::test]
async fn test_verify_signature_empty() {
    let payload = b"test payload data";
    let empty_signature = "";

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.verify_signature(payload, empty_signature).await;

    assert!(!result, "Empty signature should fail verification");
}

// ============================================================================
// MESSAGE BUILDING TESTS
// ============================================================================

#[tokio::test]
async fn test_build_message_issue_event() {
    let payload = sample_issue_created_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("jira:issue_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok());
    let message = result.unwrap();

    // Verify message structure
    assert!(!message.text.is_empty());
    assert!(!message.channel_id.is_empty());
    assert!(!message.user.id.is_empty());
    assert!(message.thread_id.is_some());
    assert!(!message.metadata.is_empty());
}

#[tokio::test]
async fn test_build_message_comment_event() {
    let payload = sample_comment_created_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("comment_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok());
    let message = result.unwrap();

    // Comments should have the same thread_id as the parent issue
    assert_eq!(message.thread_id, Some("issue-PROJ-123".to_string()));
    assert!(message.metadata.contains_key("comment_id"));
}

#[tokio::test]
async fn test_build_message_sprint_event() {
    let payload = sample_sprint_started_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("sprint_started", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok());
    let message = result.unwrap();

    // Sprint events should have sprint metadata
    assert!(message.metadata.contains_key("sprint_id"));
    assert!(message.metadata.contains_key("sprint_name"));
}

#[tokio::test]
async fn test_message_metadata_fields() {
    let payload = sample_issue_created_payload();
    let secret = "test_webhook_secret";
    let signature = compute_test_signature(&payload, secret);
    let headers = create_test_headers("jira:issue_created", &signature);

    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();
    let result = platform.parse_message(&payload, &headers).await;

    assert!(result.is_ok());
    let message = result.unwrap();

    // Verify all expected metadata fields are present
    assert!(message.metadata.contains_key("issue_key"));
    assert!(message.metadata.contains_key("issue_type"));
    assert!(message.metadata.contains_key("priority"));
    assert!(message.metadata.contains_key("project_key"));
}

// ============================================================================
// PLATFORM CAPABILITIES TESTS
// ============================================================================

#[test]
fn test_platform_name() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();

    assert_eq!(platform.platform_name(), "jira");
}

#[test]
fn test_platform_capabilities() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();

    // Jira should support threading (via issues)
    assert!(platform.supports_threading());

    // Jira should support interactive features (transitions, comments)
    assert!(platform.supports_interactive());

    // Jira supports file attachments
    assert!(platform.supports_files());
}

#[test]
fn test_bot_name() {
    let config = create_test_config();
    let platform = JiraPlatform::new(config).unwrap();

    assert_eq!(platform.bot_name(), "jira-bot");
}

#[test]
fn test_custom_bot_name() {
    let mut config = create_test_config();
    config.bot_name = "custom-automation-bot".to_string();

    let platform = JiraPlatform::new(config).unwrap();
    assert_eq!(platform.bot_name(), "custom-automation-bot");
}
