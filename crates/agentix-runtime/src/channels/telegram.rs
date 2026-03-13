//! Telegram channel gateway adapter (Phase 21)
//!
//! Bi-directional communication with Telegram chats via Bot API webhooks
//! and sendMessage responses. Notifications use Markdown formatting.

use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelMessage, ChannelPlatformType, NotificationPayload,
    NotificationSeverity, NotificationTarget,
};
use agentix_core::AgentixError;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, error};

/// Telegram channel gateway — implements ChannelGateway for Telegram Bot API
pub struct TelegramChannelGateway {
    bot_token: String,
    webhook_secret: Option<String>,
    api_base: String,
    client: reqwest::Client,
}

impl TelegramChannelGateway {
    /// Create a new TelegramChannelGateway from Telegram credentials
    pub fn new(credentials: &ChannelCredentials) -> Result<Self, AgentixError> {
        match credentials {
            ChannelCredentials::Telegram {
                bot_token,
                webhook_secret,
            } => {
                let api_base = format!("https://api.telegram.org/bot{}/", bot_token);
                Ok(Self {
                    bot_token: bot_token.clone(),
                    webhook_secret: webhook_secret.clone(),
                    api_base,
                    client: reqwest::Client::new(),
                })
            }
            _ => Err(AgentixError::Config(
                "TelegramChannelGateway requires Telegram credentials".into(),
            )),
        }
    }

    /// Returns the Telegram Bot API base URL
    pub fn api_base_url(&self) -> &str {
        &self.api_base
    }

    /// Parse a Telegram Update webhook payload into a ChannelMessage
    ///
    /// Returns None for updates without a message or without text.
    /// Returns Some(ChannelMessage) for updates with message.text present.
    pub fn parse_webhook_payload(
        &self,
        payload: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<Option<ChannelMessage>, AgentixError> {
        let body: Value = serde_json::from_slice(payload)
            .map_err(|e| AgentixError::Config(format!("Invalid Telegram webhook payload: {}", e)))?;

        let message = match body.get("message") {
            Some(m) => m,
            None => {
                debug!("Telegram update has no message field — skipping");
                return Ok(None);
            }
        };

        let text = match message.get("text").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => {
                debug!("Telegram message has no text — skipping");
                return Ok(None);
            }
        };

        let chat_id = message
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();

        let from = message.get("from");
        let user_id = from
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();

        let user_name = from
            .and_then(|f| f.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let date = message
            .get("date")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let timestamp = Utc.timestamp_opt(date, 0).single().unwrap_or_else(|| Utc::now());

        // Thread ID from reply_to_message.message_id
        let thread_id = message
            .get("reply_to_message")
            .and_then(|r| r.get("message_id"))
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string());

        Ok(Some(ChannelMessage {
            platform: ChannelPlatformType::Telegram,
            channel_id: chat_id,
            user_id,
            user_name,
            text,
            thread_id,
            timestamp,
            raw_payload: body.clone(),
        }))
    }

    /// Verify the Telegram webhook secret token
    ///
    /// If no webhook_secret is configured, always returns true (no verification).
    /// Otherwise checks the X-Telegram-Bot-Api-Secret-Token header.
    pub fn verify_webhook_secret(&self, headers: &HashMap<String, String>) -> bool {
        match &self.webhook_secret {
            None => true, // No verification configured
            Some(expected) => {
                let header_value = headers
                    .get("x-telegram-bot-api-secret-token")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                header_value == expected
            }
        }
    }

    /// Format a NotificationPayload as a Telegram Markdown message
    pub fn format_notification_text(payload: &NotificationPayload) -> String {
        let emoji = match payload.severity {
            NotificationSeverity::Info => "\u{2139}\u{fe0f}",     // info emoji
            NotificationSeverity::Warning => "\u{26a0}\u{fe0f}",  // warning emoji
            NotificationSeverity::Error => "\u{274c}",             // error emoji
            NotificationSeverity::Critical => "\u{1f6a8}",         // critical emoji
        };

        let mut text = format!("{} *{}*\n{}\n_{}_", emoji, payload.title, payload.body, payload.agent_name);

        if let Some(ref run_id) = payload.run_id {
            text.push_str(&format!(" _{}_", run_id));
        }

        text
    }
}

#[async_trait]
impl ChannelGateway for TelegramChannelGateway {
    fn parse_webhook(
        &self,
        payload: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<Option<ChannelMessage>, AgentixError> {
        self.parse_webhook_payload(payload, headers)
    }

    async fn send_message(
        &self,
        channel_id: &str,
        text: &str,
        thread_id: Option<&str>,
    ) -> Result<(), AgentixError> {
        let mut body = json!({
            "chat_id": channel_id,
            "text": text,
            "parse_mode": "MarkdownV2",
        });

        if let Some(reply_id) = thread_id {
            if let Ok(msg_id) = reply_id.parse::<i64>() {
                body["reply_to_message_id"] = json!(msg_id);
            }
        }

        let url = format!("{}sendMessage", self.api_base);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentixError::Agent(format!("Telegram API error: {}", e)))?;

        if !resp.status().is_success() {
            error!("Telegram sendMessage failed: {}", resp.status());
            return Err(AgentixError::Agent(format!(
                "Telegram API returned status {}",
                resp.status()
            )));
        }

        debug!("Telegram message sent to chat {}", channel_id);
        Ok(())
    }

    async fn send_notification(
        &self,
        target: &NotificationTarget,
        payload: &NotificationPayload,
    ) -> Result<(), AgentixError> {
        let text = Self::format_notification_text(payload);
        self.send_message(
            &target.channel_id,
            &text,
            target.thread_id.as_deref(),
        )
        .await
    }

    fn platform_type(&self) -> ChannelPlatformType {
        ChannelPlatformType::Telegram
    }
}
