use anyhow::Result;
use crate::resources::ResourceType;

/// Display available API resources (kubectl api-resources compatible)
pub async fn execute() -> Result<()> {
    println!("\nNAME                SHORTNAMES      APIVERSION      NAMESPACED   KIND");
    println!("{}", "=".repeat(95));

    for resource_type in ResourceType::all() {
        let name = resource_type.plural();
        let short_names = resource_type.short_names().join(",");
        let api_version = resource_type.api_version();
        let namespaced = if resource_type.is_namespaced() { "true" } else { "false" };
        let kind = resource_type.kind();

        println!(
            "{:<20}{:<16}{:<16}{:<13}{}",
            name,
            short_names,
            api_version,
            namespaced,
            kind
        );
    }

    println!("\n");
    Ok(())
}
