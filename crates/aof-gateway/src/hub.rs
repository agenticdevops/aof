//! Gateway hub control plane
//!
//! This module implements the central control plane that manages channel adapters,
//! routes messages, and coordinates with the agent runtime.

use std::collections::HashMap;

use tokio::sync::{broadcast, watch};
use uuid::Uuid;

use aof_core::{AofError, CoordinationEvent};
use crate::adapters::{ChannelAdapter, Platform, AgentResponse};
use crate::rate_limiter::RateLimiter;
use crate::broadcast::{BroadcastMessage, BroadcastTarget, BroadcastResult};
use crate::config::GatewayConfig;

/// Gateway hub control plane
pub struct GatewayHub {
    /// Session ID for this gateway instance (UUID, generated once)
    session_id: String,

    /// Registered channel adapters (keyed by adapter_id)
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,

    /// Rate limiters per platform
    rate_limiters: HashMap<Platform, RateLimiter>,

    /// Event sender to agent runtime (Phase 1 broadcast channel)
    event_tx: broadcast::Sender<CoordinationEvent>,

    /// Shutdown signal
    shutdown_rx: watch::Receiver<bool>,

    /// Gateway configuration (optional, for squad broadcast)
    config: Option<GatewayConfig>,
}

impl GatewayHub {
    /// Create new gateway hub
    pub fn new(
        event_tx: broadcast::Sender<CoordinationEvent>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        let session_id = Uuid::new_v4().to_string();

        Self {
            session_id,
            adapters: HashMap::new(),
            rate_limiters: HashMap::new(),
            event_tx,
            shutdown_rx,
            config: None,
        }
    }

    /// Set gateway configuration (required for squad broadcast)
    pub fn set_config(&mut self, config: GatewayConfig) {
        self.config = Some(config);
    }

    /// Register a channel adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn ChannelAdapter>) {
        let adapter_id = adapter.adapter_id().to_string();
        let platform = adapter.platform();

        // Create rate limiter for platform if not exists
        if !self.rate_limiters.contains_key(&platform) {
            let config = RateLimiter::default_config_for_platform(platform);
            self.rate_limiters.insert(platform, RateLimiter::new(platform, config));
        }

        self.adapters.insert(adapter_id, adapter);
    }

    /// Start all registered adapters
    pub async fn start(&mut self) -> Result<(), AofError> {
        tracing::info!(
            session_id = %self.session_id,
            adapter_count = self.adapters.len(),
            "Starting gateway hub"
        );

        for (adapter_id, adapter) in self.adapters.iter_mut() {
            tracing::info!(adapter_id = %adapter_id, "Starting adapter");
            adapter.start().await?;
        }

        Ok(())
    }

    /// Run gateway event loop (receive messages, translate, route to runtime)
    pub async fn run(&mut self) -> Result<(), AofError> {
        tracing::info!("Gateway hub event loop started");

        // For now, just a placeholder event loop
        // In task 03-01-09 (integration test), we'll implement the full select! loop
        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("Shutdown signal received");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop all adapters gracefully
    pub async fn stop(&mut self) -> Result<(), AofError> {
        tracing::info!("Stopping all adapters");

        for (adapter_id, adapter) in self.adapters.iter_mut() {
            let adapter_id = adapter_id.clone();
            tracing::info!(adapter_id = %adapter_id, "Stopping adapter");

            // Stop adapter (can't use tokio::join! with mutable borrows)
            if let Err(e) = adapter.stop().await {
                tracing::error!(adapter_id = %adapter_id, error = ?e, "Failed to stop adapter");
            }
        }

        Ok(())
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Broadcast message to target agents/channels
    pub async fn broadcast(
        &self,
        message: BroadcastMessage,
    ) -> Result<BroadcastResult, AofError> {
        // Resolve target to list of agent IDs
        let agents = self.resolve_broadcast_target(&message.target)?;

        let mut sent_count = 0;
        let mut failed_channels = Vec::new();

        for agent_id in agents {
            // Get channels for agent (from squad config)
            let channels = self.get_agent_channels(&agent_id)?;

            for (platform, channel_id) in channels {
                // Get adapter for platform
                let adapter = match self.get_adapter_for_platform(platform) {
                    Some(adapter) => adapter,
                    None => {
                        tracing::warn!(
                            agent_id = %agent_id,
                            platform = ?platform,
                            "No adapter found for platform"
                        );
                        failed_channels.push((platform, channel_id));
                        continue;
                    }
                };

                // Send message via adapter
                let response = AgentResponse {
                    agent_id: agent_id.clone(),
                    content: message.content.clone(),
                    target_platform: platform,
                    target_channel: channel_id.clone(),
                    thread_id: None,
                };

                match adapter.send_message(&response).await {
                    Ok(_) => sent_count += 1,
                    Err(e) => {
                        tracing::warn!(
                            agent_id = %agent_id,
                            platform = ?platform,
                            channel_id = %channel_id,
                            error = %e,
                            "Failed to broadcast to channel"
                        );
                        failed_channels.push((platform, channel_id));
                    }
                }
            }
        }

        Ok(BroadcastResult {
            sent_count,
            failed_channels,
        })
    }

    /// Resolve broadcast target to list of agent IDs
    fn resolve_broadcast_target(
        &self,
        target: &BroadcastTarget,
    ) -> Result<Vec<String>, AofError> {
        let config = self.config.as_ref().ok_or_else(|| {
            AofError::config("Gateway config not set (required for squad broadcast)")
        })?;

        match target {
            BroadcastTarget::AllAgents => {
                // Get all agents from all squads
                let agents: Vec<String> = config
                    .spec
                    .squads
                    .iter()
                    .flat_map(|s| s.agents.clone())
                    .collect();
                Ok(agents)
            }
            BroadcastTarget::Squad(name) => {
                // Get agents from specific squad
                config
                    .get_squad_agents(name)
                    .ok_or_else(|| AofError::config(format!("Squad not found: {}", name)))
            }
            BroadcastTarget::Agents(ids) => {
                // Use specific agent IDs
                Ok(ids.clone())
            }
            BroadcastTarget::Channel { platform, channel_id } => {
                // Get agents subscribed to this channel
                Ok(self.get_agents_for_channel(*platform, channel_id))
            }
        }
    }

    /// Get channels for agent (from squad config)
    fn get_agent_channels(&self, agent_id: &str) -> Result<Vec<(Platform, String)>, AofError> {
        let config = self.config.as_ref().ok_or_else(|| {
            AofError::config("Gateway config not set")
        })?;

        let mut channels = Vec::new();

        // Find squads containing this agent
        for squad in &config.spec.squads {
            if squad.agents.contains(&agent_id.to_string()) {
                // Add all configured channels from this squad
                if let Some(ref slack_id) = squad.channels.slack {
                    channels.push((Platform::Slack, slack_id.clone()));
                }
                if let Some(ref discord_id) = squad.channels.discord {
                    channels.push((Platform::Discord, discord_id.clone()));
                }
                if let Some(ref telegram_id) = squad.channels.telegram {
                    channels.push((Platform::Telegram, telegram_id.clone()));
                }
                if let Some(ref whatsapp_id) = squad.channels.whatsapp {
                    channels.push((Platform::WhatsApp, whatsapp_id.clone()));
                }
            }
        }

        Ok(channels)
    }

    /// Get agents subscribed to specific channel
    fn get_agents_for_channel(&self, platform: Platform, channel_id: &str) -> Vec<String> {
        let config = match &self.config {
            Some(c) => c,
            None => return vec![],
        };

        let mut agents = Vec::new();

        for squad in &config.spec.squads {
            let has_channel = match platform {
                Platform::Slack => {
                    squad.channels.slack.as_ref().map_or(false, |id| id == channel_id)
                }
                Platform::Discord => {
                    squad.channels.discord.as_ref().map_or(false, |id| id == channel_id)
                }
                Platform::Telegram => {
                    squad.channels.telegram.as_ref().map_or(false, |id| id == channel_id)
                }
                Platform::WhatsApp => {
                    squad.channels.whatsapp.as_ref().map_or(false, |id| id == channel_id)
                }
            };

            if has_channel {
                agents.extend(squad.agents.clone());
            }
        }

        agents
    }

    /// Get adapter for platform (returns first registered adapter for platform)
    fn get_adapter_for_platform(&self, platform: Platform) -> Option<&Box<dyn ChannelAdapter>> {
        self.adapters
            .values()
            .find(|adapter| adapter.platform() == platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::adapters::{InboundMessage, AgentResponse, MessageUser};
    use chrono::Utc;
    use serde_json::json;

    struct MockAdapter {
        id: String,
        platform: Platform,
        started: bool,
        stopped: bool,
    }

    #[async_trait]
    impl ChannelAdapter for MockAdapter {
        fn adapter_id(&self) -> &str {
            &self.id
        }

        fn platform(&self) -> Platform {
            self.platform
        }

        async fn start(&mut self) -> Result<(), AofError> {
            self.started = true;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), AofError> {
            self.stopped = true;
            Ok(())
        }

        async fn health_check(&self) -> Result<bool, AofError> {
            Ok(true)
        }

        async fn receive_message(&mut self) -> Result<InboundMessage, AofError> {
            Err(AofError::runtime("No messages"))
        }

        async fn send_message(&self, _response: &AgentResponse) -> Result<(), AofError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_hub_start_stop() {
        let (event_tx, _event_rx) = broadcast::channel(10);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        let mut hub = GatewayHub::new(event_tx, shutdown_rx);

        // Register mock adapter
        let adapter = Box::new(MockAdapter {
            id: "test-slack".to_string(),
            platform: Platform::Slack,
            started: false,
            stopped: false,
        });
        hub.register_adapter(adapter);

        // Start hub
        assert!(hub.start().await.is_ok());

        // Stop hub
        assert!(hub.stop().await.is_ok());
    }

    #[test]
    fn test_hub_session_id() {
        let (event_tx, _event_rx) = broadcast::channel(10);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        let hub = GatewayHub::new(event_tx, shutdown_rx);

        // Session ID should be UUID format
        assert!(!hub.session_id().is_empty());
        assert_eq!(hub.session_id().len(), 36); // UUID format
    }
}
