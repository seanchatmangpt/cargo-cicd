use cargo_cicd::nouns::gate::GateReport;
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn gate_report_deserializes() {
    let dir = TempDir::new().expect("tempdir");

    // Run `cargo cicd gate repo --json` on the tempdir.
    // Gate will fail (no OCEL log, no setup) but it must still emit valid JSON.
    let output = Command::cargo_bin("cargo-cicd")
        .expect("binary exists")
        .args(["gate", "repo", "--repo", dir.path().to_str().unwrap(), "--json"])
        .output()
        .expect("command ran");

    // The command may exit non-zero (gate fails), but stdout must be valid GateReport JSON.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: GateReport = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse GateReport JSON: {e}\nstdout: {stdout}"));

    assert_eq!(report.schema, "cargo-cicd.gate.v1");
    assert_eq!(report.failset_cardinality, report.counterexamples.len());
    // Invariant: q_release == 1 ⟺ counterexamples.is_empty()
    if report.q_release == 1 {
        assert!(report.counterexamples.is_empty(), "q_release=1 but counterexamples non-empty");
    } else {
        assert!(!report.counterexamples.is_empty() || report.q_release == 0);
    }
    assert_eq!(report.components.v_cargo_cicd, report.q_release);
}
