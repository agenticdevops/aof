// AOF Core - Foundation types and traits for the Agentic Ops Framework
//
// This crate provides zero-cost abstractions for building high-performance
// agentic systems targeting DevOps and SRE workflows.

pub mod agent;
pub mod error;
pub mod error_tracker;
pub mod memory;
pub mod model;
pub mod tool;

// Re-export core types
pub use agent::{
    Agent, AgentConfig, AgentContext, AgentMetadata, ExecutionMetadata, Message, MessageRole,
    ToolResult as AgentToolResult,
};
pub use error::{AofError, AofResult};
pub use error_tracker::{ErrorKnowledgeBase, ErrorRecord, ErrorStats};
pub use memory::{Memory, MemoryBackend, MemoryEntry, MemoryQuery};
pub use model::{
    Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, RequestMessage, StopReason,
    StreamChunk, ToolDefinition as ModelToolDefinition, Usage,
};
pub use tool::{
    Tool, ToolCall, ToolConfig, ToolDefinition, ToolExecutor, ToolInput, ToolResult, ToolType,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default context window size (tokens)
pub const DEFAULT_CONTEXT_WINDOW: usize = 100_000;

/// Maximum parallel tool calls
pub const MAX_PARALLEL_TOOLS: usize = 10;
