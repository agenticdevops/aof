//! Multi-channel gateway adapters for Slack, Telegram, and Discord (Phase 21)

pub mod slack;
pub mod telegram;
pub mod discord;

pub use slack::SlackChannelGateway;
pub use telegram::TelegramChannelGateway;
pub use discord::DiscordChannelGateway;
