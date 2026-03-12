use clap::Parser;

mod cli;
mod client;
mod commands;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Tracing setup — error level unless RUST_LOG set
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "error".into()),
        )
        .with_target(false)
        .init();

    // Color detection: disable if not TTY or NO_COLOR is set
    if !std::io::IsTerminal::is_terminal(&std::io::stdout())
        || std::env::var("NO_COLOR").is_ok()
    {
        colored::control::set_override(false);
    }

    // Destructure cli so we can pass parts to handlers without borrow issues
    let Cli {
        gateway_url,
        output,
        quiet,
        command,
    } = cli;

    // Build a thin context object for commands that need gateway_url/output/quiet
    let ctx = cli::CliContext {
        gateway_url,
        output,
        quiet,
    };

    let result = match command {
        Commands::Gateway { command } => commands::gateway::run(command).await,
        Commands::Agents { namespace } => commands::agents::run(&ctx, namespace).await,
        Commands::Runs { agent, limit } => commands::runs::run(&ctx, agent, limit).await,
        Commands::Logs { agent, run, follow } => commands::logs::run(&ctx, agent, run, follow).await,
        Commands::Stop { agent, run } => commands::stop::run(&ctx, agent, run).await,
        Commands::Apply { file } => commands::apply::run(&ctx, file).await,
        Commands::Validate { path } => commands::validate::run(path).await,
        Commands::Version => commands::version::run(),
        Commands::Init { name, output_dir, non_interactive } => {
            commands::init::execute(name, &output_dir, non_interactive).await
        }
        Commands::Onboard { non_interactive } => {
            commands::onboard::execute(non_interactive).await
        }
        Commands::Skills { command } => {
            commands::skills::run(&command, &ctx.output).await
        }
    };

    if let Err(e) = result {
        let msg = e.to_string();
        if colored::control::SHOULD_COLORIZE.should_colorize() {
            eprintln!("\x1b[31merror:\x1b[0m {}", msg);
        } else {
            eprintln!("error: {}", msg);
        }
        std::process::exit(1);
    }
}
