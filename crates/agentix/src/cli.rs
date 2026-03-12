use clap::{Parser, Subcommand, ValueEnum};

/// OpenAgentiX — Enterprise Agent Automation Platform
///
/// Define AI agents as directories. Run them on demand. Track everything.
/// https://openagentix.org
#[derive(Parser, Debug)]
#[command(
    name = "agentix",
    version,
    about = "OpenAgentiX — Enterprise Agent Automation Platform",
    long_about = "Define AI agents as directories. Run them on demand. Track everything.\nhttps://openagentix.org"
)]
pub struct Cli {
    /// Gateway URL
    #[arg(long, global = true, env = "AGENTIX_GATEWAY_URL", default_value = "http://127.0.0.1:7777")]
    pub gateway_url: String,

    /// Output format: text or json
    #[arg(long, global = true, default_value = "text", value_enum)]
    pub output: OutputFormat,

    /// Suppress ReAct loop details, show final result only
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage the gateway server
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },
    /// List agents registered in the gateway
    Agents {
        /// Filter by namespace
        #[arg(long, short = 'n')]
        namespace: Option<String>,
    },
    /// Show agent run history
    Runs {
        /// Agent name (all agents if omitted)
        agent: Option<String>,
        /// Maximum number of runs to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Show agent execution logs
    Logs {
        /// Agent name
        agent: String,
        /// Run ID (latest if omitted)
        #[arg(long)]
        run: Option<String>,
        /// Follow log output
        #[arg(long, short = 'f')]
        follow: bool,
    },
    /// Stop a running agent
    Stop {
        /// Agent name
        agent: String,
        /// Run ID (latest active run if omitted)
        #[arg(long)]
        run: Option<String>,
    },
    /// Apply agent configuration to the gateway
    Apply {
        /// Path to agent YAML file
        #[arg(short = 'f', long)]
        file: String,
    },
    /// Validate agent YAML or agent directory (no gateway needed)
    Validate {
        /// Path to agent.yaml file, flat YAML file, or agent directory
        path: String,
    },
    /// Print version information
    Version,

    /// Scaffold a new agent directory (GitAgent-compatible)
    Init {
        /// Agent name (e.g., my-agent)
        #[arg(long)]
        name: Option<String>,

        /// Parent directory for the new agent directory
        #[arg(long, short, default_value = "./agents")]
        output_dir: String,

        /// Non-interactive mode (requires --name)
        #[arg(long)]
        non_interactive: bool,
    },

    /// Set up a new OpenAgentiX workspace interactively
    Onboard {
        /// Skip interactive prompts, use all defaults
        #[arg(long)]
        non_interactive: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum GatewayCommands {
    /// Start the gateway server
    Start {
        /// Path to workspace config (agentix.yaml)
        #[arg(long, short = 'c', default_value = "./agentix.yaml")]
        config: String,
        /// Override agents directory from config
        #[arg(long)]
        agents_dir: Option<String>,
        /// Override port from config
        #[arg(long, short = 'p')]
        port: Option<u16>,
    },
    /// Show gateway status
    Status,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Thin context object passed to command handlers (avoids borrow issues from
/// destructuring `Cli` while also borrowing `&Cli`).
pub struct CliContext {
    pub gateway_url: String,
    pub output: OutputFormat,
    pub quiet: bool,
}
