//! publish noun — Chicago subprocess tests.
//!
//! New fail-closed contracts under test:
//! - `publish run` without `--dry-run` refuses with the typed
//!   REFUSED:explicit_dry_run_required error and a non-zero exit.
//! - `publish run --dry-run` hard-refuses when the wasm4pm oracle is
//!   unavailable (no receipt adjudication possible) and never attempts a
//!   publish.
//! - `publish check` / `publish validate` are read-only and exit 0.

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
fn publish_run_without_dry_run_refuses_typed_nonzero() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["publish", "run"]);
    assert!(
        !out.status.success(),
        "publish run without --dry-run must refuse with non-zero exit; got {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("REFUSED:explicit_dry_run_required"),
        "typed refusal REFUSED:explicit_dry_run_required expected; got:\n{combined}"
    );
    assert!(
        combined.contains("--dry-run"),
        "refusal must tell the operator which flag is required; got:\n{combined}"
    );
}

#[test]
fn publish_run_dry_run_hard_refuses_when_oracle_unavailable() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["publish", "run", "--dry-run"]);
    assert!(
        !out.status.success(),
        "publish run --dry-run without the wpm oracle must hard-refuse non-zero; got {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("REFUSED:oracle_unavailable"),
        "typed refusal REFUSED:oracle_unavailable expected; got:\n{combined}"
    );
    // No publish may be attempted: the final cargo-publish gate never ran.
    assert!(
        !combined.contains("dry-run: PASS"),
        "a hard-refused publish run must not claim a passing dry-run; got:\n{combined}"
    );
}

#[test]
fn publish_check_is_read_only_and_exits_zero() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["publish", "check"]);
    assert!(
        out.status.success(),
        "publish check is read-only and must exit 0; got {:?}",
        out.status.code()
    );
}

#[test]
fn publish_validate_is_read_only_and_exits_zero() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["publish", "validate"]);
    assert!(
        out.status.success(),
        "publish validate is read-only and must exit 0; got {:?}",
        out.status.code()
    );
}
