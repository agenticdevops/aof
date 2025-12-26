use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::mcp::McpServerConfig;
use crate::AofResult;

/// Memory specification - unified way to configure memory backends
///
/// Supports multiple formats:
/// 1. Simple string: `"file:./memory.json"` or `"in_memory"`
/// 2. Object with type: `{type: "File", config: {path: "./memory.json", max_messages: 50}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MemorySpec {
    /// Simple memory specification (backward compatible)
    /// Format: "type" or "type:path" (e.g., "in_memory", "file:./memory.json")
    Simple(String),

    /// Structured memory configuration
    Structured(StructuredMemoryConfig),
}

impl MemorySpec {
    /// Get the memory type
    pub fn memory_type(&self) -> &str {
        match self {
            MemorySpec::Simple(s) => {
                // Extract type from "type" or "type:path" format
                s.split(':').next().unwrap_or(s)
            }
            MemorySpec::Structured(config) => &config.memory_type,
        }
    }

    /// Get the file path if this is a file-based memory
    pub fn path(&self) -> Option<String> {
        match self {
            MemorySpec::Simple(s) => {
                // Extract path from "file:./path.json" format
                if s.contains(':') {
                    s.split(':').nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            }
            MemorySpec::Structured(config) => config
                .config
                .as_ref()
                .and_then(|c| c.get("path"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }

    /// Get max_messages configuration if available
    pub fn max_messages(&self) -> Option<usize> {
        match self {
            MemorySpec::Simple(_) => None,
            MemorySpec::Structured(config) => config
                .config
                .as_ref()
                .and_then(|c| c.get("max_messages"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
        }
    }

    /// Get the full configuration object
    pub fn config(&self) -> Option<&serde_json::Value> {
        match self {
            MemorySpec::Simple(_) => None,
            MemorySpec::Structured(config) => config.config.as_ref(),
        }
    }

    /// Check if this is an in-memory backend
    pub fn is_in_memory(&self) -> bool {
        let t = self.memory_type().to_lowercase();
        t == "in_memory" || t == "inmemory" || t == "memory"
    }

    /// Check if this is a file-based backend
    pub fn is_file(&self) -> bool {
        self.memory_type().to_lowercase() == "file"
    }
}

/// Structured memory configuration with type and config fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMemoryConfig {
    /// Memory backend type: "File", "InMemory", etc.
    #[serde(rename = "type")]
    pub memory_type: String,

    /// Backend-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

impl fmt::Display for MemorySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemorySpec::Simple(s) => write!(f, "{}", s),
            MemorySpec::Structured(config) => {
                if let Some(path) = self.path() {
                    write!(f, "{} (path: {})", config.memory_type, path)
                } else {
                    write!(f, "{}", config.memory_type)
                }
            }
        }
    }
}

/// Tool specification - unified way to configure both built-in and MCP tools
///
/// Supports multiple formats:
/// 1. Simple string: `"shell"` - built-in tool with defaults
/// 2. Object with source: `{name: "kubectl_get", source: "builtin", config: {...}}`
/// 3. MCP tool: `{name: "read_file", source: "mcp", server: "filesystem"}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolSpec {
    /// Simple tool name (for backward compatibility)
    /// Assumes built-in if the tool exists, otherwise tries MCP
    Simple(String),

    /// Fully qualified tool specification
    Qualified(QualifiedToolSpec),
}

impl ToolSpec {
    /// Get the tool name
    pub fn name(&self) -> &str {
        match self {
            ToolSpec::Simple(name) => name,
            ToolSpec::Qualified(spec) => &spec.name,
        }
    }

    /// Check if this is explicitly a built-in tool
    pub fn is_builtin(&self) -> bool {
        match self {
            ToolSpec::Simple(_) => true, // default to builtin for simple names
            ToolSpec::Qualified(spec) => spec.source == ToolSource::Builtin,
        }
    }

    /// Check if this is explicitly an MCP tool
    pub fn is_mcp(&self) -> bool {
        match self {
            ToolSpec::Simple(_) => false,
            ToolSpec::Qualified(spec) => spec.source == ToolSource::Mcp,
        }
    }

    /// Get the MCP server name (if this is an MCP tool)
    pub fn mcp_server(&self) -> Option<&str> {
        match self {
            ToolSpec::Simple(_) => None,
            ToolSpec::Qualified(spec) => spec.server.as_deref(),
        }
    }

    /// Get tool configuration
    pub fn config(&self) -> Option<&serde_json::Value> {
        match self {
            ToolSpec::Simple(_) => None,
            ToolSpec::Qualified(spec) => spec.config.as_ref(),
        }
    }
}

/// Fully qualified tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedToolSpec {
    /// Tool name
    pub name: String,

    /// Tool source: builtin or mcp
    #[serde(default)]
    pub source: ToolSource,

    /// MCP server name (required if source is "mcp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,

    /// Tool-specific configuration/arguments
    /// For built-in tools: default values, restrictions, etc.
    /// For MCP tools: tool-specific options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,

    /// Whether the tool is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Timeout override for this specific tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

fn default_enabled() -> bool {
    true
}

/// Tool source - where the tool comes from
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolSource {
    /// Built-in AOF tool (Rust implementation)
    #[default]
    Builtin,
    /// MCP server tool
    Mcp,
}

/// Core agent trait - the foundation of AOF
///
/// Agents orchestrate models, tools, and memory to accomplish tasks.
/// Implementations should be zero-cost wrappers where possible.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute the agent with given input
    async fn execute(&self, ctx: &mut AgentContext) -> AofResult<String>;

    /// Agent metadata
    fn metadata(&self) -> &AgentMetadata;

    /// Initialize agent (setup resources, validate config)
    async fn init(&mut self) -> AofResult<()> {
        Ok(())
    }

    /// Cleanup agent resources
    async fn cleanup(&mut self) -> AofResult<()> {
        Ok(())
    }

    /// Validate agent configuration
    fn validate(&self) -> AofResult<()> {
        Ok(())
    }
}

/// Agent execution context - passed through the execution chain
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// User input/query
    pub input: String,

    /// Conversation history
    pub messages: Vec<Message>,

    /// Session state/variables
    pub state: HashMap<String, serde_json::Value>,

    /// Tool execution results
    pub tool_results: Vec<ToolResult>,

    /// Execution metadata
    pub metadata: ExecutionMetadata,

    /// Optional output schema for structured responses
    pub output_schema: Option<crate::schema::OutputSchema>,

    /// Optional input schema for validation
    pub input_schema: Option<crate::schema::InputSchema>,
}

/// Message in conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::ToolCall>>,
    /// Tool call ID (required for Tool role messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub result: serde_json::Value,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Execution metadata
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetadata {
    /// Tokens used (input)
    pub input_tokens: usize,
    /// Tokens used (output)
    pub output_tokens: usize,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Number of tool calls
    pub tool_calls: usize,
    /// Model used
    pub model: Option<String>,
}

impl AgentContext {
    /// Create new context with input
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            messages: Vec::new(),
            state: HashMap::new(),
            tool_results: Vec::new(),
            metadata: ExecutionMetadata::default(),
            output_schema: None,
            input_schema: None,
        }
    }

    /// Set output schema for structured responses
    pub fn with_output_schema(mut self, schema: crate::schema::OutputSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Set input schema for validation
    pub fn with_input_schema(mut self, schema: crate::schema::InputSchema) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Add a message to history
    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(Message {
            role,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    /// Get state value
    pub fn get_state<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.state
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set state value
    pub fn set_state<T: Serialize>(&mut self, key: impl Into<String>, value: T) -> AofResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.state.insert(key.into(), json_value);
        Ok(())
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent name
    pub name: String,

    /// Agent description
    pub description: String,

    /// Agent version
    pub version: String,

    /// Supported capabilities
    pub capabilities: Vec<String>,

    /// Custom metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Agent configuration
/// Supports both flat format and Kubernetes-style format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "AgentConfigInput")]
pub struct AgentConfig {
    /// Agent name
    pub name: String,

    /// System prompt (also accepts "instructions" alias)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Model to use (can be "provider:model" format or just "model")
    pub model: String,

    /// LLM provider (anthropic, openai, google, ollama, groq, bedrock, azure)
    /// Optional if provider is specified in model string (e.g., "google:gemini-2.0-flash")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Tools available to agent
    /// Supports both simple strings (backward compatible) and qualified specs
    /// Simple string: "shell" - uses built-in tool with defaults
    /// Qualified: {name: "shell", source: "builtin", config: {...}}
    #[serde(default)]
    pub tools: Vec<ToolSpec>,

    /// MCP servers configuration (flexible MCP tool sources)
    /// Each server can use stdio, sse, or http transport
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,

    /// Memory backend configuration
    /// Supports both simple string format and structured object format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemorySpec>,

    /// Maximum number of conversation messages to include in context
    /// Controls token usage by limiting how much history is sent to the LLM.
    /// Default is 10 messages. Set higher for longer context, lower to save tokens.
    #[serde(default = "default_max_context_messages")]
    pub max_context_messages: usize,

    /// Max iterations
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Temperature (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Custom configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl AgentConfig {
    /// Get all tool names (for backward compatibility)
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    /// Get built-in tools only
    pub fn builtin_tools(&self) -> Vec<&ToolSpec> {
        self.tools.iter().filter(|t| t.is_builtin()).collect()
    }

    /// Get MCP tools only
    pub fn mcp_tools(&self) -> Vec<&ToolSpec> {
        self.tools.iter().filter(|t| t.is_mcp()).collect()
    }
}

/// Internal type for flexible config parsing
/// Supports both flat format and Kubernetes-style format
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AgentConfigInput {
    /// Flat format (original) - try this first since it has required fields
    Flat(FlatAgentConfig),
    /// Kubernetes-style format with apiVersion, kind, metadata, spec
    Kubernetes(KubernetesConfig),
}

/// Kubernetes-style config wrapper
#[derive(Debug, Clone, Deserialize)]
struct KubernetesConfig {
    #[serde(rename = "apiVersion")]
    api_version: String,  // Required for K8s format
    kind: String,         // Required for K8s format
    metadata: KubernetesMetadata,
    spec: AgentSpec,
}

#[derive(Debug, Clone, Deserialize)]
struct KubernetesMetadata {
    name: String,
    #[serde(default)]
    labels: HashMap<String, String>,
    #[serde(default)]
    annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentSpec {
    model: String,
    provider: Option<String>,
    #[serde(alias = "system_prompt")]
    instructions: Option<String>,
    #[serde(default)]
    tools: Vec<ToolSpec>,
    #[serde(default)]
    mcp_servers: Vec<McpServerConfig>,
    memory: Option<MemorySpec>,
    #[serde(default = "default_max_context_messages")]
    max_context_messages: usize,
    #[serde(default = "default_max_iterations")]
    max_iterations: usize,
    #[serde(default = "default_temperature")]
    temperature: f32,
    max_tokens: Option<usize>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct FlatAgentConfig {
    name: String,
    #[serde(alias = "instructions")]
    system_prompt: Option<String>,
    model: String,
    provider: Option<String>,
    #[serde(default)]
    tools: Vec<ToolSpec>,
    #[serde(default)]
    mcp_servers: Vec<McpServerConfig>,
    memory: Option<MemorySpec>,
    #[serde(default = "default_max_context_messages")]
    max_context_messages: usize,
    #[serde(default = "default_max_iterations")]
    max_iterations: usize,
    #[serde(default = "default_temperature")]
    temperature: f32,
    max_tokens: Option<usize>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl From<AgentConfigInput> for AgentConfig {
    fn from(input: AgentConfigInput) -> Self {
        match input {
            AgentConfigInput::Flat(flat) => AgentConfig {
                name: flat.name,
                system_prompt: flat.system_prompt,
                model: flat.model,
                provider: flat.provider,
                tools: flat.tools,
                mcp_servers: flat.mcp_servers,
                memory: flat.memory,
                max_context_messages: flat.max_context_messages,
                max_iterations: flat.max_iterations,
                temperature: flat.temperature,
                max_tokens: flat.max_tokens,
                extra: flat.extra,
            },
            AgentConfigInput::Kubernetes(k8s) => {
                AgentConfig {
                    name: k8s.metadata.name,
                    system_prompt: k8s.spec.instructions,
                    model: k8s.spec.model,
                    provider: k8s.spec.provider,
                    tools: k8s.spec.tools,
                    mcp_servers: k8s.spec.mcp_servers,
                    memory: k8s.spec.memory,
                    max_context_messages: k8s.spec.max_context_messages,
                    max_iterations: k8s.spec.max_iterations,
                    temperature: k8s.spec.temperature,
                    max_tokens: k8s.spec.max_tokens,
                    extra: k8s.spec.extra,
                }
            }
        }
    }
}

fn default_max_iterations() -> usize {
    10
}

fn default_max_context_messages() -> usize {
    10
}

fn default_temperature() -> f32 {
    0.7
}

/// Reference-counted agent
pub type AgentRef = Arc<dyn Agent>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context_new() {
        let ctx = AgentContext::new("Hello, world!");
        assert_eq!(ctx.input, "Hello, world!");
        assert!(ctx.messages.is_empty());
        assert!(ctx.state.is_empty());
        assert!(ctx.tool_results.is_empty());
    }

    #[test]
    fn test_agent_context_add_message() {
        let mut ctx = AgentContext::new("test");
        ctx.add_message(MessageRole::User, "user message");
        ctx.add_message(MessageRole::Assistant, "assistant response");

        assert_eq!(ctx.messages.len(), 2);
        assert_eq!(ctx.messages[0].role, MessageRole::User);
        assert_eq!(ctx.messages[0].content, "user message");
        assert_eq!(ctx.messages[1].role, MessageRole::Assistant);
        assert_eq!(ctx.messages[1].content, "assistant response");
    }

    #[test]
    fn test_agent_context_state() {
        let mut ctx = AgentContext::new("test");

        // Set string state
        ctx.set_state("name", "test_agent").unwrap();
        let name: Option<String> = ctx.get_state("name");
        assert_eq!(name, Some("test_agent".to_string()));

        // Set numeric state
        ctx.set_state("count", 42i32).unwrap();
        let count: Option<i32> = ctx.get_state("count");
        assert_eq!(count, Some(42));

        // Get non-existent key
        let missing: Option<String> = ctx.get_state("missing");
        assert!(missing.is_none());
    }

    #[test]
    fn test_message_role_serialization() {
        let user = MessageRole::User;
        let serialized = serde_json::to_string(&user).unwrap();
        assert_eq!(serialized, "\"user\"");

        let deserialized: MessageRole = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(deserialized, MessageRole::Assistant);
    }

    #[test]
    fn test_agent_config_defaults() {
        let yaml = r#"
            name: test-agent
            model: claude-3-5-sonnet
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "test-agent");
        assert_eq!(config.model, "claude-3-5-sonnet");
        assert_eq!(config.max_iterations, 10); // default
        assert_eq!(config.temperature, 0.7); // default
        assert!(config.tools.is_empty());
        assert!(config.system_prompt.is_none());
    }

    #[test]
    fn test_agent_config_full() {
        let yaml = r#"
            name: full-agent
            model: gpt-4
            system_prompt: "You are a helpful assistant."
            tools:
              - read_file
              - write_file
            max_iterations: 20
            temperature: 0.5
            max_tokens: 4096
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "full-agent");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.system_prompt, Some("You are a helpful assistant.".to_string()));
        assert_eq!(config.tool_names(), vec!["read_file", "write_file"]);
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, Some(4096));
    }

    #[test]
    fn test_tool_spec_simple() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - shell
              - kubectl_get
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 2);
        assert_eq!(config.tools[0].name(), "shell");
        assert!(config.tools[0].is_builtin());
        assert!(!config.tools[0].is_mcp());
    }

    #[test]
    fn test_tool_spec_qualified_builtin() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - name: shell
                source: builtin
                config:
                  blocked_commands:
                    - rm -rf
                  timeout_secs: 60
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.tools[0].name(), "shell");
        assert!(config.tools[0].is_builtin());
        assert!(config.tools[0].config().is_some());
    }

    #[test]
    fn test_tool_spec_qualified_mcp() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - name: read_file
                source: mcp
                server: filesystem
                config:
                  allowed_paths:
                    - /workspace
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.tools[0].name(), "read_file");
        assert!(config.tools[0].is_mcp());
        assert_eq!(config.tools[0].mcp_server(), Some("filesystem"));
    }

    #[test]
    fn test_tool_spec_mixed() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              # Simple builtin
              - shell
              # Qualified builtin with config
              - name: kubectl_get
                source: builtin
                timeout_secs: 120
              # MCP tool
              - name: github_search
                source: mcp
                server: github
            mcp_servers:
              - name: github
                command: npx
                args: ["@modelcontextprotocol/server-github"]
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 3);

        // Check builtin tools
        let builtin_tools = config.builtin_tools();
        assert_eq!(builtin_tools.len(), 2);

        // Check MCP tools
        let mcp_tools = config.mcp_tools();
        assert_eq!(mcp_tools.len(), 1);
        assert_eq!(mcp_tools[0].mcp_server(), Some("github"));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult {
            tool_name: "test_tool".to_string(),
            result: serde_json::json!({"output": "success"}),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("success"));

        let deserialized: ToolResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_name, "test_tool");
        assert!(deserialized.success);
    }

    #[test]
    fn test_execution_metadata_default() {
        let meta = ExecutionMetadata::default();
        assert_eq!(meta.input_tokens, 0);
        assert_eq!(meta.output_tokens, 0);
        assert_eq!(meta.execution_time_ms, 0);
        assert_eq!(meta.tool_calls, 0);
        assert!(meta.model.is_none());
    }

    #[test]
    fn test_agent_metadata_serialization() {
        let meta = AgentMetadata {
            name: "test".to_string(),
            description: "A test agent".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["coding".to_string(), "testing".to_string()],
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: AgentMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.capabilities.len(), 2);
    }

    #[test]
    fn test_agent_config_with_mcp_servers() {
        let yaml = r#"
            name: mcp-agent
            model: gpt-4
            mcp_servers:
              - name: filesystem
                transport: stdio
                command: npx
                args:
                  - "@anthropic-ai/mcp-server-fs"
                env:
                  MCP_FS_ROOT: /workspace
              - name: remote
                transport: sse
                endpoint: http://localhost:3000/mcp
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "mcp-agent");
        assert_eq!(config.mcp_servers.len(), 2);

        // Check first server (stdio)
        let fs_server = &config.mcp_servers[0];
        assert_eq!(fs_server.name, "filesystem");
        assert_eq!(fs_server.transport, crate::mcp::McpTransport::Stdio);
        assert_eq!(fs_server.command, Some("npx".to_string()));
        assert_eq!(fs_server.args.len(), 1);
        assert!(fs_server.env.contains_key("MCP_FS_ROOT"));

        // Check second server (sse)
        let remote_server = &config.mcp_servers[1];
        assert_eq!(remote_server.name, "remote");
        assert_eq!(remote_server.transport, crate::mcp::McpTransport::Sse);
        assert_eq!(remote_server.endpoint, Some("http://localhost:3000/mcp".to_string()));
    }

    #[test]
    fn test_agent_config_k8s_style_with_mcp_servers() {
        let yaml = r#"
            apiVersion: aof.dev/v1
            kind: Agent
            metadata:
              name: k8s-mcp-agent
              labels:
                env: test
            spec:
              model: claude-3-5-sonnet
              instructions: Test agent with MCP
              mcp_servers:
                - name: tools
                  command: ./my-mcp-server
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "k8s-mcp-agent");
        assert_eq!(config.mcp_servers.len(), 1);
        assert_eq!(config.mcp_servers[0].name, "tools");
        assert_eq!(config.mcp_servers[0].command, Some("./my-mcp-server".to_string()));
    }

    #[test]
    fn test_memory_spec_simple_string() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            memory: "file:./memory.json"
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert_eq!(memory.memory_type(), "file");
        assert_eq!(memory.path(), Some("./memory.json".to_string()));
        assert!(memory.is_file());
        assert!(!memory.is_in_memory());
    }

    #[test]
    fn test_memory_spec_simple_in_memory() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            memory: "in_memory"
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert_eq!(memory.memory_type(), "in_memory");
        assert!(memory.is_in_memory());
        assert!(!memory.is_file());
    }

    #[test]
    fn test_memory_spec_structured_file() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            memory:
              type: File
              config:
                path: ./k8s-helper-memory.json
                max_messages: 50
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert_eq!(memory.memory_type(), "File");
        assert_eq!(memory.path(), Some("./k8s-helper-memory.json".to_string()));
        assert_eq!(memory.max_messages(), Some(50));
        assert!(memory.is_file());
    }

    #[test]
    fn test_memory_spec_structured_in_memory() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            memory:
              type: InMemory
              config:
                max_messages: 100
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert_eq!(memory.memory_type(), "InMemory");
        assert!(memory.is_in_memory());
        assert_eq!(memory.max_messages(), Some(100));
    }

    #[test]
    fn test_memory_spec_k8s_style_with_structured_memory() {
        // This is the exact format from the bug report
        let yaml = r#"
            apiVersion: aof.dev/v1
            kind: Agent
            metadata:
              name: k8s-helper
              labels:
                purpose: operations
                team: platform
            spec:
              model: google:gemini-2.5-flash
              instructions: |
                You are a Kubernetes helper.
              memory:
                type: File
                config:
                  path: ./k8s-helper-memory.json
                  max_messages: 50
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "k8s-helper");
        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert_eq!(memory.memory_type(), "File");
        assert_eq!(memory.path(), Some("./k8s-helper-memory.json".to_string()));
        assert_eq!(memory.max_messages(), Some(50));
    }

    #[test]
    fn test_memory_spec_no_memory() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.memory.is_none());
    }
}
