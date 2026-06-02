use anyhow::Result;
use std::path::PathBuf;

/// Verbs for `cargo cicd git`.
#[derive(clap::Subcommand, Debug)]
pub enum GitVerb {
    /// Show branch, dirty/staged/untracked counts, ahead/behind, and recommended action.
    Status(GitStatusArgs),
    /// Verify clean, stage outputs, commit with message, record event.
    Close(GitCloseArgs),
}

/// Arguments for `cargo cicd git status`.
#[derive(clap::Args, Debug)]
pub struct GitStatusArgs {
    /// Emit output as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `cargo cicd git close`.
#[derive(clap::Args, Debug)]
pub struct GitCloseArgs {
    /// Commit message.
    #[arg(long, short = 'm')]
    pub message: String,
    /// Files to stage before committing.
    #[arg(long)]
    pub stage: Vec<PathBuf>,
    /// Allow committing with unstaged changes present.
    #[arg(long)]
    pub allow_dirty: bool,
}

pub fn run(verb: &GitVerb) -> Result<()> {
    match verb {
        GitVerb::Status(args) => run_status(args),
        GitVerb::Close(args) => run_close(args),
    }
}

fn run_status(args: &GitStatusArgs) -> Result<()> {
    let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch.trim();
    let status_raw = git_output(&["status", "--porcelain"])?;
    let dirty: usize = status_raw.lines()
        .filter(|l| l.len() >= 2 && &l[1..2] != " " && &l[0..1] == " ")
        .count();
    let staged: usize = status_raw.lines()
        .filter(|l| l.len() >= 2 && &l[0..1] != " " && !l.starts_with("??"))
        .count();
    let untracked: usize = status_raw.lines().filter(|l| l.starts_with("??")).count();
    let (ahead, behind) = ahead_behind();
    let recommended = if dirty == 0 && staged == 0 {
        "tree is clean"
    } else if staged > 0 {
        "staged changes ready to commit"
    } else {
        "commit or stash pending changes"
    };
    if args.json {
        println!(
            "{{\"branch\":{:?},\"dirty\":{},\"staged\":{},\"untracked\":{},\"ahead\":{},\"behind\":{},\"recommended_action\":{:?}}}",
            branch, dirty, staged, untracked, ahead, behind, recommended
        );
    } else {
        println!("branch    : {}", branch);
        println!("dirty     : {}", dirty);
        println!("staged    : {}", staged);
        println!("untracked : {}", untracked);
        println!("ahead     : {}", ahead);
        println!("behind    : {}", behind);
        println!("action    : {}", recommended);
    }
    Ok(())
}

fn run_close(args: &GitCloseArgs) -> Result<()> {
    let status_raw = git_output(&["status", "--porcelain"])?;
    let has_unstaged = status_raw.lines().any(|l| {
        l.len() >= 2 && &l[0..1] == " " && &l[1..2] != " "
    });
    if has_unstaged && !args.allow_dirty {
        anyhow::bail!(
            "working tree has unstaged changes — commit, stash, or pass --allow-dirty"
        );
    }
    for path in &args.stage {
        let p = path.to_string_lossy();
        println!("staging: {}", p);
        let s = std::process::Command::new("git").args(["add", p.as_ref()]).status()?;
        if !s.success() {
            anyhow::bail!("git add {} failed", p);
        }
    }
    println!("committing: {}", args.message);
    let s = std::process::Command::new("git")
        .args(["commit", "-m", &args.message])
        .status()?;
    if !s.success() {
        anyhow::bail!("git commit failed");
    }
    if std::path::Path::new("cicd.toml").exists() {
        record_close_event(&args.message)?;
    }
    println!("closed: commit recorded");
    Ok(())
}

fn record_close_event(message: &str) -> Result<()> {
    let content = std::fs::read_to_string("cicd.toml")?;
    let mut config: cargo_cicd::CicdToml = toml::from_str(&content)?;
    config.events.push(cargo_cicd::cicd_toml::EventRecord {
        kind: "git.close".into(),
        verdict: "pass".into(),
        details: Some(message.to_string()),
        timestamp: None,
    });
    let updated = toml::to_string_pretty(&config)?;
    std::fs::write("cicd.toml", updated)?;
    Ok(())
}

fn git_output(args: &[&str]) -> Result<String> {
    let out = std::process::Command::new("git").args(args).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn ahead_behind() -> (usize, usize) {
    std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            let parts: Vec<&str> = s.trim().split_whitespace().collect();
            if parts.len() == 2 {
                Some((parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0)))
            } else {
                None
            }
        })
        .unwrap_or((0, 0))
}
