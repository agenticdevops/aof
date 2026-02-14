//! Event translation layer
//!
//! This module handles translation between platform-specific messages and
//! CoordinationEvent format used by the agent runtime.

use aof_core::{AofError, CoordinationEvent, ActivityEvent, ActivityType};

use crate::adapters::InboundMessage;

/// Translate inbound message to CoordinationEvent for agent runtime
pub fn translate_to_coordination_event(
    message: &InboundMessage,
    session_id: &str,
) -> Result<CoordinationEvent, AofError> {
    // Create ActivityEvent with custom info type
    let event_message = format!(
        "Message received from {:?} in channel {}",
        message.platform, message.channel_id
    );

    let mut activity = ActivityEvent::new(ActivityType::Info, event_message);

    // Add message metadata as additional details
    if let Some(ref mut details) = activity.details {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("message_id".to_string(), message.message_id.clone());
        metadata.insert("platform".to_string(), format!("{:?}", message.platform));
        metadata.insert("channel_id".to_string(), message.channel_id.clone());
        metadata.insert("user_id".to_string(), message.user.user_id.clone());
        metadata.insert("content".to_string(), message.content.clone());
        if let Some(ref thread_id) = message.thread_id {
            metadata.insert("thread_id".to_string(), thread_id.clone());
        }
        details.metadata = Some(metadata);
    } else {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("message_id".to_string(), message.message_id.clone());
        metadata.insert("platform".to_string(), format!("{:?}", message.platform));
        metadata.insert("channel_id".to_string(), message.channel_id.clone());
        metadata.insert("user_id".to_string(), message.user.user_id.clone());
        metadata.insert("content".to_string(), message.content.clone());
        if let Some(ref thread_id) = message.thread_id {
            metadata.insert("thread_id".to_string(), thread_id.clone());
        }
        activity.details = Some(aof_core::ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: Some(metadata),
        });
    }

    // Wrap in CoordinationEvent (from aof-core)
    let agent_id = format!("gateway-{:?}", message.platform).to_lowercase();
    Ok(CoordinationEvent::from_activity(activity, agent_id, session_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{Platform, MessageUser};
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_translate_slack_message() {
        let message = InboundMessage {
            message_id: "1234.5678".to_string(),
            platform: Platform::Slack,
            channel_id: "C1234567890".to_string(),
            thread_id: None,
            user: MessageUser {
                user_id: "U1234567890".to_string(),
                username: "testuser".to_string(),
                display_name: Some("Test User".to_string()),
            },
            content: "Hello, agent!".to_string(),
            attachments: vec![],
            metadata: json!({}),
            timestamp: Utc::now(),
        };

        let event = translate_to_coordination_event(&message, "test-session").unwrap();
        assert_eq!(event.session_id, "test-session");
        assert_eq!(event.agent_id, "gateway-slack");
    }
}
