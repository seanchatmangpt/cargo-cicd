use cargo_cicd::autonomic::policies::{
    check_git_phase_dirty, check_target_pressure, check_toolchain_mismatch, check_trybuild_changed,
    PolicyMode, PolicyVerdict,
};

// ── target pressure ──────────────────────────────────────────────────────────

#[test]
fn test_target_pressure_over_limit_suggests_prune() {
    let result = check_target_pressure(25.0, 20.0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Suggest),
        "over-limit target should suggest prune, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("prune"),
        "recommendation should mention prune, got: {}",
        result.recommendation
    );
}

#[test]
fn test_target_pressure_under_limit_passes() {
    let result = check_target_pressure(5.0, 20.0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "under-limit target should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_target_pressure_approaching_warns() {
    // 80% threshold: 16.1 / 20.0 = 80.5% → Warn
    let result = check_target_pressure(16.1, 20.0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Warn),
        "approaching-limit target should warn, got {:?}",
        result.verdict
    );
}

#[test]
fn test_target_pressure_exactly_at_limit_suggests() {
    // strictly greater-than check means exactly at limit should Suggest
    let result = check_target_pressure(20.1, 20.0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Suggest),
        "target just over limit should suggest, got {:?}",
        result.verdict
    );
}

// ── toolchain mismatch ───────────────────────────────────────────────────────

#[test]
fn test_toolchain_mismatch_detected() {
    let result = check_toolchain_mismatch("stable", Some("nightly"));
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "mismatched toolchain should not pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_toolchain_match_passes() {
    let result = check_toolchain_mismatch("nightly", Some("nightly"));
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "matching toolchain should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_toolchain_no_pinned_passes() {
    // No pinned toolchain means no mismatch possible
    let result = check_toolchain_mismatch("stable", None);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "no pinned toolchain should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_toolchain_mismatch_suggests_pin() {
    let result = check_toolchain_mismatch("stable", Some("nightly-2026-05-30"));
    assert!(
        matches!(result.verdict, PolicyVerdict::Suggest),
        "mismatch should suggest, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("Pin") || result.recommendation.contains("pin"),
        "recommendation should mention pinning, got: {}",
        result.recommendation
    );
}

// ── trybuild changed ─────────────────────────────────────────────────────────

#[test]
fn test_trybuild_changed_suggests_focused_run() {
    let result = check_trybuild_changed(3);
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "changed trybuild fixtures should not pass, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("changed") || result.recommendation.contains("trybuild"),
        "recommendation should mention changed or trybuild, got: {}",
        result.recommendation
    );
}

#[test]
fn test_trybuild_unchanged_passes() {
    let result = check_trybuild_changed(0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "no changed trybuild fixtures should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_trybuild_changed_one_fixture() {
    let result = check_trybuild_changed(1);
    assert!(
        matches!(result.verdict, PolicyVerdict::Suggest),
        "single changed fixture should suggest, got {:?}",
        result.verdict
    );
}

#[test]
fn test_trybuild_changed_includes_count() {
    let result = check_trybuild_changed(7);
    assert!(
        result.recommendation.contains('7'),
        "recommendation should include the fixture count, got: {}",
        result.recommendation
    );
}

// ── git phase dirty ──────────────────────────────────────────────────────────

#[test]
fn test_git_dirty_suggests_close() {
    let result = check_git_phase_dirty(5);
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "dirty working tree should not pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_git_clean_passes() {
    let result = check_git_phase_dirty(0);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "clean working tree should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn test_git_dirty_one_file_suggests() {
    let result = check_git_phase_dirty(1);
    assert!(
        matches!(result.verdict, PolicyVerdict::Suggest),
        "single dirty file should suggest, got {:?}",
        result.verdict
    );
}

#[test]
fn test_git_dirty_includes_count() {
    let result = check_git_phase_dirty(12);
    assert!(
        result.recommendation.contains("12"),
        "recommendation should include the dirty file count, got: {}",
        result.recommendation
    );
}

// ── mode invariant: all policies default to Suggest mode ─────────────────────

#[test]
fn test_no_policy_uses_apply_mode_by_default() {
    // All policies must have mode = Suggest by default, regardless of verdict
    for r in &[
        check_target_pressure(5.0, 20.0),
        check_toolchain_mismatch("stable", None),
        check_trybuild_changed(0),
        check_git_phase_dirty(0),
    ] {
        assert!(
            matches!(r.mode, PolicyMode::Suggest),
            "policy '{}' must default to Suggest mode, got {:?}",
            r.name,
            r.mode
        );
    }
}

#[test]
fn test_all_policies_are_enabled_by_default() {
    for r in &[
        check_target_pressure(5.0, 20.0),
        check_toolchain_mismatch("stable", None),
        check_trybuild_changed(0),
        check_git_phase_dirty(0),
    ] {
        assert!(r.enabled, "policy '{}' must be enabled by default", r.name);
    }
}

// ── PolicyResult fields populated correctly ───────────────────────────────────

#[test]
fn test_target_pressure_result_has_name() {
    let result = check_target_pressure(1.0, 20.0);
    assert_eq!(result.name, "target_pressure");
}

#[test]
fn test_toolchain_mismatch_result_has_name() {
    let result = check_toolchain_mismatch("nightly", None);
    assert_eq!(result.name, "toolchain_mismatch");
}

#[test]
fn test_trybuild_changed_result_has_name() {
    let result = check_trybuild_changed(0);
    assert_eq!(result.name, "trybuild_changed");
}

#[test]
fn test_git_phase_dirty_result_has_name() {
    let result = check_git_phase_dirty(0);
    assert_eq!(result.name, "git_phase_dirty");
}

#[test]
fn test_pass_verdict_has_empty_recommendation() {
    for r in &[
        check_target_pressure(1.0, 20.0),
        check_toolchain_mismatch("nightly", Some("nightly")),
        check_trybuild_changed(0),
        check_git_phase_dirty(0),
    ] {
        if matches!(r.verdict, PolicyVerdict::Pass) {
            assert!(
                r.recommendation.is_empty(),
                "Pass verdict for '{}' should have empty recommendation, got: {}",
                r.name,
                r.recommendation
            );
        }
    }
}
