//! Telegram Bot API adapter for AOF
//!
//! This module provides integration with Telegram's Bot API, supporting:
//! - Text messages with /commands
//! - Inline keyboards for interactive responses
//! - Callback queries from button clicks
//! - Webhook secret token verification

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

/// Telegram platform adapter
pub struct TelegramPlatform {
    config: TelegramConfig,
    client: reqwest::Client,
}

/// Telegram configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot token from @BotFather
    pub bot_token: String,

    /// Webhook URL (for setting up webhook)
    #[serde(default)]
    pub webhook_url: Option<String>,

    /// Secret token for webhook verification
    #[serde(default)]
    pub webhook_secret: Option<String>,

    /// Bot username (without @)
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Allowed user IDs (optional whitelist)
    #[serde(default)]
    pub allowed_users: Option<Vec<i64>>,

    /// Allowed group/chat IDs (optional whitelist)
    #[serde(default)]
    pub allowed_groups: Option<Vec<i64>>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

/// Telegram Update object (webhook payload)
#[derive(Debug, Clone, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    #[serde(default)]
    message: Option<TelegramMessage>,
    #[serde(default)]
    callback_query: Option<CallbackQuery>,
    #[serde(default)]
    inline_query: Option<InlineQuery>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct TelegramMessage {
    #[serde(default)]
    message_id: i64,
    #[serde(default)]
    from: Option<TelegramUser>,
    #[serde(default)]
    chat: TelegramChat,
    #[serde(default)]
    date: i64,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    reply_to_message: Option<Box<TelegramMessage>>,
}

#[derive(Debug, Clone, Deserialize)]
struct TelegramUser {
    id: i64,
    is_bot: bool,
    first_name: String,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct TelegramChat {
    #[serde(default)]
    id: i64,
    #[serde(rename = "type", default)]
    chat_type: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CallbackQuery {
    id: String,
    from: TelegramUser,
    #[serde(default)]
    message: Option<TelegramMessage>,
    #[serde(default)]
    inline_message_id: Option<String>,
    chat_instance: String,
    #[serde(default)]
    data: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InlineQuery {
    id: String,
    from: TelegramUser,
    query: String,
    offset: String,
}

/// Telegram API response
#[derive(Debug, Deserialize)]
struct TelegramApiResponse<T> {
    ok: bool,
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    error_code: Option<i32>,
}

/// Inline keyboard markup
#[derive(Debug, Clone, Serialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Clone, Serialize)]
struct InlineKeyboardButton {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl TelegramPlatform {
    /// Create new Telegram platform adapter
    pub fn new(config: TelegramConfig) -> Result<Self, PlatformError> {
        if config.bot_token.is_empty() {
            return Err(PlatformError::ParseError(
                "Bot token is required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Get API base URL
    fn api_url(&self, method: &str) -> String {
        format!(
            "https://api.telegram.org/bot{}/{}",
            self.config.bot_token, method
        )
    }

    /// Set webhook
    pub async fn set_webhook(&self, url: &str) -> Result<bool, PlatformError> {
        let mut params = serde_json::json!({
            "url": url,
            "allowed_updates": ["message", "callback_query", "inline_query"]
        });

        if let Some(ref secret) = self.config.webhook_secret {
            params["secret_token"] = serde_json::json!(secret);
        }

        let response: TelegramApiResponse<bool> = self
            .client
            .post(&self.api_url("setWebhook"))
            .json(&params)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if response.ok {
            info!("Telegram webhook set successfully");
            Ok(true)
        } else {
            Err(PlatformError::ApiError(
                response.description.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Delete webhook
    pub async fn delete_webhook(&self) -> Result<bool, PlatformError> {
        let response: TelegramApiResponse<bool> = self
            .client
            .post(&self.api_url("deleteWebhook"))
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if response.ok {
            info!("Telegram webhook deleted");
            Ok(true)
        } else {
            Err(PlatformError::ApiError(
                response.description.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Send message
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        reply_to: Option<i64>,
        keyboard: Option<InlineKeyboardMarkup>,
    ) -> Result<i64, PlatformError> {
        let mut params = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "MarkdownV2"
        });

        if let Some(reply_to_id) = reply_to {
            params["reply_to_message_id"] = serde_json::json!(reply_to_id);
        }

        if let Some(kb) = keyboard {
            params["reply_markup"] = serde_json::to_value(kb)
                .map_err(|e| PlatformError::ParseError(format!("Failed to serialize keyboard: {}", e)))?;
        }

        let response: TelegramApiResponse<TelegramMessage> = self
            .client
            .post(&self.api_url("sendMessage"))
            .json(&params)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if response.ok {
            let message_id = response.result.map(|m| m.message_id).unwrap_or(0);
            debug!("Sent Telegram message: {}", message_id);
            Ok(message_id)
        } else {
            error!("Telegram API error: {:?}", response.description);
            Err(PlatformError::ApiError(
                response.description.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Answer callback query
    pub async fn answer_callback_query(
        &self,
        callback_query_id: &str,
        text: Option<&str>,
        show_alert: bool,
    ) -> Result<bool, PlatformError> {
        let mut params = serde_json::json!({
            "callback_query_id": callback_query_id,
            "show_alert": show_alert
        });

        if let Some(text) = text {
            params["text"] = serde_json::json!(text);
        }

        let response: TelegramApiResponse<bool> = self
            .client
            .post(&self.api_url("answerCallbackQuery"))
            .json(&params)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        Ok(response.ok)
    }

    /// Check if user is allowed
    fn is_user_allowed(&self, user_id: i64) -> bool {
        if let Some(ref allowed) = self.config.allowed_users {
            allowed.contains(&user_id)
        } else {
            true // All users allowed if not configured
        }
    }

    /// Check if chat is allowed
    fn is_chat_allowed(&self, chat_id: i64) -> bool {
        if let Some(ref allowed) = self.config.allowed_groups {
            allowed.contains(&chat_id)
        } else {
            true // All chats allowed if not configured
        }
    }

    /// Escape text for MarkdownV2
    fn escape_markdown(text: &str) -> String {
        let special_chars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
        let mut result = String::with_capacity(text.len() * 2);
        for c in text.chars() {
            if special_chars.contains(&c) {
                result.push('\\');
            }
            result.push(c);
        }
        result
    }

    /// Create inline keyboard from response actions
    fn create_keyboard(response: &TriggerResponse) -> Option<InlineKeyboardMarkup> {
        if response.actions.is_empty() {
            return None;
        }

        let buttons: Vec<InlineKeyboardButton> = response
            .actions
            .iter()
            .map(|action| InlineKeyboardButton {
                text: action.label.clone(),
                callback_data: Some(action.value.clone()),
                url: None,
            })
            .collect();

        // Arrange buttons in rows of 2
        let rows: Vec<Vec<InlineKeyboardButton>> = buttons
            .chunks(2)
            .map(|chunk| chunk.to_vec())
            .collect();

        Some(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }

    /// Format response text for Telegram
    fn format_response_text(response: &TriggerResponse) -> String {
        let status_emoji = match response.status {
            crate::response::ResponseStatus::Success => "✅",
            crate::response::ResponseStatus::Error => "❌",
            crate::response::ResponseStatus::Warning => "⚠️",
            crate::response::ResponseStatus::Info => "ℹ️",
        };

        let escaped_text = Self::escape_markdown(&response.text);
        format!("{} {}", status_emoji, escaped_text)
    }

    /// Generate help text
    pub fn create_help_text() -> String {
        r#"*AOF Bot Commands*

*Agent Commands:*
• `/run agent <name> <input>` \- Run an agent
• `/status task <id>` \- Check task status
• `/cancel task <id>` \- Cancel a task
• `/list tasks` \- List all tasks

*Examples:*
• `/run agent monitor Check server health`
• `/status task trigger\-user123\-abc`
• `/list tasks`

*Support:* [GitHub](https://github\.com/yourusername/aof)"#.to_string()
    }
}

#[async_trait]
impl TriggerPlatform for TelegramPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify secret token if configured
        if let Some(ref secret) = self.config.webhook_secret {
            if let Some(token) = headers.get("x-telegram-bot-api-secret-token") {
                if token != secret {
                    warn!("Invalid Telegram secret token");
                    return Err(PlatformError::InvalidSignature(
                        "Invalid secret token".to_string(),
                    ));
                }
            } else {
                warn!("Missing Telegram secret token header");
                return Err(PlatformError::InvalidSignature(
                    "Missing secret token".to_string(),
                ));
            }
        }

        let update: TelegramUpdate = serde_json::from_slice(raw).map_err(|e| {
            error!("Failed to parse Telegram update: {}", e);
            PlatformError::ParseError(format!("Invalid Telegram update: {}", e))
        })?;

        // Handle callback query (button clicks)
        if let Some(callback) = update.callback_query {
            let user = callback.from;

            // Check if user is allowed
            if !self.is_user_allowed(user.id) {
                warn!("User {} not allowed", user.id);
                return Err(PlatformError::InvalidSignature(
                    "User not allowed".to_string(),
                ));
            }

            let chat_id = callback
                .message
                .as_ref()
                .map(|m| m.chat.id)
                .unwrap_or(user.id);

            let trigger_user = TriggerUser {
                id: user.id.to_string(),
                username: user.username.clone(),
                display_name: Some(format!(
                    "{} {}",
                    user.first_name,
                    user.last_name.unwrap_or_default()
                ).trim().to_string()),
                is_bot: user.is_bot,
            };

            let mut metadata = HashMap::new();
            metadata.insert("callback_query_id".to_string(), serde_json::json!(callback.id));
            metadata.insert("update_id".to_string(), serde_json::json!(update.update_id));

            return Ok(TriggerMessage {
                id: callback.id.clone(),
                platform: "telegram".to_string(),
                channel_id: chat_id.to_string(),
                user: trigger_user,
                text: format!("callback:{}", callback.data.unwrap_or_default()),
                timestamp: chrono::Utc::now(),
                metadata,
                thread_id: None,
                reply_to: callback.message.as_ref().map(|m| m.message_id.to_string()),
            });
        }

        // Handle regular message
        if let Some(message) = update.message {
            let user = message.from.ok_or_else(|| {
                PlatformError::ParseError("Message has no sender".to_string())
            })?;

            // Check if user is allowed
            if !self.is_user_allowed(user.id) {
                warn!("User {} not allowed", user.id);
                return Err(PlatformError::InvalidSignature(
                    "User not allowed".to_string(),
                ));
            }

            // Check if chat is allowed
            if !self.is_chat_allowed(message.chat.id) {
                warn!("Chat {} not allowed", message.chat.id);
                return Err(PlatformError::InvalidSignature(
                    "Chat not allowed".to_string(),
                ));
            }

            let text = message.text.unwrap_or_default();

            let trigger_user = TriggerUser {
                id: user.id.to_string(),
                username: user.username.clone(),
                display_name: Some(format!(
                    "{} {}",
                    user.first_name,
                    user.last_name.unwrap_or_default()
                ).trim().to_string()),
                is_bot: user.is_bot,
            };

            let mut metadata = HashMap::new();
            metadata.insert("chat_type".to_string(), serde_json::json!(message.chat.chat_type));
            metadata.insert("update_id".to_string(), serde_json::json!(update.update_id));

            return Ok(TriggerMessage {
                id: message.message_id.to_string(),
                platform: "telegram".to_string(),
                channel_id: message.chat.id.to_string(),
                user: trigger_user,
                text,
                timestamp: chrono::Utc::now(),
                metadata,
                thread_id: None,
                reply_to: message.reply_to_message.as_ref().map(|m| m.message_id.to_string()),
            });
        }

        // Inline query (not fully supported yet)
        if update.inline_query.is_some() {
            return Err(PlatformError::UnsupportedMessageType);
        }

        Err(PlatformError::ParseError("No message in update".to_string()))
    }

    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        let chat_id: i64 = channel.parse().map_err(|_| {
            PlatformError::ParseError(format!("Invalid chat ID: {}", channel))
        })?;

        let text = Self::format_response_text(&response);
        let keyboard = Self::create_keyboard(&response);

        let reply_to = response
            .reply_to
            .as_ref()
            .and_then(|r| r.parse().ok());

        self.send_message(chat_id, &text, reply_to, keyboard).await?;

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "telegram"
    }

    async fn verify_signature(&self, _payload: &[u8], signature: &str) -> bool {
        // Telegram uses secret token header, not payload signature
        if let Some(ref secret) = self.config.webhook_secret {
            signature == secret
        } else {
            true // No secret configured, accept all
        }
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Telegram supports reply threads
    }

    fn supports_interactive(&self) -> bool {
        true // Telegram supports inline keyboards
    }

    fn supports_files(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> TelegramConfig {
        TelegramConfig {
            bot_token: "123456:ABC-DEF".to_string(),
            webhook_url: None,
            webhook_secret: Some("test-secret".to_string()),
            bot_name: "testbot".to_string(),
            allowed_users: None,
            allowed_groups: None,
        }
    }

    #[test]
    fn test_telegram_platform_new() {
        let config = create_test_config();
        let platform = TelegramPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_telegram_platform_invalid_config() {
        let config = TelegramConfig {
            bot_token: "".to_string(),
            webhook_url: None,
            webhook_secret: None,
            bot_name: "".to_string(),
            allowed_users: None,
            allowed_groups: None,
        };
        let platform = TelegramPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_user_allowed() {
        let mut config = create_test_config();
        config.allowed_users = Some(vec![123456, 789012]);

        let platform = TelegramPlatform::new(config).unwrap();

        assert!(platform.is_user_allowed(123456));
        assert!(platform.is_user_allowed(789012));
        assert!(!platform.is_user_allowed(999999));
    }

    #[test]
    fn test_chat_allowed() {
        let mut config = create_test_config();
        config.allowed_groups = Some(vec![-100123456789]);

        let platform = TelegramPlatform::new(config).unwrap();

        assert!(platform.is_chat_allowed(-100123456789));
        assert!(!platform.is_chat_allowed(-100999999999));
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(
            TelegramPlatform::escape_markdown("Hello *world*!"),
            "Hello \\*world\\*\\!"
        );
        assert_eq!(
            TelegramPlatform::escape_markdown("test_underscore"),
            "test\\_underscore"
        );
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = TelegramPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "telegram");
        assert_eq!(platform.bot_name(), "testbot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_help_text() {
        let help = TelegramPlatform::create_help_text();
        assert!(help.contains("AOF Bot Commands"));
        assert!(help.contains("/run agent"));
    }

    #[tokio::test]
    async fn test_parse_text_message() {
        let update_json = r#"{
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "from": {
                    "id": 12345,
                    "is_bot": false,
                    "first_name": "Test",
                    "username": "testuser"
                },
                "chat": {
                    "id": 12345,
                    "type": "private"
                },
                "date": 1234567890,
                "text": "/run agent test-agent hello world"
            }
        }"#;

        let mut config = create_test_config();
        config.webhook_secret = None; // Disable secret for test

        let platform = TelegramPlatform::new(config).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(update_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "telegram");
        assert_eq!(message.user.id, "12345");
        assert_eq!(message.text, "/run agent test-agent hello world");
    }
}
