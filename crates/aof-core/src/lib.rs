// AOF Core - Foundation types and traits for the Agentic Ops Framework
//
// This crate provides zero-cost abstractions for building high-performance
// agentic systems targeting DevOps and SRE workflows.

pub mod agent;
pub mod agentflow;
pub mod binding;
pub mod context;
pub mod error;
pub mod error_tracker;
pub mod fleet;
pub mod mcp;
pub mod memory;
pub mod model;
pub mod registry;
pub mod schema;
pub mod tool;
pub mod trigger;
pub mod workflow;

// Re-export core types
pub use agent::{
    Agent, AgentConfig, AgentContext, AgentMetadata, ExecutionMetadata, Message, MessageRole,
    QualifiedToolSpec, ToolResult as AgentToolResult, ToolSource, ToolSpec,
};
pub use error::{AofError, AofResult};
pub use error_tracker::{ErrorKnowledgeBase, ErrorRecord, ErrorStats};
pub use mcp::{McpServerConfig, McpTransport};
pub use memory::{Memory, MemoryBackend, MemoryEntry, MemoryQuery};
pub use model::{
    Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, RequestMessage, StopReason,
    StreamChunk, ToolDefinition as ModelToolDefinition, Usage,
};
pub use schema::{FormatHint, InputSchema, OutputSchema};
pub use tool::{
    Tool, ToolCall, ToolConfig, ToolDefinition, ToolExecutor, ToolInput, ToolResult, ToolType,
};
pub use workflow::{
    BackoffStrategy, CheckpointBackend, CheckpointConfig, CheckpointFrequency, ConditionalNext,
    FlatWorkflowConfig, InterruptConfig, InterruptType, JoinConfig, JoinStrategy, NextStep,
    ParallelBranch, RecoveryConfig, ReducerType, RetryConfig, StateReducer, StateSchema, StepConfig,
    StepResult, StepStatus, StepType, TerminalStatus, ValidatorType, Workflow, WorkflowConfigInput,
    WorkflowError, WorkflowMetadata, WorkflowSpec, WorkflowState, WorkflowStatus, WorkflowStep,
};
pub use fleet::{
    AgentFleet, AgentInstanceState, AgentInstanceStatus, AgentRole, CoordinationConfig,
    CoordinationMode, ConsensusConfig, ConsensusAlgorithm, DeepConfig, FinalAggregation, FleetAgent,
    FleetAgentSpec, FleetMetadata, FleetMetrics, FleetSpec, FleetState, FleetStatus, FleetTask,
    FleetTaskStatus, SharedResources, SharedMemoryConfig, SharedMemoryType, CommunicationConfig,
    MessagePattern, TaskDistribution, ScalingConfig, TieredConfig,
};
pub use agentflow::{
    AgentFlow, AgentFlowMetadata, AgentFlowSpec, AgentFlowState, FlowConfig, FlowConnection,
    FlowContext, FlowError, FlowExecutionStatus, FlowNode, FlowRetryConfig, InlineAgentConfig,
    NodeCondition, NodeConfig, NodeExecutionStatus, NodeResult, NodeType, ScriptConfig,
    ScriptOutputParse,
};
pub use binding::{
    BindingMatch, FlowBinding, FlowBindingMetadata, FlowBindingSpec, ResolvedBinding,
};
pub use context::{
    ApprovalConfig, AuditConfig, AuditEvent, Context, ContextMetadata, ContextSpec,
    LimitsConfig, SecretRef,
};
pub use registry::{
    AgentRegistry, BindingRegistry, ContextRegistry, FlowRegistry, Registry,
    ResourceManager, TriggerRegistry,
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
