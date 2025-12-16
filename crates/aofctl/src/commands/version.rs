pub async fn execute() -> anyhow::Result<()> {
    println!("aofctl version: {}", env!("CARGO_PKG_VERSION"));
    println!("aof-core version: {}", aof_core::VERSION);
    println!("MCP version: {}", aof_mcp::MCP_VERSION);
    Ok(())
}
