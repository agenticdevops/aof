//! ReAct (Reason + Act) loop engine for OpenAgentiX agents.
//!
//! The ReAct loop drives the plan-act-observe-reflect cycle:
//! 1. **Plan** — LLM analyzes the current situation
//! 2. **Act** — LLM calls a tool or produces a final answer
//! 3. **Observe** — Tool result (or error) is fed back to the LLM
//! 4. **Reflect** — LLM decides whether to continue or finish
//!
//! The loop terminates when:
//! - The LLM produces a response with no tool calls (final answer), or
//! - `max_iterations` is reached, or
//! - The configured `timeout` expires.

use std::sync::Arc;
use std::time::Duration;

use agentix_core::{AgentDefinition, AgentixError, ToolEntry};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single tool call requested by the LLM during one ReAct iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAction {
    /// Name of the tool requested.
    pub tool_name: String,
    /// JSON arguments provided by the LLM.
    pub input: serde_json::Value,
}

/// One complete plan-act-observe-reflect iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    /// LLM reasoning / planning text for this iteration.
    pub plan: String,
    /// Tool call made (if any) during the Act phase.
    pub action: Option<ToolAction>,
    /// Observation text: tool result or error fed back to the LLM.
    pub observation: String,
    /// Reflection text (currently empty; reserved for future use).
    pub reflection: String,
}

/// Events emitted by the ReAct loop during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReActEvent {
    /// Emitted after each completed iteration.
    Step(ReActStep),
    /// Emitted once when the loop finishes successfully.
    Complete(RunResult),
    /// Emitted when the loop encounters a fatal error.
    Error(String),
}

/// Final output of a completed ReAct run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// The agent's final answer text.
    pub output: String,
    /// Number of iterations executed.
    pub iterations: u32,
    /// All tool calls made across all iterations.
    pub tool_calls: Vec<ToolAction>,
    /// Whether the loop stopped because `max_iterations` was reached.
    pub reached_max_iterations: bool,
}

/// Configuration derived from an `AgentDefinition` for a single run.
#[derive(Debug, Clone)]
pub struct ReActConfig {
    /// Maximum number of plan-act-observe-reflect cycles.
    pub max_iterations: u32,
    /// Wall-clock timeout for the entire run.
    pub timeout: Duration,
}

impl ReActConfig {
    /// Build a `ReActConfig` from an assembled `AgentDefinition`.
    pub fn from_definition(def: &AgentDefinition) -> Self {
        Self {
            max_iterations: def.max_iterations,
            timeout: Duration::from_secs(def.timeout_secs),
        }
    }
}

// ---------------------------------------------------------------------------
// ToolExecutor trait (decoupled from agentix-core's ToolExecutor for ReAct)
// ---------------------------------------------------------------------------

/// Executes a single tool call on behalf of the ReAct loop.
///
/// Implementors receive the `ToolEntry` (metadata) and the JSON `input`
/// from the LLM. They must return either the output string (success) or
/// an error message (failure). Failures are fed back as observations —
/// the loop never hard-fails on a tool error.
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool.
    ///
    /// # Arguments
    /// * `tool` - The `ToolEntry` definition from `AgentDefinition.tools`.
    /// * `input` - JSON arguments provided by the LLM.
    ///
    /// # Returns
    /// * `Ok(String)` — observation text to feed back to the LLM.
    /// * `Err(String)` — error text; will be formatted as an observation.
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String>;
}

// ---------------------------------------------------------------------------
// ReActEngine
// ---------------------------------------------------------------------------

/// The ReAct loop engine.
///
/// Takes an assembled `AgentDefinition` and user input, then drives
/// plan-act-observe-reflect cycles until the agent finishes, times out,
/// or exhausts `max_iterations`.
pub struct ReActEngine {
    /// LLM backend used for generating plans and tool calls.
    model: Arc<dyn agentix_core::Model + Send + Sync>,
    /// Tool executor used for dispatching tool calls.
    tool_executor: Arc<dyn ToolExecutor>,
    /// Runtime configuration (iterations, timeout).
    config: ReActConfig,
    /// Optional broadcast channel for streaming `ReActEvent`s.
    event_tx: Option<broadcast::Sender<ReActEvent>>,
}

impl ReActEngine {
    /// Create a new `ReActEngine`.
    pub fn new(
        model: Arc<dyn agentix_core::Model + Send + Sync>,
        tool_executor: Arc<dyn ToolExecutor>,
        config: ReActConfig,
    ) -> Self {
        Self {
            model,
            tool_executor,
            config,
            event_tx: None,
        }
    }

    /// Attach a broadcast channel for streaming `ReActEvent`s.
    pub fn with_event_stream(mut self, tx: broadcast::Sender<ReActEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Run an agent against a user input.
    ///
    /// # Arguments
    /// * `definition` - Fully assembled `AgentDefinition` (SOUL + RULES + skills + tools).
    /// * `input` - The user's request or task.
    pub async fn run(
        &self,
        definition: &AgentDefinition,
        input: &str,
    ) -> Result<RunResult, AgentixError> {
        // TODO: implement in GREEN phase
        let _ = (definition, input);
        Err(AgentixError::runtime("ReActEngine::run not yet implemented"))
    }

    /// Emit an event to the broadcast channel, silently dropping if no receivers.
    fn emit(&self, event: ReActEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}
