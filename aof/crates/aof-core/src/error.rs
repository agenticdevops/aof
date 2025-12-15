use thiserror::Error;

/// Main error type for AOF framework
#[derive(Error, Debug)]
pub enum AofError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Tool execution error: {0}")]
    Tool(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("MCP protocol error: {0}")]
    Mcp(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Type alias for Results using AofError
pub type AofResult<T> = Result<T, AofError>;

impl AofError {
    /// Create an agent error
    pub fn agent(msg: impl Into<String>) -> Self {
        Self::Agent(msg.into())
    }

    /// Create a model error
    pub fn model(msg: impl Into<String>) -> Self {
        Self::Model(msg.into())
    }

    /// Create a tool error
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }

    /// Create a memory error
    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }

    /// Create an MCP error
    pub fn mcp(msg: impl Into<String>) -> Self {
        Self::Mcp(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_variants() {
        let agent_err = AofError::agent("test agent error");
        assert!(matches!(agent_err, AofError::Agent(_)));
        assert!(agent_err.to_string().contains("Agent error"));

        let model_err = AofError::model("test model error");
        assert!(matches!(model_err, AofError::Model(_)));
        assert!(model_err.to_string().contains("Model error"));

        let tool_err = AofError::tool("test tool error");
        assert!(matches!(tool_err, AofError::Tool(_)));
        assert!(tool_err.to_string().contains("Tool execution error"));

        let memory_err = AofError::memory("test memory error");
        assert!(matches!(memory_err, AofError::Memory(_)));
        assert!(memory_err.to_string().contains("Memory error"));

        let mcp_err = AofError::mcp("test mcp error");
        assert!(matches!(mcp_err, AofError::Mcp(_)));
        assert!(mcp_err.to_string().contains("MCP protocol error"));

        let config_err = AofError::config("test config error");
        assert!(matches!(config_err, AofError::Config(_)));
        assert!(config_err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let aof_err: AofError = io_err.into();
        assert!(matches!(aof_err, AofError::Io(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_result: Result<String, serde_json::Error> = serde_json::from_str("invalid json");
        let aof_err: AofError = json_result.unwrap_err().into();
        assert!(matches!(aof_err, AofError::Serialization(_)));
    }

    #[test]
    fn test_aof_result() {
        let ok_result: AofResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: AofResult<i32> = Err(AofError::agent("test"));
        assert!(err_result.is_err());
    }
}
