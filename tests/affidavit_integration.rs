//! Integration tests for the affidavit cryptographic-provenance integration.
//!
//! affidavit is integrated as an external `affi` binary (shell-out oracle), so
//! these tests cover the pure mapping helpers plus the graceful-degradation
//! path when `affi` is not installed — the realistic CI state, mirroring how the
//! wasm4pm evidence-gate tests treat a missing oracle (`Blocked`). When `affi`
//! *is* present they still pass, since both verbs exit 0 either way.
//!
//! Run with:
//!
//! ```sh
//! cargo test --features affidavit --test affidavit_integration
//! ```
#![cfg(feature = "affidavit")]

use assert_cmd::Command;
use cargo_cicd::evidence::ProcessEvent;
use cargo_cicd::integrations::affidavit_shell::{
    affidavit_receipt_dir, event_type_for, object_ref_for, AffidavitShell, AffidavitVerdict,
};
use std::path::Path;
use tempfile::TempDir;

// ── Pure mapping helpers (no binary required) ───────────────────────────────

#[test]
fn receipt_dir_is_under_evidence() {
    let p = affidavit_receipt_dir(Path::new("target/cargo-cicd/evidence"));
    assert!(p.ends_with("affidavit"), "receipt dir: {}", p.display());
}

#[test]
fn event_type_joins_command_and_lifecycle() {
    assert_eq!(
        event_type_for("status show", "complete"),
        "status:show:complete"
    );
    assert_eq!(event_type_for("publish run", ""), "publish:run");
    assert_eq!(event_type_for("", "start"), "event:start");
}

#[test]
fn object_ref_has_exactly_two_separators() {
    let mut ev = ProcessEvent::new("status show", "PASS");
    ev.workspace_id = "weird ws:name".to_string();
    let obj = object_ref_for(&ev);
    // `affi --object ID:TYPE:QUAL` parses on ':', so components must not add more.
    assert_eq!(obj.matches(':').count(), 2, "object ref: {obj}");
    assert!(obj.ends_with(":PASS"), "object ref: {obj}");
}

#[test]
fn verdict_display_is_stable() {
    assert_eq!(AffidavitVerdict::Accept.to_string(), "ACCEPT");
    assert_eq!(AffidavitVerdict::Reject.to_string(), "REJECT");
    assert_eq!(AffidavitVerdict::Blocked.to_string(), "BLOCKED");
}

#[test]
fn detect_never_panics() {
    // Returns None when `affi` is absent; if present, the version probe runs.
    if let Some(shell) = AffidavitShell::detect() {
        assert!(!shell.binary_path().is_empty());
        let _ = shell.version();
    }
}

// ── CLI smoke tests (graceful with or without `affi`) ───────────────────────

#[test]
fn cli_affidavit_seal_exits_zero() {
    let dir = TempDir::new().unwrap();
    let out = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["affidavit", "seal"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "seal should exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("affidavit seal"), "stdout: {stdout}");
}

#[test]
fn cli_affidavit_verify_exits_zero() {
    let dir = TempDir::new().unwrap();
    let out = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["affidavit", "verify"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "verify should exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("affidavit verify"), "stdout: {stdout}");
}

#[test]
fn cli_affidavit_help_lists_both_verbs() {
    let out = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["affidavit", "--help"])
        .output()
        .unwrap();
    // clap-noun-verb routes help through either stream; check both.
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("seal"), "help: {text}");
    assert!(text.contains("verify"), "help: {text}");
}
