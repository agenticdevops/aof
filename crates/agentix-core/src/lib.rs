// OpenAgentiX Core - Foundation types and traits for the OpenAgentiX framework
//
// This crate provides zero-cost abstractions for building high-performance
// agentic systems for enterprise automation workflows.

pub mod agent;
pub mod approval;
pub mod auth;
pub mod channel;
pub mod config;
pub mod context;
pub mod coordination;
pub mod cost;
pub mod telemetry;
pub mod vector_memory;
pub mod error;
pub mod error_tracker;
pub mod mcp;
pub mod memory;
pub mod model;
pub mod registry;
pub mod schema;
pub mod secret_store;
pub mod security;
pub mod skills;
pub mod tool;
pub mod trigger;
pub mod trigger_event;

// Re-export core types
pub use agent::{
    Agent, AgentConfig, AgentContext, AgentMetadata, ExecutionMetadata, HttpToolConfig,
    MemorySpec, Message, MessageRole, OutputSchemaSpec, QualifiedToolSpec, RoutingConfig,
    ShellToolConfig, StructuredMemoryConfig, ToolResult as AgentToolResult, ToolSource, ToolSpec,
    TypeBasedToolSpec, TypeBasedToolType,
};
pub use error::{AgentixError, AgentixResult};
// Backward-compatible aliases
pub use error::{AofError, AofResult};
pub use error_tracker::{ErrorKnowledgeBase, ErrorRecord, ErrorStats};
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
pub use trigger_event::{TriggerEvent, TriggerRunRegistry, TriggerSource, TriggerTrait};
pub use trigger::{
    CommandBinding, StandaloneTriggerConfig, StandaloneTriggerType, Trigger, TriggerMetadata,
    TriggerSpec,
};

// GitAgent-compatible types (v0.1.0 directory format)
pub use agent::{
    AgentDefinition, AgentDependency, AgentFormat, AgentLoader, AgentManifest, AgentMode,
    AgentModelConfig, DirectoryLoader, DirectoryToolType, FlatYamlLoader, MemoryBackendType,
    ResearchPhaseConfig, SkillEntry, ToolEntry, VectorMemoryConfig,
};
pub use config::{
    GatewayConfig, OAuthConfig, ProviderConfig, ProviderMode, TelemetryConfig, WorkspaceConfig,
    WorkspaceDefaults, WorkspaceMetadata, WorkspaceSpec,
};
pub use skills::{BuiltinSkillPack, SkillRegistry};
pub use coordination::{
    AgentInbox, CoordinatorProtocol, DelegationMessage, DelegationResult, DelegationStatus,
    INBOX_CAPACITY,
};
pub use vector_memory::{cosine_similarity, MemoryMatch, VectorEntry, VectorMemoryBackend};
pub use cost::{
    BudgetConfig, BudgetStopReason, CostRecord, CostSummary, ModelComplexityScore, ModelPricing,
    ModelTier, RunCostSummary, calculate_cost, default_model_pricing,
};
pub use telemetry::{
    LogLevel, SpanKind, SpanRecord, SpanStatus, StructuredLogEntry, TraceContext,
};
pub use secret_store::{EncryptedSecret, SecretError, SecretRedactor, SecretStore};
pub use security::{CapabilityPolicy, SecurityConfig, SsrfGuard, SsrfViolation};
pub use approval::{
    ApprovalAction, ApprovalDecision, ApprovalPolicy, ApprovalRequest, ApprovalStatus,
    AutonomyMode,
};
pub use channel::{
    ChannelConfig, ChannelCredentials, ChannelDirection, ChannelGateway, ChannelMessage,
    ChannelPlatformType, ChannelRoute, NotificationPayload, NotificationSeverity,
    NotificationTarget,
};
pub use auth::{
    AuthProfile, AuthProfileKind, AuthProfilesData, AuthProfilesStore, PkceState, TokenSet,
    generate_pkce_state, parse_query_params, profile_id, random_base64url, select_profile_id,
    url_decode, url_encode,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default context window size (tokens)
pub const DEFAULT_CONTEXT_WINDOW: usize = 100_000;

/// Maximum parallel tool calls
pub const MAX_PARALLEL_TOOLS: usize = 10;
