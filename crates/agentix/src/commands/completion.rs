//! Shell completion generation for agentix
//!
//! Commands:
//! - agentix completion bash  > /etc/bash_completion.d/agentix
//! - agentix completion zsh   > ~/.zsh/completion/_agentix
//! - agentix completion fish  > ~/.config/fish/completions/agentix.fish
//! - agentix completion powershell > agentix.ps1

use anyhow::Result;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate, Shell as ClapShell};
use std::io;

use crate::cli::Cli;

/// Supported shells for completion
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    Powershell,
    /// Elvish shell
    Elvish,
}

impl From<Shell> for ClapShell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Bash => ClapShell::Bash,
            Shell::Zsh => ClapShell::Zsh,
            Shell::Fish => ClapShell::Fish,
            Shell::Powershell => ClapShell::PowerShell,
            Shell::Elvish => ClapShell::Elvish,
        }
    }
}

/// Generate shell completion script
pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    let clap_shell: ClapShell = shell.into();
    generate(clap_shell, &mut cmd, name, &mut io::stdout());

    // Print installation instructions to stderr
    match shell {
        Shell::Bash => {
            eprintln!();
            eprintln!("# Installation instructions:");
            eprintln!("# 1. Save to file:");
            eprintln!("#    agentix completion bash > /etc/bash_completion.d/agentix");
            eprintln!("# 2. Or add to ~/.bashrc:");
            eprintln!("#    source <(agentix completion bash)");
        }
        Shell::Zsh => {
            eprintln!();
            eprintln!("# Installation instructions:");
            eprintln!("# 1. Save to a directory in your fpath:");
            eprintln!("#    agentix completion zsh > ~/.zsh/completion/_agentix");
            eprintln!("# 2. Or add to ~/.zshrc:");
            eprintln!("#    source <(agentix completion zsh)");
            eprintln!("#");
            eprintln!("# Note: You may need to run 'compinit' to load completions.");
        }
        Shell::Fish => {
            eprintln!();
            eprintln!("# Installation instructions:");
            eprintln!("# Save to fish completions directory:");
            eprintln!("#    agentix completion fish > ~/.config/fish/completions/agentix.fish");
        }
        Shell::Powershell => {
            eprintln!();
            eprintln!("# Installation instructions:");
            eprintln!("# 1. Save to a file:");
            eprintln!("#    agentix completion powershell > agentix.ps1");
            eprintln!("# 2. Add to your PowerShell profile:");
            eprintln!("#    . ./agentix.ps1");
        }
        Shell::Elvish => {
            eprintln!();
            eprintln!("# Installation instructions:");
            eprintln!("# Save to elvish completions:");
            eprintln!("#    agentix completion elvish > ~/.elvish/lib/agentix.elv");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_conversion() {
        let bash: ClapShell = Shell::Bash.into();
        assert!(matches!(bash, ClapShell::Bash));

        let zsh: ClapShell = Shell::Zsh.into();
        assert!(matches!(zsh, ClapShell::Zsh));
    }
}
