//! Lane L10 — Chicago subprocess tests for `cargo-cicd workspace {list,doctor,validate,sync}`.
//!
//! Real binary, real temp workspaces, assertions on real exit codes and stdout.
//! Fail-closed contract: `doctor`/`validate` FAIL verdicts exit non-zero;
//! WARN-class checks (missing rust-toolchain, missing cicd.toml) stay exit 0.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn workspace_cmd() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

/// Build a real two-member cargo workspace: root manifest + two crates with
/// real src/lib.rs files on disk.
fn make_workspace(root: &Path) {
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"alpha\", \"crates/beta\"]\n",
    )
    .unwrap();
    for (dir, name) in [("alpha", "alpha"), ("crates/beta", "beta")] {
        let member = root.join(dir);
        std::fs::create_dir_all(member.join("src")).unwrap();
        std::fs::write(
            member.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        std::fs::write(member.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    }
}

// ── workspace list ───────────────────────────────────────────────────────────

#[test]
fn workspace_list_prints_member_names_from_real_workspace() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());

    workspace_cmd()
        .args(["workspace", "list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("alpha")
                .and(predicate::str::contains("crates/beta"))
                .and(predicate::str::contains("total: 2 member(s)")),
        );
}

#[test]
fn workspace_list_on_single_crate_dir_reports_no_members() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"solo\"\n").unwrap();

    workspace_cmd()
        .args(["workspace", "list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no workspace members declared"));
}

// ── workspace doctor ─────────────────────────────────────────────────────────
// FAIL-class checks: missing Cargo.toml, missing .git, authority-script fraud
// scan (env-gated). WARN-class: missing rust-toolchain file, missing cicd.toml.

#[test]
fn workspace_doctor_wellformed_repo_exits_zero() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    workspace_cmd()
        .args(["workspace", "doctor"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("PASS")
                .and(predicate::str::contains("workspace is healthy")),
        );
}

#[test]
fn workspace_doctor_missing_toolchain_file_is_warn_not_fail() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    // No rust-toolchain.toml, no cicd.toml — WARN-class only.

    workspace_cmd()
        .args(["workspace", "doctor"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("WARN").and(predicate::str::contains("workspace is healthy")),
        );
}

#[test]
fn workspace_doctor_broken_repo_fails_closed_nonzero() {
    let tmp = TempDir::new().unwrap();
    // Empty dir: no Cargo.toml, no .git — both FAIL-class.

    workspace_cmd()
        .args(["workspace", "doctor"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL"));
}

#[test]
fn workspace_doctor_json_is_parseable_and_reports_fail_verdict() {
    let tmp = TempDir::new().unwrap();

    let output = workspace_cmd()
        .args(["workspace", "doctor", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "FAIL verdict must exit non-zero");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("--json output not parseable: {e}\n{stdout}"));
    assert_eq!(v["verdict"], "FAIL");
    let checks = v["checks"].as_array().unwrap();
    let cargo_check = checks
        .iter()
        .find(|c| c["name"] == "Cargo.toml")
        .expect("Cargo.toml check present");
    assert_eq!(cargo_check["status"], "FAIL");
}

#[test]
fn workspace_doctor_json_wellformed_is_parseable_pass() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    let output = workspace_cmd()
        .args(["workspace", "doctor", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse the first JSON document in the stream (the doctor report);
    // trailing framework/evidence lines are outside the verb's contract.
    let mut stream = serde_json::Deserializer::from_str(&stdout).into_iter::<serde_json::Value>();
    let v = stream
        .next()
        .unwrap_or_else(|| panic!("no JSON document in --json output:\n{stdout}"))
        .unwrap_or_else(|e| panic!("--json output not parseable: {e}\n{stdout}"));
    assert_eq!(v["verdict"], "PASS");
}

// ── workspace validate ───────────────────────────────────────────────────────

#[test]
fn workspace_validate_malformed_members_path_fails_closed() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"ghost-crate\"]\n",
    )
    .unwrap();

    workspace_cmd()
        .args(["workspace", "validate"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("FAIL").and(predicate::str::contains("validate verdict: FAIL")),
        );
}

#[test]
fn workspace_validate_wellformed_members_exits_zero() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());

    workspace_cmd()
        .args(["workspace", "validate"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("validate verdict: PASS"));
}

// ── workspace sync ───────────────────────────────────────────────────────────
// SKIP by design: the ggen.toml-present path spawns real `ggen sync`, which
// mutates the subject tree and depends on the ggen binary — out of scope for
// this lane. Only the no-ggen.toml skip path is exercised here (exit 0, real
// ggen never invoked).

#[test]
fn workspace_sync_without_ggen_toml_skips_and_exits_zero() {
    let tmp = TempDir::new().unwrap();
    make_workspace(tmp.path());

    workspace_cmd()
        .args(["workspace", "sync"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ggen.toml not found; no sync needed"));
}
