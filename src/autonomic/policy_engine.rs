use crate::engine::EngineState;
use crate::policies::{
    BranchBehindPolicy, CicdPolicy, EvidenceStalePoliciy, GitPhaseDirtyPolicy,
    PublishNotAdjudicatedPolicy, TargetPressurePolicy, ToolchainMismatchPolicy,
    TrybuildChangedPolicy,
};
use crate::state::policy::{PolicyMode, PolicyState, PolicyVerdict};

/// Operating mode for the autonomic engine.
///
/// - `Suggest` — print recommendations only; never mutate workspace state.
/// - `Apply`   — execute safe remediation commands automatically where permitted.
#[derive(Debug, Clone, PartialEq)]
pub enum AutonomicMode {
    /// Print recommendations only (default).
    Suggest,
    /// Execute safe remediation commands automatically.
    Apply,
}

/// Evaluate a single policy against a set of signals.
pub fn evaluate_policy(name: &str, mode: PolicyMode, signals: Vec<String>) -> PolicyState {
    let (verdict, recommendation) = match mode {
        PolicyMode::Disabled => (PolicyVerdict::Pass, "policy disabled".to_string()),
        PolicyMode::Suggest | PolicyMode::Apply => {
            if signals.is_empty() {
                (PolicyVerdict::Pass, "no signals — clean".to_string())
            } else {
                let rec = format!(
                    "address {} signal(s): {}",
                    signals.len(),
                    signals.join(", ")
                );
                (PolicyVerdict::Warn, rec)
            }
        }
    };

    PolicyState {
        name: name.to_string(),
        mode,
        signals,
        recommendation,
        verdict,
    }
}

/// Collect all active policy recommendations from the full policy registry.
///
/// Returns one string per non-passing policy, suitable for display.
pub fn run_suggestions(state: &EngineState) -> Vec<String> {
    run_with_mode(state, AutonomicMode::Suggest)
}

/// Run all registered policies under the given `mode` and return
/// human-readable recommendation strings for every non-passing result.
///
/// In `Apply` mode, safe automated remediation is attempted for eligible
/// policies before the recommendation list is returned:
///
/// - `evidence_stale`          → runs `cargo cicd test changed` subprocess
/// - `branch_behind`           → warning only (never auto-pull)
/// - `publish_not_adjudicated` → warning only
pub fn run_with_mode(state: &EngineState, mode: AutonomicMode) -> Vec<String> {
    let policies: Vec<Box<dyn CicdPolicy>> = vec![
        Box::new(GitPhaseDirtyPolicy),
        Box::new(TargetPressurePolicy::default()),
        Box::new(ToolchainMismatchPolicy),
        Box::new(TrybuildChangedPolicy),
        Box::new(BranchBehindPolicy),
        Box::new(EvidenceStalePoliciy),
        Box::new(PublishNotAdjudicatedPolicy),
    ];

    let mut suggestions = Vec::new();

    for policy in &policies {
        if !policy.enabled() {
            continue;
        }

        let result = policy.evaluate(state);
        let non_passing = result.verdict != "pass";

        if non_passing {
            if mode == AutonomicMode::Apply {
                apply_safe_remediation(policy.name(), &result.recommendation);
            }

            if let Some(rec) = result.recommendation {
                suggestions.push(format!("[{}] {}", result.verdict, rec));
            }
        }
    }

    suggestions
}

/// Attempt safe automated remediation for eligible policies.
///
/// Only `evidence_stale` triggers an actual subprocess. All other policies
/// emit a warning-level notice and leave the workspace unchanged.
fn apply_safe_remediation(policy_name: &str, recommendation: &Option<String>) {
    match policy_name {
        "evidence_stale" => {
            eprintln!(
                "autonomic apply: running 'cargo cicd test changed' for evidence_stale policy"
            );
            let status = std::process::Command::new("cargo")
                .args(["cicd", "test", "changed"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    eprintln!("autonomic apply: 'cargo cicd test changed' succeeded");
                }
                Ok(s) => {
                    eprintln!(
                        "autonomic apply: 'cargo cicd test changed' exited with {}",
                        s
                    );
                }
                Err(e) => {
                    eprintln!(
                        "autonomic apply: failed to spawn 'cargo cicd test changed': {}",
                        e
                    );
                }
            }
        }
        "branch_behind" => {
            eprintln!(
                "autonomic apply: branch_behind — auto-pull disabled; {}",
                recommendation
                    .as_deref()
                    .unwrap_or("run git pull --rebase manually")
            );
        }
        "publish_not_adjudicated" => {
            eprintln!(
                "autonomic apply: publish_not_adjudicated — {}",
                recommendation
                    .as_deref()
                    .unwrap_or("run cargo cicd evidence doctor manually")
            );
        }
        _ => {
            // Other policies have no apply-mode handler; suggestions are returned only.
        }
    }
}
