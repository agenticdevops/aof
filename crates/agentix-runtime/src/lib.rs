//! OpenAgentiX Runtime — Agent execution runtime
//!
//! This crate provides the ReAct (Reason + Act) loop execution engine for
//! OpenAgentiX agents, handling plan-act-observe-reflect cycles.

pub mod approval_store;
pub mod audit_store;
pub mod channels;
pub mod cost_store;
pub mod executor;
pub mod gateway;
pub mod health;
pub mod memory;
pub mod metrics;
pub mod otel_exporter;
pub mod shutdown;
pub mod trace_store;
pub mod streaming;
pub mod telemetry;
pub mod tools;

pub use approval_store::ApprovalStore;
pub use audit_store::{AuditEntry, AuditEventType, AuditOutcome, AuditStore};
pub use cost_store::CostStore;
pub use executor::{ReActConfig, ReActEngine, ReActEvent, ReActStep, RunResult, ToolAction};
pub use memory::{format_memory_context, hash_embedding, open_agent_memory};
pub use tools::CliToolExecutor;
pub use streaming::{EventReceiver, EventSender, JsonFormatter, SseEncoder, TextFormatter};
pub use health::{
    check_readiness, DependencyState, DependencyStatus, HealthResponse, ReadinessResponse,
};
pub use metrics::AofMetrics;
pub use otel_exporter::OtelExporter;
pub use telemetry::TraceCollector;
pub use trace_store::TraceStore;
pub use shutdown::{GracefulShutdown, ShutdownHandler};

// Re-export core types
pub use agentix_core::{AgentixError, AgentixResult};
// Backward-compatible aliases
pub use agentix_core::{AofError, AofResult};
