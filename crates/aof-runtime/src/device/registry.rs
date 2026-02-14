//! Device registry with approval workflows and persistent storage.
//!
//! The registry tracks all registered devices, their approval status, and connection history.

use aof_core::{AofError, AofResult, DeviceInfo, DeviceStatus};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persistent device registry.
///
/// Maintains a mapping of device_id -> DeviceInfo with automatic persistence to disk.
pub struct DeviceRegistry {
    devices: RwLock<HashMap<String, DeviceInfo>>,
    persist_path: PathBuf,
}

impl DeviceRegistry {
    /// Create a new device registry.
    ///
    /// If the registry file exists, devices are loaded from disk.
    pub fn new(persist_path: PathBuf) -> AofResult<Self> {
        let devices = if persist_path.exists() {
            Self::load(&persist_path)?
        } else {
            // Create parent directory if needed
            if let Some(parent) = persist_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| {
                        AofError::memory(format!("Failed to create registry directory: {}", e))
                    })?;
                }
            }
            HashMap::new()
        };

        Ok(Self {
            devices: RwLock::new(devices),
            persist_path,
        })
    }

    /// Register a new device with Pending status.
    pub async fn register(&self, device: DeviceInfo) -> AofResult<()> {
        {
            let mut devices = self.devices.write();
            devices.insert(device.device_id.clone(), device);
        }
        self.save().await
    }

    /// Approve a pending device.
    pub async fn approve(&self, device_id: &str, approved_by: &str) -> AofResult<()> {
        {
            let mut devices = self.devices.write();
            let device = devices
                .get_mut(device_id)
                .ok_or_else(|| AofError::agent(format!("Device {} not found", device_id)))?;

            if device.status == DeviceStatus::Pending {
                device.status = DeviceStatus::Approved;
                device.approved_at = Some(Utc::now());
                device.approved_by = Some(approved_by.to_string());
            } else {
                return Err(AofError::agent(format!(
                    "Device {} is not pending (current status: {})",
                    device_id, device.status
                )));
            }
        }
        self.save().await
    }

    /// Revoke an approved device.
    pub async fn revoke(&self, device_id: &str) -> AofResult<()> {
        {
            let mut devices = self.devices.write();
            let device = devices
                .get_mut(device_id)
                .ok_or_else(|| AofError::agent(format!("Device {} not found", device_id)))?;

            if device.status == DeviceStatus::Approved {
                device.status = DeviceStatus::Revoked;
            } else {
                return Err(AofError::agent(format!(
                    "Device {} is not approved (current status: {})",
                    device_id, device.status
                )));
            }
        }
        self.save().await
    }

    /// Check if a device is approved.
    pub async fn is_approved(&self, device_id: &str) -> bool {
        let devices = self.devices.read();
        devices
            .get(device_id)
            .map(|d| d.status == DeviceStatus::Approved)
            .unwrap_or(false)
    }

    /// Look up device by certificate fingerprint.
    pub async fn find_by_fingerprint(&self, fingerprint: &str) -> Option<DeviceInfo> {
        let devices = self.devices.read();
        devices
            .values()
            .find(|d| d.certificate_fingerprint == fingerprint)
            .cloned()
    }

    /// Update last_seen timestamp and IP for a device.
    pub async fn record_connection(&self, device_id: &str, ip: &str) -> AofResult<()> {
        {
            let mut devices = self.devices.write();
            if let Some(device) = devices.get_mut(device_id) {
                device.last_seen = Some(Utc::now());
                device.last_ip = Some(ip.to_string());
            } else {
                return Err(AofError::agent(format!("Device {} not found", device_id)));
            }
        }
        self.save().await
    }

    /// List all devices, optionally filtered by status.
    pub async fn list(&self, status_filter: Option<DeviceStatus>) -> Vec<DeviceInfo> {
        let devices = self.devices.read();
        devices
            .values()
            .filter(|d| status_filter.as_ref().map_or(true, |s| &d.status == s))
            .cloned()
            .collect()
    }

    /// Persist registry to disk.
    async fn save(&self) -> AofResult<()> {
        let devices = self.devices.read();
        let json = serde_json::to_string_pretty(&*devices).map_err(|e| {
            AofError::memory(format!("Failed to serialize device registry: {}", e))
        })?;

        tokio::fs::write(&self.persist_path, json)
            .await
            .map_err(|e| {
                AofError::memory(format!("Failed to write device registry: {}", e))
            })?;

        Ok(())
    }

    /// Load registry from disk.
    fn load(path: &PathBuf) -> AofResult<HashMap<String, DeviceInfo>> {
        let content = fs::read_to_string(path).map_err(|e| {
            AofError::memory(format!("Failed to read device registry: {}", e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            AofError::memory(format!("Failed to parse device registry: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::DeviceType;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_device(device_id: &str, name: &str) -> DeviceInfo {
        DeviceInfo {
            device_id: device_id.to_string(),
            name: name.to_string(),
            device_type: DeviceType::Cli,
            status: DeviceStatus::Pending,
            certificate_fingerprint: "test-fingerprint".to_string(),
            registered_at: Utc::now(),
            approved_at: None,
            approved_by: None,
            last_seen: None,
            last_ip: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_register_device() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let device = create_test_device("dev-1", "test-device");
        registry.register(device.clone()).await.unwrap();

        let devices = registry.list(None).await;
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "dev-1");
        assert_eq!(devices[0].status, DeviceStatus::Pending);
    }

    #[tokio::test]
    async fn test_approve_device() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let device = create_test_device("dev-1", "test-device");
        registry.register(device).await.unwrap();

        registry.approve("dev-1", "admin").await.unwrap();

        let devices = registry.list(None).await;
        assert_eq!(devices[0].status, DeviceStatus::Approved);
        assert!(devices[0].approved_at.is_some());
        assert_eq!(devices[0].approved_by, Some("admin".to_string()));
    }

    #[tokio::test]
    async fn test_revoke_device() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let device = create_test_device("dev-1", "test-device");
        registry.register(device).await.unwrap();
        registry.approve("dev-1", "admin").await.unwrap();
        registry.revoke("dev-1").await.unwrap();

        let devices = registry.list(None).await;
        assert_eq!(devices[0].status, DeviceStatus::Revoked);
    }

    #[tokio::test]
    async fn test_is_approved() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let device = create_test_device("dev-1", "test-device");
        registry.register(device).await.unwrap();

        assert!(!registry.is_approved("dev-1").await);

        registry.approve("dev-1", "admin").await.unwrap();

        assert!(registry.is_approved("dev-1").await);
    }

    #[tokio::test]
    async fn test_find_by_fingerprint() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let mut device = create_test_device("dev-1", "test-device");
        device.certificate_fingerprint = "unique-fingerprint".to_string();
        registry.register(device).await.unwrap();

        let found = registry
            .find_by_fingerprint("unique-fingerprint")
            .await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().device_id, "dev-1");

        let not_found = registry.find_by_fingerprint("wrong-fingerprint").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");

        // Create and populate registry
        {
            let registry = DeviceRegistry::new(registry_path.clone()).unwrap();
            let device = create_test_device("dev-1", "test-device");
            registry.register(device).await.unwrap();
        }

        // Load registry from disk
        let registry2 = DeviceRegistry::new(registry_path).unwrap();
        let devices = registry2.list(None).await;
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "dev-1");
    }

    #[tokio::test]
    async fn test_list_with_status_filter() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(registry_path).unwrap();

        let dev1 = create_test_device("dev-1", "device-1");
        let dev2 = create_test_device("dev-2", "device-2");
        registry.register(dev1).await.unwrap();
        registry.register(dev2).await.unwrap();
        registry.approve("dev-1", "admin").await.unwrap();

        let pending = registry.list(Some(DeviceStatus::Pending)).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].device_id, "dev-2");

        let approved = registry.list(Some(DeviceStatus::Approved)).await;
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].device_id, "dev-1");

        let all = registry.list(None).await;
        assert_eq!(all.len(), 2);
    }
}
