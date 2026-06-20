//! CLI integration tests — noun/verb command grammar.
//!
//! Pattern: each test uses `assert_cmd::Command::cargo_bin("cargo-project")`
//! against a temp workspace so tests are hermetic.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

pub mod test_status;
pub mod test_workspace;

fn cmd() -> Command {
    Command::cargo_bin("cargo-project").expect("binary must exist")
}

/// Create a minimal Cargo workspace in a temp dir.
pub fn minimal_workspace() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "test-workspace"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), b"").unwrap();
    dir
}

/// Alias for [`minimal_workspace`] — used by submodule tests.
pub fn temp_workspace() -> TempDir {
    minimal_workspace()
}

// ─────────────────────────────────────────────────────────────────────────────
// status noun
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn status_show_exits_zero() {
    let ws = minimal_workspace();
    cmd()
        .args(["status", "show"])
        .current_dir(ws.path())
        .assert()
        .success();
}

#[test]
fn status_bare_noun_injects_show() {
    // `status` with no verb should behave identically to `status show`.
    let ws = minimal_workspace();
    cmd()
        .arg("status")
        .current_dir(ws.path())
        .assert()
        .success();
}

#[test]
fn status_show_stdout_not_empty() {
    let ws = minimal_workspace();
    cmd()
        .args(["status", "show"])
        .current_dir(ws.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ─────────────────────────────────────────────────────────────────────────────
// workspace noun
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn workspace_doctor_exits_zero() {
    let ws = minimal_workspace();
    cmd()
        .args(["workspace", "doctor"])
        .current_dir(ws.path())
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Unknown noun/verb error handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn unknown_noun_exits_nonzero() {
    cmd()
        .arg("frobnicate")
        .assert()
        .failure();
}

#[test]
fn unknown_verb_exits_nonzero() {
    let ws = minimal_workspace();
    cmd()
        .args(["status", "frobnicate"])
        .current_dir(ws.path())
        .assert()
        .failure();
}

// ─────────────────────────────────────────────────────────────────────────────
// Help flags
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn top_level_help_succeeds() {
    cmd().arg("--help").assert().success();
}

#[test]
fn status_show_help_succeeds() {
    cmd().args(["status", "show", "--help"]).assert().success();
}
