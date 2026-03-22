//! Integration tests for subscription provider factory routing.
//!
//! Verifies that `ProviderFactory::create()` correctly routes to subscription
//! adapters when `provider_mode = "subscription"` is in `ModelConfig.extra`,
//! and that existing API mode routing is completely unchanged (regression).

use agentix_llm::ProviderFactory;
use agentix_core::{ModelConfig, ModelProvider};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_config(provider: ModelProvider, token: &str) -> ModelConfig {
    ModelConfig {
        model: match provider {
            ModelProvider::Anthropic => "claude-3-5-sonnet-20241022".to_string(),
            ModelProvider::OpenAI => "gpt-4o".to_string(),
            ModelProvider::Google => "gemini-2.0-flash".to_string(),
            _ => "test-model".to_string(),
        },
        provider,
        api_key: Some(token.to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(),
    }
}

fn subscription_config(provider: ModelProvider, token: &str) -> ModelConfig {
    let mut config = base_config(provider, token);
    config
        .extra
        .insert("provider_mode".to_string(), serde_json::json!("subscription"));
    config
}

fn api_config(provider: ModelProvider, api_key: &str) -> ModelConfig {
    base_config(provider, api_key)
}

// ---------------------------------------------------------------------------
// Subscription mode routing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn factory_routes_anthropic_subscription_mode() {
    let config = subscription_config(ModelProvider::Anthropic, "oauth-bearer-token");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create Anthropic subscription model");

    // Provider identity is preserved
    assert_eq!(model.provider(), ModelProvider::Anthropic);
}

#[tokio::test]
async fn factory_routes_openai_subscription_mode() {
    let config = subscription_config(ModelProvider::OpenAI, "oauth-access-token");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create OpenAI subscription model");

    assert_eq!(model.provider(), ModelProvider::OpenAI);
}

#[tokio::test]
async fn factory_routes_google_subscription_mode() {
    let config = subscription_config(ModelProvider::Google, "oauth-access-token");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create Google subscription model");

    assert_eq!(model.provider(), ModelProvider::Google);
}

// ---------------------------------------------------------------------------
// Subscription mode returns Box<dyn Model> (trait object check)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscription_provider_implements_model_trait() {
    // Verify each subscription provider returns a proper Box<dyn Model>
    for (provider, token) in [
        (ModelProvider::Anthropic, "anthropic-oauth-token"),
        (ModelProvider::OpenAI, "openai-oauth-token"),
        (ModelProvider::Google, "google-oauth-token"),
    ] {
        let config = subscription_config(provider, token);
        let model = ProviderFactory::create(config)
            .await
            .expect("Should create subscription model");

        // Verify the model config is accessible through the trait object
        let model_config = model.config();
        assert_eq!(model_config.provider, provider);
        assert!(
            model_config.api_key.is_some(),
            "Provider {:?}: api_key should be set from OAuth token",
            provider
        );

        // Verify provider() returns the correct variant
        assert_eq!(
            model.provider(),
            provider,
            "Provider {:?}: provider() should return correct variant",
            provider
        );
    }
}

// ---------------------------------------------------------------------------
// API mode routing — regression tests (existing behavior unchanged)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn factory_routes_anthropic_api_mode() {
    let config = api_config(ModelProvider::Anthropic, "sk-test-key");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create standard Anthropic model");

    assert_eq!(model.provider(), ModelProvider::Anthropic);
}

#[tokio::test]
async fn factory_routes_openai_api_mode() {
    let config = api_config(ModelProvider::OpenAI, "sk-openai-key");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create standard OpenAI model");

    assert_eq!(model.provider(), ModelProvider::OpenAI);
}

#[tokio::test]
async fn factory_routes_google_api_mode() {
    let config = api_config(ModelProvider::Google, "google-api-key");
    let model = ProviderFactory::create(config)
        .await
        .expect("Should create standard Google model");

    assert_eq!(model.provider(), ModelProvider::Google);
}

// ---------------------------------------------------------------------------
// Backward compatibility — no provider_mode field in extra
// ---------------------------------------------------------------------------

#[tokio::test]
async fn backward_compatible_no_mode_field() {
    // Configs without provider_mode in extra should still route correctly
    // and create standard API-key providers

    let anthropic_config = ModelConfig {
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: ModelProvider::Anthropic,
        api_key: Some("sk-test-api-key".to_string()),
        endpoint: None,
        temperature: 0.7,
        max_tokens: Some(4096),
        timeout_secs: 60,
        headers: HashMap::new(),
        extra: HashMap::new(), // Empty extra — no provider_mode key
    };

    let model = ProviderFactory::create(anthropic_config)
        .await
        .expect("Standard Anthropic config (no extra) should still work");

    assert_eq!(model.provider(), ModelProvider::Anthropic);
}

#[tokio::test]
async fn backward_compatible_extra_with_other_keys() {
    // extra with unrelated keys should not affect routing
    let mut config = api_config(ModelProvider::OpenAI, "sk-test-key");
    config
        .extra
        .insert("timeout_override".to_string(), serde_json::json!(120));
    config
        .extra
        .insert("retry_count".to_string(), serde_json::json!(3));
    // Note: no provider_mode key

    let model = ProviderFactory::create(config)
        .await
        .expect("Config with unrelated extra keys should route to standard OpenAI");

    assert_eq!(model.provider(), ModelProvider::OpenAI);
}

// ---------------------------------------------------------------------------
// Subscription mode requires token (missing token = error)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscription_anthropic_requires_token() {
    let mut config = subscription_config(ModelProvider::Anthropic, "placeholder");
    config.api_key = None; // Remove the token

    let result = ProviderFactory::create(config).await;
    assert!(
        result.is_err(),
        "Anthropic subscription without token should fail"
    );
}

#[tokio::test]
async fn subscription_openai_requires_token() {
    let mut config = subscription_config(ModelProvider::OpenAI, "placeholder");
    config.api_key = None;

    let result = ProviderFactory::create(config).await;
    assert!(
        result.is_err(),
        "OpenAI subscription without token should fail"
    );
}

#[tokio::test]
async fn subscription_google_requires_token() {
    let mut config = subscription_config(ModelProvider::Google, "placeholder");
    config.api_key = None;

    let result = ProviderFactory::create(config).await;
    assert!(
        result.is_err(),
        "Google subscription without token should fail"
    );
}

// ---------------------------------------------------------------------------
// Explicit "api" mode in extra — should behave same as absent mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn explicit_api_mode_routes_to_standard_provider() {
    let mut config = api_config(ModelProvider::Anthropic, "sk-test-key");
    config
        .extra
        .insert("provider_mode".to_string(), serde_json::json!("api"));

    let model = ProviderFactory::create(config)
        .await
        .expect("Explicit api mode should route to standard Anthropic provider");

    assert_eq!(model.provider(), ModelProvider::Anthropic);
}
