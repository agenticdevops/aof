// OpenAgentiX Core - Foundation types and traits for the OpenAgentiX framework
//
// This crate provides zero-cost abstractions for building high-performance
// agentic systems for enterprise automation workflows.

pub mod agent;
pub mod config;
pub mod context;
pub mod error;
pub mod error_tracker;
pub mod mcp;
pub mod memory;
pub mod model;
pub mod registry;
pub mod schema;
pub mod tool;
pub mod trigger;

// Re-export v1 spec types
pub use agent::{
    AgentSpec, AgentMetadata, AgentSpecInner, AgentMode, ToolEntry, SpecToolType,
    McpServerEntry, McpTransportType, NotificationEntry, NotificationChannel,
};
// Re-export legacy v1.0 types (still used by agentix-runtime)
pub use agent::{
    Agent, AgentConfig, AgentContext, ExecutionMetadata, HttpToolConfig,
    LegacyAgentMetadata, MemorySpec, Message, MessageRole, OutputSchemaSpec, QualifiedToolSpec,
    RoutingConfig, ShellToolConfig, StructuredMemoryConfig, ToolResult as AgentToolResult,
    ToolSource, ToolSpec, TypeBasedToolSpec, TypeBasedToolType,
};
pub use error::{AgentixError, AgentixResult};
// Backward-compatible aliases
pub use error::{AofError, AofResult};
pub use error_tracker::{ErrorKnowledgeBase, ErrorRecord, ErrorStats};
pub use config::{
    WorkspaceConfig, WorkspaceMetadata, WorkspaceSpec, WorkspaceDefaults, ProviderConfig, GatewayConfig,
};
pub use mcp::{McpServerConfig, McpTransport};
pub use memory::{Memory, MemoryBackend, MemoryEntry, MemoryQuery};
pub use model::{
    Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, RequestMessage, StopReason,
    StreamChunk, ToolDefinition as ModelToolDefinition, Usage,
};
pub use tool::{
    Tool, ToolCall, ToolConfig, ToolDefinition, ToolExecutor, ToolInput, ToolResult, ToolType,
};
pub use context::{
    ApprovalConfig, AuditConfig, AuditEvent, Context, ContextMetadata, ContextSpec,
    LimitsConfig, SecretRef,
};
pub use registry::{
    AgentRegistry, ContextRegistry, Registry, ResourceLoadSummary, ResourceManager, TriggerRegistry,
};
pub use trigger::{
    CommandBinding, StandaloneTriggerConfig, StandaloneTriggerType, Trigger, TriggerMetadata,
    TriggerSpec,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default context window size (tokens)
pub const DEFAULT_CONTEXT_WINDOW: usize = 100_000;

/// Maximum parallel tool calls
pub const MAX_PARALLEL_TOOLS: usize = 10;
