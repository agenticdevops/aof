use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::AofResult;

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
}

/// Message in conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::ToolCall>>,
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
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(Message {
            role,
            content: content.into(),
            tool_calls: None,
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

    /// Model to use
    pub model: String,

    /// Tools available to agent
    #[serde(default)]
    pub tools: Vec<String>,

    /// Memory backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

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
    #[serde(alias = "system_prompt")]
    instructions: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    memory: Option<String>,
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
    #[serde(default)]
    tools: Vec<String>,
    memory: Option<String>,
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
                tools: flat.tools,
                memory: flat.memory,
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
                    tools: k8s.spec.tools,
                    memory: k8s.spec.memory,
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
        assert_eq!(config.tools, vec!["read_file", "write_file"]);
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, Some(4096));
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
}
