use thiserror::Error;

/// Main error type for OpenAgentiX framework
#[derive(Error, Debug)]
pub enum AgentixError {
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

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Spec validation error — agent.yaml / agentix.yaml field-level validation.
    /// The inner string should include the field name and reason.
    #[error("Spec validation error: {0}")]
    SpecValidation(String),

    /// YAML parse error with field path from serde_path_to_error.
    #[error("YAML parse error in {path}: {message}")]
    YamlParse { path: String, message: String },

    /// Configuration file not found.
    #[error("Configuration not found: {0}")]
    ConfigNotFound(String),
}

/// Type alias for Results using AgentixError
pub type AgentixResult<T> = Result<T, AgentixError>;

// Keep backward-compatible type aliases for transition period
pub type AofError = AgentixError;
pub type AofResult<T> = AgentixResult<T>;

impl AgentixError {
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

    /// Create a runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a spec validation error (field-level agent/workspace config errors)
    pub fn spec_validation(msg: impl Into<String>) -> Self {
        Self::SpecValidation(msg.into())
    }

    /// Create a YAML parse error with field path
    pub fn yaml_parse(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::YamlParse {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a config-not-found error
    pub fn config_not_found(msg: impl Into<String>) -> Self {
        Self::ConfigNotFound(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_variants() {
        let agent_err = AgentixError::agent("test agent error");
        assert!(matches!(agent_err, AgentixError::Agent(_)));
        assert!(agent_err.to_string().contains("Agent error"));

        let model_err = AgentixError::model("test model error");
        assert!(matches!(model_err, AgentixError::Model(_)));
        assert!(model_err.to_string().contains("Model error"));

        let tool_err = AgentixError::tool("test tool error");
        assert!(matches!(tool_err, AgentixError::Tool(_)));
        assert!(tool_err.to_string().contains("Tool execution error"));

        let memory_err = AgentixError::memory("test memory error");
        assert!(matches!(memory_err, AgentixError::Memory(_)));
        assert!(memory_err.to_string().contains("Memory error"));

        let mcp_err = AgentixError::mcp("test mcp error");
        assert!(matches!(mcp_err, AgentixError::Mcp(_)));
        assert!(mcp_err.to_string().contains("MCP protocol error"));

        let config_err = AgentixError::config("test config error");
        assert!(matches!(config_err, AgentixError::Config(_)));
        assert!(config_err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: AgentixError = io_err.into();
        assert!(matches!(err, AgentixError::Io(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_result: Result<String, serde_json::Error> = serde_json::from_str("invalid json");
        let err: AgentixError = json_result.unwrap_err().into();
        assert!(matches!(err, AgentixError::Serialization(_)));
    }

    #[test]
    fn test_agentix_result() {
        let ok_result: AgentixResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: AgentixResult<i32> = Err(AgentixError::agent("test"));
        assert!(err_result.is_err());
    }
}
