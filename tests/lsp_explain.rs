/// Proof Tests — `cargo cicd lsp explain` catalog coverage
///
/// Verifies that the static CICD diagnostic code catalog is wired to the CLI:
/// each known code returns a structured explanation on stdout with exit 0;
/// unknown codes emit an error message on stderr and exit non-zero.
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
