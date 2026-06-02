use anyhow::Result;

/// Verbs for `cargo cicd target`.
#[derive(clap::Subcommand, Debug)]
pub enum TargetVerb {
    /// Report target directory path, size, and verdict.
    Show(TargetShowArgs),
    /// Prune the target directory (plan-only unless --confirm is given).
    Prune(TargetPruneArgs),
}

/// Arguments for `cargo cicd target show`.
#[derive(clap::Args, Debug)]
pub struct TargetShowArgs {
    /// Maximum acceptable size in GB (used for verdict).
    #[arg(long, default_value_t = 20)]
    pub max_gb: u32,
}

/// Arguments for `cargo cicd target prune`.
#[derive(clap::Args, Debug)]
pub struct TargetPruneArgs {
    /// Actually perform the prune; without this flag, only a plan is printed.
    #[arg(long)]
    pub confirm: bool,
    /// Maximum acceptable size in GB; warn if target exceeds this.
    #[arg(long, default_value_t = 20)]
    pub max_gb: u32,
}

pub fn run(verb: &TargetVerb) -> Result<()> {
    match verb {
        TargetVerb::Show(args) => run_show(args),
        TargetVerb::Prune(args) => run_prune(args),
    }
}

fn run_show(args: &TargetShowArgs) -> Result<()> {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let size_gb = total_size_gb(&target_dir);
    let verdict = verdict(size_gb, args.max_gb as f64);
    println!("path    : {}", target_dir);
    println!("size    : {:.2} GB", size_gb);
    println!("max     : {} GB", args.max_gb);
    println!("verdict : {}", verdict);
    Ok(())
}

fn run_prune(args: &TargetPruneArgs) -> Result<()> {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let size_gb = total_size_gb(&target_dir);
    let verdict = verdict(size_gb, args.max_gb as f64);
    println!("target  : {}", target_dir);
    println!("size    : {:.2} GB", size_gb);
    println!("verdict : {}", verdict);
    if !args.confirm {
        println!("plan    : would run `cargo clean` (pass --confirm to execute)");
        return Ok(());
    }
    if size_gb < args.max_gb as f64 * 0.7 {
        println!("prune   : skipped — target is within acceptable range");
        return Ok(());
    }
    println!("pruning : running cargo clean ...");
    let status = std::process::Command::new("cargo").arg("clean").status()?;
    if status.success() {
        println!("pruned  : target directory cleaned");
    } else {
        anyhow::bail!("cargo clean failed with status: {}", status);
    }
    Ok(())
}

fn total_size_gb(target_dir: &str) -> f64 {
    use walkdir::WalkDir;
    let bytes: u64 = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();
    bytes as f64 / 1_073_741_824.0
}

fn verdict(size_gb: f64, max_gb: f64) -> &'static str {
    if size_gb < max_gb * 0.7 {
        "pass"
    } else if size_gb < max_gb {
        "warn"
    } else {
        "fail"
    }
}
