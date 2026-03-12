//! `agentix version` — print version and branding.

use anyhow::Result;

pub fn run() -> Result<()> {
    println!(
        "agentix {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("OpenAgentiX — Enterprise Agent Automation Platform");
    println!("https://openagentix.org");
    Ok(())
}
