//! Device pairing and authentication types for mTLS-based device management.
//!
//! This module provides the core types for device registration, approval workflows,
//! and certificate-based authentication. Devices can be CLIs, web UIs, bots, or custom
//! clients that connect to the AOF daemon using mutual TLS.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a registered device.
///
/// Devices progress through a lifecycle: registered (Pending) → approved by operator
/// (Approved) → optionally revoked (Revoked) or expired (Expired).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    /// Unique device identifier (UUID).
    pub device_id: String,

    /// Human-readable name (e.g., "mission-control-laptop").
    pub name: String,

    /// Type of device (CLI, WebUI, bot, etc.).
    pub device_type: DeviceType,

    /// Current approval status.
    pub status: DeviceStatus,

    /// SHA256 fingerprint of the client certificate.
    pub certificate_fingerprint: String,

    /// Timestamp when the device was first registered.
    pub registered_at: DateTime<Utc>,

    /// Timestamp when the device was approved (if applicable).
    pub approved_at: Option<DateTime<Utc>>,

    /// Identity of the operator who approved the device.
    pub approved_by: Option<String>,

    /// Last time the device successfully connected.
    pub last_seen: Option<DateTime<Utc>>,

    /// Last IP address the device connected from.
    pub last_ip: Option<String>,

    /// Additional device metadata (e.g., OS version, hostname).
    pub metadata: HashMap<String, String>,
}

/// Type of device connecting to the AOF daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    /// Command-line aofctl client.
    Cli,

    /// Mission Control web dashboard.
    WebUi,

    /// Slack integration bot.
    SlackBot,

    /// Discord integration bot.
    DiscordBot,

    /// Generic API client.
    ApiClient,

    /// Custom device type with user-defined label.
    Custom(String),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Cli => write!(f, "cli"),
            DeviceType::WebUi => write!(f, "web_ui"),
            DeviceType::SlackBot => write!(f, "slack_bot"),
            DeviceType::DiscordBot => write!(f, "discord_bot"),
            DeviceType::ApiClient => write!(f, "api_client"),
            DeviceType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for DeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cli" => Ok(DeviceType::Cli),
            "web_ui" | "webui" => Ok(DeviceType::WebUi),
            "slack_bot" | "slackbot" => Ok(DeviceType::SlackBot),
            "discord_bot" | "discordbot" => Ok(DeviceType::DiscordBot),
            "api_client" | "apiclient" => Ok(DeviceType::ApiClient),
            custom => Ok(DeviceType::Custom(custom.to_string())),
        }
    }
}

/// Device approval status in the registration workflow.
///
/// Flow: Pending → Approved → (optionally) Revoked
///       Pending → Expired (if cert expires before approval)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    /// Registered but not yet approved by an operator.
    Pending,

    /// Approved by an operator and allowed to connect.
    Approved,

    /// Previously approved but revoked (blocked from connecting).
    Revoked,

    /// Certificate expired.
    Expired,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Pending => write!(f, "pending"),
            DeviceStatus::Approved => write!(f, "approved"),
            DeviceStatus::Revoked => write!(f, "revoked"),
            DeviceStatus::Expired => write!(f, "expired"),
        }
    }
}

impl DeviceStatus {
    /// Check if a device with this status can connect to the daemon.
    pub fn can_connect(&self) -> bool {
        matches!(self, DeviceStatus::Approved)
    }

    /// Check if this status represents a terminal state (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(self, DeviceStatus::Expired)
    }
}

/// A client certificate issued for a device.
///
/// Contains the device's private key, certificate, and CA certificate for mTLS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCertificate {
    /// Device identifier this certificate was issued for.
    pub device_id: String,

    /// Client certificate in PEM format.
    pub cert_pem: String,

    /// Private key in PEM format.
    pub key_pem: String,

    /// CA certificate in PEM format (for validating server cert).
    pub ca_cert_pem: String,

    /// Certificate expiration timestamp.
    pub valid_until: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_device_info_serialization() {
        let device = DeviceInfo {
            device_id: "dev-123".to_string(),
            name: "test-laptop".to_string(),
            device_type: DeviceType::Cli,
            status: DeviceStatus::Pending,
            certificate_fingerprint: "aa:bb:cc".to_string(),
            registered_at: Utc::now(),
            approved_at: None,
            approved_by: None,
            last_seen: None,
            last_ip: None,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(device.device_id, deserialized.device_id);
        assert_eq!(device.name, deserialized.name);
        assert_eq!(device.device_type, deserialized.device_type);
        assert_eq!(device.status, deserialized.status);
    }

    #[test]
    fn test_device_status_transitions() {
        // Pending -> Approved
        let mut status = DeviceStatus::Pending;
        assert!(!status.can_connect());

        status = DeviceStatus::Approved;
        assert!(status.can_connect());

        // Approved -> Revoked
        status = DeviceStatus::Revoked;
        assert!(!status.can_connect());

        // Expired is terminal
        status = DeviceStatus::Expired;
        assert!(!status.can_connect());
        assert!(status.is_terminal());
    }

    #[test]
    fn test_device_type_equality() {
        assert_eq!(DeviceType::Cli, DeviceType::Cli);
        assert_ne!(DeviceType::Cli, DeviceType::WebUi);
        assert_eq!(DeviceType::Custom("foo".into()), DeviceType::Custom("foo".into()));
        assert_ne!(DeviceType::Custom("foo".into()), DeviceType::Custom("bar".into()));
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::Cli.to_string(), "cli");
        assert_eq!(DeviceType::WebUi.to_string(), "web_ui");
        assert_eq!(DeviceType::SlackBot.to_string(), "slack_bot");
        assert_eq!(DeviceType::Custom("robot".into()).to_string(), "robot");
    }

    #[test]
    fn test_device_type_from_str() {
        assert_eq!(DeviceType::from_str("cli").unwrap(), DeviceType::Cli);
        assert_eq!(DeviceType::from_str("webui").unwrap(), DeviceType::WebUi);
        assert_eq!(DeviceType::from_str("web_ui").unwrap(), DeviceType::WebUi);
        assert_eq!(DeviceType::from_str("slack_bot").unwrap(), DeviceType::SlackBot);
        assert_eq!(DeviceType::from_str("custom_type").unwrap(), DeviceType::Custom("custom_type".into()));
    }

    #[test]
    fn test_device_status_display() {
        assert_eq!(DeviceStatus::Pending.to_string(), "pending");
        assert_eq!(DeviceStatus::Approved.to_string(), "approved");
        assert_eq!(DeviceStatus::Revoked.to_string(), "revoked");
        assert_eq!(DeviceStatus::Expired.to_string(), "expired");
    }

    #[test]
    fn test_device_certificate_serialization() {
        let cert = DeviceCertificate {
            device_id: "dev-456".to_string(),
            cert_pem: "-----BEGIN CERTIFICATE-----\n...".to_string(),
            key_pem: "-----BEGIN PRIVATE KEY-----\n...".to_string(),
            ca_cert_pem: "-----BEGIN CERTIFICATE-----\n...".to_string(),
            valid_until: Utc::now(),
        };

        let json = serde_json::to_string(&cert).unwrap();
        let deserialized: DeviceCertificate = serde_json::from_str(&json).unwrap();

        assert_eq!(cert.device_id, deserialized.device_id);
    }
}
