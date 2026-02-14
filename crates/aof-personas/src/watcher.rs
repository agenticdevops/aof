//! File watching and reload for persona workspace files
//!
//! Watches AGENTS.md and SOUL.md for changes and triggers reload events.
//! Uses the notify crate for efficient filesystem event monitoring.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::loader::{AgentLoader, SoulLoader};
use crate::types::{Agent, Soul};
use crate::validation::validate_personas;

/// Update event emitted when persona files change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaUpdate {
    /// Updated agent list
    pub agents: Vec<Agent>,
    /// Updated soul map
    pub souls: HashMap<String, Soul>,
    /// When the update was detected
    pub timestamp: DateTime<Utc>,
}

/// Watches workspace files for changes and emits reload events
///
/// Monitors AGENTS.md and SOUL.md using the notify crate for filesystem
/// events. On change, reloads and validates data, then sends a PersonaUpdate
/// through a channel that callers subscribe to.
pub struct PersonaWatcher {
    _watcher: RecommendedWatcher,
}

impl PersonaWatcher {
    /// Start watching persona files for changes
    ///
    /// Returns a receiver channel that emits PersonaUpdate events whenever
    /// AGENTS.md or SOUL.md are modified. Events are coalesced to avoid
    /// duplicate notifications from rapid successive writes.
    ///
    /// # Arguments
    /// * `agents_path` - Path to AGENTS.md
    /// * `souls_path` - Path to SOUL.md
    ///
    /// # Returns
    /// A tuple of (PersonaWatcher, Receiver<PersonaUpdate>). The watcher
    /// must be kept alive for monitoring to continue.
    pub fn watch_for_changes(
        agents_path: &str,
        souls_path: &str,
    ) -> Result<(Self, mpsc::Receiver<PersonaUpdate>)> {
        let (tx, rx) = mpsc::channel::<PersonaUpdate>(16);

        let agents_path_owned = agents_path.to_string();
        let souls_path_owned = souls_path.to_string();

        // Use a debounce channel to coalesce rapid changes
        let (debounce_tx, mut debounce_rx) = mpsc::channel::<()>(32);

        let watcher_result = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_)
                    ) {
                        debug!("Persona file change detected: {:?}", event.paths);
                        let _ = debounce_tx.try_send(());
                    }
                }
                Err(e) => {
                    error!("File watch error: {}", e);
                }
            }
        });

        let mut watcher = watcher_result.map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;

        // Watch the parent directories of both files
        let agents_parent = Path::new(agents_path)
            .parent()
            .unwrap_or(Path::new("."));
        let souls_parent = Path::new(souls_path)
            .parent()
            .unwrap_or(Path::new("."));

        watcher
            .watch(agents_parent, RecursiveMode::NonRecursive)
            .map_err(|e| anyhow::anyhow!("Failed to watch agents directory: {}", e))?;

        if agents_parent != souls_parent {
            watcher
                .watch(souls_parent, RecursiveMode::NonRecursive)
                .map_err(|e| anyhow::anyhow!("Failed to watch souls directory: {}", e))?;
        }

        // Spawn debounce + reload task
        let reload_tx = tx;
        let agents_reload_path = agents_path_owned.clone();
        let souls_reload_path = souls_path_owned.clone();

        tokio::spawn(async move {
            // Coalesce events: wait for a quiet period before reloading
            loop {
                match debounce_rx.recv().await {
                    Some(()) => {
                        // Wait briefly for additional events (coalescing)
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        // Drain any pending events
                        while debounce_rx.try_recv().is_ok() {}

                        info!("Reloading persona files after change detected");

                        // Reload agents
                        let agents = match AgentLoader::load_from_file(&agents_reload_path).await {
                            Ok(a) => a,
                            Err(e) => {
                                warn!("Failed to reload AGENTS.md: {}", e);
                                continue;
                            }
                        };

                        // Reload souls
                        let souls = match SoulLoader::load_from_file(&souls_reload_path).await {
                            Ok(s) => s,
                            Err(e) => {
                                warn!("Failed to reload SOUL.md: {}", e);
                                continue;
                            }
                        };

                        // Validate
                        if let Err(e) = validate_personas(&agents, &souls) {
                            warn!("Persona validation failed after reload: {}", e);
                            continue;
                        }

                        let update = PersonaUpdate {
                            agents,
                            souls,
                            timestamp: Utc::now(),
                        };

                        if reload_tx.send(update).await.is_err() {
                            debug!("PersonaUpdate receiver dropped, stopping watcher");
                            break;
                        }
                    }
                    None => {
                        debug!("Debounce channel closed, stopping watcher");
                        break;
                    }
                }
            }
        });

        Ok((PersonaWatcher { _watcher: watcher }, rx))
    }
}
