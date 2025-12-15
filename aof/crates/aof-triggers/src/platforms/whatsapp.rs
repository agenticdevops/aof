//! WhatsApp Business Cloud API adapter for AOF
//!
//! This module provides integration with WhatsApp's Business Cloud API, supporting:
//! - Incoming message webhooks
//! - Text and interactive message handling
//! - HMAC-SHA256 signature verification
//! - Template messages for notifications

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

type HmacSha256 = Hmac<Sha256>;

/// WhatsApp platform adapter
pub struct WhatsAppPlatform {
    config: WhatsAppConfig,
    client: reqwest::Client,
}

/// WhatsApp configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    /// Phone number ID (from Meta Business)
    pub phone_number_id: String,

    /// Access token for API calls
    pub access_token: String,

    /// Verify token for webhook verification
    pub verify_token: String,

    /// App secret for signature verification
    pub app_secret: String,

    /// Business account ID (optional)
    #[serde(default)]
    pub business_account_id: Option<String>,

    /// Allowed phone numbers (optional whitelist)
    #[serde(default)]
    pub allowed_numbers: Option<Vec<String>>,

    /// API version (default v18.0)
    #[serde(default = "default_api_version")]
    pub api_version: String,
}

fn default_api_version() -> String {
    "v18.0".to_string()
}

/// WhatsApp webhook payload
#[derive(Debug, Clone, Deserialize)]
struct WhatsAppWebhook {
    object: String,
    entry: Vec<WebhookEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookEntry {
    id: String,
    changes: Vec<WebhookChange>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookChange {
    value: WebhookValue,
    field: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookValue {
    messaging_product: String,
    metadata: WebhookMetadata,
    #[serde(default)]
    contacts: Vec<WebhookContact>,
    #[serde(default)]
    messages: Vec<WebhookMessage>,
    #[serde(default)]
    statuses: Vec<WebhookStatus>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookMetadata {
    display_phone_number: String,
    phone_number_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookContact {
    profile: ContactProfile,
    wa_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ContactProfile {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookMessage {
    from: String,
    id: String,
    timestamp: String,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(default)]
    text: Option<TextMessage>,
    #[serde(default)]
    interactive: Option<InteractiveResponse>,
}

#[derive(Debug, Clone, Deserialize)]
struct TextMessage {
    body: String,
}

#[derive(Debug, Clone, Deserialize)]
struct InteractiveResponse {
    #[serde(rename = "type")]
    interactive_type: String,
    #[serde(default)]
    button_reply: Option<ButtonReply>,
    #[serde(default)]
    list_reply: Option<ListReply>,
}

#[derive(Debug, Clone, Deserialize)]
struct ButtonReply {
    id: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ListReply {
    id: String,
    title: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebhookStatus {
    id: String,
    status: String,
    timestamp: String,
    recipient_id: String,
}

/// WhatsApp API response
#[derive(Debug, Deserialize)]
struct WhatsAppApiResponse {
    #[serde(default)]
    messages: Vec<MessageResponse>,
    #[serde(default)]
    error: Option<WhatsAppError>,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct WhatsAppError {
    message: String,
    code: i32,
}

impl WhatsAppPlatform {
    /// Create new WhatsApp platform adapter
    pub fn new(config: WhatsAppConfig) -> Result<Self, PlatformError> {
        if config.phone_number_id.is_empty() || config.access_token.is_empty() {
            return Err(PlatformError::ParseError(
                "Phone number ID and access token are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify webhook subscription (GET request)
    pub fn verify_webhook(&self, mode: &str, token: &str, challenge: &str) -> Option<String> {
        if mode == "subscribe" && token == self.config.verify_token {
            debug!("WhatsApp webhook verification successful");
            Some(challenge.to_string())
        } else {
            warn!("WhatsApp webhook verification failed");
            None
        }
    }

    /// Send text message
    pub async fn send_text_message(
        &self,
        to: &str,
        text: &str,
    ) -> Result<String, PlatformError> {
        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.config.api_version, self.config.phone_number_id
        );

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": {
                "preview_url": false,
                "body": text
            }
        });

        self.send_message_api(&url, &payload).await
    }

    /// Send interactive message with buttons
    pub async fn send_interactive_buttons(
        &self,
        to: &str,
        body_text: &str,
        buttons: Vec<(String, String)>, // (id, title)
    ) -> Result<String, PlatformError> {
        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.config.api_version, self.config.phone_number_id
        );

        let button_objects: Vec<serde_json::Value> = buttons
            .into_iter()
            .take(3) // WhatsApp allows max 3 buttons
            .map(|(id, title)| {
                serde_json::json!({
                    "type": "reply",
                    "reply": {
                        "id": id,
                        "title": title.chars().take(20).collect::<String>() // Max 20 chars
                    }
                })
            })
            .collect();

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "interactive",
            "interactive": {
                "type": "button",
                "body": {
                    "text": body_text
                },
                "action": {
                    "buttons": button_objects
                }
            }
        });

        self.send_message_api(&url, &payload).await
    }

    /// Send interactive list message
    pub async fn send_interactive_list(
        &self,
        to: &str,
        header: &str,
        body_text: &str,
        button_text: &str,
        sections: Vec<ListSection>,
    ) -> Result<String, PlatformError> {
        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.config.api_version, self.config.phone_number_id
        );

        let section_objects: Vec<serde_json::Value> = sections
            .into_iter()
            .map(|section| {
                let rows: Vec<serde_json::Value> = section
                    .rows
                    .into_iter()
                    .map(|row| {
                        serde_json::json!({
                            "id": row.id,
                            "title": row.title,
                            "description": row.description
                        })
                    })
                    .collect();

                serde_json::json!({
                    "title": section.title,
                    "rows": rows
                })
            })
            .collect();

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "interactive",
            "interactive": {
                "type": "list",
                "header": {
                    "type": "text",
                    "text": header
                },
                "body": {
                    "text": body_text
                },
                "action": {
                    "button": button_text,
                    "sections": section_objects
                }
            }
        });

        self.send_message_api(&url, &payload).await
    }

    /// Internal: Send message via API
    async fn send_message_api(
        &self,
        url: &str,
        payload: &serde_json::Value,
    ) -> Result<String, PlatformError> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        let api_response: WhatsAppApiResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = api_response.error {
            error!("WhatsApp API error: {} (code: {})", error.message, error.code);
            return Err(PlatformError::ApiError(error.message));
        }

        let message_id = api_response
            .messages
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        debug!("Successfully sent WhatsApp message: {}", message_id);
        Ok(message_id)
    }

    /// Check if phone number is allowed
    fn is_number_allowed(&self, phone_number: &str) -> bool {
        if let Some(ref allowed) = self.config.allowed_numbers {
            allowed.contains(&phone_number.to_string())
        } else {
            true // All numbers allowed if not configured
        }
    }

    /// Parse webhook payload
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<WhatsAppWebhook, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse WhatsApp webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid WhatsApp webhook payload: {}", e))
        })
    }

    /// Format response for WhatsApp
    fn format_response_text(&self, response: &TriggerResponse) -> String {
        // WhatsApp uses simple formatting
        let status_emoji = match response.status {
            crate::response::ResponseStatus::Success => "✅",
            crate::response::ResponseStatus::Error => "❌",
            crate::response::ResponseStatus::Warning => "⚠️",
            crate::response::ResponseStatus::Info => "ℹ️",
        };

        format!("{} {}", status_emoji, response.text)
    }
}

/// List section for interactive list messages
#[derive(Debug, Clone)]
pub struct ListSection {
    pub title: String,
    pub rows: Vec<ListRow>,
}

/// List row for interactive list messages
#[derive(Debug, Clone)]
pub struct ListRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

#[async_trait]
impl TriggerPlatform for WhatsAppPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify signature if present
        if let Some(signature) = headers.get("x-hub-signature-256") {
            if !self.verify_signature(raw, signature).await {
                warn!("Invalid WhatsApp signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        let webhook = self.parse_webhook_payload(raw)?;

        // WhatsApp webhooks should have object = "whatsapp_business_account"
        if webhook.object != "whatsapp_business_account" {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Extract first message from webhook
        for entry in webhook.entry {
            for change in entry.changes {
                if change.field != "messages" {
                    continue;
                }

                let value = change.value;

                // Skip status updates
                if !value.statuses.is_empty() {
                    return Err(PlatformError::UnsupportedMessageType);
                }

                for message in value.messages {
                    // Check if number is allowed
                    if !self.is_number_allowed(&message.from) {
                        warn!("Phone number {} not allowed", message.from);
                        return Err(PlatformError::InvalidSignature(
                            "Phone number not allowed".to_string(),
                        ));
                    }

                    // Get contact name
                    let contact_name = value
                        .contacts
                        .first()
                        .map(|c| c.profile.name.clone())
                        .unwrap_or_else(|| message.from.clone());

                    // Extract message text based on type
                    let text = match message.message_type.as_str() {
                        "text" => message
                            .text
                            .map(|t| t.body)
                            .unwrap_or_default(),
                        "interactive" => {
                            if let Some(interactive) = message.interactive {
                                match interactive.interactive_type.as_str() {
                                    "button_reply" => interactive
                                        .button_reply
                                        .map(|b| format!("button:{}", b.id))
                                        .unwrap_or_default(),
                                    "list_reply" => interactive
                                        .list_reply
                                        .map(|l| format!("list:{}", l.id))
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                }
                            } else {
                                String::new()
                            }
                        }
                        _ => {
                            return Err(PlatformError::UnsupportedMessageType);
                        }
                    };

                    if text.is_empty() {
                        return Err(PlatformError::ParseError("Empty message text".to_string()));
                    }

                    let trigger_user = TriggerUser {
                        id: message.from.clone(),
                        username: Some(message.from.clone()),
                        display_name: Some(contact_name),
                        is_bot: false,
                    };

                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "phone_number_id".to_string(),
                        serde_json::json!(value.metadata.phone_number_id),
                    );
                    metadata.insert(
                        "display_phone_number".to_string(),
                        serde_json::json!(value.metadata.display_phone_number),
                    );

                    return Ok(TriggerMessage {
                        id: message.id,
                        platform: "whatsapp".to_string(),
                        channel_id: message.from.clone(), // Use phone number as channel
                        user: trigger_user,
                        text,
                        timestamp: chrono::Utc::now(),
                        metadata,
                        thread_id: None,
                        reply_to: None,
                    });
                }
            }
        }

        Err(PlatformError::ParseError("No message found in webhook".to_string()))
    }

    async fn send_response(
        &self,
        channel: &str, // Phone number
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        let text = self.format_response_text(&response);

        // If response has actions, send interactive buttons
        if !response.actions.is_empty() {
            let buttons: Vec<(String, String)> = response
                .actions
                .iter()
                .take(3)
                .map(|a| (a.id.clone(), a.label.clone()))
                .collect();

            self.send_interactive_buttons(channel, &text, buttons).await?;
        } else {
            self.send_text_message(channel, &text).await?;
        }

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "whatsapp"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // WhatsApp signature format: sha256=<hex_signature>
        if !signature.starts_with("sha256=") {
            return false;
        }

        let provided_signature = &signature[7..];

        let mut mac = match HmacSha256::new_from_slice(self.config.app_secret.as_bytes()) {
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
        "aofbot"
    }

    fn supports_threading(&self) -> bool {
        false // WhatsApp doesn't support threading
    }

    fn supports_interactive(&self) -> bool {
        true // WhatsApp supports interactive buttons and lists
    }

    fn supports_files(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> WhatsAppConfig {
        WhatsAppConfig {
            phone_number_id: "123456789".to_string(),
            access_token: "test-token".to_string(),
            verify_token: "verify-me".to_string(),
            app_secret: "test-secret".to_string(),
            business_account_id: None,
            allowed_numbers: None,
            api_version: "v18.0".to_string(),
        }
    }

    #[test]
    fn test_whatsapp_platform_new() {
        let config = create_test_config();
        let platform = WhatsAppPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_whatsapp_platform_invalid_config() {
        let config = WhatsAppConfig {
            phone_number_id: "".to_string(),
            access_token: "".to_string(),
            verify_token: "".to_string(),
            app_secret: "".to_string(),
            business_account_id: None,
            allowed_numbers: None,
            api_version: "v18.0".to_string(),
        };
        let platform = WhatsAppPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_webhook_verification() {
        let config = create_test_config();
        let platform = WhatsAppPlatform::new(config).unwrap();

        // Valid verification
        let result = platform.verify_webhook("subscribe", "verify-me", "challenge123");
        assert_eq!(result, Some("challenge123".to_string()));

        // Invalid token
        let result = platform.verify_webhook("subscribe", "wrong-token", "challenge123");
        assert_eq!(result, None);

        // Invalid mode
        let result = platform.verify_webhook("unsubscribe", "verify-me", "challenge123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_number_allowed() {
        let mut config = create_test_config();
        config.allowed_numbers = Some(vec!["1234567890".to_string()]);

        let platform = WhatsAppPlatform::new(config).unwrap();

        assert!(platform.is_number_allowed("1234567890"));
        assert!(!platform.is_number_allowed("9999999999"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = WhatsAppPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "whatsapp");
        assert!(!platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(platform.supports_files());
    }

    #[tokio::test]
    async fn test_verify_signature_format() {
        let config = create_test_config();
        let platform = WhatsAppPlatform::new(config).unwrap();

        let payload = b"test payload";
        let invalid_sig = "invalid";

        let result = platform.verify_signature(payload, invalid_sig).await;
        assert!(!result);
    }
}
