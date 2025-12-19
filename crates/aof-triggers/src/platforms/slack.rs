//! Slack Events API adapter for AOF
//!
//! This module provides integration with Slack's Events API, supporting:
//! - Message events (DMs, mentions)
//! - Slash commands
//! - Interactive components
//! - Block Kit formatting
//! - HMAC-SHA256 signature verification

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

type HmacSha256 = Hmac<Sha256>;

/// Slack platform adapter
pub struct SlackPlatform {
    config: SlackConfig,
    client: reqwest::Client,
}

/// Slack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Bot OAuth token (starts with xoxb-)
    pub bot_token: String,

    /// Signing secret for request verification
    pub signing_secret: String,

    /// App ID
    pub app_id: String,

    /// Bot user ID (for mention detection and ignoring bot's own reactions)
    pub bot_user_id: String,

    /// Bot name
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Allowed workspace IDs (optional)
    #[serde(default)]
    pub allowed_workspaces: Option<Vec<String>>,

    /// Allowed channel IDs (optional)
    #[serde(default)]
    pub allowed_channels: Option<Vec<String>>,

    /// User IDs allowed to approve commands (optional - if empty, anyone can approve)
    #[serde(default)]
    pub approval_allowed_users: Option<Vec<String>>,

    /// Role labels that can approve commands (optional - for future RBAC)
    #[serde(default)]
    pub approval_allowed_roles: Option<Vec<String>>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

/// Slack Events API payload
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SlackEventPayload {
    UrlVerification {
        challenge: String,
    },
    EventCallback {
        team_id: String,
        event: SlackEvent,
    },
}

/// Slack slash command payload (form-urlencoded)
#[derive(Debug, Clone, Deserialize)]
struct SlackSlashCommand {
    command: String,
    text: String,
    user_id: String,
    user_name: String,
    channel_id: String,
    channel_name: String,
    team_id: String,
    response_url: String,
    trigger_id: String,
}

/// Slack event types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SlackEvent {
    Message {
        user: String,
        channel: String,
        text: String,
        ts: String,
        #[serde(default)]
        thread_ts: Option<String>,
    },
    AppMention {
        user: String,
        channel: String,
        text: String,
        ts: String,
        #[serde(default)]
        thread_ts: Option<String>,
    },
    AppHomeOpened {
        user: String,
        channel: String,
        tab: String,
    },
    /// Reaction added event for approval workflow
    ReactionAdded {
        user: String,
        reaction: String,
        item: ReactionItem,
    },
    /// Reaction removed event
    ReactionRemoved {
        user: String,
        reaction: String,
        item: ReactionItem,
    },
}

/// Item that received a reaction
#[derive(Debug, Clone, Deserialize)]
struct ReactionItem {
    #[serde(rename = "type")]
    item_type: String,
    channel: String,
    ts: String,
}

/// Slack user info response
#[derive(Debug, Deserialize)]
struct SlackUserInfo {
    ok: bool,
    user: Option<SlackUser>,
}

#[derive(Debug, Deserialize)]
struct SlackUser {
    id: String,
    name: String,
    real_name: Option<String>,
    is_bot: bool,
}

/// Slack API response
#[derive(Debug, Deserialize)]
struct SlackApiResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
}

/// Slack API response with message timestamp
#[derive(Debug, Deserialize)]
struct SlackPostMessageResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    ts: Option<String>,
    #[serde(default)]
    channel: Option<String>,
}

/// Response from Slack auth.test API
#[derive(Debug, Deserialize)]
struct SlackAuthTestResponse {
    ok: bool,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    bot_id: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

impl SlackPlatform {
    /// Create new Slack platform adapter
    pub fn new(config: SlackConfig) -> Result<Self, PlatformError> {
        if config.bot_token.is_empty() || config.signing_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "Bot token and signing secret are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Create new Slack platform adapter with auto-detected bot_user_id
    ///
    /// This async constructor calls Slack's auth.test API to automatically
    /// detect the bot's user ID, which is critical for preventing self-approval.
    pub async fn new_with_auto_detection(mut config: SlackConfig) -> Result<Self, PlatformError> {
        if config.bot_token.is_empty() || config.signing_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "Bot token and signing secret are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        // Auto-detect bot_user_id if not configured
        if config.bot_user_id.is_empty() {
            debug!("bot_user_id not configured, auto-detecting from Slack API...");
            match Self::fetch_bot_user_id(&client, &config.bot_token).await {
                Ok(user_id) => {
                    info!("Auto-detected bot_user_id: {}", user_id);
                    config.bot_user_id = user_id;
                }
                Err(e) => {
                    error!("Failed to auto-detect bot_user_id: {}. Self-approval prevention may not work!", e);
                    // Continue anyway, but log the warning
                }
            }
        }

        Ok(Self { config, client })
    }

    /// Fetch bot user ID from Slack's auth.test API
    async fn fetch_bot_user_id(client: &reqwest::Client, bot_token: &str) -> Result<String, PlatformError> {
        let response = client
            .get("https://slack.com/api/auth.test")
            .header("Authorization", format!("Bearer {}", bot_token))
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to call auth.test: {}", e)))?;

        let auth_response: SlackAuthTestResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse auth.test response: {}", e)))?;

        if !auth_response.ok {
            return Err(PlatformError::ApiError(format!(
                "auth.test failed: {}",
                auth_response.error.unwrap_or_else(|| "unknown error".to_string())
            )));
        }

        auth_response.user_id.ok_or_else(|| {
            PlatformError::ParseError("auth.test response missing user_id".to_string())
        })
    }

    /// Handle URL verification challenge
    pub fn handle_url_verification(&self, challenge: &str) -> String {
        debug!("Handling Slack URL verification challenge");
        challenge.to_string()
    }

    /// Verify Slack request signature using HMAC-SHA256
    ///
    /// Slack's signature is computed as:
    /// v0=HMAC_SHA256(signing_secret, "v0:{timestamp}:{body}")
    fn verify_slack_signature(&self, payload: &[u8], signature: &str, timestamp: &str) -> bool {
        // Slack signature format: v0=<hex_signature>
        if !signature.starts_with("v0=") {
            debug!("Invalid signature format - must start with v0=");
            return false;
        }

        let provided_signature = &signature[3..];

        // Build the base string: v0:{timestamp}:{body}
        let body_str = match std::str::from_utf8(payload) {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid UTF-8 in payload: {}", e);
                return false;
            }
        };

        let base_string = format!("v0:{}:{}", timestamp, body_str);
        debug!("Verifying signature for base string length: {}", base_string.len());

        let mut mac = match HmacSha256::new_from_slice(self.config.signing_secret.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                error!("HMAC setup failed: {}", e);
                return false;
            }
        };

        mac.update(base_string.as_bytes());

        let result = mac.finalize();
        let computed_signature = hex::encode(result.into_bytes());

        if computed_signature == provided_signature {
            debug!("Slack signature verified successfully");
            true
        } else {
            debug!("Signature mismatch - computed: {}, provided: {}",
                   &computed_signature[..8], &provided_signature[..8.min(provided_signature.len())]);
            false
        }
    }

    /// Format task status as Block Kit blocks
    pub fn format_task_status(&self, task_id: &str, status: &str, details: &str) -> serde_json::Value {
        serde_json::json!({
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": format!("Task {} Status", task_id),
                        "emoji": true
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("*Status:* {}\n{}", status, details)
                    }
                },
                {
                    "type": "divider"
                },
                {
                    "type": "context",
                    "elements": [{
                        "type": "mrkdwn",
                        "text": format!("Updated at <!date^{}^{{date_short_pretty}} {{time}}|now>",
                            chrono::Utc::now().timestamp())
                    }]
                }
            ]
        })
    }

    /// Create interactive message with actions
    pub fn create_interactive_message(
        &self,
        text: &str,
        actions: Vec<(String, String, String)>, // (label, action_id, value)
    ) -> serde_json::Value {
        let mut blocks = vec![serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": text
            }
        })];

        if !actions.is_empty() {
            let elements: Vec<serde_json::Value> = actions
                .into_iter()
                .map(|(label, action_id, value)| {
                    serde_json::json!({
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": label,
                            "emoji": true
                        },
                        "action_id": action_id,
                        "value": value
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

    /// Post message using chat.postMessage API
    async fn post_message(
        &self,
        channel: &str,
        response: &TriggerResponse,
    ) -> Result<(), PlatformError> {
        let blocks = response.format_for_slack();

        let mut payload = serde_json::json!({
            "channel": channel,
            "text": response.text.clone(),
            "blocks": blocks.get("blocks").unwrap_or(&serde_json::json!([]))
        });

        if let Some(ref thread_ts) = response.thread_id {
            payload["thread_ts"] = serde_json::json!(thread_ts);
        }

        let api_response = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json::<SlackApiResponse>()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if !api_response.ok {
            error!("Slack API error: {:?}", api_response.error);
            return Err(PlatformError::ApiError(
                api_response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        debug!("Successfully posted message to Slack channel {}", channel);
        Ok(())
    }

    /// Post message and return the message timestamp
    /// Returns (channel, timestamp) on success
    pub async fn post_message_with_ts(
        &self,
        channel: &str,
        text: &str,
        thread_ts: Option<&str>,
    ) -> Result<(String, String), PlatformError> {
        let mut payload = serde_json::json!({
            "channel": channel,
            "text": text,
        });

        if let Some(ts) = thread_ts {
            payload["thread_ts"] = serde_json::json!(ts);
        }

        let api_response = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json::<SlackPostMessageResponse>()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if !api_response.ok {
            error!("Slack API error: {:?}", api_response.error);
            return Err(PlatformError::ApiError(
                api_response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let ts = api_response.ts.ok_or_else(|| {
            PlatformError::ApiError("No message timestamp in response".to_string())
        })?;
        let channel = api_response.channel.unwrap_or_else(|| channel.to_string());

        debug!("Posted message to {} with ts {}", channel, ts);
        Ok((channel, ts))
    }

    /// Add a reaction to a message
    pub async fn add_reaction(
        &self,
        channel: &str,
        timestamp: &str,
        emoji: &str,
    ) -> Result<(), PlatformError> {
        let payload = serde_json::json!({
            "channel": channel,
            "timestamp": timestamp,
            "name": emoji,
        });

        let api_response = self
            .client
            .post("https://slack.com/api/reactions.add")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?
            .json::<SlackApiResponse>()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        // Don't fail if already_reacted - that's fine
        if !api_response.ok && api_response.error.as_deref() != Some("already_reacted") {
            error!("Slack API error adding reaction: {:?}", api_response.error);
            return Err(PlatformError::ApiError(
                api_response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        debug!("Added reaction {} to message {}", emoji, timestamp);
        Ok(())
    }

    /// Get bot token (for external use)
    pub fn bot_token(&self) -> &str {
        &self.config.bot_token
    }

    /// Get user info from Slack API
    async fn get_user_info(&self, user_id: &str) -> Result<TriggerUser, PlatformError> {
        let url = format!("https://slack.com/api/users.info?user={}", user_id);

        let user_info = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to get user info: {}", e)))?
            .json::<SlackUserInfo>()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse user info: {}", e)))?;

        if !user_info.ok || user_info.user.is_none() {
            return Ok(TriggerUser {
                id: user_id.to_string(),
                username: None,
                display_name: None,
                is_bot: false,
            });
        }

        let user = user_info.user.unwrap();
        Ok(TriggerUser {
            id: user.id,
            username: Some(user.name.clone()),
            display_name: user.real_name.clone().or(Some(user.name)),
            is_bot: user.is_bot,
        })
    }

    /// Parse event payload
    fn parse_event_payload(&self, payload: &[u8]) -> Result<SlackEventPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse Slack event payload: {}", e);
            PlatformError::ParseError(format!("Invalid Slack event payload: {}", e))
        })
    }

    /// Check if workspace is allowed
    fn is_workspace_allowed(&self, team_id: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_workspaces {
            allowed.contains(&team_id.to_string())
        } else {
            true // All workspaces allowed if not configured
        }
    }

    /// Check if channel is allowed
    fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_channels {
            allowed.contains(&channel_id.to_string())
        } else {
            true // All channels allowed if not configured
        }
    }

    /// Get bot user ID
    pub fn bot_user_id(&self) -> &str {
        &self.config.bot_user_id
    }

    /// Check if user is the bot itself
    pub fn is_bot_user(&self, user_id: &str) -> bool {
        self.config.bot_user_id == user_id
    }

    /// Check if user is allowed to approve commands
    /// Returns true if:
    /// - No approval whitelist is configured (anyone can approve)
    /// - User is in the approval whitelist
    pub fn can_approve(&self, user_id: &str) -> bool {
        match &self.config.approval_allowed_users {
            Some(allowed) => allowed.contains(&user_id.to_string()),
            None => true, // Anyone can approve if no whitelist configured
        }
    }

    /// Parse a slash command payload (form-urlencoded)
    ///
    /// Slack sends slash commands as application/x-www-form-urlencoded POST requests.
    /// This method parses the payload and converts it to a TriggerMessage.
    async fn parse_slash_command(&self, raw: &[u8]) -> Result<TriggerMessage, PlatformError> {
        let body_str = std::str::from_utf8(raw).map_err(|e| {
            error!("Invalid UTF-8 in slash command payload: {}", e);
            PlatformError::ParseError(format!("Invalid UTF-8: {}", e))
        })?;

        debug!("Parsing slash command payload: {}", body_str);

        // Parse form-urlencoded data
        let params: HashMap<String, String> = url::form_urlencoded::parse(body_str.as_bytes())
            .into_owned()
            .collect();

        // Extract required fields
        let command = params
            .get("command")
            .ok_or_else(|| PlatformError::ParseError("Missing command field".to_string()))?;
        let text = params.get("text").cloned().unwrap_or_default();
        let user_id = params
            .get("user_id")
            .ok_or_else(|| PlatformError::ParseError("Missing user_id field".to_string()))?;
        let user_name = params.get("user_name").cloned().unwrap_or_default();
        let channel_id = params
            .get("channel_id")
            .ok_or_else(|| PlatformError::ParseError("Missing channel_id field".to_string()))?;
        let channel_name = params.get("channel_name").cloned().unwrap_or_default();
        let team_id = params.get("team_id").cloned().unwrap_or_default();
        let response_url = params.get("response_url").cloned().unwrap_or_default();
        let trigger_id = params.get("trigger_id").cloned().unwrap_or_default();

        info!(
            "Received slash command: {} '{}' from user {} in channel {}",
            command, text, user_name, channel_name
        );

        // Check workspace/channel restrictions
        if !team_id.is_empty() && !self.is_workspace_allowed(&team_id) {
            warn!("Workspace {} not allowed for slash command", team_id);
            return Err(PlatformError::InvalidSignature(
                "Workspace not allowed".to_string(),
            ));
        }

        if !self.is_channel_allowed(channel_id) {
            warn!("Channel {} not allowed for slash command", channel_id);
            return Err(PlatformError::InvalidSignature(
                "Channel not allowed".to_string(),
            ));
        }

        // Create TriggerUser
        let trigger_user = TriggerUser {
            id: user_id.clone(),
            username: Some(user_name.clone()),
            display_name: Some(user_name),
            is_bot: false,
        };

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("team_id".to_string(), serde_json::json!(team_id));
        metadata.insert("channel_name".to_string(), serde_json::json!(channel_name));
        metadata.insert("command".to_string(), serde_json::json!(command));
        metadata.insert("response_url".to_string(), serde_json::json!(response_url));
        metadata.insert("trigger_id".to_string(), serde_json::json!(trigger_id));
        metadata.insert("event_type".to_string(), serde_json::json!("slash_command"));

        // Generate unique ID from timestamp
        let id = format!("slash_{}_{}", chrono::Utc::now().timestamp_millis(), user_id);

        Ok(TriggerMessage {
            id,
            platform: "slack".to_string(),
            channel_id: channel_id.clone(),
            user: trigger_user,
            text, // The text after the command (e.g., "show me pods" from "/aof show me pods")
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: None, // Slash commands don't have threads
            reply_to: None,
        })
    }
}

#[async_trait]
impl TriggerPlatform for SlackPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify signature first (requires both signature and timestamp headers)
        if let (Some(signature), Some(timestamp)) = (
            headers.get("x-slack-signature"),
            headers.get("x-slack-request-timestamp"),
        ) {
            if !self.verify_slack_signature(raw, signature, timestamp) {
                warn!("Invalid Slack signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        // Check content-type to determine if this is a slash command or event
        let content_type = headers
            .get("content-type")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Slash commands use application/x-www-form-urlencoded
        if content_type.contains("application/x-www-form-urlencoded") {
            return self.parse_slash_command(raw).await;
        }

        // Events API uses application/json
        let event_payload = self.parse_event_payload(raw)?;

        match event_payload {
            SlackEventPayload::UrlVerification { challenge: _ } => {
                // This should be handled separately by the webhook handler
                Err(PlatformError::UnsupportedMessageType)
            }
            SlackEventPayload::EventCallback { team_id, event } => {
                if !self.is_workspace_allowed(&team_id) {
                    warn!("Workspace {} not allowed", team_id);
                    return Err(PlatformError::InvalidSignature(
                        "Workspace not allowed".to_string(),
                    ));
                }

                match event {
                    SlackEvent::Message {
                        user,
                        channel,
                        text,
                        ts,
                        thread_ts,
                    }
                    | SlackEvent::AppMention {
                        user,
                        channel,
                        text,
                        ts,
                        thread_ts,
                    } => {
                        if !self.is_channel_allowed(&channel) {
                            warn!("Channel {} not allowed", channel);
                            return Err(PlatformError::InvalidSignature(
                                "Channel not allowed".to_string(),
                            ));
                        }

                        // Get user info (or use fallback)
                        let trigger_user = self.get_user_info(&user).await.unwrap_or_else(|_| {
                            TriggerUser {
                                id: user.clone(),
                                username: None,
                                display_name: None,
                                is_bot: false,
                            }
                        });

                        let mut metadata = HashMap::new();
                        metadata.insert("team_id".to_string(), serde_json::json!(team_id));

                        Ok(TriggerMessage {
                            id: ts.clone(),
                            platform: "slack".to_string(),
                            channel_id: channel,
                            user: trigger_user,
                            text,
                            timestamp: chrono::Utc::now(),
                            metadata,
                            thread_id: thread_ts.or(Some(ts)),
                            reply_to: None,
                        })
                    }
                    SlackEvent::ReactionAdded { user, reaction, item } => {
                        info!("Reaction added: {} by {} on message {} in channel {}", reaction, user, item.ts, item.channel);

                        // CRITICAL: Ignore reactions from the bot itself to prevent self-approval
                        if self.is_bot_user(&user) {
                            info!("Ignoring reaction from bot itself (user_id: {})", user);
                            return Err(PlatformError::UnsupportedMessageType);
                        }

                        info!("Processing user reaction: {} by {} on message {}", reaction, user, item.ts);

                        // Get user info
                        let trigger_user = self.get_user_info(&user).await.unwrap_or_else(|_| {
                            TriggerUser {
                                id: user.clone(),
                                username: None,
                                display_name: None,
                                is_bot: false,
                            }
                        });

                        // Check if user is allowed to approve (if whitelist is configured)
                        let can_approve = self.can_approve(&user);

                        let mut metadata = HashMap::new();
                        metadata.insert("team_id".to_string(), serde_json::json!(team_id));
                        metadata.insert("event_type".to_string(), serde_json::json!("reaction_added"));
                        metadata.insert("reaction".to_string(), serde_json::json!(reaction));
                        metadata.insert("item_ts".to_string(), serde_json::json!(item.ts));
                        metadata.insert("can_approve".to_string(), serde_json::json!(can_approve));

                        // Use item.ts as the thread_id so handler can look up pending approval
                        Ok(TriggerMessage {
                            id: format!("reaction_{}_{}", item.ts, reaction),
                            platform: "slack".to_string(),
                            channel_id: item.channel,
                            user: trigger_user,
                            text: format!("reaction:{}", reaction),
                            timestamp: chrono::Utc::now(),
                            metadata,
                            thread_id: Some(item.ts),
                            reply_to: None,
                        })
                    }
                    SlackEvent::ReactionRemoved { .. } => {
                        // Ignore reaction removed events for now
                        Err(PlatformError::UnsupportedMessageType)
                    }
                    SlackEvent::AppHomeOpened { .. } => Err(PlatformError::UnsupportedMessageType),
                }
            }
        }
    }

    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        self.post_message(channel, &response).await
    }

    fn platform_name(&self) -> &'static str {
        "slack"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Slack signature format: v0=<hex_signature>
        if !signature.starts_with("v0=") {
            return false;
        }

        let provided_signature = &signature[3..];

        // In production, should also verify timestamp from X-Slack-Request-Timestamp
        // to prevent replay attacks (reject if older than 5 minutes)

        let mut mac = match HmacSha256::new_from_slice(self.config.signing_secret.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                error!("HMAC setup failed: {}", e);
                return false;
            }
        };

        mac.update(payload);

        let result = mac.finalize();
        let computed_signature = hex::encode(result.into_bytes());

        computed_signature == provided_signature
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true
    }

    fn supports_interactive(&self) -> bool {
        true
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

    fn create_test_config() -> SlackConfig {
        SlackConfig {
            bot_token: "xoxb-test-token".to_string(),
            signing_secret: "test-secret".to_string(),
            app_id: "A123456".to_string(),
            bot_user_id: "U123456".to_string(),
            bot_name: "testbot".to_string(),
            allowed_workspaces: None,
            allowed_channels: None,
            approval_allowed_users: None,
            approval_allowed_roles: None,
        }
    }

    #[test]
    fn test_slack_platform_new() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_slack_platform_invalid_config() {
        let config = SlackConfig {
            bot_token: "".to_string(),
            signing_secret: "".to_string(),
            app_id: "A123456".to_string(),
            bot_user_id: "U123456".to_string(),
            bot_name: "testbot".to_string(),
            allowed_workspaces: None,
            allowed_channels: None,
            approval_allowed_users: None,
            approval_allowed_roles: None,
        };
        let platform = SlackPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_format_task_status() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        let blocks = platform.format_task_status("task-123", "completed", "All tests passed");
        assert!(blocks.get("blocks").is_some());
    }

    #[test]
    fn test_create_interactive_message() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        let actions = vec![
            (
                "Approve".to_string(),
                "approve".to_string(),
                "yes".to_string(),
            ),
            (
                "Reject".to_string(),
                "reject".to_string(),
                "no".to_string(),
            ),
        ];

        let blocks = platform.create_interactive_message("Please review this PR", actions);
        let blocks_array = blocks.get("blocks").unwrap().as_array().unwrap();
        assert!(blocks_array.len() >= 2); // Section + Actions
    }

    #[tokio::test]
    async fn test_verify_signature_format() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        let payload = b"test payload";
        let invalid_sig = "invalid";

        let result = platform.verify_signature(payload, invalid_sig).await;
        assert!(!result); // Should be false for invalid format
    }

    #[test]
    fn test_workspace_allowed() {
        let mut config = create_test_config();
        config.allowed_workspaces = Some(vec!["T123456".to_string()]);

        let platform = SlackPlatform::new(config).unwrap();
        assert!(platform.is_workspace_allowed("T123456"));
        assert!(!platform.is_workspace_allowed("T999999"));
    }

    #[test]
    fn test_channel_allowed() {
        let mut config = create_test_config();
        config.allowed_channels = Some(vec!["C123456".to_string()]);

        let platform = SlackPlatform::new(config).unwrap();
        assert!(platform.is_channel_allowed("C123456"));
        assert!(!platform.is_channel_allowed("C999999"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "slack");
        assert_eq!(platform.bot_name(), "testbot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[tokio::test]
    async fn test_parse_slash_command() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        // Simulate Slack slash command form-urlencoded payload
        let payload = "command=%2Faof&text=show+me+pods&user_id=U12345&user_name=testuser&channel_id=C12345&channel_name=general&team_id=T12345&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2Fxxx&trigger_id=123.456.xxx";

        let result = platform.parse_slash_command(payload.as_bytes()).await;
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.platform, "slack");
        assert_eq!(msg.text, "show me pods");
        assert_eq!(msg.channel_id, "C12345");
        assert_eq!(msg.user.id, "U12345");
        assert_eq!(msg.user.username.as_deref(), Some("testuser"));
        assert_eq!(
            msg.metadata.get("command").and_then(|v| v.as_str()),
            Some("/aof")
        );
        assert_eq!(
            msg.metadata.get("event_type").and_then(|v| v.as_str()),
            Some("slash_command")
        );
    }

    #[tokio::test]
    async fn test_parse_slash_command_empty_text() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        // Slash command with no text (just "/aof")
        let payload = "command=%2Faof&text=&user_id=U12345&user_name=testuser&channel_id=C12345&channel_name=general&team_id=T12345&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2Fxxx&trigger_id=123.456.xxx";

        let result = platform.parse_slash_command(payload.as_bytes()).await;
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.text, ""); // Empty text is valid
    }

    #[tokio::test]
    async fn test_parse_slash_command_missing_required_field() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        // Missing user_id
        let payload = "command=%2Faof&text=hello&channel_id=C12345";

        let result = platform.parse_slash_command(payload.as_bytes()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_message_detects_slash_command() {
        let config = create_test_config();
        let platform = SlackPlatform::new(config).unwrap();

        let payload = "command=%2Faof&text=show+me+pods&user_id=U12345&user_name=testuser&channel_id=C12345&channel_name=general&team_id=T12345&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2Fxxx&trigger_id=123.456.xxx";

        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );

        let result = platform.parse_message(payload.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.text, "show me pods");
        assert_eq!(
            msg.metadata.get("event_type").and_then(|v| v.as_str()),
            Some("slash_command")
        );
    }
}
