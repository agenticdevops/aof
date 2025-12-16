//! Command parsing tests

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
fn test_parse_run_agent_command() {
    let msg = create_test_message("/run agent test-bot hello");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Run);
    assert_eq!(cmd.target, TriggerTarget::Agent);
    assert_eq!(cmd.args, vec!["test-bot", "hello"]);
}

#[test]
fn test_parse_status_task_command() {
    let msg = create_test_message("/status task task-123");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Status);
    assert_eq!(cmd.target, TriggerTarget::Task);
    assert_eq!(cmd.get_arg(0).unwrap(), "task-123");
}

#[test]
fn test_parse_create_fleet_command() {
    let msg = create_test_message("/create fleet my-fleet --size=5");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Create);
    assert_eq!(cmd.target, TriggerTarget::Fleet);
    assert_eq!(cmd.get_arg(0).unwrap(), "my-fleet");
    assert_eq!(cmd.get_param("size").unwrap(), "5");
}

#[test]
fn test_parse_cancel_task_command() {
    let msg = create_test_message("/cancel task task-456");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Cancel);
    assert_eq!(cmd.target, TriggerTarget::Task);
    assert_eq!(cmd.get_arg(0).unwrap(), "task-456");
}

#[test]
fn test_parse_list_agent_command() {
    let msg = create_test_message("/list agent");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::List);
    assert_eq!(cmd.target, TriggerTarget::Agent);
}

#[test]
fn test_parse_run_flow_command() {
    let msg = create_test_message("/run flow workflow-1");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Run);
    assert_eq!(cmd.target, TriggerTarget::Flow);
    assert_eq!(cmd.get_arg(0).unwrap(), "workflow-1");
}

#[test]
fn test_parse_help_command() {
    let msg = create_test_message("/help");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.command_type, CommandType::Help);
}

#[test]
fn test_parse_invalid_command_no_slash() {
    let msg = create_test_message("run agent test");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_unknown_command() {
    let msg = create_test_message("/unknown agent test");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_empty_command() {
    let msg = create_test_message("");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_parse_whitespace_only() {
    let msg = create_test_message("   ");
    let result = TriggerCommand::parse(&msg);

    assert!(result.is_err());
}

#[test]
fn test_command_type_from_str() {
    assert_eq!(CommandType::from_str("run").unwrap(), CommandType::Run);
    assert_eq!(CommandType::from_str("execute").unwrap(), CommandType::Run);
    assert_eq!(CommandType::from_str("create").unwrap(), CommandType::Create);
    assert_eq!(CommandType::from_str("status").unwrap(), CommandType::Status);
    assert_eq!(CommandType::from_str("cancel").unwrap(), CommandType::Cancel);
    assert_eq!(CommandType::from_str("list").unwrap(), CommandType::List);
    assert_eq!(CommandType::from_str("help").unwrap(), CommandType::Help);
    assert!(CommandType::from_str("invalid").is_err());
}

#[test]
fn test_target_from_str() {
    assert_eq!(TriggerTarget::from_str("agent").unwrap(), TriggerTarget::Agent);
    assert_eq!(TriggerTarget::from_str("task").unwrap(), TriggerTarget::Task);
    assert_eq!(TriggerTarget::from_str("fleet").unwrap(), TriggerTarget::Fleet);
    assert_eq!(TriggerTarget::from_str("flow").unwrap(), TriggerTarget::Flow);
    assert!(TriggerTarget::from_str("invalid").is_err());
}

#[test]
fn test_parse_command_with_multiple_params() {
    let msg = create_test_message("/create fleet my-fleet --size=5 --region=us-east --priority=high");
    let cmd = TriggerCommand::parse(&msg).unwrap();

    assert_eq!(cmd.get_param("size").unwrap(), "5");
    assert_eq!(cmd.get_param("region").unwrap(), "us-east");
    assert_eq!(cmd.get_param("priority").unwrap(), "high");
}
