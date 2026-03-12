//! Tool execution implementations for OpenAgentiX agents.
//!
//! This module contains concrete `ToolExecutor` implementations:
//! - [`CliToolExecutor`] — executes CLI tools and shell commands
//! - MCP executor — Phase 14 plan 03
//! - WASM executor — Phase 14 plan 04

pub mod cli_executor;
pub mod mcp_executor;

pub use cli_executor::CliToolExecutor;
pub use mcp_executor::{CompositeToolExecutor, McpToolExecutor};
