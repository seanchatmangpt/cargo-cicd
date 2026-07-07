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

/// Workspace snapshot passed to `run_all_policies`.
pub struct WorkspaceInfo {
    /// Current size of the `target/` directory in gigabytes.
    pub target_gb: f64,
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
    /// Number of commits the local branch is behind the remote tracking branch.
    /// `None` means no upstream is configured or git is unavailable.
    pub commits_behind: Option<usize>,
}

/// Evidence snapshot passed to `run_all_policies`.
pub struct EvidenceState {
    /// Number of changed source files detected since last commit.
    pub changed_file_count: usize,
    /// Whether the events.xes evidence file is present.
    pub evidence_fresh: bool,
    /// Whether target/cargo-cicd/evidence/receipts/latest.json exists.
    pub receipt_exists: bool,
    /// Whether the receipt is older than other evidence files.
    pub receipt_stale: bool,
}

/// Searchable policy thresholds — the "hyperparameters" of the autonomic layer.
///
/// Default values reproduce the legacy hardcoded behavior exactly, so existing
/// callers that pass `PolicyConfig::default()` see zero behavior change.
/// The `autoarch tune` verb searches this space to recommend per-workspace values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyConfig {
    /// Target directory size in GB that triggers a prune suggestion.
    pub target_max_gb: f64,
    /// Fraction of `target_max_gb` at which a warning fires (0.0–1.0).
    pub target_warn_ratio: f64,
    /// Commits behind remote before sync is suggested. `0` means any lag triggers.
    pub behind_threshold: usize,
    /// Dirty files before commit is suggested. `0` means any dirt triggers.
    pub dirty_threshold: usize,
    /// Evidence age in seconds beyond which staleness is flagged.
    pub evidence_staleness_secs: u64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            target_max_gb: 20.0,
            target_warn_ratio: 0.8,
            behind_threshold: 0,
            dirty_threshold: 0,
            evidence_staleness_secs: 3600,
        }
    }
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

/// Evaluate whether evidence is stale relative to recent source changes.
///
/// - `changed_file_count > 0` and evidence not fresh → Alert
/// - `changed_file_count > 0` and evidence present   → Warn (may be outdated)
/// - otherwise                                        → Pass
pub fn check_evidence_stale(changed_file_count: usize, evidence_fresh: bool) -> PolicyResult {
    let (verdict, recommendation) = if changed_file_count > 0 && !evidence_fresh {
        (
            PolicyVerdict::Suggest,
            "evidence stale: run 'cargo cicd test changed' and 'cargo cicd workspace doctor'"
                .to_string(),
        )
    } else if changed_file_count > 0 && evidence_fresh {
        (
            PolicyVerdict::Warn,
            "source changes detected — verify evidence is current before closing".to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyResult {
        name: "evidence_stale".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "evidence_stale_check".to_string(),
    }
}

/// Evaluate whether the local branch is behind the remote tracking branch.
///
/// `commits_behind > 0` → Suggest git pull --rebase (never auto-applied).
/// `None` (no upstream / git unavailable) → Pass (graceful).
pub fn check_branch_behind(commits_behind: Option<usize>) -> PolicyResult {
    let (verdict, recommendation) = match commits_behind {
        Some(n) if n > 0 => (
            PolicyVerdict::Suggest,
            format!(
                "branch is {} commit(s) behind remote — run 'git pull --rebase' to sync",
                n
            ),
        ),
        _ => (PolicyVerdict::Pass, String::new()),
    };

    PolicyResult {
        name: "branch_behind".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "branch_behind_check".to_string(),
    }
}

/// Evaluate whether an adjudicated receipt exists and is current.
///
/// - No receipt → Alert: must run evidence doctor before publish.
/// - Receipt stale → Warn: re-run evidence doctor.
/// - Receipt fresh → Pass.
pub fn check_publish_not_adjudicated(receipt_exists: bool, receipt_stale: bool) -> PolicyResult {
    let (verdict, recommendation) = if !receipt_exists {
        (
            PolicyVerdict::Suggest,
            "no adjudicated receipt found — run 'cargo cicd evidence doctor' before publish"
                .to_string(),
        )
    } else if receipt_stale {
        (
            PolicyVerdict::Warn,
            "receipt exists but may be stale — re-run 'cargo cicd evidence doctor' to refresh"
                .to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyResult {
        name: "publish_not_adjudicated".to_string(),
        enabled: true,
        mode: PolicyMode::Suggest,
        verdict,
        recommendation,
        event: "publish_not_adjudicated_check".to_string(),
    }
}

// ── aggregate runner ─────────────────────────────────────────────────────────

/// Run all suggest-mode policies and return results.
///
/// All policies run in `Suggest` mode. No apply-mode mutations occur here.
pub fn run_all_policies(
    workspace: &WorkspaceInfo,
    git: &GitState,
    evidence: &EvidenceState,
) -> Vec<PolicyResult> {
    run_all_policies_with_config(workspace, git, evidence, &PolicyConfig::default())
}

/// Run all suggest-mode policies with explicit threshold configuration.
///
/// Used by `autoarch tune` to score candidate `PolicyConfig` values against the
/// current workspace state. The default `PolicyConfig` reproduces `run_all_policies`
/// behavior exactly. All policies remain in `Suggest` mode — no mutations.
pub fn run_all_policies_with_config(
    workspace: &WorkspaceInfo,
    git: &GitState,
    evidence: &EvidenceState,
    config: &PolicyConfig,
) -> Vec<PolicyResult> {
    vec![
        check_target_pressure(workspace.target_gb, config.target_max_gb),
        check_toolchain_mismatch(
            &workspace.active_toolchain,
            workspace.pinned_toolchain.as_deref(),
        ),
        check_trybuild_changed(workspace.changed_trybuild_fixtures),
        check_git_phase_dirty(git.dirty_count),
        check_branch_behind(git.commits_behind),
        check_evidence_stale(evidence.changed_file_count, evidence.evidence_fresh),
        check_publish_not_adjudicated(evidence.receipt_exists, evidence.receipt_stale),
    ]
}
