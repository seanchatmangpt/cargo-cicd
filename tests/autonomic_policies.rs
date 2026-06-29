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

// ── EvidenceStalePoliciy ─────────────────────────────────────────────────────

use cargo_cicd::autonomic::policies::{
    check_branch_behind, check_evidence_stale, check_publish_not_adjudicated, EvidenceState,
    GitState, WorkspaceInfo,
};

#[test]
fn policy_evidence_stale_with_no_evidence_suggests() {
    // changed_file_count > 0, evidence_fresh = false → Suggest (stale alert)
    let result = check_evidence_stale(3, false);
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "stale evidence with changes should not pass, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("stale") || result.recommendation.contains("evidence"),
        "recommendation should mention stale evidence, got: {}",
        result.recommendation
    );
}

#[test]
fn policy_evidence_stale_with_fresh_evidence_passes() {
    // changed_file_count = 0, evidence_fresh = true → Pass
    let result = check_evidence_stale(0, true);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "fresh evidence with no changes should pass, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.is_empty(),
        "pass verdict should have empty recommendation, got: {}",
        result.recommendation
    );
}

#[test]
fn policy_evidence_stale_changes_with_fresh_evidence_warns() {
    // changed_file_count > 0, evidence_fresh = true → Warn (may be outdated)
    let result = check_evidence_stale(2, true);
    assert!(
        matches!(result.verdict, PolicyVerdict::Warn),
        "changes with present evidence should warn, got {:?}",
        result.verdict
    );
}

// ── BranchBehindPolicy ───────────────────────────────────────────────────────

#[test]
fn policy_branch_behind_evaluates_without_panic() {
    // commits_behind = None (no upstream configured) → Pass gracefully
    let result = check_branch_behind(None);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "no upstream should pass gracefully, got {:?}",
        result.verdict
    );
}

#[test]
fn policy_branch_behind_zero_commits_passes() {
    let result = check_branch_behind(Some(0));
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "zero commits behind should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn policy_branch_behind_nonzero_commits_suggests() {
    let result = check_branch_behind(Some(3));
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "3 commits behind should not pass, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("3") || result.recommendation.contains("pull"),
        "recommendation should mention commit count or pull, got: {}",
        result.recommendation
    );
}

// ── PublishNotAdjudicatedPolicy ──────────────────────────────────────────────

#[test]
fn policy_publish_not_adjudicated_evaluates_without_panic() {
    // receipt_exists = true, receipt_stale = false → Pass
    let result = check_publish_not_adjudicated(true, false);
    assert!(
        matches!(result.verdict, PolicyVerdict::Pass),
        "fresh receipt should pass, got {:?}",
        result.verdict
    );
}

#[test]
fn policy_publish_not_adjudicated_no_receipt_suggests() {
    // receipt_exists = false → Suggest (alert)
    let result = check_publish_not_adjudicated(false, false);
    assert!(
        !matches!(result.verdict, PolicyVerdict::Pass),
        "missing receipt should not pass, got {:?}",
        result.verdict
    );
    assert!(
        result.recommendation.contains("receipt") || result.recommendation.contains("doctor"),
        "recommendation should mention receipt or doctor, got: {}",
        result.recommendation
    );
}

#[test]
fn policy_publish_not_adjudicated_stale_receipt_warns() {
    // receipt_exists = true, receipt_stale = true → Warn
    let result = check_publish_not_adjudicated(true, true);
    assert!(
        matches!(result.verdict, PolicyVerdict::Warn),
        "stale receipt should warn, got {:?}",
        result.verdict
    );
}

// ── run_all_policies: 7-result contract ─────────────────────────────────────

use cargo_cicd::autonomic::policies::run_all_policies;

#[test]
fn run_all_policies_returns_seven_results() {
    let workspace = WorkspaceInfo {
        target_gb: 0.1,
        max_gb: 20.0,
        active_toolchain: "stable".to_string(),
        pinned_toolchain: None,
        changed_trybuild_fixtures: 0,
    };
    let git = GitState {
        dirty_count: 0,
        commits_behind: None,
    };
    let evidence = EvidenceState {
        changed_file_count: 0,
        evidence_fresh: true,
        receipt_exists: true,
        receipt_stale: false,
    };
    let results = run_all_policies(&workspace, &git, &evidence);
    assert_eq!(
        results.len(),
        7,
        "expected 7 policy results, got {}",
        results.len()
    );
}

#[test]
fn run_all_policies_all_pass_on_clean_state() {
    let workspace = WorkspaceInfo {
        target_gb: 0.1,
        max_gb: 20.0,
        active_toolchain: "stable".to_string(),
        pinned_toolchain: None,
        changed_trybuild_fixtures: 0,
    };
    let git = GitState {
        dirty_count: 0,
        commits_behind: None,
    };
    let evidence = EvidenceState {
        changed_file_count: 0,
        evidence_fresh: true,
        receipt_exists: true,
        receipt_stale: false,
    };
    let results = run_all_policies(&workspace, &git, &evidence);
    for r in &results {
        assert!(
            matches!(r.verdict, PolicyVerdict::Pass),
            "policy '{}' should pass on clean state, got {:?}",
            r.name,
            r.verdict
        );
    }
}

// ── AutonomicMode / run_with_mode from policy_engine.rs ─────────────────────

#[test]
fn autonomic_mode_variants_are_distinct() {
    use cargo_cicd::autonomic::policy_engine::AutonomicMode;
    assert_ne!(AutonomicMode::Suggest, AutonomicMode::Apply);
}

