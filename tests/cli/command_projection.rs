/// Proof Family 1 — Command Projection Tests
///
/// Each test verifies that a public command parses, runs, and respects its
/// invariants. These are integration tests against the compiled binary.
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// ── 1. status show ────────────────────────────────────────────────────────────

#[test]
fn test_status_parses_and_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["status", "show"]);
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]))
        .stdout(predicate::str::contains("cargo-cicd workspace status"));
}

// ── 2. target show ────────────────────────────────────────────────────────────

#[test]
fn test_target_show_parses_and_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["target", "show"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("target directory"));
}

// ── 3. target prune — INVARIANT: no destructive default ──────────────────────

#[test]
fn test_target_prune_dry_run_does_not_delete() {
    // Capture via std::process::Output so we can inspect stdout independently.
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["target", "prune"])
        .output()
        .unwrap();
    assert!(output.status.success(), "target prune must exit 0 in plan mode");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // INVARIANT: plan mode must succeed and must advertise --apply as the
    // execution gate.  No files are deleted without an explicit --apply flag.
    assert!(
        stdout.contains("suggest") || stdout.contains("--apply"),
        "prune output must mention suggest mode or --apply gate; got:\n{stdout}"
    );
    // INVARIANT: must NOT confirm that files were actually deleted.
    // Note: output may say "never deleted" (a safety note); that is fine.
    // We check for active-voice deletion confirmations only.
    assert!(
        !stdout.contains("Deleted") && !stdout.contains("Removed"),
        "prune in plan mode must not confirm deletions; got:\n{stdout}"
    );
}

// ── 4. test changed — emits a plan ───────────────────────────────────────────

#[test]
fn test_test_changed_emits_plan() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["test", "changed"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changed test plan"));
}

// ── 5. trybuild changed — INVARIANT: does not run all fixtures ────────────────

#[test]
fn test_trybuild_changed_does_not_run_all_fixtures() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["trybuild", "changed"]);
    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // INVARIANT: mode must be changed-only, not a full fixture sweep.
    assert!(
        stdout.contains("changed-only"),
        "trybuild changed must report changed-only mode; got:\n{stdout}"
    );
    // Must not announce a full-fixture count that implies all fixtures ran.
    assert!(
        !stdout.contains("624 fixtures"),
        "trybuild changed must not run all 624 fixtures; got:\n{stdout}"
    );
}

// ── 6. git status — shows branch state ───────────────────────────────────────

#[test]
fn test_git_status_shows_state() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["git", "status"]);
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]))
        .stdout(predicate::str::contains("git status"));
}

// ── 7. publish run — emits cicd.toml ─────────────────────────────────────────

#[test]
fn test_publish_emits_cicd_toml() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["publish", "run"]);
    cmd.current_dir(dir.path());
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]));
    // When the command succeeds, cicd.toml must exist.
    let output = cmd.output().unwrap();
    if output.status.success() {
        assert!(
            dir.path().join("cicd.toml").exists(),
            "publish run must write cicd.toml to the working directory"
        );
    }
}

// ── 8. workspace doctor — runs health checks ─────────────────────────────────

#[test]
fn test_workspace_doctor_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["workspace", "doctor"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("workspace doctor"));
}
