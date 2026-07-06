//! Surfaces git working-tree status and performs the lawful branch-close sequence.
use crate::legacy_nouns::git::{
    GitCloseVerb, GitCommitVerb, GitDiffVerb, GitFetchVerb, GitStageVerb, GitStatusVerb,
};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("status")]
pub fn cmd_status() -> Result<()> {
    let dummy_matches = clap::Command::new("status").get_matches_from(vec!["status"]);
    let args = VerbArgs::new(dummy_matches);
    GitStatusVerb.run(&args)
}

#[verb("close")]
pub fn cmd_close() -> Result<()> {
    let dummy_matches = clap::Command::new("close").get_matches_from(vec!["close"]);
    let args = VerbArgs::new(dummy_matches);
    GitCloseVerb.run(&args)
}

#[verb("diff")]
pub fn cmd_diff() -> Result<()> {
    let dummy_matches = clap::Command::new("diff").get_matches_from(vec!["diff"]);
    let args = VerbArgs::new(dummy_matches);
    GitDiffVerb.run(&args)
}

#[verb("stage")]
pub fn cmd_stage() -> Result<()> {
    let dummy_matches = clap::Command::new("stage").get_matches_from(vec!["stage"]);
    let args = VerbArgs::new(dummy_matches);
    GitStageVerb.run(&args)
}

#[verb("commit")]
pub fn cmd_commit() -> Result<()> {
    let dummy_matches = clap::Command::new("commit").get_matches_from(vec!["commit"]);
    let args = VerbArgs::new(dummy_matches);
    GitCommitVerb.run(&args)
}

#[verb("fetch")]
pub fn cmd_fetch() -> Result<()> {
    let dummy_matches = clap::Command::new("fetch").get_matches_from(vec!["fetch"]);
    let args = VerbArgs::new(dummy_matches);
    GitFetchVerb.run(&args)
}
