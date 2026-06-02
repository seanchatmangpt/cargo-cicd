use anyhow::Result;
use std::process::Command;

pub struct GitStatusAdapter;

impl GitStatusAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn query() -> Result<GitStatusResult> {
        let mut result = GitStatusResult::default();

        let branch_out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        result.branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();

        let status_out = Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;
        let status = String::from_utf8_lossy(&status_out.stdout);
        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }
            let xy = &line[..2];
            let file = line[3..].to_string();
            let x = xy.chars().next();
            let y = xy.chars().nth(1);
            match (x, y) {
                (Some(' '), Some('M')) | (Some(' '), Some('D')) => result.dirty_files.push(file),
                (Some('M'), Some('M')) => {
                    result.staged_files.push(file.clone());
                    result.dirty_files.push(file);
                }
                (Some('M'), _) | (Some('A'), _) | (Some('D'), _) => {
                    result.staged_files.push(file)
                }
                (Some('?'), Some('?')) => result.untracked_files.push(file),
                _ => {}
            }
        }

        Ok(result)
    }

    pub fn is_dirty() -> bool {
        Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(true)
    }
}

impl Default for GitStatusAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct GitStatusResult {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}
