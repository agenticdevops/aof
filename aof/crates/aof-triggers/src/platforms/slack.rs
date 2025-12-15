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
use tracing::{debug, error, warn};

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

    /// Bot user ID (for mention detection)
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

    /// Handle URL verification challenge
    pub fn handle_url_verification(&self, challenge: &str) -> String {
        debug!("Handling Slack URL verification challenge");
        challenge.to_string()
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
}

#[async_trait]
impl TriggerPlatform for SlackPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify signature first
        if let Some(signature) = headers.get("x-slack-signature") {
            if !self.verify_signature(raw, signature).await {
                warn!("Invalid Slack signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        let event_payload = self.parse_event_payload(raw)?;

        match event_payload {
            SlackEventPayload::UrlVerification { challenge } => {
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
}
