//! Security test suite: Sandbox escape prevention
//!
//! This module validates that seccomp profiles and capability dropping
//! prevent container escape attacks.

use aof_runtime::{CapabilityConfig, SeccompProfileManager};
use std::fs;

fn setup_test_profiles() -> String {
    // Use actual config/seccomp directory (relative to workspace root)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    workspace_root.join("config/seccomp").to_string_lossy().to_string()
}

#[test]
fn test_seccomp_blocks_ptrace() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/default.json", profiles_dir))
        .expect("Failed to read default.json");

    // Parse JSON and verify ptrace is in blocked list
    assert!(
        content.contains("ptrace"),
        "default.json must explicitly block ptrace syscall"
    );

    // Verify it's in the blocked syscalls section
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    let blocked_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ERRNO")
        .expect("Must have blocked syscalls section");

    let names = blocked_syscalls["names"].as_array().unwrap();
    let has_ptrace = names.iter().any(|n| n.as_str() == Some("ptrace"));

    assert!(has_ptrace, "ptrace must be in blocked syscalls list");
}

#[test]
fn test_seccomp_blocks_mount() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/default.json", profiles_dir))
        .expect("Failed to read default.json");

    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    let blocked_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ERRNO")
        .unwrap();

    let names = blocked_syscalls["names"].as_array().unwrap();
    let has_mount = names.iter().any(|n| n.as_str() == Some("mount"));

    assert!(has_mount, "mount must be in blocked syscalls list");
}

#[test]
fn test_seccomp_blocks_module_loading() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/default.json", profiles_dir))
        .expect("Failed to read default.json");

    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    let blocked_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ERRNO")
        .unwrap();

    let names = blocked_syscalls["names"].as_array().unwrap();
    let has_init_module = names.iter().any(|n| n.as_str() == Some("init_module"));
    let has_finit_module = names.iter().any(|n| n.as_str() == Some("finit_module"));

    assert!(
        has_init_module && has_finit_module,
        "init_module and finit_module must be blocked"
    );
}

#[test]
fn test_seccomp_blocks_namespace_manipulation() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/default.json", profiles_dir))
        .expect("Failed to read default.json");

    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    let blocked_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ERRNO")
        .unwrap();

    let names = blocked_syscalls["names"].as_array().unwrap();
    let has_setns = names.iter().any(|n| n.as_str() == Some("setns"));
    let has_unshare = names.iter().any(|n| n.as_str() == Some("unshare"));

    assert!(
        has_setns && has_unshare,
        "setns and unshare must be blocked"
    );
}

#[test]
fn test_seccomp_blocks_bpf() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/default.json", profiles_dir))
        .expect("Failed to read default.json");

    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    let blocked_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ERRNO")
        .unwrap();

    let names = blocked_syscalls["names"].as_array().unwrap();
    let has_bpf = names.iter().any(|n| n.as_str() == Some("bpf"));

    assert!(has_bpf, "bpf syscall must be blocked (eBPF escape vector)");
}

#[test]
fn test_capabilities_drop_all_default() {
    let config = CapabilityConfig::default();

    assert!(config.drop_all, "Default must drop all capabilities");
    assert_eq!(
        config.allowlist_count(),
        0,
        "Default must have empty allowlist"
    );

    let args = config.docker_cap_args();
    assert_eq!(args.len(), 1, "Must have exactly one argument");
    assert_eq!(args[0], "--cap-drop=ALL", "Must drop all capabilities");
}

#[test]
fn test_capability_allowlist_per_tool() {
    // kubectl gets no extra capabilities
    let kubectl_config = CapabilityConfig::for_tool("kubectl");
    assert!(kubectl_config.drop_all);
    assert_eq!(kubectl_config.allowlist_count(), 0);

    // nc gets CAP_NET_BIND_SERVICE for port binding below 1024
    let nc_config = CapabilityConfig::for_tool("nc");
    assert!(nc_config.drop_all);
    assert_eq!(nc_config.allowlist_count(), 1);
    assert!(nc_config.allows("CAP_NET_BIND_SERVICE"));

    let args = nc_config.docker_cap_args();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "--cap-drop=ALL");
    assert_eq!(args[1], "--cap-add=CAP_NET_BIND_SERVICE");
}

#[test]
fn test_readonly_profile_minimal_syscalls() {
    let profiles_dir = setup_test_profiles();
    let content = fs::read_to_string(format!("{}/readonly-profile.json", profiles_dir))
        .expect("Failed to read readonly-profile.json");

    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let syscalls = json["syscalls"].as_array().unwrap();

    // Count allowed syscalls in first syscall section (basic read operations)
    let allowed_syscalls = syscalls
        .iter()
        .find(|s| s["action"] == "SCMP_ACT_ALLOW")
        .unwrap();

    let names = allowed_syscalls["names"].as_array().unwrap();

    // readonly profile should allow minimal syscalls (~15 for basic read operations)
    assert!(
        names.len() <= 20,
        "readonly profile must allow minimal syscalls, found {}",
        names.len()
    );

    // Verify critical read syscalls are present
    let has_read = names.iter().any(|n| n.as_str() == Some("read"));
    let has_stat = names.iter().any(|n| n.as_str() == Some("stat"));

    // Check for open or openat in any syscall block
    let has_open_syscall = syscalls.iter().any(|s| {
        if let Some(names_array) = s["names"].as_array() {
            names_array.iter().any(|n| n.as_str() == Some("open") || n.as_str() == Some("openat"))
        } else {
            false
        }
    });

    assert!(has_read && has_stat && has_open_syscall, "readonly profile must allow read, (open or openat), stat");
}

#[test]
fn test_profile_selection_by_tool_name() {
    let profiles_dir = setup_test_profiles();
    let manager =
        SeccompProfileManager::new(&profiles_dir).expect("Failed to create profile manager");

    // kubectl maps to kubectl-profile
    let kubectl_profile = manager.profile_for_tool("kubectl");
    assert_eq!(kubectl_profile.name, "kubectl");
    assert!(kubectl_profile.path.to_string_lossy().contains("kubectl-profile.json"));

    // docker maps to docker-profile
    let docker_profile = manager.profile_for_tool("docker");
    assert_eq!(docker_profile.name, "docker");
    assert!(docker_profile.path.to_string_lossy().contains("docker-profile.json"));

    // unknown tool maps to default
    let unknown_profile = manager.profile_for_tool("unknown-tool");
    assert_eq!(unknown_profile.name, "default");
    assert!(unknown_profile.path.to_string_lossy().contains("default.json"));
}

#[test]
fn test_seccomp_overhead_estimate() {
    // This is a meta-test validating that seccomp profiles exist and are parseable.
    // Actual overhead measurement would require runtime benchmarking, which is
    // covered in performance tests.

    let profiles_dir = setup_test_profiles();
    let manager = SeccompProfileManager::new(&profiles_dir)
        .expect("Failed to create profile manager");

    // Verify all profiles are loadable
    let profiles = manager.profiles();
    assert!(
        profiles.len() >= 4,
        "Must have at least 4 profiles (default, kubectl, docker, readonly)"
    );

    // Verify each profile generates valid Docker arguments
    for tool in &["kubectl", "docker", "cat", "unknown-tool"] {
        let security_opt = manager.docker_security_opt(tool);
        assert!(
            security_opt.starts_with("seccomp="),
            "Security opt must start with 'seccomp=' for tool {}",
            tool
        );
    }

    // Note: Actual overhead testing would measure:
    // 1. Baseline: 1000 tool executions without seccomp
    // 2. With seccomp: 1000 tool executions with seccomp profile
    // 3. Assert: overhead < 5% (typically 1-3% for well-designed profiles)
}
