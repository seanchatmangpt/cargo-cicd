use anyhow::Result;
use std::path::PathBuf;

/// Arguments for `cargo cicd publish`.
#[derive(clap::Args, Debug)]
pub struct PublishArgs {
    /// Path to write cicd.toml (default: ./cicd.toml).
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Collect workspace state and write cicd.toml to the output path.
pub fn run(args: &PublishArgs) -> Result<()> {
    let path = args.output.clone().unwrap_or_else(|| PathBuf::from("cicd.toml"));
    let config = cargo_cicd::CicdToml::from_current_workspace();
    config.write_to_file(&path)?;
    println!("published: {}", path.display());
    Ok(())
}
