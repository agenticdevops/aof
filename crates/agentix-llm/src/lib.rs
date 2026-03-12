// OpenAgentiX LLM - Multi-provider LLM abstraction
//
// Supports: Anthropic, OpenAI, Bedrock, Azure, Ollama
// Optimized for minimal allocations and fast streaming

pub mod provider;
pub mod stream;

pub use provider::{LlmProvider, ProviderFactory};

// Re-export from agentix-core
pub use agentix_core::{
    AgentixError, AgentixResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse,
    StopReason, StreamChunk, Usage,
};
// Backward-compatible aliases
pub use agentix_core::{AofError, AofResult};

/// Create model from configuration
pub async fn create_model(config: ModelConfig) -> AgentixResult<Box<dyn Model>> {
    ProviderFactory::create(config).await
}
