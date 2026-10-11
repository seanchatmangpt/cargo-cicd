//! Chicago subprocess tests for the `trybuild` noun (review / changed).
//!
//! `full` and `update` are skipped: `update` runs the whole workspace test
//! suite under TRYBUILD=overwrite and `full` enumerates a compile-fail
//! estate — both are real cargo compile loops, too heavy for this lean
//! suite; the read-only planner surfaces (review/changed) are covered for
//! real instead.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as OsCommand;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

fn git_repo_with_dirty(dir: &Path, dirty_files: &[&str]) {
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
    git(&["branch", "origin/main"]);
    for f in dirty_files {
        let path = dir.join(f);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "// dirty trybuild fixture\n").unwrap();
        // Stage so `git diff <commit>` sees it (untracked files are invisible).
        git(&["add", f]);
    }
}

#[test]
fn trybuild_review_lists_seeded_fixtures_with_total_count() {
    let tmp = TempDir::new().unwrap();
    let tests = tmp.path().join("tests");
    fs::create_dir_all(tests.join("ui")).unwrap();
    // Two top-level files mention trybuild (counted); a ui/ subdir file is
    // invisible to the non-recursive read_dir and a marker-free file is
    // filtered by content — neither may appear.
    fs::write(tests.join("trybuild_suite.rs"), "// uses trybuild\n#[test]\nfn t() {}\n").unwrap();
    fs::write(tests.join("other_trybuild.rs"), "// trybuild fixture\nfn main() {}\n").unwrap();
    fs::write(tests.join("ui/compile_fail_bad.rs"), "// trybuild compile_fail\nfn main() {}\n").unwrap();
    fs::write(tests.join("plain_test.rs"), "// no marker here\n#[test]\nfn t() {}\n").unwrap();

    bin().args(["trybuild", "review"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("trybuild fixture review")
                .and(predicates::str::contains("total: 2 fixture file(s)"))
                .and(predicates::str::contains("tests/trybuild_suite.rs")),
        )
        .stdout(predicates::str::contains("ui/compile_fail_bad.rs").not());
}

#[test]
fn trybuild_review_with_no_fixtures_reports_empty() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("tests")).unwrap();

    bin().args(["trybuild", "review"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("no trybuild fixture files found"));
}

#[test]
fn trybuild_changed_plans_dirty_ui_fixtures_in_git_repo() {
    let tmp = TempDir::new().unwrap();
    // The fixture detector requires the literal "/tests/" segment in the
    // diff path, so seed the change under a crate subdirectory.
    git_repo_with_dirty(
        tmp.path(),
        &["crate/tests/ui/compile_fail/dirty_fixture.rs"],
    );

    // Planner surface: exit 0 with the changed-fixture plan (no cargo).
    bin().args(["trybuild", "changed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("trybuild changed plan")
                .and(predicates::str::contains("origin/main"))
                .and(predicates::str::contains(
                    "crate/tests/ui/compile_fail/dirty_fixture.rs",
                )),
        );
}

#[test]
fn trybuild_changed_with_no_dirty_fixtures_skips_run() {
    let tmp = TempDir::new().unwrap();
    git_repo_with_dirty(tmp.path(), &[]);

    bin().args(["trybuild", "changed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("trybuild changed plan")
                .and(predicates::str::contains(
                    "no changed trybuild fixtures detected",
                )),
        );
}

// SKIPPED: `trybuild update` runs the full workspace test suite with
// TRYBUILD=overwrite and `trybuild full` plans a whole-estate compile-fail
// run — both are real cargo compile loops (minutes) and would contend the
// shared cargo lock for every lane. The read-only planner verbs above
// cover the noun's subprocess contract; the fail-closed verdict plumbing
// they share with `test run` is covered in tests/cli/test.rs.
#[test]
#[ignore = "slow: spawns real cargo compile loops (update/full)"]
fn trybuild_update_full_placeholder() {}
