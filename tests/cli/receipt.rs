//! Chicago subprocess tests for the `receipt` noun: real blake3-minted
//! receipts on real disk, adjudicated by the real binary. Valid receipts are
//! minted through the crate's own `Receipt::mint` (the same code path
//! `trace profile` uses) rather than hand-written JSON, because a valid
//! receipt's digest is a blake3 self-hash the test cannot fabricate.

use assert_cmd::Command;
use cargo_cicd::nouns::receipt::{content_digest, ExecutionTrace, Receipt};
use std::collections::BTreeMap;
use std::path::Path;
use tempfile::TempDir;

fn cicd(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(args)
        .current_dir(dir)
        .assert()
}

/// The CLI harness prints the verb's JSON on one line, then a trailing
/// status line (e.g. `null`); parse the first JSON line only.
fn parse_first_json(stdout: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(stdout);
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .expect("no JSON line on stdout");
    serde_json::from_str(line).expect("stdout JSON line should parse")
}

fn receipts_dir(dir: &Path) -> std::path::PathBuf {
    let d = dir.join(".cargo-cicd").join("receipts");
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn mint_valid(command: &[&str], stdout: &[u8]) -> Receipt {
    let trace = ExecutionTrace {
        command: command.iter().map(|s| s.to_string()).collect(),
        exit_code: 0,
        stdout_digest: content_digest(stdout),
        stderr_digest: content_digest(b""),
        git_before: "aaa".into(),
        git_after: "aaa".into(),
        input_artifacts: BTreeMap::new(),
        output_artifacts: BTreeMap::new(),
    };
    Receipt::mint(&trace)
}

fn write_receipt(dir: &Path, name: &str, receipt: &Receipt) {
    let path = receipts_dir(dir).join(name);
    std::fs::write(path, serde_json::to_string(receipt).unwrap()).unwrap();
}

#[test]
fn verify_on_empty_receipts_dir_reports_zero_valid_with_v1_schema() {
    let tmp = TempDir::new().unwrap();
    receipts_dir(tmp.path()); // exists but empty — not the typed missing-dir error
    let output = cicd(tmp.path(), &["receipt", "verify", "--repo", ".", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&output);
    assert_eq!(v["schema"], "cargo-cicd.receipt.verify.v1");
    assert_eq!(v["valid_receipts"], 0);
    assert_eq!(v["counterexamples"], serde_json::json!([]));
    assert_eq!(v["q"], 1);
}

#[test]
fn verify_counts_seeded_valid_receipt() {
    let tmp = TempDir::new().unwrap();
    let good = mint_valid(&["just", "test"], b"ok stdout");
    write_receipt(tmp.path(), "trace-profile-test.json", &good);
    let output = cicd(tmp.path(), &["receipt", "verify", "--repo", ".", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&output);
    assert_eq!(v["valid_receipts"], 1);
    assert_eq!(v["q"], 1);
}

#[test]
fn tampered_digest_is_a_counterexample_and_audit_fails_with_exit_1() {
    let tmp = TempDir::new().unwrap();
    let mut tampered = mint_valid(&["just", "test"], b"ok stdout");
    tampered.exit_code = 7; // mutate body without re-minting digest
    write_receipt(tmp.path(), "tampered.json", &tampered);

    // verify surfaces the counterexample (verify itself still exits 0)
    let out = cicd(tmp.path(), &["receipt", "verify", "--repo", ".", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&out);
    assert_eq!(v["valid_receipts"], 0);
    let cex = v["counterexamples"].as_array().unwrap();
    assert_eq!(cex.len(), 1);
    assert_eq!(cex[0], "receipt_hashes_itself");

    // audit fails closed: invalid 1, exit code 1
    let audit_out = cicd(tmp.path(), &["receipt", "audit", "--repo", ".", "--json"])
        .failure()
        .get_output()
        .stdout
        .clone();
    let a = parse_first_json(&audit_out);
    assert_eq!(a["schema"], "cargo-cicd.receipt.audit.v1");
    assert_eq!(a["invalid_receipts"], 1);
}

#[test]
fn audit_on_missing_receipts_dir_is_typed_error_with_exit_1() {
    let tmp = TempDir::new().unwrap();
    // no .cargo-cicd/receipts at all — must NOT read as vacuous success
    let output = cicd(tmp.path(), &["receipt", "audit", "--repo", ".", "--json"])
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("receipts directory not found"),
        "expected typed missing-dir error, got: {}",
        stderr
    );
}
