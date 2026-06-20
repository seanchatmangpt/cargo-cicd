//! Policy: warn when the working tree has dirty or staged files.
//!
//! A workspace with modified or staged files is not push-ready.  This policy
//! catches the most common pre-push oversight: forgetting to commit work in
//! progress.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "git_phase_dirty";

/// Evaluate whether the working tree is clean.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | No dirty or staged files | `Pass` |
/// | One or more dirty OR staged files | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    let dirty_count = state.git.dirty_files.len();
    let staged_count = state.git.staged_files.len();

    if dirty_count == 0 && staged_count == 0 {
        return PolicyEntry::pass(POLICY_NAME);
    }

    let detail = match (dirty_count, staged_count) {
        (d, 0) => format!("{d} dirty file(s)"),
        (0, s) => format!("{s} staged file(s)"),
        (d, s) => format!("{d} dirty and {s} staged file(s)"),
    };

    PolicyEntry {
        policy_name: POLICY_NAME.to_string(),
        verdict: PolicyVerdict::Warn,
        recommendation: format!(
            "Working tree has {detail}. \
             Commit or stash changes before pushing: `git stash` or `git commit -am 'wip'`"
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

    fn state_with_git(git: GitState) -> EngineState {
        EngineState {
            git,
            ..Default::default()
        }
    }

    #[test]
    fn pass_when_clean() {
        let state = state_with_git(GitState::default());
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
        assert_eq!(entry.policy_name, POLICY_NAME);
    }

    #[test]
    fn warn_when_dirty_files_present() {
        let state = state_with_git(GitState {
            dirty_files: vec!["src/main.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("dirty"));
    }

    #[test]
    fn warn_when_staged_files_present() {
        let state = state_with_git(GitState {
            staged_files: vec!["src/lib.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("staged"));
    }

    #[test]
    fn warn_when_both_dirty_and_staged() {
        let state = state_with_git(GitState {
            dirty_files: vec!["a.rs".to_string()],
            staged_files: vec!["b.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("dirty"));
        assert!(entry.recommendation.contains("staged"));
    }

    #[test]
    fn recommendation_contains_git_stash_hint() {
        let state = state_with_git(GitState {
            dirty_files: vec!["x.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("git stash"),
            "expected git stash hint in: {}",
            entry.recommendation
        );
    }

    #[test]
    fn untracked_files_alone_do_not_trigger_this_policy() {
        // git_phase_dirty only cares about dirty + staged, not untracked.
        let state = state_with_git(GitState {
            untracked_files: vec!["new_file.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }
}
