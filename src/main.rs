#![allow(dead_code, unused_imports)]
use anyhow::Result;
use clap_noun_verb::CliBuilder;

mod adapters;
mod autonomic;
mod cicd_toml;
mod engine;
mod nouns;
mod policies;
mod state;

fn main() -> Result<()> {
    let cli = CliBuilder::new()
        .name("cargo-cicd")
        .version("26.6.2")
        .about("Local-first CI/CD helpers for Rust workspaces: clean target dirs, run changed tests, check git state, and publish cicd.toml.")
        .noun(nouns::status::StatusNoun::new())
        .noun(nouns::target::TargetNoun::new())
        .noun(nouns::test::TestNoun::new())
        .noun(nouns::trybuild::TrybuildNoun::new())
        .noun(nouns::git::GitNoun::new())
        .noun(nouns::publish::PublishNoun::new())
        .noun(nouns::workspace::WorkspaceNoun::new());

    cli.run().map_err(|e| anyhow::anyhow!("{}", e))
}
