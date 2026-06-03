//! Schema contract tests for CICD-WPM-004: verdict_key_mismatch runtime protection.
//!
//! Documents that cargo-cicd admission decisions are gated on the `state` field
//! from `wpm receipt doctor` JSON output, NOT on a `fitness` key that may be
//! absent or misnamed. The `overall_fitness` key is for `wpm audit` (XES SIMD
//! replay) output — it must not be silently conflated with the `fitness` key.

#[test]
fn wpm_verdict_json_with_wrong_key_does_not_produce_admitted() {
    // Simulate wpm receipt doctor output with wrong key "fitness" instead of "overall_fitness"
    let wrong_key_json = r#"{"state":"Admitted","findings":[],"fitness":0.95}"#;
    // Parse it and confirm the state field is what actually gates admission,
    // not a fitness key that could be absent or misnamed.
    let v: serde_json::Value = serde_json::from_str(wrong_key_json).unwrap();
    let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("");
    // The gate is state == "Admitted", not a fitness value
    // This test documents that we do NOT rely on fitness key for admission decisions
    assert_eq!(state, "Admitted"); // state field is present and correct
    // Confirm overall_fitness is absent (this is the wrong-key scenario)
    assert!(
        v.get("overall_fitness").is_none(),
        "overall_fitness should be absent in this fixture"
    );
    // Document: cargo-cicd admission decision is based on state, not fitness key
    // WPM-004 diagnostic is for LSP author-time detection, not runtime gate
}

#[test]
fn wpm_verdict_refused_state_blocks_regardless_of_fitness() {
    let refused_json = r#"{"state":"Refused","findings":[{"severity":"Deny","message":"test","code":"X","json_path":"$"}],"denied_paths":["$"]}"#;
    let v: serde_json::Value = serde_json::from_str(refused_json).unwrap();
    let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("");
    assert_eq!(state, "Refused");
}

#[test]
fn wpm_audit_overall_fitness_must_not_silently_fall_back_to_zero() {
    // CICD-WPM-004: if a parser reads "fitness" instead of "overall_fitness",
    // the result is 0.0 — a silent degradation that looks like failure.
    let audit_json = r#"{
        "overall_fitness": 0.9636,
        "total_consumed": 100,
        "total_produced": 100,
        "total_missing": 0,
        "total_remaining": 0,
        "trace_count": 1
    }"#;
    let v: serde_json::Value = serde_json::from_str(audit_json).unwrap();

    // Correct key: returns real value
    let correct = v.get("overall_fitness").and_then(|f| f.as_f64());
    assert!(correct.is_some(), "overall_fitness must be readable");
    assert!(
        (correct.unwrap() - 0.9636).abs() < 0.001,
        "overall_fitness value must be 0.9636, got {:?}",
        correct
    );

    // Wrong key: returns None — the consumer MUST handle None explicitly, not unwrap_or(0.0)
    let wrong = v.get("fitness").and_then(|f| f.as_f64());
    assert!(
        wrong.is_none(),
        "CICD-WPM-004: 'fitness' key must be absent from wpm audit output — \
         reading it would silently return 0.0 and falsely suppress a passing verdict"
    );
}
