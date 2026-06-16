use assert_cmd::Command;
use std::process::Command as StdCmd;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    StdCmd::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .ok();
    StdCmd::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .ok();
    StdCmd::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .ok();
    // Disable commit signing so tests work in environments with GPG/SSH signing enforced.
    StdCmd::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(dir)
        .output()
        .ok();
    StdCmd::new("git")
        .args(["config", "tag.gpgsign", "false"])
        .current_dir(dir)
        .output()
        .ok();
}

#[test]
fn test_git_status_shows_clean_tree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("status")
        .output()
        .unwrap();
    // Should succeed and mention branch or status
    assert!(output.status.success(), "git status should not panic");
}

#[test]
fn test_git_close_dry_run_does_not_commit() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    // Create an untracked file
    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

    // git close without --dry-run: the CLI will refuse because the tree is dirty.
    // The binary doesn't expose a --dry-run flag, so we test the refusal behavior
    // directly — a dirty tree must be refused, leaving the file untracked.
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("close")
        .output()
        .unwrap();

    // After the command, the file must still be untracked (no silent commit).
    let git_status = StdCmd::new("git")
        .arg("status")
        .arg("--short")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let status_out = String::from_utf8_lossy(&git_status.stdout);
    assert!(
        status_out.contains("test.txt"),
        "git close must not commit the untracked file; status was: {}",
        status_out
    );
    // The command must have exited non-zero (refused).
    assert!(
        !output.status.success(),
        "git close on a dirty tree must exit non-zero; got: {:?}",
        output.status.code()
    );
}

#[test]
fn test_no_false_close_invariant_dirty_unrelated() {
    // INVARIANT: git close must not claim closed if unrelated dirty state remains
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    std::fs::write(dir.path().join("unrelated.rs"), "// untracked").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("close")
        .output()
        .unwrap();

    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    // Must NOT claim the phase is already closed or cleanly committed.
    let claims_closed = combined.contains("phase already closed")
        || combined.contains("phase closed")
        || combined.contains("committed");
    assert!(
        !claims_closed,
        "git close must not claim closed when unrelated dirty files remain; output: {}",
        combined
    );

    // Command must exit gracefully (with a process exit code, not a panic/signal).
    assert!(
        output.status.code().is_some(),
        "command should exit gracefully"
    );
}

#[test]
fn git_diff_exits_zero() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("diff")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git diff should exit zero; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn git_stage_exits_zero() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    // Create and commit an initial file so there are tracked files
    std::fs::write(dir.path().join("init.txt"), "initial").unwrap();
    StdCmd::new("git")
        .args(["add", "init.txt"])
        .current_dir(dir.path())
        .output()
        .ok();
    StdCmd::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .ok();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("stage")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git stage should exit zero on a clean tracked repo; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn git_fetch_exits_nonzero_without_remote() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    // No remote is configured, so fetch origin must fail
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("fetch")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "git fetch should exit non-zero when no remote is configured"
    );
}

#[test]
fn git_commit_warns_on_clean_tree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    // Create and commit an initial file to have a clean tracked tree
    std::fs::write(dir.path().join("init.txt"), "initial").unwrap();
    StdCmd::new("git")
        .args(["add", "init.txt"])
        .current_dir(dir.path())
        .output()
        .ok();
    StdCmd::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .ok();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("commit")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    // On a clean tree it should either succeed (exit 0) with a WARN message,
    // or exit 0 indicating nothing to commit.
    assert!(
        output.status.success(),
        "git commit on clean tree should exit zero (WARN, not FAIL); output: {}",
        combined
    );
    assert!(
        combined.contains("clean") || combined.contains("nothing") || combined.contains("WARN"),
        "expected clean-tree message; output: {}",
        combined
    );
}
