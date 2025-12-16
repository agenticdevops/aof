//! Fleet Coordinator - Multi-agent coordination and orchestration
//!
//! The FleetCoordinator manages groups of agents working together,
//! providing different coordination modes:
//! - Hierarchical: Manager agent delegates to workers
//! - Peer: Agents coordinate as equals with consensus
//! - Swarm: Self-organizing dynamic coordination
//! - Pipeline: Sequential handoff between agents

use aof_core::{
    AgentConfig, AgentFleet, AgentInstanceState, AgentInstanceStatus, AgentRole, AofError,
    AofResult, CoordinationMode, FleetAgent, FleetMetrics, FleetState, FleetStatus, FleetTask,
    FleetTaskStatus, TaskDistribution,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::Runtime;

/// Fleet coordinator for managing multi-agent collaboration
pub struct FleetCoordinator {
    /// Fleet configuration
    fleet: AgentFleet,

    /// Fleet state
    state: Arc<RwLock<FleetState>>,

    /// Runtime for agent execution (wrapped in RwLock for interior mutability)
    runtime: Arc<RwLock<Runtime>>,

    /// Event channel for fleet events
    event_tx: Option<mpsc::Sender<FleetEvent>>,

    /// Task queue for pending tasks
    task_queue: Arc<RwLock<Vec<FleetTask>>>,

    /// Round-robin counter for task distribution
    rr_counter: Arc<RwLock<usize>>,
}

/// Events emitted by the fleet coordinator
#[derive(Debug, Clone)]
pub enum FleetEvent {
    /// Fleet has started
    Started {
        fleet_name: String,
        agent_count: usize,
    },
    /// Agent instance has started
    AgentStarted {
        agent_name: String,
        instance_id: String,
    },
    /// Agent instance has failed
    AgentFailed {
        agent_name: String,
        instance_id: String,
        error: String,
    },
    /// Task has been submitted
    TaskSubmitted { task_id: String },
    /// Task has been assigned to an agent
    TaskAssigned {
        task_id: String,
        agent_name: String,
        instance_id: String,
    },
    /// Task has completed
    TaskCompleted {
        task_id: String,
        duration_ms: u64,
    },
    /// Task has failed
    TaskFailed { task_id: String, error: String },
    /// Consensus reached (for peer mode)
    ConsensusReached {
        task_id: String,
        votes: u32,
        result: serde_json::Value,
    },
    /// Fleet has stopped
    Stopped { fleet_name: String },
    /// General error
    Error { message: String },
}

impl FleetCoordinator {
    /// Create a new fleet coordinator
    pub fn new(fleet: AgentFleet, runtime: Runtime) -> Self {
        let state = FleetState::new(&fleet.metadata.name);
        Self {
            fleet,
            state: Arc::new(RwLock::new(state)),
            runtime: Arc::new(RwLock::new(runtime)),
            event_tx: None,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            rr_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Load fleet from YAML file
    pub async fn from_file(path: &str) -> AofResult<Self> {
        let fleet = AgentFleet::from_file(path)?;
        fleet.validate()?;
        let runtime = Runtime::new();
        Ok(Self::new(fleet, runtime))
    }

    /// Set event channel
    pub fn with_event_channel(mut self, tx: mpsc::Sender<FleetEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Get fleet configuration
    pub fn fleet(&self) -> &AgentFleet {
        &self.fleet
    }

    /// Get current fleet state
    pub async fn state(&self) -> FleetState {
        self.state.read().await.clone()
    }

    /// Initialize and start the fleet
    pub async fn start(&mut self) -> AofResult<()> {
        info!("Starting fleet: {}", self.fleet.metadata.name);

        // Validate configuration
        self.fleet.validate()?;

        // Update state
        {
            let mut state = self.state.write().await;
            state.status = FleetStatus::Initializing;
            state.started_at = Some(chrono::Utc::now());
        }

        // Load agent configurations and create instances
        // Clone the agents list to avoid borrowing issues
        let agents: Vec<_> = self.fleet.spec.agents.clone();
        for fleet_agent in agents {
            self.start_agent_instances(&fleet_agent).await?;
        }

        // Update state to ready
        {
            let mut state = self.state.write().await;
            state.status = FleetStatus::Ready;
            state.metrics.total_agents = self.fleet.total_replicas();
            state.metrics.active_agents = state.agents.len() as u32;
        }

        // Emit started event
        self.emit_event(FleetEvent::Started {
            fleet_name: self.fleet.metadata.name.clone(),
            agent_count: self.fleet.total_replicas() as usize,
        })
        .await;

        info!(
            "Fleet '{}' started with {} agent instances",
            self.fleet.metadata.name,
            self.fleet.total_replicas()
        );

        Ok(())
    }

    /// Start agent instances for a fleet agent
    async fn start_agent_instances(&self, fleet_agent: &FleetAgent) -> AofResult<()> {
        // Load agent config
        let agent_config = self.load_agent_config(fleet_agent).await?;

        // Load agent into runtime (only once per agent type, replicas are logical)
        {
            let mut runtime = self.runtime.write().await;
            runtime
                .load_agent_from_config(agent_config)
                .await
                .map_err(|e| {
                    AofError::runtime(format!(
                        "Failed to load agent '{}': {}",
                        fleet_agent.name, e
                    ))
                })?;
        }
        info!("Loaded agent '{}' into runtime", fleet_agent.name);

        // Create instance state entries for each replica
        for replica_idx in 0..fleet_agent.replicas {
            let instance_id = format!("{}-{}", fleet_agent.name, replica_idx);

            // Create instance state
            let instance_state = AgentInstanceState {
                instance_id: instance_id.clone(),
                agent_name: fleet_agent.name.clone(),
                replica_index: replica_idx,
                status: AgentInstanceStatus::Idle,
                current_task: None,
                tasks_processed: 0,
                last_activity: Some(chrono::Utc::now()),
            };

            // Add to state
            {
                let mut state = self.state.write().await;
                state.agents.insert(instance_id.clone(), instance_state);
            }

            // Emit event
            self.emit_event(FleetEvent::AgentStarted {
                agent_name: fleet_agent.name.clone(),
                instance_id,
            })
            .await;
        }

        Ok(())
    }

    /// Load agent configuration from fleet agent definition
    async fn load_agent_config(&self, fleet_agent: &FleetAgent) -> AofResult<AgentConfig> {
        if let Some(ref config_path) = fleet_agent.config {
            // Load from file
            let content = std::fs::read_to_string(config_path).map_err(|e| {
                AofError::config(format!("Failed to read agent config '{}': {}", config_path, e))
            })?;
            serde_yaml::from_str(&content).map_err(|e| {
                AofError::config(format!("Failed to parse agent config '{}': {}", config_path, e))
            })
        } else if let Some(ref spec) = fleet_agent.spec {
            // Build from inline spec
            Ok(AgentConfig {
                name: fleet_agent.name.clone(),
                model: spec.model.clone(),
                system_prompt: spec.instructions.clone(),
                provider: None,
                tools: spec.tools.clone(),
                mcp_servers: spec.mcp_servers.clone(),
                memory: None,
                max_iterations: spec.max_iterations.map(|v| v as usize).unwrap_or(10),
                temperature: spec.temperature.unwrap_or(0.7),
                max_tokens: None,
                extra: std::collections::HashMap::new(),
            })
        } else {
            Err(AofError::config(format!(
                "Agent '{}' has no config or spec defined",
                fleet_agent.name
            )))
        }
    }

    /// Submit a task to the fleet
    pub async fn submit_task(&self, input: serde_json::Value) -> AofResult<String> {
        let task_id = Uuid::new_v4().to_string();
        let task = FleetTask {
            task_id: task_id.clone(),
            input,
            assigned_to: None,
            status: FleetTaskStatus::Pending,
            result: None,
            error: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        // Add to queue
        {
            let mut queue = self.task_queue.write().await;
            queue.push(task);
        }

        // Update metrics
        {
            let mut state = self.state.write().await;
            state.metrics.total_tasks += 1;
        }

        // Emit event
        self.emit_event(FleetEvent::TaskSubmitted {
            task_id: task_id.clone(),
        })
        .await;

        info!("Task {} submitted to fleet", task_id);

        Ok(task_id)
    }

    /// Execute the next pending task
    pub async fn execute_next(&self) -> AofResult<Option<FleetTask>> {
        // Get next task from queue
        let task = {
            let mut queue = self.task_queue.write().await;
            if queue.is_empty() {
                return Ok(None);
            }
            queue.remove(0)
        };

        // Execute based on coordination mode
        match self.fleet.spec.coordination.mode {
            CoordinationMode::Hierarchical => {
                self.execute_hierarchical(task).await
            }
            CoordinationMode::Peer => {
                self.execute_peer(task).await
            }
            CoordinationMode::Swarm => {
                self.execute_swarm(task).await
            }
            CoordinationMode::Pipeline => {
                self.execute_pipeline(task).await
            }
        }
    }

    /// Execute task in hierarchical mode (manager delegates to workers)
    async fn execute_hierarchical(&self, mut task: FleetTask) -> AofResult<Option<FleetTask>> {
        // Get manager agent
        let manager = self.fleet.get_manager().ok_or_else(|| {
            AofError::runtime("No manager agent defined for hierarchical coordination".to_string())
        })?;

        // First, manager analyzes the task
        let manager_prompt = format!(
            "As the fleet manager, analyze this task and decide how to delegate:\n\n\
             Task: {}\n\n\
             Available workers: {:?}\n\n\
             Respond with a delegation plan.",
            serde_json::to_string_pretty(&task.input).unwrap_or_default(),
            self.fleet
                .get_agents_by_role(AgentRole::Worker)
                .iter()
                .map(|a| &a.name)
                .collect::<Vec<_>>()
        );

        // Execute manager analysis (TODO: actually use result for delegation)
        let _ = {
            let runtime = self.runtime.read().await;
            runtime
                .execute(&manager.name, &manager_prompt)
                .await
                .map_err(|e| AofError::runtime(format!("Manager execution failed: {}", e)))?
        };

        // Select worker based on distribution strategy
        let worker = self.select_worker().await?;

        // Assign and execute task
        task.assigned_to = Some(worker.instance_id.clone());
        task.status = FleetTaskStatus::Assigned;
        task.started_at = Some(chrono::Utc::now());

        self.emit_event(FleetEvent::TaskAssigned {
            task_id: task.task_id.clone(),
            agent_name: worker.agent_name.clone(),
            instance_id: worker.instance_id.clone(),
        })
        .await;

        // Execute on worker
        let result = self.execute_on_agent(&worker.agent_name, &task.input).await;

        self.finish_task(task, result).await
    }

    /// Execute task in peer mode (consensus-based)
    async fn execute_peer(&self, mut task: FleetTask) -> AofResult<Option<FleetTask>> {
        let consensus_config = self.fleet.spec.coordination.consensus.clone();

        // Get all peer agents
        let state = self.state.read().await;
        let agents: Vec<_> = state.agents.values().cloned().collect();
        drop(state);

        if agents.is_empty() {
            return Err(AofError::runtime("No agents available for peer execution".to_string()));
        }

        task.status = FleetTaskStatus::Running;
        task.started_at = Some(chrono::Utc::now());

        // Execute on all agents in parallel
        let mut handles = Vec::new();
        for agent in &agents {
            let agent_name = agent.agent_name.clone();
            let input = task.input.clone();
            let runtime = self.runtime.clone();

            handles.push(tokio::spawn(async move {
                let rt = runtime.read().await;
                rt.execute(&agent_name, &serde_json::to_string(&input).unwrap_or_default()).await
            }));
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => warn!("Agent execution failed: {}", e),
                Err(e) => warn!("Task panicked: {}", e),
            }
        }

        // Apply consensus
        let min_votes = consensus_config
            .as_ref()
            .and_then(|c| c.min_votes)
            .unwrap_or((agents.len() / 2 + 1) as u32);

        if results.len() >= min_votes as usize {
            // Simple consensus: take first result (TODO: implement voting)
            let consensus_result = results.first().cloned().unwrap_or_default();

            self.emit_event(FleetEvent::ConsensusReached {
                task_id: task.task_id.clone(),
                votes: results.len() as u32,
                result: serde_json::json!({"response": consensus_result}),
            })
            .await;

            task.result = Some(serde_json::json!({"response": consensus_result}));
            task.status = FleetTaskStatus::Completed;
        } else {
            task.status = FleetTaskStatus::Failed;
            task.error = Some(format!(
                "Failed to reach consensus: got {} votes, needed {}",
                results.len(),
                min_votes
            ));
        }

        task.completed_at = Some(chrono::Utc::now());

        // Update metrics
        {
            let mut state = self.state.write().await;
            if task.status == FleetTaskStatus::Completed {
                state.metrics.completed_tasks += 1;
            } else {
                state.metrics.failed_tasks += 1;
            }
            state.metrics.consensus_rounds += 1;
            state.completed_tasks.push(task.clone());
        }

        Ok(Some(task))
    }

    /// Execute task in swarm mode (self-organizing)
    async fn execute_swarm(&self, mut task: FleetTask) -> AofResult<Option<FleetTask>> {
        // In swarm mode, agents dynamically decide who handles what
        // For now, use least-loaded selection
        let worker = self.select_least_loaded().await?;

        task.assigned_to = Some(worker.instance_id.clone());
        task.status = FleetTaskStatus::Assigned;
        task.started_at = Some(chrono::Utc::now());

        self.emit_event(FleetEvent::TaskAssigned {
            task_id: task.task_id.clone(),
            agent_name: worker.agent_name.clone(),
            instance_id: worker.instance_id.clone(),
        })
        .await;

        let result = self.execute_on_agent(&worker.agent_name, &task.input).await;
        self.finish_task(task, result).await
    }

    /// Execute task in pipeline mode (sequential handoff)
    async fn execute_pipeline(&self, mut task: FleetTask) -> AofResult<Option<FleetTask>> {
        task.status = FleetTaskStatus::Running;
        task.started_at = Some(chrono::Utc::now());

        let mut current_input = task.input.clone();

        // Execute through each agent in order
        for fleet_agent in &self.fleet.spec.agents {
            debug!("Pipeline stage: {}", fleet_agent.name);

            let result = self.execute_on_agent(&fleet_agent.name, &current_input).await;

            match result {
                Ok(output) => {
                    // Pass output as input to next stage
                    current_input = serde_json::json!({
                        "previous_stage": fleet_agent.name,
                        "input": current_input,
                        "output": output
                    });
                }
                Err(e) => {
                    task.status = FleetTaskStatus::Failed;
                    task.error = Some(format!("Pipeline failed at '{}': {}", fleet_agent.name, e));
                    task.completed_at = Some(chrono::Utc::now());

                    {
                        let mut state = self.state.write().await;
                        state.metrics.failed_tasks += 1;
                        state.completed_tasks.push(task.clone());
                    }

                    return Ok(Some(task));
                }
            }
        }

        // Pipeline completed successfully
        task.result = Some(current_input);
        task.status = FleetTaskStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());

        {
            let mut state = self.state.write().await;
            state.metrics.completed_tasks += 1;
            state.completed_tasks.push(task.clone());
        }

        Ok(Some(task))
    }

    /// Select a worker based on distribution strategy
    async fn select_worker(&self) -> AofResult<AgentInstanceState> {
        match self.fleet.spec.coordination.distribution {
            TaskDistribution::RoundRobin => self.select_round_robin().await,
            TaskDistribution::LeastLoaded => self.select_least_loaded().await,
            TaskDistribution::Random => self.select_random().await,
            _ => self.select_round_robin().await, // Default
        }
    }

    /// Round-robin worker selection
    async fn select_round_robin(&self) -> AofResult<AgentInstanceState> {
        let state = self.state.read().await;
        let workers: Vec<_> = state
            .agents
            .values()
            .filter(|a| a.status == AgentInstanceStatus::Idle)
            .cloned()
            .collect();

        if workers.is_empty() {
            return Err(AofError::runtime("No idle workers available".to_string()));
        }

        let mut counter = self.rr_counter.write().await;
        let idx = *counter % workers.len();
        *counter += 1;

        Ok(workers[idx].clone())
    }

    /// Least-loaded worker selection
    async fn select_least_loaded(&self) -> AofResult<AgentInstanceState> {
        let state = self.state.read().await;
        state
            .agents
            .values()
            .filter(|a| a.status == AgentInstanceStatus::Idle)
            .min_by_key(|a| a.tasks_processed)
            .cloned()
            .ok_or_else(|| AofError::runtime("No idle workers available".to_string()))
    }

    /// Random worker selection
    async fn select_random(&self) -> AofResult<AgentInstanceState> {
        use rand::seq::SliceRandom;

        let state = self.state.read().await;
        let workers: Vec<_> = state
            .agents
            .values()
            .filter(|a| a.status == AgentInstanceStatus::Idle)
            .cloned()
            .collect();

        if workers.is_empty() {
            return Err(AofError::runtime("No idle workers available".to_string()));
        }

        let mut rng = rand::thread_rng();
        Ok(workers.choose(&mut rng).unwrap().clone())
    }

    /// Execute task on a specific agent
    async fn execute_on_agent(
        &self,
        agent_name: &str,
        input: &serde_json::Value,
    ) -> AofResult<String> {
        // Mark agent as busy
        {
            let mut state = self.state.write().await;
            if let Some(agent) = state.agents.values_mut().find(|a| a.agent_name == agent_name) {
                agent.status = AgentInstanceStatus::Busy;
            }
        }

        let input_str = serde_json::to_string(input).unwrap_or_default();
        let result = {
            let runtime = self.runtime.read().await;
            runtime.execute(agent_name, &input_str).await
        };

        // Mark agent as idle
        {
            let mut state = self.state.write().await;
            if let Some(agent) = state.agents.values_mut().find(|a| a.agent_name == agent_name) {
                agent.status = AgentInstanceStatus::Idle;
                agent.tasks_processed += 1;
                agent.last_activity = Some(chrono::Utc::now());
            }
        }

        result.map_err(|e| AofError::runtime(format!("Agent execution failed: {}", e)))
    }

    /// Finish task and update state
    async fn finish_task(
        &self,
        mut task: FleetTask,
        result: AofResult<String>,
    ) -> AofResult<Option<FleetTask>> {
        let start_time = task.started_at.unwrap_or_else(chrono::Utc::now);

        match result {
            Ok(output) => {
                task.result = Some(serde_json::json!({"response": output}));
                task.status = FleetTaskStatus::Completed;
                task.completed_at = Some(chrono::Utc::now());

                let duration_ms = (chrono::Utc::now() - start_time).num_milliseconds() as u64;

                self.emit_event(FleetEvent::TaskCompleted {
                    task_id: task.task_id.clone(),
                    duration_ms,
                })
                .await;

                // Update metrics
                {
                    let mut state = self.state.write().await;
                    state.metrics.completed_tasks += 1;
                    // Update average duration
                    let total = state.metrics.completed_tasks;
                    state.metrics.avg_task_duration_ms = (state.metrics.avg_task_duration_ms
                        * (total - 1) as f64
                        + duration_ms as f64)
                        / total as f64;
                    state.completed_tasks.push(task.clone());
                }
            }
            Err(e) => {
                task.error = Some(e.to_string());
                task.status = FleetTaskStatus::Failed;
                task.completed_at = Some(chrono::Utc::now());

                self.emit_event(FleetEvent::TaskFailed {
                    task_id: task.task_id.clone(),
                    error: e.to_string(),
                })
                .await;

                // Update metrics
                {
                    let mut state = self.state.write().await;
                    state.metrics.failed_tasks += 1;
                    state.completed_tasks.push(task.clone());
                }
            }
        }

        Ok(Some(task))
    }

    /// Stop the fleet
    pub async fn stop(&mut self) -> AofResult<()> {
        info!("Stopping fleet: {}", self.fleet.metadata.name);

        {
            let mut state = self.state.write().await;
            state.status = FleetStatus::ShuttingDown;
        }

        // Cancel pending tasks
        {
            let mut queue = self.task_queue.write().await;
            for task in queue.iter_mut() {
                task.status = FleetTaskStatus::Cancelled;
            }
            queue.clear();
        }

        // Mark agents as stopped
        {
            let mut state = self.state.write().await;
            for agent in state.agents.values_mut() {
                agent.status = AgentInstanceStatus::Stopped;
            }
        }

        self.emit_event(FleetEvent::Stopped {
            fleet_name: self.fleet.metadata.name.clone(),
        })
        .await;

        Ok(())
    }

    /// Get fleet metrics
    pub async fn metrics(&self) -> FleetMetrics {
        self.state.read().await.metrics.clone()
    }

    /// Emit an event
    async fn emit_event(&self, event: FleetEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fleet() -> AgentFleet {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test-fleet
spec:
  agents:
    - name: worker-1
      spec:
        model: openai:gpt-4
        instructions: "Test worker"
        tools: []
    - name: worker-2
      spec:
        model: openai:gpt-4
        instructions: "Test worker"
        tools: []
  coordination:
    mode: peer
    distribution: round-robin
"#;
        AgentFleet::from_yaml(yaml).unwrap()
    }

    #[test]
    fn test_fleet_coordinator_creation() {
        let fleet = create_test_fleet();
        let runtime = Runtime::new();
        let coordinator = FleetCoordinator::new(fleet, runtime);

        assert_eq!(coordinator.fleet().metadata.name, "test-fleet");
    }

    #[tokio::test]
    async fn test_fleet_state_initialization() {
        let fleet = create_test_fleet();
        let runtime = Runtime::new();
        let coordinator = FleetCoordinator::new(fleet, runtime);

        let state = coordinator.state().await;
        assert_eq!(state.status, FleetStatus::Initializing);
        assert_eq!(state.fleet_name, "test-fleet");
    }

    #[tokio::test]
    async fn test_task_submission() {
        let fleet = create_test_fleet();
        let runtime = Runtime::new();
        let coordinator = FleetCoordinator::new(fleet, runtime);

        let task_id = coordinator
            .submit_task(serde_json::json!({"query": "test"}))
            .await
            .unwrap();

        assert!(!task_id.is_empty());

        let state = coordinator.state().await;
        assert_eq!(state.metrics.total_tasks, 1);
    }
}
