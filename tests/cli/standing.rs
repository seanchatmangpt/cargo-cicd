//! Chicago subprocess tests for `cargo-cicd standing` (refresh/verify/report).
//! Real temp git repos, real binary, assertions on real files and exit codes.

use assert_cmd::Command;
use predicates::prelude::predicate;
use predicates::prelude::PredicateBooleanExt;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

/// Fresh temp git repo to run against.
fn temp_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let status = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .expect("git available");
    assert!(status.success());
    dir
}

fn standing_json(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("target").join("praxis-standing").join("standing.json")
}

#[test]
fn refresh_creates_standing_json_and_receipt_on_success() {
    let dir = temp_repo();
    bin()
        .args(["standing", "refresh", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(standing_json(&dir).is_file(), "standing.json must exist");
    let latest = dir.path().join(".cargo-cicd").join("receipts").join("latest.json");
    assert!(latest.is_file(), "latest.json receipt must exist on success");
    // Non-json mode prints the human summary.
    // (exit already asserted success above)
}

#[test]
fn refresh_json_reports_receipt_path() {
    let dir = temp_repo();
    bin()
        .args([
            "standing",
            "refresh",
            "--repo",
            dir.path().to_str().unwrap(),
            "--json",
        ])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("cargo-cicd.standing.refresh.v1")
                .and(predicate::str::contains("receipt")),
        );
}

#[test]
fn verify_after_refresh_has_zero_drift_and_exits_zero() {
    let dir = temp_repo();
    bin()
        .args(["standing", "refresh", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success();

    bin()
        .args(["standing", "verify", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn verify_after_mutating_standing_json_fails_with_drift() {
    let dir = temp_repo();
    bin()
        .args(["standing", "refresh", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success();

    // Mutate the persisted document: drop the first artifact so verify
    // re-ingests it as "added" drift.
    let path = standing_json(&dir);
    let text = std::fs::read_to_string(&path).unwrap();
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let artifacts = doc["artifacts"].as_array_mut().unwrap();
    assert!(!artifacts.is_empty(), "refresh must persist artifacts");
    artifacts.remove(0);
    std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();

    bin()
        .args(["standing", "verify", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("drifted artifact(s)").or(predicate::str::contains("drift_count")));
}

#[test]
fn report_before_refresh_fails_with_hint() {
    let dir = temp_repo();
    let output = bin()
        .args(["standing", "report", "--repo", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "report before refresh must fail");
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("run `standing refresh` first") || text.contains("no standing.json"),
        "expected missing-standing hint in output: {}",
        text
    );
}
