//! Runs trybuild compile-fail/compile-pass fixtures for crates changed since the last green commit.
use crate::legacy_nouns::trybuild::{TrybuildChangedVerb, TrybuildReviewVerb, TrybuildUpdateVerb};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("changed")]
pub fn cmd_changed() -> Result<()> {
    let dummy_matches = clap::Command::new("changed").get_matches_from(vec!["changed"]);
    let args = VerbArgs::new(dummy_matches);
    TrybuildChangedVerb.run(&args)
}

#[verb("update")]
pub fn cmd_update() -> Result<()> {
    let dummy_matches = clap::Command::new("update").get_matches_from(vec!["update"]);
    let args = VerbArgs::new(dummy_matches);
    TrybuildUpdateVerb.run(&args)
}

#[verb("review")]
pub fn cmd_review() -> Result<()> {
    let dummy_matches = clap::Command::new("review").get_matches_from(vec!["review"]);
    let args = VerbArgs::new(dummy_matches);
    TrybuildReviewVerb.run(&args)
}
