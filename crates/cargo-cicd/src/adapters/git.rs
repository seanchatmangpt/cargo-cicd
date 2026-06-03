use crate::state::git_phase::GitPhaseState;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A flat snapshot of relevant git state for CI/CD decisions.
#[derive(Debug, Clone)]
pub struct GitState {
    /// Current branch name.
    pub branch: String,
    /// Unstaged modified/deleted files.
    pub dirty: Vec<String>,
    /// Staged files (index vs HEAD).
    pub staged: Vec<String>,
    /// Untracked files not ignored.
    pub untracked: Vec<String>,
    /// Commits ahead of upstream.
    pub ahead: usize,
    /// Commits behind upstream.
    pub behind: usize,
}

/// Collect current git state from the working directory.
pub fn read_git_state() -> Result<GitState> {
    let branch = {
        let out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let (dirty, staged, untracked) = {
        let out = Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;
        let mut dirty = Vec::new();
        let mut staged = Vec::new();
        let mut untracked = Vec::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.len() < 3 {
                continue;
            }
            let x = line.chars().next().unwrap_or(' ');
            let y = line.chars().nth(1).unwrap_or(' ');
            let file = line[3..].to_string();
            match (x, y) {
                ('?', '?') => untracked.push(file),
                (' ', _) => dirty.push(file),
                (_, ' ') => staged.push(file),
                _ => {
                    staged.push(file.clone());
                    dirty.push(file);
                }
            }
        }
        (dirty, staged, untracked)
    };

    let (ahead, behind) = {
        let out = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .output()?;
        let s = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() == 2 {
            (
                parts[0].parse::<usize>().unwrap_or(0),
                parts[1].parse::<usize>().unwrap_or(0),
            )
        } else {
            (0, 0)
        }
    };

    Ok(GitState {
        branch,
        dirty,
        staged,
        untracked,
        ahead,
        behind,
    })
}

/// List files changed between `base` and HEAD using `git diff --name-only <base>`.
pub fn changed_files_since(base: &str) -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["diff", "--name-only", base])
        .output()?;
    let files = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| PathBuf::from(l.trim()))
        .collect();
    Ok(files)
}

/// Read the current git phase state from the working directory.
pub fn read_git_phase(root: &Path) -> Result<GitPhaseState> {
    let branch = git_branch(root).unwrap_or_else(|_| "unknown".to_string());
    let dirty = git_dirty_files(root).unwrap_or_default();
    let staged = git_staged_files(root).unwrap_or_default();
    let untracked = git_untracked_files(root).unwrap_or_default();
    let (ahead, behind) = git_ahead_behind(root).unwrap_or((0, 0));

    let recommended_action = if !dirty.is_empty() || !staged.is_empty() {
        "commit or stash pending changes".to_string()
    } else {
        "tree is clean".to_string()
    };

    Ok(GitPhaseState {
        branch,
        dirty_files: dirty,
        staged_files: staged,
        untracked,
        ahead,
        behind,
        recommended_action,
    })
}

fn git_branch(root: &Path) -> Result<String> {
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
        .output()?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_dirty_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let out = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "diff", "--name-only"])
        .output()?;
    let files = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| std::path::PathBuf::from(l.trim()))
        .collect();
    Ok(files)
}

fn git_staged_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "diff",
            "--name-only",
            "--cached",
        ])
        .output()?;
    let files = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| std::path::PathBuf::from(l.trim()))
        .collect();
    Ok(files)
}

fn git_untracked_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "ls-files",
            "--others",
            "--exclude-standard",
        ])
        .output()?;
    let files = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| std::path::PathBuf::from(l.trim()))
        .collect();
    Ok(files)
}

fn git_ahead_behind(root: &Path) -> Result<(usize, usize)> {
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "rev-list",
            "--left-right",
            "--count",
            "HEAD...@{upstream}",
        ])
        .output()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 2 {
        let ahead = parts[0].parse().unwrap_or(0);
        let behind = parts[1].parse().unwrap_or(0);
        Ok((ahead, behind))
    } else {
        Ok((0, 0))
    }
}
