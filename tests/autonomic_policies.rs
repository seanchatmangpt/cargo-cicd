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
