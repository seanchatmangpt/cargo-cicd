//! Policy: warn when the local branch is behind its upstream.
//!
//! Being behind remote means a push will be rejected and a pull is needed
//! before work can be integrated.  This policy surfaces that situation early
//! so it is not discovered at push-time.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "branch_behind";

/// Evaluate whether the local branch is up-to-date with its upstream.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | No upstream configured (`has_upstream = false`) | `Skip` |
/// | `behind == 0` | `Pass` |
/// | `behind > 0` | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    // Without an upstream we cannot know whether we are behind.
    if !state.git.has_upstream {
        return PolicyEntry::skip(POLICY_NAME);
    }

    if state.git.behind == 0 {
        return PolicyEntry::pass(POLICY_NAME);
    }

    let behind = state.git.behind;

    PolicyEntry {
        policy_name: POLICY_NAME.to_string(),
        verdict: PolicyVerdict::Warn,
        recommendation: format!(
            "Your branch is {behind} commit(s) behind remote. \
             Run `git pull --rebase`"
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
    fn skip_when_no_upstream() {
        let state = state_with_git(GitState {
            has_upstream: false,
            behind: 3,
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn pass_when_up_to_date() {
        let state = state_with_git(GitState {
            has_upstream: true,
            behind: 0,
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_when_behind_one_commit() {
        let state = state_with_git(GitState {
            has_upstream: true,
            behind: 1,
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("1 commit(s) behind"));
    }

    #[test]
    fn warn_when_behind_many_commits() {
        let state = state_with_git(GitState {
            has_upstream: true,
            behind: 42,
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("42 commit(s) behind"));
    }

    #[test]
    fn recommendation_contains_git_pull_rebase() {
        let state = state_with_git(GitState {
            has_upstream: true,
            behind: 2,
            ..Default::default()
        });
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("git pull --rebase"),
            "expected git pull hint: {}",
            entry.recommendation
        );
    }

    #[test]
    fn ahead_does_not_affect_this_policy() {
        // Being ahead is fine — we only care about behind.
        let state = state_with_git(GitState {
            has_upstream: true,
            ahead: 10,
            behind: 0,
            ..Default::default()
        });
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }
}
