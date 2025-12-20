//! Integration tests for Git platform adapters (GitHub, GitLab, Bitbucket)
//!
//! This test suite validates the webhook integration for all Git platforms,
//! covering:
//! - Configuration validation
//! - Event type parsing and handling
//! - Repository/project filtering (exact, wildcard, all)
//! - Event filtering
//! - User filtering
//! - Webhook payload parsing (PR/MR, push, issues, comments)
//! - Signature verification (HMAC-SHA256 for GitHub/Bitbucket, token for GitLab)
//! - Platform capabilities
//! - TriggerMessage construction
//! - Cross-platform consistency

use aof_triggers::platforms::{
    BitbucketConfig, BitbucketPlatform, GitHubConfig, GitHubPlatform, GitLabConfig,
    GitLabPlatform, TriggerPlatform,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// GITHUB TESTS
// ============================================================================

mod github_tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Configuration Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_valid_github_config() {
        let config = GitHubConfig {
            token: "ghp_test_token_1234567890".to_string(),
            webhook_secret: "test_webhook_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config);
        assert!(platform.is_ok(), "Valid config should create platform");
    }

    #[test]
    fn test_invalid_github_config_empty_token() {
        let config = GitHubConfig {
            token: "".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let result = GitHubPlatform::new(config);
        assert!(result.is_err(), "Empty token should fail");
    }

    #[test]
    fn test_invalid_github_config_empty_secret() {
        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let result = GitHubPlatform::new(config);
        assert!(result.is_err(), "Empty webhook secret should fail");
    }

    #[test]
    fn test_github_custom_api_url() {
        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://github.example.com/api/v3".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        assert_eq!(
            platform.config().api_url,
            "https://github.example.com/api/v3"
        );
    }

    // ------------------------------------------------------------------------
    // Event Type Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_github_event_type_parsing() {
        use aof_triggers::platforms::github::GitHubEventType;

        assert_eq!(
            GitHubEventType::from("push"),
            GitHubEventType::Push
        );
        assert_eq!(
            GitHubEventType::from("pull_request"),
            GitHubEventType::PullRequest
        );
        assert_eq!(
            GitHubEventType::from("issues"),
            GitHubEventType::Issues
        );
        assert_eq!(
            GitHubEventType::from("issue_comment"),
            GitHubEventType::IssueComment
        );
        assert_eq!(
            GitHubEventType::from("workflow_run"),
            GitHubEventType::WorkflowRun
        );
        assert_eq!(
            GitHubEventType::from("unknown_event"),
            GitHubEventType::Unknown
        );
    }

    #[test]
    fn test_github_event_type_display() {
        use aof_triggers::platforms::github::GitHubEventType;

        assert_eq!(GitHubEventType::Push.to_string(), "push");
        assert_eq!(GitHubEventType::PullRequest.to_string(), "pull_request");
        assert_eq!(GitHubEventType::Issues.to_string(), "issues");
    }

    // ------------------------------------------------------------------------
    // Repository Filtering Tests
    // ------------------------------------------------------------------------

    fn create_github_platform(
        allowed_repos: Option<Vec<String>>,
        allowed_events: Option<Vec<String>>,
        allowed_users: Option<Vec<String>>,
    ) -> GitHubPlatform {
        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos,
            allowed_events,
            allowed_users,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };
        GitHubPlatform::new(config).unwrap()
    }

    #[test]
    fn test_github_repo_filter_all_allowed() {
        let platform = create_github_platform(None, None, None);

        // When no filter is set, all repos should be allowed
        assert!(platform.config().allowed_repos.is_none());
    }

    #[test]
    fn test_github_repo_filter_exact_match() {
        let platform = create_github_platform(
            Some(vec!["owner/specific-repo".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_repos.is_some());
    }

    #[test]
    fn test_github_repo_filter_wildcard() {
        let platform = create_github_platform(
            Some(vec!["owner/*".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_repos.is_some());
    }

    #[test]
    fn test_github_repo_filter_global_wildcard() {
        let platform = create_github_platform(
            Some(vec!["*".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_repos.is_some());
    }

    // ------------------------------------------------------------------------
    // Event Filtering Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_github_event_filter_all_allowed() {
        let platform = create_github_platform(None, None, None);

        assert!(platform.config().allowed_events.is_none());
    }

    #[test]
    fn test_github_event_filter_specific() {
        let platform = create_github_platform(
            None,
            Some(vec!["push".to_string(), "pull_request".to_string()]),
            None,
        );

        let allowed = platform.config().allowed_events.as_ref().unwrap();
        assert!(allowed.contains(&"push".to_string()));
        assert!(allowed.contains(&"pull_request".to_string()));
    }

    // ------------------------------------------------------------------------
    // User Filtering Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_github_user_filter_all_allowed() {
        let platform = create_github_platform(None, None, None);

        assert!(platform.config().allowed_users.is_none());
    }

    #[test]
    fn test_github_user_filter_specific() {
        let platform = create_github_platform(
            None,
            None,
            Some(vec!["user1".to_string(), "user2".to_string()]),
        );

        let allowed = platform.config().allowed_users.as_ref().unwrap();
        assert!(allowed.contains(&"user1".to_string()));
        assert!(allowed.contains(&"user2".to_string()));
    }

    // ------------------------------------------------------------------------
    // Webhook Payload Parsing Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_github_parse_pr_opened() {
        let payload = r#"{
            "action": "opened",
            "pull_request": {
                "id": 123,
                "number": 42,
                "title": "Add new feature",
                "state": "open",
                "body": "This PR adds a new feature",
                "draft": false,
                "merged": false,
                "html_url": "https://github.com/owner/repo/pull/42",
                "additions": 100,
                "deletions": 50,
                "changed_files": 5,
                "user": {
                    "id": 456,
                    "login": "contributor",
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
                "login": "contributor",
                "type": "User"
            }
        }"#;

        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let mut headers = HashMap::new();
        headers.insert("x-github-event".to_string(), "pull_request".to_string());
        headers.insert("x-hub-signature-256".to_string(), signature);

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse PR webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "github");
        assert_eq!(message.channel_id, "owner/repo");
        assert!(message.text.contains("pr:opened"));
        assert_eq!(message.user.id, "456");
        assert_eq!(message.user.username, Some("contributor".to_string()));
        assert_eq!(
            message.metadata.get("pr_number").unwrap(),
            &serde_json::json!(42)
        );
        assert_eq!(message.thread_id, Some("pr-42".to_string()));
    }

    #[tokio::test]
    async fn test_github_parse_push() {
        let payload = r#"{
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456",
            "commits": [
                {
                    "id": "def456",
                    "message": "Fix bug in auth",
                    "timestamp": "2025-01-01T12:00:00Z",
                    "author": {
                        "name": "Developer",
                        "email": "dev@example.com"
                    },
                    "added": ["file1.rs"],
                    "removed": [],
                    "modified": ["file2.rs"]
                }
            ],
            "head_commit": {
                "id": "def456",
                "message": "Fix bug in auth",
                "timestamp": "2025-01-01T12:00:00Z",
                "author": {
                    "name": "Developer",
                    "email": "dev@example.com"
                },
                "added": ["file1.rs"],
                "removed": [],
                "modified": ["file2.rs"]
            },
            "repository": {
                "id": 789,
                "name": "repo",
                "full_name": "owner/repo",
                "private": false
            },
            "sender": {
                "id": 456,
                "login": "developer",
                "type": "User"
            }
        }"#;

        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let mut headers = HashMap::new();
        headers.insert("x-github-event".to_string(), "push".to_string());
        headers.insert("x-hub-signature-256".to_string(), signature);

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse push webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "github");
        assert!(message.text.contains("push:"));
        assert!(message.text.contains("refs/heads/main"));
    }

    #[tokio::test]
    async fn test_github_parse_issue() {
        let payload = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 10,
                "title": "Bug in login",
                "state": "open",
                "body": "Users cannot login",
                "user": {
                    "id": 456,
                    "login": "reporter",
                    "type": "User"
                },
                "html_url": "https://github.com/owner/repo/issues/10",
                "labels": []
            },
            "repository": {
                "id": 789,
                "name": "repo",
                "full_name": "owner/repo",
                "private": false
            },
            "sender": {
                "id": 456,
                "login": "reporter",
                "type": "User"
            }
        }"#;

        let secret = "test_secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let mut headers = HashMap::new();
        headers.insert("x-github-event".to_string(), "issues".to_string());
        headers.insert("x-hub-signature-256".to_string(), signature);

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse issue webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "github");
        assert!(message.text.contains("issue:opened"));
        assert_eq!(message.thread_id, Some("issue-10".to_string()));
    }

    #[tokio::test]
    async fn test_github_malformed_payload() {
        let payload = b"{ invalid json";

        let mut headers = HashMap::new();
        headers.insert("x-github-event".to_string(), "push".to_string());
        headers.insert(
            "x-hub-signature-256".to_string(),
            "sha256=invalid".to_string(),
        );

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.parse_message(payload, &headers).await;

        assert!(result.is_err(), "Malformed payload should fail");
    }

    // ------------------------------------------------------------------------
    // Signature Verification Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_github_valid_signature() {
        let payload = b"test payload";
        let secret = "test_secret";

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.verify_signature(payload, &signature).await;

        assert!(result, "Valid signature should verify");
    }

    #[tokio::test]
    async fn test_github_invalid_signature() {
        let payload = b"test payload";
        let signature = "sha256=invalid_signature";

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.verify_signature(payload, signature).await;

        assert!(!result, "Invalid signature should not verify");
    }

    #[tokio::test]
    async fn test_github_missing_signature_prefix() {
        let payload = b"test payload";
        let signature = "invalid_format";

        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();
        let result = platform.verify_signature(payload, signature).await;

        assert!(!result, "Signature without sha256= prefix should fail");
    }

    // ------------------------------------------------------------------------
    // Platform Capabilities Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_github_capabilities() {
        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };

        let platform = GitHubPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "github");
        assert_eq!(platform.bot_name(), "test-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }
}

// ============================================================================
// GITLAB TESTS
// ============================================================================

mod gitlab_tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Configuration Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_valid_gitlab_config() {
        let config = GitLabConfig {
            token: "glpat-test_token_1234567890".to_string(),
            webhook_secret: "test_webhook_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config);
        assert!(platform.is_ok(), "Valid config should create platform");
    }

    #[test]
    fn test_invalid_gitlab_config_empty_token() {
        let config = GitLabConfig {
            token: "".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let result = GitLabPlatform::new(config);
        assert!(result.is_err(), "Empty token should fail");
    }

    #[test]
    fn test_gitlab_self_hosted_api_url() {
        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.example.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        assert_eq!(
            platform.config().api_url,
            "https://gitlab.example.com/api/v4"
        );
    }

    // ------------------------------------------------------------------------
    // Event Type Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_gitlab_event_type_parsing() {
        use aof_triggers::platforms::gitlab::GitLabEventType;

        assert_eq!(
            GitLabEventType::from("Push Hook"),
            GitLabEventType::Push
        );
        assert_eq!(
            GitLabEventType::from("Merge Request Hook"),
            GitLabEventType::MergeRequest
        );
        assert_eq!(
            GitLabEventType::from("Issue Hook"),
            GitLabEventType::Issue
        );
        assert_eq!(
            GitLabEventType::from("Note Hook"),
            GitLabEventType::Note
        );
        assert_eq!(
            GitLabEventType::from("Pipeline Hook"),
            GitLabEventType::Pipeline
        );
        assert_eq!(
            GitLabEventType::from("unknown_event"),
            GitLabEventType::Unknown
        );
    }

    #[test]
    fn test_gitlab_event_type_display() {
        use aof_triggers::platforms::gitlab::GitLabEventType;

        assert_eq!(GitLabEventType::Push.to_string(), "push");
        assert_eq!(GitLabEventType::MergeRequest.to_string(), "merge_request");
        assert_eq!(GitLabEventType::Issue.to_string(), "issue");
        assert_eq!(GitLabEventType::Note.to_string(), "note");
    }

    // ------------------------------------------------------------------------
    // Project Filtering Tests
    // ------------------------------------------------------------------------

    fn create_gitlab_platform(
        allowed_projects: Option<Vec<String>>,
        allowed_events: Option<Vec<String>>,
        allowed_users: Option<Vec<String>>,
    ) -> GitLabPlatform {
        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects,
            allowed_events,
            allowed_users,
            enable_comments: true,
            enable_approvals: true,
        };
        GitLabPlatform::new(config).unwrap()
    }

    #[test]
    fn test_gitlab_project_filter_all_allowed() {
        let platform = create_gitlab_platform(None, None, None);

        assert!(platform.config().allowed_projects.is_none());
    }

    #[test]
    fn test_gitlab_project_filter_exact_match() {
        let platform = create_gitlab_platform(
            Some(vec!["group/specific-project".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_projects.is_some());
    }

    #[test]
    fn test_gitlab_project_filter_wildcard() {
        let platform = create_gitlab_platform(
            Some(vec!["group/*".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_projects.is_some());
    }

    // ------------------------------------------------------------------------
    // Webhook Payload Parsing Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_gitlab_parse_mr_opened() {
        let payload = r#"{
            "object_kind": "merge_request",
            "event_type": "merge_request",
            "object_attributes": {
                "id": 123,
                "iid": 42,
                "title": "Add new feature",
                "state": "opened",
                "description": "This MR adds a new feature",
                "source_branch": "feature-branch",
                "target_branch": "main",
                "work_in_progress": false,
                "draft": false,
                "merge_status": "can_be_merged",
                "web_url": "https://gitlab.com/group/project/-/merge_requests/42",
                "author": {
                    "id": 456,
                    "username": "contributor",
                    "name": "Contributor"
                },
                "last_commit": {
                    "id": "abc123",
                    "message": "Add feature",
                    "timestamp": "2025-01-01T00:00:00Z",
                    "author": {
                        "name": "Contributor",
                        "email": "contributor@example.com"
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
                "username": "contributor",
                "name": "Contributor"
            }
        }"#;

        let mut headers = HashMap::new();
        headers.insert(
            "x-gitlab-event".to_string(),
            "Merge Request Hook".to_string(),
        );
        headers.insert("x-gitlab-token".to_string(), "test_secret".to_string());

        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse MR webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "gitlab");
        assert_eq!(message.channel_id, "group/project");
        assert!(message.text.contains("mr:"));
        assert_eq!(message.user.id, "456");
        assert_eq!(message.user.username, Some("contributor".to_string()));
        assert_eq!(
            message.metadata.get("mr_iid").unwrap(),
            &serde_json::json!(42)
        );
        assert_eq!(message.thread_id, Some("mr-42".to_string()));
    }

    #[tokio::test]
    async fn test_gitlab_parse_push() {
        let payload = r#"{
            "object_kind": "push",
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456",
            "total_commits_count": 1,
            "commits": [
                {
                    "id": "def456",
                    "message": "Fix bug",
                    "timestamp": "2025-01-01T12:00:00Z",
                    "author": {
                        "name": "Developer",
                        "email": "dev@example.com"
                    }
                }
            ],
            "project": {
                "id": 789,
                "name": "project",
                "path_with_namespace": "group/project"
            },
            "user": {
                "id": 456,
                "username": "developer",
                "name": "Developer"
            }
        }"#;

        let mut headers = HashMap::new();
        headers.insert("x-gitlab-event".to_string(), "Push Hook".to_string());
        headers.insert("x-gitlab-token".to_string(), "test_secret".to_string());

        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse push webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "gitlab");
        assert!(message.text.contains("push:"));
        assert!(message.text.contains("refs/heads/main"));
    }

    #[tokio::test]
    async fn test_gitlab_parse_issue() {
        let payload = r#"{
            "object_kind": "issue",
            "issue": {
                "id": 123,
                "iid": 10,
                "title": "Bug in login",
                "state": "opened",
                "description": "Users cannot login",
                "author": {
                    "id": 456,
                    "username": "reporter",
                    "name": "Reporter"
                },
                "labels": []
            },
            "project": {
                "id": 789,
                "name": "project",
                "path_with_namespace": "group/project"
            },
            "user": {
                "id": 456,
                "username": "reporter",
                "name": "Reporter"
            }
        }"#;

        let mut headers = HashMap::new();
        headers.insert("x-gitlab-event".to_string(), "Issue Hook".to_string());
        headers.insert("x-gitlab-token".to_string(), "test_secret".to_string());

        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        let result = platform.parse_message(payload.as_bytes(), &headers).await;

        assert!(result.is_ok(), "Should parse issue webhook successfully");
        let message = result.unwrap();
        assert_eq!(message.platform, "gitlab");
        assert!(message.text.contains("issue:"));
        assert_eq!(message.thread_id, Some("issue-10".to_string()));
    }

    // ------------------------------------------------------------------------
    // Signature Verification Tests (Token-based)
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_gitlab_valid_token() {
        let payload = b"test payload";
        let token = "test_secret";

        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        let result = platform.verify_signature(payload, token).await;

        assert!(result, "Valid token should verify");
    }

    #[tokio::test]
    async fn test_gitlab_invalid_token() {
        let payload = b"test payload";
        let token = "wrong_secret";

        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();
        let result = platform.verify_signature(payload, token).await;

        assert!(!result, "Invalid token should not verify");
    }

    // ------------------------------------------------------------------------
    // Platform Capabilities Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_gitlab_capabilities() {
        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "test_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };

        let platform = GitLabPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "gitlab");
        assert_eq!(platform.bot_name(), "test-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }
}

// ============================================================================
// BITBUCKET TESTS
// ============================================================================

mod bitbucket_tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Configuration Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_valid_bitbucket_config_app_password() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("test_app_password".to_string()),
            oauth_token: None,
            webhook_secret: "test_webhook_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };

        let platform = BitbucketPlatform::new(config);
        assert!(platform.is_ok(), "Valid config should create platform");
    }

    #[test]
    fn test_valid_bitbucket_config_oauth() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: None,
            oauth_token: Some("oauth_token_123".to_string()),
            webhook_secret: "test_webhook_secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };

        let platform = BitbucketPlatform::new(config);
        assert!(platform.is_ok(), "Valid OAuth config should create platform");
    }

    #[test]
    fn test_invalid_bitbucket_config_empty_secret() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("password".to_string()),
            oauth_token: None,
            webhook_secret: "".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };

        let result = BitbucketPlatform::new(config);
        assert!(result.is_err(), "Empty webhook secret should fail");
    }

    #[test]
    fn test_bitbucket_server_api_url() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("password".to_string()),
            oauth_token: None,
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://bitbucket.example.com/rest/api/1.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };

        let platform = BitbucketPlatform::new(config).unwrap();
        assert_eq!(
            platform.config().api_url,
            "https://bitbucket.example.com/rest/api/1.0"
        );
    }

    // ------------------------------------------------------------------------
    // Event Type Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bitbucket_event_type_parsing() {
        use aof_triggers::platforms::bitbucket::BitbucketEventType;

        assert_eq!(
            BitbucketEventType::from("pullrequest:created"),
            BitbucketEventType::PullRequestCreated
        );
        assert_eq!(
            BitbucketEventType::from("pullrequest:updated"),
            BitbucketEventType::PullRequestUpdated
        );
        assert_eq!(
            BitbucketEventType::from("pullrequest:fulfilled"),
            BitbucketEventType::PullRequestMerged
        );
        assert_eq!(
            BitbucketEventType::from("repo:push"),
            BitbucketEventType::RepoPush
        );
    }

    #[test]
    fn test_bitbucket_event_type_display() {
        use aof_triggers::platforms::bitbucket::BitbucketEventType;

        assert_eq!(
            BitbucketEventType::PullRequestCreated.to_string(),
            "pullrequest:created"
        );
        assert_eq!(
            BitbucketEventType::RepoPush.to_string(),
            "repo:push"
        );
    }

    // ------------------------------------------------------------------------
    // Repository Filtering Tests
    // ------------------------------------------------------------------------

    fn create_bitbucket_platform(
        allowed_repos: Option<Vec<String>>,
        allowed_events: Option<Vec<String>>,
        allowed_users: Option<Vec<String>>,
    ) -> BitbucketPlatform {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("password".to_string()),
            oauth_token: None,
            webhook_secret: "secret".to_string(),
            bot_name: "bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos,
            allowed_events,
            allowed_users,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };
        BitbucketPlatform::new(config).unwrap()
    }

    #[test]
    fn test_bitbucket_repo_filter_all_allowed() {
        let platform = create_bitbucket_platform(None, None, None);

        assert!(platform.config().allowed_repos.is_none());
    }

    #[test]
    fn test_bitbucket_repo_filter_exact_match() {
        let platform = create_bitbucket_platform(
            Some(vec!["workspace/specific-repo".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_repos.is_some());
    }

    #[test]
    fn test_bitbucket_repo_filter_wildcard() {
        let platform = create_bitbucket_platform(
            Some(vec!["workspace/*".to_string()]),
            None,
            None,
        );

        assert!(platform.config().allowed_repos.is_some());
    }

    // ------------------------------------------------------------------------
    // Platform Capabilities Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bitbucket_capabilities() {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("password".to_string()),
            oauth_token: None,
            webhook_secret: "secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };

        let platform = BitbucketPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "bitbucket");
        assert_eq!(platform.bot_name(), "test-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_files());
    }
}

// ============================================================================
// CROSS-PLATFORM TESTS
// ============================================================================

mod cross_platform_tests {
    use super::*;
    use aof_triggers::platforms::get_platform_capabilities;

    /// Test that all platforms have consistent behavior for basic operations
    #[test]
    fn test_platform_name_consistency() {
        let github = create_test_github_platform();
        let gitlab = create_test_gitlab_platform();
        let bitbucket = create_test_bitbucket_platform();

        assert_eq!(github.platform_name(), "github");
        assert_eq!(gitlab.platform_name(), "gitlab");
        assert_eq!(bitbucket.platform_name(), "bitbucket");
    }

    #[test]
    fn test_platform_capabilities_consistency() {
        let github_caps = get_platform_capabilities("github");
        let gitlab_caps = get_platform_capabilities("gitlab");
        let bitbucket_caps = get_platform_capabilities("bitbucket");

        // All Git platforms should support threading
        assert!(github_caps.threading);
        assert!(gitlab_caps.threading);
        assert!(bitbucket_caps.threading);

        // All Git platforms should support files
        assert!(github_caps.files);
        assert!(gitlab_caps.files);
        assert!(bitbucket_caps.files);

        // All Git platforms should support rich text/markdown
        assert!(github_caps.rich_text);
        assert!(gitlab_caps.rich_text);
        assert!(bitbucket_caps.rich_text);

        // All Git platforms should support approvals
        assert!(github_caps.approvals);
        assert!(gitlab_caps.approvals);
        assert!(bitbucket_caps.approvals);
    }

    #[test]
    fn test_bot_name_customization() {
        let github = create_test_github_platform();
        let gitlab = create_test_gitlab_platform();
        let bitbucket = create_test_bitbucket_platform();

        // All platforms should allow custom bot names
        assert!(github.bot_name().len() > 0);
        assert!(gitlab.bot_name().len() > 0);
        assert!(bitbucket.bot_name().len() > 0);
    }

    // Helper functions to create test platforms
    fn create_test_github_platform() -> GitHubPlatform {
        let config = GitHubConfig {
            token: "ghp_token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.github.com".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            auto_approve_patterns: None,
            enable_status_checks: true,
            enable_reviews: true,
            enable_comments: true,
        };
        GitHubPlatform::new(config).unwrap()
    }

    fn create_test_gitlab_platform() -> GitLabPlatform {
        let config = GitLabConfig {
            token: "glpat-token".to_string(),
            webhook_secret: "secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://gitlab.com/api/v4".to_string(),
            allowed_projects: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
        };
        GitLabPlatform::new(config).unwrap()
    }

    fn create_test_bitbucket_platform() -> BitbucketPlatform {
        let config = BitbucketConfig {
            username: "testuser".to_string(),
            app_password: Some("password".to_string()),
            oauth_token: None,
            webhook_secret: "secret".to_string(),
            bot_name: "test-bot".to_string(),
            api_url: "https://api.bitbucket.org/2.0".to_string(),
            allowed_repos: None,
            allowed_events: None,
            allowed_users: None,
            enable_comments: true,
            enable_approvals: true,
            enable_build_status: true,
        };
        BitbucketPlatform::new(config).unwrap()
    }
}
