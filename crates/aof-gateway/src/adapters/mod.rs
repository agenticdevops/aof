//! Channel adapters for messaging platforms
//!
//! This module provides the platform-agnostic ChannelAdapter trait and common types
//! used by all messaging platform adapters.

pub mod channel_adapter;

pub use channel_adapter::{
    ChannelAdapter, Platform, InboundMessage, AgentResponse, MessageUser, Attachment,
};
