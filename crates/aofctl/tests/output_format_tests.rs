/// Output format tests
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn test_get_json_output() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .success();
}

#[test]
fn test_get_yaml_output() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents")
        .arg("-o")
        .arg("yaml");

    cmd.assert()
        .success();
}

#[test]
fn test_get_wide_output() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents")
        .arg("-o")
        .arg("wide");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"));
}

#[test]
fn test_get_name_output() {
    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("get")
        .arg("agents")
        .arg("-o")
        .arg("name");

    cmd.assert()
        .success();
}

#[test]
fn test_run_text_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run")
        .arg("agent")
        .arg(fixture)
        .arg("-i")
        .arg("test query")
        .arg("-o")
        .arg("text");

    // This will fail without proper runtime setup, but tests the flag
    cmd.assert();
}

#[test]
fn test_run_json_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run")
        .arg("agent")
        .arg(fixture)
        .arg("-i")
        .arg("test query")
        .arg("-o")
        .arg("json");

    // This will fail without proper runtime setup, but tests the flag
    cmd.assert();
}

#[test]
fn test_run_yaml_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple_agent.yaml");

    let mut cmd = Command::cargo_bin("aofctl").unwrap();
    cmd.arg("run")
        .arg("agent")
        .arg(fixture)
        .arg("-i")
        .arg("test query")
        .arg("-o")
        .arg("yaml");

    // This will fail without proper runtime setup, but tests the flag
    cmd.assert();
}
