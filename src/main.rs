use clap::{Parser, Subcommand};

mod nouns;

/// Local-first CI/CD helpers for Rust workspaces.
#[derive(Parser)]
#[command(name = "cargo-cicd", bin_name = "cargo cicd")]
#[command(about = "Local-first CI/CD helpers for Rust workspaces")]
#[command(long_about = "cargo-cicd keeps Rust workspaces clean, fast, and push-ready.")]
struct Cli {
    #[command(subcommand)]
    noun: Noun,
}

#[derive(Subcommand)]
enum Noun {
    /// Show workspace CI/CD status summary.
    Status(nouns::status::StatusArgs),
    /// Manage the target directory.
    Target {
        #[command(subcommand)]
        verb: nouns::target::TargetVerb,
    },
    /// Run tests for changed files.
    Test {
        #[command(subcommand)]
        verb: nouns::test::TestVerb,
    },
    /// Manage trybuild fixtures.
    Trybuild {
        #[command(subcommand)]
        verb: nouns::trybuild::TrybuildVerb,
    },
    /// Git phase helpers.
    Git {
        #[command(subcommand)]
        verb: nouns::git::GitVerb,
    },
    /// Collect workspace state and publish cicd.toml.
    Publish(nouns::publish::PublishArgs),
    /// Workspace diagnostics.
    Workspace {
        #[command(subcommand)]
        verb: nouns::workspace::WorkspaceVerb,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.noun {
        Noun::Status(args) => nouns::status::run(&args),
        Noun::Target { verb } => nouns::target::run(&verb),
        Noun::Test { verb } => nouns::test::run(&verb),
        Noun::Trybuild { verb } => nouns::trybuild::run(&verb),
        Noun::Git { verb } => nouns::git::run(&verb),
        Noun::Publish(args) => nouns::publish::run(&args),
        Noun::Workspace { verb } => nouns::workspace::run(&verb),
    }
}
