//! AgentFleet - Multi-agent coordination and orchestration
//!
//! AgentFleet provides Kubernetes-style configuration for managing groups
//! of agents with different coordination modes:
//! - Hierarchical: Manager agent coordinates workers
//! - Peer: All agents coordinate as equals
//! - Swarm: Dynamic self-organizing coordination
//! - Pipeline: Sequential handoff between agents
//! - Tiered: Tier-based parallel execution with consensus (for multi-model RCA)

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

    /// Tier number for tiered coordination mode (1 = first tier, 2 = second, etc.)
    /// Tier 1 agents run first, their results feed into tier 2, etc.
    /// If not specified, defaults to 1.
    #[serde(default)]
    pub tier: Option<u32>,

    /// Weight for weighted consensus voting (default: 1.0)
    #[serde(default)]
    pub weight: Option<f32>,
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
    /// Supports both simple strings and qualified specs
    #[serde(default)]
    pub tools: Vec<crate::agent::ToolSpec>,

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

    /// Consensus configuration (for peer/tiered mode)
    #[serde(default)]
    pub consensus: Option<ConsensusConfig>,

    /// Tiered execution configuration (for tiered mode)
    #[serde(default)]
    pub tiered: Option<TieredConfig>,
}

/// Configuration for tiered coordination mode
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TieredConfig {
    /// Per-tier consensus configuration
    /// Key: tier number (as string), Value: consensus config for that tier
    #[serde(default)]
    pub tier_consensus: HashMap<String, ConsensusConfig>,

    /// Whether to pass all tier results to next tier or just the consensus result
    #[serde(default)]
    pub pass_all_results: bool,

    /// Final tier aggregation strategy
    #[serde(default)]
    pub final_aggregation: FinalAggregation,
}

/// How to aggregate results from the final tier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalAggregation {
    /// Use consensus algorithm to pick best result
    #[default]
    Consensus,
    /// Merge all results into combined output
    Merge,
    /// Use manager agent to synthesize final result
    ManagerSynthesis,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            mode: CoordinationMode::Peer,
            manager: None,
            distribution: TaskDistribution::RoundRobin,
            consensus: None,
            tiered: None,
        }
    }
}

/// Coordination mode for the fleet
///
/// Choose the mode that best fits your use case:
/// - **Peer**: All agents work on the same task, results combined via consensus. Best for
///   diverse perspectives (code review, RCA analysis).
/// - **Hierarchical**: Manager delegates to workers, aggregates results. Best for
///   complex multi-step tasks with oversight.
/// - **Pipeline**: Sequential handoff between agents. Best for workflows where
///   each stage transforms data for the next.
/// - **Swarm**: Self-organizing dynamic coordination. Best for large-scale
///   parallel processing with adaptive load balancing.
/// - **Tiered**: Tier-based parallel execution with consensus. Best for multi-model
///   RCA where cheap data-collectors feed reasoning models.
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
    /// Tiered: Tier-based parallel execution with consensus
    /// Agents are grouped by tier (e.g., tier 1 = data collectors, tier 2 = reasoners)
    /// Each tier runs in parallel, results flow to next tier
    Tiered,
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

/// Consensus configuration for peer/tiered coordination
///
/// # Example
/// ```yaml
/// consensus:
///   algorithm: weighted
///   min_votes: 2
///   timeout_ms: 60000
///   allow_partial: true
///   weights:
///     senior-reviewer: 2.0
///     junior-reviewer: 1.0
///   min_confidence: 0.7
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm
    #[serde(default)]
    pub algorithm: ConsensusAlgorithm,

    /// Minimum votes required for consensus
    #[serde(default)]
    pub min_votes: Option<u32>,

    /// Timeout for reaching consensus (milliseconds)
    #[serde(default)]
    pub timeout_ms: Option<u64>,

    /// Allow partial consensus if timeout reached
    #[serde(default)]
    pub allow_partial: bool,

    /// Per-agent weights for weighted voting
    /// Key: agent name, Value: weight (default 1.0)
    #[serde(default)]
    pub weights: HashMap<String, f32>,

    /// Minimum confidence threshold (0.0-1.0)
    /// Below this threshold, result is flagged for human review
    #[serde(default)]
    pub min_confidence: Option<f32>,
}

/// Consensus algorithm type
///
/// Choose based on your requirements:
/// - **Majority**: >50% agreement wins. Fast, tolerates outliers.
/// - **Unanimous**: 100% agreement required. High confidence, may timeout.
/// - **Weighted**: Per-agent weights (senior reviewers count more). Balanced expertise.
/// - **FirstWins**: First response wins. Fastest, no consensus overhead.
/// - **HumanReview**: Flags for human operator decision. High-stakes scenarios.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusAlgorithm {
    /// Simple majority voting (>50% agreement)
    #[default]
    Majority,
    /// Unanimous agreement required (100%)
    Unanimous,
    /// Weighted voting based on agent weights
    Weighted,
    /// First response wins (no consensus)
    FirstWins,
    /// Flags result for human operator review
    HumanReview,
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

        // Validate tiered mode has agents with tier assignments
        if self.spec.coordination.mode == CoordinationMode::Tiered {
            let tiers = self.get_tiers();
            if tiers.is_empty() {
                return Err(crate::AofError::config(
                    "Tiered mode requires at least one agent with a tier assignment".to_string(),
                ));
            }
            // Ensure we have at least 2 tiers for meaningful tiered execution
            if tiers.len() < 2 {
                // This is a warning, not an error - single tier still works
                tracing::warn!(
                    "Tiered mode with only one tier ({}) - consider using peer mode instead",
                    tiers[0]
                );
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

    /// Get unique tiers in the fleet (sorted ascending)
    pub fn get_tiers(&self) -> Vec<u32> {
        let mut tiers: Vec<u32> = self
            .spec
            .agents
            .iter()
            .map(|a| a.tier.unwrap_or(1))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tiers.sort();
        tiers
    }

    /// Get agents for a specific tier
    pub fn get_agents_by_tier(&self, tier: u32) -> Vec<&FleetAgent> {
        self.spec
            .agents
            .iter()
            .filter(|a| a.tier.unwrap_or(1) == tier)
            .collect()
    }

    /// Get agent weight (for weighted consensus)
    pub fn get_agent_weight(&self, agent_name: &str) -> f32 {
        // First check agent-level weight
        if let Some(agent) = self.get_agent(agent_name) {
            if let Some(weight) = agent.weight {
                return weight;
            }
        }
        // Then check consensus config weights
        if let Some(ref consensus) = self.spec.coordination.consensus {
            if let Some(weight) = consensus.weights.get(agent_name) {
                return *weight;
            }
        }
        // Default weight
        1.0
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

    #[test]
    fn test_tiered_mode_fleet() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: rca-team
spec:
  agents:
    # Tier 1: Data collectors (cheap models)
    - name: loki-collector
      config: ./agents/loki.yaml
      tier: 1
    - name: prometheus-collector
      config: ./agents/prometheus.yaml
      tier: 1
    - name: k8s-collector
      config: ./agents/k8s.yaml
      tier: 1
    # Tier 2: Reasoning models
    - name: claude-analyzer
      config: ./agents/claude.yaml
      tier: 2
      weight: 2.0
    - name: gemini-analyzer
      config: ./agents/gemini.yaml
      tier: 2
    # Tier 3: Synthesizer
    - name: rca-coordinator
      config: ./agents/coordinator.yaml
      tier: 3
      role: manager
  coordination:
    mode: tiered
    consensus:
      algorithm: weighted
      min_votes: 2
      min_confidence: 0.7
    tiered:
      pass_all_results: true
      final_aggregation: manager_synthesis
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert_eq!(fleet.metadata.name, "rca-team");
        assert_eq!(fleet.spec.coordination.mode, CoordinationMode::Tiered);

        // Test tier detection
        let tiers = fleet.get_tiers();
        assert_eq!(tiers, vec![1, 2, 3]);

        // Test agents by tier
        let tier1_agents = fleet.get_agents_by_tier(1);
        assert_eq!(tier1_agents.len(), 3);

        let tier2_agents = fleet.get_agents_by_tier(2);
        assert_eq!(tier2_agents.len(), 2);

        let tier3_agents = fleet.get_agents_by_tier(3);
        assert_eq!(tier3_agents.len(), 1);

        // Test agent weights
        assert_eq!(fleet.get_agent_weight("claude-analyzer"), 2.0);
        assert_eq!(fleet.get_agent_weight("gemini-analyzer"), 1.0); // default

        // Validate configuration
        assert!(fleet.validate().is_ok());
    }

    #[test]
    fn test_consensus_algorithms() {
        // Test all consensus algorithm variants parse correctly
        let algorithms = vec![
            ("majority", ConsensusAlgorithm::Majority),
            ("unanimous", ConsensusAlgorithm::Unanimous),
            ("weighted", ConsensusAlgorithm::Weighted),
            ("first_wins", ConsensusAlgorithm::FirstWins),
            ("human_review", ConsensusAlgorithm::HumanReview),
        ];

        for (yaml_value, expected) in algorithms {
            let yaml = format!(r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test
spec:
  agents:
    - name: agent-1
      config: ./agent.yaml
  coordination:
    mode: peer
    consensus:
      algorithm: {}
"#, yaml_value);

            let fleet = AgentFleet::from_yaml(&yaml).unwrap();
            assert_eq!(
                fleet.spec.coordination.consensus.as_ref().unwrap().algorithm,
                expected,
                "Failed for algorithm: {}",
                yaml_value
            );
        }
    }

    #[test]
    fn test_weighted_consensus_config() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: weighted-team
spec:
  agents:
    - name: senior-reviewer
      config: ./reviewer.yaml
      weight: 2.0
    - name: junior-reviewer
      config: ./reviewer.yaml
  coordination:
    mode: peer
    consensus:
      algorithm: weighted
      min_votes: 2
      min_confidence: 0.8
      weights:
        senior-reviewer: 2.0
        junior-reviewer: 1.0
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        let consensus = fleet.spec.coordination.consensus.as_ref().unwrap();

        assert_eq!(consensus.algorithm, ConsensusAlgorithm::Weighted);
        assert_eq!(consensus.min_votes, Some(2));
        assert_eq!(consensus.min_confidence, Some(0.8));
        assert_eq!(consensus.weights.get("senior-reviewer"), Some(&2.0));
        assert_eq!(consensus.weights.get("junior-reviewer"), Some(&1.0));

        // Test weight lookup via fleet method
        assert_eq!(fleet.get_agent_weight("senior-reviewer"), 2.0);
    }

    #[test]
    fn test_human_review_consensus() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: critical-review
spec:
  agents:
    - name: analyzer-1
      config: ./analyzer.yaml
    - name: analyzer-2
      config: ./analyzer.yaml
  coordination:
    mode: peer
    consensus:
      algorithm: human_review
      timeout_ms: 300000
      min_confidence: 0.9
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        let consensus = fleet.spec.coordination.consensus.as_ref().unwrap();

        assert_eq!(consensus.algorithm, ConsensusAlgorithm::HumanReview);
        assert_eq!(consensus.timeout_ms, Some(300000));
        assert_eq!(consensus.min_confidence, Some(0.9));
    }

    #[test]
    fn test_all_coordination_modes() {
        let modes = vec![
            ("peer", CoordinationMode::Peer),
            ("hierarchical", CoordinationMode::Hierarchical),
            ("pipeline", CoordinationMode::Pipeline),
            ("swarm", CoordinationMode::Swarm),
            ("tiered", CoordinationMode::Tiered),
        ];

        for (yaml_value, expected) in modes {
            let yaml = format!(r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test
spec:
  agents:
    - name: agent-1
      config: ./agent.yaml
      role: manager
      tier: 1
    - name: agent-2
      config: ./agent.yaml
      tier: 2
  coordination:
    mode: {}
    manager: agent-1
"#, yaml_value);

            let fleet = AgentFleet::from_yaml(&yaml).unwrap();
            assert_eq!(
                fleet.spec.coordination.mode,
                expected,
                "Failed for mode: {}",
                yaml_value
            );
        }
    }

    #[test]
    fn test_tiered_config() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: tiered-team
spec:
  agents:
    - name: collector
      config: ./collector.yaml
      tier: 1
    - name: reasoner
      config: ./reasoner.yaml
      tier: 2
  coordination:
    mode: tiered
    tiered:
      pass_all_results: true
      final_aggregation: merge
      tier_consensus:
        "1":
          algorithm: first_wins
        "2":
          algorithm: majority
          min_votes: 1
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        let tiered = fleet.spec.coordination.tiered.as_ref().unwrap();

        assert!(tiered.pass_all_results);
        assert_eq!(tiered.final_aggregation, FinalAggregation::Merge);

        let tier1_consensus = tiered.tier_consensus.get("1").unwrap();
        assert_eq!(tier1_consensus.algorithm, ConsensusAlgorithm::FirstWins);

        let tier2_consensus = tiered.tier_consensus.get("2").unwrap();
        assert_eq!(tier2_consensus.algorithm, ConsensusAlgorithm::Majority);
    }

    #[test]
    fn test_final_aggregation_modes() {
        let modes = vec![
            ("consensus", FinalAggregation::Consensus),
            ("merge", FinalAggregation::Merge),
            ("manager_synthesis", FinalAggregation::ManagerSynthesis),
        ];

        for (yaml_value, expected) in modes {
            let yaml = format!(r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test
spec:
  agents:
    - name: agent-1
      config: ./agent.yaml
      tier: 1
    - name: agent-2
      config: ./agent.yaml
      tier: 2
  coordination:
    mode: tiered
    tiered:
      final_aggregation: {}
"#, yaml_value);

            let fleet = AgentFleet::from_yaml(&yaml).unwrap();
            let tiered = fleet.spec.coordination.tiered.as_ref().unwrap();
            assert_eq!(
                tiered.final_aggregation,
                expected,
                "Failed for aggregation: {}",
                yaml_value
            );
        }
    }

    #[test]
    fn test_existing_simple_fleet_unchanged() {
        // Ensure existing simple fleet configurations still work
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: simple-fleet
spec:
  agents:
    - name: worker
      config: ./worker.yaml
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert_eq!(fleet.spec.coordination.mode, CoordinationMode::Peer); // default
        assert!(fleet.validate().is_ok());
    }

    #[test]
    fn test_pipeline_mode_unchanged() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: pipeline-fleet
spec:
  agents:
    - name: stage1
      config: ./stage1.yaml
    - name: stage2
      config: ./stage2.yaml
    - name: stage3
      config: ./stage3.yaml
  coordination:
    mode: pipeline
"#;

        let fleet = AgentFleet::from_yaml(yaml).unwrap();
        assert_eq!(fleet.spec.coordination.mode, CoordinationMode::Pipeline);
        assert!(fleet.validate().is_ok());
    }
}
