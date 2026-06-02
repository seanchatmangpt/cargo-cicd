use anyhow::Result;

/// Verbs for `cargo cicd trybuild`.
#[derive(clap::Subcommand, Debug)]
pub enum TrybuildVerb {
    /// Detect changed trybuild fixtures (avoids all-fixture run by default).
    Changed(TrybuildChangedArgs),
}

/// Arguments for `cargo cicd trybuild changed`.
#[derive(clap::Args, Debug)]
pub struct TrybuildChangedArgs {
    /// Base branch or commit to diff against.
    #[arg(long, default_value = "origin/main")]
    pub base: String,
    /// Run the trybuild test suite for the changed fixtures.
    #[arg(long)]
    pub run: bool,
}

pub fn run(verb: &TrybuildVerb) -> Result<()> {
    match verb {
        TrybuildVerb::Changed(args) => run_changed(args),
    }
}

fn run_changed(args: &TrybuildChangedArgs) -> Result<()> {
    let all_changed = changed_files(&args.base);
    let fixtures: Vec<String> = all_changed.into_iter().filter(|f| is_trybuild_fixture(f)).collect();
    if fixtures.is_empty() {
        println!("no changed trybuild fixtures relative to {}", args.base);
        return Ok(());
    }
    println!("changed trybuild fixtures ({}):", fixtures.len());
    for f in &fixtures {
        println!("  {}", f);
    }
    if !args.run {
        println!();
        println!("pass --run to execute the ALIVE gate for these fixtures");
        return Ok(());
    }
    println!();
    println!("running: cargo test --test ui_tests -- --ignored");
    let status = std::process::Command::new("cargo")
        .args(["test", "--test", "ui_tests", "--", "--ignored"])
        .status()?;
    if !status.success() {
        anyhow::bail!("trybuild ui_tests failed");
    }
    println!("trybuild: PASS");
    Ok(())
}

fn changed_files(base: &str) -> Vec<String> {
    std::process::Command::new("git")
        .args(["diff", "--name-only", base])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

fn is_trybuild_fixture(path: &str) -> bool {
    let in_tests = path.contains("/tests/") || path.contains("\\tests\\");
    let is_rs_or_ref = path.ends_with(".rs") || path.ends_with(".stderr") || path.ends_with(".stdout");
    let in_ui = path.contains("compile_fail") || path.contains("trybuild") || path.contains("/ui/");
    in_tests && is_rs_or_ref && in_ui
}
