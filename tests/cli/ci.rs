//! ci noun — Chicago subprocess tests.
//!
//! Dry-run invariants: `ci run` in either mode must print its mode, the exact
//! command line, and a verdict, and must exit 0 without dispatching anything
//! when `--dry-run` is set. Non-dry-run paths that hit missing oracles are
//! exercised elsewhere; here we pin the dry-run contract.

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
fn ci_run_dry_run_local_exits_zero_with_mode_and_verdict() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["ci", "run", "--dry-run"]);
    assert!(
        out.status.success(),
        "ci run --dry-run must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("local"),
        "default mode is local and must be printed; got:\n{stdout}"
    );
    assert!(
        stdout.contains("verdict:"),
        "a verdict line is mandatory output; got:\n{stdout}"
    );
    assert!(
        stdout.contains("act --workflows .github/workflows/ci.yml"),
        "dry-run must print the exact command line it would run; got:\n{stdout}"
    );
}

#[test]
fn ci_run_dry_run_remote_exits_zero_and_does_not_dispatch() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(
        repo.path(),
        &["ci", "run", "--mode", "remote", "--dry-run"],
    );
    assert!(
        out.status.success(),
        "remote dry-run must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("remote"),
        "remote mode must be echoed; got:\n{stdout}"
    );
    assert!(
        stdout.contains("gh workflow run"),
        "remote dry-run must print the gh dispatch command; got:\n{stdout}"
    );
    assert!(
        stdout.contains("verdict:"),
        "a verdict line is mandatory output; got:\n{stdout}"
    );
    assert!(
        stdout.contains("yes"),
        "dry-run state (dry-run: yes) must be visible; got:\n{stdout}"
    );
}

#[test]
fn ci_run_dry_run_default_workflow_is_ci_yml() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["ci", "run", "--dry-run"]);
    assert!(out.status.success(), "must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ci.yml"),
        "workflow defaults to ci.yml and must be shown; got:\n{stdout}"
    );
}
