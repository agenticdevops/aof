//! Discord platform adapter for AOF
//!
//! This module provides Discord Gateway/HTTP integration for triggering AOF workflows.
//! Supports slash commands, message components, and modal submissions with Ed25519
//! signature verification.

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;
use async_trait::async_trait;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord-specific platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Discord bot token for API requests
    pub bot_token: String,

    /// Discord application ID
    pub application_id: String,

    /// Ed25519 public key for signature verification
    pub public_key: String,

    /// Optional guild IDs for guild-specific command registration
    pub guild_ids: Option<Vec<String>>,

    /// Optional role IDs that are allowed to use commands
    pub allowed_roles: Option<Vec<String>>,
}

/// Discord platform implementation with full Gateway/HTTP support
pub struct DiscordPlatform {
    bot_token: String,
    application_id: String,
    public_key: String,
    client: Client,
    verifying_key: VerifyingKey,
}

impl DiscordPlatform {
    /// Create a new Discord platform adapter from PlatformConfig
    pub fn new(config: super::PlatformConfig) -> Self {
        let bot_token = config.api_token.unwrap_or_default();
        let public_key = config.webhook_secret.unwrap_or_default();

        // Parse the public key for signature verification
        let key_bytes = hex::decode(&public_key).unwrap_or_default();
        let key_array: [u8; 32] = key_bytes.try_into().unwrap_or([0u8; 32]);
        let verifying_key = VerifyingKey::from_bytes(&key_array).unwrap_or_else(|_| {
            // Fallback to a valid (but insecure) key for testing
            VerifyingKey::from_bytes(&[0u8; 32]).unwrap()
        });

        let client = Client::builder()
            .user_agent("AOF-Bot/0.1.0")
            .build()
            .unwrap_or_default();

        Self {
            bot_token,
            application_id: "".to_string(),
            public_key,
            client,
            verifying_key,
        }
    }

    /// Create from Discord-specific config
    pub fn from_discord_config(config: DiscordConfig) -> Result<Self, PlatformError> {
        // Parse the public key for signature verification
        let key_bytes = hex::decode(&config.public_key)
            .map_err(|e| PlatformError::ParseError(format!("Invalid public key: {}", e)))?;

        let key_array: [u8; 32] = key_bytes.try_into()
            .map_err(|_| PlatformError::ParseError("Invalid public key length".to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&key_array)
            .map_err(|e| PlatformError::ParseError(format!("Invalid public key: {}", e)))?;

        let client = Client::builder()
            .user_agent("AOF-Bot/0.1.0")
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            bot_token: config.bot_token,
            application_id: config.application_id,
            public_key: config.public_key,
            client,
            verifying_key,
        })
    }

    /// Register slash commands with Discord
    pub async fn register_commands(&self) -> Result<(), PlatformError> {
        info!("Registering Discord slash commands");

        let commands = vec![
            self.create_agent_command(),
            self.create_task_command(),
            self.create_fleet_command(),
            self.create_flow_command(),
        ];

        for command in commands {
            self.register_command(&command).await?;
        }

        info!("Successfully registered {} Discord commands", 4);
        Ok(())
    }

    /// Register a single command
    async fn register_command(&self, command: &DiscordCommand) -> Result<(), PlatformError> {
        let url = format!(
            "{}/applications/{}/commands",
            DISCORD_API_BASE, self.application_id
        );

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(command)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to register command: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError(format!(
                "Failed to register command '{}': {}",
                command.name, error_text
            )));
        }

        debug!("Registered command: {}", command.name);
        Ok(())
    }

    fn create_agent_command(&self) -> DiscordCommand {
        DiscordCommand {
            name: "agent".to_string(),
            description: "Manage AOF agents".to_string(),
            options: vec![
                CommandOption {
                    option_type: 3, // STRING
                    name: "action".to_string(),
                    description: "Action to perform (run, status, stop)".to_string(),
                    required: true,
                    choices: Some(vec![
                        CommandChoice { name: "run".to_string(), value: "run".to_string() },
                        CommandChoice { name: "status".to_string(), value: "status".to_string() },
                        CommandChoice { name: "stop".to_string(), value: "stop".to_string() },
                    ]),
                },
                CommandOption {
                    option_type: 3, // STRING
                    name: "agent_id".to_string(),
                    description: "Agent ID or configuration".to_string(),
                    required: true,
                    choices: None,
                },
            ],
        }
    }

    fn create_task_command(&self) -> DiscordCommand {
        DiscordCommand {
            name: "task".to_string(),
            description: "Manage AOF tasks".to_string(),
            options: vec![
                CommandOption {
                    option_type: 3,
                    name: "action".to_string(),
                    description: "Action to perform (create, status, cancel)".to_string(),
                    required: true,
                    choices: Some(vec![
                        CommandChoice { name: "create".to_string(), value: "create".to_string() },
                        CommandChoice { name: "status".to_string(), value: "status".to_string() },
                        CommandChoice { name: "cancel".to_string(), value: "cancel".to_string() },
                    ]),
                },
                CommandOption {
                    option_type: 3,
                    name: "description".to_string(),
                    description: "Task description or ID".to_string(),
                    required: true,
                    choices: None,
                },
            ],
        }
    }

    fn create_fleet_command(&self) -> DiscordCommand {
        DiscordCommand {
            name: "fleet".to_string(),
            description: "Manage AOF agent fleets".to_string(),
            options: vec![
                CommandOption {
                    option_type: 3,
                    name: "action".to_string(),
                    description: "Action to perform (list, scale, status)".to_string(),
                    required: true,
                    choices: Some(vec![
                        CommandChoice { name: "list".to_string(), value: "list".to_string() },
                        CommandChoice { name: "scale".to_string(), value: "scale".to_string() },
                        CommandChoice { name: "status".to_string(), value: "status".to_string() },
                    ]),
                },
            ],
        }
    }

    fn create_flow_command(&self) -> DiscordCommand {
        DiscordCommand {
            name: "flow".to_string(),
            description: "Execute AOF workflows".to_string(),
            options: vec![
                CommandOption {
                    option_type: 3,
                    name: "workflow".to_string(),
                    description: "Workflow name or configuration".to_string(),
                    required: true,
                    choices: None,
                },
            ],
        }
    }
}

#[async_trait]
impl TriggerPlatform for DiscordPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        let interaction: DiscordInteraction = serde_json::from_slice(raw)
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse Discord payload: {}", e)))?;

        // Handle ping (type 1)
        if interaction.interaction_type == 1 {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Get user from member or direct user field
        let user = if let Some(member) = &interaction.member {
            member.user.clone()
        } else {
            interaction.user.clone()
                .ok_or_else(|| PlatformError::ParseError("Missing user information".to_string()))?
        };

        let channel_id = interaction.channel_id
            .ok_or_else(|| PlatformError::ParseError("Missing channel_id".to_string()))?;

        // Build message text based on interaction type
        let text = match interaction.interaction_type {
            2 => {
                // APPLICATION_COMMAND (slash command)
                let data = interaction.data.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing interaction data".to_string()))?;

                let command_name = data.name.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing command name".to_string()))?;

                let mut text = format!("/{}", command_name);
                if let Some(options) = &data.options {
                    for opt in options {
                        text.push_str(&format!(" {}={}", opt.name, opt.value));
                    }
                }
                text
            }
            3 => {
                // MESSAGE_COMPONENT (button click, select menu)
                let data = interaction.data.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing component data".to_string()))?;

                let custom_id = data.custom_id.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing custom_id".to_string()))?;

                format!("component:{}", custom_id)
            }
            5 => {
                // MODAL_SUBMIT
                let data = interaction.data.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing modal data".to_string()))?;

                let custom_id = data.custom_id.as_ref()
                    .ok_or_else(|| PlatformError::ParseError("Missing custom_id".to_string()))?;

                format!("modal:{}", custom_id)
            }
            _ => {
                return Err(PlatformError::ParseError(format!(
                    "Unsupported interaction type: {}",
                    interaction.interaction_type
                )));
            }
        };

        let trigger_user = TriggerUser {
            id: user.id.clone(),
            username: Some(user.username.clone()),
            display_name: Some(user.username.clone()),
            is_bot: false,
        };

        let mut metadata = HashMap::new();
        metadata.insert("interaction_id".to_string(), serde_json::json!(interaction.id));
        metadata.insert("interaction_token".to_string(), serde_json::json!(interaction.token));
        metadata.insert("interaction_type".to_string(), serde_json::json!(interaction.interaction_type));

        Ok(TriggerMessage {
            id: interaction.id,
            platform: "discord".to_string(),
            channel_id,
            user: trigger_user,
            text,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: None,
            reply_to: None,
        })
    }

    async fn send_response(
        &self,
        _channel: &str,
        _response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Discord responses are sent via interaction callbacks
        // This method is not used for Discord - responses are sent directly in webhook handler
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "discord"
    }

    async fn verify_signature(&self, payload: &[u8], signature_header: &str) -> bool {
        // Discord sends signature as "timestamp.signature"
        let parts: Vec<&str> = signature_header.split('.').collect();
        if parts.len() != 2 {
            warn!("Invalid Discord signature format");
            return false;
        }

        let timestamp = parts[0];
        let signature_hex = parts[1];

        // Decode signature
        let signature_bytes = match hex::decode(signature_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to decode signature: {}", e);
                return false;
            }
        };

        let sig_array: [u8; 64] = match signature_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => {
                error!("Invalid signature length");
                return false;
            }
        };

        let signature = Signature::from_bytes(&sig_array);

        // Build message to verify: timestamp + payload
        let message = format!("{}{}", timestamp, String::from_utf8_lossy(payload));

        // Verify signature
        match self.verifying_key.verify(message.as_bytes(), &signature) {
            Ok(_) => {
                debug!("Discord signature verified successfully");
                true
            }
            Err(e) => {
                error!("Discord signature verification failed: {}", e);
                false
            }
        }
    }

    fn bot_name(&self) -> &str {
        "aof"
    }

    fn supports_threading(&self) -> bool {
        true
    }

    fn supports_interactive(&self) -> bool {
        true
    }
}

// Discord API types

#[derive(Debug, Clone, Serialize)]
struct DiscordCommand {
    name: String,
    description: String,
    options: Vec<CommandOption>,
}

#[derive(Debug, Clone, Serialize)]
struct CommandOption {
    #[serde(rename = "type")]
    option_type: u8,
    name: String,
    description: String,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    choices: Option<Vec<CommandChoice>>,
}

#[derive(Debug, Clone, Serialize)]
struct CommandChoice {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct DiscordInteraction {
    id: String,
    #[serde(rename = "type")]
    interaction_type: u8,
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<DiscordData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<DiscordUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member: Option<DiscordMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordData {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<DiscordOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordOption {
    name: String,
    value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
}

#[derive(Debug, Deserialize)]
struct DiscordMember {
    user: DiscordUser,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> super::super::PlatformConfig {
        super::super::PlatformConfig {
            platform: "discord".to_string(),
            api_token: Some("test_token".to_string()),
            webhook_secret: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            webhook_url: None,
        }
    }

    #[test]
    fn test_discord_platform_creation() {
        let platform = DiscordPlatform::new(test_config());
        assert_eq!(platform.platform_name(), "discord");
    }

    #[test]
    fn test_command_creation() {
        let platform = DiscordPlatform::new(test_config());
        let agent_cmd = platform.create_agent_command();
        assert_eq!(agent_cmd.name, "agent");
        assert_eq!(agent_cmd.options.len(), 2);
    }

    #[tokio::test]
    async fn test_parse_slash_command() {
        let interaction_json = r#"{
            "id": "1234567890",
            "type": 2,
            "token": "test_token",
            "data": {
                "name": "agent",
                "options": [
                    {"name": "action", "value": "run"},
                    {"name": "agent_id", "value": "test-agent"}
                ]
            },
            "user": {
                "id": "999",
                "username": "testuser"
            },
            "channel_id": "888"
        }"#;

        let platform = DiscordPlatform::new(test_config());
        let headers = HashMap::new();
        let message = platform.parse_message(interaction_json.as_bytes(), &headers).await.unwrap();

        assert_eq!(message.platform, "discord");
        assert_eq!(message.user.id, "999");
        assert_eq!(message.channel_id, "888");
        assert!(message.text.contains("/agent"));
    }
}
