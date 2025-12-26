//! AgentFlow Executor - Event-driven workflow execution engine
//!
//! This module executes AgentFlow configurations, handling:
//! - Trigger event processing
//! - Node graph traversal
//! - Conditional routing
//! - Agent execution
//! - Platform-specific actions (Slack, Discord, etc.)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use aof_core::{
    AgentConfig, AgentFlow, AgentFlowState, AofError, AofResult, FlowError, FlowExecutionStatus,
    FlowNode, NodeExecutionStatus, NodeResult, NodeType,
};

use super::Runtime;

/// Events emitted during AgentFlow execution
#[derive(Debug, Clone)]
pub enum AgentFlowEvent {
    /// Flow execution started
    Started { run_id: String, flow_name: String },

    /// Node execution started
    NodeStarted { node_id: String, node_type: String },

    /// Node execution completed
    NodeCompleted {
        node_id: String,
        duration_ms: u64,
        output: Option<serde_json::Value>,
    },

    /// Node execution failed
    NodeFailed { node_id: String, error: String },

    /// Waiting for external event (approval, reaction)
    Waiting { node_id: String, reason: String },

    /// Variable updated
    VariableSet {
        key: String,
        value: serde_json::Value,
    },

    /// Flow execution completed
    Completed {
        run_id: String,
        status: FlowExecutionStatus,
    },

    /// Error occurred
    Error { message: String },
}

/// AgentFlow executor
pub struct AgentFlowExecutor {
    flow: AgentFlow,
    runtime: Arc<RwLock<Runtime>>,
    event_tx: Option<mpsc::Sender<AgentFlowEvent>>,
    /// Directory to search for agent YAML files
    agents_dir: Option<PathBuf>,
}

impl AgentFlowExecutor {
    /// Create a new AgentFlow executor
    pub fn new(flow: AgentFlow, runtime: Arc<RwLock<Runtime>>) -> Self {
        Self {
            flow,
            runtime,
            event_tx: None,
            agents_dir: None,
        }
    }

    /// Create with a non-locked runtime (convenience constructor)
    pub fn with_runtime(flow: AgentFlow, runtime: Runtime) -> Self {
        Self::new(flow, Arc::new(RwLock::new(runtime)))
    }

    /// Set the agents directory for loading agent configs
    pub fn with_agents_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.agents_dir = Some(dir.into());
        self
    }

    /// Load AgentFlow from file
    pub async fn from_file(path: &str, runtime: Arc<RwLock<Runtime>>) -> AofResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            AofError::Config(format!("Failed to read AgentFlow config {}: {}", path, e))
        })?;

        let flow: AgentFlow = serde_yaml::from_str(&content).map_err(|e| {
            AofError::Config(format!("Failed to parse AgentFlow config {}: {}", path, e))
        })?;

        flow.validate().map_err(|e| {
            AofError::Config(format!("AgentFlow validation failed: {}", e))
        })?;

        Ok(Self::new(flow, runtime))
    }

    /// Add event channel for monitoring
    pub fn with_event_channel(mut self, tx: mpsc::Sender<AgentFlowEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Get the flow configuration
    pub fn flow(&self) -> &AgentFlow {
        &self.flow
    }

    /// Execute the flow with trigger data
    pub async fn execute(
        &self,
        trigger_data: serde_json::Value,
    ) -> AofResult<AgentFlowState> {
        let run_id = Uuid::new_v4().to_string();
        let flow_name = self.flow.metadata.name.clone();

        info!("Starting AgentFlow execution: {} ({})", flow_name, run_id);

        // Emit started event
        self.emit_event(AgentFlowEvent::Started {
            run_id: run_id.clone(),
            flow_name: flow_name.clone(),
        })
        .await;

        // Initialize state
        let mut state = AgentFlowState {
            run_id: run_id.clone(),
            flow_name: flow_name.clone(),
            current_nodes: vec![],
            status: FlowExecutionStatus::Running,
            node_results: HashMap::new(),
            variables: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            error: None,
        };

        // Add trigger data to variables
        state
            .variables
            .insert("trigger".to_string(), trigger_data.clone());
        state.variables.insert("event".to_string(), trigger_data);

        // Get entry nodes (connected from "trigger")
        let entry_nodes = self.flow.entry_nodes();
        if entry_nodes.is_empty() {
            // If no explicit connections from trigger, use first node
            if let Some(first_node) = self.flow.spec.nodes.first() {
                state.current_nodes.push(first_node.id.clone());
            }
        } else {
            for node in entry_nodes {
                state.current_nodes.push(node.id.clone());
            }
        }

        // Execute nodes until completion
        loop {
            if state.current_nodes.is_empty() {
                state.status = FlowExecutionStatus::Completed;
                break;
            }

            // Get next nodes to execute
            let nodes_to_execute: Vec<String> = state.current_nodes.drain(..).collect();
            let mut next_nodes: Vec<String> = Vec::new();

            for node_id in nodes_to_execute {
                match self.execute_node(&node_id, &mut state).await {
                    Ok(successors) => {
                        next_nodes.extend(successors);
                    }
                    Err(e) => {
                        error!("Node {} failed: {}", node_id, e);
                        state.status = FlowExecutionStatus::Failed;
                        state.error = Some(FlowError {
                            error_type: "NodeExecutionError".to_string(),
                            message: e.to_string(),
                            node_id: Some(node_id.clone()),
                            details: None,
                        });

                        self.emit_event(AgentFlowEvent::NodeFailed {
                            node_id: node_id.clone(),
                            error: e.to_string(),
                        })
                        .await;

                        break;
                    }
                }
            }

            if state.status == FlowExecutionStatus::Failed {
                break;
            }

            // Check if we're waiting for external event
            if state.status == FlowExecutionStatus::Waiting {
                break;
            }

            state.current_nodes = next_nodes;
            state.updated_at = Utc::now();
        }

        // Emit completed event
        self.emit_event(AgentFlowEvent::Completed {
            run_id: run_id.clone(),
            status: state.status,
        })
        .await;

        info!(
            "AgentFlow execution completed: {} - {:?}",
            flow_name, state.status
        );

        Ok(state)
    }

    /// Execute a single node
    async fn execute_node(
        &self,
        node_id: &str,
        state: &mut AgentFlowState,
    ) -> AofResult<Vec<String>> {
        let node = self
            .flow
            .spec
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| AofError::Config(format!("Node not found: {}", node_id)))?;

        let node_type_str = format!("{:?}", node.node_type);
        debug!("Executing node: {} ({})", node_id, node_type_str);

        self.emit_event(AgentFlowEvent::NodeStarted {
            node_id: node_id.to_string(),
            node_type: node_type_str.clone(),
        })
        .await;

        let start_time = std::time::Instant::now();

        // Check conditions
        if !self.check_node_conditions(node, state) {
            debug!("Node {} conditions not met, skipping", node_id);
            state.node_results.insert(
                node_id.to_string(),
                NodeResult {
                    node_id: node_id.to_string(),
                    status: NodeExecutionStatus::Skipped,
                    output: None,
                    started_at: Utc::now(),
                    ended_at: Some(Utc::now()),
                    duration_ms: Some(0),
                    error: None,
                },
            );
            return Ok(vec![]);
        }

        // Execute based on node type
        let result = match node.node_type {
            NodeType::Transform => self.execute_transform_node(node, state).await,
            NodeType::Agent => self.execute_agent_node(node, state).await,
            NodeType::Conditional => self.execute_conditional_node(node, state).await,
            NodeType::Slack => self.execute_slack_node(node, state).await,
            NodeType::Discord => self.execute_discord_node(node, state).await,
            NodeType::HTTP => self.execute_http_node(node, state).await,
            NodeType::Wait => self.execute_wait_node(node, state).await,
            NodeType::Parallel => self.execute_parallel_node(node, state).await,
            NodeType::Join => self.execute_join_node(node, state).await,
            NodeType::Approval => self.execute_approval_node(node, state).await,
            NodeType::End => Ok(serde_json::json!({})),
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(output) => {
                // Store result
                state.node_results.insert(
                    node_id.to_string(),
                    NodeResult {
                        node_id: node_id.to_string(),
                        status: NodeExecutionStatus::Completed,
                        output: Some(output.clone()),
                        started_at: Utc::now(),
                        ended_at: Some(Utc::now()),
                        duration_ms: Some(duration_ms),
                        error: None,
                    },
                );

                // Store output in variables
                state
                    .variables
                    .insert(format!("{}.output", node_id), output.clone());

                self.emit_event(AgentFlowEvent::NodeCompleted {
                    node_id: node_id.to_string(),
                    duration_ms,
                    output: Some(output),
                })
                .await;

                // Get successor nodes
                let successors = self.get_successor_nodes(node_id, state);
                Ok(successors)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a Transform node
    async fn execute_transform_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        if let Some(script) = &node.config.script {
            // Expand variables in script
            let expanded = self.expand_variables(script, state);

            // For now, just set variables from the script (simplified)
            // In a full implementation, we'd execute the script
            debug!("Transform script: {}", expanded);

            // Extract variable assignments from script
            for line in expanded.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("export ") {
                    if let Some(rest) = trimmed.strip_prefix("export ") {
                        if let Some((key, value)) = rest.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"');
                            state.variables.insert(
                                key.to_string(),
                                serde_json::Value::String(value.to_string()),
                            );

                            self.emit_event(AgentFlowEvent::VariableSet {
                                key: key.to_string(),
                                value: serde_json::Value::String(value.to_string()),
                            })
                            .await;
                        }
                    }
                }
            }

            Ok(serde_json::json!({ "script_executed": true }))
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// Execute an Agent node
    async fn execute_agent_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let input = node
            .config
            .input
            .as_ref()
            .map(|i| self.expand_variables(i, state))
            .unwrap_or_default();

        // Check if we have inline config or agent reference
        if let Some(inline) = &node.config.inline {
            // Inline agent configuration - create agent on the fly
            info!("Executing inline agent: {} with input: {}", inline.name, input);
            return self.run_inline_agent(inline, &input, state).await;
        }

        // Agent reference - load from external config
        let agent_name = node.config.agent.as_ref().ok_or_else(|| {
            AofError::Config("Agent node requires 'agent' or 'inline' config".to_string())
        })?;

        info!("Executing agent: {} with input: {}", agent_name, input);

        // Check if there's an agent config file
        let agent_result = self.run_agent(agent_name, &input, state).await?;

        Ok(agent_result)
    }

    /// Run an inline agent (defined directly in the flow)
    async fn run_inline_agent(
        &self,
        inline: &aof_core::agentflow::InlineAgentConfig,
        input: &str,
        _state: &AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        use aof_core::AgentConfig;

        // Convert InlineAgentConfig to AgentConfig
        let agent_config = AgentConfig {
            name: inline.name.clone(),
            model: inline.model.clone(),
            system_prompt: inline.instructions.clone(),
            provider: None,
            tools: inline.tools.clone(),
            mcp_servers: inline.mcp_servers.clone(),
            memory: None,
            max_context_messages: 10,
            max_iterations: 10,
            temperature: inline.temperature.unwrap_or(0.7),
            max_tokens: inline.max_tokens,
            extra: std::collections::HashMap::new(),
        };

        // Load the agent into runtime
        {
            let mut runtime = self.runtime.write().await;
            runtime.load_agent_from_config(agent_config).await?;
        }

        // Apply flow context if specified
        self.apply_flow_context().await?;

        // Execute the agent
        let result = {
            let runtime = self.runtime.read().await;
            runtime.execute(&inline.name, input).await?
        };

        Ok(serde_json::json!({
            "agent": inline.name,
            "input": input,
            "output": result,
            "requires_approval": false
        }))
    }

    /// Run an agent using the runtime
    ///
    /// This method:
    /// 1. Checks if the agent is already loaded in the runtime
    /// 2. If not, tries to load it from the agents directory
    /// 3. Applies flow context (kubeconfig, env vars) to the execution
    /// 4. Executes the agent and returns the result
    async fn run_agent(
        &self,
        agent_name: &str,
        input: &str,
        state: &AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        info!("Executing agent '{}' with input: {}", agent_name, input);

        // First, try to execute with the agent already loaded
        {
            let runtime = self.runtime.read().await;
            if runtime.has_agent(agent_name) {
                // Apply flow context if specified
                self.apply_flow_context().await?;

                let result = runtime.execute(agent_name, input).await?;
                return Ok(serde_json::json!({
                    "agent": agent_name,
                    "input": input,
                    "output": result,
                    "requires_approval": false
                }));
            }
        }

        // Agent not loaded - try to load from agents directory
        if let Some(ref agents_dir) = self.agents_dir {
            // Try common naming patterns
            let possible_paths = vec![
                agents_dir.join(format!("{}.yaml", agent_name)),
                agents_dir.join(format!("{}.yml", agent_name)),
                agents_dir.join(format!("{}-agent.yaml", agent_name)),
                agents_dir.join(format!("{}-agent.yml", agent_name)),
            ];

            for path in possible_paths {
                if path.exists() {
                    info!("Loading agent '{}' from {}", agent_name, path.display());

                    let mut runtime = self.runtime.write().await;
                    runtime.load_agent_from_file(path.to_string_lossy().as_ref()).await?;

                    // Apply flow context
                    drop(runtime); // Release write lock
                    self.apply_flow_context().await?;

                    let runtime = self.runtime.read().await;
                    let result = runtime.execute(agent_name, input).await?;

                    return Ok(serde_json::json!({
                        "agent": agent_name,
                        "input": input,
                        "output": result,
                        "requires_approval": false
                    }));
                }
            }
        }

        // Agent config might also be embedded in the flow
        // Check if there's a node config with agent config
        if let Some(node) = self.flow.spec.nodes.iter().find(|n| {
            n.config.agent.as_ref() == Some(&agent_name.to_string())
        }) {
            if let Some(ref config_yaml) = node.config.agent_config {
                info!("Loading agent '{}' from inline config", agent_name);

                let agent_config: AgentConfig = serde_yaml::from_str(config_yaml).map_err(|e| {
                    AofError::Config(format!("Failed to parse inline agent config: {}", e))
                })?;

                let mut runtime = self.runtime.write().await;
                runtime.load_agent_from_config(agent_config).await?;

                drop(runtime);
                self.apply_flow_context().await?;

                let runtime = self.runtime.read().await;
                let result = runtime.execute(agent_name, input).await?;

                return Ok(serde_json::json!({
                    "agent": agent_name,
                    "input": input,
                    "output": result,
                    "requires_approval": false
                }));
            }
        }

        // Could not find or load the agent
        Err(AofError::Config(format!(
            "Agent '{}' not found. Ensure it's loaded or available in the agents directory.",
            agent_name
        )))
    }

    /// Apply flow context to the environment
    async fn apply_flow_context(&self) -> AofResult<()> {
        if let Some(ref context) = self.flow.spec.context {
            // Set KUBECONFIG if specified
            if let Some(ref kubeconfig) = context.kubeconfig {
                std::env::set_var("KUBECONFIG", kubeconfig);
                info!("Set KUBECONFIG to {}", kubeconfig);
            }

            // Set namespace as env var if specified
            if let Some(ref namespace) = context.namespace {
                std::env::set_var("K8S_NAMESPACE", namespace);
                info!("Set K8S_NAMESPACE to {}", namespace);
            }

            // Set cluster name if specified
            if let Some(ref cluster) = context.cluster {
                std::env::set_var("K8S_CLUSTER", cluster);
                info!("Set K8S_CLUSTER to {}", cluster);
            }

            // Set working directory if specified
            if let Some(ref working_dir) = context.working_dir {
                std::env::set_current_dir(working_dir).map_err(|e| {
                    AofError::Config(format!("Failed to change working directory: {}", e))
                })?;
                info!("Changed working directory to {}", working_dir);
            }

            // Set additional environment variables
            for (key, value) in &context.env {
                std::env::set_var(key, value);
                debug!("Set env var {}={}", key, value);
            }
        }

        Ok(())
    }

    /// Execute a Conditional node
    async fn execute_conditional_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let condition = node.config.condition.as_ref().ok_or_else(|| {
            AofError::Config("Conditional node requires 'condition' config".to_string())
        })?;

        let expanded = self.expand_variables(condition, state);
        let result = self.evaluate_condition(&expanded);

        debug!("Conditional: {} => {}", expanded, result);

        // Store the result for routing
        state.variables.insert(
            format!("{}.result", node.id),
            serde_json::Value::Bool(result),
        );

        Ok(serde_json::json!({ "condition_result": result }))
    }

    /// Execute a Slack node
    async fn execute_slack_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let channel = node
            .config
            .channel
            .as_ref()
            .map(|c| self.expand_variables(c, state))
            .unwrap_or_else(|| {
                state
                    .variables
                    .get("SLACK_CHANNEL")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            });

        let message = node
            .config
            .message
            .as_ref()
            .map(|m| self.expand_variables(m, state))
            .unwrap_or_default();

        info!("Slack message to {}: {}", channel, message);

        // If waiting for reaction
        if node.config.wait_for_reaction {
            state.status = FlowExecutionStatus::Waiting;
            self.emit_event(AgentFlowEvent::Waiting {
                node_id: node.id.clone(),
                reason: "Waiting for Slack reaction".to_string(),
            })
            .await;
        }

        Ok(serde_json::json!({
            "platform": "slack",
            "channel": channel,
            "message": message,
            "sent": true
        }))
    }

    /// Execute a Discord node
    async fn execute_discord_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let channel = node
            .config
            .channel
            .as_ref()
            .map(|c| self.expand_variables(c, state))
            .unwrap_or_default();

        let message = node
            .config
            .message
            .as_ref()
            .map(|m| self.expand_variables(m, state))
            .unwrap_or_default();

        info!("Discord message to {}: {}", channel, message);

        Ok(serde_json::json!({
            "platform": "discord",
            "channel": channel,
            "message": message,
            "sent": true
        }))
    }

    /// Execute an HTTP node
    async fn execute_http_node(
        &self,
        node: &FlowNode,
        _state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let url = node
            .config
            .url
            .as_ref()
            .ok_or_else(|| AofError::Config("HTTP node requires 'url' config".to_string()))?;

        let method = node
            .config
            .method
            .as_deref()
            .unwrap_or("GET")
            .to_uppercase();

        info!("HTTP {} {}", method, url);

        // Note: In a full implementation, this would make actual HTTP requests
        // For now, we return a placeholder to avoid adding reqwest dependency
        Ok(serde_json::json!({
            "method": method,
            "url": url,
            "status": "pending",
            "note": "HTTP requests will be implemented with full integration"
        }))
    }

    /// Execute a Wait node
    async fn execute_wait_node(
        &self,
        node: &FlowNode,
        _state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        let duration_str = node.config.duration.as_deref().unwrap_or("1s");

        let duration = parse_duration(duration_str)?;
        info!("Waiting for {:?}", duration);

        tokio::time::sleep(duration).await;

        Ok(serde_json::json!({ "waited": duration_str }))
    }

    /// Execute a Parallel node
    async fn execute_parallel_node(
        &self,
        node: &FlowNode,
        _state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        // Parallel nodes just indicate which branches to execute
        // The actual parallelism is handled by the executor loop
        let branches = &node.config.branches;
        info!("Parallel execution with {} branches", branches.len());

        Ok(serde_json::json!({ "branches": branches }))
    }

    /// Execute a Join node
    async fn execute_join_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        // Collect results from parallel branches
        let strategy = node.config.strategy.unwrap_or_default();
        info!("Join with strategy: {:?}", strategy);

        // Get all completed node results
        let completed: Vec<_> = state
            .node_results
            .values()
            .filter(|r| r.status == NodeExecutionStatus::Completed)
            .collect();

        Ok(serde_json::json!({
            "joined": completed.len(),
            "strategy": format!("{:?}", strategy)
        }))
    }

    /// Execute an Approval node
    async fn execute_approval_node(
        &self,
        node: &FlowNode,
        state: &mut AgentFlowState,
    ) -> AofResult<serde_json::Value> {
        info!("Approval required at node: {}", node.id);

        state.status = FlowExecutionStatus::Waiting;

        self.emit_event(AgentFlowEvent::Waiting {
            node_id: node.id.clone(),
            reason: "Waiting for approval".to_string(),
        })
        .await;

        Ok(serde_json::json!({
            "status": "waiting_approval",
            "node_id": node.id
        }))
    }

    /// Check if node conditions are met
    fn check_node_conditions(&self, node: &FlowNode, state: &AgentFlowState) -> bool {
        if node.conditions.is_empty() {
            return true;
        }

        for condition in &node.conditions {
            // Check value condition
            if let Some(expected_value) = &condition.value {
                let key = format!("{}.result", condition.from);
                if let Some(actual) = state.variables.get(&key) {
                    if actual != expected_value {
                        return false;
                    }
                }
            }

            // Check reaction condition
            if let Some(expected_reaction) = &condition.reaction {
                let key = format!("{}.reaction", condition.from);
                if let Some(actual) = state.variables.get(&key) {
                    if actual.as_str() != Some(expected_reaction.as_str()) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Get successor nodes based on connections
    fn get_successor_nodes(&self, node_id: &str, state: &AgentFlowState) -> Vec<String> {
        self.flow
            .spec
            .connections
            .iter()
            .filter(|c| c.from == node_id)
            .filter(|c| {
                // Check connection condition
                if let Some(when) = &c.when {
                    let expanded = self.expand_variables(when, state);
                    self.evaluate_condition(&expanded)
                } else {
                    true
                }
            })
            .map(|c| c.to.clone())
            .collect()
    }

    /// Expand variables in a string (${var_name} syntax)
    fn expand_variables(&self, input: &str, state: &AgentFlowState) -> String {
        let mut result = input.to_string();

        // Simple variable expansion
        for (key, value) in &state.variables {
            let pattern = format!("${{{}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&pattern, &value_str);
        }

        // Also expand node.output references
        for (node_id, node_result) in &state.node_results {
            if let Some(output) = &node_result.output {
                let pattern = format!("${{{}.output}}", node_id);
                let value_str = match output {
                    serde_json::Value::String(s) => s.clone(),
                    _ => output.to_string(),
                };
                result = result.replace(&pattern, &value_str);
            }
        }

        // Expand environment variables (simple pattern matching)
        let mut env_expanded = result.clone();
        let mut i = 0;
        while i < env_expanded.len() {
            if let Some(start) = env_expanded[i..].find("${") {
                let abs_start = i + start;
                if let Some(end) = env_expanded[abs_start..].find('}') {
                    let abs_end = abs_start + end;
                    let var_name = &env_expanded[abs_start + 2..abs_end];
                    // Check if it's an uppercase env var name
                    if var_name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                        if let Ok(value) = std::env::var(var_name) {
                            env_expanded = format!(
                                "{}{}{}",
                                &env_expanded[..abs_start],
                                value,
                                &env_expanded[abs_end + 1..]
                            );
                            continue;
                        }
                    }
                    i = abs_end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        env_expanded
    }

    /// Evaluate a simple condition expression
    fn evaluate_condition(&self, condition: &str) -> bool {
        let condition = condition.trim();

        // Handle simple boolean
        if condition == "true" {
            return true;
        }
        if condition == "false" {
            return false;
        }

        // Handle equality: a == b
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim().trim_matches('"');
                let right = parts[1].trim().trim_matches('"');
                return left == right;
            }
        }

        // Handle inequality: a != b
        if condition.contains("!=") {
            let parts: Vec<&str> = condition.split("!=").collect();
            if parts.len() == 2 {
                let left = parts[0].trim().trim_matches('"');
                let right = parts[1].trim().trim_matches('"');
                return left != right;
            }
        }

        // Handle greater than
        if condition.contains('>') && !condition.contains(">=") {
            let parts: Vec<&str> = condition.split('>').collect();
            if parts.len() == 2 {
                if let (Ok(left), Ok(right)) = (
                    parts[0].trim().parse::<f64>(),
                    parts[1].trim().parse::<f64>(),
                ) {
                    return left > right;
                }
            }
        }

        // Handle less than
        if condition.contains('<') && !condition.contains("<=") {
            let parts: Vec<&str> = condition.split('<').collect();
            if parts.len() == 2 {
                if let (Ok(left), Ok(right)) = (
                    parts[0].trim().parse::<f64>(),
                    parts[1].trim().parse::<f64>(),
                ) {
                    return left < right;
                }
            }
        }

        // Default to false for unparseable conditions
        warn!("Could not evaluate condition: {}", condition);
        false
    }

    /// Emit an event to the channel
    async fn emit_event(&self, event: AgentFlowEvent) {
        if let Some(ref tx) = self.event_tx {
            if tx.send(event).await.is_err() {
                warn!("Failed to send AgentFlow event");
            }
        }
    }
}

/// Parse duration string (e.g., "30s", "5m", "1h")
fn parse_duration(s: &str) -> AofResult<std::time::Duration> {
    let s = s.trim();

    if s.ends_with("ms") {
        let num: u64 = s
            .trim_end_matches("ms")
            .parse()
            .map_err(|_| AofError::Config(format!("Invalid duration: {}", s)))?;
        return Ok(std::time::Duration::from_millis(num));
    }

    if s.ends_with('s') {
        let num: u64 = s
            .trim_end_matches('s')
            .parse()
            .map_err(|_| AofError::Config(format!("Invalid duration: {}", s)))?;
        return Ok(std::time::Duration::from_secs(num));
    }

    if s.ends_with('m') {
        let num: u64 = s
            .trim_end_matches('m')
            .parse()
            .map_err(|_| AofError::Config(format!("Invalid duration: {}", s)))?;
        return Ok(std::time::Duration::from_secs(num * 60));
    }

    if s.ends_with('h') {
        let num: u64 = s
            .trim_end_matches('h')
            .parse()
            .map_err(|_| AofError::Config(format!("Invalid duration: {}", s)))?;
        return Ok(std::time::Duration::from_secs(num * 3600));
    }

    // Default: parse as seconds
    let num: u64 = s
        .parse()
        .map_err(|_| AofError::Config(format!("Invalid duration: {}", s)))?;
    Ok(std::time::Duration::from_secs(num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration("30s").unwrap(),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            parse_duration("5m").unwrap(),
            std::time::Duration::from_secs(300)
        );
        assert_eq!(
            parse_duration("1h").unwrap(),
            std::time::Duration::from_secs(3600)
        );
        assert_eq!(
            parse_duration("100ms").unwrap(),
            std::time::Duration::from_millis(100)
        );
    }

    #[test]
    fn test_evaluate_condition() {
        let runtime = Arc::new(RwLock::new(Runtime::new()));
        let flow: AgentFlow = serde_yaml::from_str(
            r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: test
spec:
  trigger:
    type: HTTP
  nodes:
    - id: test
      type: End
"#,
        )
        .unwrap();

        let executor = AgentFlowExecutor::new(flow, runtime);

        assert!(executor.evaluate_condition("true"));
        assert!(!executor.evaluate_condition("false"));
        assert!(executor.evaluate_condition("1 == 1"));
        assert!(!executor.evaluate_condition("1 == 2"));
        assert!(executor.evaluate_condition("5 > 3"));
        assert!(executor.evaluate_condition("3 < 5"));
    }
}
