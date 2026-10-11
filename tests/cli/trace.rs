//! Chicago subprocess tests for `cargo-cicd trace profile`.
//! Real `just` runs in real temp dirs. Skipped (with a comment) if `just`
//! is not on PATH.

use assert_cmd::Command;
use predicates::prelude::predicate;
use predicates::prelude::PredicateBooleanExt;
use tempfile::TempDir;

fn just_on_path() -> bool {
    std::process::Command::new("just")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_repo() -> TempDir {
    TempDir::new().unwrap()
}

fn write_justfile(dir: &TempDir, body: &str) {
    std::fs::write(dir.path().join("justfile"), body).unwrap();
}

fn trace_profile(dir: &TempDir, profile: &str) -> Command {
    let mut c = Command::cargo_bin("cargo-cicd").unwrap();
    c.args(["trace", "profile", "--repo", dir.path().to_str().unwrap(), profile])
        .current_dir(dir.path());
    c
}

fn receipts_dir(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join(".cargo-cicd").join("receipts")
}

#[test]
fn trace_profile_passing_recipe_succeeds_and_writes_receipts() {
    if !just_on_path() {
        // SKIP: `just` is not installed on this machine; the trace noun
        // shells out to it unconditionally.
        eprintln!("skipping: just not on PATH");
        return;
    }
    let dir = temp_repo();
    write_justfile(&dir, "check:\n\t@echo ok\n");

    trace_profile(&dir, "check")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("cargo-cicd.trace.v1")
                .and(predicate::str::contains("\"check\""))
                .and(predicate::str::contains("receipt_path")),
        );

    let latest = receipts_dir(&dir).join("latest.json");
    let provenance = receipts_dir(&dir).join("trace-profile-check.json");
    assert!(latest.is_file(), "latest.json must be written");
    assert!(provenance.is_file(), "provenance receipt must be written");

    // The persisted latest.json must reflect the REAL observed exit code.
    let latest_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&latest).unwrap()).unwrap();
    assert_eq!(
        latest_json["algorithms"][0]["boundary_evidence"]["exit_code"].as_i64(),
        Some(0)
    );
}

#[test]
fn trace_profile_failing_recipe_is_fail_closed_but_persists_evidence() {
    if !just_on_path() {
        // SKIP: `just` is not installed on this machine.
        eprintln!("skipping: just not on PATH");
        return;
    }
    let dir = temp_repo();
    write_justfile(&dir, "check:\n\t@exit 1\n");

    trace_profile(&dir, "check")
        .assert()
        .failure();

    // Fail-closed contract: the command exits non-zero, but the evidence
    // receipts were persisted BEFORE the error, so the failed run is
    // replayable/adjudicable.
    let latest = receipts_dir(&dir).join("latest.json");
    let provenance = receipts_dir(&dir).join("trace-profile-check.json");
    assert!(latest.is_file(), "latest.json must persist even on failure");
    assert!(provenance.is_file(), "provenance receipt must persist on failure");
    let latest_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&latest).unwrap()).unwrap();
    assert_eq!(
        latest_json["algorithms"][0]["boundary_evidence"]["exit_code"].as_i64(),
        Some(1),
        "receipt must record the real failing exit code, not 0"
    );
}

#[test]
fn trace_profile_unknown_profile_fails_with_message() {
    let dir = temp_repo();
    let output = trace_profile(&dir, "bogus").output().unwrap();
    assert!(!output.status.success(), "bogus profile must exit non-zero");
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("Unknown profile"),
        "expected 'Unknown profile' in output: {}",
        text
    );
}
