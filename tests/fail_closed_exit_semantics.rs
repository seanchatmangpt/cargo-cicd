//! Fail-closed exit semantics (Phase B2 lane B2d).
//!
//! Law: a FAIL verdict (or a missing tool) must exit the process non-zero.
//! These are Chicago tests: real subprocesses, real temp fixture repos,
//! assertions on real final state (exit codes, files on disk) — no mocks.

use assert_cmd::Command;
use std::process::Command as StdCommand;

fn bin() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let st = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .status()
        .expect("git available");
    assert!(st.success(), "git {args:?} failed");
}

/// Temp crate whose `cargo test` deterministically FAILS (syntax error).
fn failing_crate() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn broken( {}\n").unwrap();
    dir
}

// ── 1. test run / bench: cargo failure ⇒ non-zero ────────────────────────────

#[test]
fn test_run_failing_crate_exits_non_zero() {
    let dir = failing_crate();
    let output = bin()
        .args(["test", "run"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "test run must exit non-zero when cargo test fails"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cargo test failed"),
        "expected typed failure message; got:\n{stderr}"
    );
}

#[test]
fn test_bench_failing_crate_exits_non_zero() {
    let dir = failing_crate();
    let output = bin()
        .args(["test", "bench"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "test bench must exit non-zero when cargo bench fails"
    );
}

// ── 2. trybuild update: snapshot failures ⇒ non-zero ─────────────────────────

#[test]
fn trybuild_update_failing_crate_exits_non_zero() {
    let dir = failing_crate();
    let output = bin()
        .args(["trybuild", "update"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "trybuild update must exit non-zero when the cargo run fails"
    );
}

// ── 3. workspace doctor / validate: FAIL checks ⇒ non-zero ──────────────────

#[test]
fn workspace_doctor_fails_closed_in_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    // No Cargo.toml, no .git → two FAIL checks.
    let output = bin()
        .args(["workspace", "doctor"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "workspace doctor must exit non-zero on FAIL checks"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("FAIL"),
        "doctor must report FAIL checks; got:\n{stdout}"
    );
}

#[test]
fn workspace_validate_fails_closed_without_workspace_section() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
    let output = bin()
        .args(["workspace", "validate"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "workspace validate must exit non-zero on FAIL verdict"
    );
}

#[test]
fn workspace_sync_without_ggen_toml_still_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let output = bin()
        .args(["workspace", "sync"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "sync with no ggen.toml is a PASS no-op and must exit 0"
    );
}

// ── 4. target prune: --apply honored from parsed flag ────────────────────────

#[test]
fn target_prune_plan_mode_does_not_delete() {
    let dir = tempfile::tempdir().unwrap();
    let deps = dir.path().join("target/debug/deps");
    std::fs::create_dir_all(&deps).unwrap();
    std::fs::write(deps.join("artifact"), b"x").unwrap();
    let output = bin()
        .args(["target", "prune"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "plan mode must exit 0");
    assert!(
        deps.exists(),
        "plan mode must not delete target/debug/deps"
    );
}

#[test]
fn target_prune_apply_flag_deletes_incremental_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let deps = dir.path().join("target/debug/deps");
    std::fs::create_dir_all(&deps).unwrap();
    std::fs::write(deps.join("artifact"), b"x").unwrap();
    let output = bin()
        .args(["target", "prune", "--apply"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "apply mode must exit 0");
    assert!(
        !deps.exists(),
        "--apply must actually delete target/debug/deps"
    );
}

// ── 5. sbom generate: missing tool = typed Unavailable, non-zero ─────────────

#[test]
fn sbom_generate_missing_cyclonedx_is_typed_unavailable_non_zero() {
    let dir = tempfile::tempdir().unwrap();
    let output = bin()
        .args(["sbom", "generate"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "missing cargo-cyclonedx must exit non-zero (fail-closed)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("TOOL_UNAVAILABLE") && combined.contains("cyclonedx"),
        "expected typed TOOL_UNAVAILABLE error naming cyclonedx; got:\n{combined}"
    );
    assert!(
        !dir.path().join("sbom.json").exists(),
        "no sbom.json may be produced when the tool is unavailable"
    );
}

// ── 6. verify repo: typed Unavailable when cargo probe fails; --json honored ─

#[test]
fn verify_repo_without_cargo_on_path_is_typed_unavailable_non_zero() {
    let dir = tempfile::tempdir().unwrap();
    // Strip PATH down to a directory with no `cargo` binary: the
    // semver-checks availability probe fails → typed Unavailable, non-zero.
    let empty = tempfile::tempdir().unwrap();
    let output = bin()
        .args(["verify", "repo", "--json"])
        .current_dir(dir.path())
        .env("PATH", empty.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "verify with unavailable semver-checks must exit non-zero"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("TOOL_UNAVAILABLE") && combined.contains("semver-checks"),
        "expected typed TOOL_UNAVAILABLE error naming semver-checks; got:\n{combined}"
    );
}

#[test]
fn verify_repo_json_flag_emits_schema() {
    // With cargo available, --json must be honored and emit the v2 schema.
    let dir = tempfile::tempdir().unwrap();
    let output = bin()
        .args(["verify", "repo", "--json"])
        .current_dir(dir.path())
        .env("PATH", std::env::var("PATH").unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains('{') {
        assert!(
            stdout.contains("cargo-cicd.verify.v2"),
            "--json must emit the cargo-cicd.verify.v2 schema; got:\n{stdout}"
        );
    }
    // Either way the flag must not be silently ignored: non-JSON human output
    // is rejected when --json was requested.
    if output.status.success() {
        assert!(
            stdout.trim_start().starts_with('{'),
            "--json requested but output was not JSON; got:\n{stdout}"
        );
    }
}

// ── 7. publish run: refuses without --dry-run ────────────────────────────────

#[test]
fn publish_run_without_dry_run_flag_is_typed_refusal() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    let output = bin()
        .args(["publish", "run"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "publish run without --dry-run must exit non-zero (typed refusal)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("REFUSED:explicit_dry_run_required"),
        "expected typed REFUSED:explicit_dry_run_required; got:\n{combined}"
    );
    assert!(
        !dir.path().join("cicd.toml").exists(),
        "refused publish run must not write cicd.toml"
    );
}

#[test]
fn publish_run_with_dry_run_still_refuses_when_oracle_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    let output = bin()
        .args(["publish", "run", "--dry-run"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if combined.contains("wasm4pm oracle unavailable") {
        // Hard-refuse path: non-zero, never a PASS verdict.
        assert!(
            !output.status.success(),
            "oracle unavailability must hard-refuse (non-zero), not continue"
        );
        assert!(
            combined.contains("REFUSED:oracle_unavailable"),
            "expected typed REFUSED:oracle_unavailable; got:\n{combined}"
        );
    }
}

// ── 8. ci run remote: unwatched dispatch must not be PASS (unit-level law) ───
// (run_remote's watch-spawn-failure arm is exercised at the code level; a
// subprocess test would need a real gh dispatch. The arm now maps Err → FAIL,
// and run_ci bails on FAIL, so spawn failure exits non-zero.)
