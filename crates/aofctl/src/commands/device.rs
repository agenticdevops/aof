//! Device management commands (kubectl-style).
//!
//! Commands for managing device pairing and mTLS authentication:
//! - `aofctl init ca` - Initialize private CA
//! - `aofctl device register` - Register a new device
//! - `aofctl device list` - List all devices
//! - `aofctl device approve` - Approve a pending device
//! - `aofctl device revoke` - Revoke an approved device
//! - `aofctl device inspect` - Show device details

use aof_core::{AofResult, DeviceInfo, DeviceStatus, DeviceType};
use aof_runtime::device::{PrivateCA, DeviceRegistry};
use chrono::Utc;
use comfy_table::{Table, presets::UTF8_FULL, Cell, Color, Attribute};
use dirs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

/// Initialize the private Certificate Authority.
pub async fn init_ca() -> anyhow::Result<()> {
    let ca_dir = get_ca_dir()?;

    if ca_dir.join("ca.crt").exists() {
        println!("CA already initialized at: {}", ca_dir.display());
        println!("CA certificate: {}/ca.crt", ca_dir.display());
        println!("\nTo re-initialize, delete the CA directory and run again.");
        return Ok(());
    }

    println!("Initializing private Certificate Authority...");

    let ca = PrivateCA::init(ca_dir.clone())?;

    println!("✓ CA initialized successfully");
    println!("  CA directory: {}", ca_dir.display());
    println!("  CA certificate: {}/ca.crt", ca_dir.display());
    println!("  CA private key: {}/ca.key", ca_dir.display());
    println!("\n⚠ Security Warning:");
    println!("  - Keep ca.key secure (permissions: 0600)");
    println!("  - Back up the CA directory");
    println!("  - CA certificate is valid for 10 years");

    Ok(())
}

/// Register a new device and generate client certificate.
pub async fn device_register(
    name: &str,
    device_type_str: &str,
    validity_days: Option<u32>,
) -> anyhow::Result<()> {
    let ca_dir = get_ca_dir()?;
    let devices_dir = get_devices_dir()?;
    let registry_path = get_registry_path()?;

    // Load CA
    let ca = PrivateCA::load(ca_dir)?;

    // Parse device type
    let device_type = DeviceType::from_str(device_type_str)
        .map_err(|e| anyhow::anyhow!("Invalid device type: {}", e))?;

    // Generate device ID
    let device_id = Uuid::new_v4().to_string();

    // Issue client certificate
    let validity = validity_days.unwrap_or(365);
    let cert = ca.issue_client_cert(&device_id, name, &device_type, validity)?;

    // Calculate certificate fingerprint
    let fingerprint = calculate_fingerprint(&cert.cert_pem)?;

    // Create device info
    let device_info = DeviceInfo {
        device_id: device_id.clone(),
        name: name.to_string(),
        device_type: device_type.clone(),
        status: DeviceStatus::Pending,
        certificate_fingerprint: fingerprint,
        registered_at: Utc::now(),
        approved_at: None,
        approved_by: None,
        last_seen: None,
        last_ip: None,
        metadata: HashMap::new(),
    };

    // Register device in registry
    let registry = DeviceRegistry::new(registry_path)?;
    registry.register(device_info).await?;

    // Save certificate and key files
    let device_dir = devices_dir.join(&device_id);
    std::fs::create_dir_all(&device_dir)?;

    std::fs::write(device_dir.join("client.crt"), &cert.cert_pem)?;
    std::fs::write(device_dir.join("client.key"), &cert.key_pem)?;
    std::fs::write(device_dir.join("ca.crt"), &cert.ca_cert_pem)?;

    println!("✓ Device registered successfully");
    println!("  Device ID: {}", device_id);
    println!("  Name: {}", name);
    println!("  Type: {}", device_type);
    println!("  Status: Pending");
    println!("  Valid until: {}", cert.valid_until.format("%Y-%m-%d"));
    println!("\n  Certificates saved to:");
    println!("    {}/client.crt", device_dir.display());
    println!("    {}/client.key", device_dir.display());
    println!("    {}/ca.crt", device_dir.display());
    println!("\n  Next steps:");
    println!("    1. Approve this device: aofctl device approve {}", device_id);
    println!("    2. Connect using mTLS with the certificates above");

    Ok(())
}

/// List all registered devices.
pub async fn device_list(status_filter: Option<&str>) -> anyhow::Result<()> {
    let registry_path = get_registry_path()?;
    let registry = DeviceRegistry::new(registry_path)?;

    let filter = if let Some(status_str) = status_filter {
        match status_str.to_lowercase().as_str() {
            "pending" => Some(DeviceStatus::Pending),
            "approved" => Some(DeviceStatus::Approved),
            "revoked" => Some(DeviceStatus::Revoked),
            "expired" => Some(DeviceStatus::Expired),
            _ => return Err(anyhow::anyhow!("Invalid status filter: {}", status_str)),
        }
    } else {
        None
    };

    let devices = registry.list(filter).await;

    if devices.is_empty() {
        println!("No devices found.");
        if status_filter.is_some() {
            println!("Try running without --status filter to see all devices.");
        }
        return Ok(());
    }

    // Build table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["DEVICE ID", "NAME", "TYPE", "STATUS", "LAST SEEN", "IP"]);

    for device in &devices {
        let status_cell = match device.status {
            DeviceStatus::Approved => Cell::new(&device.status.to_string())
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            DeviceStatus::Pending => Cell::new(&device.status.to_string())
                .fg(Color::Yellow),
            DeviceStatus::Revoked => Cell::new(&device.status.to_string())
                .fg(Color::Red),
            DeviceStatus::Expired => Cell::new(&device.status.to_string())
                .fg(Color::DarkGrey),
        };

        let last_seen = device.last_seen
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Never".to_string());

        let last_ip = device.last_ip.clone().unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(&device.device_id),
            Cell::new(&device.name),
            Cell::new(&device.device_type.to_string()),
            status_cell,
            Cell::new(&last_seen),
            Cell::new(&last_ip),
        ]);
    }

    println!("{}", table);
    println!("\nTotal: {} device(s)", devices.len());

    Ok(())
}

/// Approve a pending device.
pub async fn device_approve(device_id: &str, approved_by: Option<&str>) -> anyhow::Result<()> {
    let registry_path = get_registry_path()?;
    let registry = DeviceRegistry::new(registry_path)?;

    let approver = approved_by.unwrap_or("admin");

    registry.approve(device_id, approver).await?;

    println!("✓ Device approved successfully");
    println!("  Device ID: {}", device_id);
    println!("  Approved by: {}", approver);
    println!("  Approved at: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    println!("\nThe device can now connect using mTLS.");

    Ok(())
}

/// Revoke an approved device.
pub async fn device_revoke(device_id: &str) -> anyhow::Result<()> {
    let registry_path = get_registry_path()?;
    let registry = DeviceRegistry::new(registry_path)?;

    registry.revoke(device_id).await?;

    println!("✓ Device revoked successfully");
    println!("  Device ID: {}", device_id);
    println!("  Revoked at: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    println!("\nThe device can no longer connect.");

    Ok(())
}

/// Inspect device details.
pub async fn device_inspect(device_id: &str) -> anyhow::Result<()> {
    let registry_path = get_registry_path()?;
    let registry = DeviceRegistry::new(registry_path)?;

    let devices = registry.list(None).await;
    let device = devices.iter().find(|d| d.device_id == device_id)
        .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_id))?;

    println!("Device Details:\n");
    println!("  Device ID: {}", device.device_id);
    println!("  Name: {}", device.name);
    println!("  Type: {}", device.device_type);
    println!("  Status: {}", device.status);
    println!("  Certificate Fingerprint: {}", device.certificate_fingerprint);
    println!("  Registered: {}", device.registered_at.format("%Y-%m-%d %H:%M:%S"));

    if let Some(approved_at) = device.approved_at {
        println!("  Approved: {}", approved_at.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(approved_by) = &device.approved_by {
        println!("  Approved By: {}", approved_by);
    }

    if let Some(last_seen) = device.last_seen {
        println!("  Last Seen: {}", last_seen.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(last_ip) = &device.last_ip {
        println!("  Last IP: {}", last_ip);
    }

    if !device.metadata.is_empty() {
        println!("\n  Metadata:");
        for (key, value) in &device.metadata {
            println!("    {}: {}", key, value);
        }
    }

    // Check if certificate files exist
    let devices_dir = get_devices_dir()?;
    let device_dir = devices_dir.join(&device.device_id);

    println!("\n  Certificate Files:");
    if device_dir.exists() {
        println!("    Location: {}", device_dir.display());
        println!("    client.crt: {}", device_dir.join("client.crt").exists());
        println!("    client.key: {}", device_dir.join("client.key").exists());
        println!("    ca.crt: {}", device_dir.join("ca.crt").exists());
    } else {
        println!("    Not found (may have been deleted)");
    }

    Ok(())
}

// Helper functions

fn get_ca_dir() -> anyhow::Result<PathBuf> {
    let ca_dir = dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("aof")
        .join("ca");

    Ok(ca_dir)
}

fn get_devices_dir() -> anyhow::Result<PathBuf> {
    let devices_dir = dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("aof")
        .join("devices");

    std::fs::create_dir_all(&devices_dir)?;
    Ok(devices_dir)
}

fn get_registry_path() -> anyhow::Result<PathBuf> {
    Ok(get_devices_dir()?.join("registry.json"))
}

fn calculate_fingerprint(cert_pem: &str) -> anyhow::Result<String> {
    use sha2::{Sha256, Digest};

    // Simple SHA256 of the PEM content
    let mut hasher = Sha256::new();
    hasher.update(cert_pem.as_bytes());
    let result = hasher.finalize();

    Ok(result.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(":"))
}
