//! Policy: warn when many untracked files are present.
//!
//! A small number of untracked files is normal (build artefacts before
//! `.gitignore` is tuned, new files being worked on).  More than 5 untracked
//! files suggests that `.gitignore` is missing entries or that generated files
//! are leaking into the working tree.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "untracked_files";

/// Threshold: more than this many untracked files triggers a warning.
const UNTRACKED_THRESHOLD: usize = 5;

/// Evaluate the number of untracked files.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | `untracked_files.len() <= UNTRACKED_THRESHOLD` | `Pass` |
/// | `untracked_files.len() > UNTRACKED_THRESHOLD` | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    let n = state.git.untracked_files.len();

    if n <= UNTRACKED_THRESHOLD {
        return PolicyEntry::pass(POLICY_NAME);
    }

    PolicyEntry {
        policy_name: POLICY_NAME.to_string(),
        verdict: PolicyVerdict::Warn,
        recommendation: format!(
            "Found {n} untracked files — review with `git status` and add patterns \
             to .gitignore if appropriate"
        ),
        emitted_at: now_iso8601(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::policy_engine::PolicyVerdict;
    use crate::engine::{EngineState, GitState};

    fn state_with_untracked(n: usize) -> EngineState {
        EngineState {
            git: GitState {
                untracked_files: (0..n).map(|i| format!("file_{i}.rs")).collect(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn pass_when_no_untracked_files() {
        let state = state_with_untracked(0);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn pass_at_exactly_threshold() {
        let state = state_with_untracked(UNTRACKED_THRESHOLD);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_at_one_over_threshold() {
        let state = state_with_untracked(UNTRACKED_THRESHOLD + 1);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
    }

    #[test]
    fn warn_with_many_untracked_files() {
        let state = state_with_untracked(20);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("20"));
    }

    #[test]
    fn recommendation_contains_git_status() {
        let state = state_with_untracked(UNTRACKED_THRESHOLD + 1);
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("git status"),
            "expected git status hint: {}",
            entry.recommendation
        );
    }

    #[test]
    fn recommendation_mentions_gitignore() {
        let state = state_with_untracked(10);
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains(".gitignore"),
            "expected .gitignore mention: {}",
            entry.recommendation
        );
    }

    #[test]
    fn policy_name_is_correct() {
        let state = state_with_untracked(10);
        let entry = eval(&state);
        assert_eq!(entry.policy_name, POLICY_NAME);
    }
}
