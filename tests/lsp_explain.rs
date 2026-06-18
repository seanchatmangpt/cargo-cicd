//! Proof Tests — `cargo cicd lsp explain` catalog coverage
//!
//! Verifies that the static CICD diagnostic code catalog is wired to the CLI:
//! each known code returns a structured explanation on stdout with exit 0;
//! unknown codes emit an error message on stderr and exit non-zero.
//!
//! The `lsp` noun is feature-gated, so the whole suite is gated behind the
//! `lsp` feature. Run it with `cargo test --features lsp --test lsp_explain`.
#![cfg(feature = "lsp")]

use assert_cmd::Command;

// ── CICD-GIT-001 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_git_001_dirty_tree() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GIT-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("dirty_tree_blocks_close"),
        "expected 'dirty_tree_blocks_close' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Code:"),
        "expected 'Code:' label in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-EVIDENCE-003 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_evidence_003_hardcoded_timestamp() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-EVIDENCE-003"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("hardcoded_timestamp"),
        "expected 'hardcoded_timestamp' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-WPM-001 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_wpm_001_unconfirmed_receipt_court() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-WPM-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("unconfirmed_receipt_court"),
        "expected 'unconfirmed_receipt_court' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PUBLIC-001 ──────────────────────────────────────────────────────────

#[test]
fn lsp_explain_public_001_private_term_leak() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PUBLIC-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("private_term_leak"),
        "expected 'private_term_leak' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-CLOSE-001 ───────────────────────────────────────────────────────────

#[test]
fn lsp_explain_close_001_false_close_risk() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-CLOSE-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("false_close_risk"),
        "expected 'false_close_risk' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── Unknown code → stderr + non-zero exit ────────────────────────────────────

#[test]
fn lsp_explain_unknown_code_exits_nonzero_with_stderr() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-BOGUS-999"]);
    let out = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert!(
        stderr.contains("unknown diagnostic code"),
        "expected 'unknown diagnostic code' in stderr; got:\n{stderr}"
    );
}

// ── Known codes exit 0; unknown codes exit non-zero (exit-code contract) ─────

#[test]
fn lsp_explain_known_code_exits_zero() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GIT-001"]);
    cmd.assert().success();
}

#[test]
fn lsp_explain_unknown_code_exits_nonzero() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-DOES-NOT-EXIST"]);
    cmd.assert().failure();
}

// ── CICD-GIT-002 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_git_002_untracked_files() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GIT-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("untracked_files_present"),
        "expected 'untracked_files_present' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-GIT-003 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_git_003_branch_behind_remote() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GIT-003"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("branch_behind_remote"),
        "expected 'branch_behind_remote' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PIPELINE-001 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_pipeline_001_pipeline_stage_failed() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PIPELINE-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("pipeline_stage_failed"),
        "expected 'pipeline_stage_failed' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PIPELINE-002 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_pipeline_002_no_cicd_toml_found() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PIPELINE-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("no_cicd_toml_found"),
        "expected 'no_cicd_toml_found' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-TEST-001 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_test_001_test_failures_block_close() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-TEST-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("test_failures_block_close"),
        "expected 'test_failures_block_close' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-WORKSPACE-001 ───────────────────────────────────────────────────────

#[test]
fn lsp_explain_workspace_001_workspace_structure_invalid() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-WORKSPACE-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("workspace_structure_invalid"),
        "expected 'workspace_structure_invalid' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-EVIDENCE-001 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_evidence_001_evidence_missing() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-EVIDENCE-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("evidence_missing"),
        "expected 'evidence_missing' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-EVIDENCE-002 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_evidence_002_stale_evidence() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-EVIDENCE-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("stale_evidence"),
        "expected 'stale_evidence' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-EVIDENCE-004 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_evidence_004_missing_case_id() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-EVIDENCE-004"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("missing_case_id"),
        "expected 'missing_case_id' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-EVIDENCE-005 ────────────────────────────────────────────────────────

#[test]
fn lsp_explain_evidence_005_receipt_before_court() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-EVIDENCE-005"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("receipt_before_court"),
        "expected 'receipt_before_court' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-WPM-002 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_wpm_002_capability_scan_missing() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-WPM-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("capability_scan_missing"),
        "expected 'capability_scan_missing' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-WPM-003 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_wpm_003_runtime_court_not_invoked() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-WPM-003"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("runtime_court_not_invoked"),
        "expected 'runtime_court_not_invoked' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-WPM-004 ─────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_wpm_004_verdict_key_mismatch() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-WPM-004"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("verdict_key_mismatch"),
        "expected 'verdict_key_mismatch' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-TARGET-001 ──────────────────────────────────────────────────────────

#[test]
fn lsp_explain_target_001_target_growth_warning() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-TARGET-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("target_growth_warning"),
        "expected 'target_growth_warning' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-TARGET-002 ──────────────────────────────────────────────────────────

#[test]
fn lsp_explain_target_002_target_prune_requires_dry_run() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-TARGET-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("target_prune_requires_dry_run"),
        "expected 'target_prune_requires_dry_run' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PUBLISH-001 ─────────────────────────────────────────────────────────

#[test]
fn lsp_explain_publish_001_dry_run_missing() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PUBLISH-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("dry_run_missing"),
        "expected 'dry_run_missing' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PUBLISH-002 ─────────────────────────────────────────────────────────

#[test]
fn lsp_explain_publish_002_dry_run_without_receipt() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PUBLISH-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("dry_run_without_receipt"),
        "expected 'dry_run_without_receipt' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PUBLISH-003 ─────────────────────────────────────────────────────────

#[test]
fn lsp_explain_publish_003_package_changed_after_dry_run() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PUBLISH-003"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("package_changed_after_dry_run"),
        "expected 'package_changed_after_dry_run' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-PUBLIC-002 ──────────────────────────────────────────────────────────

#[test]
fn lsp_explain_public_002_public_boundary_scan_stale() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-PUBLIC-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("public_boundary_scan_stale"),
        "expected 'public_boundary_scan_stale' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-GGEN-001 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_ggen_001_rendered_surface_stale() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GGEN-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("rendered_surface_stale"),
        "expected 'rendered_surface_stale' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-GGEN-002 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_ggen_002_rendered_surface_drift() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GGEN-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("rendered_surface_drift"),
        "expected 'rendered_surface_drift' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-GGEN-003 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_ggen_003_custom_region_missing() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-GGEN-003"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("custom_region_missing"),
        "expected 'custom_region_missing' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-SPEC-001 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_spec_001_spec_missing_for_change() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-SPEC-001"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("spec_missing_for_change"),
        "expected 'spec_missing_for_change' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}

// ── CICD-SPEC-002 ────────────────────────────────────────────────────────────

#[test]
fn lsp_explain_spec_002_task_done_without_evidence() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["lsp", "explain", "CICD-SPEC-002"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        stdout.contains("task_done_without_evidence"),
        "expected 'task_done_without_evidence' in stdout; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Repair:"),
        "expected 'Repair:' label in stdout; got:\n{stdout}"
    );
}
