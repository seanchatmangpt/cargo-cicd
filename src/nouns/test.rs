//! Runs cargo test restricted to crates whose source files changed since the last green commit.
use crate::legacy_nouns::test::{TestBenchVerb, TestChangedVerb, TestRunVerb};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("changed")]
pub fn cmd_changed() -> Result<()> {
    let dummy_matches = clap::Command::new("changed").get_matches_from(vec!["changed"]);
    let args = VerbArgs::new(dummy_matches);
    TestChangedVerb.run(&args)
}

#[verb("run")]
pub fn cmd_run() -> Result<()> {
    let dummy_matches = clap::Command::new("run").get_matches_from(vec!["run"]);
    let args = VerbArgs::new(dummy_matches);
    TestRunVerb.run(&args)
}

#[verb("bench")]
pub fn cmd_bench() -> Result<()> {
    let dummy_matches = clap::Command::new("bench").get_matches_from(vec!["bench"]);
    let args = VerbArgs::new(dummy_matches);
    TestBenchVerb.run(&args)
}
