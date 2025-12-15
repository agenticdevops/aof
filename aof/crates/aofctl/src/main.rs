use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod commands;
mod resources;

use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::io::IsTerminal;

    // Parse CLI to detect interactive mode early
    let cli = Cli::parse();
    let is_interactive = matches!(&cli.command, cli::Commands::Run { input, .. }
        if input.is_none() && std::io::stdin().is_terminal());

    // Initialize tracing only if NOT in interactive mode
    // Interactive mode will set up its own LogWriter-based layer in run_agent_interactive()
    if !is_interactive {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "aofctl=info,aof_runtime=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Execute command
    cli.execute().await?;

    Ok(())
}
