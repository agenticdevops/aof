//! Channel gateway manager — orchestrates all channel adapters (Phase 21)
//!
//! Routes inbound webhook messages to the correct agent and dispatches
//! outbound notifications to the correct platform.

use agentix_core::channel::{
    ChannelConfig, ChannelDirection, ChannelGateway, ChannelPlatformType, NotificationPayload,
    NotificationTarget,
};
use agentix_core::AgentixError;
use serde::Serialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::{DiscordChannelGateway, SlackChannelGateway, TelegramChannelGateway};

/// Route information for API responses
#[derive(Debug, Clone, Serialize)]
pub struct ChannelRouteInfo {
    pub platform: ChannelPlatformType,
    pub channel_id: String,
    pub agent_name: String,
    pub direction: ChannelDirection,
    pub description: Option<String>,
}

/// Manages all active channel gateways and routes
pub struct ChannelGatewayManager {
    gateways: HashMap<ChannelPlatformType, Box<dyn ChannelGateway>>,
    routes: Vec<(ChannelPlatformType, agentix_core::channel::ChannelRoute)>,
}

impl ChannelGatewayManager {
    /// Create a new ChannelGatewayManager from channel configurations.
    ///
    /// Filters out disabled configs, creates the appropriate gateway for each platform,
    /// and collects all routes.
    pub fn new(configs: Vec<ChannelConfig>) -> Result<Self, AgentixError> {
        let mut gateways: HashMap<ChannelPlatformType, Box<dyn ChannelGateway>> = HashMap::new();
        let mut routes = Vec::new();

        for config in configs {
            if !config.enabled {
                debug!("Skipping disabled channel config for {:?}", config.platform);
                continue;
            }

            if gateways.contains_key(&config.platform) {
                return Err(AgentixError::Config(format!(
                    "Duplicate channel platform: {}. Only one gateway per platform is supported.",
                    config.platform
                )));
            }

            let gateway: Box<dyn ChannelGateway> = match config.platform {
                ChannelPlatformType::Slack => {
                    Box::new(SlackChannelGateway::new(&config.credentials)?)
                }
                ChannelPlatformType::Telegram => {
                    Box::new(TelegramChannelGateway::new(&config.credentials)?)
                }
                ChannelPlatformType::Discord => {
                    Box::new(DiscordChannelGateway::new(&config.credentials)?)
                }
            };

            info!("Initialized {} channel gateway", config.platform);

            // Collect routes with platform association
            for route in &config.routes {
                routes.push((config.platform.clone(), route.clone()));
            }

            gateways.insert(config.platform, gateway);
        }

        info!(
            "ChannelGatewayManager initialized: {} platform(s), {} route(s)",
            gateways.len(),
            routes.len()
        );

        Ok(Self { gateways, routes })
    }

    /// Route an inbound message to an agent name based on platform and channel_id.
    ///
    /// Returns the agent_name if a matching inbound/bidirectional route exists.
    pub fn route_inbound(
        &self,
        platform: ChannelPlatformType,
        channel_id: &str,
    ) -> Option<String> {
        for (route_platform, route) in &self.routes {
            if *route_platform == platform
                && route.channel_id == channel_id
                && matches!(
                    route.direction,
                    ChannelDirection::Inbound | ChannelDirection::Bidirectional
                )
            {
                return Some(route.agent_name.clone());
            }
        }
        None
    }

    /// Send a response message via the appropriate platform gateway.
    pub async fn send_response(
        &self,
        platform: ChannelPlatformType,
        channel_id: &str,
        text: &str,
        thread_id: Option<&str>,
    ) -> Result<(), AgentixError> {
        let gateway = self.gateways.get(&platform).ok_or_else(|| {
            AgentixError::Config(format!("No gateway configured for platform: {}", platform))
        })?;

        gateway.send_message(channel_id, text, thread_id).await
    }

    /// Send a notification via the appropriate platform gateway.
    pub async fn send_notification(
        &self,
        target: &NotificationTarget,
        payload: &NotificationPayload,
    ) -> Result<(), AgentixError> {
        let gateway = self.gateways.get(&target.platform).ok_or_else(|| {
            AgentixError::Config(format!(
                "No gateway configured for platform: {}",
                target.platform
            ))
        })?;

        gateway.send_notification(target, payload).await
    }

    /// Get all configured routes as API-friendly info structs.
    pub fn get_all_routes(&self) -> Vec<ChannelRouteInfo> {
        self.routes
            .iter()
            .map(|(platform, route)| ChannelRouteInfo {
                platform: platform.clone(),
                channel_id: route.channel_id.clone(),
                agent_name: route.agent_name.clone(),
                direction: route.direction.clone(),
                description: route.description.clone(),
            })
            .collect()
    }

    /// Get a reference to the gateway for a specific platform.
    pub fn get_gateway(&self, platform: &ChannelPlatformType) -> Option<&dyn ChannelGateway> {
        self.gateways.get(platform).map(|g| g.as_ref())
    }
}
