//! pipeline noun — Chicago subprocess tests.
//!
//! `pipeline validate` and `pipeline status` are read-only over the working
//! directory (cicd.toml presence, evidence files) and must degrade gracefully
//! to WARN output with exit 0 in an empty workspace.

// NOTE: `pipeline run` is deliberately NOT exercised here — it executes the
// full pipeline (all declared activities against the real workspace) and is
// too heavy for a subprocess test. Covered by lane-level pipeline-run courts.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

fn run_cargo_cicd(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should build and run")
}

#[test]
fn pipeline_validate_empty_workspace_exits_zero_with_warns() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["pipeline", "validate"]);
    assert!(
        out.status.success(),
        "pipeline validate degrades to WARN, never a crash; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[WARN]"),
        "empty workspace must produce WARN checks; got:\n{stdout}"
    );
    assert!(
        stdout.contains("cicd.toml"),
        "the cicd.toml precondition must be named; got:\n{stdout}"
    );
}

#[test]
fn pipeline_status_empty_workspace_exits_zero_graceful() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["pipeline", "status"]);
    assert!(
        out.status.success(),
        "pipeline status in an empty workspace must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cicd.toml"),
        "status reports cicd.toml state (present or MISSING); got:\n{stdout}"
    );
}

#[test]
fn pipeline_validate_sees_present_preconditions_in_seeded_repo() {
    let repo = TempDir::new().expect("temp repo dir");
    std::fs::create_dir_all(repo.path().join(".git")).expect("seed .git dir");
    std::fs::write(repo.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
    std::fs::write(repo.path().join("cicd.toml"), "# seeded\n").unwrap();
    let out = run_cargo_cicd(repo.path(), &["pipeline", "validate"]);
    assert!(out.status.success(), "must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[PASS] Cargo.toml") && stdout.contains("[PASS] git initialized"),
        "seeded preconditions must PASS; got:\n{stdout}"
    );
}
