use clap::{Parser, Subcommand};

use crate::commands;

/// AOF CLI - kubectl-style agent orchestration
#[derive(Parser, Debug)]
#[command(name = "aofctl")]
#[command(version, about, long_about = None)]
pub struct Cli {
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
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Run {
                resource_type,
                name_or_config,
                input,
                output,
            } => {
                commands::run::execute(&resource_type, &name_or_config, input.as_deref(), &output)
                    .await
            }
            Commands::Get {
                resource_type,
                name,
                output,
                all_namespaces,
            } => {
                commands::get::execute(&resource_type, name.as_deref(), &output, all_namespaces)
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
        }
    }
}
