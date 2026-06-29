use crate::legacy_nouns::workspace::{
    WorkspaceDoctorVerb, WorkspaceListVerb, WorkspaceSyncVerb, WorkspaceValidateVerb,
};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("doctor")]
pub fn cmd_doctor() -> Result<()> {
    let dummy_matches = clap::Command::new("doctor").get_matches_from(vec!["doctor"]);
    let args = VerbArgs::new(dummy_matches);
    WorkspaceDoctorVerb.run(&args)
}

#[verb("validate")]
pub fn cmd_validate() -> Result<()> {
    let dummy_matches = clap::Command::new("validate").get_matches_from(vec!["validate"]);
    let args = VerbArgs::new(dummy_matches);
    WorkspaceValidateVerb.run(&args)
}

#[verb("sync")]
pub fn cmd_sync() -> Result<()> {
    let dummy_matches = clap::Command::new("sync").get_matches_from(vec!["sync"]);
    let args = VerbArgs::new(dummy_matches);
    WorkspaceSyncVerb.run(&args)
}

#[verb("list")]
pub fn cmd_list() -> Result<()> {
    let dummy_matches = clap::Command::new("list").get_matches_from(vec!["list"]);
    let args = VerbArgs::new(dummy_matches);
    WorkspaceListVerb.run(&args)
}
