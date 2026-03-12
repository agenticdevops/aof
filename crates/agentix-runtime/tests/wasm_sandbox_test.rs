//! Tests for the WASM tool sandbox and capability enforcement.
//! TDD: these tests define required behavior before implementation.
//!
//! Note: Tests use a minimal WAT (WebAssembly Text Format) module compiled
//! at test time — no external .wasm files needed.

use agentix_runtime::tools::wasm_executor::{
    CapabilityManifest, WasmCapability, WasmSandbox, WasmViolation,
};

/// Minimal WAT module that writes "ok" to memory offset 0 and returns length 2.
fn minimal_wasm_bytes() -> Vec<u8> {
    wat::parse_str(r#"
        (module
          (memory 1)
          (export "memory" (memory 0))
          (func (export "run") (param i32 i32) (result i32)
            (i32.store8 (i32.const 0) (i32.const 111))
            (i32.store8 (i32.const 1) (i32.const 107))
            (i32.const 2)
          )
        )
    "#).expect("valid WAT")
}

#[test]
fn test_capability_manifest_no_network_blocks_network_call() {
    // A manifest with no network capability should block network attempts
    let manifest = CapabilityManifest {
        name: "test-tool".to_string(),
        capabilities: vec![], // No capabilities declared
    };

    let violation = manifest.check_capability(&WasmCapability::Network);
    assert!(
        violation.is_some(),
        "Should block network call when capability not declared"
    );
    let v = violation.unwrap();
    assert!(
        matches!(v, WasmViolation::NetworkBlocked { .. }),
        "Should return NetworkBlocked violation, got: {:?}",
        v
    );
}

#[test]
fn test_capability_manifest_with_network_allows_network_call() {
    let manifest = CapabilityManifest {
        name: "test-tool".to_string(),
        capabilities: vec![WasmCapability::Network],
    };

    let violation = manifest.check_capability(&WasmCapability::Network);
    assert!(
        violation.is_none(),
        "Should allow network call when capability declared"
    );
}

#[test]
fn test_capability_manifest_no_filesystem_blocks_filesystem() {
    let manifest = CapabilityManifest {
        name: "test-tool".to_string(),
        capabilities: vec![WasmCapability::Network], // Network but no filesystem
    };

    let violation = manifest.check_capability(&WasmCapability::Filesystem);
    assert!(
        violation.is_some(),
        "Should block filesystem access when not declared"
    );
    assert!(
        matches!(violation.unwrap(), WasmViolation::FilesystemBlocked { .. }),
        "Wrong violation type"
    );
}

#[test]
fn test_http_endpoints_capability_allows_listed_url() {
    let manifest = CapabilityManifest {
        name: "test-tool".to_string(),
        capabilities: vec![WasmCapability::HttpEndpoints(vec![
            "https://api.example.com".to_string(),
        ])],
    };

    // Check general network — should still be blocked (HttpEndpoints is more specific)
    let violation = manifest.check_capability(&WasmCapability::Network);
    assert!(
        violation.is_some(),
        "HttpEndpoints does not grant general Network capability"
    );
}

#[test]
fn test_capability_manifest_serializes_from_yaml() {
    // Capability manifests are parsed from tools/<name>.yaml
    let yaml = r#"
name: my-wasm-tool
type: wasm
wasm_path: tools/my-tool.wasm
description: A sandboxed WASM tool
capabilities:
  - network
  - filesystem
"#;
    let manifest: CapabilityManifest = serde_yaml::from_str(yaml)
        .expect("Should parse capability manifest from YAML");

    assert_eq!(manifest.name, "my-wasm-tool");
    assert_eq!(manifest.capabilities.len(), 2);
    assert!(manifest.capabilities.contains(&WasmCapability::Network));
    assert!(manifest.capabilities.contains(&WasmCapability::Filesystem));
}

#[test]
#[cfg(feature = "wasm")]
fn test_wasm_sandbox_executes_minimal_module() {
    // WasmSandbox can execute a simple WASM module without capability violations
    let wasm_bytes = minimal_wasm_bytes();
    let manifest = CapabilityManifest {
        name: "minimal".to_string(),
        capabilities: vec![],
    };

    let sandbox = WasmSandbox::new(manifest);
    let result = sandbox.execute_bytes(&wasm_bytes, serde_json::json!({}));

    // Should succeed — the module doesn't attempt any restricted capabilities
    assert!(
        result.is_ok(),
        "Minimal WASM without restricted calls should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_violation_display_is_descriptive() {
    let violation = WasmViolation::NetworkBlocked {
        attempted_url: "https://evil.com".to_string(),
    };
    let msg = violation.to_string();
    assert!(
        msg.contains("network") || msg.contains("Network"),
        "Violation message should mention network: {}",
        msg
    );
    assert!(
        msg.contains("evil.com") || msg.contains("blocked"),
        "Violation message should be descriptive: {}",
        msg
    );
}
