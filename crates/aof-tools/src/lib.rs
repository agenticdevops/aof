//! AOF Tools - Modular tool implementations for AOF agents
//!
//! This crate provides a collection of built-in tools that agents can use.
//! Tools are organized by category and can be enabled/disabled via feature flags.
//!
//! # Recommended: Unified CLI Tools
//!
//! For DevOps workflows, use the unified CLI tools that take a single `command` argument:
//!
//! ```yaml
//! tools:
//!   - kubectl    # Run any kubectl command
//!   - git        # Run any git command
//!   - docker     # Run any docker command
//!   - terraform  # Run any terraform command
//!   - aws        # Run any AWS CLI command
//!   - helm       # Run any helm command
//! ```
//!
//! The LLM constructs the full command based on context. This is simpler and more
//! flexible than per-operation tools like `kubectl_get`, `git_status`, etc.
//!
//! # Feature Flags
//!
//! - `file` - File system operations (read, write, list, search)
//! - `shell` - Shell command execution
//! - `kubectl` - Legacy per-operation Kubernetes tools
//! - `docker` - Legacy per-operation Docker tools
//! - `git` - Legacy per-operation Git tools
//! - `terraform` - Legacy per-operation Terraform tools
//! - `http` - HTTP request tool
//! - `all` - Enable all tools
//!
//! # Example
//!
//! ```rust,ignore
//! use aof_tools::{ToolRegistry, KubectlTool, GitTool, ShellTool};
//!
//! let mut registry = ToolRegistry::new();
//! registry.register(KubectlTool::new());
//! registry.register(GitTool::new());
//! registry.register(ShellTool::new());
//!
//! // Use with agent
//! let executor = registry.into_executor();
//! ```

pub mod registry;
pub mod tools;

pub use registry::{ToolRegistry, BuiltinToolExecutor};

// ============================================================================
// Unified CLI Tools (Recommended)
// ============================================================================

/// Unified kubectl tool - execute any kubectl command
pub use tools::cli::KubectlTool;

/// Unified git tool - execute any git command
pub use tools::cli::GitTool;

/// Unified docker tool - execute any docker command
pub use tools::cli::DockerTool;

/// Unified terraform tool - execute any terraform command
pub use tools::cli::TerraformTool;

/// Unified AWS CLI tool - execute any aws command
pub use tools::cli::AwsTool;

/// Unified helm tool - execute any helm command
pub use tools::cli::HelmTool;

// ============================================================================
// File and Shell Tools
// ============================================================================

#[cfg(feature = "file")]
pub use tools::file::{FileTools, ReadFileTool, WriteFileTool, ListDirTool, SearchFilesTool};

#[cfg(feature = "shell")]
pub use tools::shell::ShellTool;

#[cfg(feature = "http")]
pub use tools::http::HttpTool;

#[cfg(feature = "observability")]
pub use tools::observability::{ObservabilityTools, PrometheusQueryTool, LokiQueryTool, ElasticsearchQueryTool, VictoriaMetricsQueryTool};

// ============================================================================
// Legacy Per-Operation Tools (Backward Compatibility)
// ============================================================================

#[cfg(feature = "kubectl")]
pub use tools::kubectl::{KubectlTools, KubectlGetTool, KubectlApplyTool, KubectlDeleteTool, KubectlLogsTool, KubectlExecTool, KubectlDescribeTool};

#[cfg(feature = "docker")]
pub use tools::docker::{DockerTools, DockerPsTool, DockerStatsTool, DockerBuildTool, DockerRunTool, DockerLogsTool, DockerExecTool, DockerImagesTool};

#[cfg(feature = "git")]
pub use tools::git::{GitTools, GitStatusTool, GitDiffTool, GitLogTool, GitCommitTool, GitBranchTool, GitCheckoutTool, GitPullTool, GitPushTool};

#[cfg(feature = "terraform")]
pub use tools::terraform::{TerraformTools, TerraformInitTool, TerraformPlanTool, TerraformApplyTool, TerraformDestroyTool, TerraformOutputTool};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::registry::{ToolRegistry, BuiltinToolExecutor};
    pub use aof_core::{Tool, ToolExecutor, ToolInput, ToolResult, ToolConfig, ToolDefinition};

    // Unified CLI tools (recommended)
    pub use super::tools::cli::{KubectlTool, GitTool, DockerTool, TerraformTool, AwsTool, HelmTool};

    #[cfg(feature = "file")]
    pub use super::tools::file::FileTools;

    #[cfg(feature = "shell")]
    pub use super::tools::shell::ShellTool;

    // Legacy tools
    #[cfg(feature = "kubectl")]
    pub use super::tools::kubectl::KubectlTools;

    #[cfg(feature = "docker")]
    pub use super::tools::docker::DockerTools;

    #[cfg(feature = "git")]
    pub use super::tools::git::GitTools;

    #[cfg(feature = "terraform")]
    pub use super::tools::terraform::TerraformTools;

    #[cfg(feature = "observability")]
    pub use super::tools::observability::ObservabilityTools;
}
