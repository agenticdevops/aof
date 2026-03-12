// ChannelMentionTrigger — passive trigger that fires when an agent is @mentioned
// in a Slack channel, Discord channel, or Telegram chat.
//
// Design note: This module implements its own lightweight JSON field extraction
// rather than wrapping the complex v1.0-era SlackPlatform / DiscordPlatform /
// TelegramPlatform handlers (which use the TriggerPlatform async trait and carry
// heavy state). The TriggerTrait required here is the runtime trigger interface
// from agentix-core, which is distinct from the v1.0 platform adapters.
//
// Slack:   Receives Events API callbacks; fires on event.type == "app_mention"
// Discord: Receives Gateway MESSAGE_CREATE dispatches via HTTP; fires when content contains "<@"
// Telegram: Receives webhook updates; fires on any message.text (all messages treated as triggers)

use std::collections::HashMap;

use agentix_core::{AgentixError, TriggerEvent, TriggerSource, TriggerTrait};
use async_trait::async_trait;
use chrono::Utc;

/// The messaging platform this trigger listens on.
#[derive(Debug, Clone)]
pub enum ChannelPlatform {
    /// Slack Events API — fires on app_mention events.
    Slack {
        bot_token: String,
        signing_secret: String,
    },
    /// Discord Gateway via HTTP — fires on MESSAGE_CREATE events with bot mentions.
    Discord {
        bot_token: String,
        application_id: String,
    },
    /// Telegram webhook — fires on any incoming message.
    Telegram { bot_token: String },
}

/// A unified channel mention trigger supporting Slack, Discord, and Telegram.
pub struct ChannelMentionTrigger {
    id: String,
    agent_name: String,
    platform: ChannelPlatform,
    /// Sender stored at start() time for use by async dispatch paths.
    sender: tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<(String, TriggerEvent)>>>,
}

impl ChannelMentionTrigger {
    /// Create a Slack mention trigger.
    pub fn for_slack(
        agent_name: impl Into<String>,
        trigger_id: impl Into<String>,
        bot_token: impl Into<String>,
        signing_secret: impl Into<String>,
    ) -> Self {
        Self {
            id: trigger_id.into(),
            agent_name: agent_name.into(),
            platform: ChannelPlatform::Slack {
                bot_token: bot_token.into(),
                signing_secret: signing_secret.into(),
            },
            sender: tokio::sync::Mutex::new(None),
        }
    }

    /// Create a Discord mention trigger.
    pub fn for_discord(
        agent_name: impl Into<String>,
        trigger_id: impl Into<String>,
        bot_token: impl Into<String>,
        application_id: impl Into<String>,
    ) -> Self {
        Self {
            id: trigger_id.into(),
            agent_name: agent_name.into(),
            platform: ChannelPlatform::Discord {
                bot_token: bot_token.into(),
                application_id: application_id.into(),
            },
            sender: tokio::sync::Mutex::new(None),
        }
    }

    /// Create a Telegram mention trigger.
    pub fn for_telegram(
        agent_name: impl Into<String>,
        trigger_id: impl Into<String>,
        bot_token: impl Into<String>,
    ) -> Self {
        Self {
            id: trigger_id.into(),
            agent_name: agent_name.into(),
            platform: ChannelPlatform::Telegram {
                bot_token: bot_token.into(),
            },
            sender: tokio::sync::Mutex::new(None),
        }
    }

    /// Returns the trigger_id string (for tests and external access).
    pub fn trigger_id_str(&self) -> &str {
        &self.id
    }

    /// Process an inbound HTTP webhook payload and return a TriggerEvent if it is
    /// a mention/message that should fire the agent.
    ///
    /// Returns `Ok(None)` for events that should be silently ignored (e.g. non-mention
    /// Slack messages, Slack URL verification challenges).
    /// Returns `Ok(Some(event))` when a mention is detected.
    pub fn process_event(
        &self,
        payload: serde_json::Value,
        _headers: &HashMap<String, String>,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        match &self.platform {
            ChannelPlatform::Slack { .. } => self.process_slack_event(payload),
            ChannelPlatform::Discord { .. } => self.process_discord_event(payload),
            ChannelPlatform::Telegram { .. } => self.process_telegram_event(payload),
        }
    }

    // -----------------------------------------------------------------------
    // Platform-specific event processing
    // -----------------------------------------------------------------------

    /// Slack Events API processing.
    ///
    /// Accepts:
    /// - `{type: "event_callback", event: {type: "app_mention", ...}}`
    ///
    /// Filters:
    /// - `{type: "url_verification", challenge: "..."}` → None (Slack setup)
    /// - `{type: "event_callback", event: {type: "message", ...}}` → None (non-mention)
    fn process_slack_event(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        let outer_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

        // Slack URL verification — return None (gateway should respond with challenge separately)
        if outer_type == "url_verification" {
            return Ok(None);
        }

        if outer_type != "event_callback" {
            return Ok(None);
        }

        let event = payload.get("event").ok_or_else(|| {
            AgentixError::runtime("Slack event_callback missing 'event' field")
        })?;

        let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if event_type != "app_mention" {
            return Ok(None);
        }

        let channel = event.get("channel").and_then(|v| v.as_str()).unwrap_or("");
        let user = event.get("user").and_then(|v| v.as_str()).unwrap_or("");
        let text = event.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let ts = event.get("ts").and_then(|v| v.as_str()).unwrap_or("");

        let trigger_payload = serde_json::json!({
            "channel": channel,
            "user": user,
            "text": text,
            "ts": ts,
        });

        let mut context = HashMap::new();
        context.insert("platform".to_string(), "slack".to_string());
        context.insert("channel".to_string(), channel.to_string());

        Ok(Some(TriggerEvent {
            source: TriggerSource::Slack,
            payload: trigger_payload,
            context,
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        }))
    }

    /// Discord MESSAGE_CREATE processing.
    ///
    /// Accepts:
    /// - `{t: "MESSAGE_CREATE", d: {channel_id, guild_id, content, author: {id, username}}}` when content contains `<@`
    ///
    /// Filters:
    /// - MESSAGE_CREATE without `<@` in content → None (not a mention)
    /// - Other event types → None
    fn process_discord_event(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        let event_type = payload.get("t").and_then(|v| v.as_str()).unwrap_or("");
        if event_type != "MESSAGE_CREATE" {
            return Ok(None);
        }

        let d = match payload.get("d") {
            Some(v) => v,
            None => return Ok(None),
        };

        let content = d.get("content").and_then(|v| v.as_str()).unwrap_or("");

        // Only fire if the message contains a bot mention pattern (<@USER_ID>)
        if !content.contains("<@") {
            return Ok(None);
        }

        let channel_id = d.get("channel_id").and_then(|v| v.as_str()).unwrap_or("");
        let guild_id = d.get("guild_id").and_then(|v| v.as_str()).unwrap_or("");
        let user_id = d
            .get("author")
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let username = d
            .get("author")
            .and_then(|a| a.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let trigger_payload = serde_json::json!({
            "channel_id": channel_id,
            "guild_id": guild_id,
            "user_id": user_id,
            "username": username,
            "content": content,
        });

        let mut context = HashMap::new();
        context.insert("platform".to_string(), "discord".to_string());
        context.insert("channel_id".to_string(), channel_id.to_string());

        Ok(Some(TriggerEvent {
            source: TriggerSource::Discord,
            payload: trigger_payload,
            context,
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        }))
    }

    /// Telegram webhook update processing.
    ///
    /// Accepts any `update.message.text` that is non-empty (all Telegram messages
    /// to the bot trigger the agent, since Telegram bots only receive messages sent
    /// to the bot directly or via `/start` in groups).
    fn process_telegram_event(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        let message = match payload.get("message") {
            Some(m) => m,
            None => return Ok(None), // Not a message update (could be edited_message, etc.)
        };

        let text = message.get("text").and_then(|v| v.as_str()).unwrap_or("");
        if text.is_empty() {
            return Ok(None);
        }

        let chat_id = message
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let user_id = message
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let username = message
            .get("from")
            .and_then(|f| f.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let trigger_payload = serde_json::json!({
            "chat_id": chat_id,
            "user_id": user_id,
            "username": username,
            "text": text,
        });

        let mut context = HashMap::new();
        context.insert("platform".to_string(), "telegram".to_string());
        context.insert("chat_id".to_string(), chat_id.to_string());

        Ok(Some(TriggerEvent {
            source: TriggerSource::Telegram,
            payload: trigger_payload,
            context,
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        }))
    }
}

// ---------------------------------------------------------------------------
// TriggerTrait implementation — passive trigger (stores sender at start())
// ---------------------------------------------------------------------------

#[async_trait]
impl TriggerTrait for ChannelMentionTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        match &self.platform {
            ChannelPlatform::Slack { .. } => TriggerSource::Slack,
            ChannelPlatform::Discord { .. } => TriggerSource::Discord,
            ChannelPlatform::Telegram { .. } => TriggerSource::Telegram,
        }
    }

    async fn start(
        &self,
        sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        let mut guard = self.sender.lock().await;
        *guard = Some(sender);
        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        let mut guard = self.sender.lock().await;
        *guard = None;
        Ok(())
    }
}
