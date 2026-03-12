// OpenAgentiX MCP - Model Context Protocol client implementation
//
// Supports multiple transports: stdio, SSE, HTTP
// Zero-copy JSON parsing where possible

pub mod client;
pub mod transport;

pub use client::{McpClient, McpClientBuilder};
pub use transport::{McpTransport, TransportType};

// Re-export from agentix-core
pub use agentix_core::{AgentixError, AgentixResult};
// Backward-compatible aliases
pub use agentix_core::{AofError, AofResult};

/// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";
