use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod commands;

use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::io::IsTerminal;

    // Initialize tokio-console if feature is enabled
    // To use: RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio-console -- serve
    // Then in another terminal: tokio-console
    #[cfg(feature = "tokio-console")]
    {
        console_subscriber::init();
    }

    // Parse CLI to detect interactive mode early
    let cli = Cli::parse();
    let is_interactive = matches!(&cli.command, cli::Commands::Run { input, .. }
        if input.is_none() && std::io::stdin().is_terminal());

    // Initialize tracing only if NOT in interactive mode and tokio-console is not enabled
    #[cfg(not(feature = "tokio-console"))]
    if !is_interactive {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "error".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Execute command
    cli.execute().await?;

    Ok(())
}
