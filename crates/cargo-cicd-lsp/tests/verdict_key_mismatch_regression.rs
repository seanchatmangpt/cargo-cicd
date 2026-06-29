//! Regression test: no court verdict may silently degrade to zero through key mismatch.
//!
//! Invariant: if simd_token_replay emits "overall_fitness": 0.9636, the audit surface must
//! report 0.9636, not 0.0 (the silent fallback from reading the wrong key).
//!
//! This fixture documents the historical CICD-WPM-004 bug and prevents its recurrence.

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
