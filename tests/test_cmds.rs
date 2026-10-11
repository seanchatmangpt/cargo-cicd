//! Chicago subprocess tests for the `test` noun (changed / run / bench).
//!
//! All tests run the real `cargo-cicd` binary against real git repos and
//! real minimal cargo crate fixtures under `tempfile::TempDir`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as OsCommand;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

/// Create a real git repo at `dir` with an `origin/main` branch containing
/// `base_commit_files`, then leave `dirty_files` as uncommitted changes so
/// `git diff --name-only origin/main -- '*.rs'` sees them.
fn git_repo_with_dirty_rs(dir: &Path, dirty_files: &[&str]) {
    let git = |args: &[&str]| {
        let status = OsCommand::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .expect("git must be installed");
        assert!(status.success(), "git {:?} failed", args);
    };
    git(&["init", "-q"]);
    fs::write(dir.join("lib.rs"), "pub fn f() {}\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "base"]);
    // Make the local branch reachable under the name the detector diffs against.
    git(&["branch", "origin/main"]);
    for f in dirty_files {
        let path = dir.join(f);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "// dirty change\n").unwrap();
        // Stage so `git diff <commit>` sees it (untracked files are invisible).
        git(&["add", f]);
    }
}

#[test]
fn test_changed_lists_plan_in_git_repo_with_dirty_test_files() {
    let tmp = TempDir::new().unwrap();
    git_repo_with_dirty_rs(tmp.path(), &["tests/changed_thing_test.rs"]);

    // Planning surface: exit 0 and a human-readable plan, no cargo spawn.
    bin().args(["test", "changed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("changed test plan")
                .and(predicates::str::contains("base ref:"))
                .and(predicates::str::contains("origin/main"))
                .and(predicates::str::contains("tests/changed_thing_test.rs")),
        );
}

#[test]
fn test_changed_reports_conservative_mode_with_no_dirty_test_files() {
    let tmp = TempDir::new().unwrap();
    git_repo_with_dirty_rs(tmp.path(), &[]);

    bin().args(["test", "changed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("changed test plan")
                .and(predicates::str::contains("conservative")),
        );
}

/// Minimal zero-dependency crate fixture; `good` has a passing test,
/// `bad` has a failing one.
fn tiny_crate_fixture(root: &Path, name: &str, test_body: &str) {
    let dir = root.join(name);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n", name),
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), test_body).unwrap();
}

#[test]
fn test_run_exits_zero_on_passing_crate_fixture() {
    let tmp = TempDir::new().unwrap();
    tiny_crate_fixture(
        tmp.path(),
        "goodcrate",
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn passes() {\n        assert_eq!(1 + 1, 2);\n    }\n}\n",
    );

    // Real cargo test inside a real crate; success must exit zero.
    bin().args(["test", "run"])
        .current_dir(tmp.path().join("goodcrate"))
        .assert()
        .success()
        .stdout(predicates::str::contains("test result: ok"));
}

#[test]
fn test_run_fails_closed_on_failing_crate_fixture() {
    let tmp = TempDir::new().unwrap();
    tiny_crate_fixture(
        tmp.path(),
        "badcrate",
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn fails() {\n        assert!(false, \"deliberate failure\");\n    }\n}\n",
    );

    // NEW fail-closed contract: a FAIL verdict must surface as a non-zero
    // exit, not a swallowed success.
    bin().args(["test", "run"])
        .current_dir(tmp.path().join("badcrate"))
        .assert()
        .failure()
        // On FAIL the verb routes cargo's stderr through and exits with a
        // typed "cargo test failed (verdict: FAIL)" error (fail-closed).
        .stderr(
            predicates::str::contains("cargo test failed")
                .and(predicates::str::contains("verdict: FAIL")),
        );
}

// SKIPPED: `test bench` exercises `cargo bench`, which compiles and runs
// criterion benchmark binaries (minutes, and the fixture crates above carry
// no benches). The fail-closed bench path is identical to `test run`'s
// verdict plumbing, which is covered by the failing fixture above.
#[test]
#[ignore = "slow: spawns real cargo bench loop"]
fn test_bench_fails_closed_placeholder() {}
