use clap::{Parser, Subcommand};
use std::path::Path;

use agentix_core::Context;
use crate::commands;

/// OpenAgentiX CLI - kubectl-style agent orchestration
#[derive(Parser, Debug)]
#[command(name = "agentix")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Context to use (overrides AGENTIX_CONTEXT env var)
    #[arg(long, short = 'C', global = true, env = "AGENTIX_CONTEXT")]
    pub context: Option<String>,

    /// Directory containing context definitions
    #[arg(long, global = true, env = "AGENTIX_CONTEXTS_DIR", default_value = "contexts")]
    pub contexts_dir: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run an agent (verb-first: run agent <name>)
    Run {
        /// Resource type (agent)
        resource_type: String,

        /// Resource name or configuration file
        name_or_config: String,

        /// Input/query for the agent
        #[arg(short, long, visible_alias = "prompt")]
        input: Option<String>,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        output: String,

        /// Output schema for structured responses
        #[arg(long)]
        output_schema: Option<String>,

        /// Path to JSON schema file for output validation
        #[arg(long, conflicts_with = "output_schema")]
        output_schema_file: Option<String>,

        /// Resume the latest session for this agent (interactive mode only)
        #[arg(long)]
        resume: bool,

        /// Resume a specific session by ID (interactive mode only)
        #[arg(long, conflicts_with = "resume")]
        session: Option<String>,
    },

    /// Get resources (verb-first: get agents, get agent <name>)
    Get {
        /// Resource type (agent, trigger, etc.)
        resource_type: String,

        /// Resource name (optional - lists all if omitted)
        name: Option<String>,

        /// Output format (json, yaml, wide, name)
        #[arg(short, long, default_value = "wide")]
        output: String,

        /// Show all namespaces
        #[arg(long)]
        all_namespaces: bool,
    },

    /// Apply configuration from file (verb-first: apply -f config.yaml)
    Apply {
        /// Configuration file (YAML)
        #[arg(short, long)]
        file: String,

        /// Namespace for the resources
        #[arg(short, long)]
        namespace: Option<String>,
    },

    /// Get logs from a resource (verb-first: logs agent <name>)
    Logs {
        /// Resource type (agent, job)
        resource_type: String,

        /// Resource name
        name: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(long)]
        tail: Option<usize>,
    },

    /// Validate agent configuration
    Validate {
        /// Configuration file
        #[arg(short, long)]
        file: String,
    },

    /// Show version information
    Version,

    /// Start the trigger webhook server (daemon mode)
    Serve {
        /// Configuration file (YAML)
        #[arg(short, long)]
        config: Option<String>,

        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to (overrides config)
        #[arg(long, default_value = "0.0.0.0")]
        host: Option<String>,

        /// Directory containing agent YAML files
        #[arg(long)]
        agents_dir: Option<String>,

        /// Directory containing Trigger YAML files
        #[arg(long)]
        triggers_dir: Option<String>,
    },

    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: commands::completion::Shell,
    },
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        // Load context if specified
        let context = if let Some(ref ctx_name) = self.context {
            load_context(ctx_name, &self.contexts_dir)?
        } else {
            None
        };

        match self.command {
            Commands::Run {
                resource_type,
                name_or_config,
                input,
                output,
                output_schema,
                output_schema_file,
                resume,
                session,
            } => {
                commands::run::execute(
                    &resource_type,
                    &name_or_config,
                    input.as_deref(),
                    &output,
                    output_schema.as_deref(),
                    output_schema_file.as_deref(),
                    context.as_ref(),
                    resume,
                    session.as_deref(),
                )
                .await
            }
            Commands::Get {
                resource_type,
                name,
                output,
                all_namespaces,
            } => {
                commands::get::execute(&resource_type, name.as_deref(), &output, all_namespaces, false)
                    .await
            }
            Commands::Apply { file, namespace } => {
                commands::apply::execute(&file, namespace.as_deref()).await
            }
            Commands::Logs {
                resource_type,
                name,
                follow,
                tail,
            } => commands::logs::execute(&resource_type, &name, follow, tail).await,
            Commands::Validate { file } => commands::validate::execute(&file).await,
            Commands::Version => commands::version::execute().await,
            Commands::Serve {
                config,
                port,
                host,
                agents_dir,
                triggers_dir,
            } => {
                commands::serve::execute(
                    config.as_deref(),
                    port,
                    host.as_deref(),
                    agents_dir.as_deref(),
                    None,
                    triggers_dir.as_deref(),
                    None,
                    false,
                    false,
                    None,
                    None,
                )
                .await
            }
            Commands::Completion { shell } => commands::completion::execute(shell),
        }
    }
}

/// Load a Context resource from the contexts directory
fn load_context(name: &str, contexts_dir: &str) -> anyhow::Result<Option<Context>> {
    let contexts_path = Path::new(contexts_dir);

    // Try loading from file: <name>.yaml or <name>.yml
    for ext in &["yaml", "yml"] {
        let file_path = contexts_path.join(format!("{}.{}", name, ext));
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read context file {:?}: {}", file_path, e))?;

            let mut context: Context = serde_yaml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse context file {:?}: {}", file_path, e))?;

            // Expand environment variables
            context.expand_env_vars();

            // Validate
            context.validate()
                .map_err(|e| anyhow::anyhow!("Invalid context '{}': {}", name, e))?;

            tracing::info!("Loaded context '{}' from {:?}", name, file_path);
            return Ok(Some(context));
        }
    }

    // Context file not found - check if contexts dir exists
    if !contexts_path.exists() {
        tracing::warn!(
            "Contexts directory '{}' not found. Create it with context YAML files.",
            contexts_dir
        );
    } else {
        tracing::warn!(
            "Context '{}' not found in '{}'. Available contexts: {:?}",
            name,
            contexts_dir,
            list_available_contexts(contexts_path)
        );
    }

    Err(anyhow::anyhow!(
        "Context '{}' not found. Create {}/{}.yaml with your context definition.",
        name, contexts_dir, name
    ))
}

/// List available context names in a directory
fn list_available_contexts(dir: &Path) -> Vec<String> {
    let mut contexts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                if let Some(stem) = path.file_stem() {
                    contexts.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    contexts
}
