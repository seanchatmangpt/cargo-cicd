//! Policy: warn when the working tree has a mixed staged/unstaged state.
//!
//! Having both staged *and* unstaged (dirty) changes simultaneously is a messy
//! pre-commit state.  It means a `git commit` would create a partial commit
//! that does not capture all in-progress work.  The developer likely needs to
//! review what is staged before proceeding.
//!
//! Note: this policy is more specific than `git_phase_dirty`.
//! `git_phase_dirty` warns on *any* dirty or staged file.
//! This policy only warns when *both* categories are present simultaneously.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "uncommitted_changes";

/// Evaluate whether the working tree is in a mixed staged/unstaged state.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | No staged files | `Pass` (single-category or clean state) |
/// | No dirty files | `Pass` (only staged — intentional `git add`) |
/// | Both staged AND dirty files present | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    let has_staged = !state.git.staged_files.is_empty();
    let has_dirty = !state.git.dirty_files.is_empty();

    if has_staged && has_dirty {
        let staged = state.git.staged_files.len();
        let dirty = state.git.dirty_files.len();

        return PolicyEntry {
            policy_name: POLICY_NAME.to_string(),
            verdict: PolicyVerdict::Warn,
            recommendation: format!(
                "Mixed staged/unstaged changes detected ({staged} staged, {dirty} unstaged). \
                 Run `git diff --cached` to review staged changes"
            ),
            emitted_at: now_iso8601(),
        };
    }

    PolicyEntry::pass(POLICY_NAME)
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
    fn pass_when_completely_clean() {
        let state = state_with_git(GitState::default());
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn pass_when_only_staged() {
        let state = state_with_git(GitState {
            staged_files: vec!["src/lib.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn pass_when_only_dirty() {
        let state = state_with_git(GitState {
            dirty_files: vec!["src/main.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_when_both_staged_and_dirty() {
        let state = state_with_git(GitState {
            staged_files: vec!["a.rs".to_string()],
            dirty_files: vec!["b.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
    }

    #[test]
    fn recommendation_contains_git_diff_cached() {
        let state = state_with_git(GitState {
            staged_files: vec!["a.rs".to_string()],
            dirty_files: vec!["b.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("git diff --cached"),
            "expected git diff --cached hint: {}",
            entry.recommendation
        );
    }

    #[test]
    fn recommendation_reports_counts() {
        let state = state_with_git(GitState {
            staged_files: vec!["a.rs".to_string(), "b.rs".to_string()],
            dirty_files: vec!["c.rs".to_string()],
            ..Default::default()
        });
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("2 staged"),
            "expected 2 staged in: {}",
            entry.recommendation
        );
        assert!(
            entry.recommendation.contains("1 unstaged"),
            "expected 1 unstaged in: {}",
            entry.recommendation
        );
    }
}
