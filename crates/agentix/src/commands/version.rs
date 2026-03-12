pub async fn execute() -> anyhow::Result<()> {
    println!("agentix version: {}", env!("CARGO_PKG_VERSION"));
    println!("agentix-core version: {}", agentix_core::VERSION);
    println!("MCP version: {}", agentix_mcp::MCP_VERSION);
    Ok(())
}
