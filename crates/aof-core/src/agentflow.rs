// AOF Core - AgentFlow configuration types
//
// AgentFlow is an event-driven workflow orchestration resource that connects
// triggers (Slack, Discord, HTTP, Schedule, etc.) to agent execution with
// support for conditional routing, approval flows, and interactive responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AgentFlow - Event-driven agent workflow
///
/// Example:
/// ```yaml
/// apiVersion: aof.dev/v1
/// kind: AgentFlow
/// metadata:
///   name: slack-k8s-bot-flow
/// spec:
///   trigger:
///     type: Slack
///     config:
///       events: [app_mention, message]
///   nodes:
///     - id: process
///       type: Agent
///       config:
///         agent: my-agent
///   connections:
///     - from: trigger
///       to: process
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
    /// Trigger configuration - what starts the flow
    pub trigger: FlowTrigger,

    /// Workflow nodes
    pub nodes: Vec<FlowNode>,

    /// Node connections (edges in the graph)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<FlowConnection>,

    /// Additional triggers (for multi-trigger flows)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<FlowTrigger>,

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

/// Flow trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowTrigger {
    /// Trigger type (Slack, Discord, HTTP, Schedule, etc.)
    #[serde(rename = "type")]
    pub trigger_type: TriggerType,

    /// Trigger-specific configuration
    #[serde(default)]
    pub config: TriggerConfig,
}

/// Types of triggers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerType {
    /// Slack events (mentions, messages, slash commands)
    Slack,
    /// Discord events
    Discord,
    /// Telegram events
    Telegram,
    /// WhatsApp events
    WhatsApp,
    /// Generic HTTP webhook
    HTTP,
    /// Cron/schedule-based trigger
    Schedule,
    /// Manual trigger
    Manual,
}

/// Trigger-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerConfig {
    /// Events to listen for (Slack: app_mention, message, slash_command)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    /// Channels to listen on (Slack/Discord channel names or IDs)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<String>,

    /// Users to respond to (user IDs or patterns)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,

    /// Message patterns to match (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,

    /// Bot token (or env var reference ${VAR_NAME})
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,

    /// Signing secret (or env var reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secret: Option<String>,

    /// Cron expression (for Schedule trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,

    /// Timezone (for Schedule trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// HTTP method (for HTTP trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// HTTP path pattern (for HTTP trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Additional configuration
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

/// Node configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    // Transform node
    /// Script to execute (shell or expression)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,

    // Agent node
    /// Agent name to execute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Inline agent configuration (YAML string)
    /// Use this to embed agent config directly in the flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<String>,

    /// Input to the agent (can contain ${variable} references)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,

    /// Context variables to pass to agent
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, String>,

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
        for conn in &self.spec.connections {
            if conn.from != "trigger" && !node_ids.contains(conn.from.as_str()) {
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
                    if node.config.agent.is_none() {
                        return Err(format!("Agent node '{}' requires 'agent' config", node.id));
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

    /// Get entry nodes (nodes that have a connection from "trigger")
    pub fn entry_nodes(&self) -> Vec<&FlowNode> {
        let entry_ids: std::collections::HashSet<&str> = self
            .spec
            .connections
            .iter()
            .filter(|c| c.from == "trigger")
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
  name: slack-bot-flow
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
        - message
      bot_token: ${SLACK_BOT_TOKEN}
  nodes:
    - id: parse-message
      type: Transform
      config:
        script: |
          export MESSAGE_TEXT="${event.text}"
    - id: agent-process
      type: Agent
      config:
        agent: slack-k8s-bot
        input: ${MESSAGE_TEXT}
    - id: send-response
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        message: ${agent-process.output}
  connections:
    - from: trigger
      to: parse-message
    - from: parse-message
      to: agent-process
    - from: agent-process
      to: send-response
"#;

        let flow: AgentFlow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(flow.metadata.name, "slack-bot-flow");
        assert_eq!(flow.spec.trigger.trigger_type, TriggerType::Slack);
        assert_eq!(flow.spec.nodes.len(), 3);
        assert_eq!(flow.spec.connections.len(), 3);

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
  trigger:
    type: HTTP
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
    - from: trigger
      to: entry1
    - from: trigger
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
  trigger:
    type: HTTP
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
  trigger:
    type: HTTP
  nodes:
    - id: agent
      type: Agent
  connections:
    - from: trigger
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
  trigger:
    type: Slack
  nodes:
    - id: check
      type: Conditional
      config:
        condition: ${requires_approval} == true
    - id: approve
      type: Slack
      config:
        channel: ${channel}
        message: "Approval needed"
        wait_for_reaction: true
    - id: execute
      type: Agent
      config:
        agent: executor
  connections:
    - from: trigger
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
