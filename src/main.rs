use anyhow::Result;

mod adapters;
#[cfg(feature = "advanced")]
mod advanced;
mod autonomic;
pub mod barrier;
mod certification;
mod cicd_toml;
pub mod code_provenance;
mod engine;
pub mod evidence;
pub mod evidence_helpers;
mod integrations;
pub mod legacy_nouns;
pub mod nouns;
pub mod ocel;
mod policies;
pub mod session;
mod state;
mod ui;

fn main() -> Result<()> {
    clap_noun_verb::run().map_err(|e| anyhow::anyhow!("{}", e))
}
