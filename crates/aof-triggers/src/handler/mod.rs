//! Central message handler for routing and execution
//!
//! This module coordinates message handling across platforms,
//! parsing commands, and executing them through the runtime.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::command::{CommandError, CommandType, TriggerCommand, TriggerTarget};
use crate::flow::{FlowRegistry, FlowRouter, FlowMatch};
use crate::platforms::{TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::{Action, ActionStyle, TriggerResponse, TriggerResponseBuilder};
use aof_core::{AgentContext, AofError, AofResult};
use aof_runtime::{Runtime, RuntimeOrchestrator, Task, TaskStatus, AgentFlowExecutor};

/// Pending approval request for human-in-the-loop workflow
#[derive(Debug, Clone)]
pub struct PendingApproval {
    /// The command to execute after approval
    pub command: String,
    /// User who requested the command
    pub user_id: String,
    /// Channel where the request was made
    pub channel_id: String,
    /// Message timestamp (used for thread replies)
    pub message_ts: String,
    /// Timestamp when approval was requested
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// Agent name to use for execution
    pub agent_name: String,
    /// Original user message for context
    pub original_message: String,
}

/// Parse agent output for approval-related fields
fn parse_approval_output(output: &str) -> (bool, Option<String>, String) {
    // Look for requires_approval: true and command: "..."
    let requires_approval = output.contains("requires_approval: true")
        || output.contains("requires_approval:true");

    // Extract command using regex
    let command = regex::Regex::new(r#"command:\s*["\']?([^"\'\n]+)["\']?"#)
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string());

    // Get the text content (everything before the approval fields)
    let clean_output = output
        .lines()
        .filter(|line| {
            !line.contains("requires_approval:") && !line.contains("command:")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    (requires_approval, command, clean_output)
}

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

    /// Default agent for natural language messages (non-command messages)
    pub default_agent: Option<String>,
}

impl Default for TriggerHandlerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            auto_ack: true,
            max_tasks_per_user: 3,
            command_timeout_secs: 300, // 5 minutes
            default_agent: None,
        }
    }
}

/// Conversation memory entry for maintaining context across messages
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    /// Message content
    pub content: String,
    /// Role (user or assistant)
    pub role: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
    user_tasks: Arc<DashMap<String, usize>>,

    /// Flow router for AgentFlow-based message routing
    flow_router: Option<Arc<FlowRouter>>,

    /// Runtime for agent execution (shared with AgentFlowExecutor)
    runtime: Arc<RwLock<Runtime>>,

    /// Agents directory for loading agent configs
    agents_dir: Option<PathBuf>,

    /// Pending approvals (message_ts -> PendingApproval)
    pending_approvals: Arc<DashMap<String, PendingApproval>>,

    /// Conversation memory per channel/thread (channel_id:thread_id -> messages)
    /// Maintains conversation context for natural language interactions
    conversation_memory: Arc<DashMap<String, Vec<ConversationEntry>>>,

    /// User context sessions (user_id -> context name)
    /// Tracks which context each user has selected for their session
    /// Context = Agent + Connection Parameters (replaces both agent and env sessions)
    user_context_sessions: Arc<DashMap<String, String>>,

    /// Available contexts (name -> config)
    /// Configured contexts that users can switch between
    /// Each context bundles: agent, connection params, env vars, tools
    available_contexts: Arc<DashMap<String, ContextConfig>>,
}

/// Context configuration bundling agent + connection + environment
/// Context = Agent + Connection Parameters
/// Replaces separate EnvironmentConfig and agent sessions
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Display name (e.g., "Cluster A (EKS)", "AWS Dev Account")
    pub display_name: String,
    /// Emoji for visual identification
    pub emoji: String,
    /// Description of what this context connects to
    pub description: String,

    // Connection parameters
    /// Kubernetes kubeconfig path
    pub kubeconfig: Option<String>,
    /// Kubernetes context name
    pub kubecontext: Option<String>,
    /// Default namespace
    pub namespace: Option<String>,
    /// AWS profile
    pub aws_profile: Option<String>,
    /// AWS region
    pub aws_region: Option<String>,

    /// Agent reference (e.g., "k8s-readonly", "aws-readonly")
    /// This agent will be used when this context is active
    pub agent_ref: Option<String>,

    /// Tools available in this context
    pub tools: Vec<String>,

    /// Environment variables set when this context is active
    pub env: std::collections::HashMap<String, String>,

    /// Read-only mode - blocks write/delete/dangerous operations
    /// Default: true for mobile platforms (Telegram, WhatsApp), false for CLI/Slack
    pub read_only: bool,
}

/// Simple write operation detection for MVP safety layer
/// Returns true if the input looks like a write/delete/dangerous operation
fn is_write_operation(input: &str) -> bool {
    let input_lower = input.to_lowercase();

    // kubectl write operations
    let kubectl_writes = [
        "kubectl apply", "kubectl create", "kubectl delete", "kubectl patch",
        "kubectl edit", "kubectl replace", "kubectl set", "kubectl scale",
        "kubectl rollout", "kubectl drain", "kubectl cordon", "kubectl taint",
        "kubectl label", "kubectl annotate", "kubectl expose",
    ];

    // docker write operations
    let docker_writes = [
        "docker rm", "docker rmi", "docker stop", "docker kill", "docker prune",
        "docker push", "docker build", "docker run", "docker exec",
    ];

    // helm write operations
    let helm_writes = [
        "helm install", "helm upgrade", "helm delete", "helm uninstall",
        "helm rollback",
    ];

    // terraform write operations
    let terraform_writes = [
        "terraform apply", "terraform destroy", "terraform import",
    ];

    // aws write operations
    let aws_writes = [
        "aws ec2 terminate", "aws ec2 stop", "aws ec2 start", "aws ec2 run",
        "aws s3 rm", "aws s3 cp", "aws s3 mv", "aws s3 sync",
        "aws ecs update", "aws ecs delete", "aws lambda delete",
    ];

    // git write operations
    let git_writes = [
        "git push", "git commit", "git reset", "git revert", "git merge",
        "git rebase", "git checkout", "git branch -d", "git branch -D",
    ];

    // Generic dangerous patterns
    let dangerous = [
        "rm -rf", "rm -r", "rmdir", "drop database", "truncate table",
        "delete from", "update ", "insert into",
    ];

    // Natural language write intents
    let nl_writes = [
        "create ", "deploy ", "delete ", "remove ", "scale ", "restart ",
        "update ", "apply ", "install ", "uninstall ", "rollback ",
        "push ", "commit ", "terminate ", "stop ", "kill ",
    ];

    // Check all patterns
    for pattern in kubectl_writes.iter()
        .chain(docker_writes.iter())
        .chain(helm_writes.iter())
        .chain(terraform_writes.iter())
        .chain(aws_writes.iter())
        .chain(git_writes.iter())
        .chain(dangerous.iter())
        .chain(nl_writes.iter())
    {
        if input_lower.contains(pattern) {
            return true;
        }
    }

    false
}

impl TriggerHandler {
    /// Create a new trigger handler
    pub fn new(orchestrator: Arc<RuntimeOrchestrator>) -> Self {
        let handler = Self {
            orchestrator,
            platforms: HashMap::new(),
            config: TriggerHandlerConfig::default(),
            user_tasks: Arc::new(DashMap::new()),
            flow_router: None,
            runtime: Arc::new(RwLock::new(Runtime::new())),
            agents_dir: None,
            pending_approvals: Arc::new(DashMap::new()),
            conversation_memory: Arc::new(DashMap::new()),
            user_context_sessions: Arc::new(DashMap::new()),
            available_contexts: Arc::new(DashMap::new()),
        };
        handler.init_default_contexts();
        handler
    }

    /// Create handler with custom configuration
    pub fn with_config(orchestrator: Arc<RuntimeOrchestrator>, config: TriggerHandlerConfig) -> Self {
        let handler = Self {
            orchestrator,
            platforms: HashMap::new(),
            config,
            user_tasks: Arc::new(DashMap::new()),
            flow_router: None,
            runtime: Arc::new(RwLock::new(Runtime::new())),
            agents_dir: None,
            pending_approvals: Arc::new(DashMap::new()),
            conversation_memory: Arc::new(DashMap::new()),
            user_context_sessions: Arc::new(DashMap::new()),
            available_contexts: Arc::new(DashMap::new()),
        };
        handler.init_default_contexts();
        handler
    }

    /// Initialize default contexts
    /// Each context bundles: agent + connection params + env vars
    /// Agent refs must match metadata.name in examples/agents/*.yaml
    fn init_default_contexts(&self) {
        // Development K8s context (full stack with Prometheus, Loki, Argo, AWS)
        // This is the recommended default for testing
        // Uses "devops" agent which has: kubectl, docker, helm, terraform, git, shell
        // read_only: true by default - write ops blocked on Telegram
        self.available_contexts.insert("dev-k8s".to_string(), ContextConfig {
            display_name: "Dev K8s (Full Stack)".to_string(),
            emoji: "🚀".to_string(),
            description: "Dev K8s with Prometheus, Loki, Argo, AWS".to_string(),
            kubeconfig: Some("~/.kube/config".to_string()),
            kubecontext: Some("dev-cluster".to_string()),
            namespace: Some("default".to_string()),
            aws_profile: Some("development".to_string()),
            aws_region: Some("us-west-2".to_string()),
            agent_ref: Some("devops".to_string()),  // examples/agents/devops.yaml
            tools: vec![
                "kubectl".to_string(),
                "helm".to_string(),
                "docker".to_string(),
                "git".to_string(),
                "aws".to_string(),
                "prometheus_query".to_string(),
            ],
            env: [
                ("KUBECONFIG".to_string(), "~/.kube/config".to_string()),
                ("AWS_PROFILE".to_string(), "development".to_string()),
                ("AWS_REGION".to_string(), "us-west-2".to_string()),
            ].into_iter().collect(),
            read_only: true,  // MVP safety: block writes on Telegram
        });

        // Kubernetes Cluster A context (production EKS)
        // Uses "k8s-ops" agent which has: kubectl, helm
        self.available_contexts.insert("cluster-a".to_string(), ContextConfig {
            display_name: "Cluster A (EKS)".to_string(),
            emoji: "🔷".to_string(),
            description: "Production EKS cluster in us-east-1".to_string(),
            kubeconfig: Some("~/.kube/config".to_string()),
            kubecontext: Some("cluster-a-prod".to_string()),
            namespace: Some("default".to_string()),
            aws_profile: Some("production".to_string()),
            aws_region: Some("us-east-1".to_string()),
            agent_ref: Some("k8s-ops".to_string()),  // examples/agents/k8s-ops.yaml
            tools: vec!["kubectl".to_string(), "helm".to_string()],
            env: [
                ("KUBECONFIG".to_string(), "~/.kube/config".to_string()),
                ("AWS_PROFILE".to_string(), "production".to_string()),
            ].into_iter().collect(),
            read_only: true,  // Production: always read-only from Telegram
        });

        // AWS Dev Account context
        // Uses "aws-agent" which has: aws, shell, read_file, write_file
        self.available_contexts.insert("aws-dev".to_string(), ContextConfig {
            display_name: "AWS Dev Account".to_string(),
            emoji: "☁️".to_string(),
            description: "AWS development account".to_string(),
            kubeconfig: None,
            kubecontext: None,
            namespace: None,
            aws_profile: Some("development".to_string()),
            aws_region: Some("us-west-2".to_string()),
            agent_ref: Some("aws-agent".to_string()),  // examples/agents/aws-agent.yaml
            tools: vec!["aws".to_string(), "terraform".to_string()],
            env: [
                ("AWS_PROFILE".to_string(), "development".to_string()),
                ("AWS_REGION".to_string(), "us-west-2".to_string()),
            ].into_iter().collect(),
            read_only: true,  // MVP safety: block writes on Telegram
        });

        // Database context - uses general assistant (no specific db agent yet)
        self.available_contexts.insert("database".to_string(), ContextConfig {
            display_name: "PostgreSQL".to_string(),
            emoji: "🗄️".to_string(),
            description: "Production database (read-only)".to_string(),
            kubeconfig: None,
            kubecontext: None,
            namespace: None,
            aws_profile: None,
            aws_region: None,
            agent_ref: Some("assistant".to_string()),  // examples/agents/assistant.yaml
            tools: vec!["psql".to_string()],
            env: std::collections::HashMap::new(),
            read_only: true,  // Database: always read-only
        });

        // Prometheus/Monitoring context
        // Uses "sre-agent" which has: prometheus_query, loki_query, kubectl, read_file, list_directory
        self.available_contexts.insert("prometheus".to_string(), ContextConfig {
            display_name: "Prometheus".to_string(),
            emoji: "📊".to_string(),
            description: "Monitoring and metrics".to_string(),
            kubeconfig: None,
            kubecontext: None,
            namespace: None,
            aws_profile: None,
            aws_region: None,
            agent_ref: Some("sre-agent".to_string()),  // examples/agents/sre-agent.yaml
            tools: vec!["prometheus_query".to_string(), "loki_query".to_string()],
            env: std::collections::HashMap::new(),
            read_only: true,  // Monitoring: read-only by nature
        });
    }

    /// Get pending approvals (for external access like reaction handlers)
    pub fn pending_approvals(&self) -> Arc<DashMap<String, PendingApproval>> {
        self.pending_approvals.clone()
    }

    /// Get conversation key for a channel/thread combination
    fn get_conversation_key(channel_id: &str, thread_id: Option<&str>) -> String {
        match thread_id {
            Some(tid) => format!("{}:{}", channel_id, tid),
            None => channel_id.to_string(),
        }
    }

    /// Add a message to conversation memory
    fn add_to_conversation(&self, channel_id: &str, thread_id: Option<&str>, role: &str, content: &str) {
        let key = Self::get_conversation_key(channel_id, thread_id);
        let entry = ConversationEntry {
            content: content.to_string(),
            role: role.to_string(),
            timestamp: chrono::Utc::now(),
        };

        self.conversation_memory
            .entry(key)
            .and_modify(|messages| {
                // Keep last 20 messages to avoid memory bloat
                if messages.len() >= 20 {
                    messages.remove(0);
                }
                messages.push(entry.clone());
            })
            .or_insert_with(|| vec![entry]);
    }

    /// Get conversation history for context
    fn get_conversation_history(&self, channel_id: &str, thread_id: Option<&str>) -> Vec<ConversationEntry> {
        let key = Self::get_conversation_key(channel_id, thread_id);
        self.conversation_memory
            .get(&key)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Format conversation history as context for the LLM
    fn format_conversation_context(&self, channel_id: &str, thread_id: Option<&str>) -> String {
        let history = self.get_conversation_history(channel_id, thread_id);
        if history.is_empty() {
            return String::new();
        }

        // Use clear format that helps LLM understand this is previous context
        let mut context = String::from(
            "[CONVERSATION HISTORY - Use this to understand references like 'it', 'that', 'the deployment']\n\n"
        );

        // Get last 10 messages for context
        let recent: Vec<_> = history.iter().rev().take(10).collect();
        for entry in recent.into_iter().rev() {
            let role_label = if entry.role == "user" { "User" } else { "Assistant" };
            // Truncate long messages in context
            let content = if entry.content.len() > 500 {
                format!("{}...", &entry.content[..500])
            } else {
                entry.content.clone()
            };
            context.push_str(&format!("{}: {}\n\n", role_label, content));
        }
        context.push_str("[END CONVERSATION HISTORY]\n\n[CURRENT USER MESSAGE]\n");
        context
    }

    /// Set flow router for AgentFlow-based routing
    pub fn with_flow_router(mut self, router: Arc<FlowRouter>) -> Self {
        self.flow_router = Some(router);
        self
    }

    /// Set flow router (mutable)
    pub fn set_flow_router(&mut self, router: Arc<FlowRouter>) {
        self.flow_router = Some(router);
    }

    /// Set agents directory for loading agent configs
    pub fn with_agents_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.agents_dir = Some(dir.into());
        self
    }

    /// Set agents directory (mutable)
    pub fn set_agents_dir(&mut self, dir: impl Into<PathBuf>) {
        self.agents_dir = Some(dir.into());
    }

    /// Set runtime for agent execution
    pub fn set_runtime(&mut self, runtime: Arc<RwLock<Runtime>>) {
        self.runtime = runtime;
    }

    /// Load flows from a directory and set up the router
    pub async fn load_flows_from_directory(&mut self, dir: impl AsRef<std::path::Path>) -> AofResult<usize> {
        let registry = FlowRegistry::from_directory(dir).await?;
        let count = registry.len();
        let router = FlowRouter::new(Arc::new(registry));
        self.flow_router = Some(Arc::new(router));
        Ok(count)
    }

    /// Load all agents from directory into the runtime
    /// Scans all YAML files, identifies `kind: Agent`, and indexes by `metadata.name`
    /// Returns the number of agents loaded
    pub async fn load_agents_from_directory(&mut self, dir: impl AsRef<std::path::Path>) -> AofResult<usize> {
        use std::fs;
        let dir_path = dir.as_ref();

        if !dir_path.exists() {
            return Err(AofError::config(format!(
                "Agents directory does not exist: {:?}", dir_path
            )));
        }

        let mut count = 0;
        let mut runtime = self.runtime.write().await;

        // Scan all YAML files in the directory
        let entries = fs::read_dir(dir_path).map_err(|e| {
            AofError::config(format!("Failed to read agents directory: {}", e))
        })?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            // Skip non-YAML files
            if !path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                continue;
            }

            // Try to load as agent
            match runtime.load_agent_from_file(path.to_str().unwrap()).await {
                Ok(agent_name) => {
                    info!("Loaded agent '{}' from {:?}", agent_name, path);
                    count += 1;
                }
                Err(e) => {
                    // Not all YAML files are agents, this is fine
                    debug!("Skipping {:?}: {}", path, e);
                }
            }
        }

        // Store the agents directory for dynamic loading
        self.agents_dir = Some(dir_path.to_path_buf());

        info!("Loaded {} agents from {:?}", count, dir_path);
        Ok(count)
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

        // Check for reaction events (for approval workflow)
        if let Some(event_type) = message.metadata.get("event_type") {
            info!("Detected event_type in metadata: {:?}", event_type);
            if event_type.as_str() == Some("reaction_added") {
                info!("Routing to reaction event handler");
                return self.handle_reaction_event(&message, platform_impl).await;
            }
        }

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

        // Check for callback:agent: or callback:flow: patterns (from inline keyboards)
        // Telegram wraps callback data with "callback:" prefix, so we get "callback:callback:agent:name"
        if message.text.starts_with("callback:") {
            return self.handle_callback(&message, platform_impl).await;
        }

        // First, check if we have a FlowRouter and try to match an AgentFlow
        if let Some(ref router) = self.flow_router {
            // Convert TriggerMessage to aof_triggers::TriggerMessage
            let trigger_msg = crate::TriggerMessage {
                message_id: message.id.clone(),
                user_id: message.user.id.clone(),
                user_name: message.user.username.clone().unwrap_or_default(),
                channel_id: message.channel_id.clone(),
                text: message.text.clone(),
                thread_id: message.thread_id.clone(),
                timestamp: message.timestamp,
                metadata: message.metadata.clone(),
            };

            if let Some(flow_match) = router.route_best(platform, &trigger_msg) {
                info!(
                    "Matched AgentFlow '{}' for message (score: {}, reason: {:?})",
                    flow_match.flow.metadata.name,
                    flow_match.score,
                    flow_match.reason
                );
                return self.execute_agentflow(platform_impl, &message, flow_match).await;
            }
        }

        // Parse command - if it fails and we have a default agent, route to it
        let cmd = match TriggerCommand::parse(&message) {
            Ok(cmd) => cmd,
            Err(e) => {
                // Get user's session agent or fall back to default
                let active_agent = self.get_user_agent(&message.user.id);

                if let Some(ref agent_name) = active_agent {
                    info!("Routing natural language message to agent '{}' for user '{}'", agent_name, message.user.id);
                    return self.handle_natural_language(&message, platform_impl, agent_name).await;
                }

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
            CommandType::Flows => Ok(self.handle_flows_command(cmd).await),
            CommandType::Context => Ok(self.handle_context_command(cmd).await),
        }
    }

    /// Handle run command
    ///
    /// Routes `/run agent <name> <input>` through the same path as natural language,
    /// using pre-loaded agents with their configured model, tools, and system prompt.
    async fn handle_run_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Agent => {
                let agent_name = cmd.get_arg(0).map_cmd_err()?;
                let input = cmd.args[1..].join(" ");

                if input.is_empty() {
                    return Ok(TriggerResponseBuilder::new()
                        .text(format!("Usage: `/run agent {} <your message>`", agent_name))
                        .error()
                        .build());
                }

                // Get platform for response
                let platform_impl = self
                    .platforms
                    .get(&cmd.context.platform)
                    .ok_or_else(|| aof_core::AofError::agent(format!("Unknown platform: {}", cmd.context.platform)))?;

                // Create a synthetic message to route through handle_natural_language
                // This ensures we use the pre-loaded agent with correct model, tools, and system prompt
                let message = TriggerMessage {
                    id: format!("run-{}", uuid::Uuid::new_v4()),
                    platform: cmd.context.platform.clone(),
                    channel_id: cmd.context.channel_id.clone(),
                    user: TriggerUser {
                        id: cmd.context.user_id.clone(),
                        username: Some(cmd.context.user_id.clone()),
                        display_name: None,
                        is_bot: false,
                    },
                    text: input,
                    timestamp: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                    thread_id: cmd.context.thread_id.clone(),
                    reply_to: None,
                };

                info!("Routing /run agent {} to handle_natural_language", agent_name);

                // Route through handle_natural_language which uses pre-loaded agents
                self.handle_natural_language(&message, platform_impl, agent_name).await?;

                // Return empty since handle_natural_language already sent the response
                Ok(TriggerResponseBuilder::new().build())
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

**Quick Start:**
• `/context` - Switch project/cluster (interactive)
• `/flows` - Trigger a workflow (interactive)
• `/help` - Show this help

**Context Commands:**
• `/context` - List contexts with inline selection
• `/context <name>` - Switch to context directly
• `/context info` - Show current context details

Context = Agent + Connection. Each context has:
• An agent (k8s-readonly, aws-readonly, etc.)
• Connection params (cluster, AWS profile, etc.)
• Tools available for that context

**Other Commands:**
• `/run agent <name> <input>` - Run specific agent directly
• `/status task <id>` - Check task status
• `/cancel task <id>` - Cancel a running task
• `/list tasks` - List all tasks

**Chat Mode:**
Once you select a context, just type naturally. The agent for that context will respond.

**Examples:**
• `/context` → tap "Cluster A" → "pod status"
• `/context aws-dev` → "list ec2 instances"
• `/flows` → tap approval-flow

**Support:** https://github.com/agenticdevops/aof
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

    /// Handle /context command - show or switch contexts
    ///
    /// Context = Agent + Connection Parameters
    /// Replaces both /env and /agents commands.
    ///
    /// Usage:
    /// - `/context` - List available contexts with inline selection
    /// - `/context <name>` - Switch to the specified context
    /// - `/context info` - Show detailed current context info
    async fn handle_context_command(&self, cmd: TriggerCommand) -> TriggerResponse {
        // Check if user wants to switch or just list
        let context_arg = cmd.args.first().map(|s| s.as_str());

        // Get current context for this user
        let current_context = self.get_user_context(&cmd.context.user_id);

        match context_arg {
            None => {
                // List all contexts with inline keyboard
                let mut builder = TriggerResponseBuilder::new();

                let current_display = self.available_contexts
                    .get(&current_context)
                    .map(|c| format!("{} {} ({})", c.emoji, c.display_name, current_context))
                    .unwrap_or_else(|| format!("*{}*", current_context));

                builder = builder.text(format!(
                    "**Select Context**\n\nCurrent: {}\n\nTap to switch:",
                    current_display
                ));

                // Add context buttons
                for entry in self.available_contexts.iter() {
                    let ctx_name = entry.key();
                    let ctx_config = entry.value();
                    let is_current = ctx_name == &current_context;

                    let label = if is_current {
                        format!("{} {} ✓", ctx_config.emoji, ctx_config.display_name)
                    } else {
                        format!("{} {}", ctx_config.emoji, ctx_config.display_name)
                    };

                    builder = builder.action(Action {
                        id: format!("ctx_{}", ctx_name),
                        label,
                        value: format!("callback:context:{}", ctx_name),
                        style: if is_current { ActionStyle::Primary } else { ActionStyle::Secondary },
                    });
                }

                builder.build()
            }
            Some("info") => {
                // Show detailed info about current context
                if let Some(ctx_config) = self.available_contexts.get(&current_context) {
                    let agent_display = ctx_config.agent_ref.as_deref().unwrap_or("default");
                    let tools_display = if ctx_config.tools.is_empty() {
                        "none".to_string()
                    } else {
                        ctx_config.tools.join(", ")
                    };

                    let info_text = format!(
                        "**Current Context: {} {}**\n\n\
                        **Name:** {}\n\
                        **Agent:** {}\n\
                        **Tools:** {}\n\n\
                        **Connection:**\n\
                        • Kubernetes Context: {}\n\
                        • Namespace: {}\n\
                        • AWS Profile: {}\n\
                        • AWS Region: {}\n\n\
                        {}\n\n\
                        Use `/context <name>` to switch contexts.",
                        ctx_config.emoji,
                        ctx_config.display_name,
                        current_context,
                        agent_display,
                        tools_display,
                        ctx_config.kubecontext.as_deref().unwrap_or("not set"),
                        ctx_config.namespace.as_deref().unwrap_or("default"),
                        ctx_config.aws_profile.as_deref().unwrap_or("not set"),
                        ctx_config.aws_region.as_deref().unwrap_or("not set"),
                        ctx_config.description
                    );
                    TriggerResponseBuilder::new()
                        .text(info_text)
                        .build()
                } else {
                    TriggerResponseBuilder::new()
                        .text(format!("Context '{}' not found.", current_context))
                        .error()
                        .build()
                }
            }
            Some(ctx_name) => {
                // Switch to the specified context
                if self.available_contexts.contains_key(ctx_name) {
                    self.set_user_context(&cmd.context.user_id, ctx_name);

                    let ctx_config = self.available_contexts.get(ctx_name).unwrap();
                    let agent_display = ctx_config.agent_ref.as_deref().unwrap_or("default");
                    let tools_display = if ctx_config.tools.is_empty() {
                        "standard".to_string()
                    } else {
                        ctx_config.tools.join(", ")
                    };

                    let response_text = format!(
                        "✅ Switched to {} *{}*\n\n\
                        🔄 Switching agent: {}\n\n\
                        **Connection:**\n\
                        • Cluster: {}\n\
                        • Namespace: {}\n\
                        • Region: {}\n\n\
                        **Tools Available:**\n\
                        • {}\n\n\
                        {}",
                        ctx_config.emoji,
                        ctx_config.display_name,
                        agent_display,
                        ctx_config.kubecontext.as_deref().unwrap_or("not set"),
                        ctx_config.namespace.as_deref().unwrap_or("default"),
                        ctx_config.aws_region.as_deref().unwrap_or("not set"),
                        tools_display,
                        ctx_config.description
                    );

                    TriggerResponseBuilder::new()
                        .text(response_text)
                        .success()
                        .build()
                } else {
                    // Unknown context
                    let available: Vec<String> = self.available_contexts
                        .iter()
                        .map(|e| e.key().clone())
                        .collect();

                    TriggerResponseBuilder::new()
                        .text(format!(
                            "Unknown context: '{}'\n\nAvailable: {}",
                            ctx_name,
                            available.join(", ")
                        ))
                        .error()
                        .build()
                }
            }
        }
    }

    /// Handle /flows command - show available flows with inline keyboard
    ///
    /// Returns a response with action buttons for each available flow.
    /// Users can click to trigger a flow execution.
    async fn handle_flows_command(&self, _cmd: TriggerCommand) -> TriggerResponse {
        // Get flows from the router if available
        let flows: Vec<String> = if let Some(ref router) = self.flow_router {
            router.list_flows()
        } else {
            Vec::new()
        };

        if flows.is_empty() {
            return TriggerResponseBuilder::new()
                .text("No flows available. Add flows to the flows directory.")
                .warning()
                .build();
        }

        // Build action buttons for each flow
        let mut builder = TriggerResponseBuilder::new()
            .text("**Select a Flow**\n\nTap to run:");

        for flow_name in flows.iter().take(8) { // Limit to 8 flows for UI
            builder = builder.action(Action {
                id: format!("flow_{}", flow_name),
                label: flow_name.clone(),
                value: format!("callback:flow:{}", flow_name),
                style: ActionStyle::Secondary,
            });
        }

        if flows.len() > 8 {
            builder = builder.text(format!(
                "\n\n_...and {} more flows._",
                flows.len() - 8
            ));
        }

        builder.build()
    }

    /// Set the active context for a user session
    /// Context = Agent + Connection Parameters
    pub fn set_user_context(&self, user_id: &str, ctx_name: &str) {
        self.user_context_sessions.insert(user_id.to_string(), ctx_name.to_string());
        info!("Set context '{}' for user '{}'", ctx_name, user_id);
    }

    /// Get the active context for a user session
    /// Returns the context name (defaults to first available or "cluster-a")
    pub fn get_user_context(&self, user_id: &str) -> String {
        self.user_context_sessions
            .get(user_id)
            .map(|v| v.clone())
            .unwrap_or_else(|| {
                // Return first available context or default
                self.available_contexts
                    .iter()
                    .next()
                    .map(|e| e.key().clone())
                    .unwrap_or_else(|| "cluster-a".to_string())
            })
    }

    /// Get the agent for the user's current context
    /// Used by handle_natural_language to determine which agent to use
    pub fn get_user_agent(&self, user_id: &str) -> Option<String> {
        let ctx_name = self.get_user_context(user_id);
        self.available_contexts
            .get(&ctx_name)
            .and_then(|ctx| ctx.agent_ref.clone())
            .or_else(|| self.config.default_agent.clone())
    }

    /// Check if user's current context is read-only
    /// Used by handle_natural_language to block write operations on Telegram
    pub fn is_user_context_read_only(&self, user_id: &str) -> bool {
        let ctx_name = self.get_user_context(user_id);
        self.available_contexts
            .get(&ctx_name)
            .map(|ctx| ctx.read_only)
            .unwrap_or(true)  // Default to read-only for safety
    }

    /// Handle callback from inline keyboard (context/flow selection)
    ///
    /// Callback data format:
    /// - Context selection: `callback:context:<context_name>`
    /// - Flow trigger: `callback:flow:<flow_name>`
    ///
    /// Telegram wraps with additional "callback:" so we receive "callback:callback:context:name"
    async fn handle_callback(
        &self,
        message: &TriggerMessage,
        platform_impl: &Arc<dyn TriggerPlatform>,
    ) -> AofResult<()> {
        // Strip the outer "callback:" prefix from Telegram platform wrapper
        let callback_data = message.text.trim_start_matches("callback:").trim();

        info!("Processing callback - raw: '{}', stripped: '{}'", message.text, callback_data);

        // Parse callback format: "callback:context:name" or "callback:flow:name"
        // Also support direct format: "context:name" or "flow:name" (without callback: prefix)
        let parts: Vec<&str> = callback_data.splitn(3, ':').collect();

        // Determine the callback type and value
        let (callback_type, callback_value) = if parts.len() >= 3 && parts[0] == "callback" {
            // Format: callback:context:name or callback:flow:name
            (parts[1], parts[2])
        } else if parts.len() >= 2 && (parts[0] == "context" || parts[0] == "flow") {
            // Format: context:name or flow:name (direct format)
            (parts[0], parts[1])
        } else {
            warn!("Invalid callback format: '{}' (parts: {:?})", callback_data, parts);
            let response = TriggerResponseBuilder::new()
                .text(format!("Invalid selection format. Please try again.\nReceived: {}", callback_data))
                .error()
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        };

        info!("Parsed callback - type: '{}', value: '{}'", callback_type, callback_value);

        match callback_type {
            "context" => {
                // Switch to the selected context
                // Context = Agent + Connection Parameters
                if self.available_contexts.contains_key(callback_value) {
                    self.set_user_context(&message.user.id, callback_value);

                    let ctx_config = self.available_contexts.get(callback_value).unwrap();
                    let agent_display = ctx_config.agent_ref.as_deref().unwrap_or("default");
                    let tools_display = if ctx_config.tools.is_empty() {
                        "standard".to_string()
                    } else {
                        ctx_config.tools.join(", ")
                    };

                    let response_text = format!(
                        "✅ Switched to {} *{}*\n\n\
                        🔄 Switching agent: {}\n\n\
                        **Connection:**\n\
                        • Cluster: {}\n\
                        • Namespace: {}\n\
                        • Region: {}\n\n\
                        **Tools Available:**\n\
                        • {}\n\n\
                        {}",
                        ctx_config.emoji,
                        ctx_config.display_name,
                        agent_display,
                        ctx_config.kubecontext.as_deref().unwrap_or("not set"),
                        ctx_config.namespace.as_deref().unwrap_or("default"),
                        ctx_config.aws_region.as_deref().unwrap_or("not set"),
                        tools_display,
                        ctx_config.description
                    );

                    let response = TriggerResponseBuilder::new()
                        .text(response_text)
                        .success()
                        .build();
                    let _ = platform_impl.send_response(&message.channel_id, response).await;
                } else {
                    let response = TriggerResponseBuilder::new()
                        .text(format!("Context not found: {}", callback_value))
                        .error()
                        .build();
                    let _ = platform_impl.send_response(&message.channel_id, response).await;
                }
            }
            "flow" => {
                // Trigger the selected flow
                info!("Triggering flow: {}", callback_value);

                // Send acknowledgment
                let ack = TriggerResponseBuilder::new()
                    .text(format!("Running flow: *{}*...", callback_value))
                    .build();
                let _ = platform_impl.send_response(&message.channel_id, ack).await;

                // Execute the flow if we have a router
                if let Some(ref router) = self.flow_router {
                    if let Some(flow) = router.get_flow(callback_value) {
                        // Create a synthetic message to trigger the flow
                        let synthetic_msg = TriggerMessage {
                            id: format!("flow-{}", uuid::Uuid::new_v4()),
                            platform: message.platform.clone(),
                            channel_id: message.channel_id.clone(),
                            user: message.user.clone(),
                            text: format!("Run flow {}", callback_value),
                            timestamp: chrono::Utc::now(),
                            metadata: message.metadata.clone(),
                            thread_id: message.thread_id.clone(),
                            reply_to: None,
                        };

                        let flow_match = FlowMatch {
                            flow: flow.clone(),
                            score: 100,
                            reason: crate::flow::MatchReason::ExplicitDefault,
                        };

                        return self.execute_agentflow(platform_impl, &synthetic_msg, flow_match).await;
                    } else {
                        let response = TriggerResponseBuilder::new()
                            .text(format!("Flow not found: {}", callback_value))
                            .error()
                            .build();
                        let _ = platform_impl.send_response(&message.channel_id, response).await;
                    }
                } else {
                    let response = TriggerResponseBuilder::new()
                        .text("No flows available.")
                        .error()
                        .build();
                    let _ = platform_impl.send_response(&message.channel_id, response).await;
                }
            }
            _ => {
                warn!("Unknown callback type: {}", callback_type);
                let response = TriggerResponseBuilder::new()
                    .text(format!("Unknown selection type: {}", callback_type))
                    .error()
                    .build();
                let _ = platform_impl.send_response(&message.channel_id, response).await;
            }
        }

        Ok(())
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

    /// Handle natural language message by routing to default agent
    async fn handle_natural_language(
        &self,
        message: &TriggerMessage,
        platform_impl: &Arc<dyn TriggerPlatform>,
        agent_name: &str,
    ) -> AofResult<()> {
        use aof_core::{AgentConfig, ModelConfig, ModelProvider};
        use aof_llm::ProviderFactory;
        use aof_runtime::AgentExecutor;
        use aof_memory::{InMemoryBackend, SimpleMemory};

        // Clean up the message text (remove @mentions for Slack)
        let input = message.text
            .replace(&format!("<@{}>", message.user.id), "")
            .trim()
            .to_string();

        // Remove any Slack user mentions like <@U12345>
        let input = regex::Regex::new(r"<@[A-Z0-9]+>")
            .map(|re| re.replace_all(&input, "").to_string())
            .unwrap_or(input)
            .trim()
            .to_string();

        if input.is_empty() {
            let response = TriggerResponseBuilder::new()
                .text("Hi! How can I help you? Just ask me anything.")
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        }

        info!("Processing natural language input for agent {}: {}", agent_name, input);

        // MVP Safety Layer: Block write operations on mobile platforms (Telegram, WhatsApp)
        // Platform hierarchy: CLI (full access) > Slack (approval for writes) > Telegram/WhatsApp (read-only)
        let is_mobile_platform = matches!(message.platform.as_str(), "telegram" | "whatsapp");
        if is_mobile_platform && is_write_operation(&input) {
            let ctx_name = self.get_user_context(&message.user.id);
            warn!("Blocked write operation on {} in context '{}': {}", message.platform, ctx_name, input);

            let response = TriggerResponseBuilder::new()
                .text(format!(
                    "🚫 *Write operation blocked*\n\n\
                    {} is read-only. Write, delete, and dangerous operations are not allowed from mobile.\n\n\
                    *What you can do:*\n\
                    • Use read-only commands (get, list, describe, logs)\n\
                    • Use Slack or CLI for write operations\n\n\
                    _Detected write intent: `{}`_",
                    message.platform,
                    if input.len() > 50 { &input[..50] } else { &input }
                ))
                .error()
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        }

        let thread_id = message.thread_id.as_deref();

        // Get conversation history for context BEFORE adding the current message
        // This ensures we don't duplicate the current message in context
        let conversation_context = self.format_conversation_context(&message.channel_id, thread_id);
        debug!("Conversation context length: {} chars", conversation_context.len());

        // Now store the user message in conversation memory for future context
        self.add_to_conversation(&message.channel_id, thread_id, "user", &input);

        // Send typing indicator / acknowledgment
        let ack = TriggerResponseBuilder::new()
            .text("🤔 Thinking...")
            .build();
        let _ = platform_impl.send_response(&message.channel_id, ack).await;

        // Build the full input with conversation context
        let input_with_context = if conversation_context.is_empty() {
            input.clone()
        } else {
            format!("{}\n\nCurrent message: {}", conversation_context, input)
        };

        // Check if agent is pre-loaded in the runtime (indexed by metadata.name)
        let runtime = self.runtime.read().await;
        let agent_exists = runtime.has_agent(agent_name);
        drop(runtime);

        if agent_exists {
            // Use pre-loaded agent from runtime
            info!("Using pre-loaded agent: {}", agent_name);

            let runtime = self.runtime.read().await;
            match runtime.execute(agent_name, &input_with_context).await {
                Ok(output) => {
                    info!("Agent '{}' executed successfully", agent_name);

                    // Parse output for approval requirements
                    let (requires_approval, command, clean_output) = parse_approval_output(&output);

                    if requires_approval {
                        if let Some(cmd) = command {
                            info!("Command requires approval: {}", cmd);

                            // Send approval request message
                            let approval_text = format!(
                                "{}\n\n⚠️ *This action requires approval*\n`{}`\n\nReact with ✅ to approve or ❌ to deny.",
                                clean_output,
                                cmd
                            );

                            // Try to use SlackPlatform directly for approval flow
                            if let Some(slack) = platform_impl.as_any().downcast_ref::<crate::platforms::SlackPlatform>() {
                                let thread_ts = message.thread_id.as_deref();
                                match slack.post_message_with_ts(&message.channel_id, &approval_text, thread_ts).await {
                                    Ok((channel, msg_ts)) => {
                                        // Add reactions for approve/deny
                                        let _ = slack.add_reaction(&channel, &msg_ts, "white_check_mark").await;
                                        let _ = slack.add_reaction(&channel, &msg_ts, "x").await;

                                        // Store pending approval
                                        let approval = PendingApproval {
                                            command: cmd.clone(),
                                            user_id: message.user.id.clone(),
                                            channel_id: channel.clone(),
                                            message_ts: msg_ts.clone(),
                                            requested_at: chrono::Utc::now(),
                                            agent_name: agent_name.to_string(),
                                            original_message: input.clone(),
                                        };
                                        self.pending_approvals.insert(msg_ts.clone(), approval);
                                        info!("Stored pending approval for message {}", msg_ts);
                                    }
                                    Err(e) => {
                                        error!("Failed to post approval message: {}", e);
                                        let response = TriggerResponseBuilder::new()
                                            .text(format!("❌ Failed to request approval: {}", e))
                                            .error()
                                            .build();
                                        let _ = platform_impl.send_response(&message.channel_id, response).await;
                                    }
                                }
                            } else {
                                // Fallback for non-Slack platforms
                                let response = TriggerResponseBuilder::new()
                                    .text(approval_text)
                                    .build();
                                let _ = platform_impl.send_response(&message.channel_id, response).await;
                            }
                        } else {
                            // requires_approval but no command - just send the output
                            let response = TriggerResponseBuilder::new()
                                .text(clean_output)
                                .success()
                                .build();
                            let _ = platform_impl.send_response(&message.channel_id, response).await;
                        }
                    } else {
                        // Normal response without approval
                        // Store assistant response in conversation memory
                        self.add_to_conversation(&message.channel_id, thread_id, "assistant", &output);

                        let response = TriggerResponseBuilder::new()
                            .text(output)
                            .success()
                            .build();
                        let _ = platform_impl.send_response(&message.channel_id, response).await;
                    }

                    return Ok(());
                }
                Err(e) => {
                    error!("Agent execution failed: {}", e);
                    let error_msg = format!("❌ Sorry, I encountered an error: {}", e);
                    // Store error in conversation memory too
                    self.add_to_conversation(&message.channel_id, thread_id, "assistant", &error_msg);

                    let response = TriggerResponseBuilder::new()
                        .text(error_msg)
                        .error()
                        .build();
                    let _ = platform_impl.send_response(&message.channel_id, response).await;
                    return Ok(());
                }
            }
        }

        // Fallback: Create a simple agent without tools
        debug!("Using fallback agent without tools for: {}", agent_name);

        // Determine model based on environment
        let (model_name, provider) = if std::env::var("GOOGLE_API_KEY").is_ok() {
            ("gemini-2.5-flash".to_string(), ModelProvider::Google)
        } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            ("claude-3-5-sonnet-20241022".to_string(), ModelProvider::Anthropic)
        } else if std::env::var("OPENAI_API_KEY").is_ok() {
            ("gpt-4o".to_string(), ModelProvider::OpenAI)
        } else {
            let response = TriggerResponseBuilder::new()
                .text("❌ No API key configured. Please set GOOGLE_API_KEY, ANTHROPIC_API_KEY, or OPENAI_API_KEY.")
                .error()
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        };

        // Create agent configuration
        let config = AgentConfig {
            name: agent_name.to_string(),
            system_prompt: Some(format!(
                "You are a helpful AI assistant responding in a Slack channel. \
                Keep responses concise and use Slack markdown formatting. \
                Use code blocks with ``` for code. \
                Be friendly and helpful. User: {}",
                message.user.username.as_deref().unwrap_or("unknown")
            )),
            model: model_name.clone(),
            provider: Some(format!("{:?}", provider).to_lowercase()),
            tools: vec![],
            mcp_servers: vec![],
            memory: None,
            max_iterations: 5,
            temperature: 0.7,
            max_tokens: Some(2000),
            extra: std::collections::HashMap::new(),
        };

        // Create model config
        let model_config = ModelConfig {
            model: model_name,
            provider,
            api_key: None, // Will use env var
            endpoint: None,
            temperature: 0.7,
            max_tokens: Some(2000),
            timeout_secs: 60,
            headers: std::collections::HashMap::new(),
            extra: std::collections::HashMap::new(),
        };

        // Create model
        let model = match ProviderFactory::create(model_config).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to create model: {}", e);
                let response = TriggerResponseBuilder::new()
                    .text(format!("❌ Failed to initialize AI: {}", e))
                    .error()
                    .build();
                let _ = platform_impl.send_response(&message.channel_id, response).await;
                return Ok(());
            }
        };

        // Create memory backend
        let memory_backend = InMemoryBackend::new();
        let memory = std::sync::Arc::new(SimpleMemory::new(std::sync::Arc::new(memory_backend)));

        // Create executor
        let executor = AgentExecutor::new(
            config,
            model,
            None, // No tool executor in fallback mode
            Some(memory),
        );

        // Execute with conversation context
        let mut context = AgentContext::new(&input_with_context);
        let result = executor.execute(&mut context).await;

        // Send response and store in conversation memory
        let response = match result {
            Ok(output) => {
                // Store assistant response in conversation memory
                self.add_to_conversation(&message.channel_id, thread_id, "assistant", &output);

                TriggerResponseBuilder::new()
                    .text(output)
                    .success()
                    .build()
            }
            Err(e) => {
                error!("Agent execution failed: {}", e);
                let error_msg = format!("❌ Sorry, I encountered an error: {}", e);
                // Store error in conversation memory
                self.add_to_conversation(&message.channel_id, thread_id, "assistant", &error_msg);

                TriggerResponseBuilder::new()
                    .text(error_msg)
                    .error()
                    .build()
            }
        };

        let _ = platform_impl.send_response(&message.channel_id, response).await;
        Ok(())
    }

    /// Execute a matched AgentFlow
    ///
    /// This method executes an AgentFlow that was matched by the FlowRouter.
    /// The flow contains the full workflow configuration including:
    /// - Trigger configuration (platform, channels, patterns)
    /// - Flow context (kubeconfig, namespace, env vars)
    /// - Node graph with agent execution nodes
    async fn execute_agentflow(
        &self,
        platform_impl: &Arc<dyn TriggerPlatform>,
        message: &TriggerMessage,
        flow_match: FlowMatch,
    ) -> AofResult<()> {
        let flow_name = &flow_match.flow.metadata.name;

        // Clean up the message text (remove @mentions for Slack)
        let input = message.text
            .replace(&format!("<@{}>", message.user.id), "")
            .trim()
            .to_string();

        // Remove any Slack user mentions like <@U12345>
        let input = regex::Regex::new(r"<@[A-Z0-9]+>")
            .map(|re| re.replace_all(&input, "").to_string())
            .unwrap_or(input)
            .trim()
            .to_string();

        if input.is_empty() {
            let response = TriggerResponseBuilder::new()
                .text("Hi! How can I help you? Just ask me anything.")
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        }

        info!("Executing AgentFlow '{}' with input: {}", flow_name, input);

        // Send typing indicator / acknowledgment
        let ack = TriggerResponseBuilder::new()
            .text(format!("🔄 Processing with flow `{}`...", flow_name))
            .build();
        let _ = platform_impl.send_response(&message.channel_id, ack).await;

        // Create AgentFlowExecutor
        let mut executor = AgentFlowExecutor::new(
            (*flow_match.flow).clone(),
            Arc::clone(&self.runtime),
        );

        // Set agents directory if configured
        if let Some(ref dir) = self.agents_dir {
            executor = executor.with_agents_dir(dir);
        }

        // Build trigger data from the message
        let trigger_data = serde_json::json!({
            "event": {
                "type": "message",
                "text": input,
                "user": {
                    "id": message.user.id,
                    "username": message.user.username,
                },
                "channel_id": message.channel_id,
                "thread_id": message.thread_id,
                "timestamp": message.timestamp.to_rfc3339(),
                "metadata": message.metadata,
            },
            "platform": platform_impl.platform_name(),
            "flow_name": flow_name,
            "match_reason": format!("{:?}", flow_match.reason),
            "match_score": flow_match.score,
        });

        // Execute the flow
        let result = executor.execute(trigger_data).await;

        // Send response based on execution result
        let response = match result {
            Ok(state) => {
                // Extract output from the final node or state
                let output = if let Some(last_result) = state.node_results.values().last() {
                    if let Some(ref output) = last_result.output {
                        // Try to get an "output" field, or stringify the whole thing
                        output.get("output")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| {
                                // Check if there's a text response in a nested structure
                                output.get("response")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| serde_json::to_string_pretty(output).unwrap_or_default())
                            })
                    } else {
                        format!("✅ Flow `{}` completed successfully.", flow_name)
                    }
                } else {
                    format!("✅ Flow `{}` completed.", flow_name)
                };

                TriggerResponseBuilder::new()
                    .text(output)
                    .success()
                    .build()
            }
            Err(e) => {
                error!("AgentFlow '{}' execution failed: {}", flow_name, e);
                TriggerResponseBuilder::new()
                    .text(format!("❌ Flow `{}` failed: {}", flow_name, e))
                    .error()
                    .build()
            }
        };

        let _ = platform_impl.send_response(&message.channel_id, response).await;
        Ok(())
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

    /// Handle reaction events for approval workflow
    ///
    /// When a user reacts to a pending approval message with ✅ (approve) or ❌ (deny),
    /// this function processes the reaction and either executes the command or cancels it.
    async fn handle_reaction_event(
        &self,
        message: &TriggerMessage,
        platform_impl: &Arc<dyn TriggerPlatform>,
    ) -> AofResult<()> {
        // Extract reaction and item_ts from metadata
        let reaction = message.metadata.get("reaction")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let item_ts = message.metadata.get("item_ts")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        info!("Processing reaction '{}' on message '{}'", reaction, item_ts);

        // Only process approve (white_check_mark) or deny (x) reactions
        let is_approve = reaction == "white_check_mark" || reaction == "+1" || reaction == "heavy_check_mark";
        let is_deny = reaction == "x" || reaction == "-1" || reaction == "no_entry";

        if !is_approve && !is_deny {
            info!("Ignoring non-approval reaction: {}", reaction);
            return Ok(());
        }

        // Log current pending approvals for debugging
        let pending_keys: Vec<String> = self.pending_approvals.iter().map(|r| r.key().clone()).collect();
        info!("Looking up approval for '{}', pending approvals: {:?}", item_ts, pending_keys);

        // Look up pending approval by item_ts
        let approval = match self.pending_approvals.remove(item_ts) {
            Some((_, approval)) => approval,
            None => {
                info!("No pending approval found for message '{}'", item_ts);
                return Ok(());
            }
        };

        info!(
            "Processing {} for command '{}' by user {}",
            if is_approve { "approval" } else { "denial" },
            approval.command,
            message.user.id
        );

        // Check if user has permission to approve
        let can_approve = message.metadata.get("can_approve")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to true for backward compatibility

        if !can_approve {
            info!(
                "User {} is not authorized to approve commands",
                message.user.id
            );

            // Re-insert the pending approval (it wasn't consumed)
            self.pending_approvals.insert(item_ts.to_string(), approval);

            // Send unauthorized message
            let response = TriggerResponseBuilder::new()
                .text(format!(
                    "⚠️ <@{}> is not authorized to approve commands. Please contact an admin.",
                    message.user.id
                ))
                .thread_id(message.thread_id.clone().unwrap_or_default())
                .build();
            let _ = platform_impl.send_response(&message.channel_id, response).await;
            return Ok(());
        }

        if is_deny {
            // Send denial message
            let denial_text = format!(
                "❌ *Action denied by <@{}>*\n```{}```",
                message.user.id,
                approval.command
            );

            let response = TriggerResponseBuilder::new()
                .text(denial_text)
                .thread_id(approval.message_ts.clone())
                .build();
            let _ = platform_impl.send_response(&approval.channel_id, response).await;

            return Ok(());
        }

        // Approve - execute the command
        info!("Executing approved command: {}", approval.command);

        // Send "executing" message
        let executing_text = format!(
            "⚡ *Executing approved command...*\n```{}```",
            approval.command
        );
        let response = TriggerResponseBuilder::new()
            .text(executing_text)
            .thread_id(approval.message_ts.clone())
            .build();
        let _ = platform_impl.send_response(&approval.channel_id, response).await;

        // Execute the command using shell
        let output = match tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&approval.command)
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    let result = if stdout.is_empty() {
                        "Command completed successfully (no output)".to_string()
                    } else {
                        stdout.to_string()
                    };
                    (true, result)
                } else {
                    let error = if stderr.is_empty() {
                        format!("Command failed with exit code: {:?}", output.status.code())
                    } else {
                        stderr.to_string()
                    };
                    (false, error)
                }
            }
            Err(e) => (false, format!("Failed to execute command: {}", e)),
        };

        // Send result back to Slack
        let (success, result_text) = output;
        let result_message = if success {
            format!(
                "✅ *Command completed successfully*\n```{}```\n*Approved by:* <@{}>",
                truncate_output(&result_text, 2500),
                message.user.id
            )
        } else {
            format!(
                "❌ *Command failed*\n```{}```\n*Approved by:* <@{}>",
                truncate_output(&result_text, 2500),
                message.user.id
            )
        };

        let response = TriggerResponseBuilder::new()
            .text(result_message)
            .thread_id(approval.message_ts)
            .build();
        let _ = platform_impl.send_response(&approval.channel_id, response).await;

        Ok(())
    }
}

/// Truncate output to a maximum length, adding ellipsis if needed
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...\n[Output truncated - {} more characters]", &output[..max_len], output.len() - max_len)
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
