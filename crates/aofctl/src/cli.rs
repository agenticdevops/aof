use clap::{Parser, Subcommand};
use std::path::Path;

use aof_core::Context;
use crate::commands;

/// AOF CLI - kubectl-style agent orchestration
#[derive(Parser, Debug)]
#[command(name = "aofctl")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Context to use (overrides AOFCTL_CONTEXT env var)
    ///
    /// Context determines environment-specific settings like:
    /// - Environment variables (credentials, endpoints)
    /// - Approval requirements and allowed approvers
    /// - Rate limits and resource quotas
    #[arg(long, short = 'C', global = true, env = "AOFCTL_CONTEXT")]
    pub context: Option<String>,

    /// Directory containing context definitions
    #[arg(long, global = true, env = "AOFCTL_CONTEXTS_DIR", default_value = "contexts")]
    pub contexts_dir: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run an agent with configuration (verb-first: run agent)
    Run {
        /// Resource type (agent, workflow, job)
        resource_type: String,

        /// Resource name or configuration file
        name_or_config: String,

        /// Input/query for the agent
        #[arg(short, long)]
        input: Option<String>,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        output: String,

        /// Output schema for structured responses
        /// Use pre-built schema names (container-list, resource-stats, simple-list, key-value)
        /// or provide inline JSON schema
        #[arg(long)]
        output_schema: Option<String>,

        /// Path to JSON schema file for output validation
        #[arg(long, conflicts_with = "output_schema")]
        output_schema_file: Option<String>,
    },

    /// Get resources (verb-first: get agents, get agent <name>)
    Get {
        /// Resource type (agent, workflow, tool, etc.)
        resource_type: String,

        /// Resource name (optional - lists all if omitted)
        name: Option<String>,

        /// Output format (json, yaml, wide, name)
        #[arg(short, long, default_value = "wide")]
        output: String,

        /// Show all namespaces
        #[arg(long)]
        all_namespaces: bool,

        /// List resources from the built-in library
        #[arg(long)]
        library: bool,
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

    /// Delete resources (verb-first: delete agent <name>)
    Delete {
        /// Resource type
        resource_type: String,

        /// Resource name
        name: String,

        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },

    /// Describe resources in detail (verb-first: describe agent <name>)
    Describe {
        /// Resource type
        resource_type: String,

        /// Resource name
        name: String,

        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },

    /// Get logs from a resource (verb-first: logs agent <name>)
    Logs {
        /// Resource type (agent, job, task)
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

    /// Execute a command in a resource (verb-first: exec agent <name> -- command)
    Exec {
        /// Resource type (agent, workflow)
        resource_type: String,

        /// Resource name
        name: String,

        /// Command to execute
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// List available API resources
    ApiResources,

    /// List MCP tools (legacy command, use 'get mcptools' instead)
    #[command(hide = true)]
    Tools {
        /// MCP server command
        #[arg(long)]
        server: String,

        /// Server arguments
        #[arg(long)]
        args: Vec<String>,
    },

    /// Validate agent configuration (legacy command, use 'apply --dry-run' instead)
    #[command(hide = true)]
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

        /// Directory containing AgentFlow YAML files for event-driven routing
        #[arg(long)]
        flows_dir: Option<String>,
    },

    /// Manage agent fleets (multi-agent coordination)
    ///
    /// NOTE: Prefer kubectl-style commands: aofctl get fleets, aofctl run fleet <name>
    #[command(hide = true)]
    Fleet {
        #[command(subcommand)]
        command: commands::fleet::FleetCommands,
    },

    /// Manage agent flows (workflow orchestration)
    ///
    /// NOTE: Prefer kubectl-style commands: aofctl get flows, aofctl run flow <name>
    #[command(hide = true)]
    Flow {
        #[command(subcommand)]
        command: commands::flow::FlowCommands,
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
            } => {
                commands::run::execute(
                    &resource_type,
                    &name_or_config,
                    input.as_deref(),
                    &output,
                    output_schema.as_deref(),
                    output_schema_file.as_deref(),
                    context.as_ref(),
                )
                .await
            }
            Commands::Get {
                resource_type,
                name,
                output,
                all_namespaces,
                library,
            } => {
                commands::get::execute(&resource_type, name.as_deref(), &output, all_namespaces, library)
                    .await
            }
            Commands::Apply { file, namespace } => {
                commands::apply::execute(&file, namespace.as_deref()).await
            }
            Commands::Delete {
                resource_type,
                name,
                namespace,
            } => commands::delete::execute(&resource_type, &name, namespace.as_deref()).await,
            Commands::Describe {
                resource_type,
                name,
                namespace: _,
            } => {
                commands::describe::execute(&resource_type, &name)
                    .await
            }
            Commands::Logs {
                resource_type,
                name,
                follow,
                tail,
            } => commands::logs::execute(&resource_type, &name, follow, tail).await,
            Commands::Exec {
                resource_type,
                name,
                command,
            } => commands::exec::execute(&resource_type, &name, command).await,
            Commands::ApiResources => commands::api_resources::execute().await,
            Commands::Tools { server, args } => commands::tools::execute(&server, &args).await,
            Commands::Validate { file } => commands::validate::execute(&file).await,
            Commands::Version => commands::version::execute().await,
            Commands::Serve {
                config,
                port,
                host,
                agents_dir,
                flows_dir,
            } => {
                commands::serve::execute(
                    config.as_deref(),
                    port,
                    host.as_deref(),
                    agents_dir.as_deref(),
                    flows_dir.as_deref(),
                )
                .await
            }
            Commands::Fleet { command } => commands::fleet::execute(command).await,
            Commands::Flow { command } => commands::flow::execute(command).await,
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
