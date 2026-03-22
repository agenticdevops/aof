use agentix_core::{AofError, AofResult, Model, ModelConfig, ModelProvider};
use agentix_core::ProviderMode;

pub mod anthropic;
pub mod google;
pub mod openai;
pub mod subscription;

#[cfg(feature = "bedrock")]
pub mod bedrock;

/// LLM provider trait
pub trait LlmProvider {
    fn create(config: ModelConfig) -> AofResult<Box<dyn Model>>;
}

/// Provider factory
///
/// Routes to the correct provider implementation based on `ModelConfig.provider`
/// and an optional `provider_mode` in `ModelConfig.extra`.
///
/// # Subscription routing
///
/// When `extra["provider_mode"]` is `"subscription"`, the factory creates a
/// subscription adapter that uses OAuth Bearer tokens instead of API keys.
/// The OAuth token must be present in `ModelConfig.api_key`.
///
/// ```json
/// { "provider_mode": "subscription" }
/// ```
pub struct ProviderFactory;

impl ProviderFactory {
    pub async fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        // Check for subscription mode in config.extra
        let mode = config
            .extra
            .get("provider_mode")
            .and_then(|v| serde_json::from_value::<ProviderMode>(v.clone()).ok());

        match (config.provider, mode) {
            // --- Subscription mode routing ---
            (ModelProvider::Anthropic, Some(ProviderMode::Subscription)) => {
                subscription::AnthropicSubscriptionProvider::create(config)
            }
            (ModelProvider::OpenAI, Some(ProviderMode::Subscription)) => {
                subscription::OpenAISubscriptionProvider::create(config)
            }
            (ModelProvider::Google, Some(ProviderMode::Subscription)) => {
                subscription::GoogleSubscriptionProvider::create(config)
            }

            // --- Standard API key routing (existing, unchanged) ---
            (ModelProvider::Anthropic, _) => anthropic::AnthropicProvider::create(config),
            (ModelProvider::OpenAI, _) => openai::OpenAIProvider::create(config),
            (ModelProvider::Google, _) => google::GoogleProvider::create(config),
            (ModelProvider::Groq, _) => {
                // Groq uses OpenAI-compatible API with different endpoint
                let mut groq_config = config;
                if groq_config.endpoint.is_none() {
                    groq_config.endpoint = Some("https://api.groq.com/openai/v1".to_string());
                }
                if groq_config.api_key.is_none() {
                    groq_config.api_key = std::env::var("GROQ_API_KEY").ok();
                }
                openai::OpenAIProvider::create(groq_config)
            }
            #[cfg(feature = "bedrock")]
            (ModelProvider::Bedrock, _) => bedrock::BedrockProvider::create(config).await,
            #[cfg(not(feature = "bedrock"))]
            (ModelProvider::Bedrock, _) => Err(AofError::config(
                "Bedrock provider not enabled - requires 'bedrock' feature",
            )),
            (ModelProvider::Ollama, _) => {
                // Ollama uses OpenAI-compatible API at localhost
                let mut ollama_config = config;
                if ollama_config.endpoint.is_none() {
                    ollama_config.endpoint = Some(
                        std::env::var("OLLAMA_HOST")
                            .unwrap_or_else(|_| "http://localhost:11434/v1".to_string()),
                    );
                }
                // Ollama doesn't require API key
                if ollama_config.api_key.is_none() {
                    ollama_config.api_key = Some("ollama".to_string());
                }
                openai::OpenAIProvider::create(ollama_config)
            }
            (ModelProvider::Azure, _) => {
                Err(AofError::config("Azure provider not yet implemented"))
            }
            (ModelProvider::Custom, _) => Err(AofError::config(
                "Custom provider requires manual implementation",
            )),
        }
    }
}
