//! # AOF Gateway - Messaging Gateway Hub
//!
//! The `aof-gateway` crate provides a hub-and-spoke messaging gateway that connects
//! multiple messaging platforms (Slack, Discord, Telegram, WhatsApp) to the AOF agent runtime.
//!
//! ## Architecture
//!
//! The gateway follows an enterprise integration hub-and-spoke pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                       AOF MESSAGING GATEWAY                          │
//! │                                                                       │
//! │  ┌─────────────────────────────────────────────────────────────┐   │
//! │  │                    GATEWAY HUB (Control Plane)               │   │
//! │  │  - Message routing                                           │   │
//! │  │  - Event translation (Platform → CoordinationEvent)          │   │
//! │  │  - Rate limiting (per-platform token buckets)                │   │
//! │  │  - Adapter lifecycle management                              │   │
//! │  │  - Connection to agent runtime via broadcast channel         │   │
//! │  └──────────┬──────────────┬──────────────┬──────────────┬──────┘   │
//! │             │              │              │              │           │
//! │  ┌──────────▼─────┐  ┌────▼────┐  ┌──────▼──────┐  ┌───▼──────┐   │
//! │  │ Slack Adapter  │  │ Discord │  │ Telegram    │  │ WhatsApp │   │
//! │  │ (Socket Mode)  │  │ (Gateway)│ │ (Polling)   │  │ (Future) │   │
//! │  └────────┬───────┘  └────┬─────┘  └──────┬──────┘  └────┬─────┘   │
//! │           │               │               │              │          │
//! └───────────┼───────────────┼───────────────┼──────────────┼──────────┘
//!             │               │               │              │
//!             ▼               ▼               ▼              ▼
//!     NAT-TRANSPARENT (outbound WebSocket/polling)
//! ```
//!
//! ## Core Components
//!
//! - **GatewayHub**: Central control plane that manages adapters, routes messages, and coordinates with agent runtime
//! - **ChannelAdapter**: Platform-agnostic trait for messaging platform adapters
//! - **Event Translation**: Normalizes platform-specific messages to standard `CoordinationEvent` format
//! - **Rate Limiting**: Token bucket (GCRA) algorithm per platform to prevent API rate limits
//! - **Configuration**: YAML-based gateway configuration with environment variable substitution
//!
//! ## Key Features
//!
//! - **NAT-transparent**: All connections are outbound (WebSocket/polling), no ngrok needed
//! - **Platform-agnostic**: Unified interface for all messaging platforms via ChannelAdapter trait
//! - **Rate limiting**: Automatic rate limiting per platform to prevent 429 errors
//! - **Event normalization**: All platforms map to standard CoordinationEvent format
//! - **Lifecycle management**: Start/stop adapters gracefully, health checks
//!
//! ## Usage
//!
//! ```rust,no_run
//! use aof_gateway::{GatewayHub, config::load_gateway_config};
//! use tokio::sync::broadcast;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = load_gateway_config("gateway.yaml")?;
//!
//!     // Create event channel for agent runtime
//!     let (event_tx, _event_rx) = broadcast::channel(1000);
//!
//!     // Create shutdown signal
//!     let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
//!
//!     // Initialize gateway hub
//!     let mut hub = GatewayHub::new(event_tx, shutdown_rx);
//!
//!     // Register adapters (from config)
//!     // hub.register_adapter(Box::new(slack_adapter));
//!
//!     // Start gateway
//!     hub.start().await?;
//!
//!     // Run event loop
//!     hub.run().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod adapters;
pub mod config;
pub mod hub;
pub mod rate_limiter;
pub mod translation;

pub use hub::GatewayHub;
pub use adapters::channel_adapter::{ChannelAdapter, Platform, InboundMessage, AgentResponse, MessageUser, Attachment};
pub use rate_limiter::{RateLimiter, RateLimitConfig};
pub use config::GatewayConfig;
