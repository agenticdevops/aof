//! Response formatting for different platforms
//!
//! This module handles formatting responses for various messaging platforms,
//! supporting markdown, rich formatting, and interactive elements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Plain text
    Text,

    /// Markdown formatting
    Markdown,

    /// HTML formatting
    Html,

    /// Platform-specific rich format (e.g., Slack Block Kit)
    Rich,
}

/// Response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
    Warning,
    Info,
}

/// Trigger response to send back to platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResponse {
    /// Response text content
    pub text: String,

    /// Response format
    pub format: ResponseFormat,

    /// Response status
    pub status: ResponseStatus,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Attachments (file URLs, images, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,

    /// Interactive buttons/actions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,

    /// Thread ID to reply in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Message ID to reply to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

impl TriggerResponse {
    /// Create a simple text response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            format: ResponseFormat::Text,
            status: ResponseStatus::Info,
            metadata: HashMap::new(),
            attachments: Vec::new(),
            actions: Vec::new(),
            thread_id: None,
            reply_to: None,
        }
    }

    /// Create a markdown response
    pub fn markdown(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            format: ResponseFormat::Markdown,
            status: ResponseStatus::Info,
            metadata: HashMap::new(),
            attachments: Vec::new(),
            actions: Vec::new(),
            thread_id: None,
            reply_to: None,
        }
    }

    /// Create an error response
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            format: ResponseFormat::Markdown,
            status: ResponseStatus::Error,
            metadata: HashMap::new(),
            attachments: Vec::new(),
            actions: Vec::new(),
            thread_id: None,
            reply_to: None,
        }
    }

    /// Create a success response
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            format: ResponseFormat::Markdown,
            status: ResponseStatus::Success,
            metadata: HashMap::new(),
            attachments: Vec::new(),
            actions: Vec::new(),
            thread_id: None,
            reply_to: None,
        }
    }

    /// Set thread ID for threading
    pub fn with_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    /// Set reply-to message ID
    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Add an attachment
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add an action button
    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Format response for Telegram
    pub fn format_for_telegram(&self) -> String {
        // Telegram supports markdown
        match self.status {
            ResponseStatus::Success => format!("✅ {}", self.text),
            ResponseStatus::Error => format!("❌ {}", self.text),
            ResponseStatus::Warning => format!("⚠️ {}", self.text),
            ResponseStatus::Info => self.text.clone(),
        }
    }

    /// Format response for Slack (Block Kit JSON)
    pub fn format_for_slack(&self) -> serde_json::Value {
        let mut blocks = vec![serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": self.text.clone()
            }
        })];

        // Add actions if present
        if !self.actions.is_empty() {
            let elements: Vec<_> = self
                .actions
                .iter()
                .map(|action| {
                    serde_json::json!({
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": action.label.clone()
                        },
                        "action_id": action.id.clone(),
                        "value": action.value.clone()
                    })
                })
                .collect();

            blocks.push(serde_json::json!({
                "type": "actions",
                "elements": elements
            }));
        }

        serde_json::json!({ "blocks": blocks })
    }

    /// Format response for Discord
    pub fn format_for_discord(&self) -> serde_json::Value {
        let color = match self.status {
            ResponseStatus::Success => 0x00ff00, // Green
            ResponseStatus::Error => 0xff0000,   // Red
            ResponseStatus::Warning => 0xffaa00, // Orange
            ResponseStatus::Info => 0x0099ff,    // Blue
        };

        serde_json::json!({
            "embeds": [{
                "description": self.text.clone(),
                "color": color
            }]
        })
    }
}

/// Attachment (file, image, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment type
    pub attachment_type: AttachmentType,

    /// URL to attachment
    pub url: String,

    /// Optional filename
    pub filename: Option<String>,

    /// Optional title
    pub title: Option<String>,
}

/// Attachment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    File,
    Video,
    Audio,
}

/// Interactive action button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action ID
    pub id: String,

    /// Button label
    pub label: String,

    /// Action value/callback data
    pub value: String,

    /// Action style
    pub style: ActionStyle,
}

/// Action button style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    Primary,
    Secondary,
    Danger,
    Success,
}

/// Response builder for fluent API
pub struct TriggerResponseBuilder {
    response: TriggerResponse,
}

impl TriggerResponseBuilder {
    /// Create a new response builder
    pub fn new() -> Self {
        Self {
            response: TriggerResponse {
                text: String::new(),
                format: ResponseFormat::Markdown,
                status: ResponseStatus::Info,
                metadata: HashMap::new(),
                attachments: Vec::new(),
                actions: Vec::new(),
                thread_id: None,
                reply_to: None,
            },
        }
    }

    /// Set response text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.response.text = text.into();
        self
    }

    /// Set format
    pub fn format(mut self, format: ResponseFormat) -> Self {
        self.response.format = format;
        self
    }

    /// Mark as success
    pub fn success(mut self) -> Self {
        self.response.status = ResponseStatus::Success;
        self
    }

    /// Mark as error
    pub fn error(mut self) -> Self {
        self.response.status = ResponseStatus::Error;
        self
    }

    /// Mark as warning
    pub fn warning(mut self) -> Self {
        self.response.status = ResponseStatus::Warning;
        self
    }

    /// Add attachment
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.response.attachments.push(attachment);
        self
    }

    /// Add action button
    pub fn action(mut self, action: Action) -> Self {
        self.response.actions.push(action);
        self
    }

    /// Set thread ID
    pub fn thread_id(mut self, thread_id: String) -> Self {
        self.response.thread_id = Some(thread_id);
        self
    }

    /// Set reply-to
    pub fn reply_to(mut self, reply_to: String) -> Self {
        self.response.reply_to = Some(reply_to);
        self
    }

    /// Build the response
    pub fn build(self) -> TriggerResponse {
        self.response
    }
}

impl Default for TriggerResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_response() {
        let resp = TriggerResponse::text("Hello world");
        assert_eq!(resp.text, "Hello world");
        assert_eq!(resp.format, ResponseFormat::Text);
        assert_eq!(resp.status, ResponseStatus::Info);
    }

    #[test]
    fn test_builder() {
        let resp = TriggerResponseBuilder::new()
            .text("Test message")
            .success()
            .build();

        assert_eq!(resp.text, "Test message");
        assert_eq!(resp.status, ResponseStatus::Success);
    }

    #[test]
    fn test_format_for_telegram() {
        let resp = TriggerResponse::success("Task completed");
        let formatted = resp.format_for_telegram();
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("Task completed"));
    }

    #[test]
    fn test_action_builder() {
        let action = Action {
            id: "btn1".to_string(),
            label: "Click me".to_string(),
            value: "action_value".to_string(),
            style: ActionStyle::Primary,
        };

        let resp = TriggerResponseBuilder::new()
            .text("Choose an option")
            .action(action)
            .build();

        assert_eq!(resp.actions.len(), 1);
        assert_eq!(resp.actions[0].label, "Click me");
    }
}
