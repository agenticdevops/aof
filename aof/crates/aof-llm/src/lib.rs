// AOF LLM - Multi-provider LLM abstraction
//
// Supports: Anthropic, OpenAI, Bedrock, Azure, Ollama
// Optimized for minimal allocations and fast streaming

pub mod provider;
pub mod stream;

pub use provider::{LlmProvider, ProviderFactory};

// Re-export from aof-core
pub use aof_core::{
    AofError, AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse,
    StopReason, StreamChunk, Usage,
};

/// Create model from configuration
pub async fn create_model(config: ModelConfig) -> AofResult<Box<dyn Model>> {
    ProviderFactory::create(config).await
}
