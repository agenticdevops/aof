//! OpenAI subscription provider.
//!
//! OpenAI already uses `Authorization: Bearer {token}` for its standard API.
//! The subscription variant is identical — it uses the same
//! `https://api.openai.com/v1/chat/completions` endpoint but with an OAuth
//! access token (from auth.openai.com) instead of an `sk-` API key.
//!
//! Because the auth format is identical, this adapter delegates directly to
//! the standard `OpenAIProvider::create()`. The `ModelConfig.api_key` field
//! holds the OAuth access token; the standard provider puts it in the
//! `Authorization: Bearer` header, which is exactly what we need.
//!
//! On HTTP 401: returns `AofError::Model` indicating the subscription token
//! may have expired. The caller must re-authenticate.

use agentix_core::{AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, StreamChunk};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use super::super::openai::OpenAIProvider;

/// OpenAI subscription provider — OAuth Bearer token to the official API.
///
/// Delegates to the standard `OpenAIProvider` since OpenAI already uses
/// Bearer auth. The only difference is the token origin (OAuth vs sk- key).
pub struct OpenAISubscriptionProvider;

impl OpenAISubscriptionProvider {
    /// Create an OpenAI subscription model.
    ///
    /// `config.api_key` must contain a valid OAuth access token obtained via
    /// the OpenAI auth.openai.com PKCE flow.
    pub fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        // Validate token presence before delegating
        if config.api_key.is_none() {
            return Err(AofError::config(
                "OpenAI subscription mode requires an OAuth access token in api_key field",
            ));
        }

        // The standard OpenAI provider already uses Bearer auth.
        // Wrapping it in a subscription model provides 401 detection with
        // a user-friendly message.
        let inner = OpenAIProvider::create(config.clone())?;
        Ok(Box::new(OpenAISubscriptionModel { config, inner }))
    }
}

/// Wraps a standard OpenAI model to intercept 401 errors with subscription-
/// specific error messages.
struct OpenAISubscriptionModel {
    config: ModelConfig,
    inner: Box<dyn Model>,
}

#[async_trait]
impl Model for OpenAISubscriptionModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        self.inner.generate(request).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("401") || msg.contains("Unauthorized") || msg.contains("unauthorized") {
                AofError::model(
                    "OpenAI subscription token is invalid or expired. \
                     Re-authenticate via `agentix auth login openai`.",
                )
            } else {
                e
            }
        })
    }

    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>> {
        self.inner.generate_stream(request).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("401") || msg.contains("Unauthorized") || msg.contains("unauthorized") {
                AofError::model(
                    "OpenAI subscription token is invalid or expired. \
                     Re-authenticate via `agentix auth login openai`.",
                )
            } else {
                e
            }
        })
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::OpenAI
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.inner.count_tokens(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config(token: Option<&str>) -> ModelConfig {
        ModelConfig {
            model: "gpt-4o".to_string(),
            provider: ModelProvider::OpenAI,
            api_key: token.map(|t| t.to_string()),
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn create_succeeds_with_token() {
        let config = make_config(Some("oauth-access-token"));
        let result = OpenAISubscriptionProvider::create(config);
        assert!(result.is_ok(), "Should create provider with valid token");
    }

    #[test]
    fn create_fails_without_token() {
        let config = make_config(None);
        let result = OpenAISubscriptionProvider::create(config);
        assert!(result.is_err(), "Should fail without OAuth token");
        let err = result.err().unwrap().to_string();
        assert!(err.contains("api_key"), "Error should mention api_key field");
    }

    #[test]
    fn provider_returns_openai() {
        let config = make_config(Some("test-token"));
        let model = OpenAISubscriptionProvider::create(config).unwrap();
        assert_eq!(model.provider(), ModelProvider::OpenAI);
    }

    #[test]
    fn config_is_accessible() {
        let config = make_config(Some("test-token"));
        let model = OpenAISubscriptionProvider::create(config.clone()).unwrap();
        assert_eq!(model.config().model, "gpt-4o");
        assert_eq!(model.config().provider, ModelProvider::OpenAI);
    }

    #[test]
    fn token_counting_is_delegated() {
        let config = make_config(Some("test-token"));
        let model = OpenAISubscriptionProvider::create(config).unwrap();
        let text = "Hello, world!";
        let tokens = model.count_tokens(text);
        // OpenAI uses ~4 chars per token; "Hello, world!" ≈ 3-4 tokens
        assert!(tokens >= 3 && tokens <= 5, "Token count should be ~3-4 for short text");
    }
}
