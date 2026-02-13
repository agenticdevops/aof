//! Telegram adapter using long polling
//!
//! This adapter implements the ChannelAdapter trait for Telegram using long polling (outbound HTTP).
//! Long polling eliminates the need for a public endpoint, making the connection NAT-transparent.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use aof_core::AofError;
use crate::adapters::{ChannelAdapter, Platform, InboundMessage, AgentResponse};
use crate::rate_limiter::RateLimiter;

/// Telegram platform adapter (long polling)
pub struct TelegramAdapter {
    adapter_id: String,
    config: TelegramConfig,
    rate_limiter: RateLimiter,
    message_rx: Option<mpsc::Receiver<InboundMessage>>,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Telegram adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot token
    pub bot_token: String,
    /// Chat whitelist (empty = all chats)
    #[serde(default)]
    pub allowed_chats: Vec<i64>,
}

impl TelegramAdapter {
    /// Create new Telegram adapter
    pub fn new(adapter_id: String, config: TelegramConfig) -> Self {
        let rate_limit_config = crate::rate_limiter::RateLimiter::default_config_for_platform(Platform::Telegram);
        let rate_limiter = RateLimiter::new(Platform::Telegram, rate_limit_config);

        Self {
            adapter_id,
            config,
            rate_limiter,
            message_rx: None,
            stop_tx: None,
        }
    }

    /// Validate bot token
    async fn validate_token(&self) -> Result<(), AofError> {
        // Use HTTP client to get bot info
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/getMe", self.config.bot_token);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Failed to validate Telegram token: {}", e)))?;

        if !response.status().is_success() {
            let token_prefix = self.config.bot_token.chars().take(8).collect::<String>();
            error!(
                adapter_id = %self.adapter_id,
                token_prefix = %token_prefix,
                "Invalid Telegram bot token"
            );
            return Err(AofError::runtime("Invalid Telegram bot token"));
        }

        // Parse response to check if bot is active
        let json: serde_json::Value = response.json().await
            .map_err(|e| AofError::runtime(format!("Failed to parse getMe response: {}", e)))?;

        if !json["ok"].as_bool().unwrap_or(false) {
            return Err(AofError::runtime("Telegram bot is not active"));
        }

        info!(adapter_id = %self.adapter_id, "Telegram bot token validated");
        Ok(())
    }

    /// Escape markdown for Telegram MarkdownV2
    fn escape_telegram_markdown(text: &str) -> String {
        // Telegram MarkdownV2 requires escaping these special chars:
        // _ * [ ] ( ) ~ ` > # + - = | { } . !
        let special_chars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];

        let mut escaped = String::with_capacity(text.len() * 2);
        for ch in text.chars() {
            if special_chars.contains(&ch) {
                escaped.push('\\');
            }
            escaped.push(ch);
        }
        escaped
    }
}

#[async_trait]
impl ChannelAdapter for TelegramAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn platform(&self) -> Platform {
        Platform::Telegram
    }

    async fn start(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Starting Telegram adapter (long polling)");

        // Validate token first
        self.validate_token().await?;

        // Create message channel
        let (_message_tx, message_rx) = mpsc::channel(100);
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();

        // TODO: Initialize long polling loop
        // This requires implementing getUpdates polling
        // For now, just set up the infrastructure

        // Spawn background task to handle long polling
        let adapter_id = self.adapter_id.clone();
        let bot_token = self.config.bot_token.clone();

        tokio::spawn(async move {
            debug!(adapter_id = %adapter_id, "Telegram long polling started");

            // TODO: Implement long polling loop
            // while let Ok(updates) = get_updates(&bot_token, offset).await {
            //     for update in updates {
            //         // Normalize and send via message_tx
            //     }
            // }

            tokio::select! {
                _ = stop_rx => {
                    debug!(adapter_id = %adapter_id, "Telegram long polling stopped");
                }
            }
        });

        self.message_rx = Some(message_rx);
        self.stop_tx = Some(stop_tx);

        info!(adapter_id = %self.adapter_id, "Telegram adapter started");
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<InboundMessage, AofError> {
        self.message_rx
            .as_mut()
            .ok_or_else(|| AofError::runtime("Adapter not started"))?
            .recv()
            .await
            .ok_or_else(|| AofError::runtime("Message channel closed"))
    }

    async fn send_message(&self, response: &AgentResponse) -> Result<(), AofError> {
        // Apply rate limiting
        self.rate_limiter.acquire().await?;

        debug!(
            adapter_id = %self.adapter_id,
            agent_id = %response.agent_id,
            channel = %response.target_channel,
            "Sending Telegram message"
        );

        // Escape markdown for Telegram MarkdownV2
        let escaped_content = Self::escape_telegram_markdown(&response.content);

        // Build request payload
        let mut payload = serde_json::json!({
            "chat_id": response.target_channel,
            "text": escaped_content,
            "parse_mode": "MarkdownV2",
        });

        if let Some(reply_to) = &response.thread_id {
            if let Ok(message_id) = reply_to.parse::<i64>() {
                payload["reply_to_message_id"] = serde_json::Value::Number(message_id.into());
            }
        }

        // Send via Telegram API
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.config.bot_token);

        let res = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Telegram API error: {}", e)))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            error!(
                adapter_id = %self.adapter_id,
                error = %error_text,
                "Failed to send Telegram message"
            );
            return Err(AofError::runtime(format!("Telegram API error: {}", error_text)));
        }

        debug!(
            adapter_id = %self.adapter_id,
            channel = %response.target_channel,
            "Telegram message sent successfully"
        );

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Stopping Telegram adapter");

        if let Some(stop_tx) = self.stop_tx.take() {
            stop_tx.send(()).ok();
        }

        self.message_rx = None;

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/getMe", self.config.bot_token);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Telegram health check failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        // Parse response
        let json: serde_json::Value = response.json().await
            .map_err(|e| AofError::runtime(format!("Failed to parse health check response: {}", e)))?;

        Ok(json["ok"].as_bool().unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_config_serialization() {
        let config = TelegramConfig {
            bot_token: "test-token".to_string(),
            allowed_chats: vec![12345, 67890],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-token"));
    }

    #[test]
    fn test_escape_telegram_markdown() {
        let text = "Hello_world*bold*[link](url)";
        let escaped = TelegramAdapter::escape_telegram_markdown(text);
        assert_eq!(escaped, "Hello\\_world\\*bold\\*\\[link\\]\\(url\\)");

        // Test with no special chars
        let text = "Normal text";
        let escaped = TelegramAdapter::escape_telegram_markdown(text);
        assert_eq!(escaped, "Normal text");

        // Test with all special chars
        let text = "_*[]()~`>#+-=|{}.!";
        let escaped = TelegramAdapter::escape_telegram_markdown(text);
        assert_eq!(escaped, "\\_\\*\\[\\]\\(\\)\\~\\`\\>\\#\\+\\-\\=\\|\\{\\}\\.\\!");
    }
}
