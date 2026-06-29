use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use crate::legacy_nouns::publish::{PublishRunVerb, PublishCheckVerb, PublishValidateVerb};
use clap_noun_verb::{VerbArgs, VerbCommand};

#[verb("run")]
pub fn cmd_run() -> Result<()> {
    let dummy_matches = clap::Command::new("run").get_matches_from(vec!["run"]);
    let args = VerbArgs::new(dummy_matches);
    PublishRunVerb.run(&args)
}

#[verb("check")]
pub fn cmd_check() -> Result<()> {
    let dummy_matches = clap::Command::new("check").get_matches_from(vec!["check"]);
    let args = VerbArgs::new(dummy_matches);
    PublishCheckVerb.run(&args)
}

#[verb("validate")]
pub fn cmd_validate() -> Result<()> {
    let dummy_matches = clap::Command::new("validate").get_matches_from(vec!["validate"]);
    let args = VerbArgs::new(dummy_matches);
    PublishValidateVerb.run(&args)
}
