//! MCP (Model Context Protocol) tool executor for the ReAct loop.
//!
//! Connects to MCP servers configured in an agent's `mcp_servers` list
//! and dispatches tool calls to the appropriate server.
//!
//! ## Connection lifecycle
//!
//! MCP connections are per-run — established when the first MCP tool is called,
//! torn down when the run completes. This avoids holding open stdio processes
//! for idle agents.

use std::collections::HashMap;

use agentix_core::mcp::{McpServerConfig, McpTransport};
use agentix_core::DirectoryToolType;
use agentix_core::ToolEntry;
use agentix_mcp::{McpClient, McpClientBuilder};

use crate::executor::react_loop::ToolExecutor;

/// Executes MCP tool calls by connecting to configured MCP servers.
///
/// Holds a map of server name → `McpServerConfig` for lazy connection.
/// On first call to a given server, establishes the connection; subsequent
/// calls within the same run reuse it.
pub struct McpToolExecutor {
    /// Configured MCP servers keyed by server name.
    server_configs: HashMap<String, McpServerConfig>,
}

impl McpToolExecutor {
    /// Create a new MCP executor from a list of server configurations.
    pub fn new(configs: Vec<McpServerConfig>) -> Self {
        let server_configs = configs
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        Self { server_configs }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        // Only handle MCP tools
        if tool.tool_type != DirectoryToolType::Mcp {
            return Err(format!(
                "McpToolExecutor cannot execute tool type {:?} — use CliToolExecutor",
                tool.tool_type
            ));
        }

        let server_name = tool
            .server
            .as_deref()
            .ok_or_else(|| format!("MCP tool '{}' has no server configured", tool.name))?;

        let config = self.server_configs.get(server_name).ok_or_else(|| {
            format!(
                "MCP server '{}' not configured. Add it to your agent's mcp_servers list.",
                server_name
            )
        })?;

        // Connect to the MCP server and call the tool
        call_mcp_tool(config, &tool.name, input).await
    }
}

/// Connect to an MCP server, call the specified tool, and return the result.
async fn call_mcp_tool(
    config: &McpServerConfig,
    tool_name: &str,
    input: serde_json::Value,
) -> Result<String, String> {
    // Build client using existing agentix-mcp transport infrastructure
    let client = build_mcp_client(config)
        .map_err(|e| format!("Failed to create MCP client for '{}': {}", config.name, e))?;

    // Initialize the connection
    client
        .initialize()
        .await
        .map_err(|e| format!("Failed to connect to MCP server '{}': {}", config.name, e))?;

    // Call the tool
    let result = client
        .call_tool(tool_name, input)
        .await
        .map_err(|e| format!("MCP tool '{}' on server '{}' failed: {}", tool_name, config.name, e))?;

    // Extract text from result value
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string_pretty(&result).unwrap_or_default());

    Ok(text)
}

/// Build an McpClient from an McpServerConfig using the builder pattern.
fn build_mcp_client(config: &McpServerConfig) -> agentix_core::AgentixResult<McpClient> {
    let mut builder = McpClientBuilder::new();

    match config.transport {
        McpTransport::Stdio => {
            let command = config.command.as_deref().ok_or_else(|| {
                agentix_core::AgentixError::Config(format!(
                    "MCP server '{}' with stdio transport requires a command",
                    config.name
                ))
            })?;
            builder = builder.stdio(command, config.args.clone());
            for (k, v) in &config.env {
                builder = builder.with_env(k, v);
            }
        }
        McpTransport::Sse => {
            let endpoint = config.endpoint.as_deref().ok_or_else(|| {
                agentix_core::AgentixError::Config(format!(
                    "MCP server '{}' with sse transport requires an endpoint",
                    config.name
                ))
            })?;
            // SSE is only available with the 'sse' feature; return error if not enabled
            let _ = endpoint;
            return Err(agentix_core::AgentixError::Config(
                "SSE transport requires the 'sse' feature to be enabled".to_string(),
            ));
        }
        McpTransport::Http => {
            let endpoint = config.endpoint.as_deref().ok_or_else(|| {
                agentix_core::AgentixError::Config(format!(
                    "MCP server '{}' with http transport requires an endpoint",
                    config.name
                ))
            })?;
            // HTTP is only available with the 'http' feature; return error if not enabled
            let _ = endpoint;
            return Err(agentix_core::AgentixError::Config(
                "HTTP transport requires the 'http' feature to be enabled".to_string(),
            ));
        }
    }

    builder.build()
}

// ---------------------------------------------------------------------------
// CompositeToolExecutor — combines CLI and MCP execution
// ---------------------------------------------------------------------------

/// A `ToolExecutor` that delegates to the appropriate executor based on tool type.
///
/// - `DirectoryToolType::Cli` and `Shell` → CLI executor
/// - `DirectoryToolType::Mcp` → MCP executor
pub struct CompositeToolExecutor<C, M> {
    cli_executor: C,
    mcp_executor: M,
}

impl<C, M> CompositeToolExecutor<C, M>
where
    C: ToolExecutor,
    M: ToolExecutor,
{
    /// Create a new composite executor.
    pub fn new(cli_executor: C, mcp_executor: M) -> Self {
        Self {
            cli_executor,
            mcp_executor,
        }
    }
}

#[async_trait::async_trait]
impl<C, M> ToolExecutor for CompositeToolExecutor<C, M>
where
    C: ToolExecutor + Send + Sync,
    M: ToolExecutor + Send + Sync,
{
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        match tool.tool_type {
            DirectoryToolType::Cli | DirectoryToolType::Shell => {
                self.cli_executor.execute(tool, input).await
            }
            DirectoryToolType::Mcp => {
                self.mcp_executor.execute(tool, input).await
            }
        }
    }
}
