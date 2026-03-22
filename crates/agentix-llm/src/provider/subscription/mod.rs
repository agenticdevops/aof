//! Subscription-mode LLM provider adapters.
//!
//! These adapters implement the `Model` trait using OAuth Bearer token
//! authentication instead of static API keys. The key architectural insight:
//! subscription providers use the **same API endpoints** as their API-key
//! counterparts but with different auth headers.
//!
//! # Token Lifecycle
//!
//! Adapters do NOT handle token refresh. The caller (AuthService, plan 02)
//! ensures a valid token exists before constructing the model. If the token
//! expires mid-run, the HTTP 401 propagates as an `AofError::Model` with
//! a descriptive message indicating the token may have expired.
//!
//! # Usage
//!
//! Set `provider_mode = "subscription"` in `ModelConfig.extra` and place the
//! OAuth access token in `ModelConfig.api_key`:
//!
//! ```rust,ignore
//! use std::collections::HashMap;
//! use agentix_core::{ModelConfig, ModelProvider};
//!
//! let mut extra = HashMap::new();
//! extra.insert("provider_mode".to_string(), serde_json::json!("subscription"));
//!
//! let config = ModelConfig {
//!     provider: ModelProvider::Anthropic,
//!     api_key: Some("oauth-access-token".to_string()),
//!     extra,
//!     ..Default::default()
//! };
//! ```

pub mod anthropic_sub;
pub mod google_sub;
pub mod openai_sub;

pub use anthropic_sub::AnthropicSubscriptionProvider;
pub use google_sub::GoogleSubscriptionProvider;
pub use openai_sub::OpenAISubscriptionProvider;
