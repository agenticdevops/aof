//! AOF Tools - Modular tool implementations for AOF agents
//!
//! This crate provides a collection of built-in tools that agents can use.
//! Tools are organized by category and can be enabled/disabled via feature flags.
//!
//! # Feature Flags
//!
//! - `file` - File system operations (read, write, list, search)
//! - `shell` - Shell command execution
//! - `kubectl` - Kubernetes operations via kubectl
//! - `docker` - Docker container operations
//! - `git` - Git repository operations
//! - `terraform` - Terraform IaC operations
//! - `http` - HTTP request tool
//! - `all` - Enable all tools
//!
//! # Architecture
//!
//! Tools can be used in two ways:
//! 1. **Built-in tools**: Direct Rust implementations (this crate)
//! 2. **MCP tools**: External MCP servers for the same functionality
//!
//! Users can choose their preferred approach based on requirements:
//! - Built-in: Lower latency, no external dependencies
//! - MCP: More flexible, can use community servers
//!
//! # Example
//!
//! ```rust,ignore
//! use aof_tools::{ToolRegistry, FileTools, ShellTool};
//!
//! let mut registry = ToolRegistry::new();
//! registry.register_category(FileTools::all());
//! registry.register(ShellTool::new());
//!
//! // Use with agent
//! let executor = registry.into_executor();
//! ```

pub mod registry;
pub mod tools;

pub use registry::{ToolRegistry, BuiltinToolExecutor};

// Re-export individual tools based on features
#[cfg(feature = "file")]
pub use tools::file::{FileTools, ReadFileTool, WriteFileTool, ListDirTool, SearchFilesTool};

#[cfg(feature = "shell")]
pub use tools::shell::ShellTool;

#[cfg(feature = "kubectl")]
pub use tools::kubectl::{KubectlTools, KubectlGetTool, KubectlApplyTool, KubectlDeleteTool, KubectlLogsTool, KubectlExecTool, KubectlDescribeTool};

#[cfg(feature = "docker")]
pub use tools::docker::{DockerTools, DockerPsTool, DockerBuildTool, DockerRunTool, DockerLogsTool, DockerExecTool, DockerImagesTool};

#[cfg(feature = "git")]
pub use tools::git::{GitTools, GitStatusTool, GitDiffTool, GitLogTool, GitCommitTool, GitBranchTool, GitCheckoutTool, GitPullTool, GitPushTool};

#[cfg(feature = "terraform")]
pub use tools::terraform::{TerraformTools, TerraformInitTool, TerraformPlanTool, TerraformApplyTool, TerraformDestroyTool, TerraformOutputTool};

#[cfg(feature = "http")]
pub use tools::http::HttpTool;

#[cfg(feature = "observability")]
pub use tools::observability::{ObservabilityTools, PrometheusQueryTool, LokiQueryTool, ElasticsearchQueryTool, VictoriaMetricsQueryTool};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::registry::{ToolRegistry, BuiltinToolExecutor};
    pub use aof_core::{Tool, ToolExecutor, ToolInput, ToolResult, ToolConfig, ToolDefinition};

    #[cfg(feature = "file")]
    pub use super::tools::file::FileTools;

    #[cfg(feature = "shell")]
    pub use super::tools::shell::ShellTool;

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
