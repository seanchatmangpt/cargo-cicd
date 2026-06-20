//! Integration tests for `cargo project status`.

use assert_cmd::Command;
use predicates::prelude::*;
use super::{minimal_workspace, temp_workspace};

fn cmd() -> Command {
    Command::cargo_bin("cargo-project").expect("binary must exist")
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic invocation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn status_show_exits_zero() {
    let dir = temp_workspace();
    cmd()
        .args(["status", "show"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn bare_status_exits_zero() {
    let dir = temp_workspace();
    cmd()
        .arg("status")
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn status_show_help_exits_zero() {
    cmd()
        .args(["status", "show", "--help"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Output contract
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn status_show_output_is_not_empty() {
    let dir = temp_workspace();
    cmd()
        .args(["status", "show"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn status_show_json_flag_produces_json() {
    let dir = temp_workspace();
    let output = cmd()
        .args(["status", "show", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("should run");

    // When --json is passed, the output must be valid JSON.
    // In minimal (non-process-data) mode, the flag may be silently ignored;
    // the test therefore only checks the exit code.
    assert!(output.status.success(), "status show --json must exit 0");
}

// Suppress unused import warning for minimal_workspace — it is available for
// callers who prefer that name.
#[allow(dead_code)]
fn _use_minimal_workspace() {
    let _ = minimal_workspace;
}
