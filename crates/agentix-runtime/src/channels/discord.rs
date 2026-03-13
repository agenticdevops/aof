//! Discord channel gateway adapter (Phase 21)
//!
//! Bi-directional communication with Discord channels via gateway events
//! and REST API messages. Notifications use Discord embed formatting.

use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelMessage, ChannelPlatformType, NotificationPayload,
    NotificationSeverity, NotificationTarget,
};
use agentix_core::AgentixError;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, error};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord channel gateway — implements ChannelGateway for Discord REST API
pub struct DiscordChannelGateway {
    bot_token: String,
    application_id: String,
    public_key: String,
    client: reqwest::Client,
}

impl DiscordChannelGateway {
    /// Create a new DiscordChannelGateway from Discord credentials
    pub fn new(credentials: &ChannelCredentials) -> Result<Self, AgentixError> {
        match credentials {
            ChannelCredentials::Discord {
                bot_token,
                application_id,
                public_key,
            } => Ok(Self {
                bot_token: bot_token.clone(),
                application_id: application_id.clone(),
                public_key: public_key.clone(),
                client: reqwest::Client::new(),
            }),
            _ => Err(AgentixError::Config(
                "DiscordChannelGateway requires Discord credentials".into(),
            )),
        }
    }

    /// Returns the application ID for bot mention detection
    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    /// Parse a Discord webhook/gateway event payload into a ChannelMessage
    ///
    /// Returns None for PING interactions, messages without bot mention,
    /// and messages from bot authors.
    /// Returns Some(ChannelMessage) for MESSAGE_CREATE events with bot mention.
    pub fn parse_webhook_payload(
        &self,
        payload: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<Option<ChannelMessage>, AgentixError> {
        let body: Value = serde_json::from_slice(payload)
            .map_err(|e| AgentixError::Config(format!("Invalid Discord webhook payload: {}", e)))?;

        // Check for PING interaction (type 1)
        if body.get("type").and_then(|v| v.as_i64()) == Some(1) {
            debug!("Discord PING interaction — returning None");
            return Ok(None);
        }

        // Check for gateway event (MESSAGE_CREATE)
        let event_type = body.get("t").and_then(|v| v.as_str()).unwrap_or("");
        if event_type != "MESSAGE_CREATE" {
            debug!("Discord event type '{}' — not MESSAGE_CREATE, skipping", event_type);
            return Ok(None);
        }

        let data = match body.get("d") {
            Some(d) => d,
            None => return Ok(None),
        };

        // Ignore bot's own messages
        let author = data.get("author");
        if author
            .and_then(|a| a.get("bot"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            debug!("Discord message from bot — skipping");
            return Ok(None);
        }

        let content = data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Check for bot mention: <@APPLICATION_ID>
        let mention = format!("<@{}>", self.application_id);
        if !content.contains(&mention) {
            debug!("Discord message without bot mention — skipping");
            return Ok(None);
        }

        let channel_id = data
            .get("channel_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let user_id = author
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let user_name = author
            .and_then(|a| a.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let text = content.to_string();

        // Thread ID from message_reference.message_id
        let thread_id = data
            .get("message_reference")
            .and_then(|r| r.get("message_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let timestamp_str = data
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Some(ChannelMessage {
            platform: ChannelPlatformType::Discord,
            channel_id,
            user_id,
            user_name,
            text,
            thread_id,
            timestamp,
            raw_payload: body.clone(),
        }))
    }

    /// Verify a Discord webhook signature using Ed25519
    ///
    /// NOTE: Full Ed25519 verification requires the ed25519-dalek crate.
    /// This implementation validates the signature format and delegates
    /// to the crypto library for actual verification.
    pub fn verify_signature(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: &str,
    ) -> bool {
        use ed25519_dalek::{Signature, VerifyingKey};

        let public_key_bytes = match hex::decode(&self.public_key) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let pk_array: [u8; 32] = match public_key_bytes.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };

        let verifying_key = match VerifyingKey::from_bytes(&pk_array) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let sig_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let sig_array: [u8; 64] = match sig_bytes.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };

        let sig = Signature::from_bytes(&sig_array);

        let message = format!("{}{}", timestamp, String::from_utf8_lossy(payload));

        use ed25519_dalek::Verifier;
        verifying_key.verify(message.as_bytes(), &sig).is_ok()
    }

    /// Format a NotificationPayload as a Discord embed
    pub fn format_notification_embed(payload: &NotificationPayload) -> Value {
        let color = match payload.severity {
            NotificationSeverity::Info => 0x36a64f,
            NotificationSeverity::Warning => 0xffc107,
            NotificationSeverity::Error => 0xdc3545,
            NotificationSeverity::Critical => 0xff0000,
        };

        let mut footer_text = format!("agent: {}", payload.agent_name);
        if let Some(ref run_id) = payload.run_id {
            footer_text.push_str(&format!(" | run: {}", run_id));
        }

        json!({
            "embeds": [{
                "title": payload.title,
                "description": payload.body,
                "color": color,
                "footer": {
                    "text": footer_text
                },
                "timestamp": Utc::now().to_rfc3339()
            }]
        })
    }
}

#[async_trait]
impl ChannelGateway for DiscordChannelGateway {
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
            "content": text,
        });

        if let Some(ref_id) = thread_id {
            body["message_reference"] = json!({
                "message_id": ref_id
            });
        }

        let url = format!("{}/channels/{}/messages", DISCORD_API_BASE, channel_id);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentixError::Agent(format!("Discord API error: {}", e)))?;

        if !resp.status().is_success() {
            error!("Discord send message failed: {}", resp.status());
            return Err(AgentixError::Agent(format!(
                "Discord API returned status {}",
                resp.status()
            )));
        }

        debug!("Discord message sent to channel {}", channel_id);
        Ok(())
    }

    async fn send_notification(
        &self,
        target: &NotificationTarget,
        payload: &NotificationPayload,
    ) -> Result<(), AgentixError> {
        let embed = Self::format_notification_embed(payload);

        let mut body = json!({
            "content": format!("{}: {}", payload.title, payload.body),
        });

        if let Some(embeds) = embed.get("embeds") {
            body["embeds"] = embeds.clone();
        }

        if let Some(ref ts) = target.thread_id {
            body["message_reference"] = json!({
                "message_id": ts
            });
        }

        let url = format!("{}/channels/{}/messages", DISCORD_API_BASE, target.channel_id);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentixError::Agent(format!("Discord notification error: {}", e)))?;

        if !resp.status().is_success() {
            error!("Discord notification failed: {}", resp.status());
            return Err(AgentixError::Agent(format!(
                "Discord notification API returned status {}",
                resp.status()
            )));
        }

        debug!("Discord notification sent to channel {}", target.channel_id);
        Ok(())
    }

    fn platform_type(&self) -> ChannelPlatformType {
        ChannelPlatformType::Discord
    }
}
