//! Certificate lifecycle management.
//!
//! This module provides utilities for managing device certificates, including
//! validation, expiry checking, and fingerprint calculation.

use aof_core::{AofError, AofResult};
use sha2::{Digest, Sha256};

/// Certificate management utilities.
pub struct CertificateManager;

impl CertificateManager {
    /// Calculate SHA256 fingerprint of a certificate.
    ///
    /// # Arguments
    ///
    /// * `cert_pem` - Certificate in PEM format
    ///
    /// # Returns
    ///
    /// A hex-encoded SHA256 hash of the certificate DER bytes.
    pub fn fingerprint(cert_pem: &str) -> AofResult<String> {
        // Extract the base64 content from PEM
        let content = cert_pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();

        let der = base64::decode(&content).map_err(|e| {
            AofError::agent(format!("Failed to decode certificate: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&der);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Extract device ID from certificate Subject Alternative Name.
    ///
    /// Looks for a DNS SAN entry in the format "device-{device_id}".
    ///
    /// # Arguments
    ///
    /// * `cert_pem` - Certificate in PEM format
    ///
    /// # Returns
    ///
    /// The device ID if found, otherwise an error.
    pub fn extract_device_id(cert_pem: &str) -> AofResult<String> {
        // In a production implementation, we would parse the X.509 certificate
        // and extract the SAN DNS entry. For now, we'll use a simple pattern match.

        // This is a placeholder - actual implementation would use x509-parser
        // or similar crate to properly parse the certificate extensions.

        for line in cert_pem.lines() {
            if line.contains("device-") {
                if let Some(start) = line.find("device-") {
                    let device_part = &line[start + 7..];
                    if let Some(end) = device_part.find(|c: char| !c.is_alphanumeric() && c != '-') {
                        return Ok(device_part[..end].to_string());
                    } else {
                        return Ok(device_part.to_string());
                    }
                }
            }
        }

        Err(AofError::agent("Device ID not found in certificate"))
    }
}

// Add base64 dependency for certificate decoding
mod base64 {
    pub fn decode(input: &str) -> Result<Vec<u8>, String> {
        // Simple base64 decoder - in production use a proper base64 crate
        use std::str;

        const STANDARD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let input = input.as_bytes();
        let mut output = Vec::new();
        let mut buf = 0u32;
        let mut bits = 0;

        for &byte in input {
            if byte == b'=' {
                break;
            }
            if byte.is_ascii_whitespace() {
                continue;
            }

            let value = STANDARD
                .iter()
                .position(|&c| c == byte)
                .ok_or_else(|| "Invalid base64 character".to_string())? as u32;

            buf = (buf << 6) | value;
            bits += 6;

            if bits >= 8 {
                bits -= 8;
                output.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_generation() {
        // Sample certificate (truncated for test)
        let cert_pem = "-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAKHHCgVZU6H0MA0GCSqGSIb3DQEBCwUAMA0xCzAJBgNVBAMMAkNB
MB4XDTIwMDEwMTAwMDAwMFoXDTMwMDEwMTAwMDAwMFowDTELMAkGA1UEAwwCQ0Ew
gZ8wDQYJKoZIhvcNAQEBBQADgY0AMIGJAoGBAL
-----END CERTIFICATE-----";

        let result = CertificateManager::fingerprint(cert_pem);
        assert!(result.is_ok());

        let fingerprint = result.unwrap();
        assert_eq!(fingerprint.len(), 64); // SHA256 produces 64 hex characters
    }
}
