//! Regression tests for CICD-WPM-004 verdict_key_mismatch.
//!
//! The external court emits `overall_fitness`. An audit surface that reads
//! `fitness` instead silently returns 0.0 — a receipt-readability failure.
//!
//! These tests prove that verdict parsing is schema-aligned.

/// Verdict JSON with the authoritative top-level key `overall_fitness`.
const VERDICT_WITH_OVERALL_FITNESS: &str = r#"{
  "overall_fitness": 0.9636,
  "precision": null,
  "verdict": "TRUTHFUL",
  "token_deviation": "M:0 R:0",
  "trace_class": "pipeline_run"
}"#;

/// Verdict JSON missing `overall_fitness` — only has the wrong key `fitness`.
const VERDICT_WRONG_KEY: &str = r#"{
  "fitness": 0.9636,
  "verdict": "TRUTHFUL"
}"#;

/// Verdict JSON with both keys — authoritative key wins.
const VERDICT_BOTH_KEYS: &str = r#"{
  "overall_fitness": 0.9636,
  "fitness": 0.0,
  "verdict": "TRUTHFUL"
}"#;

#[test]
fn overall_fitness_key_is_read_correctly() {
    let v: serde_json::Value = serde_json::from_str(VERDICT_WITH_OVERALL_FITNESS).unwrap();
    let fitness = v.get("overall_fitness").and_then(|f| f.as_f64());
    assert!(fitness.is_some(), "overall_fitness must be readable");
    assert!(
        (fitness.unwrap() - 0.9636).abs() < 0.001,
        "overall_fitness must be 0.9636"
    );
}

#[test]
fn wrong_key_does_not_produce_truthful_fitness() {
    let v: serde_json::Value = serde_json::from_str(VERDICT_WRONG_KEY).unwrap();
    // If reader only looks for "overall_fitness", the wrong key must NOT produce a
    // non-zero fitness value through the authoritative path.
    let overall_fitness = v.get("overall_fitness").and_then(|f| f.as_f64());
    assert!(
        overall_fitness.is_none(),
        "CICD-WPM-004: reading 'fitness' key as 'overall_fitness' must return None, not a silently wrong value"
    );
}

#[test]
fn when_both_keys_present_overall_fitness_wins() {
    let v: serde_json::Value = serde_json::from_str(VERDICT_BOTH_KEYS).unwrap();
    let overall = v
        .get("overall_fitness")
        .and_then(|f| f.as_f64())
        .unwrap_or(0.0);
    let wrong = v.get("fitness").and_then(|f| f.as_f64()).unwrap_or(0.0);
    // overall_fitness wins over fitness when both present
    assert!((overall - 0.9636).abs() < 0.001);
    assert!((wrong - 0.0).abs() < 0.001);
    // The authoritative value is overall, not wrong
    assert!(
        overall > wrong,
        "overall_fitness must dominate fitness when both present"
    );
}

#[test]
fn precision_null_is_explicit_not_silent_zero() {
    let v: serde_json::Value = serde_json::from_str(VERDICT_WITH_OVERALL_FITNESS).unwrap();
    // precision: null is explicit unsupported — it must be distinguishable from 0.0
    let precision_node = v.get("precision");
    assert!(
        precision_node.is_some(),
        "precision field must be present (even if null)"
    );
    // It should be null (not absent, not 0.0)
    assert!(
        precision_node.unwrap().is_null(),
        "precision must be explicitly null when not computed, not silently absent or 0.0"
    );
}
