//! Security types for OpenAgentiX — SSRF protection, capability policies, and security configuration.
//!
//! This module provides the foundational security primitives used throughout the framework:
//! - `SsrfGuard` — validates URLs before outbound HTTP requests
//! - `SsrfViolation` — error type for SSRF policy violations
//! - `CapabilityPolicy` — wraps WASM capability checks with runtime enforcement
//! - `SecurityConfig` — workspace-level security settings

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;

// ---------------------------------------------------------------------------
// SsrfViolation
// ---------------------------------------------------------------------------

/// Error returned when an outbound URL violates SSRF protection policy.
#[derive(Debug, Clone)]
pub struct SsrfViolation {
    /// The URL that was blocked.
    pub url: String,
    /// Human-readable reason for the block.
    pub reason: String,
}

impl fmt::Display for SsrfViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SSRF blocked: {} ({})", self.url, self.reason)
    }
}

impl std::error::Error for SsrfViolation {}

// ---------------------------------------------------------------------------
// SsrfGuard
// ---------------------------------------------------------------------------

/// Validates URLs before outbound HTTP requests to prevent SSRF attacks.
///
/// Blocks requests to:
/// - RFC 1918 private IPs (10.x, 172.16-31.x, 192.168.x)
/// - Loopback addresses (127.0.0.0/8, ::1, localhost)
/// - Link-local addresses (169.254.x.x — AWS/cloud metadata endpoints)
///
/// Supports a configurable allow-list to permit specific private IPs for
/// internal services (e.g., Prometheus, Grafana).
#[derive(Debug, Clone)]
pub struct SsrfGuard {
    /// Private IPs/hostnames that are explicitly allowed despite being in blocked ranges.
    pub allowed_private_endpoints: Vec<String>,
}

impl Default for SsrfGuard {
    fn default() -> Self {
        Self {
            allowed_private_endpoints: Vec::new(),
        }
    }
}

impl SsrfGuard {
    /// Create an SsrfGuard with an explicit allow-list of private endpoints.
    pub fn with_allowed(endpoints: Vec<String>) -> Self {
        Self {
            allowed_private_endpoints: endpoints,
        }
    }

    /// Check whether a URL is safe for outbound requests.
    ///
    /// Returns `Ok(())` if the URL targets a public IP or an allow-listed private IP.
    /// Returns `Err(SsrfViolation)` if the URL targets a blocked private/internal address.
    pub fn check_url(&self, url: &str) -> Result<(), SsrfViolation> {
        // Parse the URL to extract the host
        let host = Self::extract_host(url).ok_or_else(|| SsrfViolation {
            url: url.to_string(),
            reason: "invalid URL: cannot extract host".to_string(),
        })?;

        // Check if the host is "localhost"
        if host == "localhost" {
            if self.is_allowed(&host) {
                return Ok(());
            }
            return Err(SsrfViolation {
                url: url.to_string(),
                reason: "blocked: localhost".to_string(),
            });
        }

        // Try to parse the host as an IP address
        // Strip brackets from IPv6 (e.g., "[::1]" -> "::1")
        let clean_host = host.trim_start_matches('[').trim_end_matches(']');

        if let Ok(ip) = clean_host.parse::<IpAddr>() {
            // Check allow-list first (match against the clean host without brackets)
            if self.is_allowed(clean_host) {
                return Ok(());
            }

            if Self::is_blocked_ip(&ip) {
                return Err(SsrfViolation {
                    url: url.to_string(),
                    reason: format!("blocked: private/internal IP {}", ip),
                });
            }
        }
        // If the host is not a recognized IP and not "localhost", allow it
        // (public DNS names are fine)

        Ok(())
    }

    /// Extract the host portion from a URL string.
    fn extract_host(url: &str) -> Option<String> {
        // Remove scheme
        let after_scheme = if let Some(idx) = url.find("://") {
            &url[idx + 3..]
        } else {
            url
        };

        // Handle IPv6 addresses in brackets
        if after_scheme.starts_with('[') {
            // Find the closing bracket
            if let Some(end) = after_scheme.find(']') {
                // Return the bracketed content without brackets
                return Some(after_scheme[1..end].to_string());
            }
            return None;
        }

        // Find the end of the host (port, path, query, or end of string)
        let host_end = after_scheme
            .find(|c: char| c == ':' || c == '/' || c == '?' || c == '#')
            .unwrap_or(after_scheme.len());

        let host = &after_scheme[..host_end];
        if host.is_empty() {
            None
        } else {
            Some(host.to_string())
        }
    }

    /// Check if an IP address falls into any blocked range.
    fn is_blocked_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();

                // Loopback: 127.0.0.0/8
                if octets[0] == 127 {
                    return true;
                }
                // RFC 1918: 10.0.0.0/8
                if octets[0] == 10 {
                    return true;
                }
                // RFC 1918: 172.16.0.0/12 (172.16.x.x through 172.31.x.x)
                if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                    return true;
                }
                // RFC 1918: 192.168.0.0/16
                if octets[0] == 192 && octets[1] == 168 {
                    return true;
                }
                // Link-local: 169.254.0.0/16 (AWS/cloud metadata)
                if octets[0] == 169 && octets[1] == 254 {
                    return true;
                }

                false
            }
            IpAddr::V6(ipv6) => {
                // IPv6 loopback: ::1
                ipv6.is_loopback()
            }
        }
    }

    /// Check if a host is in the allow-list.
    fn is_allowed(&self, host: &str) -> bool {
        self.allowed_private_endpoints.iter().any(|allowed| allowed == host)
    }
}

// ---------------------------------------------------------------------------
// SecurityConfig
// ---------------------------------------------------------------------------

/// Workspace-level security configuration loaded from `agentix.yaml`.
///
/// ```yaml
/// security:
///   ssrf_protection: true
///   secret_encryption: true
///   audit_enabled: true
///   allowed_private_endpoints:
///     - "10.0.0.5"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable SSRF protection for outbound HTTP tool calls (default: true).
    #[serde(default = "default_true")]
    pub ssrf_protection: bool,
    /// Enable AES-256-GCM encryption for secrets at rest (default: true).
    #[serde(default = "default_true")]
    pub secret_encryption: bool,
    /// Enable audit trail logging (default: true).
    #[serde(default = "default_true")]
    pub audit_enabled: bool,
    /// Private IPs/CIDRs allowed despite SSRF protection (default: empty).
    #[serde(default)]
    pub allowed_private_endpoints: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            ssrf_protection: true,
            secret_encryption: true,
            audit_enabled: true,
            allowed_private_endpoints: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// CapabilityPolicy
// ---------------------------------------------------------------------------

/// Runtime capability enforcement policy for WASM tools.
///
/// Wraps the tool name with explicit allow/deny lists for capabilities.
/// Used by the WASM sandbox to enforce capability boundaries at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    /// The tool this policy applies to.
    pub tool_name: String,
    /// Capabilities explicitly granted to the tool.
    pub allowed_capabilities: Vec<String>,
    /// Capabilities explicitly denied for the tool.
    pub denied_capabilities: Vec<String>,
}
