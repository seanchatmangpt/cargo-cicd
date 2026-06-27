#![allow(dead_code, unused_imports)]
use anyhow::Result;

mod adapters;
#[cfg(feature = "advanced")]
mod advanced;
mod autonomic;
mod certification;
mod cicd_toml;
mod engine;
pub mod evidence;
mod integrations;
pub mod legacy_nouns;
pub mod nouns;
mod policies;
pub mod session;
mod state;
mod ui;

fn main() -> Result<()> {
    // If invoked as `cargo cicd`, clap expects argv[1] to be the noun. But if we run `cargo run --bin cargo-cicd -- cicd doctor workspace`, argv[1] is `cicd`.
    // Actually, clap_noun_verb has a way to handle cargo subcommands? 
    // If not, we just rely on `clap_noun_verb::run()`. Wait, if `cicd` is present, it will try to find a noun named `cicd`.
    // Let's strip it by altering argv if possible, or wait, std::env::args is fixed.
    // We can use `clap_noun_verb::run_with(args)`. Let's check if it exists by compiling.
    
    // For now, let's just see if run() works. If it fails due to `cicd`, we'll find out.
    clap_noun_verb::run().map_err(|e| anyhow::anyhow!("{}", e))
}
