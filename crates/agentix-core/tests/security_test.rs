use agentix_core::{SsrfGuard, SsrfViolation, SecurityConfig, CapabilityPolicy};

#[test]
fn test_ssrf_blocks_rfc1918_10x() {
    let guard = SsrfGuard::default();
    let result = guard.check_url("http://10.0.0.1:8080/api");
    assert!(result.is_err());
    let violation = result.unwrap_err();
    let msg = violation.to_string().to_lowercase();
    assert!(msg.contains("private") || msg.contains("blocked"), "Expected 'private' or 'blocked' in: {}", msg);
}

#[test]
fn test_ssrf_blocks_rfc1918_172x() {
    let guard = SsrfGuard::default();
    assert!(guard.check_url("http://172.16.0.1/").is_err());
    assert!(guard.check_url("http://172.31.255.255/").is_err());
    // 172.32.x.x is outside the private range
    assert!(guard.check_url("http://172.32.0.1/").is_ok());
}

#[test]
fn test_ssrf_blocks_rfc1918_192_168() {
    let guard = SsrfGuard::default();
    assert!(guard.check_url("http://192.168.1.1/").is_err());
}

#[test]
fn test_ssrf_blocks_localhost() {
    let guard = SsrfGuard::default();
    assert!(guard.check_url("http://127.0.0.1/").is_err());
    assert!(guard.check_url("http://localhost:3000/").is_err());
    assert!(guard.check_url("http://[::1]/").is_err());
}

#[test]
fn test_ssrf_blocks_cloud_metadata() {
    let guard = SsrfGuard::default();
    assert!(guard.check_url("http://169.254.169.254/latest/meta-data/").is_err());
    assert!(guard.check_url("http://169.254.0.1/").is_err());
}

#[test]
fn test_ssrf_allows_public_urls() {
    let guard = SsrfGuard::default();
    assert!(guard.check_url("https://api.openai.com/v1/chat").is_ok());
    assert!(guard.check_url("https://hooks.slack.com/services/T01/B02/xxx").is_ok());
}

#[test]
fn test_ssrf_allow_list_overrides() {
    let guard = SsrfGuard::with_allowed(vec!["10.0.0.5".to_string()]);
    // Allowed endpoint should pass
    assert!(guard.check_url("http://10.0.0.5:9090/metrics").is_ok());
    // Other private IPs still blocked
    assert!(guard.check_url("http://10.0.0.6:9090/").is_err());
}

#[test]
fn test_security_config_defaults() {
    let config = SecurityConfig::default();
    assert!(config.ssrf_protection);
    assert!(config.secret_encryption);
    assert!(config.audit_enabled);
    assert!(config.allowed_private_endpoints.is_empty());
}

#[test]
fn test_security_config_serde_round_trip() {
    let config = SecurityConfig {
        ssrf_protection: true,
        secret_encryption: false,
        audit_enabled: true,
        allowed_private_endpoints: vec!["10.0.0.5".to_string(), "172.16.1.10".to_string()],
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let deserialized: SecurityConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.ssrf_protection, config.ssrf_protection);
    assert_eq!(deserialized.secret_encryption, config.secret_encryption);
    assert_eq!(deserialized.audit_enabled, config.audit_enabled);
    assert_eq!(deserialized.allowed_private_endpoints, config.allowed_private_endpoints);
}

#[test]
fn test_ssrf_violation_display() {
    let violation = SsrfViolation {
        url: "http://10.0.0.1/api".to_string(),
        reason: "private IP range".to_string(),
    };
    let display = format!("{}", violation);
    assert!(display.contains("10.0.0.1"), "Display should contain the URL: {}", display);
    assert!(display.contains("private IP range"), "Display should contain the reason: {}", display);
}
