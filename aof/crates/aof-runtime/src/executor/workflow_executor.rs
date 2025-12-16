//! Workflow Executor - Graph-based workflow execution engine
//!
//! Executes workflows with conditional routing, parallel execution,
//! human-in-the-loop approval, and state management.

use aof_core::{
    AofError, AofResult, JoinStrategy, NextStep, StepResult, StepStatus, StepType, TerminalStatus,
    Workflow, WorkflowConfigInput, WorkflowError, WorkflowState, WorkflowStatus, WorkflowStep,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::Runtime;

/// Events emitted during workflow execution
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// Workflow started
    Started {
        run_id: String,
        workflow_name: String,
    },
    /// Step started
    StepStarted {
        step_name: String,
    },
    /// Step completed
    StepCompleted {
        step_name: String,
        duration_ms: u64,
    },
    /// Step failed
    StepFailed {
        step_name: String,
        error: String,
    },
    /// State updated
    StateUpdated {
        key: String,
        value: serde_json::Value,
    },
    /// Waiting for approval
    WaitingApproval {
        step_name: String,
        approvers: Vec<String>,
    },
    /// Waiting for input
    WaitingInput {
        step_name: String,
        prompt: String,
    },
    /// Workflow completed
    Completed {
        run_id: String,
        status: WorkflowStatus,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// Workflow executor for running graph-based workflows
pub struct WorkflowExecutor {
    /// The workflow definition
    workflow: Workflow,
    /// Current execution state
    state: Arc<RwLock<WorkflowState>>,
    /// Agent runtime for executing agent steps
    runtime: Arc<Runtime>,
    /// Event channel sender
    event_tx: Option<mpsc::Sender<WorkflowEvent>>,
    /// Approval channel for receiving approval decisions
    approval_rx: Option<mpsc::Receiver<ApprovalDecision>>,
    /// Input channel for receiving human input
    input_rx: Option<mpsc::Receiver<HumanInput>>,
}

/// Approval decision from human
#[derive(Debug, Clone)]
pub struct ApprovalDecision {
    pub step_name: String,
    pub approved: bool,
    pub approver: String,
    pub comment: Option<String>,
}

/// Human input response
#[derive(Debug, Clone)]
pub struct HumanInput {
    pub step_name: String,
    pub data: serde_json::Value,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new(workflow: Workflow, runtime: Arc<Runtime>) -> Self {
        let run_id = Uuid::new_v4().to_string();
        let state = WorkflowState {
            run_id: run_id.clone(),
            workflow_name: workflow.metadata.name.clone(),
            current_step: workflow.spec.entrypoint.clone(),
            status: WorkflowStatus::Pending,
            data: serde_json::json!({}),
            completed_steps: Vec::new(),
            step_results: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            error: None,
        };

        Self {
            workflow,
            state: Arc::new(RwLock::new(state)),
            runtime,
            event_tx: None,
            approval_rx: None,
            input_rx: None,
        }
    }

    /// Load a workflow from a YAML file
    pub async fn from_file(path: &str, runtime: Arc<Runtime>) -> AofResult<Self> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            AofError::config(format!("Failed to read workflow file {}: {}", path, e))
        })?;

        let input: WorkflowConfigInput = serde_yaml::from_str(&content)
            .map_err(|e| AofError::config(format!("Failed to parse workflow YAML: {}", e)))?;

        let workflow: Workflow = input.into();
        Ok(Self::new(workflow, runtime))
    }

    /// Set the event channel for streaming workflow events
    pub fn with_event_channel(mut self, tx: mpsc::Sender<WorkflowEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Set the approval channel for receiving approval decisions
    pub fn with_approval_channel(mut self, rx: mpsc::Receiver<ApprovalDecision>) -> Self {
        self.approval_rx = Some(rx);
        self
    }

    /// Set the input channel for receiving human input
    pub fn with_input_channel(mut self, rx: mpsc::Receiver<HumanInput>) -> Self {
        self.input_rx = Some(rx);
        self
    }

    /// Execute the workflow with initial state
    pub async fn execute(&mut self, initial_state: serde_json::Value) -> AofResult<WorkflowState> {
        // Initialize state with input
        {
            let mut state = self.state.write().await;
            state.data = initial_state;
            state.status = WorkflowStatus::Running;
            state.updated_at = Utc::now();
        }

        // Send started event
        let state_snapshot = self.state.read().await.clone();
        self.emit_event(WorkflowEvent::Started {
            run_id: state_snapshot.run_id.clone(),
            workflow_name: state_snapshot.workflow_name.clone(),
        })
        .await;

        // Execute workflow loop
        loop {
            let current_step = {
                let state = self.state.read().await;
                state.current_step.clone()
            };

            // Find the step definition
            let step = self
                .workflow
                .spec
                .steps
                .iter()
                .find(|s| s.name == current_step)
                .cloned();

            let step = match step {
                Some(s) => s,
                None => {
                    let error_msg = format!("Step '{}' not found in workflow", current_step);
                    self.set_error(&error_msg).await;
                    return Err(AofError::workflow(error_msg));
                }
            };

            // Execute the step
            match self.execute_step(&step).await {
                Ok(next_step) => {
                    match next_step {
                        Some(next) => {
                            // Move to next step
                            let mut state = self.state.write().await;
                            state.completed_steps.push(current_step);
                            state.current_step = next;
                            state.updated_at = Utc::now();
                        }
                        None => {
                            // Workflow completed (terminal step)
                            let mut state = self.state.write().await;
                            state.completed_steps.push(current_step);
                            state.status = WorkflowStatus::Completed;
                            state.updated_at = Utc::now();
                            break;
                        }
                    }
                }
                Err(e) => {
                    // Handle error - check for error handler
                    if let Some(ref error_handler) = self.workflow.spec.error_handler {
                        warn!("Step '{}' failed, routing to error handler", current_step);
                        let mut state = self.state.write().await;
                        state.current_step = error_handler.clone();
                        state.updated_at = Utc::now();
                    } else {
                        self.set_error(&e.to_string()).await;
                        return Err(e);
                    }
                }
            }
        }

        // Send completed event
        let final_state = self.state.read().await.clone();
        self.emit_event(WorkflowEvent::Completed {
            run_id: final_state.run_id.clone(),
            status: final_state.status,
        })
        .await;

        Ok(final_state)
    }

    /// Execute a single step
    async fn execute_step(&mut self, step: &WorkflowStep) -> AofResult<Option<String>> {
        info!("Executing step: {} (type: {:?})", step.name, step.step_type);
        self.emit_event(WorkflowEvent::StepStarted {
            step_name: step.name.clone(),
        })
        .await;

        let start_time = Utc::now();
        let start_instant = std::time::Instant::now();

        let result = match step.step_type {
            StepType::Agent => self.execute_agent_step(step).await,
            StepType::Approval => self.execute_approval_step(step).await,
            StepType::Validation => self.execute_validation_step(step).await,
            StepType::Parallel => self.execute_parallel_step(step).await,
            StepType::Join => {
                // Join steps are handled internally by parallel execution
                Ok(self.resolve_next_step(&step.next, &serde_json::json!({})))
            }
            StepType::Terminal => {
                self.execute_terminal_step(step).await?;
                Ok(None) // Terminal steps have no next step
            }
        };

        let duration_ms = start_instant.elapsed().as_millis() as u64;

        // Record step result
        let step_result = match &result {
            Ok(_) => StepResult {
                step_name: step.name.clone(),
                status: StepStatus::Completed,
                output: None,
                started_at: start_time,
                ended_at: Some(Utc::now()),
                duration_ms: Some(duration_ms),
                error: None,
            },
            Err(e) => StepResult {
                step_name: step.name.clone(),
                status: StepStatus::Failed,
                output: None,
                started_at: start_time,
                ended_at: Some(Utc::now()),
                duration_ms: Some(duration_ms),
                error: Some(e.to_string()),
            },
        };

        {
            let mut state = self.state.write().await;
            state.step_results.insert(step.name.clone(), step_result);
        }

        // Emit event
        match &result {
            Ok(_) => {
                self.emit_event(WorkflowEvent::StepCompleted {
                    step_name: step.name.clone(),
                    duration_ms,
                })
                .await;
            }
            Err(e) => {
                self.emit_event(WorkflowEvent::StepFailed {
                    step_name: step.name.clone(),
                    error: e.to_string(),
                })
                .await;
            }
        }

        result
    }

    /// Execute an agent step
    async fn execute_agent_step(&mut self, step: &WorkflowStep) -> AofResult<Option<String>> {
        let agent_name = step.agent.as_ref().ok_or_else(|| {
            AofError::workflow(format!("Agent step '{}' missing agent field", step.name))
        })?;

        // Get current state data
        let state_data = {
            let state = self.state.read().await;
            state.data.clone()
        };

        // Create input for the agent from state
        let input = serde_json::to_string(&state_data).unwrap_or_default();

        // Check if agent is already loaded, if not try to load from fleet or config
        if self.runtime.get_agent(agent_name).is_none() {
            debug!(
                "Agent '{}' not loaded, attempting to load from config",
                agent_name
            );
            // In a full implementation, this would look up agent config from fleet or registry
            return Err(AofError::workflow(format!(
                "Agent '{}' not found. Load agents before executing workflow.",
                agent_name
            )));
        }

        // Execute the agent
        let result = self.runtime.execute(agent_name, &input).await?;

        // Parse agent output and update state
        let output: serde_json::Value = serde_json::from_str(&result).unwrap_or_else(|_| {
            serde_json::json!({
                "response": result
            })
        });

        // Update state with agent output
        self.update_state(output.clone()).await;

        // Run validations if specified
        for validation in &step.validation {
            self.run_validation(validation, &output).await?;
        }

        // Resolve next step
        Ok(self.resolve_next_step(&step.next, &output))
    }

    /// Execute an approval step
    async fn execute_approval_step(&mut self, step: &WorkflowStep) -> AofResult<Option<String>> {
        let config = step.config.as_ref().ok_or_else(|| {
            AofError::workflow(format!("Approval step '{}' missing config", step.name))
        })?;

        // Check for auto-approve condition
        if let Some(ref auto_approve) = config.auto_approve {
            let state_data = self.state.read().await.data.clone();
            if self.evaluate_condition(&auto_approve.condition, &state_data) {
                info!("Auto-approving step '{}' based on condition", step.name);
                return Ok(self.resolve_next_step(
                    &step.next,
                    &serde_json::json!({"approved": true}),
                ));
            }
        }

        // Collect approver identifiers
        let approvers: Vec<String> = config
            .approvers
            .iter()
            .map(|a| {
                a.role
                    .clone()
                    .unwrap_or_else(|| a.user.clone().unwrap_or_default())
            })
            .collect();

        // Update state to waiting
        {
            let mut state = self.state.write().await;
            state.status = WorkflowStatus::WaitingApproval;
            state.updated_at = Utc::now();
        }

        self.emit_event(WorkflowEvent::WaitingApproval {
            step_name: step.name.clone(),
            approvers: approvers.clone(),
        })
        .await;

        // Wait for approval decision
        // In a real implementation, this would wait for external input
        // For now, we'll use the approval channel if available
        if let Some(ref mut approval_rx) = self.approval_rx {
            // Parse timeout
            let timeout_duration = config
                .timeout
                .as_ref()
                .map(|t| parse_duration(t))
                .transpose()?
                .unwrap_or(std::time::Duration::from_secs(3600)); // 1 hour default

            match tokio::time::timeout(timeout_duration, approval_rx.recv()).await {
                Ok(Some(decision)) => {
                    if decision.step_name != step.name {
                        return Err(AofError::workflow("Approval received for wrong step"));
                    }

                    // Update state
                    {
                        let mut state = self.state.write().await;
                        state.status = WorkflowStatus::Running;
                        state.updated_at = Utc::now();
                    }

                    let output = serde_json::json!({
                        "approved": decision.approved,
                        "approver": decision.approver,
                        "comment": decision.comment
                    });

                    Ok(self.resolve_next_step(&step.next, &output))
                }
                Ok(None) => Err(AofError::workflow("Approval channel closed")),
                Err(_) => {
                    // Timeout
                    let output = serde_json::json!({"timeout": true});
                    Ok(self.resolve_next_step(&step.next, &output))
                }
            }
        } else {
            // No approval channel - auto-approve for testing
            warn!(
                "No approval channel configured, auto-approving step '{}'",
                step.name
            );
            Ok(self.resolve_next_step(
                &step.next,
                &serde_json::json!({"approved": true}),
            ))
        }
    }

    /// Execute a validation step
    async fn execute_validation_step(&mut self, step: &WorkflowStep) -> AofResult<Option<String>> {
        let state_data = self.state.read().await.data.clone();

        // Get validators from config
        let empty_validators = Vec::new();
        let validators = step
            .config
            .as_ref()
            .map(|c| &c.validators)
            .unwrap_or(&empty_validators);

        for validator in validators {
            match &validator.validator_type {
                aof_core::ValidatorType::Function => {
                    // Function-based validation
                    if let Some(ref name) = validator.name {
                        debug!("Running function validator: {}", name);
                        // In a full implementation, this would call a registered function
                        // For now, we'll skip
                    }
                }
                aof_core::ValidatorType::Llm => {
                    // LLM-based validation
                    if let (Some(ref _model), Some(ref _prompt)) =
                        (&validator.model, &validator.prompt)
                    {
                        debug!("Running LLM validator");
                        // In a full implementation, this would call the LLM for validation
                    }
                }
                aof_core::ValidatorType::Script => {
                    // Script-based validation
                    if let Some(ref command) = validator.command {
                        debug!("Running script validator: {}", command);
                        let output = tokio::process::Command::new("sh")
                            .arg("-c")
                            .arg(command)
                            .env("STATE", serde_json::to_string(&state_data).unwrap_or_default())
                            .output()
                            .await
                            .map_err(|e| AofError::workflow(format!("Validation script failed: {}", e)))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            return Err(AofError::workflow(format!(
                                "Validation failed: {}",
                                stderr
                            )));
                        }
                    }
                }
            }
        }

        Ok(self.resolve_next_step(&step.next, &state_data))
    }

    /// Execute a parallel step (fork-join)
    async fn execute_parallel_step(&mut self, step: &WorkflowStep) -> AofResult<Option<String>> {
        let branches = step.branches.as_ref().ok_or_else(|| {
            AofError::workflow(format!("Parallel step '{}' missing branches", step.name))
        })?;

        let join_config = step.join.as_ref();
        let strategy = join_config
            .map(|j| j.strategy)
            .unwrap_or(JoinStrategy::All);

        info!(
            "Starting parallel execution with {} branches, strategy: {:?}",
            branches.len(),
            strategy
        );

        // Create tasks for each branch
        let mut handles = Vec::new();
        let runtime = Arc::clone(&self.runtime);
        let state = Arc::clone(&self.state);

        for branch in branches {
            let branch_name = branch.name.clone();
            let branch_steps = branch.steps.clone();
            let runtime = Arc::clone(&runtime);
            let state = Arc::clone(&state);

            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                for branch_step in branch_steps {
                    if let Some(agent_name) = branch_step.agent {
                        let state_data = state.read().await.data.clone();
                        let input = serde_json::to_string(&state_data).unwrap_or_default();

                        match runtime.execute(&agent_name, &input).await {
                            Ok(result) => results.push((branch_name.clone(), Ok(result))),
                            Err(e) => results.push((branch_name.clone(), Err(e))),
                        }
                    }
                }
                results
            });

            handles.push(handle);
        }

        // Wait based on join strategy
        let results = match strategy {
            JoinStrategy::All => {
                // Wait for all to complete
                let mut all_results = Vec::new();
                for handle in handles {
                    match handle.await {
                        Ok(results) => all_results.extend(results),
                        Err(e) => {
                            return Err(AofError::workflow(format!(
                                "Parallel branch panicked: {}",
                                e
                            )));
                        }
                    }
                }
                all_results
            }
            JoinStrategy::Any => {
                // Wait for first to complete
                let (result, _idx, _remaining) = futures::future::select_all(handles).await;
                match result {
                    Ok(results) => results,
                    Err(e) => {
                        return Err(AofError::workflow(format!("Parallel branch panicked: {}", e)));
                    }
                }
            }
            JoinStrategy::Majority => {
                // Wait for majority (50%+1)
                let threshold = (branches.len() / 2) + 1;
                let mut completed = Vec::new();
                let mut remaining = handles;

                while completed.len() < threshold && !remaining.is_empty() {
                    let (result, _, rest) = futures::future::select_all(remaining).await;
                    remaining = rest;
                    if let Ok(results) = result {
                        completed.extend(results);
                    }
                }

                completed
            }
        };

        // Merge results into state
        let mut merged_output = serde_json::json!({
            "branches": {}
        });

        for (branch_name, result) in results {
            match result {
                Ok(output) => {
                    let output_value: serde_json::Value =
                        serde_json::from_str(&output).unwrap_or(serde_json::json!({"output": output}));
                    merged_output["branches"][&branch_name] = output_value;
                }
                Err(e) => {
                    merged_output["branches"][&branch_name] =
                        serde_json::json!({"error": e.to_string()});
                }
            }
        }

        self.update_state(merged_output.clone()).await;

        Ok(self.resolve_next_step(&step.next, &merged_output))
    }

    /// Execute a terminal step
    async fn execute_terminal_step(&mut self, step: &WorkflowStep) -> AofResult<()> {
        let status = step.status.unwrap_or(TerminalStatus::Completed);

        let mut state = self.state.write().await;
        state.status = match status {
            TerminalStatus::Completed => WorkflowStatus::Completed,
            TerminalStatus::Failed => WorkflowStatus::Failed,
            TerminalStatus::Cancelled => WorkflowStatus::Cancelled,
        };
        state.updated_at = Utc::now();

        info!(
            "Workflow reached terminal step '{}' with status {:?}",
            step.name, status
        );

        Ok(())
    }

    /// Resolve the next step based on conditions
    fn resolve_next_step(
        &self,
        next: &Option<NextStep>,
        output: &serde_json::Value,
    ) -> Option<String> {
        match next {
            Some(NextStep::Simple(target)) => Some(target.clone()),
            Some(NextStep::Conditional(conditions)) => {
                for cond in conditions {
                    if let Some(ref condition) = cond.condition {
                        if self.evaluate_condition(condition, output) {
                            return Some(cond.target.clone());
                        }
                    } else {
                        // Default case (no condition)
                        return Some(cond.target.clone());
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Evaluate a condition expression against output/state
    fn evaluate_condition(&self, condition: &str, context: &serde_json::Value) -> bool {
        // Simple condition evaluation
        // Supports: "state.field == 'value'", "state.field > 0.5", etc.
        // Also supports: "approved", "rejected", "timeout" as shortcuts

        // Handle shortcut conditions
        match condition {
            "approved" => {
                return context.get("approved") == Some(&serde_json::json!(true));
            }
            "rejected" => {
                return context.get("approved") == Some(&serde_json::json!(false));
            }
            "timeout" => {
                return context.get("timeout") == Some(&serde_json::json!(true));
            }
            _ => {}
        }

        // Parse state.field comparisons
        if condition.starts_with("state.") {
            // Extract field path and comparison
            let parts: Vec<&str> = if condition.contains("==") {
                condition.splitn(2, "==").collect()
            } else if condition.contains("!=") {
                condition.splitn(2, "!=").collect()
            } else if condition.contains(">=") {
                condition.splitn(2, ">=").collect()
            } else if condition.contains("<=") {
                condition.splitn(2, "<=").collect()
            } else if condition.contains('>') {
                condition.splitn(2, '>').collect()
            } else if condition.contains('<') {
                condition.splitn(2, '<').collect()
            } else {
                return false;
            };

            if parts.len() != 2 {
                return false;
            }

            let field_path = parts[0].trim().strip_prefix("state.").unwrap_or(parts[0].trim());
            let expected = parts[1].trim().trim_matches('\'').trim_matches('"');

            // Navigate to the field value
            let value = self.get_nested_value(context, field_path);

            // Compare based on operator
            if condition.contains("==") {
                match value {
                    Some(serde_json::Value::String(s)) => s == expected,
                    Some(serde_json::Value::Bool(b)) => b.to_string() == expected,
                    Some(serde_json::Value::Number(n)) => n.to_string() == expected,
                    _ => false,
                }
            } else if condition.contains("!=") {
                match value {
                    Some(serde_json::Value::String(s)) => s != expected,
                    _ => true,
                }
            } else if condition.contains(">=") || condition.contains('>') {
                let expected_num: f64 = expected.parse().unwrap_or(0.0);
                match value {
                    Some(serde_json::Value::Number(n)) => {
                        let val = n.as_f64().unwrap_or(0.0);
                        if condition.contains(">=") {
                            val >= expected_num
                        } else {
                            val > expected_num
                        }
                    }
                    _ => false,
                }
            } else if condition.contains("<=") || condition.contains('<') {
                let expected_num: f64 = expected.parse().unwrap_or(0.0);
                match value {
                    Some(serde_json::Value::Number(n)) => {
                        let val = n.as_f64().unwrap_or(0.0);
                        if condition.contains("<=") {
                            val <= expected_num
                        } else {
                            val < expected_num
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // Direct field check
            context.get(condition).is_some()
        }
    }

    /// Get a nested value from JSON using dot notation
    fn get_nested_value<'a>(
        &self,
        value: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(part)?;
                }
                serde_json::Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Update state with new data
    async fn update_state(&self, new_data: serde_json::Value) {
        let mut state = self.state.write().await;

        // Merge new data into existing state
        if let (serde_json::Value::Object(existing), serde_json::Value::Object(new)) =
            (&mut state.data, new_data)
        {
            for (key, value) in new {
                // Apply reducers if defined
                if let Some(reducer) = self.workflow.spec.reducers.get(&key) {
                    match reducer.reducer_type {
                        aof_core::ReducerType::Append => {
                            if let serde_json::Value::Array(ref mut arr) =
                                existing.entry(&key).or_insert(serde_json::json!([]))
                            {
                                if let serde_json::Value::Array(new_arr) = value {
                                    arr.extend(new_arr);
                                } else {
                                    arr.push(value);
                                }
                            }
                        }
                        aof_core::ReducerType::Merge => {
                            if let (
                                serde_json::Value::Object(ref mut obj),
                                serde_json::Value::Object(new_obj),
                            ) = (
                                existing.entry(&key).or_insert(serde_json::json!({})),
                                value,
                            ) {
                                for (k, v) in new_obj {
                                    obj.insert(k, v);
                                }
                            }
                        }
                        aof_core::ReducerType::Sum => {
                            let current = existing
                                .get(&key)
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let new_val = value.as_f64().unwrap_or(0.0);
                            existing.insert(key, serde_json::json!(current + new_val));
                        }
                        aof_core::ReducerType::Replace => {
                            existing.insert(key, value);
                        }
                    }
                } else {
                    // Default: replace
                    existing.insert(key, value);
                }
            }
        }

        state.updated_at = Utc::now();
    }

    /// Run a validation rule
    async fn run_validation(
        &self,
        rule: &aof_core::workflow::ValidationRule,
        output: &serde_json::Value,
    ) -> AofResult<()> {
        match &rule.rule_type {
            aof_core::ValidatorType::Function => {
                if let Some(ref _script) = rule.script {
                    // Function validation - would call registered function
                    debug!("Function validation placeholder");
                }
            }
            aof_core::ValidatorType::Llm => {
                if let Some(ref _prompt) = rule.prompt {
                    // LLM validation - would call LLM
                    debug!("LLM validation placeholder");
                }
            }
            aof_core::ValidatorType::Script => {
                // Script validation already handled in validation step
            }
        }
        Ok(())
    }

    /// Set error state
    async fn set_error(&self, message: &str) {
        let mut state = self.state.write().await;
        state.status = WorkflowStatus::Failed;
        state.error = Some(WorkflowError {
            error_type: "execution_error".to_string(),
            message: message.to_string(),
            step: Some(state.current_step.clone()),
            details: None,
        });
        state.updated_at = Utc::now();

        error!("Workflow error: {}", message);
    }

    /// Emit a workflow event
    async fn emit_event(&self, event: WorkflowEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event).await;
        }
    }

    /// Get current workflow state
    pub async fn get_state(&self) -> WorkflowState {
        self.state.read().await.clone()
    }

    /// Get workflow definition
    pub fn get_workflow(&self) -> &Workflow {
        &self.workflow
    }
}

/// Parse duration string (e.g., "30m", "1h", "10s")
fn parse_duration(s: &str) -> AofResult<std::time::Duration> {
    let s = s.trim();
    let (num, unit) = if s.ends_with("ms") {
        (s.trim_end_matches("ms"), "ms")
    } else if s.ends_with('s') {
        (s.trim_end_matches('s'), "s")
    } else if s.ends_with('m') {
        (s.trim_end_matches('m'), "m")
    } else if s.ends_with('h') {
        (s.trim_end_matches('h'), "h")
    } else {
        return Err(AofError::config(format!("Invalid duration: {}", s)));
    };

    let num: u64 = num
        .parse()
        .map_err(|_| AofError::config(format!("Invalid duration number: {}", s)))?;

    Ok(match unit {
        "ms" => std::time::Duration::from_millis(num),
        "s" => std::time::Duration::from_secs(num),
        "m" => std::time::Duration::from_secs(num * 60),
        "h" => std::time::Duration::from_secs(num * 3600),
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap().as_secs(), 30);
        assert_eq!(parse_duration("5m").unwrap().as_secs(), 300);
        assert_eq!(parse_duration("1h").unwrap().as_secs(), 3600);
        assert_eq!(parse_duration("100ms").unwrap().as_millis(), 100);
    }

    #[test]
    fn test_condition_evaluation() {
        let runtime = Runtime::new();
        let workflow = Workflow {
            api_version: "aof.dev/v1".to_string(),
            kind: "Workflow".to_string(),
            metadata: aof_core::WorkflowMetadata {
                name: "test".to_string(),
                namespace: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
            },
            spec: aof_core::WorkflowSpec {
                state: None,
                entrypoint: "start".to_string(),
                steps: vec![],
                reducers: HashMap::new(),
                error_handler: None,
                retry: None,
                checkpointing: None,
                recovery: None,
                fleet: None,
            },
        };

        let executor = WorkflowExecutor::new(workflow, Arc::new(runtime));

        // Test approved/rejected shortcuts
        assert!(executor.evaluate_condition("approved", &serde_json::json!({"approved": true})));
        assert!(!executor.evaluate_condition("approved", &serde_json::json!({"approved": false})));
        assert!(executor.evaluate_condition("rejected", &serde_json::json!({"approved": false})));

        // Test state.field comparisons
        assert!(executor.evaluate_condition(
            "state.score > 0.5",
            &serde_json::json!({"score": 0.8})
        ));
        assert!(!executor.evaluate_condition(
            "state.score > 0.5",
            &serde_json::json!({"score": 0.3})
        ));
        assert!(executor.evaluate_condition(
            "state.severity == 'high'",
            &serde_json::json!({"severity": "high"})
        ));
    }

    #[test]
    fn test_nested_value_extraction() {
        let runtime = Runtime::new();
        let workflow = Workflow {
            api_version: "aof.dev/v1".to_string(),
            kind: "Workflow".to_string(),
            metadata: aof_core::WorkflowMetadata {
                name: "test".to_string(),
                namespace: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
            },
            spec: aof_core::WorkflowSpec {
                state: None,
                entrypoint: "start".to_string(),
                steps: vec![],
                reducers: HashMap::new(),
                error_handler: None,
                retry: None,
                checkpointing: None,
                recovery: None,
                fleet: None,
            },
        };

        let executor = WorkflowExecutor::new(workflow, Arc::new(runtime));

        let value = serde_json::json!({
            "nested": {
                "deep": {
                    "value": 42
                }
            }
        });

        assert_eq!(
            executor.get_nested_value(&value, "nested.deep.value"),
            Some(&serde_json::json!(42))
        );
    }
}
