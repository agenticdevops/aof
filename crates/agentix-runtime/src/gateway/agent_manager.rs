//! Agent manager for the gateway — loads, stores, and runs agents.
//!
//! The `AgentManager` is the core state object shared across all axum handlers.
//! It holds:
//! - A map of loaded `AgentDefinition`s keyed by agent name
//! - A map of active/completed `RunState`s keyed by run ID
//! - The workspace configuration (provider credentials, defaults)
//!
//! All fields use `DashMap` for lock-free concurrent reads from multiple
//! request handlers.

use std::path::Path;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use uuid::Uuid;

use agentix_core::{
    AgentDefinition, AgentixError, AgentLoader, FlatYamlLoader, ModelConfig,
    ModelProvider, TriggerEvent, TriggerRunRegistry, WorkspaceConfig,
};
use agentix_llm::ProviderFactory;

use crate::executor::react_loop::{ReActConfig, ReActEngine, ReActEvent, RunResult, ToolExecutor};
use crate::streaming::EventReceiver;
use crate::tools::{CliToolExecutor, CompositeToolExecutor, McpToolExecutor, WasmToolExecutor};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Status of a loaded agent.
#[derive(Debug, Clone)]
pub enum AgentStatus {
    /// Agent loaded successfully and ready to run.
    Ready,
    /// Agent is currently running one or more tasks.
    Running,
    /// Agent failed to load or encountered an error.
    Error(String),
}

/// A loaded agent with its assembled definition and metadata.
#[derive(Debug, Clone)]
pub struct LoadedAgent {
    /// Fully assembled agent definition (SOUL + RULES + skills + tools).
    pub definition: AgentDefinition,
    /// Filesystem path the agent was loaded from.
    pub source_path: std::path::PathBuf,
    /// When the agent was loaded.
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// Current agent status.
    pub status: AgentStatus,
}

/// Summary view of an agent for API responses.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentSummary {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub status: AgentStatusSummary,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

/// Serialisable version of `AgentStatus`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatusSummary {
    Ready,
    Running,
    Error(String),
}

impl From<&AgentStatus> for AgentStatusSummary {
    fn from(s: &AgentStatus) -> Self {
        match s {
            AgentStatus::Ready => AgentStatusSummary::Ready,
            AgentStatus::Running => AgentStatusSummary::Running,
            AgentStatus::Error(e) => AgentStatusSummary::Error(e.clone()),
        }
    }
}

/// Status of a single agent run.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// Complete state for a single agent run.
pub struct RunState {
    pub id: String,
    pub agent_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: RunStatus,
    pub input: String,
    pub output: Option<String>,
    pub iterations: u32,
    pub events: Vec<ReActEvent>,
    /// Cancel signal sender. Consumed when `stop_run` is called.
    pub cancel_tx: Option<oneshot::Sender<()>>,
}

/// Summary view of a run for API responses.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunSummary {
    pub id: String,
    pub agent_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: RunStatus,
    pub input: String,
    pub output: Option<String>,
    pub iterations: u32,
}

// ---------------------------------------------------------------------------
// AgentManager
// ---------------------------------------------------------------------------

/// The core state object shared across all gateway HTTP handlers.
///
/// Uses `DashMap` for lock-free concurrent access from multiple async tasks.
/// Wrap in `Arc` (see `AgentManager::new`) for sharing via axum state.
pub struct AgentManager {
    agents: DashMap<String, LoadedAgent>,
    runs: DashMap<String, RunState>,
    workspace_config: Option<WorkspaceConfig>,
    /// Root directory where agents are loaded from (used for WASM tool file resolution).
    agents_dir: Option<std::path::PathBuf>,
    /// Registry of active trigger instances (CronTrigger, WebhookTrigger, etc.).
    pub trigger_registry: Mutex<TriggerRunRegistry>,
    /// Sender side of the trigger event channel.
    /// Trigger background tasks send (agent_name, TriggerEvent) here.
    trigger_event_tx: mpsc::Sender<(String, TriggerEvent)>,
}

impl AgentManager {
    /// Create a new `AgentManager` wrapped in an `Arc` for sharing.
    pub fn new(workspace_config: Option<WorkspaceConfig>) -> Arc<Self> {
        let (trigger_event_tx, trigger_event_rx) = mpsc::channel::<(String, TriggerEvent)>(256);
        let manager = Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
            agents_dir: None,
            trigger_registry: Mutex::new(TriggerRunRegistry::new()),
            trigger_event_tx,
        });
        // Spawn the trigger dispatcher background task
        manager.clone().spawn_trigger_dispatcher(trigger_event_rx);
        manager
    }

    /// Create an `AgentManager` with a known agents directory (used for WASM tool resolution).
    pub fn with_agents_dir(workspace_config: Option<WorkspaceConfig>, agents_dir: std::path::PathBuf) -> Arc<Self> {
        let (trigger_event_tx, trigger_event_rx) = mpsc::channel::<(String, TriggerEvent)>(256);
        let manager = Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
            agents_dir: Some(agents_dir),
            trigger_registry: Mutex::new(TriggerRunRegistry::new()),
            trigger_event_tx,
        });
        manager.clone().spawn_trigger_dispatcher(trigger_event_rx);
        manager
    }

    /// Spawn the background task that reads from the trigger event channel
    /// and calls `run_agent_with_trigger()` for each event.
    fn spawn_trigger_dispatcher(
        self: Arc<Self>,
        mut rx: mpsc::Receiver<(String, TriggerEvent)>,
    ) {
        tokio::spawn(async move {
            while let Some((agent_name, event)) = rx.recv().await {
                let manager = self.clone();
                tokio::spawn(async move {
                    match manager.run_agent_with_trigger(&agent_name, event).await {
                        Ok(run_id) => {
                            tracing::info!(
                                "Trigger fired agent '{}' → run_id '{}'",
                                agent_name,
                                run_id
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Trigger failed to run agent '{}': {}",
                                agent_name,
                                e
                            );
                        }
                    }
                });
            }
        });
    }

    /// Get a clone of the trigger event sender (for use in gateway HTTP handlers).
    pub fn trigger_event_tx(&self) -> mpsc::Sender<(String, TriggerEvent)> {
        self.trigger_event_tx.clone()
    }

    // -------------------------------------------------------------------------
    // Agent loading
    // -------------------------------------------------------------------------

    /// Scan a directory and load all agents.
    ///
    /// For each entry in `dir`:
    /// - Subdirectory → try `DirectoryLoader` (GitAgent format)
    /// - `*.yaml` / `*.yml` file → try `FlatYamlLoader` (backward compat)
    ///
    /// Failed loads are logged as warnings; other agents still load.
    /// Returns the list of successfully loaded agent names.
    pub fn load_from_dir(&self, dir: &Path) -> Vec<String> {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("Cannot scan agents_dir {:?}: {}", dir, err);
                return vec![];
            }
        };

        let mut loaded_names = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();
            let is_yaml = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "yaml" || s == "yml")
                .unwrap_or(false);

            if !is_dir && !is_yaml {
                continue;
            }

            match AgentLoader::load(&path) {
                Ok(mut def) => {
                    // Apply workspace defaults (provider model, timeout, etc.)
                    if let Some(ws) = &self.workspace_config {
                        def.apply_workspace_defaults(ws);
                    }
                    let name = def.name.clone();
                    // Register triggers from agent YAML before inserting into agents map
                    self.register_agent_triggers_sync(&def);
                    self.agents.insert(
                        name.clone(),
                        LoadedAgent {
                            definition: def,
                            source_path: path.clone(),
                            loaded_at: chrono::Utc::now(),
                            status: AgentStatus::Ready,
                        },
                    );
                    tracing::info!("Loaded agent '{}' from {:?}", name, path);
                    loaded_names.push(name);
                }
                Err(err) => {
                    tracing::warn!("Failed to load agent from {:?}: {}", path, err);
                }
            }
        }

        loaded_names
    }

    /// Register triggers from agent YAML `triggers:` field into `TriggerRunRegistry`.
    ///
    /// Called synchronously during `load_from_dir`. Actual trigger `start()` calls
    /// happen asynchronously at server startup time via `start_all_triggers()`.
    fn register_agent_triggers_sync(&self, def: &AgentDefinition) {
        use cron::Schedule;
        use std::str::FromStr;

        for (i, trigger_cfg) in def.triggers.iter().enumerate() {
            let trigger_id = format!("{}-{}-{}", def.name, trigger_cfg.trigger_type, i);
            match trigger_cfg.trigger_type.as_str() {
                "cron" | "schedule" => {
                    if let Some(expr) = &trigger_cfg.expression {
                        // Normalize 5-field to 7-field
                        let normalized = normalize_cron_expr(expr);
                        match Schedule::from_str(&normalized) {
                            Ok(_) => {
                                let trigger = CronTriggerImpl {
                                    id: trigger_id.clone(),
                                    expression: expr.clone(),
                                    agent_name: def.name.clone(),
                                    stop_tx: tokio::sync::Mutex::new(None),
                                };
                                // We can't call async here, so store in a sync way
                                // The actual start() is called from start_all_triggers()
                                if let Ok(mut registry) = self.trigger_registry.try_lock() {
                                    registry.register(Arc::new(trigger));
                                    tracing::info!(
                                        "Registered cron trigger '{}' for agent '{}'",
                                        trigger_id,
                                        def.name
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Agent '{}' has invalid cron expression '{}': {}",
                                    def.name, expr, e
                                );
                            }
                        }
                    } else {
                        tracing::warn!(
                            "Agent '{}' has cron trigger without expression field",
                            def.name
                        );
                    }
                }
                "slack" => {
                    let trigger = ChannelMentionTriggerImpl {
                        id: trigger_id.clone(),
                        agent_name: def.name.clone(),
                        platform: ChannelPlatformImpl::Slack {
                            signing_secret: trigger_cfg.signing_secret.clone(),
                        },
                        sender: tokio::sync::Mutex::new(None),
                    };
                    if let Ok(mut registry) = self.trigger_registry.try_lock() {
                        registry.register(Arc::new(trigger));
                        tracing::info!(
                            "Registered slack mention trigger '{}' for agent '{}'",
                            trigger_id, def.name
                        );
                    }
                }
                "discord" => {
                    let trigger = ChannelMentionTriggerImpl {
                        id: trigger_id.clone(),
                        agent_name: def.name.clone(),
                        platform: ChannelPlatformImpl::Discord,
                        sender: tokio::sync::Mutex::new(None),
                    };
                    if let Ok(mut registry) = self.trigger_registry.try_lock() {
                        registry.register(Arc::new(trigger));
                        tracing::info!(
                            "Registered discord mention trigger '{}' for agent '{}'",
                            trigger_id, def.name
                        );
                    }
                }
                "telegram" => {
                    let trigger = ChannelMentionTriggerImpl {
                        id: trigger_id.clone(),
                        agent_name: def.name.clone(),
                        platform: ChannelPlatformImpl::Telegram,
                        sender: tokio::sync::Mutex::new(None),
                    };
                    if let Ok(mut registry) = self.trigger_registry.try_lock() {
                        registry.register(Arc::new(trigger));
                        tracing::info!(
                            "Registered telegram trigger '{}' for agent '{}'",
                            trigger_id, def.name
                        );
                    }
                }
                "webhook" | "github" | "jira" => {
                    // Passive webhook triggers — no background task needed.
                    // The /webhooks/:trigger_id route dispatches directly.
                    tracing::debug!(
                        "Webhook-type trigger '{}' for agent '{}' dispatched via HTTP route",
                        trigger_id, def.name
                    );
                }
                other => {
                    tracing::debug!(
                        "Unknown trigger type '{}' for agent '{}' — skipped",
                        other, def.name
                    );
                }
            }
        }
    }

    /// Start all registered triggers. Call this after all agents are loaded,
    /// during gateway server startup.
    pub async fn start_all_triggers(self: &Arc<Self>) {
        let triggers = {
            let registry = self.trigger_registry.lock().await;
            registry.all()
        };

        let sender = self.trigger_event_tx.clone();
        for trigger in triggers {
            let sender_clone = sender.clone();
            if let Err(e) = trigger.start(sender_clone).await {
                tracing::warn!("Failed to start trigger '{}': {}", trigger.trigger_id(), e);
            } else {
                tracing::info!("Started trigger '{}'", trigger.trigger_id());
            }
        }
    }

    /// Register a new agent from raw YAML content.
    ///
    /// Parses using `FlatYamlLoader` (the API sends inline YAML).
    /// Returns 409 conflict if an agent with the same name already exists.
    pub fn register_from_yaml(&self, yaml_content: &str) -> Result<String, AgentixError> {
        let mut def = FlatYamlLoader::load_from_str(yaml_content)?;

        if self.agents.contains_key(&def.name) {
            return Err(AgentixError::Runtime(format!(
                "Agent '{}' already exists. Use PUT to update.",
                def.name
            )));
        }

        if let Some(ws) = &self.workspace_config {
            def.apply_workspace_defaults(ws);
        }

        let name = def.name.clone();
        self.agents.insert(
            name.clone(),
            LoadedAgent {
                definition: def,
                source_path: std::path::PathBuf::new(),
                loaded_at: chrono::Utc::now(),
                status: AgentStatus::Ready,
            },
        );

        Ok(name)
    }

    /// Update an existing agent from raw YAML content.
    ///
    /// Returns 404 if agent does not exist.
    pub fn update_from_yaml(&self, name: &str, yaml_content: &str) -> Result<(), AgentixError> {
        if !self.agents.contains_key(name) {
            return Err(AgentixError::Runtime(format!(
                "Agent '{}' not found. Use POST to register a new agent.",
                name
            )));
        }

        let mut def = FlatYamlLoader::load_from_str(yaml_content)?;

        if let Some(ws) = &self.workspace_config {
            def.apply_workspace_defaults(ws);
        }

        // Preserve loaded_at from existing entry, update definition
        let loaded_at = self
            .agents
            .get(name)
            .map(|a| a.loaded_at)
            .unwrap_or_else(chrono::Utc::now);

        self.agents.insert(
            name.to_string(),
            LoadedAgent {
                definition: def,
                source_path: std::path::PathBuf::new(),
                loaded_at,
                status: AgentStatus::Ready,
            },
        );

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Run management
    // -------------------------------------------------------------------------

    /// Start a new agent run and return the run ID and an event receiver.
    ///
    /// Spawns a tokio task that drives the ReAct loop and updates the `RunState`
    /// on completion or failure.
    pub async fn start_run(
        self: &Arc<Self>,
        agent_name: &str,
        input: &str,
    ) -> Result<(String, EventReceiver), AgentixError> {
        // Look up agent
        let loaded = self
            .agents
            .get(agent_name)
            .ok_or_else(|| {
                AgentixError::Runtime(format!("Agent '{}' not found", agent_name))
            })?;

        let definition = loaded.definition.clone();
        drop(loaded); // Release dashmap guard

        // Create run ID and channels
        let run_id = Uuid::new_v4().to_string();
        let (event_tx, event_rx): (
            broadcast::Sender<ReActEvent>,
            broadcast::Receiver<ReActEvent>,
        ) = broadcast::channel(256);
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        // Store initial run state
        self.runs.insert(
            run_id.clone(),
            RunState {
                id: run_id.clone(),
                agent_name: agent_name.to_string(),
                started_at: chrono::Utc::now(),
                completed_at: None,
                status: RunStatus::Running,
                input: input.to_string(),
                output: None,
                iterations: 0,
                events: Vec::new(),
                cancel_tx: Some(cancel_tx),
            },
        );

        // Mark agent as running
        if let Some(mut agent) = self.agents.get_mut(agent_name) {
            agent.status = AgentStatus::Running;
        }

        // Spawn the run task
        let manager = self.clone();
        let run_id_task = run_id.clone();
        let agent_name_task = agent_name.to_string();
        let input_task = input.to_string();
        let event_tx_task = event_tx.clone();

        tokio::spawn(async move {
            let result = manager
                .execute_run(&definition, &input_task, event_tx_task, cancel_rx)
                .await;

            // Update run state on completion
            if let Some(mut run) = manager.runs.get_mut(&run_id_task) {
                run.completed_at = Some(chrono::Utc::now());
                match result {
                    Ok(run_result) => {
                        run.status = RunStatus::Completed;
                        run.output = Some(run_result.output);
                        run.iterations = run_result.iterations;
                    }
                    Err(err) => {
                        // Check if it was cancelled
                        if let Some(RunStatus::Cancelled) = Some(&run.status) {
                            // Already marked cancelled
                        } else {
                            run.status = RunStatus::Failed(err.to_string());
                        }
                    }
                }
            }

            // Mark agent as ready again
            if let Some(mut agent) = manager.agents.get_mut(&agent_name_task) {
                agent.status = AgentStatus::Ready;
            }
        });

        Ok((run_id, event_rx))
    }

    /// Internal: drive the ReAct loop with cancellation support.
    async fn execute_run(
        &self,
        definition: &AgentDefinition,
        input: &str,
        event_tx: broadcast::Sender<ReActEvent>,
        cancel_rx: oneshot::Receiver<()>,
    ) -> Result<RunResult, AgentixError> {
        // Create the LLM model from the definition's model_preferred field
        let model = create_provider_from_definition(definition, &self.workspace_config)?;

        let config = ReActConfig::from_definition(definition);

        // Build composite tool executor: CLI + MCP + WASM
        let cli_executor = CliToolExecutor::new();
        let mcp_executor = McpToolExecutor::new(definition.mcp_servers.clone());
        // WasmToolExecutor needs the agents directory for resolving .wasm files
        let agents_dir = self
            .agents_dir
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("agents"));
        let wasm_executor = WasmToolExecutor::new(agents_dir);
        let composite = CompositeToolExecutor::new(cli_executor, mcp_executor);
        // TODO: wire wasm_executor into CompositeToolExecutor when it supports 3-way dispatch
        let _ = wasm_executor; // Reserved for future 3-way composite
        let tool_executor = Arc::new(composite) as Arc<dyn ToolExecutor>;

        let engine = ReActEngine::new(model, tool_executor, config)
            .with_event_stream(event_tx.clone());

        // Race between the run and the cancel signal
        tokio::select! {
            result = engine.run(definition, input) => result,
            _ = cancel_rx => {
                let _ = event_tx.send(ReActEvent::Error("Run cancelled".to_string()));
                Err(AgentixError::Runtime("Run cancelled".to_string()))
            }
        }
    }

    /// Run an agent triggered by a `TriggerEvent`.
    ///
    /// Serializes the event as the user input (JSON) and starts a new agent run.
    /// Returns the run ID for tracking purposes.
    ///
    /// This method is called by the trigger dispatcher background task and by the
    /// `/api/v1/agents/:name/trigger` HTTP endpoint.
    pub async fn run_agent_with_trigger(
        self: &Arc<Self>,
        agent_name: &str,
        trigger_event: agentix_core::TriggerEvent,
    ) -> Result<String, AgentixError> {
        // Serialize TriggerEvent as the agent's user input
        let input = serde_json::to_string(&trigger_event)
            .map_err(|e| AgentixError::Runtime(format!("Failed to serialize TriggerEvent: {}", e)))?;

        // Delegate to existing start_run — returns (run_id, event_receiver)
        let (run_id, _event_rx) = self.start_run(agent_name, &input).await?;
        Ok(run_id)
    }

    /// Dispatch an incoming webhook HTTP payload to the agent subscribed to `trigger_id`.
    ///
    /// Looks up which agent has a `webhook` trigger whose auto-generated id matches
    /// `trigger_id` (format: `{agent_name}-webhook-{index}`), then fires a
    /// `TriggerEvent` by sending it through the trigger dispatcher channel.
    ///
    /// Returns the agent name on success.
    /// Returns `Err` if no agent is registered for `trigger_id` or the channel is closed.
    pub async fn dispatch_webhook_payload(
        &self,
        trigger_id: &str,
        payload: serde_json::Value,
        source: agentix_core::TriggerSource,
        context: std::collections::HashMap<String, String>,
    ) -> Result<String, AgentixError> {
        // Find the agent that owns this trigger_id by examining agent trigger configs.
        // trigger_id format: "{agent_name}-{type}-{index}"
        let agent_name = self
            .agents
            .iter()
            .find(|entry| {
                let def = &entry.value().definition;
                def.triggers.iter().enumerate().any(|(i, t)| {
                    let id = format!("{}-{}-{}", def.name, t.trigger_type, i);
                    id == trigger_id
                })
            })
            .map(|entry| entry.value().definition.name.clone())
            .ok_or_else(|| {
                AgentixError::Runtime(format!(
                    "No agent registered for trigger_id '{}'",
                    trigger_id
                ))
            })?;

        let event = agentix_core::TriggerEvent {
            source,
            payload,
            context,
            fired_at: chrono::Utc::now(),
            trigger_id: trigger_id.to_string(),
        };

        self.trigger_event_tx
            .send((agent_name.clone(), event))
            .await
            .map_err(|e| AgentixError::Runtime(format!("Trigger channel closed: {}", e)))?;

        Ok(agent_name)
    }

    /// Stop a running agent run by sending a cancel signal.
    pub fn stop_run(&self, run_id: &str) -> Result<(), AgentixError> {
        let mut run = self.runs.get_mut(run_id).ok_or_else(|| {
            AgentixError::Runtime(format!("Run '{}' not found", run_id))
        })?;

        // Take the cancel sender (consumes it)
        if let Some(cancel_tx) = run.cancel_tx.take() {
            let _ = cancel_tx.send(());
        }
        run.status = RunStatus::Cancelled;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Query methods
    // -------------------------------------------------------------------------

    /// List all loaded agents as summaries.
    pub fn list_agents(&self) -> Vec<AgentSummary> {
        let mut agents: Vec<AgentSummary> = self
            .agents
            .iter()
            .map(|entry| {
                let a = entry.value();
                AgentSummary {
                    name: a.definition.name.clone(),
                    description: a.definition.description.clone(),
                    model: a.definition.model_preferred.clone(),
                    status: AgentStatusSummary::from(&a.status),
                    loaded_at: a.loaded_at,
                }
            })
            .collect();

        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Get a single agent summary by name.
    pub fn get_agent(&self, name: &str) -> Option<AgentSummary> {
        self.agents.get(name).map(|entry| {
            let a = entry.value();
            AgentSummary {
                name: a.definition.name.clone(),
                description: a.definition.description.clone(),
                model: a.definition.model_preferred.clone(),
                status: AgentStatusSummary::from(&a.status),
                loaded_at: a.loaded_at,
            }
        })
    }

    /// List all runs for a specific agent.
    pub fn list_runs(&self, agent_name: &str) -> Vec<RunSummary> {
        let mut runs: Vec<RunSummary> = self
            .runs
            .iter()
            .filter(|entry| entry.value().agent_name == agent_name)
            .map(|entry| {
                let r = entry.value();
                RunSummary {
                    id: r.id.clone(),
                    agent_name: r.agent_name.clone(),
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    status: r.status.clone(),
                    input: r.input.clone(),
                    output: r.output.clone(),
                    iterations: r.iterations,
                }
            })
            .collect();

        // Sort by started_at descending (most recent first)
        runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        runs
    }

    /// Get the stored events for a completed run.
    pub fn get_run_events(&self, run_id: &str) -> Option<RunSummary> {
        self.runs.get(run_id).map(|entry| {
            let r = entry.value();
            RunSummary {
                id: r.id.clone(),
                agent_name: r.agent_name.clone(),
                started_at: r.started_at,
                completed_at: r.completed_at,
                status: r.status.clone(),
                input: r.input.clone(),
                output: r.output.clone(),
                iterations: r.iterations,
            }
        })
    }
}

// ---------------------------------------------------------------------------
// LLM provider factory helper
// ---------------------------------------------------------------------------

/// Create an LLM model from an `AgentDefinition`.
///
/// Parses `model_preferred` (format: `provider/model`), looks up credentials
/// in `workspace_config.spec.providers`, and creates the provider.
fn create_provider_from_definition(
    definition: &AgentDefinition,
    workspace_config: &Option<WorkspaceConfig>,
) -> Result<Arc<dyn agentix_core::Model + Send + Sync>, AgentixError> {
    let model_preferred = definition.model_preferred.as_deref().ok_or_else(|| {
        AgentixError::Config(format!(
            "Agent '{}' has no model configured. Set model.preferred in agent.yaml or workspace defaults.",
            definition.name
        ))
    })?;

    // Parse "provider/model"
    let slash_pos = model_preferred.find('/').ok_or_else(|| {
        AgentixError::Config(format!(
            "model_preferred '{}' must be in 'provider/model' format (e.g. 'anthropic/claude-sonnet-4-6')",
            model_preferred
        ))
    })?;

    let provider_name = &model_preferred[..slash_pos];
    let model_name = &model_preferred[slash_pos + 1..];

    // Parse provider enum
    let provider = parse_provider_name(provider_name)?;

    // Look up credentials from workspace config
    let (api_key, endpoint) = if let Some(ws) = workspace_config {
        let provider_cfg = ws.spec.providers.get(provider_name);
        (
            provider_cfg.and_then(|p| p.api_key.clone()),
            provider_cfg.and_then(|p| p.base_url.clone()),
        )
    } else {
        // Fall back to environment variables
        (get_default_api_key_from_env(provider_name), None)
    };

    let config = ModelConfig {
        model: model_name.to_string(),
        provider,
        api_key,
        endpoint,
        temperature: 0.7,
        max_tokens: None,
        timeout_secs: definition.timeout_secs,
        headers: std::collections::HashMap::new(),
        extra: std::collections::HashMap::new(),
    };

    // Use a blocking runtime call to invoke the async factory
    // ProviderFactory::create is async but most providers are sync-under-the-hood
    // Model: Send + Sync so we can safely upcast.
    let model = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(ProviderFactory::create(config))
    })?;

    // Model trait requires Send + Sync so Box<dyn Model> coerces to Box<dyn Model + Send + Sync>
    let model_boxed: Box<dyn agentix_core::Model + Send + Sync> = model;
    Ok(Arc::from(model_boxed))
}

/// Parse a provider name string into a `ModelProvider`.
fn parse_provider_name(name: &str) -> Result<ModelProvider, AgentixError> {
    match name {
        "anthropic" => Ok(ModelProvider::Anthropic),
        "openai" => Ok(ModelProvider::OpenAI),
        "google" | "gemini" => Ok(ModelProvider::Google),
        "groq" => Ok(ModelProvider::Groq),
        "bedrock" | "aws" => Ok(ModelProvider::Bedrock),
        "azure" => Ok(ModelProvider::Azure),
        "ollama" => Ok(ModelProvider::Ollama),
        other => Err(AgentixError::Config(format!(
            "Unknown provider '{}'. Supported: anthropic, openai, google, groq, bedrock, ollama",
            other
        ))),
    }
}

/// Get the default API key from environment variables for a provider.
fn get_default_api_key_from_env(provider_name: &str) -> Option<String> {
    let var_name = match provider_name {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "google" | "gemini" => "GOOGLE_API_KEY",
        "groq" => "GROQ_API_KEY",
        _ => return None,
    };
    std::env::var(var_name).ok()
}

// ---------------------------------------------------------------------------
// CronTriggerImpl — minimal cron trigger for gateway-level scheduling
// ---------------------------------------------------------------------------

/// Normalize a 5-field cron expression to 7-field format required by the `cron` crate.
fn normalize_cron_expr(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => format!("0 {} *", expr),
        6 => format!("{} *", expr),
        _ => expr.to_string(),
    }
}

/// Minimal cron trigger implementation for use within agentix-runtime.
///
/// This avoids a circular dependency: agentix-runtime cannot import agentix-triggers
/// (which depends on agentix-runtime). This struct duplicates just enough logic
/// from `agentix_triggers::CronTrigger` for the gateway to schedule agents.
struct CronTriggerImpl {
    id: String,
    expression: String,
    agent_name: String,
    stop_tx: tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

#[async_trait::async_trait]
impl agentix_core::TriggerTrait for CronTriggerImpl {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> agentix_core::TriggerSource {
        agentix_core::TriggerSource::Cron
    }

    async fn start(
        &self,
        sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        use cron::Schedule;
        use std::str::FromStr;

        let normalized = normalize_cron_expr(&self.expression);
        let schedule = Schedule::from_str(&normalized)
            .map_err(|e| AgentixError::Config(format!("Cron parse error: {}", e)))?;

        let agent_name = self.agent_name.clone();
        let trigger_id = self.id.clone();
        let expression = self.expression.clone();
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        *self.stop_tx.lock().await = Some(stop_tx);

        tokio::spawn(async move {
            for next in schedule.upcoming(chrono::Utc) {
                let now = chrono::Utc::now();
                let duration = (next - now).to_std().unwrap_or_default();

                tokio::select! {
                    _ = tokio::time::sleep(duration) => {
                        let event = TriggerEvent::new(
                            agentix_core::TriggerSource::Cron,
                            serde_json::json!({
                                "expression": expression,
                                "scheduled_at": next.to_rfc3339(),
                            }),
                            &trigger_id,
                        );
                        if sender.send((agent_name.clone(), event)).await.is_err() {
                            break;
                        }
                    }
                    _ = &mut stop_rx => break,
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        if let Some(tx) = self.stop_tx.lock().await.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ChannelMentionTriggerImpl — passive trigger for Slack/Discord/Telegram
// ---------------------------------------------------------------------------

/// Which messaging platform this trigger listens on.
enum ChannelPlatformImpl {
    Slack { signing_secret: Option<String> },
    Discord,
    Telegram,
}

/// Minimal channel mention trigger for use within agentix-runtime.
///
/// Passive trigger: stores the sender at start() time.
/// The /webhooks/:trigger_id route calls dispatch_webhook_payload() which
/// sends a pre-built TriggerEvent; this struct's start() only stores the sender
/// for potential future direct dispatch.
struct ChannelMentionTriggerImpl {
    id: String,
    agent_name: String,
    platform: ChannelPlatformImpl,
    sender: tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<(String, TriggerEvent)>>>,
}

#[async_trait::async_trait]
impl agentix_core::TriggerTrait for ChannelMentionTriggerImpl {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> agentix_core::TriggerSource {
        match &self.platform {
            ChannelPlatformImpl::Slack { .. } => agentix_core::TriggerSource::Slack,
            ChannelPlatformImpl::Discord => agentix_core::TriggerSource::Discord,
            ChannelPlatformImpl::Telegram => agentix_core::TriggerSource::Telegram,
        }
    }

    async fn start(
        &self,
        sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        // Passive trigger: just store the sender for potential async dispatch
        *self.sender.lock().await = Some(sender);
        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        *self.sender.lock().await = None;
        Ok(())
    }
}
