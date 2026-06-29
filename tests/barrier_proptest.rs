use proptest::prelude::*;
use tempfile::TempDir;
use cargo_cicd::barrier::{detect_barriers, Counterexample};

/// Helper: write a file and run detect_barriers on the tempdir.
/// Also writes a valid .agents/hooks.json so hook_not_installed doesn't pollute results.
fn detect_in_tempdir(content: &str, filename: &str) -> Vec<Counterexample> {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join(filename), content).unwrap();
    // Suppress hook_not_installed noise
    let agents = dir.path().join(".agents");
    std::fs::create_dir_all(&agents).unwrap();
    std::fs::write(
        agents.join("hooks.json"),
        r#"{"pre-tool-use":"cargo-cicd.execute"}"#,
    )
    .unwrap();
    detect_barriers(dir.path())
}

// ── Property 1: fake_test trigger always detected ────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn fake_test_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}assert!(true){}", prefix, suffix);
        let found = detect_in_tempdir(&content, "test_fake.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::fake_test)),
            "expected fake_test in {:?}", found
        );
    }
}

// ── Property 2: clean files don't false-positive for fake_test ───────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn clean_file_no_fake_test(content in "[a-zA-Z0-9 \n]{0,200}") {
        prop_assume!(!content.contains("assert!(true)"));
        prop_assume!(!content.contains("assert_eq!(1, 1)"));
        let found = detect_in_tempdir(&content, "clean.rs");
        prop_assert!(
            !found.iter().any(|c| matches!(c, Counterexample::fake_test)),
            "unexpected fake_test in {:?}", found
        );
    }
}

// ── Property 3: dummy_gate trigger always detected ───────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn dummy_gate_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}dummy gate{}", prefix, suffix);
        let found = detect_in_tempdir(&content, "gates.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::dummy_gate)),
            "expected dummy_gate in {:?}", found
        );
    }
}

// ── Property 4: placeholder_authority trigger always detected ─────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn placeholder_authority_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}placeholder_authority{}", prefix, suffix);
        let found = detect_in_tempdir(&content, "auth.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::placeholder_authority)),
            "expected placeholder_authority in {:?}", found
        );
    }
}

// ── Property 5: manual_receipt_json trigger always detected ──────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn manual_receipt_json_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}receipt_json{}", prefix, suffix);
        let found = detect_in_tempdir(&content, "receipts.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::manual_receipt_json)),
            "expected manual_receipt_json in {:?}", found
        );
    }
}

// ── Property 6: ocel_replay_placeholder trigger always detected ───────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn ocel_replay_placeholder_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}ocel replay placeholder{}", prefix, suffix);
        let found = detect_in_tempdir(&content, "ocel.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::ocel_replay_placeholder)),
            "expected ocel_replay_placeholder in {:?}", found
        );
    }
}

// ── Property 7: hardcoded_commitment trigger always detected ──────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn hardcoded_commitment_always_detected(prefix in ".*", suffix in ".*") {
        let content = format!("{}hardcoded commitment{}", prefix, suffix);
        let found = detect_in_tempdir(&content, "commit.rs");
        prop_assert!(
            found.iter().any(|c| matches!(c, Counterexample::hardcoded_commitment)),
            "expected hardcoded_commitment in {:?}", found
        );
    }
}

// ── Fixed-seed smoke tests for CI determinism ─────────────────────────────────

#[test]
fn just_called_by_agent_detected_in_justfile() {
    let content = "build:\n    just build-inner\n";
    let found = detect_in_tempdir(content, "justfile");
    assert!(
        found.iter().any(|c| matches!(c, Counterexample::just_called_by_agent)),
        "expected just_called_by_agent, got {:?}", found
    );
}

#[test]
fn gate_without_trace_receipt_detected() {
    let content = "pub fn gate(x: u32) -> bool { x > 0 }\n";
    let found = detect_in_tempdir(content, "mygate.rs");
    assert!(
        found.iter().any(|c| matches!(c, Counterexample::gate_without_trace_receipt)),
        "expected gate_without_trace_receipt, got {:?}", found
    );
}

#[test]
fn verify_without_trace_receipt_detected() {
    let content = "pub fn verify(sig: &str) -> bool { sig.len() > 0 }\n";
    let found = detect_in_tempdir(content, "verifier.rs");
    assert!(
        found.iter().any(|c| matches!(c, Counterexample::verify_without_trace_receipt)),
        "expected verify_without_trace_receipt, got {:?}", found
    );
}

#[test]
fn gate_with_receipt_digest_not_flagged() {
    // If the file itself references receipt_digest, we assume it's wired up
    let content = "pub fn gate(x: u32) -> bool {\n    let _d = receipt_digest();\n    x > 0\n}\n";
    let found = detect_in_tempdir(content, "mygate.rs");
    assert!(
        !found.iter().any(|c| matches!(c, Counterexample::gate_without_trace_receipt)),
        "false positive gate_without_trace_receipt, got {:?}", found
    );
}
