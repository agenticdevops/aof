//! Integration tests for aof-triggers

use aof_triggers::command::{CommandType, TriggerCommand, TriggerTarget};
use aof_triggers::platforms::{TriggerMessage, TriggerUser};

fn create_test_message(text: &str) -> TriggerMessage {
    let user = TriggerUser {
        id: "user123".to_string(),
        username: Some("testuser".to_string()),
        display_name: Some("Test User".to_string()),
        is_bot: false,
    };

    TriggerMessage::new(
        "msg123".to_string(),
        "telegram".to_string(),
        "chat456".to_string(),
        user,
        text.to_string(),
    )
}

#[test]
fn test_trigger_message_creation() {
    let msg = create_test_message("/run agent test-bot");

    assert_eq!(msg.platform, "telegram");
    assert_eq!(msg.user.id, "user123");
    assert_eq!(msg.channel_id, "chat456");
    assert_eq!(msg.text, "/run agent test-bot");
}

#[test]
fn test_trigger_message_is_command() {
    let cmd_msg = create_test_message("/help");
    assert!(cmd_msg.is_command());

    let text_msg = create_test_message("Hello world");
    assert!(!text_msg.is_command());
}

#[test]
fn test_trigger_message_mentions_bot() {
    let mention_msg = create_test_message("Hey @aofbot can you help?");
    assert!(mention_msg.mentions_bot("aofbot"));
    assert!(!mention_msg.mentions_bot("otherbot"));
}

#[test]
fn test_command_parsing_across_targets() {
    let test_cases = vec![
        ("/run agent test-bot", CommandType::Run, TriggerTarget::Agent),
        ("/status task task-123", CommandType::Status, TriggerTarget::Task),
        ("/create fleet my-fleet", CommandType::Create, TriggerTarget::Fleet),
        ("/cancel flow flow-1", CommandType::Cancel, TriggerTarget::Flow),
        ("/list agent", CommandType::List, TriggerTarget::Agent),
    ];

    for (text, expected_cmd_type, expected_target) in test_cases {
        let msg = create_test_message(text);
        let cmd = TriggerCommand::parse(&msg).unwrap();

        assert_eq!(cmd.command_type, expected_cmd_type, "Failed for: {}", text);
        assert_eq!(cmd.target, expected_target, "Failed for: {}", text);
    }
}

#[test]
fn test_command_context_from_message() {
    let msg = create_test_message("/run agent test-bot hello");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.context.user_id, "user123");
    assert_eq!(cmd.context.channel_id, "chat456");
    assert_eq!(cmd.context.platform, "telegram");
}

#[test]
fn test_trigger_message_with_thread() {
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
    .with_thread_id("thread-456".to_string());

    assert_eq!(msg.thread_id, Some("thread-456".to_string()));
}

#[test]
fn test_trigger_message_with_metadata() {
    let user = TriggerUser {
        id: "user123".to_string(),
        username: None,
        display_name: None,
        is_bot: false,
    };

    let msg = TriggerMessage::new(
        "msg123".to_string(),
        "discord".to_string(),
        "channel123".to_string(),
        user,
        "/help".to_string(),
    )
    .with_metadata("interaction_id".to_string(), serde_json::json!("123456"));

    assert!(msg.metadata.contains_key("interaction_id"));
}
