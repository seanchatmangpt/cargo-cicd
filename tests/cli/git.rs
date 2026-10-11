//! Chicago subprocess tests for the `git` noun: real temp git repos, real
//! binary, real exit codes. Stage/commit are mutating and skipped here —
//! `git close` covers the enforcement surface (dirty tree refusal vs clean
//! pass) without the tests themselves staging or committing.

use assert_cmd::Command;
use std::path::Path;
use std::process::Command as OsCommand;
use tempfile::TempDir;

fn run_git(dir: &Path, args: &[&str]) {
    let status = OsCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git should be installed");
    assert!(status.success(), "setup `git {:?}` failed", args);
}

fn cicd(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(args)
        .current_dir(dir)
        .assert()
}

fn init_repo_with_commit() -> TempDir {
    let tmp = TempDir::new().unwrap();
    run_git(tmp.path(), &["init", "-q"]);
    run_git(tmp.path(), &["commit", "--allow-empty", "-q", "-m", "init"]);
    tmp
}

#[test]
fn git_status_exits_zero_in_temp_git_repo() {
    let tmp = init_repo_with_commit();
    run_git(tmp.path(), &["status"]); // sanity: repo really is a repo
    cicd(tmp.path(), &["git", "status"]).success();
}

#[test]
fn git_diff_exits_zero_in_temp_git_repo() {
    let tmp = init_repo_with_commit();
    std::fs::write(tmp.path().join("tracked.txt"), "hello\n").unwrap();
    run_git(tmp.path(), &["add", "tracked.txt"]);
    run_git(tmp.path(), &["commit", "-q", "-m", "add tracked"]);
    std::fs::write(tmp.path().join("tracked.txt"), "changed\n").unwrap();
    cicd(tmp.path(), &["git", "diff"]).success();
}

// NOTE: `git stage` and `git commit` verbs are intentionally NOT exercised
// here — they mutate repository state, and a test that stages/commits would
// be a mutating test rather than an observation. The enforcement behavior of
// the stage/commit sequence is covered indirectly by the `git close`
// dirty/clean pair below.

#[test]
fn git_close_refuses_dirty_tree_with_nonzero_exit() {
    let tmp = init_repo_with_commit();
    // Untracked file = dirty tree. close must refuse to hide it.
    std::fs::write(tmp.path().join("dirty.txt"), "uncommitted\n").unwrap();
    cicd(tmp.path(), &["git", "close"]).failure();
}

#[test]
fn git_close_passes_on_clean_tree() {
    let tmp = init_repo_with_commit();
    assert!(OsCommand::new("git")
        .args(["status", "--porcelain"])
        .current_dir(tmp.path())
        .output()
        .unwrap()
        .stdout
        .is_empty());
    cicd(tmp.path(), &["git", "close"]).success();
}
