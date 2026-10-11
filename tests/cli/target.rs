//! Chicago subprocess tests for `cargo-cicd target` (show / prune).

use assert_cmd::Command;
use predicates::prelude::predicate;

fn bin() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

#[test]
fn target_show_exits_zero_with_state_report() {
    bin()
        .args(["target", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target directory state"));
}

#[test]
fn target_prune_dry_run_is_a_plan_not_a_delete() {
    bin()
        .args(["target", "prune"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target prune plan"));
}

// SKIP: `target prune --apply` physically deletes the shared checkout's real
// target/ directory. Running it in-process would destroy the build cache for
// every other lane on this tree; the binary has no --repo override for target
// commands (cmd_prune takes no repo arg), so there is no safe subprocess
// isolation available. Dry-run plan coverage above is the safe surface.
#[test]
#[ignore = "destructive: --apply deletes the real target/ of the shared checkout; no repo isolation for the target noun"]
fn target_prune_apply() {
    bin().args(["target", "prune", "--apply"]).assert().success();
}
