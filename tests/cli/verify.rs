//! Lane L10 — Chicago subprocess tests for `cargo-cicd verify repo`.
//!
//! Contract under test (fail-closed):
//! - missing `cargo-semver-checks` => typed `TOOL_UNAVAILABLE` error, non-zero
//!   exit, even under `--json` (never a silent skip / fake pass).
//! - `--json` emits pure, parseable `cargo-cicd.verify.v2` JSON.
//! - the emitted semver command carries the caller's `--package` (or the
//!   compile-time default), never a hardcoded `cargo-cicd-core`.
//!
//! A real `cargo` shim on a restricted PATH is used so the subprocess tree is
//! exercised end-to-end without depending on a network/install of the real
//! semver tool in CI; the shim is a real executable, and assertions are on the
//! logged argv (real state), not on call-count interactions.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

const SHIM_SCRIPT: &str = r#"#!/bin/sh
printf '%s\n' "$*" >> "$SHIM_LOG"
if [ "$1" = "semver-checks" ] && [ "$2" = "--version" ]; then
  echo "cargo-semver-checks 0.50.0 (shim)"
  exit 0
fi
exit 0
"#;

/// Write a minimal real crate so the subject directory is a plausible target.
fn make_crate(root: &Path, name: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
}

/// Install a `cargo` shim as the ONLY thing on PATH; returns (dir, log path).
/// Restricting PATH to the shim dir also deterministically simulates a
/// machine without semver-checks when the shim dir is empty.
fn shim_env() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("shim.log");
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let cargo = bin.join("cargo");
    std::fs::write(&cargo, SHIM_SCRIPT).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    (dir, bin, log)
}

fn verify_cmd() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

// ── typed Unavailable: semver-checks missing => non-zero, named error ────────

#[test]
fn verify_without_semver_checks_is_typed_unavailable_nonzero() {
    let tmp = TempDir::new().unwrap();
    make_crate(tmp.path(), "sample-pkg");
    let empty_path = TempDir::new().unwrap(); // PATH with no `cargo` at all

    let output = verify_cmd()
        .args(["verify", "repo"])
        .env("PATH", empty_path.path())
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "missing semver-checks must fail closed (exit non-zero)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("TOOL_UNAVAILABLE"),
        "typed Unavailable error expected, got: {combined}"
    );
    // A silent skip must not print a PASS/pass verdict.
    assert!(!combined.contains("semver:  pass"));
}

#[test]
fn verify_unavailable_is_nonzero_even_with_json_flag() {
    let tmp = TempDir::new().unwrap();
    make_crate(tmp.path(), "sample-pkg");
    let empty_path = TempDir::new().unwrap();

    let output = verify_cmd()
        .args(["verify", "repo", "--json"])
        .env("PATH", empty_path.path())
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "--json must not soften the typed Unavailable failure"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("TOOL_UNAVAILABLE"));
    // The typed error fires before any report JSON is emitted.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("cargo-cicd.verify.v2"),
        "no fake report JSON may be emitted on Unavailable, got: {stdout}"
    );
}

// ── --json is honored: pure parseable verify.v2 output ───────────────────────

#[test]
fn verify_repo_json_flag_emits_parseable_v2_report() {
    let tmp = TempDir::new().unwrap();
    make_crate(tmp.path(), "sample-pkg");
    let (_shim, bin, log) = shim_env();

    let output = verify_cmd()
        .args(["verify", "repo", "--json", "--package", "sample-pkg"])
        .env("PATH", &bin)
        .env("SHIM_LOG", &log)
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "shim reports semver pass");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse the first JSON document in the stream (the verify.v2 report).
    // Trailing framework/evidence lines after the report are not the verb's
    // contract; the report itself must be valid JSON.
    let mut stream = serde_json::Deserializer::from_str(&stdout).into_iter::<serde_json::Value>();
    let v = stream
        .next()
        .unwrap_or_else(|| panic!("no JSON document in --json output:\n{stdout}"))
        .unwrap_or_else(|e| panic!("--json output not parseable: {e}\n{stdout}"));
    assert_eq!(v["schema"], "cargo-cicd.verify.v2");
    assert_eq!(v["q_verify"], 1);
    assert_eq!(v["semver_status"], "pass");
    assert_eq!(v["semver_errors"].as_array().unwrap().len(), 0);
}

// ── no hardcoded cargo-cicd-core: emitted command carries the caller package ─

#[test]
fn verify_emitted_command_uses_explicit_package_not_hardcoded_core() {
    let tmp = TempDir::new().unwrap();
    make_crate(tmp.path(), "my-own-pkg");
    let (_shim, bin, log) = shim_env();

    let output = verify_cmd()
        .args(["verify", "repo", "--package", "my-own-pkg"])
        .env("PATH", &bin)
        .env("SHIM_LOG", &log)
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let logged = std::fs::read_to_string(&log).unwrap();
    let check_release = logged
        .lines()
        .find(|l| l.contains("check-release"))
        .expect("check-release invocation logged");
    assert!(
        check_release.contains("--package my-own-pkg"),
        "emitted command must carry caller's package: {check_release}"
    );
    assert!(
        !logged.contains("cargo-cicd-core"),
        "verify must not hardcode cargo-cicd-core; logged: {logged}"
    );
}

#[test]
fn verify_default_package_is_compiled_default_not_core() {
    let tmp = TempDir::new().unwrap();
    make_crate(tmp.path(), "sample-pkg");
    let (_shim, bin, log) = shim_env();

    let output = verify_cmd()
        .args(["verify", "repo"])
        .env("PATH", &bin)
        .env("SHIM_LOG", &log)
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let logged = std::fs::read_to_string(&log).unwrap();
    let check_release = logged
        .lines()
        .find(|l| l.contains("check-release"))
        .expect("check-release invocation logged");
    // The default must be a concrete package name (the binary's own
    // CARGO_PKG_NAME), never the retired cargo-cicd-core identifier.
    assert!(
        check_release.contains("--package cargo-cicd"),
        "default package expected in emitted command: {check_release}"
    );
    assert!(!logged.contains("cargo-cicd-core"));
}
