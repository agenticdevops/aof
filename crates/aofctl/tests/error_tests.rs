/// Error handling tests
mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_run_missing_required_input() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run")
        .arg("agent")
        .arg("config.yaml");

    // Should fail - either missing input or file not found
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed").or(predicate::str::contains("required")));
}

#[test]
fn test_run_missing_resource_type() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_get_missing_resource_type() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_delete_missing_resource_name() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("delete")
        .arg("agent");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_apply_missing_file_flag() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("apply");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_validate_missing_file_flag() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_describe_missing_resource_type() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("describe");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_logs_missing_resource_name() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("logs")
        .arg("agent");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_file_not_found_error() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("validate")
        .arg("--file")
        .arg("nonexistent.yaml");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized"));
}
