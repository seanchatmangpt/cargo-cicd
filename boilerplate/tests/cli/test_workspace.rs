//! Integration tests for `cargo project workspace`.

use assert_cmd::Command;
use predicates::prelude::*;
use super::temp_workspace;

fn cmd() -> Command {
    Command::cargo_bin("cargo-project").expect("binary must exist")
}

#[test]
fn workspace_doctor_exits_zero() {
    let dir = temp_workspace();
    cmd()
        .args(["workspace", "doctor"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn bare_workspace_exits_zero() {
    let dir = temp_workspace();
    cmd()
        .arg("workspace")
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn workspace_doctor_output_not_empty() {
    let dir = temp_workspace();
    cmd()
        .args(["workspace", "doctor"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
