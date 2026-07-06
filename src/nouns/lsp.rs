//! Language server for local CI/CD readiness diagnostics.
#![cfg(feature = "anti-llm-cheat")]

use crate::legacy_nouns::lsp::LspCheckVerb;
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("check")]
pub fn cmd_check() -> Result<()> {
    let dummy_matches = clap::Command::new("check").get_matches_from(vec!["check"]);
    let args = VerbArgs::new(dummy_matches);
    LspCheckVerb.run(&args)
}
