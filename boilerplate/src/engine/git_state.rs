//! Git repository state dimension.

/// Git repository health snapshot.
#[derive(Debug, Clone, Default)]
pub struct GitState {
    /// Current branch name.  Empty string if HEAD is detached.
    pub branch: String,
    /// Files with unstaged modifications.
    pub dirty_files: Vec<String>,
    /// Files staged for commit.
    pub staged_files: Vec<String>,
    /// Files not tracked by git.
    pub untracked_files: Vec<String>,
    /// Commits the local branch is ahead of its upstream.
    pub ahead: u32,
    /// Commits the local branch is behind its upstream.
    pub behind: u32,
    /// `true` when an upstream tracking branch is configured.
    pub has_upstream: bool,
}

impl GitState {
    /// Returns `true` when the working tree is perfectly clean.
    pub fn is_clean(&self) -> bool {
        self.dirty_files.is_empty()
            && self.staged_files.is_empty()
            && self.untracked_files.is_empty()
    }

    /// Total count of files that differ from HEAD in any way.
    pub fn changed_count(&self) -> usize {
        self.dirty_files.len() + self.staged_files.len() + self.untracked_files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_state() {
        let state = GitState::default();
        assert!(state.is_clean());
        assert_eq!(state.changed_count(), 0);
    }

    #[test]
    fn dirty_state() {
        let state = GitState {
            dirty_files: vec!["src/main.rs".to_owned()],
            ..Default::default()
        };
        assert!(!state.is_clean());
        assert_eq!(state.changed_count(), 1);
    }
}
