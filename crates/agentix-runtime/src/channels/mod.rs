//! Multi-channel gateway adapters for Slack, Telegram, and Discord (Phase 21)

pub mod discord;
pub mod manager;
pub mod slack;
pub mod telegram;

pub use discord::DiscordChannelGateway;
pub use manager::{ChannelGatewayManager, ChannelRouteInfo};
pub use slack::SlackChannelGateway;
pub use telegram::TelegramChannelGateway;
