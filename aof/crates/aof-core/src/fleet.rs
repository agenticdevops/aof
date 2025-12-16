//! AgentFleet - Multi-agent coordination and orchestration
//!
//! AgentFleet provides Kubernetes-style configuration for managing groups
//! of agents with different coordination modes:
//! - Hierarchical: Manager agent coordinates workers
//! - Peer: All agents coordinate as equals
//! - Swarm: Dynamic self-organizing coordination

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AgentFleet configuration (K8s-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFleet {
    /// API version (e.g., "aof.dev/v1")
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    /// Resource kind (always "AgentFleet")
    pub kind: String,

    /// Fleet metadata
    pub metadata: FleetMetadata,

    /// Fleet specification
    pub spec: FleetSpec,
}

/// Fleet metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMetadata {
    /// Fleet name
    pub name: String,

    /// Namespace
    #[serde(default)]
    pub namespace: Option<String>,

    /// Labels for filtering
    #[serde(default)]
    pub labels: HashMap<String, String>,

    /// Annotations for metadata
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

/// Fleet specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSpec {
    /// Agent definitions in the fleet
    pub agents: Vec<FleetAgent>,

    /// Coordination configuration
    #[serde(default)]
    pub coordination: CoordinationConfig,

    /// Shared resources across agents
    #[serde(default)]
    pub shared: Option<SharedResources>,

    /// Communication patterns
    #[serde(default)]
    pub communication: Option<CommunicationConfig>,

    /// Scaling configuration
    #[serde(default)]
    pub scaling: Option<ScalingConfig>,
}

/// Agent definition within a fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetAgent {
    /// Agent name (unique within fleet)
    pub name: String,

    /// Path to agent configuration file
    #[serde(default)]
    pub config: Option<String>,

    /// Inline agent configuration
    #[serde(default)]
    pub spec: Option<FleetAgentSpec>,

    /// Number of agent replicas
    #[serde(default = "default_replicas")]
    pub replicas: u32,

    /// Role in the fleet
    #[serde(default)]
    pub role: AgentRole,

    /// Agent-specific labels
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

fn default_replicas() -> u32 {
    1
}

/// Inline agent specification (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetAgentSpec {
    /// Model to use
    pub model: String,

    /// Agent instructions/system prompt
    #[serde(default)]
    pub instructions: Option<String>,

    /// Tools available to this agent
    #[serde(default)]
    pub tools: Vec<String>,

    /// MCP servers for this agent
    #[serde(default)]
    pub mcp_servers: Vec<crate::McpServerConfig>,

    /// Maximum iterations
    #[serde(default)]
    pub max_iterations: Option<u32>,

    /// Temperature for model
    #[serde(default)]
    pub temperature: Option<f32>,
}

/// Agent role within the fleet
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    /// Regular worker agent
    #[default]
    Worker,
    /// Manager/coordinator agent
    Manager,
    /// Specialist agent for specific tasks
    Specialist,
    /// Validator/reviewer agent
    Validator,
}

/// Coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    /// Coordination mode
    #[serde(default)]
    pub mode: CoordinationMode,

    /// Manager agent name (for hierarchical mode)
    #[serde(default)]
    pub manager: Option<String>,

    /// Task distribution strategy
    #[serde(default)]
    pub distribution: TaskDistribution,

    /// Consensus configuration (for peer mode)
    #[serde(default)]
    pub consensus: Option<ConsensusConfig>,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            mode: CoordinationMode::Peer,
            manager: None,
            distribution: TaskDistribution::RoundRobin,
            consensus: None,
        }
    }
}

/// Coordination mode for the fleet
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CoordinationMode {
    /// Hierarchical: Manager coordinates workers
    Hierarchical,
    /// Peer-to-peer: Agents coordinate as equals
    #[default]
    Peer,
    /// Swarm: Self-organizing dynamic coordination
    Swarm,
    /// Pipeline: Sequential handoff between agents
    Pipeline,
}

/// Task distribution strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskDistribution {
    /// Round-robin distribution
    #[default]
    RoundRobin,
    /// Least-loaded agent gets the task
    LeastLoaded,
    /// Random distribution
    Random,
    /// Skill-based routing (agents with matching skills)
    SkillBased,
    /// Sticky routing (same task types go to same agent)
    Sticky,
}

/// Consensus configuration for peer coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm
    #[serde(default)]
    pub algorithm: ConsensusAlgorithm,

    /// Minimum votes required for consensus
    #[serde(default)]
    pub min_votes: Option<u32>,

    /// Timeout for reaching consensus
    #[serde(default)]
    pub timeout_ms: Option<u64>,

    /// Allow partial consensus
    #[serde(default)]
    pub allow_partial: bool,
}

/// Consensus algorithm type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConsensusAlgorithm {
    /// Simple majority voting
    #[default]
    Majority,
    /// Unanimous agreement required
    Unanimous,
    /// Weighted voting based on agent roles
    Weighted,
    /// First response wins
    FirstWins,
}

/// Shared resources configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SharedResources {
    /// Shared memory configuration
    #[serde(default)]
    pub memory: Option<SharedMemoryConfig>,

    /// Shared tools available to all agents
    #[serde(default)]
    pub tools: Vec<SharedToolConfig>,

    /// Shared knowledge base
    #[serde(default)]
    pub knowledge: Option<SharedKnowledgeConfig>,
}

/// Shared memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryConfig {
    /// Memory backend type
    #[serde(rename = "type")]
    pub memory_type: SharedMemoryType,

    /// Connection URL
    #[serde(default)]
    pub url: Option<String>,

    /// Memory namespace/prefix
    #[serde(default)]
    pub namespace: Option<String>,

    /// TTL for memory entries (seconds)
    #[serde(default)]
    pub ttl: Option<u64>,
}

/// Shared memory backend type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SharedMemoryType {
    /// In-memory (single process)
    #[default]
    InMemory,
    /// Redis backend
    Redis,
    /// SQLite backend
    Sqlite,
    /// PostgreSQL backend
    Postgres,
}

/// Shared tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedToolConfig {
    /// MCP server reference
    #[serde(rename = "mcp-server")]
    pub mcp_server: Option<String>,

    /// Built-in tool name
    pub tool: Option<String>,

    /// Tool configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Shared knowledge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedKnowledgeConfig {
    /// Knowledge base type
    #[serde(rename = "type")]
    pub kb_type: String,

    /// Source path or URL
    pub source: String,

    /// Additional configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Communication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    /// Message passing pattern
    #[serde(default)]
    pub pattern: MessagePattern,

    /// Message queue configuration
    #[serde(default)]
    pub queue: Option<QueueConfig>,

    /// Broadcast configuration
    #[serde(default)]
    pub broadcast: Option<BroadcastConfig>,
}

/// Message passing pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MessagePattern {
    /// Direct point-to-point messaging
    #[default]
    Direct,
    /// Publish-subscribe pattern
    PubSub,
    /// Request-reply pattern
    RequestReply,
    /// Broadcast to all agents
    Broadcast,
}

/// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Queue type
    #[serde(rename = "type")]
    pub queue_type: String,

    /// Connection URL
    pub url: String,

    /// Queue name
    pub name: String,
}

/// Broadcast configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastConfig {
    /// Broadcast channel name
    pub channel: String,

    /// Include sender in broadcast
    #[serde(default)]
    pub include_sender: bool,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Minimum replicas
    #[serde(default)]
    pub min_replicas: Option<u32>,

    /// Maximum replicas
    #[serde(default)]
    pub max_replicas: Option<u32>,

    /// Auto-scaling enabled
    #[serde(default)]
    pub auto_scale: bool,

    /// Scaling metrics
    #[serde(default)]
    pub metrics: Vec<ScalingMetric>,
}

/// Scaling metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetric {
    /// Metric name
    pub name: String,

    /// Target value
    pub target: f64,

    /// Metric type
    #[serde(rename = "type")]
    pub metric_type: String,
}

/// Fleet runtime state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetState {
    /// Fleet name
    pub fleet_name: String,

    /// Fleet status
    pub status: FleetStatus,

    /// Agent instances
    pub agents: HashMap<String, AgentInstanceState>,

    /// Active tasks
    pub active_tasks: Vec<FleetTask>,

    /// Completed tasks
    pub completed_tasks: Vec<FleetTask>,

    /// Fleet metrics
    pub metrics: FleetMetrics,

    /// Start time
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl FleetState {
    /// Create new fleet state
    pub fn new(fleet_name: &str) -> Self {
        Self {
            fleet_name: fleet_name.to_string(),
            status: FleetStatus::Initializing,
            agents: HashMap::new(),
            active_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            metrics: FleetMetrics::default(),
            started_at: None,
        }
    }
}

/// Fleet status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FleetStatus {
    /// Fleet is initializing
    #[default]
    Initializing,
    /// Fleet is ready and idle
    Ready,
    /// Fleet is actively processing tasks
    Active,
    /// Fleet is paused
    Paused,
    /// Fleet has failed
    Failed,
    /// Fleet is shutting down
    ShuttingDown,
}

/// Agent instance state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstanceState {
    /// Instance ID
    pub instance_id: String,

    /// Agent name from fleet config
    pub agent_name: String,

    /// Replica index
    pub replica_index: u32,

    /// Instance status
    pub status: AgentInstanceStatus,

    /// Current task (if any)
    pub current_task: Option<String>,

    /// Tasks processed count
    pub tasks_processed: u64,

    /// Last activity timestamp
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Agent instance status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentInstanceStatus {
    /// Instance is starting
    #[default]
    Starting,
    /// Instance is idle and ready
    Idle,
    /// Instance is processing a task
    Busy,
    /// Instance has failed
    Failed,
    /// Instance is stopped
    Stopped,
}

/// Fleet task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetTask {
    /// Task ID
    pub task_id: String,

    /// Task input
    pub input: serde_json::Value,

    /// Assigned agent instance
    pub assigned_to: Option<String>,

    /// Task status
    pub status: FleetTaskStatus,

    /// Task result (if completed)
    pub result: Option<serde_json::Value>,

    /// Error (if failed)
    pub error: Option<String>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Started timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Completed timestamp
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Fleet task status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FleetTaskStatus {
    /// Task is pending assignment
    #[default]
    Pending,
    /// Task is assigned to an agent
    Assigned,
    /// Task is being processed
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Fleet metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetMetrics {
    /// Total tasks submitted
    pub total_tasks: u64,

    /// Completed tasks
    pub completed_tasks: u64,

    /// Failed tasks
    pub failed_tasks: u64,

    /// Average task duration (ms)
    pub avg_task_duration_ms: f64,

    /// Active agent count
    pub active_agents: u32,

    /// Total agent count
    pub total_agents: u32,

    /// Messages exchanged between agents
    pub messages_exchanged: u64,

    /// Consensus rounds (for peer mode)
    pub consensus_rounds: u64,
}

impl AgentFleet {
    /// Load fleet from YAML file
    pub fn from_yaml(yaml: &str) -> Result<Self, crate::AofError> {
        serde_yaml::from_str(yaml).map_err(|e| crate::AofError::config(format!("Failed to parse fleet YAML: {}", e)))
    }

    /// Load fleet from file
    pub fn from_file(path: &str) -> Result<Self, crate::AofError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::AofError::config(format!("Failed to read fleet file: {}", e)))?;
        Self::from_yaml(&content)
    }

    /// Get agent by name
    pub fn get_agent(&self, name: &str) -> Option<&FleetAgent> {
        self.spec.agents.iter().find(|a| a.name == name)
    }

    /// Get all agents with a specific role
    pub fn get_agents_by_role(&self, role: AgentRole) -> Vec<&FleetAgent> {
        self.spec.agents.iter().filter(|a| a.role == role).collect()
    }

    /// Get the manager agent (for hierarchical mode)
    pub fn get_manager(&self) -> Option<&FleetAgent> {
        if let Some(ref manager_name) = self.spec.coordination.manager {
            self.get_agent(manager_name)
        } else {
            self.get_agents_by_role(AgentRole::Manager).first().copied()
        }
    }

    /// Total replica count across all agents
    pub fn total_replicas(&self) -> u32 {
        self.spec.agents.iter().map(|a| a.replicas).sum()
    }

    /// Validate fleet configuration
    pub fn validate(&self) -> Result<(), crate::AofError> {
        // Check for duplicate agent names
        let mut names = std::collections::HashSet::new();
        for agent in &self.spec.agents {
            if !names.insert(&agent.name) {
                return Err(crate::AofError::config(format!(
                    "Duplicate agent name in fleet: {}",
                    agent.name
                )));
            }
        }

        // Validate hierarchical mode has a manager
        if self.spec.coordination.mode == CoordinationMode::Hierarchical {
            if self.get_manager().is_none() {
                return Err(crate::AofError::config(
                    "Hierarchical mode requires a manager agent".to_string(),
                ));
            }
        }

        // Validate agent configurations
        for agent in &self.spec.agents {
            if agent.config.is_none() && agent.spec.is_none() {
                return Err(crate::AofError::config(format!(
                    "Agent '{}' must have either 'config' or 'spec' defined",
                    agent.name
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fleet_yaml() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: incident-team
  labels:
    team: sre
spec:
  agents:
    - name: detector
      config: ./agents/detector.yaml
      replicas: 2
      role: worker
    - name: analyzer
      config: ./agents/analyzer.yaml
      replicas: 1
      role: specialist
    - name: coordinator
      config: ./agents/coordinator.yaml
      replicas: 1
      role: manager
  coordination:
    mode: hierarchical
    manager: coordinator
    distribution: skill-based
  shared:
    memory:
      type: redis
      url: redis://localhost:6379
    tools:
      - mcp-server: kubectl-ai
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert_eq!(fleet.metadata.name, "incident-team");
        assert_eq!(fleet.spec.agents.len(), 3);
        assert_eq!(fleet.spec.coordination.mode, CoordinationMode::Hierarchical);
        assert_eq!(fleet.total_replicas(), 4);

        let manager = fleet.get_manager().unwrap();
        assert_eq!(manager.name, "coordinator");
    }

    #[test]
    fn test_peer_mode_fleet() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: review-team
spec:
  agents:
    - name: reviewer-1
      config: ./reviewer.yaml
    - name: reviewer-2
      config: ./reviewer.yaml
    - name: reviewer-3
      config: ./reviewer.yaml
  coordination:
    mode: peer
    distribution: round-robin
    consensus:
      algorithm: majority
      minVotes: 2
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert_eq!(fleet.spec.coordination.mode, CoordinationMode::Peer);
        assert!(fleet.spec.coordination.consensus.is_some());
    }

    #[test]
    fn test_fleet_validation() {
        // Valid fleet
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test-fleet
spec:
  agents:
    - name: agent-1
      config: ./agent.yaml
"#;
        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert!(fleet.validate().is_ok());

        // Invalid: hierarchical without manager
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test-fleet
spec:
  agents:
    - name: agent-1
      config: ./agent.yaml
  coordination:
    mode: hierarchical
"#;
        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert!(fleet.validate().is_err());
    }

    #[test]
    fn test_fleet_state() {
        let mut state = FleetState::new("test-fleet");
        assert_eq!(state.status, FleetStatus::Initializing);

        state.status = FleetStatus::Ready;
        state.metrics.total_agents = 3;
        state.metrics.active_agents = 3;

        assert_eq!(state.metrics.total_agents, 3);
    }
}
