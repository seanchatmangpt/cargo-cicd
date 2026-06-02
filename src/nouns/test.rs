use anyhow::Result;

/// Verbs for `cargo cicd test`.
#[derive(clap::Subcommand, Debug)]
pub enum TestVerb {
    /// Detect changed .rs files, derive a test plan, and optionally run tests.
    Changed(TestChangedArgs),
}

/// Arguments for `cargo cicd test changed`.
#[derive(clap::Args, Debug)]
pub struct TestChangedArgs {
    /// Base branch or commit to diff against.
    #[arg(long, default_value = "origin/main")]
    pub base: String,
    /// Actually run the derived test plan (default: print plan only).
    #[arg(long)]
    pub run: bool,
    /// Pass --all-features to cargo test.
    #[arg(long)]
    pub all_features: bool,
}

pub fn run(verb: &TestVerb) -> Result<()> {
    match verb {
        TestVerb::Changed(args) => run_changed(args),
    }
}

fn run_changed(args: &TestChangedArgs) -> Result<()> {
    let changed = changed_rs_files(&args.base);
    if changed.is_empty() {
        println!("no changed .rs files relative to {}", args.base);
        return Ok(());
    }
    println!("changed .rs files ({}):", changed.len());
    let mut filters: Vec<String> = Vec::new();
    for file in &changed {
        let is_test = file.contains("/tests/")
            || file.ends_with("_test.rs")
            || file.ends_with("_tests.rs");
        let marker = if is_test { "[test]" } else { "[src]" };
        println!("  {} {}", marker, file);
        if let Some(stem) = std::path::Path::new(file).file_stem().and_then(|s| s.to_str()) {
            if stem != "mod" && stem != "lib" && stem != "main" {
                filters.push(stem.to_string());
            }
        }
    }
    println!();
    if filters.is_empty() {
        println!("test plan: no actionable test targets derived");
        return Ok(());
    }
    println!("test plan:");
    for f in &filters {
        println!("  cargo test --tests {}", f);
    }
    if !args.run {
        println!();
        println!("pass --run to execute the plan");
        return Ok(());
    }
    println!();
    for filter in &filters {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("test");
        if args.all_features {
            cmd.arg("--all-features");
        }
        cmd.arg("--tests").arg(filter);
        println!("running: cargo test --tests {}", filter);
        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("cargo test {} failed", filter);
        }
    }
    Ok(())
}

fn changed_rs_files(base: &str) -> Vec<String> {
    std::process::Command::new("git")
        .args(["diff", "--name-only", base, "--", "*.rs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect())
        .unwrap_or_default()
}
