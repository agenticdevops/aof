use aof_core::{AofError, AofResult, Model, ModelConfig, ModelProvider};

pub mod anthropic;
pub mod google;
pub mod openai;

#[cfg(feature = "bedrock")]
pub mod bedrock;

/// LLM provider trait
pub trait LlmProvider {
    fn create(config: ModelConfig) -> AofResult<Box<dyn Model>>;
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    pub async fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        match config.provider {
            ModelProvider::Anthropic => anthropic::AnthropicProvider::create(config),
            ModelProvider::OpenAI => openai::OpenAIProvider::create(config),
            ModelProvider::Google => google::GoogleProvider::create(config),
            ModelProvider::Groq => {
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
            ModelProvider::Bedrock => bedrock::BedrockProvider::create(config).await,
            #[cfg(not(feature = "bedrock"))]
            ModelProvider::Bedrock => Err(AofError::config("Bedrock provider not enabled - requires 'bedrock' feature")),
            ModelProvider::Ollama => {
                // Ollama uses OpenAI-compatible API at localhost
                let mut ollama_config = config;
                if ollama_config.endpoint.is_none() {
                    ollama_config.endpoint = Some(
                        std::env::var("OLLAMA_HOST")
                            .unwrap_or_else(|_| "http://localhost:11434/v1".to_string())
                    );
                }
                // Ollama doesn't require API key
                if ollama_config.api_key.is_none() {
                    ollama_config.api_key = Some("ollama".to_string());
                }
                openai::OpenAIProvider::create(ollama_config)
            }
            ModelProvider::Azure => Err(AofError::config("Azure provider not yet implemented")),
            ModelProvider::Custom => Err(AofError::config("Custom provider requires manual implementation")),
        }
    }
}
