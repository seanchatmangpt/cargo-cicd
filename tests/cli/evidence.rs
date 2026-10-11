//! Chicago subprocess tests for the `evidence` noun (doctor/show/list/reset).
//!
//! Every test runs the real compiled `cargo-cicd` binary in an isolated
//! `TempDir` cwd and asserts on real exit codes, real stdout, and real
//! on-disk state — no mocks, no doubles.

use assert_cmd::Command;
use tempfile::TempDir;

fn cicd() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

/// `evidence doctor` with no recorded process receipt must fail closed with a
/// typed BLOCKED (non-zero exit) and must NOT mint a sentinel receipt file —
/// self-attestation is not evidence.
#[test]
fn doctor_without_evidence_is_blocked_and_never_mints_a_receipt() {
    let dir = TempDir::new().unwrap();
    let out = cicd()
        .args(["evidence", "doctor"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "doctor must fail when evidence is missing (BLOCKED), got exit {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("BLOCKED"),
        "failure must be a typed BLOCKED, got: {combined}"
    );
    let latest = dir
        .path()
        .join("target/cargo-cicd/evidence/receipts/latest.json");
    assert!(
        !latest.exists(),
        "no sentinel receipt may be created for missing evidence, found {}",
        latest.display()
    );
}

/// `evidence show` on an empty evidence dir: exit 0 with informative output.
#[test]
fn evidence_show_exits_zero_with_non_empty_output() {
    let dir = TempDir::new().unwrap();
    let out = cicd()
        .args(["evidence", "show"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "evidence show must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "evidence show must print something, got empty stdout"
    );
}

/// `evidence list` on an empty evidence dir: exit 0 with non-empty output.
#[test]
fn evidence_list_exits_zero_with_non_empty_output() {
    let dir = TempDir::new().unwrap();
    let out = cicd()
        .args(["evidence", "list"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "evidence list must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "evidence list must print something, got empty stdout"
    );
}

/// `evidence reset` on a seeded evidence tree: removes the journal
/// (`events.jsonl` + `events.ocel.json`), preserves `receipts/`, exits 0.
#[test]
fn reset_clears_journals_and_preserves_receipts() {
    let dir = TempDir::new().unwrap();
    let evidence = dir.path().join("target/cargo-cicd/evidence");
    std::fs::create_dir_all(evidence.join("receipts")).unwrap();
    std::fs::write(evidence.join("events.ocel.json"), r#"{"events":[]}"#).unwrap();
    std::fs::write(evidence.join("events.jsonl"), "{\"command\":\"x\"}\n").unwrap();
    let receipt = evidence.join("receipts/permanent.json");
    std::fs::write(&receipt, "{\"permanent\":true}").unwrap();

    let out = cicd()
        .args(["evidence", "reset"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "evidence reset must succeed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        !evidence.join("events.ocel.json").exists(),
        "reset must remove the OCEL journal"
    );
    assert!(
        !evidence.join("events.jsonl").exists(),
        "reset must remove the events journal"
    );
    assert!(
        receipt.exists(),
        "reset must preserve receipts/ (permanent records)"
    );
}
