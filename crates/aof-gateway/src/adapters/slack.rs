//! Slack adapter using Socket Mode
//!
//! This adapter implements the ChannelAdapter trait for Slack using Socket Mode (outbound WebSocket).
//! Socket Mode eliminates the need for a public endpoint, making the connection NAT-transparent.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use aof_core::AofError;
use crate::adapters::{ChannelAdapter, Platform, InboundMessage, AgentResponse};
use crate::rate_limiter::RateLimiter;

/// Slack platform adapter (Socket Mode)
pub struct SlackAdapter {
    adapter_id: String,
    config: SlackConfig,
    rate_limiter: RateLimiter,
    message_rx: Option<mpsc::Receiver<InboundMessage>>,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Slack adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Bot token (xoxb-...)
    pub bot_token: String,
    /// App-level token for Socket Mode (xapp-...)
    pub app_token: String,
    /// Bot user ID (for filtering own messages)
    pub bot_user_id: String,
    /// Channel whitelist (empty = all channels)
    #[serde(default)]
    pub allowed_channels: Vec<String>,
}

impl SlackAdapter {
    /// Create new Slack adapter
    pub fn new(adapter_id: String, config: SlackConfig) -> Self {
        let rate_limit_config = crate::rate_limiter::RateLimiter::default_config_for_platform(Platform::Slack);
        let rate_limiter = RateLimiter::new(Platform::Slack, rate_limit_config);

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
        // Use HTTP client to validate token
        let client = reqwest::Client::new();
        let response = client
            .post("https://slack.com/api/auth.test")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Failed to validate Slack token: {}", e)))?;

        if !response.status().is_success() {
            let token_prefix = self.config.bot_token.chars().take(8).collect::<String>();
            error!(
                adapter_id = %self.adapter_id,
                token_prefix = %token_prefix,
                "Invalid Slack bot token"
            );
            return Err(AofError::runtime("Invalid Slack bot token"));
        }

        info!(adapter_id = %self.adapter_id, "Slack bot token validated");
        Ok(())
    }

    /// Translate markdown to Slack Block Kit JSON
    fn markdown_to_slack_blocks(markdown: &str) -> serde_json::Value {
        // Simple markdown → Block Kit translation
        // Create a section block with markdown text
        serde_json::json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": markdown
                }
            }
        ])
    }

    /// Check if Slack timestamp is stale (>5 min old)
    fn is_timestamp_stale(ts_str: &str) -> bool {
        if let Ok(ts_float) = ts_str.parse::<f64>() {
            let now = Utc::now().timestamp() as f64;
            let age_seconds = now - ts_float;
            age_seconds > 300.0 // 5 minutes
        } else {
            false
        }
    }
}

#[async_trait]
impl ChannelAdapter for SlackAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn platform(&self) -> Platform {
        Platform::Slack
    }

    async fn start(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Starting Slack adapter (Socket Mode)");

        // Validate token first
        self.validate_token().await?;

        // Create message channel
        let (_message_tx, message_rx) = mpsc::channel(100);
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();

        // TODO: Initialize Socket Mode WebSocket connection
        // For now, just set up the infrastructure

        // Spawn background task to handle Socket Mode events
        let adapter_id = self.adapter_id.clone();
        let _app_token = self.config.app_token.clone();
        let _bot_user_id = self.config.bot_user_id.clone();

        tokio::spawn(async move {
            debug!(adapter_id = %adapter_id, "Socket Mode listener started");

            // TODO: Connect to Slack Socket Mode WebSocket
            // This requires implementing the full Socket Mode protocol
            // For now, just wait for stop signal

            tokio::select! {
                _ = stop_rx => {
                    debug!(adapter_id = %adapter_id, "Socket Mode listener stopped");
                }
            }
        });

        self.message_rx = Some(message_rx);
        self.stop_tx = Some(stop_tx);

        info!(adapter_id = %self.adapter_id, "Slack adapter started");
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
            "Sending Slack message"
        );

        // Translate markdown to Slack Block Kit
        let blocks = Self::markdown_to_slack_blocks(&response.content);

        // Build request payload
        let mut payload = serde_json::json!({
            "channel": response.target_channel,
            "blocks": blocks,
        });

        if let Some(thread_ts) = &response.thread_id {
            payload["thread_ts"] = serde_json::Value::String(thread_ts.clone());
        }

        // Send via Slack API
        let client = reqwest::Client::new();
        let res = client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Slack API error: {}", e)))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            error!(
                adapter_id = %self.adapter_id,
                error = %error_text,
                "Failed to send Slack message"
            );
            return Err(AofError::runtime(format!("Slack API error: {}", error_text)));
        }

        debug!(
            adapter_id = %self.adapter_id,
            channel = %response.target_channel,
            "Slack message sent successfully"
        );

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Stopping Slack adapter");

        if let Some(stop_tx) = self.stop_tx.take() {
            stop_tx.send(()).ok();
        }

        self.message_rx = None;

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://slack.com/api/auth.test")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Slack health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_config_serialization() {
        let config = SlackConfig {
            bot_token: "xoxb-test".to_string(),
            app_token: "xapp-test".to_string(),
            bot_user_id: "U123".to_string(),
            allowed_channels: vec!["C123".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("xoxb-test"));
    }

    #[test]
    fn test_is_timestamp_stale() {
        // Create recent timestamp (now)
        let now = Utc::now().timestamp();
        let recent_ts = format!("{}.000000", now);
        assert!(!SlackAdapter::is_timestamp_stale(&recent_ts));

        // Create stale timestamp (10 minutes ago)
        let old_ts = format!("{}.000000", now - 600);
        assert!(SlackAdapter::is_timestamp_stale(&old_ts));
    }

    #[test]
    fn test_markdown_to_slack_blocks() {
        let markdown = "# Hello\n\nWorld";
        let blocks = SlackAdapter::markdown_to_slack_blocks(markdown);
        assert!(blocks.is_array());
        assert_eq!(blocks.as_array().unwrap().len(), 1);
    }
}
