//! AgentExecutor - Manages agent execution lifecycle
//!
//! The AgentExecutor handles the core execution loop:
//! 1. Call model with messages + tools
//! 2. If stop_reason == ToolUse, execute tools
//! 3. Add tool results to context
//! 4. Repeat until EndTurn or max iterations

use aof_core::{
    AgentConfig, AgentContext, AofError, AofResult, Memory, MessageRole, Model, ModelRequest,
    ModelToolDefinition, RequestMessage, StopReason, StreamChunk, ToolCall, ToolExecutor, ToolInput, ToolResult,
};
use aof_memory::SimpleMemory;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

/// Stream event types for real-time agent execution updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Text delta from LLM streaming
    TextDelta {
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<u64>,
    },
    /// Tool call started
    ToolCallStart {
        tool_name: String,
        tool_id: String,
        arguments: serde_json::Value,
    },
    /// Tool call completed
    ToolCallComplete {
        tool_name: String,
        tool_id: String,
        success: bool,
        execution_time_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Thinking/reasoning chunk (for models that support it)
    Thinking {
        content: String,
    },
    /// Iteration started
    IterationStart {
        iteration: usize,
        max_iterations: usize,
    },
    /// Iteration completed
    IterationComplete {
        iteration: usize,
        stop_reason: StopReason,
    },
    /// Agent execution completed
    Done {
        content: String,
        total_iterations: usize,
        execution_time_ms: u64,
        input_tokens: usize,
        output_tokens: usize,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// Error category for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ErrorCategory {
    /// Retryable errors (network, timeout, transient issues)
    Retryable,
    /// Terminal errors (validation, configuration, permanent failures)
    Terminal,
}

/// Agent executor that manages the execution lifecycle
///
/// This is the core execution engine that orchestrates the interaction
/// between the model, tools, and memory to accomplish agent tasks.
pub struct AgentExecutor {
    /// Agent configuration
    config: AgentConfig,

    /// LLM model
    model: Box<dyn Model>,

    /// Tool executor (optional)
    tool_executor: Option<Arc<dyn ToolExecutor>>,

    /// Memory backend (optional)
    memory: Option<Arc<SimpleMemory>>,
}

impl AgentExecutor {
    /// Create a new agent executor
    pub fn new(
        config: AgentConfig,
        model: Box<dyn Model>,
        tool_executor: Option<Arc<dyn ToolExecutor>>,
        memory: Option<Arc<SimpleMemory>>,
    ) -> Self {
        Self {
            config,
            model,
            tool_executor,
            memory,
        }
    }

    /// Execute the agent with streaming support for real-time updates
    ///
    /// This runs the main execution loop with streaming:
    /// 1. Build model request from context
    /// 2. Call model.generate_stream()
    /// 3. Stream text deltas and events to channel
    /// 4. Handle tool calls and emit events
    /// 5. Repeat until done or max iterations
    ///
    /// # Arguments
    /// * `ctx` - Agent context (mutable for state updates)
    /// * `stream_tx` - Channel sender for streaming events
    ///
    /// # Returns
    /// The final response content
    ///
    /// # Example
    /// ```no_run
    /// use tokio::sync::mpsc;
    /// # use aof_runtime::executor::AgentExecutor;
    /// # use aof_core::AgentContext;
    /// # async fn example(executor: AgentExecutor) {
    /// let (tx, mut rx) = mpsc::channel(100);
    /// let mut ctx = AgentContext::new("Hello");
    ///
    /// // Spawn task to handle stream events
    /// tokio::spawn(async move {
    ///     while let Some(event) = rx.recv().await {
    ///         println!("Event: {:?}", event);
    ///     }
    /// });
    ///
    /// let result = executor.execute_streaming(&mut ctx, tx).await;
    /// # }
    /// ```
    pub async fn execute_streaming(
        &self,
        ctx: &mut AgentContext,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> AofResult<String> {
        info!("Starting streaming agent execution: {}", self.config.name);
        let execution_start = Instant::now();

        // Add user message if not already in history
        if ctx.messages.is_empty() {
            ctx.add_message(MessageRole::User, ctx.input.clone());
        }

        let mut iteration = 0;
        let max_iterations = self.config.max_iterations;
        let mut accumulated_content = String::new();

        loop {
            iteration += 1;

            if iteration > max_iterations {
                let error_msg = format!("Exceeded max iterations ({})", max_iterations);
                let _ = stream_tx.send(StreamEvent::Error {
                    message: error_msg.clone(),
                }).await;

                warn!("Reached max iterations ({}) for agent: {}", max_iterations, self.config.name);
                return Err(AofError::agent(error_msg));
            }

            // Emit iteration start event
            let _ = stream_tx.send(StreamEvent::IterationStart {
                iteration,
                max_iterations,
            }).await;

            debug!("Agent iteration {}/{} for: {}", iteration, max_iterations, self.config.name);

            // Build model request with streaming enabled
            let mut request = self.build_model_request(ctx)?;
            request.stream = true;

            // Call model streaming API
            let stream_result = self.model.generate_stream(&request).await;

            let mut stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    let error_msg = format!("Model streaming failed: {}", e);
                    let _ = stream_tx.send(StreamEvent::Error {
                        message: error_msg.clone(),
                    }).await;
                    return Err(AofError::agent(error_msg));
                }
            };

            let mut iteration_content = String::new();
            let mut tool_calls_buffer: Vec<ToolCall> = Vec::new();
            let mut current_stop_reason = StopReason::EndTurn;
            let mut usage = aof_core::Usage::default();

            // Process stream chunks
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        match chunk {
                            StreamChunk::ContentDelta { delta } => {
                                iteration_content.push_str(&delta);

                                // Send text delta event
                                let _ = stream_tx.send(StreamEvent::TextDelta {
                                    delta,
                                    timestamp: Some(
                                        std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis() as u64
                                    ),
                                }).await;
                            }
                            StreamChunk::ToolCall { tool_call } => {
                                // Emit tool call start event
                                let _ = stream_tx.send(StreamEvent::ToolCallStart {
                                    tool_name: tool_call.name.clone(),
                                    tool_id: tool_call.id.clone(),
                                    arguments: tool_call.arguments.clone(),
                                }).await;

                                tool_calls_buffer.push(tool_call);
                            }
                            StreamChunk::Done { usage: chunk_usage, stop_reason } => {
                                usage = chunk_usage;
                                current_stop_reason = stop_reason;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Stream chunk error: {}", e);
                        let _ = stream_tx.send(StreamEvent::Error {
                            message: error_msg.clone(),
                        }).await;
                        error!("Stream error: {}", e);
                        return Err(AofError::agent(error_msg));
                    }
                }
            }

            // Update usage statistics
            ctx.metadata.input_tokens += usage.input_tokens;
            ctx.metadata.output_tokens += usage.output_tokens;
            ctx.metadata.model = Some(self.model.config().model.clone());

            debug!(
                "Model stream complete - stop_reason: {:?}, content length: {}, tool_calls: {}",
                current_stop_reason,
                iteration_content.len(),
                tool_calls_buffer.len()
            );

            // Add assistant message to history
            let mut assistant_msg = aof_core::Message {
                role: MessageRole::Assistant,
                content: iteration_content.clone(),
                tool_calls: None,
            };

            if !tool_calls_buffer.is_empty() {
                assistant_msg.tool_calls = Some(tool_calls_buffer.clone());
            }

            ctx.messages.push(assistant_msg);
            accumulated_content.push_str(&iteration_content);

            // Emit iteration complete event
            let _ = stream_tx.send(StreamEvent::IterationComplete {
                iteration,
                stop_reason: current_stop_reason,
            }).await;

            // Handle stop reason
            match current_stop_reason {
                StopReason::EndTurn => {
                    info!("Agent execution completed in {} iterations", iteration);
                    ctx.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;

                    // Emit done event
                    let _ = stream_tx.send(StreamEvent::Done {
                        content: accumulated_content.clone(),
                        total_iterations: iteration,
                        execution_time_ms: ctx.metadata.execution_time_ms,
                        input_tokens: ctx.metadata.input_tokens,
                        output_tokens: ctx.metadata.output_tokens,
                    }).await;

                    return Ok(accumulated_content);
                }

                StopReason::ToolUse => {
                    // Log tool calls for visibility
                    info!("→ TOOL CALLS: {}", tool_calls_buffer.iter()
                        .map(|tc| tc.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "));

                    for tool_call in &tool_calls_buffer {
                        let args_str = serde_json::to_string(&tool_call.arguments)
                            .unwrap_or_else(|_| "{}".to_string());
                        info!("  • {} {}", tool_call.name, args_str);
                    }

                    // Execute tools and emit events
                    debug!("Executing {} tool calls", tool_calls_buffer.len());
                    let tool_results = self.execute_tools_streaming(&tool_calls_buffer, &stream_tx).await?;

                    ctx.metadata.tool_calls += tool_results.len();

                    // Add tool results to context and log them
                    for (tool_call, result) in tool_calls_buffer.iter().zip(tool_results.iter()) {
                        // Log tool result
                        if result.success {
                            let result_summary = match &result.data {
                                serde_json::Value::String(s) => {
                                    if s.len() > 100 {
                                        format!("{}...", &s[..100])
                                    } else {
                                        s.clone()
                                    }
                                }
                                other => {
                                    let s = other.to_string();
                                    if s.len() > 100 {
                                        format!("{}...", &s[..100])
                                    } else {
                                        s
                                    }
                                }
                            };
                            info!("✓ {}: {}", tool_call.name, result_summary);
                        } else {
                            info!("✗ {}: {}", tool_call.name, result.error.as_deref().unwrap_or("Unknown error"));
                        }

                        let agent_result = aof_core::AgentToolResult {
                            tool_name: tool_call.name.clone(),
                            result: result.data.clone(),
                            success: result.success,
                            error: result.error.clone(),
                        };
                        ctx.tool_results.push(agent_result);

                        let tool_msg = aof_core::Message {
                            role: MessageRole::Tool,
                            content: serde_json::to_string(&result.data)
                                .unwrap_or_else(|_| "{}".to_string()),
                            tool_calls: None,
                        };
                        ctx.messages.push(tool_msg);
                    }

                    // Continue loop for next iteration
                    continue;
                }

                StopReason::MaxTokens => {
                    warn!("Model reached max tokens");
                    ctx.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;

                    let _ = stream_tx.send(StreamEvent::Done {
                        content: accumulated_content.clone(),
                        total_iterations: iteration,
                        execution_time_ms: ctx.metadata.execution_time_ms,
                        input_tokens: ctx.metadata.input_tokens,
                        output_tokens: ctx.metadata.output_tokens,
                    }).await;

                    return Ok(accumulated_content);
                }

                StopReason::StopSequence => {
                    info!("Model hit stop sequence");
                    ctx.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;

                    let _ = stream_tx.send(StreamEvent::Done {
                        content: accumulated_content.clone(),
                        total_iterations: iteration,
                        execution_time_ms: ctx.metadata.execution_time_ms,
                        input_tokens: ctx.metadata.input_tokens,
                        output_tokens: ctx.metadata.output_tokens,
                    }).await;

                    return Ok(accumulated_content);
                }

                StopReason::ContentFilter => {
                    let error_msg = "Content filter triggered by model".to_string();
                    let _ = stream_tx.send(StreamEvent::Error {
                        message: error_msg.clone(),
                    }).await;
                    error!("Content filter triggered");
                    return Err(AofError::agent(error_msg));
                }
            }
        }
    }

    /// Execute the agent with the given context (non-streaming)
    ///
    /// This runs the main execution loop:
    /// 1. Restore conversation history from memory (if available)
    /// 2. Build model request from context
    /// 3. Call model.generate()
    /// 4. Store conversation turn in memory
    /// 5. Handle response (execute tools if needed)
    /// 6. Repeat until done or max iterations
    pub async fn execute(&self, context: &mut AgentContext) -> AofResult<String> {
        warn!("=== AGENT EXECUTOR START === name={}", self.config.name);
        let execution_start = Instant::now();

        // Restore conversation history from memory if available
        if let Some(memory) = &self.memory {
            warn!("[EXECUTOR] Restoring conversation history from memory...");
            self.restore_conversation_history(context, memory).await?;
            warn!("[EXECUTOR] Memory restore complete, messages count: {}", context.messages.len());
        }

        // Always add the current user input to the message history for this execution
        // This is critical for multi-turn conversations - even if we restored history,
        // we need to add the NEW user query that triggered this execution
        warn!("[EXECUTOR] Adding current user input to context: {:?}", context.input.chars().take(50).collect::<String>());
        context.add_message(MessageRole::User, context.input.clone());

        let mut iteration = 0;
        let max_iterations = self.config.max_iterations;
        warn!("[EXECUTOR] Starting execution loop, max_iterations={}", max_iterations);

        loop {
            iteration += 1;

            if iteration > max_iterations {
                error!(
                    "[EXECUTOR] Reached max iterations ({}) for agent: {}",
                    max_iterations, self.config.name
                );
                return Err(AofError::agent(format!(
                    "Exceeded max iterations ({})",
                    max_iterations
                )));
            }

            warn!(
                "[EXECUTOR] Iteration {}/{} for agent: {}",
                iteration, max_iterations, self.config.name
            );

            // Build model request
            warn!("[EXECUTOR] Building model request...");
            let request = match self.build_model_request(context) {
                Ok(req) => {
                    warn!("[EXECUTOR] Model request built: messages={}, tools={}, system={:?}",
                        req.messages.len(),
                        req.tools.len(),
                        req.system.as_ref().map(|s| s.chars().take(30).collect::<String>())
                    );
                    req
                }
                Err(e) => {
                    error!("[EXECUTOR] Failed to build model request: {:?}", e);
                    return Err(e);
                }
            };

            // Call model
            warn!("[EXECUTOR] Calling model.generate()...");
            let generate_start = Instant::now();
            let response = match self.model.generate(&request).await {
                Ok(resp) => {
                    warn!("[EXECUTOR] model.generate() SUCCESS in {}ms: stop_reason={:?}, content_len={}, tool_calls={}",
                        generate_start.elapsed().as_millis(),
                        resp.stop_reason,
                        resp.content.len(),
                        resp.tool_calls.len()
                    );
                    resp
                }
                Err(e) => {
                    error!("[EXECUTOR] model.generate() FAILED in {}ms: {:?}",
                        generate_start.elapsed().as_millis(), e
                    );
                    return Err(AofError::agent(format!("Model generation failed: {}", e)));
                }
            };

            // Update usage statistics
            context.metadata.input_tokens += response.usage.input_tokens;
            context.metadata.output_tokens += response.usage.output_tokens;
            context.metadata.model = Some(self.model.config().model.clone());

            debug!(
                "Model response - stop_reason: {:?}, content length: {}, tool_calls: {}",
                response.stop_reason,
                response.content.len(),
                response.tool_calls.len()
            );

            // Add assistant message to history
            let mut assistant_msg = aof_core::Message {
                role: MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: None,
            };

            if !response.tool_calls.is_empty() {
                assistant_msg.tool_calls = Some(response.tool_calls.clone());
            }

            context.messages.push(assistant_msg);

            // Store conversation turn in memory after each response
            if let Some(memory) = &self.memory {
                self.store_conversation_turn(context, memory, iteration).await?;
            }

            // Handle stop reason
            match response.stop_reason {
                StopReason::EndTurn => {
                    info!(
                        "Agent execution completed in {} iterations",
                        iteration
                    );
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::ToolUse => {
                    // Log tool calls for visibility
                    info!("→ TOOL CALLS: {}", response.tool_calls.iter()
                        .map(|tc| tc.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "));

                    for tool_call in &response.tool_calls {
                        let args_str = serde_json::to_string(&tool_call.arguments)
                            .unwrap_or_else(|_| "{}".to_string());
                        info!("  • {} {}", tool_call.name, args_str);
                    }

                    // Execute tools
                    debug!("Executing {} tool calls", response.tool_calls.len());
                    let tool_results = self.execute_tools(&response.tool_calls).await?;

                    context.metadata.tool_calls += tool_results.len();

                    // Add tool results to context and log them
                    for (tool_call, result) in response.tool_calls.iter().zip(tool_results.iter()) {
                        // Log tool result
                        if result.success {
                            let result_summary = match &result.data {
                                serde_json::Value::String(s) => {
                                    if s.len() > 100 {
                                        format!("{}...", &s[..100])
                                    } else {
                                        s.clone()
                                    }
                                }
                                other => {
                                    let s = other.to_string();
                                    if s.len() > 100 {
                                        format!("{}...", &s[..100])
                                    } else {
                                        s
                                    }
                                }
                            };
                            info!("✓ {}: {}", tool_call.name, result_summary);
                        } else {
                            info!("✗ {}: {}", tool_call.name, result.error.as_deref().unwrap_or("Unknown error"));
                        }

                        // Convert tool result to agent tool result
                        let agent_result = aof_core::AgentToolResult {
                            tool_name: tool_call.name.clone(),
                            result: result.data.clone(),
                            success: result.success,
                            error: result.error.clone(),
                        };
                        context.tool_results.push(agent_result);

                        // Add tool result message to history
                        let tool_msg = aof_core::Message {
                            role: MessageRole::Tool,
                            content: serde_json::to_string(&result.data)
                                .unwrap_or_else(|_| "{}".to_string()),
                            tool_calls: None,
                        };
                        context.messages.push(tool_msg);
                    }

                    // Continue loop for next iteration
                    continue;
                }

                StopReason::MaxTokens => {
                    warn!("Model reached max tokens");
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::StopSequence => {
                    info!("Model hit stop sequence");
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::ContentFilter => {
                    error!("Content filter triggered");
                    return Err(AofError::agent(
                        "Content filter triggered by model".to_string(),
                    ));
                }
            }
        }
    }

    /// Build a model request from the current context
    fn build_model_request(&self, context: &AgentContext) -> AofResult<ModelRequest> {
        warn!("[BUILD_REQUEST] Building model request...");

        // Convert context messages to request messages
        let messages: Vec<RequestMessage> = context
            .messages
            .iter()
            .map(|m| RequestMessage {
                role: match m.role {
                    MessageRole::User => aof_core::model::MessageRole::User,
                    MessageRole::Assistant => aof_core::model::MessageRole::Assistant,
                    MessageRole::System => aof_core::model::MessageRole::System,
                    MessageRole::Tool => aof_core::model::MessageRole::Tool,
                },
                content: m.content.clone(),
                tool_calls: m.tool_calls.clone(),
            })
            .collect();

        warn!("[BUILD_REQUEST] Converted {} messages", messages.len());

        // Get tool definitions if available
        let tools: Vec<ModelToolDefinition> = if let Some(executor) = &self.tool_executor {
            warn!("[BUILD_REQUEST] Tool executor available, listing tools...");
            let tool_defs = executor.list_tools();
            warn!("[BUILD_REQUEST] Got {} tool definitions from executor", tool_defs.len());
            for t in &tool_defs {
                warn!("[BUILD_REQUEST] Tool: name={}, desc_len={}", t.name, t.description.len());
            }
            tool_defs
                .into_iter()
                .map(|t| ModelToolDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                })
                .collect()
        } else {
            warn!("[BUILD_REQUEST] No tool executor, tools will be empty");
            Vec::new()
        };

        warn!("[BUILD_REQUEST] Final: messages={}, tools={}, system_prompt={:?}",
            messages.len(),
            tools.len(),
            self.config.system_prompt.as_ref().map(|s| s.len())
        );

        Ok(ModelRequest {
            messages,
            system: self.config.system_prompt.clone(),
            tools,
            temperature: Some(self.config.temperature),
            max_tokens: self.config.max_tokens,
            stream: false,
            extra: std::collections::HashMap::new(),
        })
    }

    /// Execute a list of tool calls with streaming events
    async fn execute_tools_streaming(
        &self,
        tool_calls: &[ToolCall],
        stream_tx: &mpsc::Sender<StreamEvent>,
    ) -> AofResult<Vec<ToolResult>> {
        const MAX_PARALLEL_TOOLS: usize = 10;

        let executor = self
            .tool_executor
            .as_ref()
            .ok_or_else(|| AofError::tool("No tool executor available".to_string()))?;

        if tool_calls.is_empty() {
            return Ok(Vec::new());
        }

        // Single tool - execute with retry and emit events
        if tool_calls.len() == 1 {
            debug!("Executing single tool with streaming: {}", tool_calls[0].name);
            let result = self.execute_tool_with_retry(executor, &tool_calls[0]).await;

            // Emit tool complete event
            let _ = stream_tx.send(StreamEvent::ToolCallComplete {
                tool_name: tool_calls[0].name.clone(),
                tool_id: tool_calls[0].id.clone(),
                success: result.success,
                execution_time_ms: result.execution_time_ms,
                error: result.error.clone(),
            }).await;

            return Ok(vec![result]);
        }

        // Parallel execution for multiple tools
        info!(
            "Executing {} tools in parallel (max concurrency: {})",
            tool_calls.len(),
            MAX_PARALLEL_TOOLS
        );

        let parallel_start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(MAX_PARALLEL_TOOLS));
        let mut join_set = JoinSet::new();


        // Spawn tasks for each tool call
        for (idx, tool_call) in tool_calls.iter().enumerate() {
            let tool_call_clone = tool_call.clone();
            let executor_clone = Arc::clone(executor);
            let semaphore_clone = Arc::clone(&semaphore);
            let config_name = self.config.name.clone();

            join_set.spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                debug!("Executing tool [{}]: {}", idx, tool_call_clone.name);

                let result = Self::execute_tool_with_retry_static(
                    &executor_clone,
                    &tool_call_clone,
                    &config_name
                ).await;

                (idx, tool_call_clone, result)
            });
        }

        // Collect results and emit events
        let mut results = vec![None; tool_calls.len()];
        let mut success_count = 0;
        let mut failure_count = 0;

        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((idx, tool_call, tool_result)) => {
                    if tool_result.success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }

                    // Emit tool complete event
                    let _ = stream_tx.send(StreamEvent::ToolCallComplete {
                        tool_name: tool_call.name.clone(),
                        tool_id: tool_call.id.clone(),
                        success: tool_result.success,
                        execution_time_ms: tool_result.execution_time_ms,
                        error: tool_result.error.clone(),
                    }).await;

                    results[idx] = Some(tool_result);
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    failure_count += 1;
                    if let Some(idx) = results.iter().position(|r| r.is_none()) {
                        results[idx] = Some(ToolResult {
                            success: false,
                            data: serde_json::Value::Null,
                            error: Some(format!("Task failed to join: {}", e)),
                            execution_time_ms: 0,
                        });
                    }
                }
            }
        }

        let parallel_duration = parallel_start.elapsed();
        info!(
            "Parallel tool execution completed: {} tools in {}ms ({} success, {} failures)",
            tool_calls.len(),
            parallel_duration.as_millis(),
            success_count,
            failure_count
        );

        let final_results: Vec<ToolResult> = results
            .into_iter()
            .enumerate()
            .map(|(idx, opt_result)| {
                opt_result.unwrap_or_else(|| {
                    error!("Tool at index {} did not complete", idx);
                    ToolResult {
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some("Tool execution did not complete".to_string()),
                        execution_time_ms: 0,
                    }
                })
            })
            .collect();

        Ok(final_results)
    }

    /// Execute a list of tool calls in parallel with semaphore-based concurrency control
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> AofResult<Vec<ToolResult>> {
        const MAX_PARALLEL_TOOLS: usize = 10;

        let executor = self
            .tool_executor
            .as_ref()
            .ok_or_else(|| AofError::tool("No tool executor available".to_string()))?;

        // Early return for empty tool calls
        if tool_calls.is_empty() {
            return Ok(Vec::new());
        }

        // Single tool - execute with retry and resilience
        if tool_calls.len() == 1 {
            debug!("Executing single tool with resilience: {}", tool_calls[0].name);
            let result = self.execute_tool_with_retry(executor, &tool_calls[0]).await;
            return Ok(vec![result]);
        }

        // Parallel execution for multiple tools
        info!(
            "Executing {} tools in parallel (max concurrency: {})",
            tool_calls.len(),
            MAX_PARALLEL_TOOLS
        );

        let parallel_start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(MAX_PARALLEL_TOOLS));
        let mut join_set = JoinSet::new();

        // Spawn tasks for each tool call with resilience
        for (idx, tool_call) in tool_calls.iter().enumerate() {
            let tool_call_clone = tool_call.clone();
            let executor_clone = Arc::clone(executor);
            let semaphore_clone = Arc::clone(&semaphore);

            // Clone self methods needed for retry logic
            let config_name = self.config.name.clone();

            join_set.spawn(async move {
                // Acquire semaphore permit to limit concurrency
                let _permit = semaphore_clone.acquire().await.unwrap();

                debug!("Executing tool [{}] with resilience: {}", idx, tool_call_clone.name);

                // Execute with retry, timeout, and validation
                let result = Self::execute_tool_with_retry_static(
                    &executor_clone,
                    &tool_call_clone,
                    &config_name
                ).await;

                (idx, result)
            });
        }

        // Collect results while maintaining order
        let mut results = vec![None; tool_calls.len()];
        let mut success_count = 0;
        let mut failure_count = 0;

        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((idx, tool_result)) => {
                    if tool_result.success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                    results[idx] = Some(tool_result);
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    failure_count += 1;
                    // Create error result for failed task
                    // We need to find which index failed - use first None
                    if let Some(idx) = results.iter().position(|r| r.is_none()) {
                        results[idx] = Some(ToolResult {
                            success: false,
                            data: serde_json::Value::Null,
                            error: Some(format!("Task failed to join: {}", e)),
                            execution_time_ms: 0,
                        });
                    }
                }
            }
        }

        let parallel_duration = parallel_start.elapsed();

        info!(
            "Parallel tool execution completed: {} tools in {}ms ({} success, {} failures)",
            tool_calls.len(),
            parallel_duration.as_millis(),
            success_count,
            failure_count
        );

        // Convert Option<ToolResult> to ToolResult, filling any missing with errors
        let final_results: Vec<ToolResult> = results
            .into_iter()
            .enumerate()
            .map(|(idx, opt_result)| {
                opt_result.unwrap_or_else(|| {
                    error!("Tool at index {} did not complete", idx);
                    ToolResult {
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some("Tool execution did not complete".to_string()),
                        execution_time_ms: 0,
                    }
                })
            })
            .collect();

        Ok(final_results)
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get model reference
    pub fn model(&self) -> &dyn Model {
        self.model.as_ref()
    }

    /// Get tool executor reference
    pub fn tool_executor(&self) -> Option<&Arc<dyn ToolExecutor>> {
        self.tool_executor.as_ref()
    }

    /// Get memory reference
    pub fn memory(&self) -> Option<&Arc<SimpleMemory>> {
        self.memory.as_ref()
    }

    /// Restore conversation history from memory
    async fn restore_conversation_history(
        &self,
        context: &mut AgentContext,
        memory: &Arc<SimpleMemory>,
    ) -> AofResult<()> {
        let conversation_key = format!("agent:{}:conversation", self.config.name);

        if let Some(history) = memory.retrieve::<Vec<aof_core::Message>>(&conversation_key).await? {
            debug!(
                "Restored {} messages from memory for agent: {}",
                history.len(),
                self.config.name
            );

            // Prune history if it exceeds context window
            let pruned_history = self.prune_conversation_history(history);
            context.messages = pruned_history;
        } else {
            debug!("No conversation history found for agent: {}", self.config.name);
        }

        Ok(())
    }

    /// Store conversation turn in memory
    async fn store_conversation_turn(
        &self,
        context: &AgentContext,
        memory: &Arc<SimpleMemory>,
        iteration: usize,
    ) -> AofResult<()> {
        let conversation_key = format!("agent:{}:conversation", self.config.name);
        let turn_key = format!("agent:{}:turn:{}", self.config.name, iteration);

        // Store full conversation history
        let conversation_value = serde_json::to_value(&context.messages)
            .map_err(|e| AofError::memory(format!("Failed to serialize messages: {}", e)))?;

        memory.store(&conversation_key, conversation_value).await?;

        // Store individual turn with metadata for semantic search
        let turn_value = serde_json::json!({
            "iteration": iteration,
            "message_count": context.messages.len(),
            "input_tokens": context.metadata.input_tokens,
            "output_tokens": context.metadata.output_tokens,
            "tool_calls": context.metadata.tool_calls,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        memory.store(&turn_key, turn_value).await?;

        debug!(
            "Stored conversation turn {} for agent: {}",
            iteration, self.config.name
        );

        Ok(())
    }

    /// Prune conversation history to fit context window
    fn prune_conversation_history(&self, mut history: Vec<aof_core::Message>) -> Vec<aof_core::Message> {
        const MAX_MESSAGES: usize = 100;

        if history.len() > MAX_MESSAGES {
            warn!(
                "Pruning conversation history from {} to {} messages for agent: {}",
                history.len(),
                MAX_MESSAGES,
                self.config.name
            );

            // Keep system messages and most recent messages
            let system_messages: Vec<_> = history
                .iter()
                .filter(|m| m.role == MessageRole::System)
                .cloned()
                .collect();

            // Take most recent non-system messages
            let recent_messages: Vec<_> = history
                .into_iter()
                .filter(|m| m.role != MessageRole::System)
                .rev()
                .take(MAX_MESSAGES - system_messages.len())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            // Combine system + recent
            history = system_messages;
            history.extend(recent_messages);
        }

        history
    }

    /// Cleanup expired memory entries
    pub async fn cleanup_expired_memory(&self) -> AofResult<()> {
        if let Some(memory) = &self.memory {
            let prefix = format!("agent:{}:", self.config.name);

            // List all keys for this agent
            let keys = memory.list_keys().await?;
            let agent_keys: Vec<_> = keys.iter()
                .filter(|k| k.starts_with(&prefix))
                .collect();

            debug!(
                "Checking {} memory entries for expiry for agent: {}",
                agent_keys.len(),
                self.config.name
            );

            // Memory backend handles lazy cleanup on retrieve
            for key in agent_keys {
                let _: Option<serde_json::Value> = memory.retrieve(key).await?;
            }
        }

        Ok(())
    }

    /// Search memory for relevant context
    pub async fn search_memory(
        &self,
        query: &str,
    ) -> AofResult<Vec<aof_core::MemoryEntry>> {
        if let Some(memory) = &self.memory {
            let prefix = format!("agent:{}:turn:", self.config.name);

            let keys = memory.list_keys().await?;
            let mut entries = Vec::new();

            for key in keys {
                if key.starts_with(&prefix) {
                    if let Some(value) = memory.retrieve::<serde_json::Value>(&key).await? {
                        entries.push(aof_core::MemoryEntry::new(key, value));
                    }
                }
            }

            debug!(
                "Found {} memory entries matching query '{}' for agent: {}",
                entries.len(),
                query,
                self.config.name
            );

            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }

    /// Execute a single tool call with timeout, retry, and validation (instance method)
    async fn execute_tool_with_retry(
        &self,
        executor: &Arc<dyn ToolExecutor>,
        tool_call: &ToolCall,
    ) -> ToolResult {
        Self::execute_tool_with_retry_static(executor, tool_call, &self.config.name).await
    }

    /// Execute a single tool call with timeout, retry, and validation (static method for parallel tasks)
    async fn execute_tool_with_retry_static(
        executor: &Arc<dyn ToolExecutor>,
        tool_call: &ToolCall,
        agent_name: &str,
    ) -> ToolResult {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 1000; // 1 second
        const TIMEOUT_SECS: u64 = 30; // 30 seconds per attempt

        let mut attempt = 0;
        let tool_start = Instant::now();

        while attempt < MAX_RETRIES {
            attempt += 1;
            let attempt_start = Instant::now();

            debug!(
                "[{}] Tool {} attempt {}/{} (tool_id: {})",
                agent_name, tool_call.name, attempt, MAX_RETRIES, tool_call.id
            );

            // Execute with timeout
            let input = ToolInput::new(tool_call.arguments.clone());
            let timeout_duration = Duration::from_secs(TIMEOUT_SECS);

            let result = tokio::time::timeout(
                timeout_duration,
                executor.execute_tool(&tool_call.name, input),
            )
            .await;

            let attempt_duration = attempt_start.elapsed();

            match result {
                // Timeout occurred
                Err(_) => {
                    let error_msg = format!(
                        "Tool {} timed out after {}s (attempt {}/{})",
                        tool_call.name, TIMEOUT_SECS, attempt, MAX_RETRIES
                    );
                    warn!("[{}] {}", agent_name, error_msg);

                    // Timeout is retryable - check if we should retry
                    if attempt < MAX_RETRIES {
                        let backoff = INITIAL_BACKOFF_MS * (2_u64.pow(attempt - 1));
                        info!(
                            "[{}] Retrying tool {} after {}ms backoff",
                            agent_name, tool_call.name, backoff
                        );
                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                        continue;
                    }

                    // Max retries exceeded
                    return ToolResult {
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some(error_msg),
                        execution_time_ms: tool_start.elapsed().as_millis() as u64,
                    };
                }

                // Tool execution completed (success or error)
                Ok(tool_result) => {
                    match tool_result {
                        Ok(mut result) => {
                            // Validate tool result
                            if let Err(validation_error) = Self::validate_tool_result(&result) {
                                error!(
                                    "[{}] Tool {} validation failed: {}",
                                    agent_name, tool_call.name, validation_error
                                );

                                // Validation errors are terminal (not retryable)
                                return ToolResult {
                                    success: false,
                                    data: serde_json::Value::Null,
                                    error: Some(format!("Validation failed: {}", validation_error)),
                                    execution_time_ms: attempt_duration.as_millis() as u64,
                                };
                            }

                            // Update execution time and log metrics
                            result.execution_time_ms = attempt_duration.as_millis() as u64;
                            info!(
                                "[{}] Tool {} succeeded on attempt {} in {}ms",
                                agent_name, tool_call.name, attempt, result.execution_time_ms
                            );

                            // Collect metrics
                            Self::collect_tool_metrics(agent_name, &tool_call.name, attempt, &result);

                            return result;
                        }

                        Err(e) => {
                            let error_msg = e.to_string();
                            error!(
                                "[{}] Tool {} execution error (attempt {}/{}): {}",
                                agent_name, tool_call.name, attempt, MAX_RETRIES, error_msg
                            );

                            // Categorize error to determine if retryable
                            let error_category = Self::categorize_error(&e);

                            match error_category {
                                ErrorCategory::Retryable => {
                                    if attempt < MAX_RETRIES {
                                        let backoff = INITIAL_BACKOFF_MS * (2_u64.pow(attempt - 1));
                                        info!(
                                            "[{}] Retrying tool {} after {}ms backoff (retryable error: {})",
                                            agent_name, tool_call.name, backoff, error_msg
                                        );
                                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                                        continue;
                                    }
                                }
                                ErrorCategory::Terminal => {
                                    // Terminal errors - don't retry
                                    warn!(
                                        "[{}] Tool {} failed with terminal error, not retrying: {}",
                                        agent_name, tool_call.name, error_msg
                                    );
                                }
                            }

                            // Return error result (max retries or terminal error)
                            return ToolResult {
                                success: false,
                                data: serde_json::Value::Null,
                                error: Some(error_msg),
                                execution_time_ms: attempt_duration.as_millis() as u64,
                            };
                        }
                    }
                }
            }
        }

        // Should never reach here, but return error as fallback
        ToolResult {
            success: false,
            data: serde_json::Value::Null,
            error: Some(format!(
                "Tool {} failed after {} attempts",
                tool_call.name, MAX_RETRIES
            )),
            execution_time_ms: tool_start.elapsed().as_millis() as u64,
        }
    }

    /// Categorize errors as retryable or terminal
    fn categorize_error(error: &AofError) -> ErrorCategory {
        match error {
            // Network-related errors are retryable
            AofError::Timeout(_) => ErrorCategory::Retryable,
            AofError::Io(_) => ErrorCategory::Retryable,

            // Model and MCP errors might be transient
            AofError::Model(msg) if msg.contains("timeout") || msg.contains("network") => {
                ErrorCategory::Retryable
            }
            AofError::Mcp(msg) if msg.contains("timeout") || msg.contains("connection") => {
                ErrorCategory::Retryable
            }

            // Validation, config, and serialization errors are terminal
            AofError::Config(_) => ErrorCategory::Terminal,
            AofError::Serialization(_) => ErrorCategory::Terminal,
            AofError::InvalidState(_) => ErrorCategory::Terminal,

            // Tool-specific validation errors are terminal
            AofError::Tool(msg) if msg.contains("validation") || msg.contains("invalid") => {
                ErrorCategory::Terminal
            }

            // Default to terminal for safety
            _ => ErrorCategory::Terminal,
        }
    }

    /// Validate tool result data
    fn validate_tool_result(result: &ToolResult) -> Result<(), String> {
        // Check if error occurred but success flag is true
        if result.success && result.error.is_some() {
            return Err("Inconsistent state: success=true but error is present".to_string());
        }

        // Check if success but data is null
        if result.success && result.data.is_null() {
            warn!("Tool succeeded but returned null data");
        }

        // Check if failure but no error message
        if !result.success && result.error.is_none() {
            return Err("Inconsistent state: success=false but no error message".to_string());
        }

        // Validate execution time is reasonable
        if result.execution_time_ms > 300_000 {
            // > 5 minutes
            warn!(
                "Tool execution time seems excessive: {}ms",
                result.execution_time_ms
            );
        }

        Ok(())
    }

    /// Collect metrics for tool execution
    fn collect_tool_metrics(agent_name: &str, tool_name: &str, attempts: u32, result: &ToolResult) {
        // Log metrics with structured data
        info!(
            agent = %agent_name,
            tool = %tool_name,
            attempts = attempts,
            success = result.success,
            execution_time_ms = result.execution_time_ms,
            "Tool execution metrics"
        );

        // Additional metric logging for monitoring systems
        if attempts > 1 {
            warn!(
                "[{}] Tool {} required {} attempts to complete",
                agent_name, tool_name, attempts
            );
        }

        if result.execution_time_ms > 5000 {
            warn!(
                "[{}] Tool {} took {}ms (>5s)",
                agent_name, tool_name, result.execution_time_ms
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{ModelConfig, ModelProvider, ModelResponse, StopReason, Usage};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock model for testing
    struct MockModel {
        responses: Vec<ModelResponse>,
        current: std::sync::Mutex<usize>,
        config: ModelConfig,
    }

    impl MockModel {
        fn new(responses: Vec<ModelResponse>) -> Self {
            Self {
                responses,
                current: std::sync::Mutex::new(0),
                config: ModelConfig {
                    model: "mock-model".to_string(),
                    provider: ModelProvider::Custom,
                    api_key: None,
                    endpoint: None,
                    temperature: 0.7,
                    max_tokens: None,
                    timeout_secs: 60,
                    headers: HashMap::new(),
                    extra: HashMap::new(),
                },
            }
        }
    }

    #[async_trait]
    impl Model for MockModel {
        async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
            let mut current = self.current.lock().unwrap();
            let idx = *current;
            *current += 1;

            if idx < self.responses.len() {
                Ok(self.responses[idx].clone())
            } else {
                Ok(ModelResponse {
                    content: "Done".to_string(),
                    tool_calls: vec![],
                    stop_reason: StopReason::EndTurn,
                    usage: Usage {
                        input_tokens: 100,
                        output_tokens: 50,
                    },
                    metadata: HashMap::new(),
                })
            }
        }

        async fn generate_stream(
            &self,
            _request: &ModelRequest,
        ) -> AofResult<std::pin::Pin<Box<dyn futures::Stream<Item = AofResult<aof_core::StreamChunk>> + Send>>>
        {
            unimplemented!()
        }

        fn config(&self) -> &ModelConfig {
            &self.config
        }

        fn provider(&self) -> ModelProvider {
            ModelProvider::Custom
        }
    }

    #[tokio::test]
    async fn test_agent_executor_simple() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: Some("You are a helpful assistant".to_string()),
            model: "test-model".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: Some(1000),
            extra: HashMap::new(),
        };

        let model = Box::new(MockModel::new(vec![ModelResponse {
            content: "Hello! How can I help?".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
            metadata: HashMap::new(),
        }]));

        let executor = AgentExecutor::new(config, model, None, None);

        let mut context = AgentContext::new("Hello");
        let response = executor.execute(&mut context).await.unwrap();

        assert_eq!(response, "Hello! How can I help?");
        assert_eq!(context.metadata.input_tokens, 100);
        assert_eq!(context.metadata.output_tokens, 50);
    }

    #[tokio::test]
    async fn test_agent_executor_max_iterations() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "test-model".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 2,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        // Mock model that keeps requesting tools
        let model = Box::new(MockModel::new(vec![
            ModelResponse {
                content: "Calling tool...".to_string(),
                tool_calls: vec![ToolCall {
                    id: "1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
            ModelResponse {
                content: "Calling tool again...".to_string(),
                tool_calls: vec![ToolCall {
                    id: "2".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
            ModelResponse {
                content: "Should not reach here".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
        ]));

        let executor = AgentExecutor::new(config, model, None, None);

        let mut context = AgentContext::new("Test");
        let result = executor.execute(&mut context).await;

        // Should fail due to max iterations, but we have no tool executor
        // so it will fail on tool execution first
        assert!(result.is_err());
    }
}
