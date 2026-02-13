//! Discord adapter using Gateway
//!
//! This adapter implements the ChannelAdapter trait for Discord using the Gateway (outbound WebSocket).
//! The Gateway connection eliminates the need for a public endpoint, making it NAT-transparent.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use aof_core::AofError;
use crate::adapters::{ChannelAdapter, Platform, InboundMessage, AgentResponse, MessageUser};
use crate::rate_limiter::RateLimiter;

/// Discord platform adapter (Gateway)
pub struct DiscordAdapter {
    adapter_id: String,
    config: DiscordConfig,
    rate_limiter: RateLimiter,
    message_rx: Option<mpsc::Receiver<InboundMessage>>,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Discord adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Bot token
    pub bot_token: String,
    /// Application ID
    pub application_id: String,
    /// Guild whitelist (empty = all guilds)
    #[serde(default)]
    pub guild_ids: Vec<String>,
    /// Allowed role IDs for role-based access
    #[serde(default)]
    pub allowed_roles: Vec<String>,
}

impl DiscordAdapter {
    /// Create new Discord adapter
    pub fn new(adapter_id: String, config: DiscordConfig) -> Self {
        let rate_limit_config = crate::rate_limiter::RateLimiter::default_config_for_platform(Platform::Discord);
        let rate_limiter = RateLimiter::new(Platform::Discord, rate_limit_config);

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
        // Use HTTP client to get current user (bot)
        let client = reqwest::Client::new();
        let response = client
            .get("https://discord.com/api/v10/users/@me")
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Failed to validate Discord token: {}", e)))?;

        if !response.status().is_success() {
            let token_prefix = self.config.bot_token.chars().take(8).collect::<String>();
            error!(
                adapter_id = %self.adapter_id,
                token_prefix = %token_prefix,
                "Invalid Discord bot token"
            );
            return Err(AofError::runtime("Invalid Discord bot token"));
        }

        info!(adapter_id = %self.adapter_id, "Discord bot token validated");
        Ok(())
    }

    /// Translate markdown to Discord embed JSON
    fn markdown_to_discord_embed(markdown: &str, max_len: usize) -> serde_json::Value {
        // Split content if too long (Discord embed description limit: 4096 chars)
        let content = if markdown.len() > max_len {
            &markdown[..max_len]
        } else {
            markdown
        };

        serde_json::json!({
            "description": content,
            "color": 0x5865F2, // Discord blurple
        })
    }

    /// Split long responses into chunks
    fn split_long_response(content: &str, max_len: usize) -> Vec<String> {
        if content.len() <= max_len {
            return vec![content.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for line in content.lines() {
            if current_chunk.len() + line.len() + 1 > max_len {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

#[async_trait]
impl ChannelAdapter for DiscordAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn platform(&self) -> Platform {
        Platform::Discord
    }

    async fn start(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Starting Discord adapter (Gateway)");

        // Validate token first
        self.validate_token().await?;

        // Create message channel
        let (_message_tx, message_rx) = mpsc::channel(100);
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();

        // TODO: Initialize Discord Gateway WebSocket connection
        // This requires serenity client setup with event handlers
        // For now, just set up the infrastructure

        // Spawn background task to handle Gateway events
        let adapter_id = self.adapter_id.clone();

        tokio::spawn(async move {
            debug!(adapter_id = %adapter_id, "Discord Gateway listener started");

            // TODO: Connect to Discord Gateway WebSocket
            // This requires implementing serenity EventHandler
            // For now, just wait for stop signal

            tokio::select! {
                _ = stop_rx => {
                    debug!(adapter_id = %adapter_id, "Discord Gateway listener stopped");
                }
            }
        });

        self.message_rx = Some(message_rx);
        self.stop_tx = Some(stop_tx);

        info!(adapter_id = %self.adapter_id, "Discord adapter started");
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
            "Sending Discord message"
        );

        // Split long responses if needed (5,500 char limit with buffer)
        let chunks = Self::split_long_response(&response.content, 5500);

        for (idx, chunk) in chunks.iter().enumerate() {
            // Translate markdown to Discord embed
            let embed = Self::markdown_to_discord_embed(chunk, 4096);

            // Build request payload
            let payload = serde_json::json!({
                "embeds": [embed],
            });

            // Send via Discord API
            let client = reqwest::Client::new();
            let url = format!("https://discord.com/api/v10/channels/{}/messages", response.target_channel);

            let res = client
                .post(&url)
                .header("Authorization", format!("Bot {}", self.config.bot_token))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| AofError::runtime(format!("Discord API error: {}", e)))?;

            if !res.status().is_success() {
                let error_text = res.text().await.unwrap_or_default();
                error!(
                    adapter_id = %self.adapter_id,
                    error = %error_text,
                    "Failed to send Discord message"
                );
                return Err(AofError::runtime(format!("Discord API error: {}", error_text)));
            }

            debug!(
                adapter_id = %self.adapter_id,
                channel = %response.target_channel,
                chunk = idx + 1,
                total = chunks.len(),
                "Discord message sent successfully"
            );

            // Add small delay between chunks to avoid rate limits
            if idx < chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), AofError> {
        info!(adapter_id = %self.adapter_id, "Stopping Discord adapter");

        if let Some(stop_tx) = self.stop_tx.take() {
            stop_tx.send(()).ok();
        }

        self.message_rx = None;

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, AofError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://discord.com/api/v10/users/@me")
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| AofError::runtime(format!("Discord health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_config_serialization() {
        let config = DiscordConfig {
            bot_token: "test-token".to_string(),
            application_id: "12345".to_string(),
            guild_ids: vec!["67890".to_string()],
            allowed_roles: vec!["role1".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-token"));
    }

    #[test]
    fn test_markdown_to_discord_embed() {
        let markdown = "# Hello\n\nWorld";
        let embed = DiscordAdapter::markdown_to_discord_embed(markdown, 4096);
        assert!(embed["description"].is_string());
        assert_eq!(embed["color"], 0x5865F2);
    }

    #[test]
    fn test_split_long_response() {
        let short_text = "Short message";
        let chunks = DiscordAdapter::split_long_response(short_text, 5500);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short message\n");

        // Test with long text
        let long_text = "Line\n".repeat(1000); // ~5000 chars
        let chunks = DiscordAdapter::split_long_response(&long_text, 5500);
        assert_eq!(chunks.len(), 1); // Should fit in one chunk

        // Test with very long text
        let very_long_text = "Line\n".repeat(2000); // ~10000 chars
        let chunks = DiscordAdapter::split_long_response(&very_long_text, 5500);
        assert!(chunks.len() >= 2); // Should split into multiple chunks
    }
}
