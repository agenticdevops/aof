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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use agentix_core::{
    AgentDefinition, AgentixError, ModelRequest, RequestMessage, ToolCall, ToolEntry,
};
use agentix_core::model::MessageRole;
use agentix_core::vector_memory::VectorMemoryBackend;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::memory::{format_memory_context, hash_embedding, open_agent_memory};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single tool call requested by the LLM during one ReAct iteration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolAction {
    /// Name of the tool requested.
    pub tool_name: String,
    /// JSON arguments provided by the LLM.
    pub input: serde_json::Value,
}

/// One complete plan-act-observe-reflect iteration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReActEvent {
    /// Emitted after each completed iteration.
    Step(ReActStep),
    /// Emitted once when the loop finishes successfully.
    Complete(RunResult),
    /// Emitted when the loop encounters a fatal error.
    Error(String),
}

/// Final output of a completed ReAct run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
// ReAct system prompt prefix
// ---------------------------------------------------------------------------

const REACT_INSTRUCTIONS: &str = "\n\n---\nYou are an AI agent operating in a ReAct (Reason + Act) loop.\n\nFor each step:\n1. Plan: Analyze the current situation and decide what to do next\n2. Act: Call a tool if needed, or provide your final answer\n3. Observe: Review the tool result\n4. Reflect: Decide if you need another step or if you have the answer\n\nWhen you have enough information, respond without calling any tools.\n---";

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
    ///
    /// The engine:
    /// 1. Assembles the system prompt via `definition.resolved_system_prompt()`.
    /// 2. Prepends ReAct instructions.
    /// 3. Loops up to `config.max_iterations` times, calling the LLM and dispatching tools.
    /// 4. Feeds tool results (or errors) back as observations — never hard-fails on tool error.
    /// 5. Terminates when the LLM produces no tool calls (final answer).
    pub async fn run(
        &self,
        definition: &AgentDefinition,
        input: &str,
    ) -> Result<RunResult, AgentixError> {
        // ---------------------------------------------------------------------------
        // Memory recall (CORE-04): retrieve top_k past contexts before the ReAct loop
        // ---------------------------------------------------------------------------
        let (memory_prefix, memory_backend, query_embedding) =
            if definition.vector_memory.enabled {
                let db_path = definition.vector_memory.db_path.as_deref();
                match open_agent_memory(&definition.name, db_path).await {
                    Ok(backend) => {
                        let embedding = hash_embedding(input);
                        let top_k = definition.vector_memory.top_k;
                        let matches = backend
                            .search_similar(&definition.name, &embedding, top_k)
                            .await
                            .unwrap_or_default();
                        let prefix = format_memory_context(&matches);
                        (prefix, Some(backend), embedding)
                    }
                    Err(e) => {
                        tracing::warn!(
                            agent = %definition.name,
                            error = %e,
                            "Failed to open vector memory — running without memory"
                        );
                        (String::new(), None, Vec::new())
                    }
                }
            } else {
                (String::new(), None, Vec::new())
            };

        // ---------------------------------------------------------------------------
        // Research phase (CORE-06): proactive fact-gathering before main ReAct loop
        // ---------------------------------------------------------------------------
        let research_context = if definition.research_phase.enabled {
            let ctx = self
                .run_research_phase(
                    input,
                    &definition.resolved_system_prompt(),
                    definition.research_phase.max_iterations,
                )
                .await
                .unwrap_or_default();
            if !ctx.is_empty() {
                format!("## Research Context\n\n{}\n\n---\n\n", ctx)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // ---------------------------------------------------------------------------
        // Assemble system prompt: memory prefix + research context + base + ReAct instructions
        // ---------------------------------------------------------------------------
        let system_prompt = format!(
            "{}{}{}{}",
            memory_prefix,
            research_context,
            definition.resolved_system_prompt(),
            REACT_INSTRUCTIONS
        );

        // Conversation history — starts with the user's message.
        let mut messages: Vec<RequestMessage> = vec![RequestMessage {
            role: MessageRole::User,
            content: input.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];

        let mut all_tool_calls: Vec<ToolAction> = vec![];
        let mut iteration: u32 = 0;

        // Wrap the whole run in a timeout.
        let result = tokio::time::timeout(self.config.timeout, async {
            loop {
                if iteration >= self.config.max_iterations {
                    // Max iterations reached without a final answer.
                    let run_result = RunResult {
                        output: String::new(),
                        iterations: iteration,
                        tool_calls: all_tool_calls.clone(),
                        reached_max_iterations: true,
                    };
                    self.emit(ReActEvent::Complete(run_result.clone()));
                    return Ok(run_result);
                }

                // [Plan + Act] Call the LLM.
                let request = ModelRequest {
                    messages: messages.clone(),
                    system: Some(system_prompt.clone()),
                    tools: vec![], // Tool dispatch is managed by the loop itself.
                    temperature: None,
                    max_tokens: None,
                    stream: false,
                    extra: HashMap::new(),
                };

                let response = self.model.generate(&request).await?;

                if response.tool_calls.is_empty() {
                    // No tool calls → final answer.
                    // Count: if no tool iterations happened yet (pure Q&A), count 1.
                    // Otherwise keep the existing iteration count (tool iterations only).
                    let final_iterations = if iteration == 0 { 1 } else { iteration };
                    let run_result = RunResult {
                        output: response.content.clone(),
                        iterations: final_iterations,
                        tool_calls: all_tool_calls.clone(),
                        reached_max_iterations: false,
                    };
                    self.emit(ReActEvent::Complete(run_result.clone()));

                    // Memory store (CORE-04): persist final answer for future recall.
                    // Non-critical — a store failure must not fail the run.
                    if let Some(backend) = &memory_backend {
                        if !query_embedding.is_empty() {
                            let mut meta = HashMap::new();
                            meta.insert("trigger_source".to_string(), "react".to_string());
                            let _ = backend
                                .store_embedding(
                                    &definition.name,
                                    "unknown", // run_id not available here; overridden by gateway
                                    &response.content,
                                    query_embedding.clone(),
                                    meta,
                                )
                                .await;
                        }
                    }

                    return Ok(run_result);
                }

                // [Act] Dispatch each tool call and collect observations.
                iteration += 1;
                let mut observations: Vec<String> = vec![];
                let mut step_tool_actions: Vec<ToolAction> = vec![];

                for tool_call in &response.tool_calls {
                    let observation = self.dispatch_tool(definition, tool_call).await;

                    let action = ToolAction {
                        tool_name: tool_call.name.clone(),
                        input: tool_call.arguments.clone(),
                    };
                    all_tool_calls.push(action.clone());
                    step_tool_actions.push(action);
                    observations.push(observation);
                }

                // [Observe] Format all observations for this iteration.
                let observation_text = observations.join("\n---\n");

                // Build the ReActStep for this iteration.
                let step = ReActStep {
                    plan: response.content.clone(),
                    action: step_tool_actions.into_iter().next(),
                    observation: observation_text.clone(),
                    reflection: String::new(),
                };

                self.emit(ReActEvent::Step(step));

                // [Reflect] Append the assistant message + observation to conversation.
                messages.push(RequestMessage {
                    role: MessageRole::Assistant,
                    content: response.content.clone(),
                    tool_calls: Some(response.tool_calls.clone()),
                    tool_call_id: None,
                });
                messages.push(RequestMessage {
                    role: MessageRole::User,
                    content: format!("Observation:\n{}", observation_text),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        })
        .await;

        match result {
            Ok(run_result) => run_result,
            Err(_elapsed) => Err(AgentixError::Timeout(format!(
                "ReAct loop timed out after {:?}",
                self.config.timeout
            ))),
        }
    }

    /// Run the research phase pre-loop (CORE-06).
    ///
    /// Fires a single LLM call asking the model to identify and summarise
    /// relevant facts for the query. Tool calls are not executed in this
    /// lightweight pass — the model summarises what it knows from context.
    ///
    /// Returns the research summary text, or an empty string on failure.
    async fn run_research_phase(
        &self,
        input: &str,
        system_prompt: &str,
        _max_iterations: usize,
    ) -> Result<String, AgentixError> {
        let research_prompt = format!(
            "You are in the RESEARCH PHASE. Your goal is to identify key facts relevant to the query.\n\
             Do NOT generate a final answer yet.\n\n\
             System context:\n{system_prompt}\n\n\
             Query: {input}\n\n\
             List 3-5 specific facts you already know that are relevant to this query, then summarise \
             them in a concise ## RESEARCH COMPLETE block.\n\
             Format:\n\
             ## RESEARCH COMPLETE\n\
             <bullet list of gathered facts>"
        );

        let request = ModelRequest {
            messages: vec![RequestMessage {
                role: MessageRole::User,
                content: research_prompt,
                tool_calls: None,
                tool_call_id: None,
            }],
            system: None,
            tools: vec![],
            temperature: Some(0.3),
            max_tokens: Some(512),
            stream: false,
            extra: HashMap::new(),
        };

        let response = self.model.generate(&request).await?;
        let content = response.content;

        // Extract the block after "## RESEARCH COMPLETE" if present.
        if let Some(pos) = content.find("## RESEARCH COMPLETE") {
            Ok(content[pos..].to_string())
        } else {
            Ok(content)
        }
    }

    /// Dispatch a single tool call, returning an observation string.
    ///
    /// Looks up the `ToolEntry` from `definition.tools` by name.
    /// If not found, returns an error observation rather than panicking.
    /// If the tool executor returns an error, formats it as an observation.
    async fn dispatch_tool(
        &self,
        definition: &AgentDefinition,
        tool_call: &ToolCall,
    ) -> String {
        // Find the ToolEntry by name.
        let tool_entry = definition.tools.iter().find(|t| t.name == tool_call.name);

        match tool_entry {
            None => format!(
                "Error: Tool '{}' not found in agent definition",
                tool_call.name
            ),
            Some(entry) => {
                match self
                    .tool_executor
                    .execute(entry, tool_call.arguments.clone())
                    .await
                {
                    Ok(output) => output,
                    Err(err) => format!("Tool '{}' failed: {}", tool_call.name, err),
                }
            }
        }
    }

    /// Emit an event to the broadcast channel, silently dropping if no receivers.
    fn emit(&self, event: ReActEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}
