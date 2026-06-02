use clap::{Parser, Subcommand};

/// Local-first CI/CD helpers for Rust workspaces.
#[derive(Parser)]
#[command(name = "cargo-cicd", version, about, long_about = None)]
#[command(bin_name = "cargo cicd")]
struct Cli {
    #[command(subcommand)]
    noun: Noun,
}

#[derive(Subcommand)]
enum Noun {
    /// Manage the workspace state
    Workspace {
        #[command(subcommand)]
        verb: WorkspaceVerb,
    },
    /// Manage the target directory
    Target {
        #[command(subcommand)]
        verb: TargetVerb,
    },
    /// Manage test execution
    Test {
        #[command(subcommand)]
        verb: TestVerb,
    },
    /// Manage git phase checks
    Git {
        #[command(subcommand)]
        verb: GitVerb,
    },
    /// Manage cicd.toml configuration
    Config {
        #[command(subcommand)]
        verb: ConfigVerb,
    },
}

#[derive(Subcommand)]
enum WorkspaceVerb {
    /// Show current workspace state
    Status,
    /// Scan workspace and update state snapshot
    Scan,
}

#[derive(Subcommand)]
enum TargetVerb {
    /// Show target directory size and verdict
    Status,
    /// Clean the target directory
    Clean {
        /// Force clean even if under threshold
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum TestVerb {
    /// Show which tests would run given changed files
    Plan,
    /// Run tests for changed files only
    Run {
        /// Base branch/commit to diff against
        #[arg(long, default_value = "origin/main")]
        base: String,
    },
}

#[derive(Subcommand)]
enum GitVerb {
    /// Show git phase state (branch, dirty, staged, ahead/behind)
    Status,
    /// Check git state and emit recommended action
    Check,
}

#[derive(Subcommand)]
enum ConfigVerb {
    /// Print the current cicd.toml (or defaults if absent)
    Show,
    /// Write a default cicd.toml to the current directory
    Init {
        /// Overwrite existing cicd.toml
        #[arg(long)]
        force: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.noun {
        Noun::Workspace { verb } => match verb {
            WorkspaceVerb::Status => {
                println!("workspace status: not yet implemented");
            }
            WorkspaceVerb::Scan => {
                println!("workspace scan: not yet implemented");
            }
        },

        Noun::Target { verb } => match verb {
            TargetVerb::Status => {
                println!("target status: not yet implemented");
            }
            TargetVerb::Clean { force } => {
                if force {
                    println!("target clean --force: not yet implemented");
                } else {
                    println!("target clean: not yet implemented");
                }
            }
        },

        Noun::Test { verb } => match verb {
            TestVerb::Plan => {
                println!("test plan: not yet implemented");
            }
            TestVerb::Run { base } => {
                println!("test run --base {}: not yet implemented", base);
            }
        },

        Noun::Git { verb } => match verb {
            GitVerb::Status => {
                println!("git status: not yet implemented");
            }
            GitVerb::Check => {
                println!("git check: not yet implemented");
            }
        },

        Noun::Config { verb } => match verb {
            ConfigVerb::Show => {
                let config = cargo_cicd::CicdToml::default();
                let toml_str = toml::to_string_pretty(&config)?;
                println!("{}", toml_str);
            }
            ConfigVerb::Init { force } => {
                let path = std::path::Path::new("cicd.toml");
                if path.exists() && !force {
                    eprintln!("cicd.toml already exists — use --force to overwrite");
                    std::process::exit(1);
                }
                let config = cargo_cicd::CicdToml::default();
                config.write_to_file(path)?;
                println!("wrote cicd.toml");
            }
        },
    }

    Ok(())
}
