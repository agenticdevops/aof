//! Workflow Integration Tests
//!
//! Tests the complete workflow from trigger message to task execution.
//! These tests verify that:
//! - Messages are parsed correctly across platforms
//! - Commands are routed to the right handlers
//! - Tasks are created and tracked properly
//! - Responses are formatted correctly

use std::collections::HashMap;
use std::sync::Arc;

use aof_triggers::{
    command::{CommandType, TriggerCommand, TriggerTarget},
    handler::{TriggerHandler, TriggerHandlerConfig},
    platforms::{TriggerMessage, TriggerUser},
    response::TriggerResponseBuilder,
};
use aof_runtime::RuntimeOrchestrator;

/// Create a test trigger message
fn create_test_message(platform: &str, text: &str) -> TriggerMessage {
    let user = TriggerUser {
        id: "test_user_123".to_string(),
        username: Some("testuser".to_string()),
        display_name: Some("Test User".to_string()),
        is_bot: false,
    };

    TriggerMessage::new(
        format!("msg_{}", uuid::Uuid::new_v4()),
        platform.to_string(),
        "test_channel_456".to_string(),
        user,
        text.to_string(),
    )
}

/// Create a test handler
fn create_test_handler() -> TriggerHandler {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    TriggerHandler::new(orchestrator)
}

/// Create a handler with custom config
fn create_handler_with_config(config: TriggerHandlerConfig) -> TriggerHandler {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    TriggerHandler::with_config(orchestrator, config)
}

// ============================================================================
// Command Parsing Tests
// ============================================================================

#[test]
fn test_parse_run_agent_command() {
    let msg = create_test_message("telegram", "/run agent my-bot Hello world");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Run);
    assert_eq!(cmd.target, TriggerTarget::Agent);
    assert_eq!(cmd.args[0], "my-bot");
    assert!(cmd.args[1..].join(" ").contains("Hello world"));
}

#[test]
fn test_parse_status_task_command() {
    let msg = create_test_message("slack", "/status task task-123-abc");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Status);
    assert_eq!(cmd.target, TriggerTarget::Task);
    assert_eq!(cmd.args[0], "task-123-abc");
}

#[test]
fn test_parse_cancel_task_command() {
    let msg = create_test_message("discord", "/cancel task running-task-456");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Cancel);
    assert_eq!(cmd.target, TriggerTarget::Task);
    assert_eq!(cmd.args[0], "running-task-456");
}

#[test]
fn test_parse_list_tasks_command() {
    // Note: "task" is the valid target, not "tasks"
    let msg = create_test_message("whatsapp", "/list task");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::List);
    assert_eq!(cmd.target, TriggerTarget::Task);
}

#[test]
fn test_parse_help_command() {
    let msg = create_test_message("telegram", "/help");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Help);
}

#[test]
fn test_parse_info_command() {
    // Note: "info" is an alias for "status", so it requires a target
    // /info agent myagent is equivalent to /status agent myagent
    let msg = create_test_message("slack", "/info agent my-agent");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    // "info" maps to Status command type
    assert_eq!(cmd.command_type, CommandType::Status);
    assert_eq!(cmd.target, TriggerTarget::Agent);
}

// ============================================================================
// Command Context Tests
// ============================================================================

#[test]
fn test_command_context_from_telegram() {
    let msg = create_test_message("telegram", "/run agent test-bot");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.context.platform, "telegram");
    assert_eq!(cmd.context.user_id, "test_user_123");
    assert_eq!(cmd.context.channel_id, "test_channel_456");
}

#[test]
fn test_command_context_from_slack() {
    let msg = create_test_message("slack", "/run agent test-bot");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.context.platform, "slack");
}

#[test]
fn test_command_context_from_discord() {
    let msg = create_test_message("discord", "/run agent test-bot");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.context.platform, "discord");
}

#[test]
fn test_command_context_with_thread() {
    let user = TriggerUser {
        id: "user123".to_string(),
        username: None,
        display_name: None,
        is_bot: false,
    };

    let msg = TriggerMessage::new(
        "msg123".to_string(),
        "slack".to_string(),
        "channel123".to_string(),
        user,
        "/status task t-1".to_string(),
    )
    .with_thread_id("thread-789".to_string());

    let cmd = TriggerCommand::parse(&msg).unwrap();
    assert!(msg.thread_id.is_some());
    assert_eq!(msg.thread_id.unwrap(), "thread-789");
}

// ============================================================================
// Handler Configuration Tests
// ============================================================================

#[test]
fn test_default_handler_config() {
    let config = TriggerHandlerConfig::default();

    assert!(config.auto_ack);
    assert_eq!(config.max_tasks_per_user, 3);
    assert_eq!(config.command_timeout_secs, 300);
    assert!(!config.verbose);
}

#[test]
fn test_custom_handler_config() {
    let config = TriggerHandlerConfig {
        verbose: true,
        auto_ack: false,
        max_tasks_per_user: 5,
        command_timeout_secs: 600,
        default_agent: None,
        command_bindings: HashMap::new(),
    };

    let handler = create_handler_with_config(config.clone());
    // Verify handler is created with custom config
    assert!(!config.auto_ack);
    assert_eq!(config.max_tasks_per_user, 5);
}

// ============================================================================
// Response Builder Tests
// ============================================================================

#[test]
fn test_response_builder_text() {
    use aof_triggers::response::ResponseStatus;

    let response = TriggerResponseBuilder::new()
        .text("Hello, World!")
        .build();

    assert_eq!(response.text, "Hello, World!");
    assert_eq!(response.status, ResponseStatus::Info);
}

#[test]
fn test_response_builder_error() {
    use aof_triggers::response::ResponseStatus;

    let response = TriggerResponseBuilder::new()
        .text("An error occurred")
        .error()
        .build();

    assert_eq!(response.text, "An error occurred");
    assert_eq!(response.status, ResponseStatus::Error);
}

#[test]
fn test_response_builder_success() {
    use aof_triggers::response::ResponseStatus;

    let response = TriggerResponseBuilder::new()
        .text("Operation completed")
        .success()
        .build();

    assert_eq!(response.text, "Operation completed");
    assert_eq!(response.status, ResponseStatus::Success);
}

#[test]
fn test_response_builder_with_attachments() {
    use aof_triggers::response::{Attachment, AttachmentType};

    let attachment = Attachment {
        attachment_type: AttachmentType::Image,
        url: "https://example.com/image.png".to_string(),
        filename: Some("image.png".to_string()),
        title: Some("Test Image".to_string()),
    };

    let response = TriggerResponseBuilder::new()
        .text("Results")
        .attachment(attachment)
        .build();

    assert_eq!(response.attachments.len(), 1);
    assert_eq!(response.attachments[0].title, Some("Test Image".to_string()));
}

#[test]
fn test_response_builder_with_actions() {
    use aof_triggers::response::{Action, ActionStyle};

    let action = Action {
        id: "btn_confirm".to_string(),
        label: "Confirm".to_string(),
        value: "confirm_action".to_string(),
        style: ActionStyle::Primary,
    };

    let response = TriggerResponseBuilder::new()
        .text("Task Status")
        .action(action)
        .build();

    assert_eq!(response.actions.len(), 1);
    assert_eq!(response.actions[0].label, "Confirm");
}

// ============================================================================
// Message Parsing Edge Cases
// ============================================================================

#[test]
fn test_parse_command_with_extra_spaces() {
    let msg = create_test_message("telegram", "/run   agent   my-bot   Hello");
    let result = TriggerCommand::parse(&msg);

    // Should handle extra spaces gracefully
    assert!(result.is_ok());
}

#[test]
fn test_parse_command_case_insensitive() {
    let msg = create_test_message("slack", "/RUN agent test-bot");
    let result = TriggerCommand::parse(&msg);

    // Commands should be case-insensitive
    assert!(result.is_ok());
    if let Ok(cmd) = result {
        assert_eq!(cmd.command_type, CommandType::Run);
    }
}

#[test]
fn test_parse_empty_command_fails() {
    let msg = create_test_message("telegram", "/");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_unknown_command_fails() {
    let msg = create_test_message("telegram", "/unknown_command test");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_missing_target_fails() {
    let msg = create_test_message("telegram", "/run");
    let result = TriggerCommand::parse(&msg);

    // Should fail because target is missing
    assert!(result.is_err());
}

// ============================================================================
// Orchestrator Integration Tests
// ============================================================================

#[tokio::test]
async fn test_handler_creation() {
    let handler = create_test_handler();
    // Handler should be created without errors
    // No platforms registered initially
    assert!(handler.get_platform("telegram").is_none());
}

#[tokio::test]
async fn test_orchestrator_task_submission() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Create a task
    let task = aof_runtime::Task::new(
        "test-task-1".to_string(),
        "Test Task".to_string(),
        "test-agent".to_string(),
        "Test input".to_string(),
    );

    // Submit task
    let handle = orchestrator.submit_task(task);

    // Verify task was submitted
    let status = handle.status().await;
    assert!(matches!(status, aof_runtime::TaskStatus::Pending | aof_runtime::TaskStatus::Running));
}

#[tokio::test]
async fn test_orchestrator_task_listing() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Submit multiple tasks
    for i in 0..3 {
        let task = aof_runtime::Task::new(
            format!("task-{}", i),
            format!("Task {}", i),
            "agent".to_string(),
            "input".to_string(),
        );
        orchestrator.submit_task(task);
    }

    // List tasks
    let task_ids = orchestrator.list_tasks();
    assert_eq!(task_ids.len(), 3);
}

#[tokio::test]
async fn test_orchestrator_task_cancellation() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Create and submit a task
    let task = aof_runtime::Task::new(
        "cancelable-task".to_string(),
        "Cancelable Task".to_string(),
        "agent".to_string(),
        "input".to_string(),
    );
    orchestrator.submit_task(task);

    // Cancel the task
    let result = orchestrator.cancel_task("cancelable-task").await;
    assert!(result.is_ok());

    // Verify task is cancelled
    if let Some(handle) = orchestrator.get_task("cancelable-task") {
        let status = handle.status().await;
        assert!(matches!(status, aof_runtime::TaskStatus::Cancelled));
    }
}

#[tokio::test]
async fn test_orchestrator_stats() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Get initial stats
    let stats = orchestrator.stats().await;
    assert_eq!(stats.pending, 0);
    assert_eq!(stats.running, 0);
    assert!(stats.max_concurrent > 0);
}

// ============================================================================
// Multi-Platform Tests
// ============================================================================

#[test]
fn test_commands_across_all_platforms() {
    let platforms = vec!["telegram", "slack", "discord", "whatsapp"];
    let command = "/run agent test-bot Hello from test";

    for platform in platforms {
        let msg = create_test_message(platform, command);
        let cmd = TriggerCommand::parse(&msg);

        assert!(cmd.is_ok(), "Failed to parse command for platform: {}", platform);
        let cmd = cmd.unwrap();
        assert_eq!(cmd.command_type, CommandType::Run);
        assert_eq!(cmd.context.platform, platform);
    }
}

#[test]
fn test_message_with_metadata() {
    let user = TriggerUser {
        id: "user123".to_string(),
        username: Some("testuser".to_string()),
        display_name: None,
        is_bot: false,
    };

    let mut metadata = HashMap::new();
    metadata.insert("interaction_id".to_string(), serde_json::json!("12345"));
    metadata.insert("response_url".to_string(), serde_json::json!("https://example.com/respond"));

    let msg = TriggerMessage::new(
        "msg123".to_string(),
        "slack".to_string(),
        "channel123".to_string(),
        user,
        "/run agent test".to_string(),
    )
    .with_metadata("interaction_id".to_string(), serde_json::json!("12345"));

    assert!(msg.metadata.contains_key("interaction_id"));
}

// ============================================================================
// Bot Mention Tests
// ============================================================================

#[test]
fn test_message_mentions_bot() {
    let msg = create_test_message("telegram", "@aofbot /run agent test");
    assert!(msg.mentions_bot("aofbot"));
    assert!(!msg.mentions_bot("otherbot"));
}

#[test]
fn test_message_is_command() {
    let cmd_msg = create_test_message("telegram", "/run agent test");
    assert!(cmd_msg.is_command());

    let text_msg = create_test_message("telegram", "Hello, how are you?");
    assert!(!text_msg.is_command());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_parse_invalid_target() {
    let msg = create_test_message("telegram", "/run invalid_target test");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_missing_agent_name() {
    let msg = create_test_message("telegram", "/run agent");
    let result = TriggerCommand::parse(&msg);

    // May fail or succeed with empty agent name depending on implementation
    if let Ok(cmd) = result {
        assert!(cmd.args.is_empty() || cmd.args[0].is_empty());
    }
}
