use assert_cmd::prelude::*;
use std::process::Command;

fn run_doctor_on_fixture(fixture_path: &str) -> (std::process::Output, serde_json::Value) {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["doctor", "repo", "--repo", fixture_path, "--json"])
        .output()
        .expect("failed to run cargo-cicd");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean_stdout = stdout.trim().trim_end_matches("null").trim();
    let json: serde_json::Value = serde_json::from_str(clean_stdout).unwrap_or_else(|e| {
        panic!(
            "Failed to parse JSON from fixture {}: {}\nstdout: {}",
            fixture_path, e, stdout
        )
    });
    (output, json)
}

fn load_expected(fixture_path: &str) -> (u64, Vec<String>) {
    let expected_path = format!("{}/expected.json", fixture_path);
    let content = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Missing expected.json at {}", expected_path));
    let v: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", expected_path, e));
    let q = v["Expected(q)"].as_u64().unwrap_or(0);
    let counterexamples = v["CounterexampleSet"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|s| s.as_str().map(String::from))
        .collect();
    (q, counterexamples)
}

fn fixture_dir(name: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    format!("{}/playground/{}", manifest, name)
}

fn assert_fixture(name: &str) {
    let path = fixture_dir(name);
    let (expected_q, expected_ces) = load_expected(&path);
    let (output, json) = run_doctor_on_fixture(&path);

    let actual_ces: Vec<String> = json["counterexamples"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|s| s.as_str().map(String::from))
        .collect();

    for ce in &expected_ces {
        println!("FIXTURE: {} — checking for {}", name, ce);
        assert!(
            actual_ces.iter().any(|a| a.contains(ce)),
            "FIXTURE: {} — expected counterexample '{}' not found in {:?}",
            name,
            ce,
            actual_ces
        );
    }

    if expected_q == 0 {
        assert!(
            !output.status.success(),
            "FIXTURE: {} — expected non-zero exit (q=0) but command succeeded",
            name
        );
    } else {
        assert!(
            output.status.success(),
            "FIXTURE: {} — expected zero exit (q={}) but command failed\nstderr: {}",
            name,
            expected_q,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fixture_fake_tests() {
    assert_fixture("fake-tests");
}

#[test]
fn fixture_dummy_gates() {
    assert_fixture("dummy-gates");
}

#[test]
fn fixture_manual_receipts() {
    assert_fixture("manual-receipts");
}

#[test]
fn fixture_hardcoded_commitments() {
    assert_fixture("hardcoded-commitments");
}

#[test]
fn fixture_token_gates() {
    assert_fixture("token-gates");
}

#[test]
fn fixture_raw_cargo() {
    assert_fixture("raw-cargo");
}

#[test]
fn fixture_raw_just() {
    assert_fixture("raw-just");
}

#[test]
fn fixture_synthetic_receipts() {
    assert_fixture("synthetic-receipts");
}

#[test]
fn fixture_ocel_placeholder() {
    // Expected(q) == 1 means no counterexamples, command should succeed
    assert_fixture("ocel-placeholder");
}

#[test]
fn fixture_closure_prose() {
    assert_fixture("closure-prose");
}
