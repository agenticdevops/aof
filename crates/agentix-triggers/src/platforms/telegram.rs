//! Telegram Bot API adapter for AOF
//!
//! This module provides integration with Telegram's Bot API, supporting:
//! - Text messages with /commands
//! - Inline keyboards for interactive responses
//! - Callback queries from button clicks
//! - Webhook secret token verification

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
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
            .post(self.api_url("setWebhook"))
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
            .post(self.api_url("deleteWebhook"))
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
            "parse_mode": "HTML"
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
            .post(self.api_url("sendMessage"))
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
            .post(self.api_url("answerCallbackQuery"))
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

    /// Escape text for HTML (only &, <, >)
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Convert standard markdown (from LLM output) to Telegram-compatible HTML.
    ///
    /// Handles: **bold**, *italic*, `inline code`, ```code blocks```,
    /// [links](url), and # headers → <b>bold</b>.
    fn markdown_to_html(text: &str) -> String {
        static RE_BOLD: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").unwrap());
        static RE_INLINE_CODE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"`([^`\n]+?)`").unwrap());
        static RE_LINK: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
        // Auto-link bare URLs (applied after markdown links are converted)
        static RE_BARE_URL: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"https?://[^\s<>")\]]+"#).unwrap());

        let mut result = String::with_capacity(text.len());
        let mut in_code_block = false;

        for line in text.lines() {
            let trimmed = line.trim();

            // Toggle code blocks
            if trimmed.starts_with("```") {
                if in_code_block {
                    result.push_str("</pre>\n");
                    in_code_block = false;
                } else {
                    result.push_str("<pre>");
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                result.push_str(&Self::html_escape(line));
                result.push('\n');
                continue;
            }

            // Strip header markers → bold
            let line = if trimmed.starts_with("### ") {
                let content = &trimmed[4..];
                let escaped = Self::html_escape(content);
                let converted = Self::convert_inline(&escaped, &RE_BOLD, &RE_INLINE_CODE, &RE_LINK, &RE_BARE_URL);
                result.push_str(&format!("<b>{}</b>\n", converted));
                continue;
            } else if trimmed.starts_with("## ") {
                let content = &trimmed[3..];
                let escaped = Self::html_escape(content);
                let converted = Self::convert_inline(&escaped, &RE_BOLD, &RE_INLINE_CODE, &RE_LINK, &RE_BARE_URL);
                result.push_str(&format!("<b>{}</b>\n", converted));
                continue;
            } else if trimmed.starts_with("# ") {
                let content = &trimmed[2..];
                let escaped = Self::html_escape(content);
                let converted = Self::convert_inline(&escaped, &RE_BOLD, &RE_INLINE_CODE, &RE_LINK, &RE_BARE_URL);
                result.push_str(&format!("<b>{}</b>\n", converted));
                continue;
            } else {
                line
            };

            // First escape HTML entities, then apply markdown→HTML conversions
            // We need to handle inline code and links BEFORE escaping because
            // they contain special chars. So: escape first, then convert markdown.
            let escaped = Self::html_escape(line);
            let converted = Self::convert_inline(&escaped, &RE_BOLD, &RE_INLINE_CODE, &RE_LINK, &RE_BARE_URL);
            result.push_str(&converted);
            result.push('\n');
        }

        if in_code_block {
            result.push_str("</pre>\n");
        }

        // Trim trailing newline
        while result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Convert inline markdown elements to HTML tags
    fn convert_inline(
        text: &str,
        re_bold: &Regex,
        re_code: &Regex,
        re_link: &Regex,
        re_bare_url: &Regex,
    ) -> String {
        // Order: code first (protect from other conversions), then bold, then italic, then links
        let result = re_code.replace_all(text, "\x00CODE_S\x00$1\x00CODE_E\x00");
        let result = re_bold.replace_all(&result, "<b>$1</b>");
        let result = Self::convert_italic(&result);
        // Convert [text](url) markdown links first
        let result = re_link.replace_all(&result, r#"<a href="$2">$1</a>"#);
        // Auto-link bare URLs that aren't already inside <a> tags
        let result = Self::auto_link_urls(&result, re_bare_url);
        // Restore code tags
        result
            .replace("\x00CODE_S\x00", "<code>")
            .replace("\x00CODE_E\x00", "</code>")
    }

    /// Auto-link bare URLs that aren't already inside <a href="..."> tags
    fn auto_link_urls(text: &str, re_url: &Regex) -> String {
        // Skip if no URLs at all
        if !text.contains("http") {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for mat in re_url.find_iter(text) {
            let start = mat.start();
            let url = mat.as_str();

            // Check if this URL is already inside an <a href="..."> tag
            let before = &text[..start];
            let in_href = before.ends_with("href=\"") || before.ends_with("href='");
            let in_a_tag = before.rfind("<a ").map_or(false, |a_pos| {
                // Check there's no </a> between the <a> and this position
                !before[a_pos..].contains("</a>")
            });

            result.push_str(&text[last_end..start]);

            if in_href || in_a_tag {
                // Already inside a link tag, don't wrap
                result.push_str(url);
            } else {
                result.push_str(&format!(r#"<a href="{}">{}</a>"#, url, url));
            }

            last_end = mat.end();
        }

        result.push_str(&text[last_end..]);
        result
    }

    /// Convert remaining single *text* to <i>text</i> after bold is already handled
    fn convert_italic(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
                // Found a single *, look for closing *
                if let Some(end) = chars[i + 1..].iter().position(|&c| c == '*') {
                    let end = i + 1 + end;
                    // Make sure closing * is also single
                    if end + 1 >= len || chars[end + 1] != '*' {
                        result.push_str("<i>");
                        for c in &chars[i + 1..end] {
                            result.push(*c);
                        }
                        result.push_str("</i>");
                        i = end + 1;
                        continue;
                    }
                }
            }
            result.push(chars[i]);
            i += 1;
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

    /// Format response text for Telegram (converts markdown → HTML)
    fn format_response_text(response: &TriggerResponse) -> String {
        Self::markdown_to_html(&response.text)
    }

    /// Generate help text
    pub fn create_help_text() -> String {
        r#"<b>Xops - Your Ops/SRE Agent</b>

Just type naturally. I'll handle it or call in a specialist.

<b>The Squad:</b>
• kubo - Kubernetes operator
• ergo - Monitoring &amp; observability
• ir-triage - Incident response
• sentinel - Safe change executor
• nux, doku, zure, rafo, zibl, wos

<b>Examples:</b>
• "show me pods in production"
• "why is the API slow?"
• "deploy v2.1 to staging"
• "check cluster health"

<b>Commands:</b>
• /help - This menu
• /run agent &lt;name&gt; &lt;task&gt; - Talk to a specific agent

<b>More:</b> <a href="https://github.com/agenticdevops/aof">GitHub</a>"#.to_string()
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
    fn test_html_escape() {
        assert_eq!(
            TelegramPlatform::html_escape("Hello <world> & \"test\""),
            "Hello &lt;world&gt; &amp; \"test\""
        );
    }

    #[test]
    fn test_markdown_to_html_bold() {
        let result = TelegramPlatform::markdown_to_html("**Status: Healthy**");
        assert_eq!(result, "<b>Status: Healthy</b>");
    }

    #[test]
    fn test_markdown_to_html_italic() {
        let result = TelegramPlatform::markdown_to_html("this is *important*");
        assert_eq!(result, "this is <i>important</i>");
    }

    #[test]
    fn test_markdown_to_html_code() {
        let result = TelegramPlatform::markdown_to_html("run `kubectl get pods`");
        assert_eq!(result, "run <code>kubectl get pods</code>");
    }

    #[test]
    fn test_markdown_to_html_header() {
        let result = TelegramPlatform::markdown_to_html("## Cluster Health");
        assert_eq!(result, "<b>Cluster Health</b>");
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let input = "text\n```\ncode here\n```\nmore text";
        let result = TelegramPlatform::markdown_to_html(input);
        assert!(result.contains("<pre>"));
        assert!(result.contains("code here"));
        assert!(result.contains("</pre>"));
    }

    #[test]
    fn test_markdown_to_html_mixed() {
        let input = "**Components:**\n- ✅ API: healthy\n- ❌ DB: *down*";
        let result = TelegramPlatform::markdown_to_html(input);
        assert!(result.contains("<b>Components:</b>"));
        assert!(result.contains("<i>down</i>"));
        assert!(result.contains("✅"));
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
        assert!(help.contains("Xops"));
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
