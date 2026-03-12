// OpenAgentiX Core — Runtime Trigger Abstractions
//
// This module defines the runtime trigger system: the TriggerEvent envelope delivered
// to every triggered agent run, the TriggerTrait pluggable interface for implementing
// new trigger types, and the TriggerRunRegistry that the gateway uses to manage all
// active trigger instances.
//
// Note: This is distinct from trigger.rs (which handles the YAML resource type for
// declarative trigger configuration). trigger_event.rs handles the runtime layer.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::AgentixError;

/// Identifies the source that fired a trigger event.
///
/// Used to route events, customize agent prompts, and record trigger history.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerSource {
    /// Cron/schedule-based trigger — agent fires on a cron expression schedule.
    Cron,
    /// Generic HTTP webhook trigger — agent fires when POST /webhooks/:id is called.
    Webhook,
    /// GitHub webhook trigger — fires on pull_request, push, tag, and other GitHub events.
    GitHub,
    /// Jira webhook trigger — fires on issue_updated, comment_created, and mention events.
    Jira,
    /// Slack mention trigger — fires when @agent-name is mentioned in a Slack channel.
    Slack,
    /// Discord mention trigger — fires when the agent bot is mentioned in Discord.
    Discord,
    /// Telegram trigger — fires when the Telegram bot receives a message.
    Telegram,
    /// Agent-to-agent trigger — fires when a parent agent delegates work to this agent.
    Agent,
    /// CLI trigger — fires when `agentix run <agent>` is invoked from the command line.
    Cli,
}

impl fmt::Display for TriggerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Cron => "cron",
            Self::Webhook => "webhook",
            Self::GitHub => "github",
            Self::Jira => "jira",
            Self::Slack => "slack",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
            Self::Agent => "agent",
            Self::Cli => "cli",
        };
        write!(f, "{}", s)
    }
}

/// The envelope delivered to every triggered agent run.
///
/// When any trigger fires, it produces a `TriggerEvent` that is serialized and
/// passed as the user input to the agent. The agent can read the structured data
/// from the event to understand what triggered it and what to do.
///
/// # Example
///
/// ```rust
/// use agentix_core::{TriggerEvent, TriggerSource};
/// use serde_json::json;
///
/// let event = TriggerEvent::new(
///     TriggerSource::GitHub,
///     json!({"action": "opened", "number": 42, "title": "Fix bug"}),
///     "github-pr-trigger",
/// )
/// .with_context("repo", "openagentix/openagentix");
///
/// assert_eq!(event.payload["action"], "opened");
/// assert!(event.context.get("repo").is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    /// What fired this trigger — used for routing and logging.
    pub source: TriggerSource,

    /// Platform-specific payload. Contents depend on trigger source:
    /// - Cron: `{"expression": "...", "scheduled_at": "..."}`
    /// - GitHub: `{"action": "opened", "number": 42, ...}`
    /// - Webhook: raw POST body as parsed JSON
    /// - Jira: `{"event_type": "issue_updated", "issue": {...}}`
    pub payload: serde_json::Value,

    /// Arbitrary key/value metadata added by the trigger implementation.
    /// Common keys: `repo`, `channel_id`, `caller_agent`, `invocation`.
    #[serde(default)]
    pub context: HashMap<String, String>,

    /// RFC 3339 timestamp when this event was created (UTC).
    pub fired_at: chrono::DateTime<chrono::Utc>,

    /// The trigger_id of the trigger instance that produced this event.
    /// Matches the key in `TriggerRunRegistry`.
    pub trigger_id: String,
}

impl TriggerEvent {
    /// Create a new TriggerEvent with the given source, payload, and trigger_id.
    ///
    /// Sets `fired_at` to the current UTC time and leaves `context` empty.
    pub fn new(
        source: TriggerSource,
        payload: serde_json::Value,
        trigger_id: impl Into<String>,
    ) -> Self {
        Self {
            source,
            payload,
            context: HashMap::new(),
            fired_at: chrono::Utc::now(),
            trigger_id: trigger_id.into(),
        }
    }

    /// Add a context key/value pair using builder syntax.
    ///
    /// # Example
    /// ```rust
    /// # use agentix_core::{TriggerEvent, TriggerSource};
    /// let event = TriggerEvent::new(TriggerSource::Agent, serde_json::json!({}), "t1")
    ///     .with_context("caller_agent", "orchestrator")
    ///     .with_context("priority", "high");
    /// ```
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Pluggable trigger interface.
///
/// Implement this trait to add a new trigger type without modifying agentix-core.
/// The gateway holds all registered `TriggerTrait` objects in a `TriggerRunRegistry`
/// and calls `start()` on each at boot.
///
/// # Implementation notes
///
/// - **Active triggers** (Cron): `start()` spawns a background tokio task that
///   sends events via the provided mpsc sender when the schedule fires.
/// - **Passive triggers** (Webhook, GitHub, Jira, Channel): `start()` just stores
///   the sender internally. The gateway HTTP handler calls the trigger's
///   `receive_payload()` method when a POST arrives.
/// - `stop()` must cleanly terminate any background tasks started by `start()`.
#[async_trait]
pub trait TriggerTrait: Send + Sync {
    /// Return the unique identifier for this trigger instance.
    ///
    /// This is the key used in `TriggerRunRegistry` and appears in `TriggerEvent.trigger_id`.
    fn trigger_id(&self) -> &str;

    /// Return the trigger source type.
    fn source(&self) -> TriggerSource;

    /// Start the trigger.
    ///
    /// For active triggers (cron): spawn a background task that sends
    /// `(agent_name, TriggerEvent)` tuples via `sender` when the trigger fires.
    ///
    /// For passive triggers (webhooks): store the sender for later use when
    /// an HTTP request arrives.
    async fn start(
        &self,
        sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError>;

    /// Gracefully stop the trigger and release any resources.
    async fn stop(&self) -> Result<(), AgentixError>;
}

/// Runtime registry for active `TriggerTrait` instances.
///
/// This registry is owned by the gateway and populated at agent load time.
/// It is separate from the declarative `TriggerRegistry` in `registry.rs`
/// (which holds YAML resource types). This registry holds live instances
/// ready to start/stop.
///
/// # Example
///
/// ```rust,ignore
/// let mut registry = TriggerRunRegistry::new();
/// registry.register(Arc::new(CronTrigger::new("0 9 * * 1", "reporter", "weekly")?));
/// if let Some(trigger) = registry.get("weekly") {
///     trigger.start(sender).await?;
/// }
/// ```
#[derive(Default)]
pub struct TriggerRunRegistry {
    triggers: HashMap<String, Arc<dyn TriggerTrait>>,
}

impl TriggerRunRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a trigger instance. Overwrites any existing trigger with the same id.
    pub fn register(&mut self, trigger: Arc<dyn TriggerTrait>) {
        self.triggers
            .insert(trigger.trigger_id().to_string(), trigger);
    }

    /// Look up a trigger by its id.
    pub fn get(&self, id: &str) -> Option<Arc<dyn TriggerTrait>> {
        self.triggers.get(id).cloned()
    }

    /// Return all registered triggers.
    pub fn all(&self) -> Vec<Arc<dyn TriggerTrait>> {
        self.triggers.values().cloned().collect()
    }

    /// Number of registered triggers.
    pub fn len(&self) -> usize {
        self.triggers.len()
    }

    /// Returns `true` if no triggers are registered.
    pub fn is_empty(&self) -> bool {
        self.triggers.is_empty()
    }
}

// Manual Debug impl — dyn TriggerTrait does not implement Debug
impl fmt::Debug for TriggerRunRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TriggerRunRegistry")
            .field("trigger_count", &self.triggers.len())
            .field("trigger_ids", &self.triggers.keys().collect::<Vec<_>>())
            .finish()
    }
}
