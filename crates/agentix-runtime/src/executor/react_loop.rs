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
    SecurityConfig, SsrfGuard, SecretRedactor,
};
use agentix_core::model::MessageRole;
use agentix_core::vector_memory::VectorMemoryBackend;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::audit_store::{AuditStore, AuditEntry, AuditEventType, AuditOutcome};
use crate::memory::{format_memory_context, hash_embedding, open_agent_memory};
use crate::telemetry::TraceCollector;
use agentix_core::telemetry::{LogLevel, SpanKind};

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
    /// Emitted when the agent is waiting for human approval (Phase 20).
    ApprovalWaiting {
        /// Unique ID for this approval request.
        request_id: String,
        /// Name of the tool that requires approval.
        tool_name: String,
        /// Human-readable description.
        description: String,
    },
    /// Emitted when an approval decision is received (Phase 20).
    ApprovalDecided {
        /// Unique ID for this approval request.
        request_id: String,
        /// Whether the action was approved.
        approved: bool,
    },
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
    /// Total input tokens consumed across all LLM calls in this run (COST-01).
    #[serde(default)]
    pub total_input_tokens: u64,
    /// Total output tokens produced across all LLM calls in this run (COST-01).
    #[serde(default)]
    pub total_output_tokens: u64,
    /// Estimated total cost for this run in USD (COST-02).
    #[serde(default)]
    pub total_cost_usd: f64,
    /// Reason the run was stopped by a budget limit, if applicable (COST-04/COST-05).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stopped_reason: Option<agentix_core::BudgetStopReason>,
}

/// Configuration derived from an `AgentDefinition` for a single run.
#[derive(Debug, Clone)]
pub struct ReActConfig {
    /// Maximum number of plan-act-observe-reflect cycles.
    pub max_iterations: u32,
    /// Wall-clock timeout for the entire run.
    pub timeout: Duration,
    /// Per-run token limit (input + output combined). Run stops when exceeded (COST-05).
    pub max_tokens_per_run: Option<u64>,
    /// SSRF guard for HTTP tool calls (Phase 19).
    pub ssrf_guard: Option<Arc<SsrfGuard>>,
    /// Secret redactor for sanitizing tool outputs in traces/logs (Phase 19).
    pub secret_redactor: Option<Arc<SecretRedactor>>,
    /// Audit store for logging every tool call and LLM call (Phase 19).
    pub audit_store: Option<Arc<AuditStore>>,
    /// Agent name for audit entries (Phase 19).
    pub agent_name: String,
    /// Approval policy for human-in-the-loop (Phase 20).
    pub approval_policy: Option<agentix_core::ApprovalPolicy>,
    /// Approval store for persisting approval requests (Phase 20).
    pub approval_store: Option<Arc<crate::approval_store::ApprovalStore>>,
    /// Run ID — needed to create approval requests scoped to the run (Phase 20).
    pub run_id: Option<String>,
}

impl ReActConfig {
    /// Build a `ReActConfig` from an assembled `AgentDefinition`.
    pub fn from_definition(def: &AgentDefinition) -> Self {
        Self {
            max_iterations: def.max_iterations,
            timeout: Duration::from_secs(def.timeout_secs),
            max_tokens_per_run: def.budget.as_ref().and_then(|b| b.max_tokens_per_run),
            ssrf_guard: None,
            secret_redactor: None,
            audit_store: None,
            agent_name: def.name.clone(),
            approval_policy: def.approval.clone(),
            approval_store: None,
            run_id: None,
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
    /// Optional trace collector for telemetry spans (Phase 18).
    trace_collector: Option<TraceCollector>,
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
            trace_collector: None,
        }
    }

    /// Attach a broadcast channel for streaming `ReActEvent`s.
    pub fn with_event_stream(mut self, tx: broadcast::Sender<ReActEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Attach a trace collector for telemetry span recording (Phase 18).
    ///
    /// When set, the engine records spans for each execution phase:
    /// Run, Iteration, LlmCall, ToolCall, Research, and MemoryRecall.
    /// Zero-cost when not set — all instrumentation is guarded by `if let Some`.
    pub fn with_trace_collector(mut self, collector: TraceCollector) -> Self {
        self.trace_collector = Some(collector);
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
        // Telemetry: start root Run span (Phase 18)
        // ---------------------------------------------------------------------------
        let mut run_span = self.trace_collector.as_ref().map(|c| {
            let span = c
                .start_span("agent_run", SpanKind::Run)
                .with_attribute("agent", &definition.name);
            span
        });

        if let Some(ref collector) = self.trace_collector {
            let mut fields = HashMap::new();
            fields.insert(
                "input_length".to_string(),
                serde_json::json!(input.len()),
            );
            collector.log(LogLevel::Info, "Agent run started", fields);
        }

        // ---------------------------------------------------------------------------
        // Memory recall (CORE-04): retrieve top_k past contexts before the ReAct loop
        // ---------------------------------------------------------------------------
        let (memory_prefix, memory_backend, query_embedding) =
            if definition.vector_memory.enabled {
                // Telemetry: MemoryRecall span
                let mut mem_span = self.trace_collector.as_ref().map(|c| {
                    c.start_span("memory_recall", SpanKind::MemoryRecall)
                });

                let db_path = definition.vector_memory.db_path.as_deref();
                let result = match open_agent_memory(&definition.name, db_path).await {
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
                        if let Some(ref mut span) = mem_span {
                            span.complete_with_error(&e.to_string());
                        }
                        (String::new(), None, Vec::new())
                    }
                };

                if let Some(ref mut span) = mem_span {
                    if span.end_time.is_none() {
                        span.complete();
                    }
                }
                if let (Some(span), Some(ref collector)) = (mem_span, &self.trace_collector) {
                    collector.record_span(span);
                }

                result
            } else {
                (String::new(), None, Vec::new())
            };

        // ---------------------------------------------------------------------------
        // Research phase (CORE-06): proactive fact-gathering before main ReAct loop
        // ---------------------------------------------------------------------------
        let research_context = if definition.research_phase.enabled {
            // Telemetry: Research span
            let mut research_span = self.trace_collector.as_ref().map(|c| {
                c.start_span("research_phase", SpanKind::Research)
            });

            let ctx = self
                .run_research_phase(
                    input,
                    &definition.resolved_system_prompt(),
                    definition.research_phase.max_iterations,
                )
                .await
                .unwrap_or_default();

            if let Some(ref mut span) = research_span {
                span.complete();
            }
            if let (Some(span), Some(ref collector)) = (research_span, &self.trace_collector) {
                collector.record_span(span);
            }

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
        // Token accumulators for cost tracking (COST-01)
        let mut total_input_tokens: u64 = 0;
        let mut total_output_tokens: u64 = 0;

        // Wrap the whole run in a timeout.
        let result = tokio::time::timeout(self.config.timeout, async {
            loop {
                if iteration >= self.config.max_iterations {
                    // Telemetry: finalize run span on max iterations
                    if let (Some(ref mut span), Some(ref collector)) =
                        (&mut run_span, &self.trace_collector)
                    {
                        let span = span
                            .clone()
                            .with_attribute("total_iterations", &iteration.to_string())
                            .with_attribute("total_tool_calls", &all_tool_calls.len().to_string())
                            .with_attribute("total_input_tokens", &total_input_tokens.to_string())
                            .with_attribute("total_output_tokens", &total_output_tokens.to_string())
                            .with_attribute("final_status", "max_iterations");
                        let mut span = span;
                        span.complete();
                        collector.record_span(span);
                    }

                    // Max iterations reached without a final answer.
                    let run_result = RunResult {
                        output: String::new(),
                        iterations: iteration,
                        tool_calls: all_tool_calls.clone(),
                        reached_max_iterations: true,
                        total_input_tokens,
                        total_output_tokens,
                        total_cost_usd: 0.0,
                        stopped_reason: None,
                    };
                    self.emit(ReActEvent::Complete(run_result.clone()));
                    return Ok(run_result);
                }

                // Telemetry: start Iteration span
                let mut iter_span = self.trace_collector.as_ref().map(|c| {
                    c.start_span(
                        &format!("iteration_{}", iteration + 1),
                        SpanKind::Iteration,
                    )
                    .with_attribute("iteration_number", &(iteration + 1).to_string())
                });

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

                // Telemetry: start LlmCall span
                let mut llm_span = self.trace_collector.as_ref().map(|c| {
                    c.start_span("llm_call", SpanKind::LlmCall)
                });

                let response = self.model.generate(&request).await?;

                // Telemetry: complete LlmCall span with attributes
                if let Some(ref mut span) = llm_span {
                    *span = span
                        .clone()
                        .with_attribute("model", definition.model_preferred.as_deref().unwrap_or("unknown"))
                        .with_attribute("input_tokens", &response.usage.input_tokens.to_string())
                        .with_attribute("output_tokens", &response.usage.output_tokens.to_string());
                    span.complete();
                }
                if let (Some(span), Some(ref collector)) = (llm_span, &self.trace_collector) {
                    collector.record_span(span);
                }

                // Phase 19: Audit logging for LLM calls (non-critical)
                if let Some(ref audit) = self.config.audit_store {
                    let mut details = HashMap::new();
                    details.insert("model".to_string(), serde_json::json!(definition.model_preferred.as_deref().unwrap_or("unknown")));
                    details.insert("input_tokens".to_string(), serde_json::json!(response.usage.input_tokens));
                    details.insert("output_tokens".to_string(), serde_json::json!(response.usage.output_tokens));
                    let audit_entry = AuditEntry {
                        id: None,
                        timestamp: chrono::Utc::now(),
                        event_type: AuditEventType::LlmCall,
                        agent_name: self.config.agent_name.clone(),
                        run_id: None,
                        actor: format!("agent:{}", self.config.agent_name),
                        action: format!("LLM call: {}", definition.model_preferred.as_deref().unwrap_or("unknown")),
                        outcome: AuditOutcome::Success,
                        details,
                        trace_id: None,
                    };
                    if let Err(e) = audit.log_event(audit_entry) {
                        tracing::warn!("Failed to log audit LlmCall: {}", e);
                    }
                }

                // Accumulate token usage (COST-01)
                total_input_tokens += response.usage.input_tokens as u64;
                total_output_tokens += response.usage.output_tokens as u64;

                // Check per-run token budget (COST-05)
                if let Some(max_tokens) = self.config.max_tokens_per_run {
                    let used = total_input_tokens + total_output_tokens;
                    if used > max_tokens {
                        // Telemetry: complete iteration span
                        if let Some(ref mut span) = iter_span {
                            span.complete();
                        }
                        if let (Some(span), Some(ref collector)) =
                            (iter_span, &self.trace_collector)
                        {
                            collector.record_span(span);
                        }

                        // Telemetry: finalize run span on token limit
                        if let (Some(ref mut span), Some(ref collector)) =
                            (&mut run_span, &self.trace_collector)
                        {
                            let span = span
                                .clone()
                                .with_attribute("total_iterations", &iteration.to_string())
                                .with_attribute("total_tool_calls", &all_tool_calls.len().to_string())
                                .with_attribute("total_input_tokens", &total_input_tokens.to_string())
                                .with_attribute("total_output_tokens", &total_output_tokens.to_string())
                                .with_attribute("final_status", "token_limit_exceeded");
                            let mut span = span;
                            span.complete_with_error("token limit exceeded");
                            collector.record_span(span);
                        }

                        let run_result = RunResult {
                            output: format!(
                                "Run stopped: token limit exceeded ({used} tokens used, limit is {max_tokens})."
                            ),
                            iterations: iteration,
                            tool_calls: all_tool_calls.clone(),
                            reached_max_iterations: false,
                            total_input_tokens,
                            total_output_tokens,
                            total_cost_usd: 0.0,
                            stopped_reason: Some(agentix_core::BudgetStopReason::TokenLimitExceeded {
                                limit: max_tokens,
                                used,
                            }),
                        };
                        self.emit(ReActEvent::Complete(run_result.clone()));
                        return Ok(run_result);
                    }
                }

                if response.tool_calls.is_empty() {
                    // No tool calls → final answer.
                    // Count: if no tool iterations happened yet (pure Q&A), count 1.
                    // Otherwise keep the existing iteration count (tool iterations only).
                    let final_iterations = if iteration == 0 { 1 } else { iteration };

                    // Telemetry: complete iteration span
                    if let Some(ref mut span) = iter_span {
                        *span = span
                            .clone()
                            .with_attribute("tool_count", "0");
                        span.complete();
                    }
                    if let (Some(span), Some(ref collector)) =
                        (iter_span, &self.trace_collector)
                    {
                        collector.record_span(span);
                    }

                    // Telemetry: finalize run span on completion
                    if let (Some(ref mut span), Some(ref collector)) =
                        (&mut run_span, &self.trace_collector)
                    {
                        let span = span
                            .clone()
                            .with_attribute("total_iterations", &final_iterations.to_string())
                            .with_attribute("total_tool_calls", &all_tool_calls.len().to_string())
                            .with_attribute("total_input_tokens", &total_input_tokens.to_string())
                            .with_attribute("total_output_tokens", &total_output_tokens.to_string())
                            .with_attribute("final_status", "completed");
                        let mut span = span;
                        span.complete();
                        collector.record_span(span);
                    }

                    let run_result = RunResult {
                        output: response.content.clone(),
                        iterations: final_iterations,
                        tool_calls: all_tool_calls.clone(),
                        reached_max_iterations: false,
                        total_input_tokens,
                        total_output_tokens,
                        total_cost_usd: 0.0, // computed by AgentManager using pricing table
                        stopped_reason: None,
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
                let mut tool_count: u32 = 0;

                for tool_call in &response.tool_calls {
                    // Telemetry: start ToolCall span
                    let mut tool_span = self.trace_collector.as_ref().map(|c| {
                        c.start_span(
                            &format!("tool_call_{}", tool_call.name),
                            SpanKind::ToolCall,
                        )
                        .with_attribute("tool_name", &tool_call.name)
                    });

                    // Phase 20: Approval gate — check if this tool call needs approval
                    let run_id_ref = self.config.run_id.as_deref();
                    if let Some(ref policy) = self.config.approval_policy {
                        let tool_desc = definition.tools.iter()
                            .find(|t| t.name == tool_call.name)
                            .map(|t| t.description.as_deref().unwrap_or(""))
                            .unwrap_or("");
                        if policy.requires_approval(&tool_call.name, &tool_call.arguments) {
                            let approval_observation = self.wait_for_approval(
                                &tool_call.name,
                                &tool_call.arguments,
                                tool_desc,
                                run_id_ref,
                                policy,
                            ).await;

                            // If approval was denied or expired, use that as the observation
                            if let Some(denied_obs) = approval_observation {
                                // Log denial to audit store
                                if let Some(ref audit) = self.config.audit_store {
                                    let mut details = HashMap::new();
                                    details.insert("tool_name".to_string(), serde_json::json!(tool_call.name));
                                    let audit_entry = AuditEntry {
                                        id: None,
                                        timestamp: chrono::Utc::now(),
                                        event_type: AuditEventType::ApprovalDecision,
                                        agent_name: self.config.agent_name.clone(),
                                        run_id: run_id_ref.map(|s| s.to_string()),
                                        actor: format!("agent:{}", self.config.agent_name),
                                        action: format!("Approval denied/expired for tool: {}", tool_call.name),
                                        outcome: AuditOutcome::Denied(denied_obs.clone()),
                                        details,
                                        trace_id: None,
                                    };
                                    if let Err(e) = audit.log_event(audit_entry) {
                                        tracing::warn!("Failed to log audit ApprovalDecision: {}", e);
                                    }
                                }

                                // Complete telemetry span
                                if let Some(ref mut span) = tool_span {
                                    *span = span.clone().with_attribute("status", "denied");
                                    span.complete_with_error(&denied_obs);
                                }
                                if let (Some(span), Some(ref collector)) = (tool_span, &self.trace_collector) {
                                    collector.record_span(span);
                                }

                                let action = ToolAction {
                                    tool_name: tool_call.name.clone(),
                                    input: tool_call.arguments.clone(),
                                };
                                all_tool_calls.push(action.clone());
                                step_tool_actions.push(action);
                                observations.push(denied_obs);
                                tool_count += 1;
                                continue;
                            }
                            // If approved, fall through to execute the tool
                        }
                    }

                    let observation = self.dispatch_tool(definition, tool_call, run_id_ref).await;

                    // Telemetry: complete ToolCall span
                    let tool_status = if observation.starts_with("Error:") || observation.starts_with("Tool '") && observation.contains("failed:") {
                        "error"
                    } else {
                        "ok"
                    };
                    if let Some(ref mut span) = tool_span {
                        *span = span.clone().with_attribute("status", tool_status);
                        if tool_status == "error" {
                            span.complete_with_error(&observation);
                        } else {
                            span.complete();
                        }
                    }
                    if let (Some(span), Some(ref collector)) = (tool_span, &self.trace_collector) {
                        collector.record_span(span);
                    }

                    let action = ToolAction {
                        tool_name: tool_call.name.clone(),
                        input: tool_call.arguments.clone(),
                    };
                    all_tool_calls.push(action.clone());
                    step_tool_actions.push(action);
                    observations.push(observation);
                    tool_count += 1;
                }

                // Telemetry: complete Iteration span
                if let Some(ref mut span) = iter_span {
                    *span = span
                        .clone()
                        .with_attribute("tool_count", &tool_count.to_string());
                    span.complete();
                }
                if let (Some(span), Some(ref collector)) = (iter_span, &self.trace_collector) {
                    collector.record_span(span);
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
            Err(_elapsed) => {
                // Telemetry: finalize run span on timeout
                if let (Some(ref mut span), Some(ref collector)) =
                    (&mut run_span, &self.trace_collector)
                {
                    let span = span
                        .clone()
                        .with_attribute("total_iterations", &iteration.to_string())
                        .with_attribute("total_tool_calls", &all_tool_calls.len().to_string())
                        .with_attribute("final_status", "timeout");
                    let mut span = span;
                    span.complete_with_error("timeout");
                    collector.record_span(span);
                }

                Err(AgentixError::Timeout(format!(
                    "ReAct loop timed out after {:?}",
                    self.config.timeout
                )))
            }
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
    ///
    /// Security integrations (Phase 19):
    /// - SSRF guard checks URL in tool input before execution
    /// - Audit logging for every tool call (success or failure)
    /// - Secret redaction on tool output before returning
    async fn dispatch_tool(
        &self,
        definition: &AgentDefinition,
        tool_call: &ToolCall,
        run_id: Option<&str>,
    ) -> String {
        // Find the ToolEntry by name.
        let tool_entry = definition.tools.iter().find(|t| t.name == tool_call.name);

        match tool_entry {
            None => format!(
                "Error: Tool '{}' not found in agent definition",
                tool_call.name
            ),
            Some(entry) => {
                // Phase 19: SSRF guard — check URL in tool input before execution
                if let Some(ref guard) = self.config.ssrf_guard {
                    if let Some(url) = tool_call.arguments.get("url").and_then(|v| v.as_str()) {
                        if let Err(violation) = guard.check_url(url) {
                            // Log security violation to audit store (non-critical)
                            if let Some(ref audit) = self.config.audit_store {
                                let mut details = HashMap::new();
                                details.insert("tool_name".to_string(), serde_json::json!(tool_call.name));
                                details.insert("blocked_url".to_string(), serde_json::json!(url));
                                let audit_entry = AuditEntry {
                                    id: None,
                                    timestamp: chrono::Utc::now(),
                                    event_type: AuditEventType::SecurityViolation,
                                    agent_name: self.config.agent_name.clone(),
                                    run_id: run_id.map(|s| s.to_string()),
                                    actor: format!("agent:{}", self.config.agent_name),
                                    action: format!("SSRF blocked: {}", violation),
                                    outcome: AuditOutcome::Blocked(violation.to_string()),
                                    details,
                                    trace_id: None,
                                };
                                if let Err(e) = audit.log_event(audit_entry) {
                                    tracing::warn!("Failed to log audit SSRF violation: {}", e);
                                }
                            }
                            return format!("SSRF protection: {}", violation);
                        }
                    }
                }

                let start_time = std::time::Instant::now();
                let result = self
                    .tool_executor
                    .execute(entry, tool_call.arguments.clone())
                    .await;
                let duration_ms = start_time.elapsed().as_millis() as u64;

                // Phase 19: Audit logging for tool calls (non-critical)
                if let Some(ref audit) = self.config.audit_store {
                    let mut details = HashMap::new();
                    details.insert("tool_name".to_string(), serde_json::json!(tool_call.name));
                    details.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
                    let audit_entry = AuditEntry {
                        id: None,
                        timestamp: chrono::Utc::now(),
                        event_type: AuditEventType::ToolCall,
                        agent_name: self.config.agent_name.clone(),
                        run_id: run_id.map(|s| s.to_string()),
                        actor: format!("agent:{}", self.config.agent_name),
                        action: format!("Tool call: {}", tool_call.name),
                        outcome: match &result {
                            Ok(_) => AuditOutcome::Success,
                            Err(e) => AuditOutcome::Failure(e.to_string()),
                        },
                        details,
                        trace_id: None,
                    };
                    if let Err(e) = audit.log_event(audit_entry) {
                        tracing::warn!("Failed to log audit ToolCall: {}", e);
                    }
                }

                let output = match result {
                    Ok(output) => output,
                    Err(err) => format!("Tool '{}' failed: {}", tool_call.name, err),
                };

                // Phase 19: Secret redaction on tool output
                if let Some(ref redactor) = self.config.secret_redactor {
                    redactor.redact(&output)
                } else {
                    output
                }
            }
        }
    }

    /// Phase 20: Wait for human approval before executing a flagged tool call.
    ///
    /// Creates an approval request in the store (if available), emits an
    /// `ApprovalWaiting` event, and polls the store until the request is
    /// approved, denied, or expired.
    ///
    /// Returns:
    /// - `None` if approved (caller should proceed with tool execution)
    /// - `Some(observation)` if denied or expired (caller should use this as the observation)
    async fn wait_for_approval(
        &self,
        tool_name: &str,
        tool_input: &serde_json::Value,
        tool_desc: &str,
        run_id: Option<&str>,
        policy: &agentix_core::ApprovalPolicy,
    ) -> Option<String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let description = format!(
            "Agent '{}' wants to call tool '{}': {}",
            self.config.agent_name, tool_name, tool_desc
        );

        // Create the approval request (new API: run_id, agent_name, action_description, tool_name, tool_input, timeout_secs)
        let mut request = agentix_core::ApprovalRequest::new(
            run_id.unwrap_or("unknown").to_string(),
            self.config.agent_name.clone(),
            description.clone(),
            Some(tool_name.to_string()),
            Some(tool_input.clone()),
            policy.default_timeout_secs,
        );
        // Override the UUID with the pre-generated one for event correlation
        request.id = request_id.clone();

        // Persist to store (non-critical — if store is unavailable, auto-deny)
        if let Some(ref store) = self.config.approval_store {
            if let Err(e) = store.create_request(&request) {
                tracing::warn!("Failed to create approval request: {}", e);
                return Some(format!(
                    "Approval required for '{}' but approval store is unavailable: {}",
                    tool_name, e
                ));
            }
        }

        // Emit waiting event
        self.emit(ReActEvent::ApprovalWaiting {
            request_id: request_id.clone(),
            tool_name: tool_name.to_string(),
            description: description.clone(),
        });

        tracing::info!(
            agent = %self.config.agent_name,
            tool = %tool_name,
            request_id = %request_id,
            "Waiting for approval"
        );

        // Poll the store for a decision
        let poll_interval = Duration::from_secs(1);
        let timeout = Duration::from_secs(policy.default_timeout_secs as u64);
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            if tokio::time::Instant::now() >= deadline {
                // Expire the request
                if let Some(ref store) = self.config.approval_store {
                    let _ = store.expire_stale();
                }

                self.emit(ReActEvent::ApprovalDecided {
                    request_id: request_id.clone(),
                    approved: false,
                });

                return Some(format!(
                    "Approval request '{}' for tool '{}' expired after {}s without a decision.",
                    request_id, tool_name, policy.default_timeout_secs
                ));
            }

            // Check the store for a decision
            if let Some(ref store) = self.config.approval_store {
                match store.get_request(&request_id) {
                    Ok(Some(req)) => match &req.status {
                        agentix_core::ApprovalStatus::Approved { approver, .. } => {
                            self.emit(ReActEvent::ApprovalDecided {
                                request_id: request_id.clone(),
                                approved: true,
                            });

                            // Log approval to audit store
                            if let Some(ref audit) = self.config.audit_store {
                                let mut details = HashMap::new();
                                details.insert("tool_name".to_string(), serde_json::json!(tool_name));
                                details.insert("request_id".to_string(), serde_json::json!(request_id));
                                details.insert("approver".to_string(), serde_json::json!(approver));
                                let audit_entry = AuditEntry {
                                    id: None,
                                    timestamp: chrono::Utc::now(),
                                    event_type: AuditEventType::ApprovalDecision,
                                    agent_name: self.config.agent_name.clone(),
                                    run_id: run_id.map(|s| s.to_string()),
                                    actor: approver.clone(),
                                    action: format!("Approved tool call: {}", tool_name),
                                    outcome: AuditOutcome::Success,
                                    details,
                                    trace_id: None,
                                };
                                if let Err(e) = audit.log_event(audit_entry) {
                                    tracing::warn!("Failed to log audit ApprovalDecision: {}", e);
                                }
                            }

                            tracing::info!(
                                agent = %self.config.agent_name,
                                tool = %tool_name,
                                request_id = %request_id,
                                "Approval granted — proceeding with tool call"
                            );
                            return None; // Approved — proceed with execution
                        }
                        agentix_core::ApprovalStatus::Denied { reason, .. } => {
                            self.emit(ReActEvent::ApprovalDecided {
                                request_id: request_id.clone(),
                                approved: false,
                            });

                            let reason_str = reason.clone()
                                .unwrap_or_else(|| "no reason given".to_string());

                            return Some(format!(
                                "Approval denied for tool '{}': {}",
                                tool_name, reason_str
                            ));
                        }
                        agentix_core::ApprovalStatus::TimedOut { .. } => {
                            self.emit(ReActEvent::ApprovalDecided {
                                request_id: request_id.clone(),
                                approved: false,
                            });

                            return Some(format!(
                                "Approval request '{}' for tool '{}' expired.",
                                request_id, tool_name
                            ));
                        }
                        agentix_core::ApprovalStatus::Pending => {
                            // Still waiting — continue polling
                        }
                    },
                    Ok(None) => {
                        return Some(format!(
                            "Approval request '{}' disappeared from store.",
                            request_id
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to poll approval store: {}", e);
                    }
                }
            } else {
                // No store — auto-deny (shouldn't happen if properly wired)
                return Some(format!(
                    "Approval required for '{}' but no approval store configured.",
                    tool_name
                ));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Emit an event to the broadcast channel, silently dropping if no receivers.
    fn emit(&self, event: ReActEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}
