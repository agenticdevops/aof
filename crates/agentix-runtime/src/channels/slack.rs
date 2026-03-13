//! Slack channel gateway adapter (Phase 21)
//!
//! Bi-directional communication with Slack channels via Events API webhooks
//! and chat.postMessage responses. Notifications use Block Kit formatting.

use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelMessage, ChannelPlatformType, NotificationPayload,
    NotificationSeverity, NotificationTarget,
};
use agentix_core::AgentixError;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error};

type HmacSha256 = Hmac<Sha256>;

/// Slack channel gateway — implements ChannelGateway for Slack Events API
pub struct SlackChannelGateway {
    bot_token: String,
    signing_secret: String,
    app_id: String,
    client: reqwest::Client,
}

impl SlackChannelGateway {
    /// Create a new SlackChannelGateway from Slack credentials
    pub fn new(credentials: &ChannelCredentials) -> Result<Self, AgentixError> {
        match credentials {
            ChannelCredentials::Slack {
                bot_token,
                signing_secret,
                app_id,
            } => Ok(Self {
                bot_token: bot_token.clone(),
                signing_secret: signing_secret.clone(),
                app_id: app_id.clone(),
                client: reqwest::Client::new(),
            }),
            _ => Err(AgentixError::Config(
                "SlackChannelGateway requires Slack credentials".into(),
            )),
        }
    }

    /// Parse a Slack Events API webhook payload into a ChannelMessage
    ///
    /// Returns None for url_verification events and non-mention messages.
    /// Returns Some(ChannelMessage) for app_mention events.
    pub fn parse_webhook(
        &self,
        payload: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<Option<ChannelMessage>, AgentixError> {
        let body: Value = serde_json::from_slice(payload)
            .map_err(|e| AgentixError::Config(format!("Invalid Slack webhook payload: {}", e)))?;

        let event_type = body.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match event_type {
            "url_verification" => {
                debug!("Slack url_verification event — returning None");
                Ok(None)
            }
            "event_callback" => {
                let event = match body.get("event") {
                    Some(e) => e,
                    None => return Ok(None),
                };

                let inner_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");

                if inner_type != "app_mention" {
                    debug!("Slack event type '{}' — not app_mention, skipping", inner_type);
                    return Ok(None);
                }

                let channel_id = event
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let user_id = event
                    .get("user")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let text = event
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let ts = event
                    .get("ts")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");

                let timestamp = parse_slack_ts(ts);

                let thread_id = event
                    .get("thread_ts")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(Some(ChannelMessage {
                    platform: ChannelPlatformType::Slack,
                    channel_id,
                    user_id,
                    user_name: String::new(), // Slack Events API doesn't include username in events
                    text,
                    thread_id,
                    timestamp,
                    raw_payload: body.clone(),
                }))
            }
            _ => {
                debug!("Unknown Slack event type: {}", event_type);
                Ok(None)
            }
        }
    }

    /// Verify a Slack webhook signature using HMAC-SHA256
    ///
    /// Computes HMAC-SHA256 of `v0:{timestamp}:{body}` using signing_secret
    /// and compares against the provided `v0={hex}` signature.
    pub fn verify_signature(&self, payload: &[u8], signature: &str, timestamp: &str) -> bool {
        if !signature.starts_with("v0=") {
            return false;
        }

        let base_string = format!(
            "v0:{}:{}",
            timestamp,
            String::from_utf8_lossy(payload)
        );

        let mut mac = match HmacSha256::new_from_slice(self.signing_secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(base_string.as_bytes());
        let result = mac.finalize();
        let computed = format!("v0={}", hex::encode(result.into_bytes()));

        computed == signature
    }

    /// Format a NotificationPayload as Slack Block Kit blocks
    pub fn format_notification_blocks(payload: &NotificationPayload) -> Value {
        let (color, severity_text) = match payload.severity {
            NotificationSeverity::Info => ("#36a64f", "Info"),
            NotificationSeverity::Warning => ("#ffc107", "Warning"),
            NotificationSeverity::Error => ("#dc3545", "Error"),
            NotificationSeverity::Critical => ("#dc3545", "Critical"),
        };

        let mut context_elements = vec![
            json!({
                "type": "mrkdwn",
                "text": format!("*Agent:* {}", payload.agent_name)
            }),
            json!({
                "type": "mrkdwn",
                "text": format!("*Severity:* {}", severity_text)
            }),
        ];

        if let Some(ref run_id) = payload.run_id {
            context_elements.push(json!({
                "type": "mrkdwn",
                "text": format!("*Run:* {}", run_id)
            }));
        }

        let blocks = json!([
            {
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": &payload.title,
                    "emoji": true
                }
            },
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": &payload.body
                }
            },
            {
                "type": "context",
                "elements": context_elements
            }
        ]);

        json!({
            "blocks": blocks,
            "attachments": [{
                "color": color,
                "blocks": []
            }]
        })
    }
}

/// Parse a Slack timestamp string (e.g., "1234567890.123456") into a DateTime<Utc>
fn parse_slack_ts(ts: &str) -> DateTime<Utc> {
    let secs: i64 = ts
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Utc.timestamp_opt(secs, 0).single().unwrap_or_else(|| Utc::now())
}

#[async_trait]
impl ChannelGateway for SlackChannelGateway {
    async fn send_message(
        &self,
        channel_id: &str,
        text: &str,
        thread_id: Option<&str>,
    ) -> Result<(), AgentixError> {
        let mut body = json!({
            "channel": channel_id,
            "text": text,
        });

        if let Some(ts) = thread_id {
            body["thread_ts"] = json!(ts);
        }

        let resp = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentixError::Agent(format!("Slack API error: {}", e)))?;

        if !resp.status().is_success() {
            error!("Slack chat.postMessage failed: {}", resp.status());
            return Err(AgentixError::Agent(format!(
                "Slack API returned status {}",
                resp.status()
            )));
        }

        debug!("Slack message sent to channel {}", channel_id);
        Ok(())
    }

    async fn send_notification(
        &self,
        target: &NotificationTarget,
        payload: &NotificationPayload,
    ) -> Result<(), AgentixError> {
        let formatted = Self::format_notification_blocks(payload);

        let mut body = json!({
            "channel": target.channel_id,
        });

        // Merge formatted blocks into body
        if let Some(blocks) = formatted.get("blocks") {
            body["blocks"] = blocks.clone();
        }
        if let Some(attachments) = formatted.get("attachments") {
            body["attachments"] = attachments.clone();
        }
        body["text"] = json!(format!("{}: {}", payload.title, payload.body));

        if let Some(ref ts) = target.thread_id {
            body["thread_ts"] = json!(ts);
        }

        let resp = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentixError::Agent(format!("Slack notification error: {}", e)))?;

        if !resp.status().is_success() {
            error!("Slack notification failed: {}", resp.status());
            return Err(AgentixError::Agent(format!(
                "Slack notification API returned status {}",
                resp.status()
            )));
        }

        debug!("Slack notification sent to channel {}", target.channel_id);
        Ok(())
    }

    fn platform_type(&self) -> ChannelPlatformType {
        ChannelPlatformType::Slack
    }
}
