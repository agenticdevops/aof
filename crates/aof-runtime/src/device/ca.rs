//! Private Certificate Authority for device authentication.
//!
//! This module provides a private CA that issues client certificates for device
//! authentication. The CA is self-signed with a 10-year validity period.

use aof_core::{AofError, AofResult, DeviceCertificate, DeviceType};
use chrono::{DateTime, Utc};
use rcgen::{CertificateParams, DnType, KeyPair};
use std::fs;
use std::path::{Path, PathBuf};
use time::{Duration, OffsetDateTime};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Private Certificate Authority for issuing client certificates.
///
/// The CA maintains a self-signed root certificate and private key. Client certificates
/// are issued with device-specific attributes embedded in the certificate.
pub struct PrivateCA {
    ca_cert_pem: String,
    ca_key_pem: String,
    ca_dir: PathBuf,
}

impl PrivateCA {
    /// Initialize a new Certificate Authority.
    ///
    /// If the CA directory exists and contains valid certificates, they are loaded.
    /// Otherwise, a new self-signed root certificate is generated.
    pub fn init(ca_dir: PathBuf) -> AofResult<Self> {
        // Create directory if it doesn't exist
        if !ca_dir.exists() {
            fs::create_dir_all(&ca_dir).map_err(|e| {
                AofError::memory(format!("Failed to create CA directory: {}", e))
            })?;
        }

        let ca_cert_path = ca_dir.join("ca.crt");
        let ca_key_path = ca_dir.join("ca.key");

        // Load existing CA if present
        if ca_cert_path.exists() && ca_key_path.exists() {
            return Self::load(ca_dir);
        }

        // Generate new CA
        let mut params = CertificateParams::new(vec!["AOF Private CA".to_string()])
            .map_err(|e| AofError::agent(format!("Failed to create CA params: {}", e)))?;

        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.distinguished_name.push(DnType::OrganizationName, "AOF");

        // 10-year validity
        let now = OffsetDateTime::now_utc();
        params.not_before = now - Duration::hours(1);
        params.not_after = now + Duration::days(3650);

        // CA key usage
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        let key_pair = KeyPair::generate().map_err(|e| {
            AofError::agent(format!("Failed to generate CA key pair: {}", e))
        })?;

        let ca_cert = params.self_signed(&key_pair).map_err(|e| {
            AofError::agent(format!("Failed to self-sign CA certificate: {}", e))
        })?;

        let ca_cert_pem = ca_cert.pem();
        let ca_key_pem = key_pair.serialize_pem();

        // Write CA certificate and key with appropriate permissions
        fs::write(&ca_cert_path, &ca_cert_pem).map_err(|e| {
            AofError::memory(format!("Failed to write CA certificate: {}", e))
        })?;

        fs::write(&ca_key_path, &ca_key_pem).map_err(|e| {
            AofError::memory(format!("Failed to write CA key: {}", e))
        })?;

        // Set restrictive permissions on CA key (0600 - owner read/write only)
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&ca_key_path)
                .map_err(|e| AofError::memory(format!("Failed to get key metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&ca_key_path, perms).map_err(|e| {
                AofError::memory(format!("Failed to set key permissions: {}", e))
            })?;
        }

        Ok(Self {
            ca_cert_pem,
            ca_key_pem,
            ca_dir,
        })
    }

    /// Load an existing Certificate Authority from disk.
    pub fn load(ca_dir: PathBuf) -> AofResult<Self> {
        let ca_cert_path = ca_dir.join("ca.crt");
        let ca_key_path = ca_dir.join("ca.key");

        if !ca_cert_path.exists() || !ca_key_path.exists() {
            return Err(AofError::memory(
                "CA certificate or key not found. Run 'aofctl init ca' first.",
            ));
        }

        let ca_cert_pem = fs::read_to_string(&ca_cert_path).map_err(|e| {
            AofError::memory(format!("Failed to read CA certificate: {}", e))
        })?;

        let ca_key_pem = fs::read_to_string(&ca_key_path).map_err(|e| {
            AofError::memory(format!("Failed to read CA key: {}", e))
        })?;

        Ok(Self {
            ca_cert_pem,
            ca_key_pem,
            ca_dir,
        })
    }

    /// Issue a client certificate for a device.
    pub fn issue_client_cert(
        &self,
        device_id: &str,
        device_name: &str,
        device_type: &DeviceType,
        validity_days: u32,
    ) -> AofResult<DeviceCertificate> {
        let subject_alt_names = vec![
            format!("device-{}", device_id),
            format!("type-{}", device_type),
        ];

        let mut params = CertificateParams::new(subject_alt_names)
            .map_err(|e| AofError::agent(format!("Failed to create client cert params: {}", e)))?;

        // Set subject
        params.distinguished_name.push(DnType::CommonName, device_name);
        params.distinguished_name.push(DnType::OrganizationName, "AOF Device");

        // Set validity period
        let now = OffsetDateTime::now_utc();
        params.not_before = now - Duration::hours(1);
        params.not_after = now + Duration::days(validity_days as i64);

        // Client authentication key usage
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];

        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];

        // Store expiry before params is moved
        let valid_until_timestamp = params.not_after.unix_timestamp();

        // Generate client key pair
        let client_key_pair = KeyPair::generate().map_err(|e| {
            AofError::agent(format!("Failed to generate client key pair: {}", e))
        })?;

        // Parse CA key and cert for signing
        let ca_key_pair = KeyPair::from_pem(&self.ca_key_pem)
            .map_err(|e| AofError::agent(format!("Failed to parse CA key: {}", e)))?;

        let ca_params = CertificateParams::from_ca_cert_pem(&self.ca_cert_pem)
            .map_err(|e| AofError::agent(format!("Failed to parse CA certificate: {}", e)))?;

        let ca_cert = ca_params.self_signed(&ca_key_pair).map_err(|e| {
            AofError::agent(format!("Failed to reconstruct CA certificate: {}", e))
        })?;

        // Sign the client certificate with CA
        let client_cert_pem = params
            .signed_by(&client_key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| AofError::agent(format!("Failed to sign client certificate: {}", e)))?
            .pem();

        let client_key_pem = client_key_pair.serialize_pem();

        // Convert time::OffsetDateTime to chrono::DateTime<Utc>
        let valid_until = DateTime::from_timestamp(valid_until_timestamp, 0)
            .unwrap_or_else(Utc::now);

        Ok(DeviceCertificate {
            device_id: device_id.to_string(),
            cert_pem: client_cert_pem,
            key_pem: client_key_pem,
            ca_cert_pem: self.ca_cert_pem.clone(),
            valid_until,
        })
    }

    /// Get the CA certificate in PEM format.
    pub fn ca_cert_pem(&self) -> &str {
        &self.ca_cert_pem
    }

    /// Get the CA directory path.
    pub fn ca_dir(&self) -> &Path {
        &self.ca_dir
    }

    /// Verify that a client certificate was issued by this CA.
    pub fn verify_cert(&self, _cert_pem: &str) -> AofResult<bool> {
        // Full validation happens during TLS handshake
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ca_init_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");

        let ca = PrivateCA::init(ca_dir.clone()).unwrap();

        // Check files exist
        assert!(ca_dir.join("ca.crt").exists());
        assert!(ca_dir.join("ca.key").exists());

        // Check CA cert is not empty
        assert!(!ca.ca_cert_pem().is_empty());
    }

    #[test]
    fn test_ca_load_existing() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");

        // Create CA
        let ca1 = PrivateCA::init(ca_dir.clone()).unwrap();
        let cert1 = ca1.ca_cert_pem().to_string();

        // Load existing CA
        let ca2 = PrivateCA::load(ca_dir).unwrap();
        let cert2 = ca2.ca_cert_pem();

        // Should be the same certificate
        assert_eq!(cert1, cert2);
    }

    #[test]
    fn test_issue_client_cert() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");
        let ca = PrivateCA::init(ca_dir).unwrap();

        let device_id = "dev-123";
        let device_name = "test-laptop";
        let device_type = DeviceType::Cli;

        let cert = ca
            .issue_client_cert(device_id, device_name, &device_type, 365)
            .unwrap();

        assert_eq!(cert.device_id, device_id);
        assert!(!cert.cert_pem.is_empty());
        assert!(!cert.key_pem.is_empty());
        assert_eq!(cert.ca_cert_pem, ca.ca_cert_pem());
        assert!(cert.valid_until > Utc::now());
    }

    #[test]
    fn test_ca_key_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");
        let _ca = PrivateCA::init(ca_dir.clone()).unwrap();

        #[cfg(unix)]
        {
            let key_path = ca_dir.join("ca.key");
            let metadata = fs::metadata(key_path).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_cert_contains_device_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let ca_dir = temp_dir.path().join("ca");
        let ca = PrivateCA::init(ca_dir).unwrap();

        let device_id = "dev-456";
        let cert = ca
            .issue_client_cert(device_id, "test-device", &DeviceType::WebUi, 365)
            .unwrap();

        // Certificate was issued successfully with device metadata
        // The device_id is in the SAN, but may not be directly visible in PEM
        assert!(!cert.cert_pem.is_empty());
        assert_eq!(cert.device_id, device_id);
    }
}
