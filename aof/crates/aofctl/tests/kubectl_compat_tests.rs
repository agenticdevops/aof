/// kubectl compatibility tests
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn test_api_resources_command() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("api-resources");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"));
}

#[test]
fn test_describe_command() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("describe")
        .arg("agent")
        .arg("test-agent");

    cmd.assert()
        .success();
}

#[test]
fn test_logs_command() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("logs")
        .arg("agent")
        .arg("test-agent");

    cmd.assert()
        .success();
}

#[test]
fn test_exec_command() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("exec")
        .arg("agent")
        .arg("test-agent")
        .arg("--")
        .arg("echo")
        .arg("test");

    cmd.assert()
        .success();
}

#[test]
fn test_get_all_namespaces() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents")
        .arg("--all-namespaces");

    cmd.assert()
        .success();
}

#[test]
fn test_apply_with_file_flag() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("apply")
        .arg("-f")
        .arg(fixture);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration validated"));
}

#[test]
fn test_delete_verb_noun_pattern() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("delete")
        .arg("agent")
        .arg("test-agent");

    cmd.assert()
        .success();
}

#[test]
fn test_get_resource_type_plural() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"));
}

#[test]
fn test_get_resource_type_singular() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agent")
        .arg("my-agent");

    cmd.assert()
        .success();
}

#[test]
fn test_version_short_flag() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("-V");

    cmd.assert()
        .success();
}

#[test]
fn test_help_short_flag() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("-h");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
