//! Git adapter — reads repository state via `git status --porcelain=v2`.

#![cfg(feature = "process-data")]

use crate::engine::GitState;
use std::process::Command;

/// Populates [`GitState`] from the `git` command.
pub struct GitAdapter;

impl GitAdapter {
    /// Build a [`GitState`] from the current git repository.
    ///
    /// Returns `Default` state silently if `git` is not installed or the
    /// current directory is not inside a git repository.
    pub fn populate() -> GitState {
        match Self::try_populate() {
            Ok(state) => state,
            Err(err) => {
                tracing::warn!("GitAdapter failed, using defaults: {err}");
                GitState::default()
            }
        }
    }

    fn try_populate() -> anyhow::Result<GitState> {
        let mut state = GitState::default();

        // Branch name.
        state.branch = Self::current_branch().unwrap_or_else(|| "HEAD".to_owned());

        // Dirty / staged / untracked files via porcelain format.
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            Self::parse_porcelain(&text, &mut state);
        }

        // Ahead/behind counts.
        if let Some((ahead, behind)) = Self::ahead_behind() {
            state.ahead = ahead;
            state.behind = behind;
            state.has_upstream = true;
        }

        Ok(state)
    }

    /// Returns the current branch name, or `None` on detached HEAD.
    fn current_branch() -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if name == "HEAD" {
                None // detached HEAD
            } else {
                Some(name)
            }
        } else {
            None
        }
    }

    /// Parse `git status --porcelain` output into file lists.
    ///
    /// Porcelain format: two-character status code + space + filename.
    /// - Column 1: staged status (`A`, `M`, `D`, `R`, `C`)
    /// - Column 2: unstaged/worktree status (`M`, `D`)
    /// - `??`: untracked
    fn parse_porcelain(text: &str, state: &mut GitState) {
        for line in text.lines() {
            if line.len() < 3 {
                continue;
            }
            let staged = &line[0..1];
            let unstaged = &line[1..2];
            let file = line[3..].to_owned();

            match (staged, unstaged) {
                ("?", "?") => state.untracked_files.push(file),
                (s, _) if s != " " && s != "?" => {
                    state.staged_files.push(file.clone());
                    // If column 2 also dirty, count as dirty too.
                    if unstaged != " " && unstaged != "?" {
                        state.dirty_files.push(file);
                    }
                }
                (_, u) if u != " " && u != "?" => state.dirty_files.push(file),
                _ => {}
            }
        }
    }

    /// Returns `(ahead, behind)` relative to the tracking upstream, or `None`.
    fn ahead_behind() -> Option<(u32, u32)> {
        let output = Command::new("git")
            .args([
                "rev-list",
                "--left-right",
                "--count",
                "HEAD...@{upstream}",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() == 2 {
            let ahead = parts[0].parse().ok()?;
            let behind = parts[1].parse().ok()?;
            Some((ahead, behind))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_does_not_panic() {
        let _state = GitAdapter::populate();
    }

    #[test]
    fn parse_porcelain_untracked() {
        let mut state = GitState::default();
        GitAdapter::parse_porcelain("?? new_file.rs\n", &mut state);
        assert_eq!(state.untracked_files, ["new_file.rs"]);
        assert!(state.dirty_files.is_empty());
        assert!(state.staged_files.is_empty());
    }

    #[test]
    fn parse_porcelain_modified() {
        let mut state = GitState::default();
        GitAdapter::parse_porcelain(" M src/main.rs\n", &mut state);
        assert_eq!(state.dirty_files, ["src/main.rs"]);
        assert!(state.staged_files.is_empty());
    }

    #[test]
    fn parse_porcelain_staged() {
        let mut state = GitState::default();
        GitAdapter::parse_porcelain("A  new.rs\n", &mut state);
        assert_eq!(state.staged_files, ["new.rs"]);
        assert!(state.dirty_files.is_empty());
    }
}
