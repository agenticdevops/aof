//! mTLS configuration for device authentication.
//!
//! This module provides TLS configuration for mutual authentication using client certificates.
//! It integrates with the device registry to validate device approval status.

use aof_core::{AofError, AofResult};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rustls::{ServerConfig, RootCertStore};
use rustls::server::WebPkiClientVerifier;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Sha256, Digest};
use x509_parser::prelude::FromDer;

use super::registry::DeviceRegistry;

/// mTLS configuration for the AOF daemon server.
///
/// When enabled, the server requires client certificates signed by the private CA
/// and validates device approval status from the registry before accepting connections.
pub struct MtlsConfig {
    /// Enable mTLS (if false, server operates without client cert validation)
    pub enabled: bool,

    /// Path to CA certificate for validating client certs
    pub ca_cert_path: PathBuf,

    /// Path to server certificate
    pub server_cert_path: PathBuf,

    /// Path to server private key
    pub server_key_path: PathBuf,

    /// Require client certificate (true for mTLS)
    pub require_client_cert: bool,

    /// Device registry for approval status checks
    pub device_registry: Option<Arc<DeviceRegistry>>,
}

impl MtlsConfig {
    /// Create a new mTLS configuration.
    pub fn new(
        ca_cert_path: PathBuf,
        server_cert_path: PathBuf,
        server_key_path: PathBuf,
    ) -> Self {
        Self {
            enabled: true,
            ca_cert_path,
            server_cert_path,
            server_key_path,
            require_client_cert: true,
            device_registry: None,
        }
    }

    /// Attach a device registry for approval checks.
    pub fn with_registry(mut self, registry: Arc<DeviceRegistry>) -> Self {
        self.device_registry = Some(registry);
        self
    }

    /// Build a rustls ServerConfig configured for mTLS.
    pub fn build_tls_config(&self) -> AofResult<Arc<ServerConfig>> {
        if !self.enabled {
            return Err(AofError::config("mTLS not enabled"));
        }

        // Load CA certificate for client verification
        let ca_cert_pem = fs::read_to_string(&self.ca_cert_path).map_err(|e| {
            AofError::config(format!("Failed to read CA cert: {}", e))
        })?;

        let ca_certs = load_certs_from_pem(&ca_cert_pem).map_err(|e| {
            AofError::config(format!("Failed to parse CA cert: {}", e))
        })?;

        // Build root cert store with CA cert
        let mut root_store = RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(cert).map_err(|e| {
                AofError::config(format!("Failed to add CA to root store: {}", e))
            })?;
        }

        // Load server certificate
        let server_cert_pem = fs::read_to_string(&self.server_cert_path).map_err(|e| {
            AofError::config(format!("Failed to read server cert: {}", e))
        })?;

        let server_certs = load_certs_from_pem(&server_cert_pem).map_err(|e| {
            AofError::config(format!("Failed to parse server cert: {}", e))
        })?;

        // Load server private key
        let server_key_pem = fs::read_to_string(&self.server_key_path).map_err(|e| {
            AofError::config(format!("Failed to read server key: {}", e))
        })?;

        let server_key = load_private_key_from_pem(&server_key_pem).map_err(|e| {
            AofError::config(format!("Failed to parse server key: {}", e))
        })?;

        // Create client verifier
        let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .map_err(|e| AofError::config(format!("Failed to build client verifier: {}", e)))?;

        // Build server config with client authentication
        let mut config = ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, server_key)
            .map_err(|e| AofError::config(format!("Failed to build TLS config: {}", e)))?;

        // Enable ALPN for HTTP/2 and HTTP/1.1
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(Arc::new(config))
    }

    /// Extract device_id from client certificate.
    ///
    /// The device_id is embedded in the certificate's Subject Alternative Names (SAN)
    /// as "device-{uuid}".
    pub fn extract_device_id(cert_der: &[u8]) -> AofResult<String> {
        // For now, parse the DER-encoded cert to extract SANs
        // This is a simplified implementation - production would use x509-parser

        // Parse the certificate
        let cert = x509_parser::parse_x509_certificate(cert_der)
            .map_err(|e| AofError::agent(format!("Failed to parse client cert: {}", e)))?
            .1;

        // Look for SubjectAlternativeName extension
        if let Some(san_ext) = cert.tbs_certificate.get_extension_unique(
            &x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME
        ).map_err(|e| AofError::agent(format!("Failed to get SAN extension: {}", e)))? {

            let san = x509_parser::extensions::SubjectAlternativeName::from_der(san_ext.value)
                .map_err(|e| AofError::agent(format!("Failed to parse SAN: {}", e)))?
                .1;

            // Find device-{uuid} in DNS names
            for name in &san.general_names {
                if let x509_parser::extensions::GeneralName::DNSName(dns_name) = name {
                    if let Some(device_id) = dns_name.strip_prefix("device-") {
                        return Ok(device_id.to_string());
                    }
                }
            }
        }

        Err(AofError::agent("No device_id found in client certificate SAN"))
    }

    /// Calculate SHA256 fingerprint of a certificate.
    pub fn cert_fingerprint(cert_der: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_der);
        let result = hasher.finalize();

        // Format as hex string with colons
        result.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Check if a device with the given ID is approved in the registry.
    pub async fn is_device_approved(&self, device_id: &str) -> bool {
        if let Some(registry) = &self.device_registry {
            registry.is_approved(device_id).await
        } else {
            // No registry configured - allow all devices with valid certs
            true
        }
    }
}

/// Load certificates from PEM-encoded string.
fn load_certs_from_pem(pem: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(pem.as_bytes());
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

/// Load private key from PEM-encoded string.
fn load_private_key_from_pem(pem: &str) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(pem.as_bytes());

    // Try PKCS8 first
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut reader).next() {
        return Ok(PrivateKeyDer::Pkcs8(key?));
    }

    // Try RSA
    let mut reader = BufReader::new(pem.as_bytes());
    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut reader).next() {
        return Ok(PrivateKeyDer::Pkcs1(key?));
    }

    Err("No valid private key found in PEM".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::device::ca::PrivateCA;
    use aof_core::DeviceType;

    #[test]
    fn test_mtls_config_creation() {
        let config = MtlsConfig::new(
            PathBuf::from("/tmp/ca.crt"),
            PathBuf::from("/tmp/server.crt"),
            PathBuf::from("/tmp/server.key"),
        );

        assert!(config.enabled);
        assert!(config.require_client_cert);
        assert!(config.device_registry.is_none());
    }

    #[test]
    fn test_cert_fingerprint() {
        let cert_data = b"test certificate data";
        let fingerprint = MtlsConfig::cert_fingerprint(cert_data);

        // Should be hex string with colons
        assert!(fingerprint.contains(':'));
        assert_eq!(fingerprint.split(':').count(), 32); // SHA256 = 32 bytes
    }

    #[tokio::test]
    async fn test_build_tls_config_with_real_certs() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");

        // Create CA
        let ca = PrivateCA::init(ca_dir.clone()).unwrap();

        // Issue server certificate
        let server_cert = ca.issue_client_cert(
            "server-001",
            "aof-server",
            &DeviceType::ApiClient,
            365,
        ).unwrap();

        // Write server cert and key
        let server_cert_path = temp_dir.path().join("server.crt");
        let server_key_path = temp_dir.path().join("server.key");

        fs::write(&server_cert_path, &server_cert.cert_pem).unwrap();
        fs::write(&server_key_path, &server_cert.key_pem).unwrap();

        // Create mTLS config
        let mtls_config = MtlsConfig::new(
            ca_dir.join("ca.crt"),
            server_cert_path,
            server_key_path,
        );

        // Should successfully build TLS config
        let tls_config = mtls_config.build_tls_config();
        assert!(tls_config.is_ok());
    }

    #[tokio::test]
    async fn test_device_approval_check() {
        use super::super::registry::DeviceRegistry;
        use aof_core::{DeviceInfo, DeviceStatus};
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let registry = Arc::new(DeviceRegistry::new(registry_path).unwrap());

        // Register a device
        let device = DeviceInfo {
            device_id: "dev-123".to_string(),
            name: "test-device".to_string(),
            device_type: DeviceType::Cli,
            status: DeviceStatus::Pending,
            certificate_fingerprint: "test-fingerprint".to_string(),
            registered_at: chrono::Utc::now(),
            approved_at: None,
            approved_by: None,
            last_seen: None,
            last_ip: None,
            metadata: HashMap::new(),
        };

        registry.register(device).await.unwrap();

        // Create mTLS config with registry
        let mtls_config = MtlsConfig::new(
            PathBuf::from("/tmp/ca.crt"),
            PathBuf::from("/tmp/server.crt"),
            PathBuf::from("/tmp/server.key"),
        ).with_registry(Arc::clone(&registry));

        // Device should not be approved yet
        assert!(!mtls_config.is_device_approved("dev-123").await);

        // Approve device
        registry.approve("dev-123", "admin").await.unwrap();

        // Now should be approved
        assert!(mtls_config.is_device_approved("dev-123").await);
    }
}
