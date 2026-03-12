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
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

use agentix_core::{
    AgentDefinition, AgentixError, AgentLoader, FlatYamlLoader, ModelConfig,
    ModelProvider, WorkspaceConfig,
};
use agentix_llm::ProviderFactory;

use crate::executor::react_loop::{ReActConfig, ReActEngine, ReActEvent, RunResult, ToolExecutor};
use crate::streaming::EventReceiver;
use crate::tools::{CliToolExecutor, CompositeToolExecutor, McpToolExecutor};

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
}

impl AgentManager {
    /// Create a new `AgentManager` wrapped in an `Arc` for sharing.
    pub fn new(workspace_config: Option<WorkspaceConfig>) -> Arc<Self> {
        Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
        })
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
        // Build composite tool executor: CLI + MCP
        let cli_executor = CliToolExecutor::new();
        let mcp_executor = McpToolExecutor::new(definition.mcp_servers.clone());
        let tool_executor = Arc::new(CompositeToolExecutor::new(cli_executor, mcp_executor))
            as Arc<dyn ToolExecutor>;

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
