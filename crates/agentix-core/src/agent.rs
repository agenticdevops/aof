use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::mcp::McpServerConfig;
use crate::AofResult;

/// Output schema specification using JSON Schema format
/// Enables structured, validated agent responses
///
/// This is the YAML-friendly version for config files. It gets converted
/// to `crate::schema::OutputSchema` for runtime use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchemaSpec {
    /// JSON Schema type (object, array, string, number, boolean)
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Properties for object type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,

    /// Required properties for object type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Items schema for array type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<serde_json::Value>>,

    /// Enum values for string type
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Description of the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allow additional properties (default: false for strict validation)
    #[serde(default, rename = "additionalProperties")]
    pub additional_properties: Option<bool>,

    /// Validation mode: strict (default), lenient, coerce
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_mode: Option<String>,

    /// Behavior on validation error: fail (default), retry, passthrough
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_validation_error: Option<String>,

    /// Max retries if on_validation_error is "retry"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,

    /// Additional JSON Schema properties (oneOf, anyOf, etc.)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl OutputSchemaSpec {
    /// Convert to JSON Schema Value for validation
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut schema = serde_json::json!({
            "type": self.schema_type
        });

        if let Some(props) = &self.properties {
            schema["properties"] = serde_json::json!(props);
        }

        if let Some(req) = &self.required {
            schema["required"] = serde_json::json!(req);
        }

        if let Some(items) = &self.items {
            schema["items"] = serde_json::json!(items);
        }

        if let Some(enum_vals) = &self.enum_values {
            schema["enum"] = serde_json::json!(enum_vals);
        }

        if let Some(desc) = &self.description {
            schema["description"] = serde_json::json!(desc);
        }

        if let Some(additional) = &self.additional_properties {
            schema["additionalProperties"] = serde_json::json!(additional);
        }

        // Merge extra fields (oneOf, anyOf, etc.)
        if let serde_json::Value::Object(ref mut map) = schema {
            for (key, value) in &self.extra {
                map.insert(key.clone(), value.clone());
            }
        }

        schema
    }

    /// Get validation mode (defaults to "strict")
    pub fn get_validation_mode(&self) -> &str {
        self.validation_mode.as_deref().unwrap_or("strict")
    }

    /// Get error handling behavior (defaults to "fail")
    pub fn get_error_behavior(&self) -> &str {
        self.on_validation_error.as_deref().unwrap_or("fail")
    }

    /// Generate schema instructions for the LLM
    pub fn to_instructions(&self) -> String {
        let schema = self.to_json_schema();
        format!(
            "You MUST respond with valid JSON matching this schema:\n```json\n{}\n```\nDo not include any text outside the JSON object.",
            serde_json::to_string_pretty(&schema).unwrap_or_default()
        )
    }
}

/// Convert YAML-friendly OutputSchemaSpec to runtime OutputSchema
impl From<OutputSchemaSpec> for crate::schema::OutputSchema {
    fn from(spec: OutputSchemaSpec) -> Self {
        // Get validation mode before moving spec
        let strict = spec.get_validation_mode() == "strict";
        let description = spec.description.clone();

        let schema = spec.to_json_schema();
        let mut output = crate::schema::OutputSchema::from_json_schema(schema);

        // Transfer description if present
        if let Some(desc) = description {
            output = output.with_description(desc);
        }

        // Set strict mode based on validation_mode
        output = output.with_strict(strict);

        output
    }
}

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

// ---------------------------------------------------------------------------
// Vector memory configuration (v2.0 — distinct from the legacy MemorySpec)
// ---------------------------------------------------------------------------

/// Backend type for vector memory.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryBackendType {
    /// SQLite-backed vector store (default, no external dependencies).
    #[default]
    Sqlite,
}

fn default_memory_backend() -> MemoryBackendType {
    MemoryBackendType::Sqlite
}

fn default_top_k() -> usize {
    5
}

/// Vector memory configuration for an agent.
///
/// Controls whether the agent recalls relevant past run contexts before each
/// run and stores its final answer after each run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorMemoryConfig {
    /// Enable vector memory for this agent.
    #[serde(default)]
    pub enabled: bool,
    /// Storage backend (sqlite is the only supported backend in v2.0).
    #[serde(default = "default_memory_backend")]
    pub backend: MemoryBackendType,
    /// Path to the SQLite database file.
    /// Default: `./memory/<agent-name>.db`.
    pub db_path: Option<String>,
    /// Number of similar past contexts to retrieve per run.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Optional override for the embedding model.
    /// Default: use the agent's configured model.
    pub embed_model: Option<String>,
}

fn default_research_iterations() -> usize {
    3
}

/// Research phase configuration — pre-loop fact-gathering step (CORE-06).
///
/// When enabled, a proactive fact-gathering pass runs before the main ReAct
/// loop, injecting gathered facts as a `## Research Context` block into the
/// system prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchPhaseConfig {
    /// Enable the research phase before the main ReAct loop.
    #[serde(default)]
    pub enabled: bool,
    /// Maximum fact-gathering iterations (default: 3).
    #[serde(default = "default_research_iterations")]
    pub max_iterations: usize,
}

/// Tool specification - unified way to configure both built-in and MCP tools
///
/// Supports multiple formats:
/// 1. Simple string: `"shell"` - built-in tool with defaults
/// 2. Type-based: `{type: "Shell", config: {allowed_commands: [...]}}`
/// 3. Object with source: `{name: "kubectl_get", source: "builtin", config: {...}}`
/// 4. MCP tool: `{name: "read_file", source: "mcp", server: "filesystem"}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolSpec {
    /// Simple tool name (for backward compatibility)
    /// Assumes built-in if the tool exists, otherwise tries MCP
    Simple(String),

    /// Type-based tool specification (type: Shell/MCP/HTTP)
    /// Format: {type: "Shell", config: {allowed_commands: [...]}}
    TypeBased(TypeBasedToolSpec),

    /// Fully qualified tool specification
    Qualified(QualifiedToolSpec),
}

/// Type-based tool specification
/// Supports: Shell, MCP, HTTP tool types with their configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeBasedToolSpec {
    /// Tool type: Shell, MCP, HTTP
    #[serde(rename = "type")]
    pub tool_type: TypeBasedToolType,

    /// Tool-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Tool types for type-based specification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TypeBasedToolType {
    /// Shell command execution with restrictions
    Shell,
    /// MCP server tool
    MCP,
    /// HTTP API tool
    HTTP,
}

impl ToolSpec {
    /// Get the tool name
    pub fn name(&self) -> &str {
        match self {
            ToolSpec::Simple(name) => name,
            ToolSpec::TypeBased(spec) => match spec.tool_type {
                TypeBasedToolType::Shell => "shell",
                TypeBasedToolType::MCP => spec
                    .config
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("mcp"),
                TypeBasedToolType::HTTP => "http",
            },
            ToolSpec::Qualified(spec) => &spec.name,
        }
    }

    /// Check if this is explicitly a built-in tool
    pub fn is_builtin(&self) -> bool {
        match self {
            ToolSpec::Simple(_) => true, // default to builtin for simple names
            ToolSpec::TypeBased(spec) => spec.tool_type == TypeBasedToolType::Shell,
            ToolSpec::Qualified(spec) => spec.source == ToolSource::Builtin,
        }
    }

    /// Check if this is explicitly an MCP tool
    pub fn is_mcp(&self) -> bool {
        match self {
            ToolSpec::Simple(_) => false,
            ToolSpec::TypeBased(spec) => spec.tool_type == TypeBasedToolType::MCP,
            ToolSpec::Qualified(spec) => spec.source == ToolSource::Mcp,
        }
    }

    /// Check if this is an HTTP tool
    pub fn is_http(&self) -> bool {
        match self {
            ToolSpec::Simple(_) => false,
            ToolSpec::TypeBased(spec) => spec.tool_type == TypeBasedToolType::HTTP,
            ToolSpec::Qualified(_) => false,
        }
    }

    /// Check if this is a Shell tool (type-based)
    pub fn is_shell(&self) -> bool {
        match self {
            ToolSpec::Simple(name) => name == "shell",
            ToolSpec::TypeBased(spec) => spec.tool_type == TypeBasedToolType::Shell,
            ToolSpec::Qualified(spec) => spec.name == "shell",
        }
    }

    /// Get the tool type (for type-based specs)
    pub fn tool_type(&self) -> Option<TypeBasedToolType> {
        match self {
            ToolSpec::TypeBased(spec) => Some(spec.tool_type),
            _ => None,
        }
    }

    /// Get the MCP server name (if this is an MCP tool)
    pub fn mcp_server(&self) -> Option<&str> {
        match self {
            ToolSpec::Simple(_) => None,
            ToolSpec::TypeBased(spec) => {
                if spec.tool_type == TypeBasedToolType::MCP {
                    spec.config.get("name").and_then(|v| v.as_str())
                } else {
                    None
                }
            }
            ToolSpec::Qualified(spec) => spec.server.as_deref(),
        }
    }

    /// Get tool configuration
    pub fn config(&self) -> Option<&serde_json::Value> {
        match self {
            ToolSpec::Simple(_) => None,
            ToolSpec::TypeBased(spec) => Some(&spec.config),
            ToolSpec::Qualified(spec) => spec.config.as_ref(),
        }
    }

    /// Get type-based spec if this is a type-based tool
    pub fn type_based_spec(&self) -> Option<&TypeBasedToolSpec> {
        match self {
            ToolSpec::TypeBased(spec) => Some(spec),
            _ => None,
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

    /// Output schema for structured responses (JSON Schema format)
    /// When specified, agent responses will be validated against this schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<OutputSchemaSpec>,

    /// Routing configuration for config-driven intent routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingConfig>,

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

    /// Get type-based Shell tools
    pub fn type_based_shell_tools(&self) -> Vec<&TypeBasedToolSpec> {
        self.tools
            .iter()
            .filter_map(|t| match t {
                ToolSpec::TypeBased(spec) if spec.tool_type == TypeBasedToolType::Shell => {
                    Some(spec)
                }
                _ => None,
            })
            .collect()
    }

    /// Get type-based MCP tools
    pub fn type_based_mcp_tools(&self) -> Vec<&TypeBasedToolSpec> {
        self.tools
            .iter()
            .filter_map(|t| match t {
                ToolSpec::TypeBased(spec) if spec.tool_type == TypeBasedToolType::MCP => Some(spec),
                _ => None,
            })
            .collect()
    }

    /// Get type-based HTTP tools
    pub fn type_based_http_tools(&self) -> Vec<&TypeBasedToolSpec> {
        self.tools
            .iter()
            .filter_map(|t| match t {
                ToolSpec::TypeBased(spec) if spec.tool_type == TypeBasedToolType::HTTP => {
                    Some(spec)
                }
                _ => None,
            })
            .collect()
    }

    /// Check if there are any type-based tools
    pub fn has_type_based_tools(&self) -> bool {
        self.tools.iter().any(|t| matches!(t, ToolSpec::TypeBased(_)))
    }

    /// Convert type-based MCP tools to McpServerConfig
    /// Returns configs that can be used with create_mcp_executor_from_config
    pub fn type_based_mcp_to_server_configs(&self) -> Vec<crate::mcp::McpServerConfig> {
        self.type_based_mcp_tools()
            .iter()
            .filter_map(|spec| {
                let config = &spec.config;
                let name = config.get("name")?.as_str()?;

                // Extract command - can be string or array
                let command = config.get("command").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                });

                // Extract args from command array (skip first element)
                let args: Vec<String> = config
                    .get("command")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .skip(1)
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Extract env vars
                let env: std::collections::HashMap<String, String> = config
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();

                Some(crate::mcp::McpServerConfig {
                    name: name.to_string(),
                    transport: crate::mcp::McpTransport::Stdio,
                    command,
                    args,
                    env,
                    endpoint: None,
                    tools: vec![],
                    init_options: None,
                    timeout_secs: config
                        .get("timeout_seconds")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(30),
                    auto_reconnect: true,
                })
            })
            .collect()
    }

    /// Get Shell tool configuration (allowed_commands, working_directory, etc.)
    pub fn shell_tool_config(&self) -> Option<ShellToolConfig> {
        self.type_based_shell_tools().first().map(|spec| {
            let config = &spec.config;
            ShellToolConfig {
                allowed_commands: config
                    .get("allowed_commands")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                working_directory: config
                    .get("working_directory")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                timeout_seconds: config
                    .get("timeout_seconds")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32),
            }
        })
    }

    /// Get HTTP tool configuration
    pub fn http_tool_config(&self) -> Option<HttpToolConfig> {
        self.type_based_http_tools().first().map(|spec| {
            let config = &spec.config;
            HttpToolConfig {
                base_url: config
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                timeout_seconds: config
                    .get("timeout_seconds")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32),
                allowed_methods: config
                    .get("allowed_methods")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        })
    }
}

/// Shell tool configuration extracted from type-based spec
#[derive(Debug, Clone, Default)]
pub struct ShellToolConfig {
    pub allowed_commands: Vec<String>,
    pub working_directory: Option<String>,
    pub timeout_seconds: Option<u32>,
}

/// HTTP tool configuration extracted from type-based spec
#[derive(Debug, Clone, Default)]
pub struct HttpToolConfig {
    pub base_url: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub allowed_methods: Vec<String>,
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
    #[allow(dead_code)]
    api_version: String,  // Required for K8s format
    #[allow(dead_code)]
    kind: String,         // Required for K8s format
    metadata: KubernetesMetadata,
    spec: AgentSpec,
}

#[derive(Debug, Clone, Deserialize)]
struct KubernetesMetadata {
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    labels: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
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
    output_schema: Option<OutputSchemaSpec>,
    routing: Option<RoutingConfig>,
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
    output_schema: Option<OutputSchemaSpec>,
    routing: Option<RoutingConfig>,
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
                output_schema: flat.output_schema,
                routing: flat.routing,
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
                    output_schema: k8s.spec.output_schema,
                    routing: k8s.spec.routing,
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

fn default_routing_priority() -> f32 {
    0.8
}

/// Routing configuration for config-driven intent routing.
/// Agents declare keywords and domains so the router can match
/// user messages directly to specialists without an LLM call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Keywords that trigger routing to this agent
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Domain categories (for future squad/channel mapping)
    #[serde(default)]
    pub domains: Vec<String>,
    /// Routing priority (0.0-1.0). Higher = stronger match. Default 0.8
    #[serde(default = "default_routing_priority")]
    pub priority: f32,
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
    fn test_tool_spec_type_based_shell() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - type: Shell
                config:
                  allowed_commands:
                    - kubectl
                    - helm
                  working_directory: /tmp
                  timeout_seconds: 30
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.tools[0].name(), "shell");
        assert!(config.tools[0].is_shell());
        assert!(config.tools[0].is_builtin());
        assert!(config.tools[0].config().is_some());

        let config_val = config.tools[0].config().unwrap();
        assert!(config_val.get("allowed_commands").is_some());
    }

    #[test]
    fn test_tool_spec_type_based_mcp() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - type: MCP
                config:
                  name: kubectl-mcp
                  command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
                  env:
                    KUBECONFIG: "${KUBECONFIG}"
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 1);
        assert!(config.tools[0].is_mcp());
        assert_eq!(config.tools[0].mcp_server(), Some("kubectl-mcp"));
    }

    #[test]
    fn test_tool_spec_type_based_http() {
        let yaml = r#"
            name: test-agent
            model: gpt-4
            tools:
              - type: HTTP
                config:
                  base_url: http://localhost:8080
                  timeout_seconds: 10
                  allowed_methods: [GET, POST]
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.tools[0].name(), "http");
        assert!(config.tools[0].is_http());

        let config_val = config.tools[0].config().unwrap();
        assert_eq!(config_val.get("base_url").unwrap(), "http://localhost:8080");
    }

    #[test]
    fn test_tool_spec_type_based_mixed() {
        // This is the exact format from the user's final_agents.yaml
        let yaml = r#"
            apiVersion: aof.dev/v1
            kind: Agent
            metadata:
              name: k8s-helper
              labels:
                purpose: operations
            spec:
              model: google:gemini-2.5-flash
              instructions: You are a K8s helper.
              tools:
                - type: Shell
                  config:
                    allowed_commands:
                      - kubectl
                      - helm
                    working_directory: /tmp
                    timeout_seconds: 30
                - type: MCP
                  config:
                    name: kubectl-mcp
                    command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
                - type: HTTP
                  config:
                    base_url: http://localhost
                    timeout_seconds: 10
              memory:
                type: File
                config:
                  path: ./k8s-helper-memory.json
                  max_messages: 50
        "#;
        let config: AgentConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.name, "k8s-helper");
        assert_eq!(config.tools.len(), 3);

        // First tool: Shell
        assert!(config.tools[0].is_shell());
        assert!(config.tools[0].is_builtin());

        // Second tool: MCP
        assert!(config.tools[1].is_mcp());
        assert_eq!(config.tools[1].mcp_server(), Some("kubectl-mcp"));

        // Third tool: HTTP
        assert!(config.tools[2].is_http());

        // Memory
        assert!(config.memory.is_some());
        let memory = config.memory.as_ref().unwrap();
        assert!(memory.is_file());
        assert_eq!(memory.path(), Some("./k8s-helper-memory.json".to_string()));
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

// =============================================================================
// GitAgent-compatible types (v0.1.0 directory format)
// =============================================================================
//
// These types implement the agent directory specification:
//   - docs/spec/agent-yaml-v1.md       (AgentManifest)
//   - docs/spec/agent-directory-structure.md (AgentDefinition, DirectoryLoader)
//
// The key conceptual split:
//   AgentManifest   = thin manifest from agent.yaml (metadata + model only)
//   AgentDefinition = assembled runtime type (manifest + SOUL.md + RULES.md + skills + tools)
//   DirectoryLoader = reads a directory from filesystem and assembles AgentDefinition
//   AgentLoader     = unified entry point supporting both directory and flat YAML formats

use std::path::Path;

// ---------------------------------------------------------------------------
// AgentManifest — the minimal agent.yaml manifest
// ---------------------------------------------------------------------------

/// Minimal `agent.yaml` manifest for a GitAgent-compatible agent directory.
///
/// Carries only metadata and model preference. All behavior lives in markdown
/// files (`SOUL.md`, `RULES.md`, `skills/`) that the runtime assembles into
/// a system prompt at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    /// OpenAgentiX agent spec version (e.g. "0.1.0").
    pub spec_version: String,
    /// Agent name — DNS-compatible, lowercase, hyphens only.
    pub name: String,
    /// Optional semver agent version.
    pub version: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Model preference for this agent.
    pub model: Option<AgentModelConfig>,
    /// Inherit base agent definition from a remote git URL.
    pub extends: Option<String>,
    /// External sub-agent dependencies to fetch and mount.
    #[serde(default)]
    pub dependencies: Vec<AgentDependency>,
    /// MCP server configurations for this agent.
    #[serde(default)]
    pub mcp_servers: Vec<crate::mcp::McpServerConfig>,
    /// Trigger configurations — what events fire this agent.
    #[serde(default)]
    pub triggers: Vec<AgentTriggerConfig>,
    /// Notification routing — where to send run results.
    #[serde(default)]
    pub notifications: Vec<AgentNotificationConfig>,
    /// Vector memory configuration (v2.0).
    #[serde(default)]
    pub vector_memory: VectorMemoryConfig,
    /// Research phase configuration (v2.0 / CORE-06).
    #[serde(default)]
    pub research_phase: ResearchPhaseConfig,
}

/// Model configuration within an `AgentManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelConfig {
    /// Preferred model in `provider/model` format (e.g. `anthropic/claude-sonnet-4-6`).
    pub preferred: Option<String>,
}

/// A declared external sub-agent dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDependency {
    /// Local alias within `agents/`.
    pub name: String,
    /// Git URL or local path to the sub-agent directory.
    pub source: String,
    /// Semver range (e.g. `^1.0.0`).
    pub version: Option<String>,
    /// Mount point inside `agents/` (defaults to `agents/<name>`).
    pub mount: Option<String>,
}

impl AgentManifest {
    /// Parse an `agent.yaml` YAML string.
    ///
    /// Uses `serde_path_to_error` to provide precise field-level error messages.
    pub fn from_yaml(content: &str) -> crate::AgentixResult<Self> {
        let de = serde_yaml::Deserializer::from_str(content);
        serde_path_to_error::deserialize(de).map_err(|e| {
            crate::AgentixError::yaml_parse(e.path().to_string(), e.inner().to_string())
        })
    }

    /// Validate the manifest fields against the spec rules.
    ///
    /// Returns `Err(AgentixError::SpecValidation(...))` on the first violation.
    pub fn validate(&self) -> crate::AgentixResult<()> {
        // name: ^[a-z][a-z0-9-]*[a-z0-9]$, max 63 chars
        validate_agent_name(&self.name)?;

        // model.preferred must contain exactly one '/' if present
        if let Some(model) = &self.model {
            if let Some(preferred) = &model.preferred {
                let slash_count = preferred.chars().filter(|&c| c == '/').count();
                if slash_count != 1 {
                    return Err(crate::AgentixError::SpecValidation(
                        format!(
                            "model.preferred: must be \"provider/model\" format (e.g. \"anthropic/claude-sonnet-4-6\"), got \"{}\"",
                            preferred
                        )
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Validate an agent name against the spec regex `^[a-z][a-z0-9-]*[a-z0-9]$`, max 63 chars.
fn validate_agent_name(name: &str) -> crate::AgentixResult<()> {
    use regex::Regex;
    // Allow single lowercase char too (min length 1)
    let re = Regex::new(r"^[a-z][a-z0-9-]*[a-z0-9]$|^[a-z]$").unwrap();
    if name.len() > 63 {
        return Err(crate::AgentixError::SpecValidation(format!(
            "name: must be at most 63 characters, got {} characters",
            name.len()
        )));
    }
    if !re.is_match(name) {
        return Err(crate::AgentixError::SpecValidation(format!(
            "name: must match ^[a-z][a-z0-9-]*[a-z0-9]$ (lowercase alphanumeric with hyphens), got \"{}\"",
            name
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// AgentDefinition — the assembled runtime type
// ---------------------------------------------------------------------------

/// Execution mode for the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentMode {
    /// The agent asks for human approval before each action.
    Manual,
    /// The agent executes autonomously but pauses on risky actions.
    SemiAutonomous,
    /// The agent executes fully autonomously without pauses.
    #[default]
    Autonomous,
}

/// A reusable capability module loaded from `skills/<name>/SKILL.md`.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    /// Skill name (directory name under `skills/`).
    pub name: String,
    /// Content of `SKILL.md`.
    pub content: String,
}

/// Tool type discriminator from `tools/*.yaml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectoryToolType {
    /// Execute a specific CLI binary.
    Cli,
    /// Connect to an MCP server.
    Mcp,
    /// Execute an arbitrary shell command.
    Shell,
    /// Execute a WASM module in the sandboxed executor.
    Wasm,
}

/// A tool definition loaded from `tools/<name>.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    /// Tool name.
    pub name: String,
    /// Tool type: cli, mcp, or shell.
    #[serde(rename = "type")]
    pub tool_type: DirectoryToolType,
    /// CLI binary or shell command.
    pub command: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// MCP server name (for `type: mcp`).
    pub server: Option<String>,
    /// Argument templates using `{{var}}` syntax.
    #[serde(default)]
    pub args: Vec<String>,
}

/// Trigger configuration entry from agent YAML `triggers:` field.
///
/// Each entry in the `triggers:` list specifies a trigger type and its options.
/// The gateway reads these at agent load time and registers the appropriate
/// `TriggerTrait` implementations in `TriggerRunRegistry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTriggerConfig {
    /// Trigger type: "cron", "webhook", "github", "jira", "slack", "discord", "telegram", "agent"
    #[serde(rename = "type")]
    pub trigger_type: String,

    /// Cron expression (for type=cron). Standard 5-field format: `min hour dom month dow`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,

    /// Webhook/GitHub/Jira secret for HMAC signature verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_secret: Option<String>,

    /// GitHub/Jira event types to filter on (e.g., ["pull_request", "push"]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    /// Bot token for Slack/Discord/Telegram triggers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,

    /// Signing secret for Slack request verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secret: Option<String>,

    /// Discord application ID (for Discord triggers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,

    /// URL path for webhook triggers (defaults to /webhooks/<trigger_id>).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Notification destination configuration entry from `notifications:` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNotificationConfig {
    /// Notification type: "webhook", "log"
    #[serde(rename = "type")]
    pub notification_type: String,

    /// URL for webhook notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The assembled runtime agent type, built from a directory or flat YAML.
///
/// This is the primary type consumed by the ReAct loop and gateway.
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    /// Agent name from `agent.yaml`.
    pub name: String,
    /// Semver version from `agent.yaml`.
    pub version: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Preferred model in `provider/model` format.
    pub model_preferred: Option<String>,
    /// Content of `SOUL.md` — the primary system prompt.
    pub soul_content: String,
    /// Content of `RULES.md` — hard constraints (optional).
    pub rules_content: Option<String>,
    /// Loaded skill modules from `skills/*/SKILL.md`.
    pub skills: Vec<SkillEntry>,
    /// Tool definitions from `tools/*.yaml`.
    pub tools: Vec<ToolEntry>,
    /// Loaded sub-agent definitions from `agents/*/`.
    pub sub_agents: Vec<AgentDefinition>,
    /// MCP server configurations for this agent.
    pub mcp_servers: Vec<crate::mcp::McpServerConfig>,
    /// Maximum ReAct loop iterations (from workspace defaults or built-in).
    pub max_iterations: u32,
    /// Wall-clock timeout in seconds.
    pub timeout_secs: u64,
    /// Execution mode.
    pub mode: AgentMode,
    /// Trigger configurations from agent YAML `triggers:` field.
    pub triggers: Vec<AgentTriggerConfig>,
    /// Notification routing configurations from `notifications:` field.
    pub notifications: Vec<AgentNotificationConfig>,
    /// Vector memory configuration (v2.0 — CORE-04).
    pub vector_memory: VectorMemoryConfig,
    /// Research phase configuration (v2.0 — CORE-06).
    pub research_phase: ResearchPhaseConfig,
}

impl AgentDefinition {
    /// Assemble the full system prompt from SOUL + RULES + skills.
    ///
    /// Assembly order (per spec):
    /// 1. `SOUL.md` content (base)
    /// 2. `## Constraints` + `RULES.md` (if present)
    /// 3. `## Skill: <name>` + `SKILL.md` for each skill (alphabetical)
    pub fn resolved_system_prompt(&self) -> String {
        let mut parts = vec![self.soul_content.clone()];

        if let Some(rules) = &self.rules_content {
            parts.push(format!("\n\n## Constraints\n\n{}", rules));
        }

        for skill in &self.skills {
            parts.push(format!("\n\n## Skill: {}\n\n{}", skill.name, skill.content));
        }

        parts.concat()
    }

    /// Apply workspace defaults to this definition.
    ///
    /// Workspace values override built-in defaults but do NOT override
    /// values explicitly set per-agent (model_preferred is only set from
    /// workspace when the agent has none).
    pub fn apply_workspace_defaults(&mut self, workspace: &crate::WorkspaceConfig) {
        let defaults = &workspace.spec.defaults;

        if self.model_preferred.is_none() {
            if let Some(model) = &defaults.model {
                self.model_preferred = Some(model.clone());
            }
        }

        if let Some(max_iter) = defaults.max_iterations {
            self.max_iterations = max_iter;
        }

        if let Some(timeout) = &defaults.timeout {
            self.timeout_secs = parse_duration_str(timeout).unwrap_or(300);
        }

        if let Some(mode) = defaults.mode {
            self.mode = mode;
        }
    }
}

/// Parse a duration string like "5m", "30s", "1h" into seconds.
fn parse_duration_str(s: &str) -> Option<u64> {
    if let Some(s) = s.strip_suffix('s') {
        s.parse::<u64>().ok()
    } else if let Some(s) = s.strip_suffix('m') {
        s.parse::<u64>().ok().map(|n| n * 60)
    } else if let Some(s) = s.strip_suffix('h') {
        s.parse::<u64>().ok().map(|n| n * 3600)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// DirectoryLoader — loads an agent directory from filesystem
// ---------------------------------------------------------------------------

/// Loads an agent directory from the filesystem and assembles an `AgentDefinition`.
///
/// Implements the assembly algorithm from `docs/spec/agent-directory-structure.md`.
pub struct DirectoryLoader;

impl DirectoryLoader {
    /// Load and assemble an `AgentDefinition` from a directory path.
    ///
    /// # Required files
    /// - `agent.yaml` — minimal manifest
    /// - `SOUL.md` — identity and system prompt
    ///
    /// # Optional files
    /// - `RULES.md`, `skills/*/SKILL.md`, `tools/*.yaml`, `agents/*/`
    pub fn load(dir: &Path) -> crate::AgentixResult<AgentDefinition> {
        // 1. Read and parse agent.yaml
        let manifest_path = dir.join("agent.yaml");
        if !manifest_path.exists() {
            return Err(crate::AgentixError::SpecValidation(format!(
                "agent.yaml not found in {}",
                dir.display()
            )));
        }
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest = AgentManifest::from_yaml(&manifest_content)?;
        manifest.validate()?;

        // 2. Read SOUL.md (required)
        let soul_path = dir.join("SOUL.md");
        if !soul_path.exists() {
            return Err(crate::AgentixError::SpecValidation(format!(
                "SOUL.md not found for agent \"{}\"",
                manifest.name
            )));
        }
        let soul_content = std::fs::read_to_string(&soul_path)?;
        if soul_content.trim().is_empty() {
            return Err(crate::AgentixError::SpecValidation(format!(
                "SOUL.md is empty for agent \"{}\"",
                manifest.name
            )));
        }

        // 3. Read RULES.md (optional)
        let rules_path = dir.join("RULES.md");
        let rules_content = if rules_path.exists() {
            let content = std::fs::read_to_string(&rules_path)?;
            if content.trim().is_empty() {
                None
            } else {
                Some(content)
            }
        } else {
            None
        };

        // 4. Load skills/*/SKILL.md (alphabetical)
        let mut skills = Vec::new();
        let skills_dir = dir.join("skills");
        if skills_dir.is_dir() {
            let mut skill_dirs: Vec<_> = std::fs::read_dir(&skills_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            skill_dirs.sort_by_key(|e| e.file_name());
            for entry in skill_dirs {
                let skill_name = entry.file_name().to_string_lossy().to_string();
                let skill_md = entry.path().join("SKILL.md");
                if skill_md.exists() {
                    let content = std::fs::read_to_string(&skill_md)?;
                    skills.push(SkillEntry { name: skill_name, content });
                }
            }
        }

        // 5. Load tools/*.yaml
        let mut tools = Vec::new();
        let tools_dir = dir.join("tools");
        if tools_dir.is_dir() {
            let mut tool_files: Vec<_> = std::fs::read_dir(&tools_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let p = e.path();
                    p.is_file()
                        && p.extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s == "yaml" || s == "yml")
                            .unwrap_or(false)
                })
                .collect();
            tool_files.sort_by_key(|e| e.file_name());
            for entry in tool_files {
                let content = std::fs::read_to_string(entry.path())?;
                let de = serde_yaml::Deserializer::from_str(&content);
                let tool: ToolEntry = serde_path_to_error::deserialize(de).map_err(|e| {
                    crate::AgentixError::yaml_parse(
                        format!("tools/{}: {}", entry.file_name().to_string_lossy(), e.path()),
                        e.inner().to_string(),
                    )
                })?;
                tools.push(tool);
            }
        }

        // 6. Recursively load agents/* sub-agents
        let mut sub_agents = Vec::new();
        let agents_dir = dir.join("agents");
        if agents_dir.is_dir() {
            let mut sub_dirs: Vec<_> = std::fs::read_dir(&agents_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            sub_dirs.sort_by_key(|e| e.file_name());
            for entry in sub_dirs {
                // Only load if it's a valid agent dir (has agent.yaml)
                if entry.path().join("agent.yaml").exists() {
                    let sub = DirectoryLoader::load(&entry.path())?;
                    sub_agents.push(sub);
                }
            }
        }

        Ok(AgentDefinition {
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            model_preferred: manifest.model.and_then(|m| m.preferred),
            soul_content,
            rules_content,
            skills,
            tools,
            sub_agents,
            mcp_servers: manifest.mcp_servers,
            max_iterations: 10,
            timeout_secs: 300,
            mode: AgentMode::Autonomous,
            triggers: manifest.triggers,
            notifications: manifest.notifications,
            vector_memory: manifest.vector_memory,
            research_phase: manifest.research_phase,
        })
    }
}

// ---------------------------------------------------------------------------
// FlatYamlLoader — backward compat for apiVersion: openagentix.dev/v1
// ---------------------------------------------------------------------------

/// Internal: flat YAML agent spec (the old Kubernetes-style format).
#[derive(Debug, Deserialize)]
struct FlatAgentSpec {
    #[serde(rename = "apiVersion")]
    #[allow(dead_code)]
    api_version: Option<String>,
    #[allow(dead_code)]
    kind: Option<String>,
    metadata: Option<FlatAgentMetadata>,
    spec: Option<FlatAgentSpecBody>,
}

#[derive(Debug, Deserialize)]
struct FlatAgentMetadata {
    name: String,
}

#[derive(Debug, Deserialize)]
struct FlatAgentSpecBody {
    model: Option<String>,
    system_prompt: Option<String>,
    instructions: Option<String>,
    #[serde(default)]
    tools: Vec<serde_yaml::Value>,
    #[serde(default)]
    triggers: Vec<AgentTriggerConfig>,
    #[serde(default)]
    notifications: Vec<AgentNotificationConfig>,
    #[serde(default)]
    vector_memory: VectorMemoryConfig,
    #[serde(default)]
    research_phase: ResearchPhaseConfig,
}

/// Loads flat YAML agents (`apiVersion: openagentix.dev/v1 / kind: Agent`) into `AgentDefinition`.
pub struct FlatYamlLoader;

impl FlatYamlLoader {
    /// Load a flat YAML file and convert it to an `AgentDefinition`.
    pub fn load(path: &Path) -> crate::AgentixResult<AgentDefinition> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    /// Parse a flat YAML string directly into an `AgentDefinition`.
    ///
    /// Used by the gateway API where the YAML content arrives as a request body.
    pub fn load_from_str(content: &str) -> crate::AgentixResult<AgentDefinition> {
        let de = serde_yaml::Deserializer::from_str(content);
        let flat: FlatAgentSpec = serde_path_to_error::deserialize(de).map_err(|e| {
            crate::AgentixError::yaml_parse(e.path().to_string(), e.inner().to_string())
        })?;

        let meta = flat.metadata.ok_or_else(|| {
            crate::AgentixError::SpecValidation("flat YAML missing metadata.name".to_string())
        })?;

        let spec = flat.spec.unwrap_or_else(|| FlatAgentSpecBody {
            model: None,
            system_prompt: None,
            instructions: None,
            tools: vec![],
            triggers: vec![],
            notifications: vec![],
            vector_memory: VectorMemoryConfig::default(),
            research_phase: ResearchPhaseConfig::default(),
        });

        // system_prompt and instructions are aliases
        let soul_content = spec
            .system_prompt
            .or(spec.instructions)
            .unwrap_or_default();

        // Convert flat tool entries to ToolEntry (best-effort)
        let mut tools = Vec::new();
        for val in spec.tools {
            if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                let tool_type_str = val
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cli");
                let tool_type = match tool_type_str {
                    "mcp" => DirectoryToolType::Mcp,
                    "shell" => DirectoryToolType::Shell,
                    _ => DirectoryToolType::Cli,
                };
                tools.push(ToolEntry {
                    name: name.to_string(),
                    tool_type,
                    command: val
                        .get("command")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    description: val
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    server: val
                        .get("server")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    args: vec![],
                });
            }
        }

        Ok(AgentDefinition {
            name: meta.name,
            version: None,
            description: None,
            model_preferred: spec.model,
            soul_content,
            rules_content: None,
            skills: vec![],
            tools,
            sub_agents: vec![],
            mcp_servers: vec![],
            max_iterations: 10,
            timeout_secs: 300,
            mode: AgentMode::Autonomous,
            triggers: spec.triggers,
            notifications: spec.notifications,
            vector_memory: spec.vector_memory,
            research_phase: spec.research_phase,
        })
    }
}

// ---------------------------------------------------------------------------
// AgentLoader — unified entry point
// ---------------------------------------------------------------------------

/// The detected format of an agent path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFormat {
    /// A directory containing `agent.yaml` (GitAgent-compatible).
    Directory,
    /// A flat `*.yaml` file with `kind: Agent` (backward compat).
    FlatYaml,
}

/// Unified agent loader — detects format and delegates to the appropriate loader.
pub struct AgentLoader;

impl AgentLoader {
    /// Detect the format of an agent path.
    ///
    /// - Directory path → `AgentFormat::Directory`
    /// - `*.yaml` / `*.yml` file → `AgentFormat::FlatYaml`
    pub fn detect_format(path: &Path) -> AgentFormat {
        if path.is_dir() {
            AgentFormat::Directory
        } else {
            AgentFormat::FlatYaml
        }
    }

    /// Load an `AgentDefinition` from a path, auto-detecting the format.
    pub fn load(path: &Path) -> crate::AgentixResult<AgentDefinition> {
        match Self::detect_format(path) {
            AgentFormat::Directory => DirectoryLoader::load(path),
            AgentFormat::FlatYaml => FlatYamlLoader::load(path),
        }
    }
}
