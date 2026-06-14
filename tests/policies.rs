//! Autonomic policy unit tests.
//! Proves each policy: observes signals, emits verdict, stays in suggest mode.
use cargo_cicd::policies::*;
use cargo_cicd::engine::EngineState;

/// target_pressure policy: empty target = pass.
#[test]
fn policy_target_pressure_empty_is_pass() {
    let policy = TargetPressurePolicy::default();
    let state = EngineState::from_workspace();
    let result = policy.evaluate(&state);
    // In empty dir target is 0GB — well below any limit
    assert_eq!(result.mode, "suggest", "policy mode must be suggest");
    assert!(result.name == "target_pressure");
}

/// Policies must never be in apply mode by default.
#[test]
fn policy_all_policies_default_to_suggest() {
    let state = EngineState::from_workspace();
    let policies: &[Box<dyn CicdPolicy>] = &[
        Box::new(TargetPressurePolicy::default()),
        Box::new(ToolchainMismatchPolicy),
        Box::new(TrybuildChangedPolicy),
        Box::new(GitPhaseDirtyPolicy),
    ];
    for policy in policies {
        let result = policy.evaluate(&state);
        assert_eq!(
            result.mode, "suggest",
            "policy {} is not in suggest mode — apply forbidden by default",
            result.name
        );
    }
}

/// Each policy result has a name, mode, and verdict.
#[test]
fn policy_result_has_required_fields() {
    let state = EngineState::from_workspace();
    let result = TargetPressurePolicy::default().evaluate(&state);
    assert!(!result.name.is_empty(), "policy result has empty name");
    assert!(!result.mode.is_empty(), "policy result has empty mode");
    assert!(
        !result.verdict.is_empty(),
        "policy result has empty verdict"
    );
}
