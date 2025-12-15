/// CLI integration tests
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("version");
    cmd.assert().success();
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CLI for AOF framework"));
}

#[test]
fn test_run_command_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Run an agent with configuration"));
}

#[test]
fn test_get_command_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get resources"));
}

#[test]
fn test_apply_command_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("apply").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Apply configuration"));
}

#[test]
fn test_delete_command_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("delete").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete resource"));
}

#[test]
fn test_validate_command_help() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validate agent configuration"));
}

#[test]
fn test_run_missing_config() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run")
        .arg("--config")
        .arg("nonexistent.yaml")
        .arg("--input")
        .arg("test");

    cmd.assert().failure();
}

#[test]
fn test_validate_simple_agent() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate").arg("--file").arg(fixture);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_validate_agent_with_tools() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/agent_with_tools.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate").arg("--file").arg(fixture);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"))
        .stdout(predicate::str::contains("agent-with-tools"));
}

#[test]
fn test_validate_invalid_config() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/invalid_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate").arg("--file").arg(fixture);

    // Should fail validation
    cmd.assert().failure();
}

#[test]
fn test_apply_valid_config() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("apply").arg("--file").arg(fixture);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration validated"));
}

#[test]
fn test_get_agents() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get").arg("agents");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"));
}

#[test]
fn test_get_specific_agent() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get").arg("agent").arg("my-agent");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"));
}

#[test]
fn test_delete_agent() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("delete").arg("agent").arg("test-agent");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Not yet implemented"));
}
