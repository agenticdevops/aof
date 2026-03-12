//! Gateway router — HTTP server entry point.
//!
//! `Gateway::start` is called by the CLI `agentix gateway start` command.
//! It:
//! 1. Creates an `AgentManager` with the workspace config
//! 2. Scans `agents_dir` and loads all agents
//! 3. Starts the axum HTTP server on `config.host:config.port`

use std::path::Path;

use agentix_core::{AgentixError, GatewayConfig, WorkspaceConfig};

use super::agent_manager::AgentManager;
use super::api;

/// The OpenAgentiX gateway service.
///
/// Encapsulates startup logic for the HTTP server.
pub struct Gateway;

impl Gateway {
    /// Start the gateway HTTP server.
    ///
    /// # Arguments
    /// * `config` — Gateway host/port settings (from `WorkspaceConfig.spec.gateway`)
    /// * `workspace_config` — Full workspace configuration (provider credentials, defaults)
    /// * `agents_dir` — Directory to scan for agent subdirectories and flat YAML files
    ///
    /// This method runs until the server is shut down (e.g. via Ctrl+C).
    pub async fn start(
        config: GatewayConfig,
        workspace_config: Option<WorkspaceConfig>,
        agents_dir: &Path,
    ) -> Result<(), AgentixError> {
        let manager = AgentManager::new(workspace_config);

        let loaded = manager.load_from_dir(agents_dir);

        let addr = format!("{}:{}", config.host, config.port);

        println!("OpenAgentiX Gateway");
        println!("Listening on http://{}", addr);
        println!(
            "Loaded {} agent(s) from {}",
            loaded.len(),
            agents_dir.display()
        );
        for name in &loaded {
            println!("  - {}", name);
        }

        let app = api::create_router(manager);

        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            AgentixError::Runtime(format!("Failed to bind {}: {}", addr, e))
        })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| AgentixError::Runtime(format!("Gateway server error: {}", e)))?;

        Ok(())
    }
}
