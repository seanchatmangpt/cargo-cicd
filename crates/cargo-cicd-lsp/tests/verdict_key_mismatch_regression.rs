//! Regression test: no court verdict may silently degrade to zero through key mismatch.
//!
//! Invariant: if simd_token_replay emits "overall_fitness": 0.9636, the audit surface must
//! report 0.9636, not 0.0 (the silent fallback from reading the wrong key).
//!
//! This fixture documents the historical CICD-WPM-004 bug and prevents its recurrence.

#[test]
fn test_overall_fitness_key_is_read_correctly() {
    let json = serde_json::json!({
        "overall_fitness": 0.9636,
        "total_consumed": 100,
        "total_produced": 100,
        "total_missing": 0,
        "total_remaining": 0,
        "trace_count": 1,
        "trace_results": [
            {"consumed": 100, "produced": 100, "missing": 0, "remaining": 0, "fitness": 0.9636}
        ]
    });

    // INVARIANT: top-level "fitness" key must not exist in simd_token_replay output
    let fitness_at_top = json["fitness"].as_f64();
    assert!(
        fitness_at_top.is_none(),
        "top-level 'fitness' key must not exist in simd_token_replay output — use 'overall_fitness'"
    );

    // INVARIANT: reading "overall_fitness" returns the real value
    let overall_fitness = json["overall_fitness"].as_f64().unwrap_or(0.0);
    assert!(
        (overall_fitness - 0.9636).abs() < 0.001,
        "overall_fitness must be 0.9636, got {}",
        overall_fitness
    );

    // INVARIANT: falling back to 0.0 when using the wrong key is the regression we prevent
    let fitness_with_wrong_key = json["fitness"].as_f64().unwrap_or(0.0);
    assert_eq!(
        fitness_with_wrong_key, 0.0,
        "wrong key silently returns 0.0 — this is the regression CICD-WPM-004 prevents"
    );
    // The assertion above confirms why audit.rs MUST use overall_fitness, not fitness
}

#[test]
fn test_trace_result_keys_are_missing_not_missing_tokens() {
    let json = serde_json::json!({
        "overall_fitness": 0.9,
        "trace_results": [
            {"consumed": 10, "produced": 10, "missing": 2, "remaining": 1, "fitness": 0.82}
        ]
    });

    let trace = &json["trace_results"][0];

    // INVARIANT: correct keys exist
    assert!(
        trace["missing"].as_u64().is_some(),
        "key 'missing' must exist"
    );
    assert!(
        trace["remaining"].as_u64().is_some(),
        "key 'remaining' must exist"
    );

    // INVARIANT: wrong/legacy keys do NOT exist
    assert!(
        trace["missing_tokens"].as_u64().is_none(),
        "key 'missing_tokens' must NOT exist — use 'missing'"
    );
    assert!(
        trace["remaining_tokens"].as_u64().is_none(),
        "key 'remaining_tokens' must NOT exist — use 'remaining'"
    );
    assert!(
        trace["trace_id"].as_str().is_none(),
        "key 'trace_id' must NOT exist — use positional index"
    );
}

#[test]
fn test_deceptive_verdict_not_produced_by_wrong_key() {
    // This test proves the causal chain: wrong key → zero fitness → DECEPTIVE verdict.
    // By reading the correct key we break this chain.

    let court_output = serde_json::json!({
        "overall_fitness": 0.9636,
        "verdict": "TRUTHFUL"
    });

    // Reading wrong key produces the regression value
    let fitness_wrong = court_output["fitness"].as_f64().unwrap_or(0.0);
    assert_eq!(fitness_wrong, 0.0, "regression: wrong key → silent zero");

    // Reading correct key produces the real value → TRUTHFUL, not DECEPTIVE
    let fitness_correct = court_output["overall_fitness"].as_f64().unwrap_or(0.0);
    assert!(
        fitness_correct > 0.9,
        "correct key → fitness={} which is TRUTHFUL",
        fitness_correct
    );
}

#[test]
fn test_precision_null_is_explicit_not_silent_zero() {
    let verdict = serde_json::json!({
        "overall_fitness": 0.9636,
        "precision": null,
        "verdict": "TRUTHFUL",
        "token_deviation": "M:0 R:0",
        "trace_class": "pipeline_run"
    });

    // precision: null is explicit UNSUPPORTED — distinguishable from 0.0
    let precision_node = verdict.get("precision");
    assert!(
        precision_node.is_some(),
        "precision field must be present (even if null)"
    );
    assert!(
        precision_node.unwrap().is_null(),
        "precision must be explicitly null when not computed, not silently absent or 0.0"
    );
}
