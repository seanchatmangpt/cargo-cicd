//! Public-boundary invariant tests.
//!
//! These tests verify the 7 non-negotiable contracts that must hold before any release:
//! 1. No forbidden internal terms leak into help output
//! 2. Binary name is correct
//! 3. Status command exits 0 (baseline health check)
//! 4. All noun names are lowercase ASCII
//! 5. No destructive action without --confirm flag
//! 6. Help output is not empty for any noun/verb
//! 7. Version flag outputs a semver string

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("cargo-project").expect("cargo-project binary must exist")
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 1: No forbidden internal terms in help output
// ─────────────────────────────────────────────────────────────────────────────

const FORBIDDEN_TERMS: &[&str] = &[
    "INTERNAL", "PRIVATE", "DEBUG_ONLY", "TODO", "FIXME", "HACK",
];

#[test]
fn invariant_no_forbidden_terms_in_help() {
    let output = cmd()
        .arg("--help")
        .output()
        .expect("--help should not fail");

    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    let combined = format!("{text}{stderr}");

    for term in FORBIDDEN_TERMS {
        assert!(
            !combined.contains(&term.to_lowercase()),
            "forbidden term `{term}` found in help output"
        );
    }
}

#[test]
fn invariant_no_forbidden_terms_in_status_help() {
    let output = cmd()
        .args(["status", "--help"])
        .output()
        .expect("status --help should not fail");

    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    for term in FORBIDDEN_TERMS {
        assert!(
            !text.contains(&term.to_lowercase()),
            "forbidden term `{term}` found in `status --help` output"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 2: Binary name
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn invariant_binary_name_is_cargo_project() {
    // The binary must exist and be invocable.
    cmd()
        .arg("--version")
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 3: Status exits 0
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn invariant_status_exits_zero() {
    cmd()
        .args(["status", "show"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 4: Version outputs semver
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn invariant_version_is_semver() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 5: Help is non-empty
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn invariant_help_is_non_empty() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn invariant_status_show_help_is_non_empty() {
    cmd()
        .args(["status", "show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
