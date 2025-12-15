//! Central message handler for routing and execution
//!
//! This module coordinates message handling across platforms,
//! parsing commands, and executing them through the runtime.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::command::{CommandError, CommandType, TriggerCommand, TriggerTarget};
use crate::platforms::{TriggerMessage, TriggerPlatform};
use crate::response::{TriggerResponse, TriggerResponseBuilder};
use aof_core::{AgentContext, AofError, AofResult};
use aof_runtime::{RuntimeOrchestrator, Task, TaskStatus};

/// Helper trait to convert CommandError to AofError
trait CommandErrorExt<T> {
    fn map_cmd_err(self) -> AofResult<T>;
}

impl<T> CommandErrorExt<T> for Result<T, CommandError> {
    fn map_cmd_err(self) -> AofResult<T> {
        self.map_err(|e| AofError::Config(e.to_string()))
    }
}

/// Handler configuration
#[derive(Debug, Clone)]
pub struct TriggerHandlerConfig {
    /// Enable verbose logging
    pub verbose: bool,

    /// Auto-acknowledge commands
    pub auto_ack: bool,

    /// Maximum concurrent tasks per user
    pub max_tasks_per_user: usize,

    /// Command timeout in seconds
    pub command_timeout_secs: u64,
}

impl Default for TriggerHandlerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            auto_ack: true,
            max_tasks_per_user: 3,
            command_timeout_secs: 300, // 5 minutes
        }
    }
}

/// Central trigger handler
///
/// Routes messages from platforms to appropriate handlers and
/// executes commands through the runtime orchestrator.
pub struct TriggerHandler {
    /// Runtime orchestrator for task execution
    orchestrator: Arc<RuntimeOrchestrator>,

    /// Registered platforms
    platforms: HashMap<String, Arc<dyn TriggerPlatform>>,

    /// Handler configuration
    config: TriggerHandlerConfig,

    /// User task counters (user_id -> active task count)
    user_tasks: Arc<dashmap::DashMap<String, usize>>,
}

impl TriggerHandler {
    /// Create a new trigger handler
    pub fn new(orchestrator: Arc<RuntimeOrchestrator>) -> Self {
        Self {
            orchestrator,
            platforms: HashMap::new(),
            config: TriggerHandlerConfig::default(),
            user_tasks: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Create handler with custom configuration
    pub fn with_config(orchestrator: Arc<RuntimeOrchestrator>, config: TriggerHandlerConfig) -> Self {
        Self {
            orchestrator,
            platforms: HashMap::new(),
            config,
            user_tasks: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Register a platform
    pub fn register_platform(&mut self, platform: Arc<dyn TriggerPlatform>) {
        let name = platform.platform_name();
        info!("Registering platform: {}", name);
        self.platforms.insert(name.to_string(), platform);
    }

    /// Get registered platform
    pub fn get_platform(&self, name: &str) -> Option<&Arc<dyn TriggerPlatform>> {
        self.platforms.get(name)
    }

    /// Handle incoming message from platform
    pub async fn handle_message(&self, platform: &str, message: TriggerMessage) -> AofResult<()> {
        debug!(
            "Handling message from {}: {} (user: {})",
            platform, message.id, message.user.id
        );

        // Get platform for response
        let platform_impl = self
            .platforms
            .get(platform)
            .ok_or_else(|| aof_core::AofError::agent(format!("Unknown platform: {}", platform)))?;

        // Check if user has too many active tasks
        if let Some(count) = self.user_tasks.get(&message.user.id) {
            if *count >= self.config.max_tasks_per_user {
                let response = TriggerResponseBuilder::new()
                    .text(format!(
                        "You have too many active tasks ({}). Please wait for some to complete.",
                        *count
                    ))
                    .error()
                    .build();

                let _ = platform_impl.send_response(&message.channel_id, response).await;
                return Ok(());
            }
        }

        // Parse command
        let cmd = match TriggerCommand::parse(&message) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("Failed to parse command: {}", e);
                let response = self.handle_parse_error(&message, e).await;
                let _ = platform_impl.send_response(&message.channel_id, response).await;
                return Ok(());
            }
        };

        // Auto-acknowledge if enabled
        if self.config.auto_ack {
            let ack = TriggerResponseBuilder::new()
                .text("Processing your request...")
                .build();
            let _ = platform_impl.send_response(&message.channel_id, ack).await;
        }

        // Execute command
        let response = match self.execute_command(cmd).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Command execution failed: {}", e);
                TriggerResponseBuilder::new()
                    .text(format!("Command failed: {}", e))
                    .error()
                    .build()
            }
        };

        // Send response
        if let Err(e) = platform_impl.send_response(&message.channel_id, response).await {
            error!("Failed to send response: {:?}", e);
        }

        Ok(())
    }

    /// Execute a parsed command
    pub async fn execute_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        info!(
            "Executing command: {:?} {:?} (user: {})",
            cmd.command_type, cmd.target, cmd.context.user_id
        );

        match cmd.command_type {
            CommandType::Run => self.handle_run_command(cmd).await,
            CommandType::Create => self.handle_create_command(cmd).await,
            CommandType::Status => self.handle_status_command(cmd).await,
            CommandType::Cancel => self.handle_cancel_command(cmd).await,
            CommandType::List => self.handle_list_command(cmd).await,
            CommandType::Help => Ok(self.handle_help_command(cmd).await),
            CommandType::Info => Ok(self.handle_info_command(cmd).await),
        }
    }

    /// Handle run command
    async fn handle_run_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Agent => {
                let agent_name = cmd.get_arg(0).map_cmd_err()?;
                let input = cmd.args[1..].join(" ");

                // Create task
                let task_id = format!("trigger-{}-{}", cmd.context.user_id, uuid::Uuid::new_v4());
                let task = Task::new(
                    task_id.clone(),
                    format!("{} (user: {})", agent_name, cmd.context.user_id),
                    agent_name.to_string(),
                    input.clone(),
                );

                // Submit to orchestrator
                let handle = self.orchestrator.submit_task(task);

                // Track user task
                self.increment_user_tasks(&cmd.context.user_id);

                // Execute task through runtime with AgentExecutor
                let user_id = cmd.context.user_id.clone();
                let user_tasks = Arc::clone(&self.user_tasks);
                let orchestrator = Arc::clone(&self.orchestrator);
                let task_id_clone = task_id.clone();
                let agent_name_clone = agent_name.to_string();
                let platform = cmd.context.platform.clone();
                let channel_id = cmd.context.channel_id.clone();
                let platforms = self.platforms.clone();

                tokio::spawn(async move {
                    // Execute task through orchestrator
                    let result = orchestrator
                        .execute_task(&task_id_clone, |task| async move {
                            // Create AgentContext
                            let mut context = AgentContext::new(&task.input);

                            // Create a minimal agent configuration for the task
                            use aof_core::{AgentConfig, ModelConfig, ModelProvider};
                            use aof_llm::ProviderFactory;
                            use aof_runtime::AgentExecutor;
                            use aof_memory::{InMemoryBackend, SimpleMemory};
                            use std::collections::HashMap;

                            let config = AgentConfig {
                                name: task.agent_name.clone(),
                                system_prompt: Some("You are a helpful AI assistant.".to_string()),
                                model: "claude-3-5-sonnet-20241022".to_string(),
                                tools: vec![],
                                memory: None,
                                max_iterations: 10,
                                temperature: 0.7,
                                max_tokens: Some(4096),
                                extra: HashMap::new(),
                            };

                            // Create model
                            let model_config = ModelConfig {
                                model: "claude-3-5-sonnet-20241022".to_string(),
                                provider: ModelProvider::Anthropic,
                                api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
                                endpoint: None,
                                temperature: 0.7,
                                max_tokens: Some(4096),
                                timeout_secs: 60,
                                headers: HashMap::new(),
                                extra: HashMap::new(),
                            };

                            let model = match ProviderFactory::create(model_config).await {
                                Ok(m) => m,
                                Err(e) => {
                                    return Ok(format!("Failed to create model: {}", e));
                                }
                            };

                            // Create memory backend
                            let memory_backend = InMemoryBackend::new();
                            let memory = std::sync::Arc::new(SimpleMemory::new(std::sync::Arc::new(memory_backend)));

                            // Create AgentExecutor with model and memory, but no tool executor for now
                            let executor = AgentExecutor::new(
                                config,
                                model,
                                None, // No tool executor for trigger-based agents
                                Some(memory),
                            );

                            // Execute the agent
                            match executor.execute(&mut context).await {
                                Ok(response) => Ok(response),
                                Err(e) => Ok(format!("Agent execution failed: {}", e)),
                            }
                        })
                        .await;

                    // Send completion notification to platform
                    if let Some(platform_impl) = platforms.get(&platform) {
                        let response = match result {
                            Ok(_handle) => {
                                // Wait for task completion
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                                if let Some(task_handle) = orchestrator.get_task(&task_id_clone) {
                                    let status = task_handle.status().await;

                                    match status {
                                        TaskStatus::Completed => {
                                            TriggerResponseBuilder::new()
                                                .text(format!("✅ Task completed: `{}`", task_id_clone))
                                                .success()
                                                .build()
                                        }
                                        TaskStatus::Failed => {
                                            TriggerResponseBuilder::new()
                                                .text(format!("❌ Task failed: `{}`", task_id_clone))
                                                .error()
                                                .build()
                                        }
                                        _ => {
                                            TriggerResponseBuilder::new()
                                                .text(format!("ℹ️ Task status: {:?} - `{}`", status, task_id_clone))
                                                .build()
                                        }
                                    }
                                } else {
                                    TriggerResponseBuilder::new()
                                        .text("Task execution started but handle lost")
                                        .build()
                                }
                            }
                            Err(e) => TriggerResponseBuilder::new()
                                .text(format!("Task execution error: {}", e))
                                .error()
                                .build(),
                        };

                        let _ = platform_impl.send_response(&channel_id, response).await;
                    }

                    // Decrement user task count
                    if let Some(mut count) = user_tasks.get_mut(&user_id) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                });

                Ok(TriggerResponseBuilder::new()
                    .text(format!(
                        "✓ Task started: `{}`\nAgent: {}\nInput: {}\nUse `/status task {}` to check progress",
                        task_id, agent_name,
                        if input.len() > 50 { format!("{}...", &input[..50]) } else { input },
                        task_id
                    ))
                    .success()
                    .build())
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Run command not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle create command
    async fn handle_create_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        Ok(TriggerResponseBuilder::new()
            .text("Create command not yet implemented")
            .build())
    }

    /// Handle status command
    async fn handle_status_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_id = cmd.get_arg(0).map_cmd_err()?;

                if let Some(handle) = self.orchestrator.get_task(task_id) {
                    let task = handle.task().await;
                    let status = handle.status().await;

                    // Build detailed status message
                    let status_icon = match status {
                        TaskStatus::Pending => "⏳",
                        TaskStatus::Running => "▶️",
                        TaskStatus::Completed => "✅",
                        TaskStatus::Failed => "❌",
                        TaskStatus::Cancelled => "🚫",
                    };

                    let mut text = format!(
                        "{} **Task Status**\n\n**ID:** `{}`\n**Name:** {}\n**Agent:** {}\n**Status:** {:?}",
                        status_icon, task.id, task.name, task.agent_name, status
                    );

                    // Add priority if set
                    if task.priority > 0 {
                        text.push_str(&format!("\n**Priority:** {}", task.priority));
                    }

                    // Add metadata if present
                    if !task.metadata.is_empty() {
                        text.push_str("\n\n**Metadata:**");
                        for (key, value) in &task.metadata {
                            text.push_str(&format!("\n• {}: {}", key, value));
                        }
                    }

                    // Add input preview
                    let input_preview = if task.input.len() > 100 {
                        format!("{}...", &task.input[..100])
                    } else {
                        task.input.clone()
                    };
                    text.push_str(&format!("\n\n**Input:** {}", input_preview));

                    Ok(TriggerResponseBuilder::new()
                        .text(text)
                        .build())
                } else {
                    Ok(TriggerResponseBuilder::new()
                        .text(format!("❌ Task not found: `{}`", task_id))
                        .error()
                        .build())
                }
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Status not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle cancel command
    async fn handle_cancel_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_id = cmd.get_arg(0).map_cmd_err()?;

                match self.orchestrator.cancel_task(task_id).await {
                    Ok(_) => Ok(TriggerResponseBuilder::new()
                        .text(format!("✓ Task cancelled: {}", task_id))
                        .success()
                        .build()),
                    Err(e) => Ok(TriggerResponseBuilder::new()
                        .text(format!("Failed to cancel task: {}", e))
                        .error()
                        .build()),
                }
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Cancel not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle list command
    async fn handle_list_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_ids = self.orchestrator.list_tasks();
                let stats = self.orchestrator.stats().await;

                let mut text = format!(
                    "📋 **Task Overview**\n\n**Statistics:**\n⏳ Pending: {}\n▶️ Running: {}\n✅ Completed: {}\n❌ Failed: {}\n🚫 Cancelled: {}\n\n**Capacity:**\n• Max Concurrent: {}\n• Available Slots: {}",
                    stats.pending,
                    stats.running,
                    stats.completed,
                    stats.failed,
                    stats.cancelled,
                    stats.max_concurrent,
                    stats.available_permits
                );

                if !task_ids.is_empty() {
                    text.push_str(&format!("\n\n**Active Tasks ({}):**", task_ids.len()));

                    // Show first 10 tasks with status
                    let display_limit = 10;
                    for (i, task_id) in task_ids.iter().take(display_limit).enumerate() {
                        if let Some(handle) = self.orchestrator.get_task(task_id) {
                            let status = handle.status().await;
                            let icon = match status {
                                TaskStatus::Pending => "⏳",
                                TaskStatus::Running => "▶️",
                                TaskStatus::Completed => "✅",
                                TaskStatus::Failed => "❌",
                                TaskStatus::Cancelled => "🚫",
                            };
                            text.push_str(&format!("\n{}. {} `{}`", i + 1, icon, task_id));
                        } else {
                            text.push_str(&format!("\n{}. `{}`", i + 1, task_id));
                        }
                    }

                    if task_ids.len() > display_limit {
                        text.push_str(&format!("\n\n...and {} more tasks", task_ids.len() - display_limit));
                    }
                } else {
                    text.push_str("\n\n_No active tasks_");
                }

                Ok(TriggerResponseBuilder::new().text(text).build())
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("List not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle help command
    async fn handle_help_command(&self, _cmd: TriggerCommand) -> TriggerResponse {
        let help_text = r#"
**AOF Bot Commands**

**Basic Commands:**
• `/run agent <name> <input>` - Run an agent
• `/status task <id>` - Check task status
• `/cancel task <id>` - Cancel a running task
• `/list tasks` - List all tasks
• `/help` - Show this help

**Examples:**
• `/run agent monitor Check server health`
• `/status task trigger-user123-abc`
• `/list tasks`

**Support:** https://github.com/yourusername/aof
        "#;

        TriggerResponseBuilder::new()
            .text(help_text.trim())
            .build()
    }

    /// Handle info command
    async fn handle_info_command(&self, _cmd: TriggerCommand) -> TriggerResponse {
        let stats = self.orchestrator.stats().await;

        let info_text = format!(
            r#"
**AOF System Info**

**Version:** {}
**Runtime Stats:**
• Max Concurrent: {}
• Available Permits: {}
• Active Tasks: {}
• Pending: {}
• Running: {}
• Completed: {}
• Failed: {}

**Platforms:** {}
            "#,
            crate::VERSION,
            stats.max_concurrent,
            stats.available_permits,
            stats.pending + stats.running,
            stats.pending,
            stats.running,
            stats.completed,
            stats.failed,
            self.platforms.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        );

        TriggerResponseBuilder::new()
            .text(info_text.trim())
            .build()
    }

    /// Handle parse error
    async fn handle_parse_error(
        &self,
        _message: &TriggerMessage,
        error: CommandError,
    ) -> TriggerResponse {
        let text = match error {
            CommandError::InvalidFormat(msg) => {
                format!("Invalid command format: {}\n\nUse `/help` for usage.", msg)
            }
            CommandError::UnknownCommand(cmd) => {
                format!("Unknown command: {}\n\nUse `/help` for available commands.", cmd)
            }
            CommandError::MissingArgument(arg) => {
                format!("Missing required argument: {}\n\nUse `/help` for usage.", arg)
            }
            CommandError::InvalidTarget(target) => {
                format!("Invalid target: {}\n\nValid targets: agent, task, fleet, flow", target)
            }
        };

        TriggerResponseBuilder::new().text(text).error().build()
    }

    /// Increment user task count
    fn increment_user_tasks(&self, user_id: &str) {
        self.user_tasks
            .entry(user_id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    /// Format error for specific platform
    ///
    /// Provides platform-specific error formatting to enhance user experience
    fn format_error_for_platform(&self, platform: &str, error: &AofError) -> String {
        // Base error message
        let base_msg = match error {
            AofError::Agent(msg) => format!("Agent Error: {}", msg),
            AofError::Model(msg) => format!("Model Error: {}", msg),
            AofError::Tool(msg) => format!("Tool Error: {}", msg),
            AofError::Config(msg) => format!("Configuration Error: {}", msg),
            AofError::Timeout(msg) => format!("Timeout: {}", msg),
            AofError::InvalidState(msg) => format!("Invalid State: {}", msg),
            _ => format!("Error: {}", error),
        };

        // Platform-specific formatting
        match platform.to_lowercase().as_str() {
            "slack" => {
                // Slack uses markdown-style formatting
                format!("❌ *Error*\n```{}```", base_msg)
            }
            "discord" => {
                // Discord uses markdown with code blocks
                format!("❌ **Error**\n```\n{}\n```", base_msg)
            }
            "telegram" => {
                // Telegram supports markdown
                format!("❌ *Error*\n`{}`", base_msg)
            }
            "whatsapp" => {
                // WhatsApp has limited formatting
                format!("❌ Error: {}", base_msg)
            }
            _ => {
                // Generic formatting
                format!("❌ {}", base_msg)
            }
        }
    }

    /// Format success message for specific platform
    fn format_success_for_platform(&self, platform: &str, message: &str) -> String {
        match platform.to_lowercase().as_str() {
            "slack" => format!("✅ *Success*\n{}", message),
            "discord" => format!("✅ **Success**\n{}", message),
            "telegram" => format!("✅ *Success*\n{}", message),
            _ => format!("✅ {}", message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        let orchestrator = Arc::new(RuntimeOrchestrator::new());
        let handler = TriggerHandler::new(orchestrator);

        assert_eq!(handler.platforms.len(), 0);
        assert!(handler.config.auto_ack);
    }
}
