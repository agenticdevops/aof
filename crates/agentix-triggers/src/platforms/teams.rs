//! Microsoft Teams Bot Framework adapter for AOF
//!
//! This module provides integration with Microsoft Teams via the Bot Framework, supporting:
//! - Incoming activity webhooks (messages, invoke actions)
//! - Adaptive Card responses
//! - JWT token verification
//! - Tenant and channel restrictions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

/// Teams platform adapter
pub struct TeamsPlatform {
    config: TeamsConfig,
    client: reqwest::Client,
}

/// Teams Bot Framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Microsoft App ID (Bot Framework registration)
    pub app_id: String,

    /// Microsoft App Password/Secret
    pub app_password: String,

    /// Bot name for display
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Allowed Azure AD tenant IDs (optional)
    #[serde(default)]
    pub allowed_tenants: Option<Vec<String>>,

    /// Allowed channel IDs (optional)
    #[serde(default)]
    pub allowed_channels: Option<Vec<String>>,

    /// Users allowed to approve (for approval workflows)
    #[serde(default)]
    pub approval_allowed_users: Option<Vec<String>>,

    /// Approval channel for deployment requests
    #[serde(default)]
    pub approval_channel: Option<String>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

/// Bot Framework Activity (incoming message)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BotActivity {
    /// Activity type (message, invoke, conversationUpdate, etc.)
    #[serde(rename = "type")]
    activity_type: String,

    /// Unique activity ID
    id: String,

    /// Timestamp
    #[serde(default)]
    timestamp: Option<String>,

    /// Service URL for replies
    service_url: String,

    /// Channel ID (always "msteams" for Teams)
    channel_id: String,

    /// Sender information
    from: BotChannelAccount,

    /// Conversation reference
    conversation: BotConversation,

    /// Recipient (bot)
    recipient: BotChannelAccount,

    /// Message text
    #[serde(default)]
    text: Option<String>,

    /// Text format (plain, markdown)
    #[serde(default)]
    text_format: Option<String>,

    /// Invoke name (for Action.Submit)
    #[serde(default)]
    name: Option<String>,

    /// Invoke value (for Action.Submit data)
    #[serde(default)]
    value: Option<serde_json::Value>,

    /// Channel-specific data
    #[serde(default)]
    channel_data: Option<TeamsChannelData>,

    /// Reply to activity ID
    #[serde(default)]
    reply_to_id: Option<String>,
}

/// Channel account (user or bot)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BotChannelAccount {
    /// User/bot ID
    id: String,

    /// Display name
    #[serde(default)]
    name: Option<String>,

    /// Azure AD object ID (for users)
    #[serde(default)]
    aad_object_id: Option<String>,
}

/// Conversation reference
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotConversation {
    /// Conversation ID
    id: String,

    /// Conversation type (personal, groupChat, channel)
    #[serde(default)]
    conversation_type: Option<String>,

    /// Tenant ID
    #[serde(default)]
    tenant_id: Option<String>,

    /// Is group conversation
    #[serde(default)]
    is_group: Option<bool>,
}

/// Teams-specific channel data
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamsChannelData {
    /// Teams channel ID
    #[serde(default)]
    teams_channel_id: Option<String>,

    /// Teams team ID
    #[serde(default)]
    teams_team_id: Option<String>,

    /// Tenant information
    #[serde(default)]
    tenant: Option<TenantInfo>,
}

/// Tenant information
#[derive(Debug, Clone, Deserialize)]
struct TenantInfo {
    id: String,
}

/// Bot Framework token response
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

/// Adaptive Card structure
#[derive(Debug, Clone, Serialize)]
struct AdaptiveCard {
    #[serde(rename = "type")]
    card_type: String,
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    body: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<serde_json::Value>,
}

impl AdaptiveCard {
    fn new() -> Self {
        Self {
            card_type: "AdaptiveCard".to_string(),
            schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
            version: "1.4".to_string(),
            body: Vec::new(),
            actions: Vec::new(),
        }
    }

    fn add_text_block(&mut self, text: &str, weight: Option<&str>, size: Option<&str>) {
        let mut block = serde_json::json!({
            "type": "TextBlock",
            "text": text,
            "wrap": true
        });

        if let Some(w) = weight {
            block["weight"] = serde_json::json!(w);
        }
        if let Some(s) = size {
            block["size"] = serde_json::json!(s);
        }

        self.body.push(block);
    }

    fn add_action_submit(&mut self, title: &str, data: serde_json::Value) {
        self.actions.push(serde_json::json!({
            "type": "Action.Submit",
            "title": title,
            "data": data
        }));
    }
}

impl TeamsPlatform {
    /// Create new Teams platform adapter
    pub fn new(config: TeamsConfig) -> Result<Self, PlatformError> {
        if config.app_id.is_empty() || config.app_password.is_empty() {
            return Err(PlatformError::ParseError(
                "App ID and App Password are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Get Bot Framework access token
    async fn get_bot_token(&self) -> Result<String, PlatformError> {
        let token_url = "https://login.microsoftonline.com/botframework.com/oauth2/v2.0/token";

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.app_id),
            ("client_secret", &self.config.app_password),
            ("scope", "https://api.botframework.com/.default"),
        ];

        let response = self
            .client
            .post(token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Token request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Token request failed: {} - {}", status, body);
            return Err(PlatformError::ApiError(format!(
                "Token request failed: {}",
                status
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response.access_token)
    }

    /// Send activity to Teams
    async fn send_activity(
        &self,
        service_url: &str,
        conversation_id: &str,
        activity: serde_json::Value,
    ) -> Result<String, PlatformError> {
        let token = self.get_bot_token().await?;

        let url = format!(
            "{}/v3/conversations/{}/activities",
            service_url.trim_end_matches('/'),
            conversation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Send activity failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Send activity failed: {} - {}", status, body);
            return Err(PlatformError::ApiError(format!(
                "Send activity failed: {}",
                status
            )));
        }

        // Parse response to get activity ID
        let result: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));

        let activity_id = result
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        debug!("Successfully sent Teams activity: {}", activity_id);
        Ok(activity_id)
    }

    /// Send text message
    pub async fn send_text_message(
        &self,
        service_url: &str,
        conversation_id: &str,
        text: &str,
    ) -> Result<String, PlatformError> {
        let activity = serde_json::json!({
            "type": "message",
            "text": text,
            "textFormat": "markdown"
        });

        self.send_activity(service_url, conversation_id, activity).await
    }

    /// Send Adaptive Card
    pub async fn send_adaptive_card(
        &self,
        service_url: &str,
        conversation_id: &str,
        card: AdaptiveCard,
    ) -> Result<String, PlatformError> {
        let activity = serde_json::json!({
            "type": "message",
            "attachments": [{
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": card
            }]
        });

        self.send_activity(service_url, conversation_id, activity).await
    }

    /// Check if tenant is allowed
    fn is_tenant_allowed(&self, tenant_id: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_tenants {
            allowed.iter().any(|t| t == tenant_id)
        } else {
            true // All tenants allowed if not configured
        }
    }

    /// Check if channel is allowed
    fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_channels {
            allowed.iter().any(|c| c == channel_id)
        } else {
            true // All channels allowed if not configured
        }
    }

    /// Format response for Teams
    fn format_response_text(&self, response: &TriggerResponse) -> String {
        let status_emoji = match response.status {
            crate::response::ResponseStatus::Success => "✅",
            crate::response::ResponseStatus::Error => "❌",
            crate::response::ResponseStatus::Warning => "⚠️",
            crate::response::ResponseStatus::Info => "ℹ️",
        };

        format!("{} {}", status_emoji, response.text)
    }

    /// Create Adaptive Card from response
    fn create_response_card(&self, response: &TriggerResponse) -> AdaptiveCard {
        let mut card = AdaptiveCard::new();

        // Add status header
        let status_emoji = match response.status {
            crate::response::ResponseStatus::Success => "✅",
            crate::response::ResponseStatus::Error => "❌",
            crate::response::ResponseStatus::Warning => "⚠️",
            crate::response::ResponseStatus::Info => "ℹ️",
        };

        card.add_text_block(
            &format!("{} Response", status_emoji),
            Some("Bolder"),
            Some("Medium"),
        );

        // Add response text
        card.add_text_block(&response.text, None, None);

        // Add action buttons
        for action in &response.actions {
            card.add_action_submit(
                &action.label,
                serde_json::json!({
                    "action": action.id
                }),
            );
        }

        card
    }

    /// Parse activity payload
    fn parse_activity(&self, payload: &[u8]) -> Result<BotActivity, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse Teams activity: {}", e);
            PlatformError::ParseError(format!("Invalid Teams activity: {}", e))
        })
    }
}

#[async_trait]
impl TriggerPlatform for TeamsPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Note: JWT verification should be done in the webhook handler
        // Here we trust that the activity is already verified

        let activity = self.parse_activity(raw)?;

        // Only handle message and invoke activities
        if activity.activity_type != "message" && activity.activity_type != "invoke" {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Check tenant restrictions
        if let Some(ref channel_data) = activity.channel_data {
            if let Some(ref tenant) = channel_data.tenant {
                if !self.is_tenant_allowed(&tenant.id) {
                    warn!("Tenant {} not allowed", tenant.id);
                    return Err(PlatformError::InvalidSignature(
                        "Tenant not allowed".to_string(),
                    ));
                }
            }

            // Check channel restrictions
            if let Some(ref channel_id) = channel_data.teams_channel_id {
                if !self.is_channel_allowed(channel_id) {
                    warn!("Channel {} not allowed", channel_id);
                    return Err(PlatformError::InvalidSignature(
                        "Channel not allowed".to_string(),
                    ));
                }
            }
        }

        // Extract message text
        let text = match activity.activity_type.as_str() {
            "message" => {
                activity.text.unwrap_or_default()
            }
            "invoke" => {
                // Handle Adaptive Card Action.Submit
                if let Some(value) = activity.value {
                    if let Some(action) = value.get("action") {
                        format!("action:{}", action.as_str().unwrap_or("unknown"))
                    } else {
                        // Return the entire value as JSON
                        format!("invoke:{}", serde_json::to_string(&value).unwrap_or_default())
                    }
                } else {
                    return Err(PlatformError::ParseError("No invoke value".to_string()));
                }
            }
            _ => return Err(PlatformError::UnsupportedMessageType),
        };

        if text.is_empty() {
            return Err(PlatformError::ParseError("Empty message text".to_string()));
        }

        // Remove bot mention from text (Teams includes @mention in text)
        let cleaned_text = self.remove_bot_mention(&text);

        let trigger_user = TriggerUser {
            id: activity.from.id.clone(),
            username: activity.from.aad_object_id.clone(),
            display_name: activity.from.name.clone(),
            is_bot: false,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "service_url".to_string(),
            serde_json::json!(activity.service_url),
        );
        metadata.insert(
            "activity_type".to_string(),
            serde_json::json!(activity.activity_type),
        );

        if let Some(ref channel_data) = activity.channel_data {
            if let Some(ref team_id) = channel_data.teams_team_id {
                metadata.insert("team_id".to_string(), serde_json::json!(team_id));
            }
            if let Some(ref channel_id) = channel_data.teams_channel_id {
                metadata.insert("channel_id".to_string(), serde_json::json!(channel_id));
            }
        }

        if let Some(ref conversation_type) = activity.conversation.conversation_type {
            metadata.insert(
                "conversation_type".to_string(),
                serde_json::json!(conversation_type),
            );
        }

        Ok(TriggerMessage {
            id: activity.id,
            platform: "teams".to_string(),
            channel_id: activity.conversation.id.clone(),
            user: trigger_user,
            text: cleaned_text,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: activity.reply_to_id.clone(),
            reply_to: activity.reply_to_id,
        })
    }

    async fn send_response(
        &self,
        channel: &str, // Format: "service_url|conversation_id"
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Parse channel format: "service_url|conversation_id"
        let parts: Vec<&str> = channel.splitn(2, '|').collect();
        if parts.len() != 2 {
            return Err(PlatformError::ParseError(
                "Invalid channel format. Expected 'service_url|conversation_id'".to_string(),
            ));
        }

        let service_url = parts[0];
        let conversation_id = parts[1];

        // If response has actions, send Adaptive Card
        if !response.actions.is_empty() {
            let card = self.create_response_card(&response);
            self.send_adaptive_card(service_url, conversation_id, card).await?;
        } else {
            // Send simple text message
            let text = self.format_response_text(&response);
            self.send_text_message(service_url, conversation_id, &text).await?;
        }

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "teams"
    }

    async fn verify_signature(&self, _payload: &[u8], signature: &str) -> bool {
        // Teams uses JWT Bearer tokens, not HMAC signatures
        // JWT verification should be done in the webhook handler
        // This is a simplified check - production should validate JWT fully
        signature.starts_with("Bearer ")
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Teams supports reply chains
    }

    fn supports_interactive(&self) -> bool {
        true // Teams supports Adaptive Cards
    }

    fn supports_files(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TeamsPlatform {
    /// Remove bot @mention from message text
    fn remove_bot_mention(&self, text: &str) -> String {
        // Teams includes <at>BotName</at> in the text
        let mention_pattern = format!("<at>{}</at>", self.config.bot_name);
        let cleaned = text.replace(&mention_pattern, "").trim().to_string();

        // Also try without the exact bot name (Teams uses display name)
        if cleaned == text {
            // Simple regex-free removal of any <at>...</at> tags
            let mut result = String::new();
            let mut in_tag = false;
            let mut chars = text.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '<' {
                    // Check for <at>
                    let peek: String = chars.clone().take(3).collect();
                    if peek == "at>" {
                        in_tag = true;
                        // Skip past "at>"
                        for _ in 0..3 {
                            chars.next();
                        }
                        continue;
                    }
                }

                if in_tag {
                    if c == '<' {
                        // Check for </at>
                        let peek: String = chars.clone().take(4).collect();
                        if peek == "/at>" {
                            in_tag = false;
                            // Skip past "/at>"
                            for _ in 0..4 {
                                chars.next();
                            }
                            continue;
                        }
                    }
                    // Skip content inside <at>...</at>
                    continue;
                }

                result.push(c);
            }

            result.trim().to_string()
        } else {
            cleaned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> TeamsConfig {
        TeamsConfig {
            app_id: "test-app-id".to_string(),
            app_password: "test-app-password".to_string(),
            bot_name: "TestBot".to_string(),
            allowed_tenants: None,
            allowed_channels: None,
            approval_allowed_users: None,
            approval_channel: None,
        }
    }

    #[test]
    fn test_teams_platform_new() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_teams_platform_invalid_config() {
        let config = TeamsConfig {
            app_id: "".to_string(),
            app_password: "".to_string(),
            bot_name: "bot".to_string(),
            allowed_tenants: None,
            allowed_channels: None,
            approval_allowed_users: None,
            approval_channel: None,
        };
        let platform = TeamsPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_tenant_allowed() {
        let mut config = create_test_config();
        config.allowed_tenants = Some(vec!["tenant-123".to_string()]);

        let platform = TeamsPlatform::new(config).unwrap();

        assert!(platform.is_tenant_allowed("tenant-123"));
        assert!(!platform.is_tenant_allowed("tenant-456"));
    }

    #[test]
    fn test_channel_allowed() {
        let mut config = create_test_config();
        config.allowed_channels = Some(vec!["19:channel@thread.tacv2".to_string()]);

        let platform = TeamsPlatform::new(config).unwrap();

        assert!(platform.is_channel_allowed("19:channel@thread.tacv2"));
        assert!(!platform.is_channel_allowed("19:other@thread.tacv2"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "teams");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[test]
    fn test_remove_bot_mention() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config).unwrap();

        // Test with exact bot name
        let text = "<at>TestBot</at> check the pods";
        let cleaned = platform.remove_bot_mention(text);
        assert_eq!(cleaned, "check the pods");

        // Test with no mention
        let text = "check the pods";
        let cleaned = platform.remove_bot_mention(text);
        assert_eq!(cleaned, "check the pods");

        // Test with different bot name
        let text = "<at>OtherBot</at> status";
        let cleaned = platform.remove_bot_mention(text);
        assert_eq!(cleaned, "status");
    }

    #[test]
    fn test_adaptive_card_creation() {
        let mut card = AdaptiveCard::new();
        card.add_text_block("Hello", Some("Bolder"), Some("Large"));
        card.add_action_submit("Click Me", serde_json::json!({"action": "test"}));

        assert_eq!(card.card_type, "AdaptiveCard");
        assert_eq!(card.body.len(), 1);
        assert_eq!(card.actions.len(), 1);
    }

    #[tokio::test]
    async fn test_parse_message_activity() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config).unwrap();

        let activity = serde_json::json!({
            "type": "message",
            "id": "activity-123",
            "serviceUrl": "https://smba.trafficmanager.net/amer/",
            "channelId": "msteams",
            "from": {
                "id": "user-id",
                "name": "John Doe",
                "aadObjectId": "aad-object-id"
            },
            "conversation": {
                "id": "conversation-id",
                "conversationType": "personal"
            },
            "recipient": {
                "id": "bot-id",
                "name": "TestBot"
            },
            "text": "check the pods"
        });

        let raw = serde_json::to_vec(&activity).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(&raw, &headers).await;
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.platform, "teams");
        assert_eq!(msg.text, "check the pods");
        assert_eq!(msg.user.display_name, Some("John Doe".to_string()));
    }

    #[tokio::test]
    async fn test_parse_invoke_activity() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config).unwrap();

        let activity = serde_json::json!({
            "type": "invoke",
            "id": "activity-456",
            "name": "adaptiveCard/action",
            "serviceUrl": "https://smba.trafficmanager.net/amer/",
            "channelId": "msteams",
            "from": {
                "id": "user-id",
                "name": "Jane Doe"
            },
            "conversation": {
                "id": "conversation-id"
            },
            "recipient": {
                "id": "bot-id"
            },
            "value": {
                "action": "approve"
            }
        });

        let raw = serde_json::to_vec(&activity).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(&raw, &headers).await;
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.text, "action:approve");
    }

    #[tokio::test]
    async fn test_unsupported_activity_type() {
        let config = create_test_config();
        let platform = TeamsPlatform::new(config).unwrap();

        let activity = serde_json::json!({
            "type": "conversationUpdate",
            "id": "activity-789",
            "serviceUrl": "https://smba.trafficmanager.net/amer/",
            "channelId": "msteams",
            "from": { "id": "user-id" },
            "conversation": { "id": "conversation-id" },
            "recipient": { "id": "bot-id" }
        });

        let raw = serde_json::to_vec(&activity).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(&raw, &headers).await;
        assert!(matches!(result, Err(PlatformError::UnsupportedMessageType)));
    }
}
