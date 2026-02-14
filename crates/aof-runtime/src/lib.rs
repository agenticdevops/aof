//! AOF Runtime - Agent execution runtime with task orchestration
//!
//! This crate provides the core execution engine for AOF agents, handling:
//! - Agent lifecycle management
//! - Tool call execution loops
//! - Context management
//! - Error handling and recovery
//! - Task orchestration

pub mod credential_anomaly;
pub mod credential_audit;
pub mod device;
pub mod executor;
pub mod fleet;
pub mod health;
pub mod metrics;
pub mod orchestrator;
pub mod sandbox;
pub mod shutdown;
pub mod task;

pub use credential_anomaly::{AnomalyDetector, AgentBaseline, FrequencyBaseline, VolumeBaseline};
pub use credential_audit::CredentialAccessInterceptor;
pub use device::{CertificateManager, DeviceRegistry, MtlsConfig, PrivateCA};
pub use executor::{
    AgentExecutor, AgentFlowEvent, AgentFlowExecutor, ApprovalDecision, HumanInput, Runtime,
    StreamEvent, WorkflowEvent, WorkflowExecutor,
};
pub use fleet::{FleetCoordinator, FleetEvent};
pub use health::{
    check_readiness, HealthResponse, ReadinessResponse, DependencyStatus, DependencyState,
};
pub use metrics::AofMetrics;
pub use orchestrator::RuntimeOrchestrator;
pub use sandbox::{CapabilityConfig, SeccompProfile, SeccompProfileManager};
pub use shutdown::{GracefulShutdown, ShutdownHandler};
pub use task::{Task, TaskHandle, TaskStatus};

// Re-export core types
pub use aof_core::{AofError, AofResult};
