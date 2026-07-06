//! Runs, checks the status of, and validates the workspace's CI/CD pipeline definition.
use crate::legacy_nouns::pipeline::{PipelineRunVerb, PipelineStatusVerb, PipelineValidateVerb};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("run")]
pub fn cmd_run() -> Result<()> {
    let dummy_matches = clap::Command::new("run").get_matches_from(vec!["run"]);
    let args = VerbArgs::new(dummy_matches);
    PipelineRunVerb.run(&args)
}

#[verb("status")]
pub fn cmd_status() -> Result<()> {
    let dummy_matches = clap::Command::new("status").get_matches_from(vec!["status"]);
    let args = VerbArgs::new(dummy_matches);
    PipelineStatusVerb.run(&args)
}

#[verb("validate")]
pub fn cmd_validate() -> Result<()> {
    let dummy_matches = clap::Command::new("validate").get_matches_from(vec!["validate"]);
    let args = VerbArgs::new(dummy_matches);
    PipelineValidateVerb.run(&args)
}
