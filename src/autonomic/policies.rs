use serde::{Deserialize, Serialize};

/// Policy evaluation mode. Only `Suggest` is active by default; `Apply` is reserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyMode {
    Suggest,
    Apply,
}

/// Outcome of a single policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Suggest,
}

/// Result record returned by each policy check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub name: String,
    pub enabled: bool,
    pub mode: PolicyMode,
    pub verdict: PolicyVerdict,
    pub recommendation: String,
    pub event: String,
}

/// Autonomic policy marker — zero-sized type carrying policy identity.
pub struct AutomicPolicy;

/// Workspace snapshot passed to `run_all_policies`.
pub struct WorkspaceInfo {
    /// Current size of the `target/` directory in gigabytes.
    pub target_gb: f64,
    /// Maximum allowed target directory size in gigabytes.
    pub max_gb: f64,
    /// Active rustup toolchain string (e.g. `"nightly-2026-05-30"`).
    pub active_toolchain: String,
    /// Channel pinned in `rust-toolchain.toml`, if present.
    pub pinned_toolchain: Option<String>,
    /// Number of trybuild fixture files that have changed since last commit.
    pub changed_trybuild_fixtures: usize,
}

/// Git repository snapshot passed to `run_all_policies`.
pub struct GitState {
    /// Number of files with uncommitted modifications (dirty working tree).
    pub dirty_count: usize,
}

// ── individual policy checks ─────────────────────────────────────────────────

/// Evaluate target-directory pressure against `max_gb`.
///
/// - `target_gb > max_gb`       → Suggest pruning
/// - `target_gb > max_gb * 0.8` → Warn approaching limit
/// - otherwise                  → Pass
pub fn check_target_pressure(target_gb: f64, max_gb: f64) -> PolicyResult {
    let (verdict, recommendation) = if target_gb > max_gb {
        (
            PolicyVerdict::Suggest,
            "Run cargo cicd target prune to reclaim disk space".to_string(),
        )
    } else if target_gb > max_gb * 0.8 {
        (
            PolicyVerdict::Warn,
            "Target directory approaching limit".to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyResult {
        name: "target_pressure".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "target_pressure_check".to_string(),
    }
}

/// Evaluate whether the active toolchain matches the pinned toolchain.
///
/// If `pinned` is `Some` and differs from `active` → Suggest pinning.
/// Otherwise → Pass.
pub fn check_toolchain_mismatch(active: &str, pinned: Option<&str>) -> PolicyResult {
    let (verdict, recommendation) = match pinned {
        Some(p) if active != p => (
            PolicyVerdict::Suggest,
            "Pin toolchain in rust-toolchain.toml".to_string(),
        ),
        _ => (PolicyVerdict::Pass, String::new()),
    };

    PolicyResult {
        name: "toolchain_mismatch".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "toolchain_mismatch_check".to_string(),
    }
}

/// Evaluate whether any trybuild fixtures have changed and need re-running.
///
/// `changed_fixtures > 0` → Suggest running trybuild for changed fixtures.
/// Otherwise → Pass.
pub fn check_trybuild_changed(changed_fixtures: usize) -> PolicyResult {
    let (verdict, recommendation) = if changed_fixtures > 0 {
        (
            PolicyVerdict::Suggest,
            format!(
                "Run cargo cicd trybuild changed ({} fixtures changed)",
                changed_fixtures
            ),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyResult {
        name: "trybuild_changed".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "trybuild_changed_check".to_string(),
    }
}

/// Evaluate whether the git working tree has uncommitted changes.
///
/// `dirty_count > 0` → Suggest committing dirty files.
/// Otherwise → Pass.
pub fn check_git_phase_dirty(dirty_count: usize) -> PolicyResult {
    let (verdict, recommendation) = if dirty_count > 0 {
        (
            PolicyVerdict::Suggest,
            format!(
                "Run cargo cicd git close to commit {} dirty files",
                dirty_count
            ),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyResult {
        name: "git_phase_dirty".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "git_phase_dirty_check".to_string(),
    }
}

// ── aggregate runner ─────────────────────────────────────────────────────────

/// Run all four suggest-mode policies and return results.
///
/// All policies run in `Suggest` mode. No apply-mode mutations occur here.
pub fn run_all_policies(workspace: &WorkspaceInfo, git: &GitState) -> Vec<PolicyResult> {
    vec![
        check_target_pressure(workspace.target_gb, workspace.max_gb),
        check_toolchain_mismatch(
            &workspace.active_toolchain,
            workspace.pinned_toolchain.as_deref(),
        ),
        check_trybuild_changed(workspace.changed_trybuild_fixtures),
        check_git_phase_dirty(git.dirty_count),
    ]
}
