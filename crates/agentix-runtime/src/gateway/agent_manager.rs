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
    AgentDefinition, AgentixError, AgentLoader, AuthService, FlatYamlLoader, ModelConfig,
    ModelProvider, PkceState, ProviderMode, TriggerEvent, TriggerRunRegistry, WorkspaceConfig,
    coordination::{AgentInbox, CoordinatorProtocol, DelegationMessage, DelegationResult, DelegationStatus},
    TriggerSource,
};
use agentix_llm::ProviderFactory;

use crate::audit_store::{AuditStore, AuditEntry, AuditEventType, AuditOutcome};
use crate::cost_store::CostStore;
use crate::executor::react_loop::{ReActConfig, ReActEngine, ReActEvent, RunResult, ToolExecutor};
use crate::gateway::run_store::{RunRecord, RunStore};
use crate::otel_exporter::OtelExporter;
use crate::streaming::EventReceiver;
use crate::telemetry::TraceCollector;
use crate::tools::{CliToolExecutor, CompositeToolExecutor, McpToolExecutor, WasmToolExecutor};
use crate::trace_store::TraceStore;
use agentix_core::telemetry::TraceContext;
use serde::Serialize;

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
    /// Persistent SQLite run history store.
    pub run_store: Arc<RunStore>,
    /// Persistent SQLite cost tracking store (Phase 17).
    pub cost_store: Arc<CostStore>,
    /// Persistent SQLite trace store for spans and structured logs (Phase 18).
    pub trace_store: Arc<TraceStore>,
    /// Persistent SQLite audit trail store (Phase 19).
    pub audit_store: Arc<AuditStore>,
    /// Persistent SQLite approval request store (Phase 20).
    pub approval_store: Arc<crate::approval_store::ApprovalStore>,
    /// Optional OTel exporter for pushing spans to external collectors (Phase 18).
    otel_exporter: Option<OtelExporter>,
    /// Per-agent bounded inboxes for coordination (COORD-01).
    /// One inbox per registered agent, keyed by agent name.
    inboxes: DashMap<String, Arc<AgentInbox>>,
    /// Multi-channel gateway manager (Phase 21).
    channel_manager: Option<crate::channels::ChannelGatewayManager>,
    /// Runtime provider overrides set via the `/api/v1/providers` endpoint.
    /// Checked before `workspace_config.spec.providers` during model creation.
    provider_overrides: DashMap<String, agentix_core::ProviderConfig>,
    /// AuthService for OAuth credential management (Phase 23 — LLM Subscription Proxy).
    /// Stored as `Arc` so it can be shared with axum handler state without cloning the manager.
    pub auth_service: Arc<AuthService>,
    /// In-flight PKCE states for UI OAuth flows (gateway callback path).
    ///
    /// Keyed by the `state` parameter in the OAuth authorization URL.
    /// Entries are inserted in `/api/v1/auth/:provider/start` and consumed in
    /// `/api/v1/auth/:provider/callback`. Entries older than 10 minutes are
    /// ignored as a safety measure.
    pub pending_pkce: DashMap<String, (String, PkceState, std::time::Instant)>,
}

impl AgentManager {
    /// Open the RunStore at the given path (`:memory:` for tests, real path for production).
    fn open_run_store_at(path: &str) -> Arc<RunStore> {
        match RunStore::open(path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                tracing::warn!("Failed to open run store at '{}': {} — falling back to in-memory", path, e);
                Arc::new(RunStore::open(":memory:").expect("in-memory SQLite"))
            }
        }
    }

    /// Open the CostStore at the given path (`:memory:` for tests, real path for production).
    fn open_cost_store_at(path: &str) -> Arc<CostStore> {
        match CostStore::open(path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                tracing::warn!("Failed to open cost store at '{}': {} — falling back to in-memory", path, e);
                Arc::new(CostStore::open(":memory:").expect("in-memory SQLite"))
            }
        }
    }

    /// Open the TraceStore at the given path (`:memory:` for tests, real path for production).
    fn open_trace_store_at(path: &str) -> Arc<TraceStore> {
        match TraceStore::open(path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                tracing::warn!("Failed to open trace store at '{}': {} — falling back to in-memory", path, e);
                Arc::new(TraceStore::open(":memory:").expect("in-memory SQLite"))
            }
        }
    }

    fn open_audit_store_at(path: &str) -> Arc<AuditStore> {
        match AuditStore::open(path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                tracing::warn!("Failed to open audit store at '{}': {} — falling back to in-memory", path, e);
                Arc::new(AuditStore::open(":memory:").expect("in-memory SQLite"))
            }
        }
    }

    /// Open the ApprovalStore at the given path (`:memory:` for tests, real path for production).
    fn open_approval_store_at(path: &str) -> Arc<crate::approval_store::ApprovalStore> {
        match crate::approval_store::ApprovalStore::open(path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                tracing::warn!("Failed to open approval store at '{}': {} — falling back to in-memory", path, e);
                Arc::new(crate::approval_store::ApprovalStore::open(":memory:").expect("in-memory SQLite"))
            }
        }
    }

    /// Build an OtelExporter from workspace config if telemetry is configured.
    /// Build the ChannelGatewayManager from workspace config (Phase 21).
    fn build_channel_manager(
        workspace_config: &Option<WorkspaceConfig>,
    ) -> Option<crate::channels::ChannelGatewayManager> {
        workspace_config.as_ref().and_then(|wc| {
            wc.spec.channels.as_ref().and_then(|channels| {
                match crate::channels::ChannelGatewayManager::new(channels.clone()) {
                    Ok(mgr) => {
                        tracing::info!("Channel gateway manager initialized");
                        Some(mgr)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize channel gateway: {}", e);
                        None
                    }
                }
            })
        })
    }

    /// Get a reference to the channel gateway manager (Phase 21).
    pub fn channel_manager(&self) -> Option<&crate::channels::ChannelGatewayManager> {
        self.channel_manager.as_ref()
    }

    /// Build an `AuthService` using the data directory for profile storage.
    ///
    /// Creates `<data_dir>/.agentix-auth/` if it doesn't exist.
    /// Falls back to a temp-directory-based store for in-memory/test configs.
    fn build_auth_service(data_dir: Option<&std::path::Path>) -> Arc<AuthService> {
        let state_dir = match data_dir {
            Some(dir) => {
                let auth_dir = dir.join(".agentix-auth");
                let _ = std::fs::create_dir_all(&auth_dir);
                auth_dir
            }
            None => {
                // For tests / in-memory mode: use a temporary directory
                std::env::temp_dir().join("agentix-auth-test")
            }
        };
        Arc::new(AuthService::new(&state_dir, true))
    }

    fn build_otel_exporter(workspace_config: &Option<WorkspaceConfig>) -> Option<OtelExporter> {
        workspace_config.as_ref().and_then(|wc| {
            wc.spec.telemetry.as_ref().and_then(|tc| {
                if tc.enabled && tc.otlp_endpoint.is_some() {
                    Some(OtelExporter::new(tc.clone()))
                } else {
                    None
                }
            })
        })
    }

    /// Ensure the data directory exists and return paths for run, cost, and trace DBs.
    fn data_db_paths(data_dir: &std::path::Path) -> (String, String, String, String, String) {
        if let Err(e) = std::fs::create_dir_all(data_dir) {
            tracing::warn!("Failed to create data dir '{}': {}", data_dir.display(), e);
        }
        let run_db = data_dir.join("runs.db");
        let cost_db = data_dir.join("cost.db");
        let trace_db = data_dir.join("trace.db");
        let audit_db = data_dir.join("audit.db");
        let approval_db = data_dir.join("approval.db");
        (
            run_db.to_str().unwrap_or("./agentix-runs.db").to_string(),
            cost_db.to_str().unwrap_or("./agentix-cost.db").to_string(),
            trace_db.to_str().unwrap_or("./agentix-trace.db").to_string(),
            audit_db.to_str().unwrap_or("./agentix-audit.db").to_string(),
            approval_db.to_str().unwrap_or("./agentix-approval.db").to_string(),
        )
    }

    /// Create a new `AgentManager` wrapped in an `Arc` for sharing.
    ///
    /// Uses in-memory SQLite for both run and cost stores (suitable for tests).
    /// In production, call `new_with_data_dir` instead.
    pub fn new(workspace_config: Option<WorkspaceConfig>) -> Arc<Self> {
        let (trigger_event_tx, trigger_event_rx) = mpsc::channel::<(String, TriggerEvent)>(256);
        let run_store = Self::open_run_store_at(":memory:");
        let cost_store = Self::open_cost_store_at(":memory:");
        let trace_store = Self::open_trace_store_at(":memory:");
        let audit_store = Self::open_audit_store_at(":memory:");
        let approval_store = Self::open_approval_store_at(":memory:");
        let otel_exporter = Self::build_otel_exporter(&workspace_config);
        let channel_manager = Self::build_channel_manager(&workspace_config);
        let auth_service = Self::build_auth_service(None);
        let manager = Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
            agents_dir: None,
            trigger_registry: Mutex::new(TriggerRunRegistry::new()),
            trigger_event_tx,
            run_store,
            cost_store,
            trace_store,
            audit_store,
            approval_store,
            otel_exporter,
            inboxes: DashMap::new(),
            channel_manager,
            provider_overrides: DashMap::new(),
            auth_service,
            pending_pkce: DashMap::new(),
        });
        // Spawn the trigger dispatcher background task
        manager.clone().spawn_trigger_dispatcher(trigger_event_rx);
        manager
    }

    /// Create a new `AgentManager` with persistent SQLite stores under `data_dir`.
    ///
    /// Creates `data_dir/runs.db` and `data_dir/cost.db` on startup.
    /// Used by `Gateway::start` for production deployments.
    pub fn new_with_data_dir(workspace_config: Option<WorkspaceConfig>, data_dir: &std::path::Path) -> Arc<Self> {
        let (trigger_event_tx, trigger_event_rx) = mpsc::channel::<(String, TriggerEvent)>(256);
        let (run_db_path, cost_db_path, trace_db_path, audit_db_path, approval_db_path) = Self::data_db_paths(data_dir);
        let run_store = Self::open_run_store_at(&run_db_path);
        let cost_store = Self::open_cost_store_at(&cost_db_path);
        let trace_store = Self::open_trace_store_at(&trace_db_path);
        let audit_store = Self::open_audit_store_at(&audit_db_path);
        let approval_store = Self::open_approval_store_at(&approval_db_path);
        let otel_exporter = Self::build_otel_exporter(&workspace_config);
        let channel_manager = Self::build_channel_manager(&workspace_config);
        let auth_service = Self::build_auth_service(Some(data_dir));
        tracing::info!("Run store: {}", run_db_path);
        tracing::info!("Cost store: {}", cost_db_path);
        tracing::info!("Trace store: {}", trace_db_path);
        tracing::info!("Audit store: {}", audit_db_path);
        tracing::info!("Approval store: {}", approval_db_path);
        let manager = Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
            agents_dir: None,
            trigger_registry: Mutex::new(TriggerRunRegistry::new()),
            trigger_event_tx,
            run_store,
            cost_store,
            trace_store,
            audit_store,
            approval_store,
            otel_exporter,
            inboxes: DashMap::new(),
            channel_manager,
            provider_overrides: DashMap::new(),
            auth_service,
            pending_pkce: DashMap::new(),
        });
        manager.clone().spawn_trigger_dispatcher(trigger_event_rx);
        manager
    }

    /// Create an `AgentManager` with a known agents directory (used for WASM tool resolution).
    pub fn with_agents_dir(workspace_config: Option<WorkspaceConfig>, agents_dir: std::path::PathBuf) -> Arc<Self> {
        let (trigger_event_tx, trigger_event_rx) = mpsc::channel::<(String, TriggerEvent)>(256);
        let run_store = Self::open_run_store_at(":memory:");
        let cost_store = Self::open_cost_store_at(":memory:");
        let trace_store = Self::open_trace_store_at(":memory:");
        let audit_store = Self::open_audit_store_at(":memory:");
        let approval_store = Self::open_approval_store_at(":memory:");
        let otel_exporter = Self::build_otel_exporter(&workspace_config);
        let channel_manager = Self::build_channel_manager(&workspace_config);
        let auth_service = Self::build_auth_service(None);
        let manager = Arc::new(Self {
            agents: DashMap::new(),
            runs: DashMap::new(),
            workspace_config,
            agents_dir: Some(agents_dir),
            trigger_registry: Mutex::new(TriggerRunRegistry::new()),
            trigger_event_tx,
            run_store,
            cost_store,
            trace_store,
            audit_store,
            approval_store,
            otel_exporter,
            inboxes: DashMap::new(),
            channel_manager,
            provider_overrides: DashMap::new(),
            auth_service,
            pending_pkce: DashMap::new(),
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
                    // Create inbox for coordination (COORD-01)
                    self.inboxes.entry(name.clone()).or_insert_with(|| Arc::new(AgentInbox::default_capacity()));
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
        // Create inbox for coordination (COORD-01)
        self.inboxes.entry(name.clone()).or_insert_with(|| Arc::new(AgentInbox::default_capacity()));

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

        // Check daily budget limit before starting run (COST-04)
        if let Some(budget) = &definition.budget {
            if let Some(daily_limit) = budget.daily_limit_usd {
                match self.cost_store.get_today_spend(agent_name) {
                    Ok(today_spend) if today_spend >= daily_limit => {
                        return Err(AgentixError::Runtime(format!(
                            "Daily budget limit exceeded for agent '{}': limit=${:.4}, today=${:.4}",
                            agent_name, daily_limit, today_spend
                        )));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to check daily budget for '{}': {}", agent_name, e);
                    }
                    _ => {}
                }
            }
        }

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

        // Capture model info for cost recording (before spawning task)
        let agent_model = definition.model_preferred.clone().unwrap_or_default();
        let agent_provider = {
            // Extract provider from "provider/model" format
            agent_model.split('/').next().unwrap_or("unknown").to_string()
        };
        let agent_model_name = {
            // Extract model name after the "/"
            agent_model.splitn(2, '/').nth(1).unwrap_or(&agent_model).to_string()
        };

        // Audit: log AgentStart event (non-critical)
        {
            let audit_entry = AuditEntry {
                id: None,
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::AgentStart,
                agent_name: agent_name.to_string(),
                run_id: Some(run_id.clone()),
                actor: "system".to_string(),
                action: format!("Starting agent run {}", run_id),
                outcome: AuditOutcome::Success,
                details: std::collections::HashMap::new(),
                trace_id: None,
            };
            if let Err(e) = self.audit_store.log_event(audit_entry) {
                tracing::warn!("Failed to log audit AgentStart: {}", e);
            }
        }

        // Spawn the run task
        let manager = self.clone();
        let run_id_task = run_id.clone();
        let agent_name_task = agent_name.to_string();
        let input_task = input.to_string();
        let event_tx_task = event_tx.clone();

        tokio::spawn(async move {
            let result = manager
                .execute_run(&definition, &input_task, &run_id_task, event_tx_task, cancel_rx)
                .await;

            // Update run state on completion
            if let Some(mut run) = manager.runs.get_mut(&run_id_task) {
                run.completed_at = Some(chrono::Utc::now());
                match &result {
                    Ok((run_result, _)) => {
                        run.status = RunStatus::Completed;
                        run.output = Some(run_result.output.clone());
                        run.iterations = run_result.iterations;

                        // Audit: log AgentComplete (non-critical)
                        let mut details = std::collections::HashMap::new();
                        details.insert("iterations".to_string(), serde_json::json!(run_result.iterations));
                        details.insert("tool_calls_count".to_string(), serde_json::json!(run_result.tool_calls.len()));
                        let audit_entry = AuditEntry {
                            id: None,
                            timestamp: chrono::Utc::now(),
                            event_type: AuditEventType::AgentComplete,
                            agent_name: agent_name_task.clone(),
                            run_id: Some(run_id_task.clone()),
                            actor: "system".to_string(),
                            action: format!("Agent run {} completed", run_id_task),
                            outcome: AuditOutcome::Success,
                            details,
                            trace_id: None,
                        };
                        if let Err(e) = manager.audit_store.log_event(audit_entry) {
                            tracing::warn!("Failed to log audit AgentComplete: {}", e);
                        }
                    }
                    Err(err) => {
                        // Check if it was cancelled
                        if let Some(RunStatus::Cancelled) = Some(&run.status) {
                            // Already marked cancelled
                        } else {
                            run.status = RunStatus::Failed(err.to_string());
                        }

                        // Audit: log AgentError (non-critical)
                        let audit_entry = AuditEntry {
                            id: None,
                            timestamp: chrono::Utc::now(),
                            event_type: AuditEventType::AgentError,
                            agent_name: agent_name_task.clone(),
                            run_id: Some(run_id_task.clone()),
                            actor: "system".to_string(),
                            action: format!("Agent run {} failed", run_id_task),
                            outcome: AuditOutcome::Failure(err.to_string()),
                            details: std::collections::HashMap::new(),
                            trace_id: None,
                        };
                        if let Err(e) = manager.audit_store.log_event(audit_entry) {
                            tracing::warn!("Failed to log audit AgentError: {}", e);
                        }
                    }
                }
            }

            // Record cost after successful run (COST-01, COST-02)
            if let Ok((run_result, _)) = &result {
                if run_result.total_input_tokens > 0 || run_result.total_output_tokens > 0 {
                    use agentix_core::{CostRecord, default_model_pricing, calculate_cost};
                    let pricing_table = default_model_pricing();
                    let pricing_key = format!("{}/{}", agent_provider, agent_model_name);
                    let cost_usd = if let Some(pricing) = pricing_table.get(&pricing_key) {
                        calculate_cost(run_result.total_input_tokens, run_result.total_output_tokens, pricing)
                    } else {
                        run_result.total_cost_usd
                    };
                    let record = CostRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_name: agent_name_task.clone(),
                        run_id: run_id_task.clone(),
                        model: agent_model_name.clone(),
                        provider: agent_provider.clone(),
                        input_tokens: run_result.total_input_tokens,
                        output_tokens: run_result.total_output_tokens,
                        cost_usd,
                        actual_cost_usd: None,
                        recorded_at: chrono::Utc::now(),
                    };
                    if let Err(e) = manager.cost_store.insert_record(&record) {
                        tracing::warn!("Failed to record cost for run '{}': {}", run_id_task, e);
                    }
                }
            }

            // Persist trace spans and logs after run (Phase 18 — TELE-01/TELE-04)
            if let Ok((_, Some(ref collector))) = &result {
                let spans = collector.get_spans();
                let logs = collector.get_logs();
                if let Err(e) = manager.trace_store.insert_spans(&spans, &agent_name_task, &run_id_task) {
                    tracing::warn!("Failed to persist trace spans for run '{}': {}", run_id_task, e);
                }
                if let Err(e) = manager.trace_store.insert_logs(&logs) {
                    tracing::warn!("Failed to persist structured logs for run '{}': {}", run_id_task, e);
                }

                // Push spans to OTel collector if configured (fire-and-forget)
                if let Some(ref exporter) = manager.otel_exporter {
                    let exporter = exporter.clone();
                    let spans = spans.clone();
                    tokio::spawn(async move {
                        if let Err(e) = exporter.push_to_collector(&spans).await {
                            tracing::warn!("Failed to push spans to OTel collector: {}", e);
                        }
                    });
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
        run_id: &str,
        event_tx: broadcast::Sender<ReActEvent>,
        cancel_rx: oneshot::Receiver<()>,
    ) -> Result<(RunResult, Option<TraceCollector>), AgentixError> {
        // Create the LLM model from the definition's model_preferred field
        // Pass auth_service for subscription-mode token resolution (Phase 23)
        let model = create_provider_from_definition(definition, &self.workspace_config, &self.provider_overrides, Some(&self.auth_service)).await?;

        let mut config = ReActConfig::from_definition(definition);

        // Phase 19: Wire security components into ReActConfig
        let security_config = self.workspace_config.as_ref()
            .and_then(|wc| wc.spec.security.clone())
            .unwrap_or_default();
        if security_config.ssrf_protection {
            config.ssrf_guard = Some(Arc::new(
                agentix_core::SsrfGuard::with_allowed(security_config.allowed_private_endpoints.clone())
            ));
        }
        config.audit_store = Some(self.audit_store.clone());
        // SecretRedactor is populated per-agent when secrets are available
        config.secret_redactor = None;

        // Phase 20: Wire approval components into ReActConfig
        config.approval_store = Some(self.approval_store.clone());
        config.run_id = Some(run_id.to_string());

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

        // Create trace collector for telemetry (Phase 18)
        let trace_ctx = TraceContext::new_root(&definition.name, run_id);
        let trace_collector = TraceCollector::new(trace_ctx);

        let engine = ReActEngine::new(model, tool_executor, config)
            .with_event_stream(event_tx.clone())
            .with_trace_collector(trace_collector.clone());

        // Race between the run and the cancel signal
        let result = tokio::select! {
            result = engine.run(definition, input) => result,
            _ = cancel_rx => {
                let _ = event_tx.send(ReActEvent::Error("Run cancelled".to_string()));
                Err(AgentixError::Runtime("Run cancelled".to_string()))
            }
        };

        result.map(|r| (r, Some(trace_collector)))
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
        let trigger_source = trigger_event.source.to_string();
        let trigger_id_val = trigger_event.trigger_id.clone();

        // Serialize TriggerEvent as the agent's user input
        let input = serde_json::to_string(&trigger_event)
            .map_err(|e| AgentixError::Runtime(format!("Failed to serialize TriggerEvent: {}", e)))?;

        // Persist a RunRecord before starting
        let run_id_for_record = Uuid::new_v4().to_string();
        let input_summary = if input.len() > 200 { input[..200].to_string() } else { input.clone() };
        let record = RunRecord {
            id: run_id_for_record.clone(),
            agent_name: agent_name.to_string(),
            trigger_source,
            trigger_id: Some(trigger_id_val),
            started_at: chrono::Utc::now(),
            ended_at: None,
            duration_ms: None,
            status: "running".to_string(),
            iterations: None,
            input_summary: Some(input_summary),
            output_summary: None,
            error: None,
        };
        if let Err(e) = self.run_store.insert(&record) {
            tracing::warn!("Failed to persist run record: {}", e);
        }

        // Delegate to existing start_run — returns (run_id, event_receiver)
        let self_clone = self.clone();
        let run_store_clone = self.run_store.clone();
        let record_id = run_id_for_record.clone();
        let agent_name_owned = agent_name.to_string();
        let (run_id, mut event_rx) = self.start_run(agent_name, &input).await?;

        // Spawn a task to update the RunRecord when the run completes
        tokio::spawn(async move {
            let mut output_buf = String::new();
            let mut iter_count: u32 = 0;
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        use crate::executor::react_loop::ReActEvent;
                        match &event {
                            ReActEvent::Complete(result) => {
                                output_buf = result.output.clone();
                                iter_count = result.iterations;
                                break;
                            }
                            ReActEvent::Error(_) => break,
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
            let ended = chrono::Utc::now();
            let out = if output_buf.is_empty() { None } else {
                Some(if output_buf.len() > 500 { output_buf[..500].to_string() } else { output_buf })
            };
            let _ = run_store_clone.update_completed(&record_id, ended, out.as_deref(), Some(iter_count as i32));
            // Dispatch notifications if configured
            self_clone.dispatch_notifications(&agent_name_owned, &record_id).await;
        });

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

    /// Dispatch notifications after a triggered run completes.
    ///
    /// Reads the agent's `notifications:` YAML field and dispatches a run summary
    /// to each configured destination. Currently supports:
    /// - `type: webhook` — HTTP POST with run summary to any URL
    /// - Other types — logged as a warning
    async fn dispatch_notifications(&self, agent_name: &str, run_id: &str) {
        let notifications = {
            match self.agents.get(agent_name) {
                Some(entry) => entry.value().definition.notifications.clone(),
                None => return,
            }
        };

        if notifications.is_empty() {
            return;
        }

        let run_record = self.run_store.get(run_id).ok().flatten();

        for notif in &notifications {
            match notif.notification_type.as_str() {
                "webhook" => {
                    if let Some(url) = &notif.url {
                        let payload = serde_json::json!({
                            "agent": agent_name,
                            "run_id": run_id,
                            "status": run_record.as_ref().map(|r| r.status.as_str()).unwrap_or("unknown"),
                            "output": run_record.as_ref().and_then(|r| r.output_summary.as_deref()).unwrap_or(""),
                            "trigger_source": run_record.as_ref().map(|r| r.trigger_source.as_str()).unwrap_or(""),
                            "duration_ms": run_record.as_ref().and_then(|r| r.duration_ms),
                        });
                        let client = reqwest::Client::new();
                        match client.post(url).json(&payload).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                tracing::info!("Notification sent to {} for run {}", url, run_id);
                            }
                            Ok(resp) => {
                                tracing::warn!("Notification to {} returned status {}", url, resp.status());
                            }
                            Err(e) => {
                                tracing::warn!("Failed to send notification to {}: {}", url, e);
                            }
                        }
                    }
                }
                "log" => {
                    tracing::info!(
                        "Run notification: agent={} run_id={} status={}",
                        agent_name, run_id,
                        run_record.as_ref().map(|r| r.status.as_str()).unwrap_or("unknown")
                    );
                }
                other => {
                    tracing::warn!("Notification type '{}' not yet supported in Phase 15", other);
                }
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
    // Coordination methods (COORD-01 through COORD-05)
    // -------------------------------------------------------------------------

    /// Get the inbox for a named agent.
    /// Returns None if the agent is not registered.
    pub fn inbox_for(&self, agent_name: &str) -> Option<Arc<AgentInbox>> {
        self.inboxes.get(agent_name).map(|v| v.clone())
    }

    /// Delegate a task to a specialist agent and return the result.
    ///
    /// Creates a TriggerEvent wrapping the delegation, fires the target agent,
    /// and waits for the run to complete. Records a delegation audit log entry.
    pub async fn delegate_task(
        self: &Arc<Self>,
        from_agent: &str,
        target: &str,
        task: &str,
        payload: serde_json::Value,
    ) -> Result<DelegationResult, AgentixError> {
        let delegation_id = Uuid::new_v4().to_string();

        if !self.agents.contains_key(target) {
            return Err(AgentixError::Runtime(format!(
                "Delegation target agent '{}' not found",
                target
            )));
        }

        // Build a TriggerEvent representing the delegation
        let event = agentix_core::TriggerEvent {
            source: TriggerSource::Agent,
            payload: serde_json::json!({
                "delegation_id": delegation_id,
                "from_agent": from_agent,
                "task": task,
                "payload": payload,
            }),
            context: {
                let mut ctx = std::collections::HashMap::new();
                ctx.insert("delegation_id".to_string(), delegation_id.clone());
                ctx.insert("from_agent".to_string(), from_agent.to_string());
                ctx
            },
            fired_at: chrono::Utc::now(),
            trigger_id: delegation_id.clone(),
        };

        // Run the target agent; wait for completion via run_agent_with_trigger + run store polling
        let run_id = self.run_agent_with_trigger(target, event).await?;

        // Log delegation to audit trail via run_store metadata
        tracing::info!(
            delegation_id = %delegation_id,
            from_agent = %from_agent,
            target = %target,
            run_id = %run_id,
            "Delegation dispatched"
        );

        // Poll the run store for completion (max 60s)
        let output = self.await_run_completion(&run_id, 60).await;
        let success = output.is_some();

        Ok(DelegationResult {
            delegation_id,
            from_agent: target.to_string(),
            output: output.unwrap_or_else(|| format!("Agent '{}' did not produce output", target)),
            status: if success { DelegationStatus::Success } else { DelegationStatus::Failure },
            completed_at: chrono::Utc::now(),
        })
    }

    /// Wait for a run to complete and return its output summary.
    /// Polls the run store every 500ms up to `timeout_secs`.
    async fn await_run_completion(&self, run_id: &str, timeout_secs: u64) -> Option<String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
        loop {
            if std::time::Instant::now() >= deadline {
                return None;
            }
            if let Ok(Some(record)) = self.run_store.get(run_id) {
                if record.status == "completed" || record.status == "failed" {
                    return record.output_summary;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    // -------------------------------------------------------------------------
    // Memory management methods (CORE-04 / Phase 16-05)
    // -------------------------------------------------------------------------

    /// List all vector memory entries for a named agent.
    ///
    /// Opens (or creates) the agent's `SqliteVectorBackend` and returns all
    /// stored entries ordered by `stored_at DESC`.
    /// Returns `Err` if the agent is not loaded or if the backend cannot be opened.
    pub async fn list_agent_memory(
        &self,
        agent_name: &str,
    ) -> Result<Vec<agentix_core::VectorEntry>, AgentixError> {
        use agentix_core::vector_memory::VectorMemoryBackend;

        let definition = self
            .agents
            .get(agent_name)
            .map(|e| e.value().definition.clone())
            .ok_or_else(|| AgentixError::Runtime(format!("Agent '{}' not found", agent_name)))?;

        if !definition.vector_memory.enabled {
            return Ok(Vec::new());
        }

        let backend = crate::memory::open_agent_memory(
            agent_name,
            definition.vector_memory.db_path.as_deref(),
        )
        .await
        .map_err(|e| AgentixError::Runtime(format!("Failed to open memory backend: {}", e)))?;

        backend
            .list_entries(agent_name)
            .await
            .map_err(|e| AgentixError::Runtime(format!("Failed to list memory entries: {}", e)))
    }

    /// Clear all vector memory entries for a named agent.
    ///
    /// Opens the agent's `SqliteVectorBackend` and deletes all entries for that agent.
    /// Returns the number of entries deleted.
    pub async fn clear_agent_memory(&self, agent_name: &str) -> Result<u64, AgentixError> {
        use agentix_core::vector_memory::VectorMemoryBackend;

        let definition = self
            .agents
            .get(agent_name)
            .map(|e| e.value().definition.clone())
            .ok_or_else(|| AgentixError::Runtime(format!("Agent '{}' not found", agent_name)))?;

        if !definition.vector_memory.enabled {
            return Ok(0);
        }

        let backend = crate::memory::open_agent_memory(
            agent_name,
            definition.vector_memory.db_path.as_deref(),
        )
        .await
        .map_err(|e| AgentixError::Runtime(format!("Failed to open memory backend: {}", e)))?;

        // list_entries to count, then clear
        let entries = backend.list_entries(agent_name).await.unwrap_or_default();
        let count = entries.len() as u64;
        backend
            .clear_agent_memory(agent_name)
            .await
            .map_err(|e| AgentixError::Runtime(format!("Failed to clear memory: {}", e)))?;
        Ok(count)
    }

    // -------------------------------------------------------------------------
    // Cost query methods (Phase 17 — COST-06)
    // -------------------------------------------------------------------------

    /// List cost summaries for all agents (sorted by total_cost_usd DESC).
    pub fn get_all_cost_summaries(&self) -> Vec<agentix_core::CostSummary> {
        let mut summaries = self.cost_store.list_all_summaries().unwrap_or_default();
        summaries.sort_by(|a, b| {
            b.total_cost_usd
                .partial_cmp(&a.total_cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        summaries
    }

    /// Get cost summary for a single agent.
    pub fn get_agent_cost_summary(&self, agent_name: &str) -> Option<agentix_core::CostSummary> {
        self.cost_store.get_agent_summary(agent_name).ok()
    }

    /// List per-run cost summaries for an agent (newest first, paginated).
    pub fn get_agent_run_cost_summaries(
        &self,
        agent_name: &str,
        limit: usize,
    ) -> Vec<agentix_core::RunCostSummary> {
        self.cost_store
            .list_run_summaries(agent_name, limit)
            .unwrap_or_default()
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
    // ---------------------------------------------------------------------------
    // Provider configuration (runtime overrides)
    // ---------------------------------------------------------------------------

    /// List all configured providers (from workspace config + runtime overrides).
    ///
    /// Returns provider name, whether it has a key configured, and a masked key.
    /// Runtime overrides take precedence over workspace config values.
    pub fn list_providers(&self) -> Vec<ProviderStatus> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        // Runtime overrides first (take precedence)
        for entry in self.provider_overrides.iter() {
            let name = entry.key().clone();
            let cfg = entry.value();
            let has_key = cfg.api_key.as_ref().map_or(false, |k| !k.is_empty());
            result.push(ProviderStatus {
                name: name.clone(),
                configured: has_key,
                api_key_masked: cfg.api_key.as_ref().map(|k| mask_key(k)),
                base_url: cfg.base_url.clone(),
                source: "runtime".to_string(),
            });
            seen.insert(name);
        }

        // Workspace config providers (if not overridden)
        if let Some(ws) = &self.workspace_config {
            for (name, cfg) in &ws.spec.providers {
                if seen.contains(name) {
                    continue;
                }
                let has_key = cfg.api_key.as_ref().map_or(false, |k| !k.is_empty());
                result.push(ProviderStatus {
                    name: name.clone(),
                    configured: has_key,
                    api_key_masked: cfg.api_key.as_ref().map(|k| mask_key(k)),
                    base_url: cfg.base_url.clone(),
                    source: "config".to_string(),
                });
            }
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Set a runtime provider override.
    pub fn set_provider(&self, name: String, config: agentix_core::ProviderConfig) {
        self.provider_overrides.insert(name, config);
    }

    /// Get the effective provider config (override first, then workspace config).
    pub fn get_provider_config(&self, provider_name: &str) -> Option<agentix_core::ProviderConfig> {
        // Check runtime overrides first
        if let Some(entry) = self.provider_overrides.get(provider_name) {
            return Some(entry.value().clone());
        }
        // Fall back to workspace config
        self.workspace_config
            .as_ref()
            .and_then(|ws| ws.spec.providers.get(provider_name).cloned())
    }
}

/// Summary of a configured provider (returned by the API).
#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_masked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub source: String,
}

/// Mask an API key, showing only the first 4 and last 4 characters.
fn mask_key(key: &str) -> String {
    if key.contains("${") {
        // Environment variable reference — show as-is
        return key.to_string();
    }
    let len = key.len();
    if len <= 8 {
        return "***".to_string();
    }
    format!("{}***{}", &key[..4], &key[len - 4..])
}

// ---------------------------------------------------------------------------
// CoordinatorProtocol implementation for AgentManager (COORD-02, COORD-03, COORD-04)
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl CoordinatorProtocol for AgentManager {
    async fn delegate(
        &self,
        target: &str,
        task: &str,
        payload: serde_json::Value,
    ) -> Result<DelegationResult, AgentixError> {
        // Use a temporary Arc wrapping for the delegate_task method.
        // This method is called on an existing Arc<AgentManager> via axum State.
        let delegation_id = Uuid::new_v4().to_string();

        if !self.agents.contains_key(target) {
            return Err(AgentixError::Runtime(format!(
                "Delegation target agent '{}' not found",
                target
            )));
        }

        let event = agentix_core::TriggerEvent {
            source: TriggerSource::Agent,
            payload: serde_json::json!({
                "delegation_id": delegation_id,
                "task": task,
                "payload": payload,
            }),
            context: {
                let mut ctx = std::collections::HashMap::new();
                ctx.insert("delegation_id".to_string(), delegation_id.clone());
                ctx
            },
            fired_at: chrono::Utc::now(),
            trigger_id: delegation_id.clone(),
        };

        // Fire the target agent — this persists a RunRecord and starts the run
        // Note: run_agent_with_trigger requires Arc<Self>, so we use trigger_event_tx instead
        self.trigger_event_tx
            .send((target.to_string(), event))
            .await
            .map_err(|e| AgentixError::Runtime(format!("Delegation channel closed: {e}")))?;

        tracing::info!(
            delegation_id = %delegation_id,
            target = %target,
            task = %task,
            "Delegation dispatched via CoordinatorProtocol"
        );

        // For now, return an accepted result. Full synchronous result routing
        // (await run completion) is provided via delegate_task() which uses Arc<Self>.
        Ok(DelegationResult {
            delegation_id,
            from_agent: target.to_string(),
            output: format!("Delegation accepted by '{}'", target),
            status: DelegationStatus::Success,
            completed_at: chrono::Utc::now(),
        })
    }

    async fn delegate_parallel(
        &self,
        delegations: Vec<(String, String, serde_json::Value)>,
    ) -> Result<Vec<DelegationResult>, AgentixError> {
        let mut results = Vec::with_capacity(delegations.len());
        // Sequential fallback for trait impl (Arc version is parallel)
        for (target, task, payload) in delegations {
            let result = self.delegate(&target, &task, payload).await?;
            results.push(result);
        }
        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// LLM provider factory helper
// ---------------------------------------------------------------------------

/// Create an LLM model from an `AgentDefinition`.
///
/// Parses `model_preferred` (format: `provider/model`), looks up credentials
/// in `workspace_config.spec.providers`, and creates the provider.
///
/// When the provider is configured in `ProviderMode::Subscription`, an OAuth
/// access token is obtained from `auth_service` and used in place of an API key.
async fn create_provider_from_definition(
    definition: &AgentDefinition,
    workspace_config: &Option<WorkspaceConfig>,
    provider_overrides: &DashMap<String, agentix_core::ProviderConfig>,
    auth_service: Option<&AuthService>,
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

    // Check if this provider is configured in subscription mode
    let provider_mode = provider_overrides
        .get(provider_name)
        .and_then(|cfg| cfg.mode)
        .or_else(|| {
            workspace_config
                .as_ref()
                .and_then(|wc| wc.spec.providers.get(provider_name))
                .and_then(|pc| pc.mode)
        });

    // Look up credentials: runtime overrides → workspace config → env vars
    let (mut api_key, endpoint) = if let Some(override_cfg) = provider_overrides.get(provider_name) {
        (
            override_cfg.api_key.clone(),
            override_cfg.base_url.clone(),
        )
    } else if let Some(ws) = workspace_config {
        let provider_cfg = ws.spec.providers.get(provider_name);
        (
            provider_cfg.and_then(|p| p.api_key.clone()),
            provider_cfg.and_then(|p| p.base_url.clone()),
        )
    } else {
        // Fall back to environment variables
        (get_default_api_key_from_env(provider_name), None)
    };

    // If subscription mode, resolve an OAuth access token via AuthService
    let mut extra: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    if matches!(provider_mode, Some(ProviderMode::Subscription)) {
        if let Some(svc) = auth_service {
            let token = match provider {
                ModelProvider::OpenAI => svc.get_valid_openai_access_token(None).await
                    .map_err(|e| AgentixError::Config(format!("Failed to get OpenAI subscription token: {}", e)))?,
                ModelProvider::Google => svc.get_valid_gemini_access_token(None).await
                    .map_err(|e| AgentixError::Config(format!("Failed to get Gemini subscription token: {}", e)))?,
                ModelProvider::Anthropic => svc.get_provider_bearer_token("anthropic", None).await
                    .map_err(|e| AgentixError::Config(format!("Failed to get Anthropic subscription token: {}", e)))?,
                _ => None,
            };
            match token {
                Some(t) => {
                    api_key = Some(t);
                    extra.insert("provider_mode".to_string(), serde_json::json!("subscription"));
                }
                None => {
                    return Err(AgentixError::Config(format!(
                        "Provider '{}' is configured for subscription mode but no OAuth token was found. \
                        Run `agentix auth start {}` to authenticate.",
                        provider_name, provider_name
                    )));
                }
            }
        }
    }

    let config = ModelConfig {
        model: model_name.to_string(),
        provider,
        api_key,
        endpoint,
        temperature: 0.7,
        max_tokens: None,
        timeout_secs: definition.timeout_secs,
        headers: std::collections::HashMap::new(),
        extra,
    };

    let model = ProviderFactory::create(config).await?;

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
