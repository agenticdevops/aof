// AOF Core - Workflow configuration types for AgentFlow
//
// This module provides types for defining graph-based workflows with
// conditional routing, human-in-the-loop approval, and parallel execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow definition following Kubernetes-style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    /// API version (e.g., "aof.dev/v1")
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind, always "Workflow"
    #[serde(default = "default_workflow_kind")]
    pub kind: String,

    /// Workflow metadata
    pub metadata: WorkflowMetadata,

    /// Workflow specification
    pub spec: WorkflowSpec,
}

fn default_api_version() -> String {
    "aof.dev/v1".to_string()
}

fn default_workflow_kind() -> String {
    "Workflow".to_string()
}

/// Workflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Workflow name
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

/// Workflow specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSpec {
    /// State schema definition (JSON Schema format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<StateSchema>,

    /// Entry point step name
    pub entrypoint: String,

    /// Workflow steps
    pub steps: Vec<WorkflowStep>,

    /// State reducers for custom update behavior
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub reducers: HashMap<String, StateReducer>,

    /// Global error handler step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handler: Option<String>,

    /// Global retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Checkpointing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpointing: Option<CheckpointConfig>,

    /// Recovery configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoveryConfig>,

    /// Reference to an AgentFleet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet: Option<String>,
}

/// State schema definition (JSON Schema format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    /// Schema type
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Property definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, PropertySchema>,

    /// Required properties
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

/// Property schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Property type
    #[serde(rename = "type")]
    pub prop_type: String,

    /// Enum values for string type
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Items schema for array type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// State reducer for custom update behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct StateReducer {
    /// Reducer type
    #[serde(rename = "type")]
    pub reducer_type: ReducerType,
}

/// Reducer types for state updates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReducerType {
    /// Append to list
    Append,
    /// Merge objects
    Merge,
    /// Sum numeric values
    Sum,
    /// Replace value (default)
    Replace,
}

impl Default for ReducerType {
    fn default() -> Self {
        Self::Replace
    }
}

/// Workflow step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    /// Step name (must be unique within workflow)
    pub name: String,

    /// Step type
    #[serde(rename = "type")]
    pub step_type: StepType,

    /// Agent to execute (for agent steps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Step configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<StepConfig>,

    /// Validation rules
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation: Vec<ValidationRule>,

    /// Next step(s) - can be conditional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<NextStep>,

    /// Parallel execution flag
    #[serde(default)]
    pub parallel: bool,

    /// Parallel branches (for parallel steps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches: Option<Vec<ParallelBranch>>,

    /// Join configuration (for parallel steps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join: Option<JoinConfig>,

    /// Error handling for this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_error: Option<Vec<ConditionalNext>>,

    /// Interrupt configuration for human input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt: Option<InterruptConfig>,

    /// Terminal status (for terminal steps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TerminalStatus>,

    /// Step timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// Step types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// Execute an agent with tools
    Agent,
    /// Human-in-the-loop approval gate
    Approval,
    /// Automated validation step
    Validation,
    /// Fork into multiple parallel steps
    Parallel,
    /// Wait for parallel steps to complete
    Join,
    /// End of workflow
    Terminal,
}

/// Step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepConfig {
    /// Approvers list (for approval steps)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approvers: Vec<Approver>,

    /// Timeout duration (e.g., "30m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,

    /// Required number of approvals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<u32>,

    /// Auto-approve condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_approve: Option<AutoApproveConfig>,

    /// Validators (for validation steps)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validators: Vec<Validator>,

    /// Max retries on validation failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,

    /// Action on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_failure: Option<String>,
}

/// Approver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approver {
    /// Role-based approver
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// User-based approver
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Auto-approve configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApproveConfig {
    /// Condition expression
    pub condition: String,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    /// Validator type
    #[serde(rename = "type")]
    pub validator_type: ValidatorType,

    /// Function name (for function validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Arguments (for function validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,

    /// Model (for LLM validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Prompt (for LLM validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Command (for script validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Timeout (for script validators)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// Validator types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidatorType {
    /// Function-based validation
    Function,
    /// LLM-based validation
    Llm,
    /// Script-based validation
    Script,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Validation type
    #[serde(rename = "type")]
    pub rule_type: ValidatorType,

    /// Script name (for function type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,

    /// Prompt (for LLM type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// Next step configuration - can be a string or conditional list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NextStep {
    /// Simple next step name
    Simple(String),
    /// Conditional next steps
    Conditional(Vec<ConditionalNext>),
}

/// Conditional next step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalNext {
    /// Condition expression (e.g., "state.score > 0.8")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Target step name
    pub target: String,
}

/// Parallel branch definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBranch {
    /// Branch name
    pub name: String,

    /// Steps in this branch
    pub steps: Vec<BranchStep>,
}

/// Step within a parallel branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchStep {
    /// Agent to execute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Join configuration for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinConfig {
    /// Join strategy
    pub strategy: JoinStrategy,

    /// Timeout for waiting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// Join strategies for parallel execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JoinStrategy {
    /// Wait for all branches
    All,
    /// Wait for any branch
    Any,
    /// Wait for majority of branches
    Majority,
}

impl Default for JoinStrategy {
    fn default() -> Self {
        Self::All
    }
}

/// Interrupt configuration for human input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptConfig {
    /// Interrupt type
    #[serde(rename = "type")]
    pub interrupt_type: InterruptType,

    /// Prompt to display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Input schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<StateSchema>,
}

/// Interrupt types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InterruptType {
    /// Request input from user
    Input,
    /// Request confirmation
    Confirm,
}

/// Terminal status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TerminalStatus {
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow cancelled
    Cancelled,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// Maximum retry attempts
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Backoff strategy
    #[serde(default)]
    pub backoff: BackoffStrategy,

    /// Initial delay
    #[serde(default = "default_initial_delay")]
    pub initial_delay: String,

    /// Maximum delay
    #[serde(default = "default_max_delay")]
    pub max_delay: String,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_initial_delay() -> String {
    "1s".to_string()
}

fn default_max_delay() -> String {
    "30s".to_string()
}

/// Backoff strategies for retries
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linearly increasing delay
    Linear,
    /// Exponentially increasing delay
    #[default]
    Exponential,
}

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable checkpointing
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Backend type
    #[serde(default)]
    pub backend: CheckpointBackend,

    /// Path for file backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// URL for redis/postgres backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Checkpoint frequency
    #[serde(default)]
    pub frequency: CheckpointFrequency,

    /// Number of checkpoints to keep in history
    #[serde(default = "default_history")]
    pub history: u32,
}

fn default_true() -> bool {
    true
}

fn default_history() -> u32 {
    10
}

/// Checkpoint backends
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckpointBackend {
    /// File-based storage
    #[default]
    File,
    /// Redis storage
    Redis,
    /// PostgreSQL storage
    Postgres,
}

/// Checkpoint frequency
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckpointFrequency {
    /// Checkpoint after each step
    #[default]
    Step,
    /// Checkpoint on state changes only
    Change,
    /// Checkpoint at intervals
    Interval,
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryConfig {
    /// Auto-resume from last checkpoint on failure
    #[serde(default = "default_true")]
    pub auto_resume: bool,

    /// Skip completed steps on resume
    #[serde(default = "default_true")]
    pub skip_completed: bool,
}

// ============================================================================
// Workflow Execution State
// ============================================================================

/// Workflow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow run ID
    pub run_id: String,

    /// Workflow name
    pub workflow_name: String,

    /// Current step name
    pub current_step: String,

    /// Execution status
    pub status: WorkflowStatus,

    /// State data
    pub data: serde_json::Value,

    /// Completed steps
    pub completed_steps: Vec<String>,

    /// Step results
    pub step_results: HashMap<String, StepResult>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Error information (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkflowError>,
}

/// Workflow execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    /// Workflow is pending
    Pending,
    /// Workflow is running
    Running,
    /// Waiting for approval
    WaitingApproval,
    /// Waiting for input
    WaitingInput,
    /// Workflow completed
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow cancelled
    Cancelled,
}

/// Result of a step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name
    pub step_name: String,

    /// Execution status
    pub status: StepStatus,

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

    /// Error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Step execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Step is pending
    Pending,
    /// Step is running
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

/// Workflow error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error type
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Step where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,

    /// Stack trace or additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// ============================================================================
// Flat Configuration Format (non-K8s style)
// ============================================================================

/// Flat workflow configuration (non-Kubernetes style)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatWorkflowConfig {
    /// Workflow name
    pub name: String,

    /// Entry point step name
    pub entrypoint: String,

    /// Workflow steps
    pub steps: Vec<WorkflowStep>,

    /// State schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<StateSchema>,

    /// State reducers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub reducers: HashMap<String, StateReducer>,

    /// Global error handler
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handler: Option<String>,

    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Checkpointing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpointing: Option<CheckpointConfig>,
}

/// Input format that accepts both flat and Kubernetes-style configs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkflowConfigInput {
    /// Kubernetes-style configuration
    Kubernetes(Workflow),
    /// Flat configuration
    Flat(FlatWorkflowConfig),
}

impl From<WorkflowConfigInput> for Workflow {
    fn from(input: WorkflowConfigInput) -> Self {
        match input {
            WorkflowConfigInput::Kubernetes(w) => w,
            WorkflowConfigInput::Flat(flat) => Workflow {
                api_version: default_api_version(),
                kind: default_workflow_kind(),
                metadata: WorkflowMetadata {
                    name: flat.name,
                    namespace: None,
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                },
                spec: WorkflowSpec {
                    state: flat.state,
                    entrypoint: flat.entrypoint,
                    steps: flat.steps,
                    reducers: flat.reducers,
                    error_handler: flat.error_handler,
                    retry: flat.retry,
                    checkpointing: flat.checkpointing,
                    recovery: None,
                    fleet: None,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kubernetes_workflow() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: test-workflow
  labels:
    category: test
spec:
  entrypoint: start
  steps:
    - name: start
      type: agent
      agent: test-agent
      next: end
    - name: end
      type: terminal
      status: completed
"#;

        let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(workflow.metadata.name, "test-workflow");
        assert_eq!(workflow.spec.entrypoint, "start");
        assert_eq!(workflow.spec.steps.len(), 2);
    }

    #[test]
    fn test_parse_flat_workflow() {
        let yaml = r#"
name: simple-workflow
entrypoint: step1
steps:
  - name: step1
    type: agent
    agent: my-agent
    next: step2
  - name: step2
    type: terminal
    status: completed
"#;

        let input: WorkflowConfigInput = serde_yaml::from_str(yaml).unwrap();
        let workflow: Workflow = input.into();
        assert_eq!(workflow.metadata.name, "simple-workflow");
        assert_eq!(workflow.spec.steps.len(), 2);
    }

    #[test]
    fn test_conditional_routing() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: conditional-workflow
spec:
  entrypoint: check
  steps:
    - name: check
      type: agent
      agent: checker
      next:
        - condition: "state.score > 0.8"
          target: high
        - condition: "state.score > 0.5"
          target: medium
        - target: low
    - name: high
      type: terminal
      status: completed
    - name: medium
      type: terminal
      status: completed
    - name: low
      type: terminal
      status: completed
"#;

        let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
        let check_step = &workflow.spec.steps[0];
        match &check_step.next {
            Some(NextStep::Conditional(conds)) => {
                assert_eq!(conds.len(), 3);
                assert_eq!(conds[0].condition.as_ref().unwrap(), "state.score > 0.8");
                assert_eq!(conds[0].target, "high");
                assert!(conds[2].condition.is_none()); // Default case
            }
            _ => panic!("Expected conditional next"),
        }
    }

    #[test]
    fn test_parallel_execution() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: parallel-workflow
spec:
  entrypoint: analyze
  steps:
    - name: analyze
      type: parallel
      branches:
        - name: logs
          steps:
            - agent: log-analyzer
        - name: metrics
          steps:
            - agent: metric-analyzer
      join:
        strategy: all
        timeout: 10m
      next: aggregate
    - name: aggregate
      type: terminal
      status: completed
"#;

        let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
        let parallel_step = &workflow.spec.steps[0];
        assert_eq!(parallel_step.step_type, StepType::Parallel);
        let branches = parallel_step.branches.as_ref().unwrap();
        assert_eq!(branches.len(), 2);
        let join = parallel_step.join.as_ref().unwrap();
        assert_eq!(join.strategy, JoinStrategy::All);
    }

    #[test]
    fn test_approval_step() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: approval-workflow
spec:
  entrypoint: deploy-approval
  steps:
    - name: deploy-approval
      type: approval
      config:
        approvers:
          - role: sre-team
          - user: admin@example.com
        timeout: 30m
        requiredApprovals: 2
        autoApprove:
          condition: "state.environment == 'dev'"
      next:
        - condition: approved
          target: deploy
        - condition: rejected
          target: notify
        - condition: timeout
          target: escalate
    - name: deploy
      type: terminal
      status: completed
    - name: notify
      type: terminal
      status: completed
    - name: escalate
      type: terminal
      status: completed
"#;

        let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
        let approval_step = &workflow.spec.steps[0];
        assert_eq!(approval_step.step_type, StepType::Approval);
        let config = approval_step.config.as_ref().unwrap();
        assert_eq!(config.approvers.len(), 2);
        assert_eq!(config.required_approvals, Some(2));
    }

    #[test]
    fn test_workflow_state_serialization() {
        let state = WorkflowState {
            run_id: "run-123".to_string(),
            workflow_name: "test".to_string(),
            current_step: "step1".to_string(),
            status: WorkflowStatus::Running,
            data: serde_json::json!({"key": "value"}),
            completed_steps: vec!["step0".to_string()],
            step_results: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            error: None,
        };

        let json = serde_json::to_string(&state).unwrap();
        let parsed: WorkflowState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.run_id, "run-123");
        assert_eq!(parsed.status, WorkflowStatus::Running);
    }
}
