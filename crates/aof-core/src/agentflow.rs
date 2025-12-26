// AOF Core - AgentFlow configuration types
//
// AgentFlow defines multi-step workflows with conditional routing, approval flows,
// and interactive responses. Flows are triggered via Trigger CRDs which contain
// command bindings that route to specific flows.
//
// Architecture:
//   Trigger CRD (platform + commands) → references → AgentFlow (workflow logic)
//
// This separation allows:
// - Same flow used from multiple platforms (Slack, Telegram, etc.)
// - Different commands routing to same flow
// - Cleaner separation of concerns

use crate::{McpServerConfig, agent::ToolSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AgentFlow - Multi-step agent workflow
///
/// Example:
/// ```yaml
/// apiVersion: aof.dev/v1
/// kind: AgentFlow
/// metadata:
///   name: deploy-flow
/// spec:
///   description: "Deployment workflow with approval"
///   nodes:
///     - id: validate
///       type: Agent
///       config:
///         agent: validator
///     - id: approve
///       type: Approval
///       config:
///         message: "Deploy to production?"
///     - id: deploy
///       type: Agent
///       config:
///         agent: deployer
///   connections:
///     - from: start
///       to: validate
///     - from: validate
///       to: approve
///     - from: approve
///       to: deploy
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFlow {
    /// API version (e.g., "aof.dev/v1")
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind, always "AgentFlow"
    #[serde(default = "default_agentflow_kind")]
    pub kind: String,

    /// Flow metadata
    pub metadata: AgentFlowMetadata,

    /// Flow specification
    pub spec: AgentFlowSpec,
}

fn default_api_version() -> String {
    "aof.dev/v1".to_string()
}

fn default_agentflow_kind() -> String {
    "AgentFlow".to_string()
}

/// AgentFlow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFlowMetadata {
    /// Flow name
    pub name: String,

    /// Namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Labels for categorization
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Annotations for additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

/// AgentFlow specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFlowSpec {
    /// Human-readable description of the flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Workflow nodes
    pub nodes: Vec<FlowNode>,

    /// Node connections (edges in the graph)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<FlowConnection>,

    /// Execution context (environment, kubeconfig, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FlowContext>,

    /// Global flow configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FlowConfig>,
}

/// Flow execution context - environment and runtime configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowContext {
    /// Kubeconfig file path (for kubectl tools)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubeconfig: Option<String>,

    /// Kubernetes namespace (default context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Kubernetes cluster name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,

    /// Environment variables to set for agent execution
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Working directory for tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Additional context variables available in templates
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Flow node - a step in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
    /// Unique node identifier
    pub id: String,

    /// Node type
    #[serde(rename = "type")]
    pub node_type: NodeType,

    /// Node configuration
    #[serde(default)]
    pub config: NodeConfig,

    /// Conditions for node execution
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<NodeCondition>,
}

/// Types of nodes in a flow
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Transform/extract data
    Transform,
    /// Execute an agent
    Agent,
    /// Conditional routing
    Conditional,
    /// Slack-specific action (send message, etc.)
    Slack,
    /// Discord-specific action
    Discord,
    /// HTTP request
    HTTP,
    /// Wait/delay
    Wait,
    /// Parallel fan-out
    Parallel,
    /// Join/merge parallel branches
    Join,
    /// Human approval gate
    Approval,
    /// End of flow
    End,
}

/// Inline agent configuration for flow nodes
/// Allows defining agent config directly in the flow without a separate Agent CRD
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineAgentConfig {
    /// Agent name (used for logging and identification)
    pub name: String,

    /// Model to use (e.g., "google:gemini-2.5-flash", "anthropic:claude-sonnet-4-20250514")
    pub model: String,

    /// System instructions for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Tools available to the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolSpec>,

    /// MCP servers for the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,

    /// Temperature for model sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens for response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

/// Node configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    // Transform node
    /// Script to execute (shell or expression)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,

    // Agent node
    /// Agent name to execute (reference to external agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Inline agent configuration (structured)
    /// Use this to embed agent config directly in the flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<InlineAgentConfig>,

    /// Inline agent configuration (YAML string) - legacy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<String>,

    /// Input to the agent (can contain ${variable} references)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,

    /// Context variables to pass to agent
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, String>,

    /// Tools available to the agent node
    /// These override or extend the agent's default tools
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolSpec>,

    /// MCP servers for the agent node
    /// These override or extend the agent's default MCP servers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,

    // Conditional node
    /// Condition expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    // Slack/messaging node
    /// Channel to send to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,

    /// Message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Thread timestamp (for replies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,

    /// Wait for reaction
    #[serde(default)]
    pub wait_for_reaction: bool,

    /// Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,

    /// Block Kit blocks (Slack)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<serde_json::Value>,

    // HTTP node
    /// URL for HTTP requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// HTTP method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// HTTP headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// HTTP body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    // Wait node
    /// Duration to wait (e.g., "30s", "5m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,

    // Parallel node
    /// Branches for parallel execution
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,

    // Join node
    /// Join strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<JoinStrategy>,

    /// Additional configuration
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Join strategy for parallel branches
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JoinStrategy {
    /// Wait for all branches
    All,
    /// Wait for any branch
    Any,
    /// Wait for majority
    Majority,
}

impl Default for JoinStrategy {
    fn default() -> Self {
        Self::All
    }
}

/// Condition for node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    /// Source node
    pub from: String,

    /// Expected value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Expected reaction (for Slack approval)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConnection {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Condition for this connection (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

/// Global flow configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowConfig {
    /// Default timeout for nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timeout_seconds: Option<u32>,

    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<FlowRetryConfig>,

    /// Error handler node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handler: Option<String>,

    /// Enable tracing/logging
    #[serde(default)]
    pub verbose: bool,
}

/// Retry configuration for flow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRetryConfig {
    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_attempts: u32,

    /// Initial delay between retries
    #[serde(default = "default_retry_delay")]
    pub initial_delay: String,

    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> String {
    "1s".to_string()
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

// ============================================================================
// AgentFlow Execution State
// ============================================================================

/// AgentFlow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFlowState {
    /// Flow run ID
    pub run_id: String,

    /// Flow name
    pub flow_name: String,

    /// Current node(s) being executed
    pub current_nodes: Vec<String>,

    /// Execution status
    pub status: FlowExecutionStatus,

    /// Node execution results
    pub node_results: HashMap<String, NodeResult>,

    /// Flow variables (accumulated from trigger and nodes)
    pub variables: HashMap<String, serde_json::Value>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Error information (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FlowError>,
}

/// Flow execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FlowExecutionStatus {
    /// Flow is pending
    Pending,
    /// Flow is running
    Running,
    /// Waiting for external event (approval, reaction)
    Waiting,
    /// Flow completed
    Completed,
    /// Flow failed
    Failed,
    /// Flow was cancelled
    Cancelled,
}

/// Result of node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    /// Node ID
    pub node_id: String,

    /// Execution status
    pub status: NodeExecutionStatus,

    /// Output data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,

    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// End time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Node execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeExecutionStatus {
    /// Node is pending
    Pending,
    /// Node is running
    Running,
    /// Waiting for external event
    Waiting,
    /// Node completed
    Completed,
    /// Node failed
    Failed,
    /// Node was skipped
    Skipped,
}

/// Flow error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowError {
    /// Error type
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Node where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Stack trace or additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// ============================================================================
// Validation
// ============================================================================

impl AgentFlow {
    /// Validate the AgentFlow configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check name
        if self.metadata.name.is_empty() {
            return Err("Flow name is required".to_string());
        }

        // Check nodes
        if self.spec.nodes.is_empty() {
            return Err("At least one node is required".to_string());
        }

        // Collect node IDs
        let node_ids: std::collections::HashSet<&str> =
            self.spec.nodes.iter().map(|n| n.id.as_str()).collect();

        // Check for duplicate node IDs
        if node_ids.len() != self.spec.nodes.len() {
            return Err("Duplicate node IDs found".to_string());
        }

        // Validate connections reference existing nodes
        // "start" is a special source that represents the flow entry point
        for conn in &self.spec.connections {
            if conn.from != "start" && !node_ids.contains(conn.from.as_str()) {
                return Err(format!("Connection references unknown node: {}", conn.from));
            }
            if !node_ids.contains(conn.to.as_str()) {
                return Err(format!("Connection references unknown node: {}", conn.to));
            }
        }

        // Validate node configurations
        for node in &self.spec.nodes {
            match node.node_type {
                NodeType::Agent => {
                    // Agent node requires either 'agent' (reference) OR 'inline' (embedded config)
                    if node.config.agent.is_none() && node.config.inline.is_none() {
                        return Err(format!(
                            "Agent node '{}' requires either 'agent' (reference) or 'inline' (embedded config)",
                            node.id
                        ));
                    }
                }
                NodeType::Conditional => {
                    if node.config.condition.is_none() {
                        return Err(format!(
                            "Conditional node '{}' requires 'condition' config",
                            node.id
                        ));
                    }
                }
                NodeType::Slack | NodeType::Discord => {
                    if node.config.channel.is_none() && node.config.message.is_none() {
                        // Might be reading input from previous node, so this is ok
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get entry nodes (nodes that have a connection from "start")
    pub fn entry_nodes(&self) -> Vec<&FlowNode> {
        let entry_ids: std::collections::HashSet<&str> = self
            .spec
            .connections
            .iter()
            .filter(|c| c.from == "start")
            .map(|c| c.to.as_str())
            .collect();

        self.spec
            .nodes
            .iter()
            .filter(|n| entry_ids.contains(n.id.as_str()))
            .collect()
    }

    /// Get successor nodes for a given node
    pub fn successors(&self, node_id: &str) -> Vec<(&FlowNode, Option<&str>)> {
        let node_map: HashMap<&str, &FlowNode> =
            self.spec.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        self.spec
            .connections
            .iter()
            .filter(|c| c.from == node_id)
            .filter_map(|c| {
                node_map
                    .get(c.to.as_str())
                    .map(|n| (*n, c.when.as_deref()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agentflow() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
spec:
  description: "Deployment workflow with approval"
  nodes:
    - id: validate
      type: Agent
      config:
        agent: validator
        input: ${input}
    - id: deploy
      type: Agent
      config:
        agent: deployer
  connections:
    - from: start
      to: validate
    - from: validate
      to: deploy
"#;

        let flow: AgentFlow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(flow.metadata.name, "deploy-flow");
        assert_eq!(flow.spec.description, Some("Deployment workflow with approval".to_string()));
        assert_eq!(flow.spec.nodes.len(), 2);
        assert_eq!(flow.spec.connections.len(), 2);

        // Validate
        assert!(flow.validate().is_ok());
    }

    #[test]
    fn test_entry_nodes() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: test-flow
spec:
  nodes:
    - id: entry1
      type: Transform
    - id: entry2
      type: Agent
      config:
        agent: test
    - id: other
      type: End
  connections:
    - from: start
      to: entry1
    - from: start
      to: entry2
    - from: entry1
      to: other
    - from: entry2
      to: other
"#;

        let flow: AgentFlow = serde_yaml::from_str(yaml).unwrap();
        let entries = flow.entry_nodes();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_validation_errors() {
        // Missing nodes
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: bad-flow
spec:
  nodes: []
  connections: []
"#;

        let flow: AgentFlow = serde_yaml::from_str(yaml).unwrap();
        assert!(flow.validate().is_err());

        // Agent node without agent config
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: bad-flow
spec:
  nodes:
    - id: agent
      type: Agent
  connections:
    - from: start
      to: agent
"#;

        let flow2: AgentFlow = serde_yaml::from_str(yaml2).unwrap();
        assert!(flow2.validate().is_err());
    }

    #[test]
    fn test_conditional_flow() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: conditional-flow
spec:
  nodes:
    - id: check
      type: Conditional
      config:
        condition: ${requires_approval} == true
    - id: approve
      type: Approval
      config:
        message: "Approval needed"
    - id: execute
      type: Agent
      config:
        agent: executor
  connections:
    - from: start
      to: check
    - from: check
      to: approve
      when: requires_approval == true
    - from: check
      to: execute
      when: requires_approval == false
    - from: approve
      to: execute
"#;

        let flow: AgentFlow = serde_yaml::from_str(yaml).unwrap();
        assert!(flow.validate().is_ok());

        let successors = flow.successors("check");
        assert_eq!(successors.len(), 2);
    }
}
