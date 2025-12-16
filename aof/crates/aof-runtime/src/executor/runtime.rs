//! Runtime - Top-level runtime coordinator
//!
//! The Runtime loads agent configurations, creates models, tools, and memory,
//! and executes agents with proper lifecycle management.

use super::{AgentExecutor, agent_executor::StreamEvent};
use aof_core::{
    AgentConfig, AgentContext, AofError, AofResult, McpServerConfig, McpTransport,
    ModelConfig, ModelProvider, Tool, ToolDefinition, ToolExecutor, ToolInput, ToolSpec,
};
use aof_llm::create_model;
use aof_mcp::McpClientBuilder;
use aof_memory::{InMemoryBackend, SimpleMemory};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Top-level runtime for agent execution
///
/// The Runtime coordinates all aspects of agent execution:
/// - Loading agent configurations
/// - Creating and managing models
/// - Setting up tool executors
/// - Managing memory backends
/// - Executing agents with proper lifecycle management
pub struct Runtime {
    /// Loaded agents
    agents: HashMap<String, Arc<AgentExecutor>>,
}

impl Runtime {
    /// Create a new runtime instance
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Load an agent from YAML configuration file
    ///
    /// # Arguments
    /// * `config_path` - Path to the YAML configuration file
    ///
    /// # Returns
    /// The agent name for later execution
    pub async fn load_agent_from_file(&mut self, config_path: &str) -> AofResult<String> {
        info!("Loading agent from config file: {}", config_path);

        // Read and parse YAML config
        let config_content = tokio::fs::read_to_string(config_path).await.map_err(|e| {
            AofError::config(format!("Failed to read config file {}: {}", config_path, e))
        })?;

        let config: AgentConfig = serde_yaml::from_str(&config_content).map_err(|e| {
            AofError::config(format!("Failed to parse YAML config: {}", e))
        })?;

        self.load_agent_from_config(config).await
    }

    /// Load an agent from configuration struct
    ///
    /// # Arguments
    /// * `config` - Agent configuration
    ///
    /// # Returns
    /// The agent name for later execution
    pub async fn load_agent_from_config(&mut self, config: AgentConfig) -> AofResult<String> {
        let agent_name = config.name.clone();
        info!("Loading agent: {}", agent_name);

        // Create model from config
        let model_config = self.create_model_config(&config)?;
        let model = create_model(model_config).await?;
        debug!("Model created for agent: {}", agent_name);

        // Create tool executor
        // Priority: mcp_servers > tools (legacy)
        let tool_executor: Option<Arc<dyn ToolExecutor>> = if !config.mcp_servers.is_empty() {
            // Use the new flexible MCP configuration
            Some(self.create_mcp_executor_from_config(&config.mcp_servers).await?)
        } else if !config.tools.is_empty() {
            // Separate built-in tools from MCP tools
            let builtin_tools: Vec<&str> = config.tools.iter()
                .filter(|t| t.is_builtin())
                .map(|t| t.name())
                .collect();
            let mcp_tools: Vec<&str> = config.tools.iter()
                .filter(|t| t.is_mcp())
                .map(|t| t.name())
                .collect();

            // Known system/builtin tools
            let system_tools = ["shell", "kubectl", "bash", "sh", "python", "node",
                               "read_file", "write_file", "list_directory", "search_files",
                               "kubectl_get", "kubectl_apply", "kubectl_delete", "kubectl_logs",
                               "kubectl_exec", "kubectl_describe", "docker_ps", "docker_logs",
                               "docker_build", "docker_run", "docker_exec", "docker_images",
                               "git_status", "git_diff", "git_log", "git_commit", "git_branch",
                               "git_checkout", "git_pull", "git_push", "terraform_init",
                               "terraform_plan", "terraform_apply", "terraform_destroy",
                               "terraform_output", "prometheus_query", "loki_query",
                               "elasticsearch_query", "victoriametrics_query", "aws_s3",
                               "aws_ec2", "aws_logs", "aws_iam", "aws_lambda", "aws_ecs"];

            let has_system_tools = builtin_tools.iter().any(|t| system_tools.contains(t));
            let has_mcp_tools = !mcp_tools.is_empty();

            if has_system_tools && !has_mcp_tools {
                debug!("Agent has only built-in tools, creating system executor");
                let tool_names: Vec<String> = builtin_tools.iter().map(|s| s.to_string()).collect();
                Some(self.create_system_executor(&tool_names)?)
            } else if has_mcp_tools {
                let tool_names: Vec<String> = config.tool_names().iter().map(|s| s.to_string()).collect();
                Some(self.create_tool_executor(&tool_names).await?)
            } else {
                None
            }
        } else {
            None
        };

        // Create memory backend
        let memory = self.create_memory(&config)?;
        debug!("Memory backend created for agent: {}", agent_name);

        // Create agent executor
        let executor = AgentExecutor::new(config, model, tool_executor, Some(memory));

        self.agents.insert(agent_name.clone(), Arc::new(executor));
        info!("Agent loaded successfully: {}", agent_name);

        Ok(agent_name)
    }

    /// Execute an agent with the given input
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute(&self, agent_name: &str, input: &str) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);
        executor.execute(&mut context).await
    }

    /// Execute an agent with a pre-built context
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `context` - Pre-configured agent context
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute_with_context(
        &self,
        agent_name: &str,
        context: &mut AgentContext,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        executor.execute(context).await
    }

    /// Execute an agent with streaming support for real-time updates
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    /// * `stream_tx` - Channel sender for streaming events
    ///
    /// # Returns
    /// The agent's final response
    ///
    /// # Example
    /// ```no_run
    /// use tokio::sync::mpsc;
    /// # use aof_runtime::Runtime;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runtime = Runtime::new();
    /// runtime.load_agent_from_file("config.yaml").await?;
    ///
    /// let (tx, mut rx) = mpsc::channel(100);
    ///
    /// // Spawn task to handle stream events
    /// tokio::spawn(async move {
    ///     while let Some(event) = rx.recv().await {
    ///         println!("Event: {:?}", event);
    ///     }
    /// });
    ///
    /// let result = runtime.execute_streaming("my-agent", "Hello", tx).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_streaming(
        &self,
        agent_name: &str,
        input: &str,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);
        executor.execute_streaming(&mut context, stream_tx).await
    }

    /// Execute an agent with streaming and a pre-built context
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `context` - Pre-configured agent context
    /// * `stream_tx` - Channel sender for streaming events
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute_streaming_with_context(
        &self,
        agent_name: &str,
        context: &mut AgentContext,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        executor.execute_streaming(context, stream_tx).await
    }

    /// Execute an agent with streaming and cancellation support
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    /// * `stream_tx` - Channel sender for streaming events
    /// * `cancel_rx` - Channel receiver for cancellation signal
    ///
    /// # Returns
    /// The agent's final response or cancellation error
    ///
    /// # Example
    /// ```no_run
    /// use tokio::sync::{mpsc, oneshot};
    /// # use aof_runtime::Runtime;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runtime = Runtime::new();
    /// runtime.load_agent_from_file("config.yaml").await?;
    ///
    /// let (stream_tx, mut stream_rx) = mpsc::channel(100);
    /// let (cancel_tx, cancel_rx) = oneshot::channel();
    ///
    /// // Spawn task to handle cancellation
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    ///     let _ = cancel_tx.send(());
    /// });
    ///
    /// let result = runtime.execute_streaming_cancellable(
    ///     "my-agent",
    ///     "Long running task",
    ///     stream_tx,
    ///     cancel_rx
    /// ).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_streaming_cancellable(
        &self,
        agent_name: &str,
        input: &str,
        stream_tx: mpsc::Sender<StreamEvent>,
        mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);

        tokio::select! {
            result = executor.execute_streaming(&mut context, stream_tx.clone()) => {
                result
            }
            _ = &mut cancel_rx => {
                let _ = stream_tx.send(StreamEvent::Error {
                    message: "Execution cancelled by user".to_string(),
                }).await;
                Err(AofError::agent("Execution cancelled".to_string()))
            }
        }
    }

    /// List all loaded agents
    pub fn list_agents(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get agent executor by name
    pub fn get_agent(&self, name: &str) -> Option<Arc<AgentExecutor>> {
        self.agents.get(name).cloned()
    }

    // Helper: Create model config from agent config
    fn create_model_config(&self, config: &AgentConfig) -> AofResult<ModelConfig> {
        // Parse provider from model string (format: "provider:model") or separate provider field
        let (provider, model) = if config.model.contains(':') {
            // Format: "google:gemini-2.0-flash" or "openai:gpt-4"
            let parts: Vec<&str> = config.model.splitn(2, ':').collect();
            let provider = Self::parse_provider(parts[0]);
            (provider, parts[1].to_string())
        } else if let Some(ref provider_str) = config.provider {
            // Separate provider field: provider: google, model: gemini-2.0-flash
            let provider = Self::parse_provider(provider_str);
            (provider, config.model.clone())
        } else {
            // Default to Anthropic if no provider specified
            (ModelProvider::Anthropic, config.model.clone())
        };

        Ok(ModelConfig {
            model,
            provider,
            api_key: None, // Will use environment variables
            endpoint: None,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        })
    }

    // Helper: Parse provider string to ModelProvider enum
    fn parse_provider(provider_str: &str) -> ModelProvider {
        match provider_str.to_lowercase().as_str() {
            "anthropic" | "claude" => ModelProvider::Anthropic,
            "openai" | "gpt" => ModelProvider::OpenAI,
            "google" | "gemini" => ModelProvider::Google,
            "bedrock" | "aws" => ModelProvider::Bedrock,
            "azure" => ModelProvider::Azure,
            "ollama" => ModelProvider::Ollama,
            "groq" => ModelProvider::Groq,
            _ => ModelProvider::Custom,
        }
    }

    // Helper: Create tool executor from tool list (legacy)
    async fn create_tool_executor(
        &self,
        tool_names: &[String],
    ) -> AofResult<Arc<dyn ToolExecutor>> {
        info!("Creating tool executor with {} tools (legacy mode)", tool_names.len());

        // Find smoke-test-mcp binary in standard locations
        let mcp_path = if std::path::Path::new("/usr/local/bin/smoke-test-mcp").exists() {
            "/usr/local/bin/smoke-test-mcp".to_string()
        } else if std::path::Path::new("/usr/bin/smoke-test-mcp").exists() {
            "/usr/bin/smoke-test-mcp".to_string()
        } else {
            // Fallback to relative path for development
            "./target/release/smoke-test-mcp".to_string()
        };

        let mcp_client = McpClientBuilder::new()
            .stdio(
                mcp_path,
                vec![],
            )
            .build()
            .map_err(|e| AofError::tool(format!("Failed to create MCP client: {}", e)))?;

        // Initialize the MCP client
        mcp_client.initialize()
            .await
            .map_err(|e| AofError::tool(format!("Failed to initialize MCP client: {}", e)))?;

        info!("MCP client initialized successfully with tools: {:?}", tool_names);

        Ok(Arc::new(McpToolExecutor {
            client: Arc::new(mcp_client),
            tool_names: tool_names.to_vec(),
        }))
    }

    // Helper: Create MCP executor from flexible config
    async fn create_mcp_executor_from_config(
        &self,
        mcp_servers: &[McpServerConfig],
    ) -> AofResult<Arc<dyn ToolExecutor>> {
        info!("Creating MCP executor from {} server configs", mcp_servers.len());

        let mut clients: Vec<Arc<aof_mcp::McpClient>> = Vec::new();
        let mut all_tool_names: Vec<String> = Vec::new();

        for server_config in mcp_servers {
            // Validate the config
            if let Err(e) = server_config.validate() {
                warn!("Invalid MCP server config '{}': {}", server_config.name, e);
                continue;
            }

            info!("Initializing MCP server: {} ({:?})", server_config.name, server_config.transport);

            let mcp_client = match server_config.transport {
                McpTransport::Stdio => {
                    let command = server_config.command.as_ref()
                        .ok_or_else(|| AofError::config("Stdio transport requires command"))?;

                    let mut builder = McpClientBuilder::new()
                        .stdio(command.clone(), server_config.args.clone());

                    // Add environment variables
                    for (key, value) in &server_config.env {
                        builder = builder.with_env(key.clone(), value.clone());
                    }

                    builder.build()
                        .map_err(|e| AofError::tool(format!(
                            "Failed to create MCP client for '{}': {}", server_config.name, e
                        )))?
                }
                #[cfg(feature = "sse")]
                McpTransport::Sse => {
                    let endpoint = server_config.endpoint.as_ref()
                        .ok_or_else(|| AofError::config("SSE transport requires endpoint"))?;

                    McpClientBuilder::new()
                        .sse(endpoint.clone())
                        .build()
                        .map_err(|e| AofError::tool(format!(
                            "Failed to create SSE MCP client for '{}': {}", server_config.name, e
                        )))?
                }
                #[cfg(feature = "http")]
                McpTransport::Http => {
                    let endpoint = server_config.endpoint.as_ref()
                        .ok_or_else(|| AofError::config("HTTP transport requires endpoint"))?;

                    McpClientBuilder::new()
                        .http(endpoint.clone())
                        .build()
                        .map_err(|e| AofError::tool(format!(
                            "Failed to create HTTP MCP client for '{}': {}", server_config.name, e
                        )))?
                }
                #[cfg(not(feature = "sse"))]
                McpTransport::Sse => {
                    return Err(AofError::config(
                        "SSE transport not enabled. Enable the 'sse' feature in aof-mcp"
                    ));
                }
                #[cfg(not(feature = "http"))]
                McpTransport::Http => {
                    return Err(AofError::config(
                        "HTTP transport not enabled. Enable the 'http' feature in aof-mcp"
                    ));
                }
            };

            // Initialize the client with optional init_options
            match mcp_client.initialize_with_options(server_config.init_options.clone()).await {
                Ok(_) => {
                    info!("MCP server '{}' initialized successfully", server_config.name);

                    // Get tool list from the server
                    if let Ok(tools) = mcp_client.list_tools().await {
                        for tool in tools {
                            // Apply tool filter if specified
                            if server_config.tools.is_empty() || server_config.tools.contains(&tool.name) {
                                all_tool_names.push(tool.name);
                            }
                        }
                    }

                    clients.push(Arc::new(mcp_client));
                }
                Err(e) => {
                    warn!("Failed to initialize MCP server '{}': {}", server_config.name, e);
                    if !server_config.auto_reconnect {
                        return Err(AofError::tool(format!(
                            "MCP server '{}' initialization failed: {}", server_config.name, e
                        )));
                    }
                }
            }
        }

        if clients.is_empty() {
            return Err(AofError::tool("No MCP servers could be initialized"));
        }

        info!("MCP executor created with {} servers and {} tools", clients.len(), all_tool_names.len());

        Ok(Arc::new(MultiMcpToolExecutor {
            clients,
            tool_names: all_tool_names,
        }))
    }

    // Helper: Create system tool executor for shell/kubectl commands
    fn create_system_executor(
        &self,
        tool_names: &[String],
    ) -> AofResult<Arc<dyn ToolExecutor>> {
        info!("Creating system tool executor with {} tools", tool_names.len());
        Ok(Arc::new(SystemToolExecutor {
            tool_names: tool_names.to_vec(),
        }))
    }

    // Helper: Create memory backend
    fn create_memory(&self, _config: &AgentConfig) -> AofResult<Arc<SimpleMemory>> {
        let backend = InMemoryBackend::new();
        Ok(Arc::new(SimpleMemory::new(Arc::new(backend))))
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP-based tool executor implementation
struct McpToolExecutor {
    client: Arc<aof_mcp::McpClient>,
    tool_names: Vec<String>,
}

#[async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
    ) -> AofResult<aof_core::ToolResult> {
        debug!("Executing MCP tool: {}", name);
        let start = std::time::Instant::now();

        // Call MCP tool
        let result = self
            .client
            .call_tool(name, input.arguments)
            .await
            .map_err(|e| AofError::tool(format!("MCP tool call failed: {}", e)))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(aof_core::ToolResult {
            success: true,
            data: result,
            error: None,
            execution_time_ms,
        })
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        // In a real implementation, this would query MCP for tool definitions
        // For now, return basic definitions
        self.tool_names
            .iter()
            .map(|name| ToolDefinition {
                name: name.clone(),
                description: format!("MCP tool: {}", name),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                }),
            })
            .collect()
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn Tool>> {
        // MCP tools are dynamically resolved, not stored as objects
        None
    }
}

/// Multi-server MCP tool executor
/// Supports multiple MCP servers with different transports
struct MultiMcpToolExecutor {
    clients: Vec<Arc<aof_mcp::McpClient>>,
    tool_names: Vec<String>,
}

#[async_trait]
impl ToolExecutor for MultiMcpToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
    ) -> AofResult<aof_core::ToolResult> {
        debug!("Executing MCP tool (multi-server): {}", name);
        let start = std::time::Instant::now();

        // Try each client until one succeeds
        let mut last_error = None;
        for client in &self.clients {
            match client.call_tool(name, input.arguments.clone()).await {
                Ok(result) => {
                    let execution_time_ms = start.elapsed().as_millis() as u64;
                    return Ok(aof_core::ToolResult {
                        success: true,
                        data: result,
                        error: None,
                        execution_time_ms,
                    });
                }
                Err(e) => {
                    debug!("Tool '{}' not found on server, trying next: {}", name, e);
                    last_error = Some(e);
                }
            }
        }

        // All clients failed
        let execution_time_ms = start.elapsed().as_millis() as u64;
        let error_msg = last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| format!("Tool '{}' not found on any MCP server", name));

        Ok(aof_core::ToolResult {
            success: false,
            data: serde_json::json!({}),
            error: Some(error_msg),
            execution_time_ms,
        })
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tool_names
            .iter()
            .map(|name| ToolDefinition {
                name: name.clone(),
                description: format!("MCP tool: {}", name),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                }),
            })
            .collect()
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn Tool>> {
        // MCP tools are dynamically resolved, not stored as objects
        None
    }
}

/// System tool executor for shell, kubectl, and other local commands
struct SystemToolExecutor {
    tool_names: Vec<String>,
}

#[async_trait]
impl ToolExecutor for SystemToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
    ) -> AofResult<aof_core::ToolResult> {
        debug!("Executing system tool: {}", name);
        let start = std::time::Instant::now();

        // Extract command from input arguments
        let command = if let Some(serde_json::Value::String(cmd)) = input.arguments.get("command") {
            cmd.clone()
        } else if let Some(serde_json::Value::String(cmd)) = input.arguments.get("_") {
            // Fallback for positional argument
            cmd.clone()
        } else {
            return Err(AofError::tool(format!(
                "Tool {} requires 'command' argument",
                name
            )));
        };

        // Execute the command based on tool type
        let result = match name {
            "shell" | "bash" | "sh" => {
                // Execute shell command
                self.execute_shell_command(&command).await
            }
            "kubectl" => {
                // Execute kubectl command
                self.execute_kubectl_command(&command).await
            }
            "python" => {
                // Execute Python code
                self.execute_command("python3", &[&command]).await
            }
            "node" => {
                // Execute Node.js code
                self.execute_command("node", &["-e", &command]).await
            }
            _ => Err(AofError::tool(format!(
                "Unknown system tool: {}",
                name
            ))),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(aof_core::ToolResult {
                success: true,
                data,
                error: None,
                execution_time_ms,
            }),
            Err(e) => Ok(aof_core::ToolResult {
                success: false,
                data: serde_json::json!({}),
                error: Some(e.to_string()),
                execution_time_ms,
            }),
        }
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tool_names
            .iter()
            .map(|name| {
                let (description, parameters) = match name.as_str() {
                    "shell" | "bash" | "sh" => (
                        "Execute shell commands".to_string(),
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "Shell command to execute"
                                }
                            },
                            "required": ["command"]
                        }),
                    ),
                    "kubectl" => (
                        "Execute kubectl commands against Kubernetes cluster".to_string(),
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "kubectl command to execute (e.g., 'get pods', 'describe node')"
                                }
                            },
                            "required": ["command"]
                        }),
                    ),
                    "python" => (
                        "Execute Python code".to_string(),
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "Python code to execute"
                                }
                            },
                            "required": ["command"]
                        }),
                    ),
                    "node" => (
                        "Execute Node.js code".to_string(),
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "JavaScript code to execute"
                                }
                            },
                            "required": ["command"]
                        }),
                    ),
                    _ => (
                        format!("System tool: {}", name),
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "Command to execute"
                                }
                            }
                        }),
                    ),
                };

                ToolDefinition {
                    name: name.clone(),
                    description,
                    parameters,
                }
            })
            .collect()
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn Tool>> {
        // System tools are executed directly, not stored as objects
        None
    }
}

impl SystemToolExecutor {
    async fn execute_shell_command(&self, command: &str) -> AofResult<serde_json::Value> {
        self.execute_command("sh", &["-c", command]).await
    }

    async fn execute_kubectl_command(&self, command: &str) -> AofResult<serde_json::Value> {
        // Parse kubectl command
        let args: Vec<&str> = command.split_whitespace().collect();
        self.execute_command("kubectl", &args).await
    }

    async fn execute_command(
        &self,
        program: &str,
        args: &[&str],
    ) -> AofResult<serde_json::Value> {
        debug!(
            "Executing command: {} {}",
            program,
            args.join(" ")
        );

        let output = tokio::process::Command::new(program)
            .args(args)
            .output()
            .await
            .map_err(|e| AofError::tool(format!(
                "Failed to execute {}: {}",
                program, e
            )))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        debug!(
            "Command exit code: {}, stdout: {}, stderr: {}",
            exit_code, stdout, stderr
        );

        Ok(serde_json::json!({
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "success": output.status.success()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert_eq!(runtime.list_agents().len(), 0);
    }

    #[test]
    fn test_model_config_parsing() {
        let runtime = Runtime::new();

        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "anthropic:claude-3-5-sonnet-20241022".to_string(),
            provider: None,
            tools: vec![],
            mcp_servers: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        let model_config = runtime.create_model_config(&config).unwrap();
        assert_eq!(model_config.provider, ModelProvider::Anthropic);
        assert_eq!(model_config.model, "claude-3-5-sonnet-20241022");
        assert_eq!(model_config.temperature, 0.7);
    }

    #[test]
    fn test_model_config_default_provider() {
        let runtime = Runtime::new();

        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "gpt-4".to_string(),
            provider: None,
            tools: vec![],
            mcp_servers: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        let model_config = runtime.create_model_config(&config).unwrap();
        assert_eq!(model_config.provider, ModelProvider::Anthropic);
        assert_eq!(model_config.model, "gpt-4");
    }
}
