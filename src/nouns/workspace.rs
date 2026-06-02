use anyhow::Result;

/// Verbs for `cargo cicd workspace`.
#[derive(clap::Subcommand, Debug)]
pub enum WorkspaceVerb {
    /// Run diagnostics — check toolchain, target size, git state, emit verdict.
    Doctor(WorkspaceDoctorArgs),
}

/// Arguments for `cargo cicd workspace doctor`.
#[derive(clap::Args, Debug)]
pub struct WorkspaceDoctorArgs {
    /// Maximum acceptable target size in GB.
    #[arg(long, default_value_t = 20)]
    pub max_gb: u32,
}

pub fn run(verb: &WorkspaceVerb) -> Result<()> {
    match verb {
        WorkspaceVerb::Doctor(args) => run_doctor(args),
    }
}

fn run_doctor(args: &WorkspaceDoctorArgs) -> Result<()> {
    let mut pass = true;
    let toolchain = active_toolchain();
    println!("[toolchain      ] {}", toolchain);
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let size_gb = target_size_gb(&target_dir);
    let target_verdict = if size_gb < args.max_gb as f64 * 0.7 {
        "pass"
    } else if size_gb < args.max_gb as f64 {
        "warn"
    } else {
        pass = false;
        "fail"
    };
    println!("[target         ] {:.2} GB — {}", size_gb, target_verdict);
    let branch = git_branch();
    let is_clean = git_is_clean();
    println!("[git            ] branch={} state={}", branch, if is_clean { "clean" } else { "dirty" });
    let has_toolchain_file = std::path::Path::new("rust-toolchain.toml").exists()
        || std::path::Path::new("rust-toolchain").exists();
    println!("[rust-toolchain ] {}", if has_toolchain_file { "present" } else { "missing" });
    let has_cargo_toml = std::path::Path::new("Cargo.toml").exists();
    if !has_cargo_toml {
        pass = false;
    }
    println!("[Cargo.toml     ] {}", if has_cargo_toml { "present" } else { "missing" });
    println!();
    if pass {
        println!("verdict: PASS");
    } else {
        println!("verdict: FAIL");
        std::process::exit(1);
    }
    Ok(())
}

fn active_toolchain() -> String {
    std::process::Command::new("rustup")
        .args(["show", "active-toolchain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().split_whitespace().next().unwrap_or("unknown").to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn target_size_gb(target_dir: &str) -> f64 {
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

fn git_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn git_is_clean() -> bool {
    std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| o.stdout.is_empty())
        .unwrap_or(false)
}
