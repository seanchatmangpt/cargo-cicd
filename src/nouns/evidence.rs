//! Adjudicates and inspects recorded process evidence (XES/OCEL logs).
use crate::legacy_nouns::evidence::{
    AuditVerb, DoctorVerb, EvidenceListVerb, EvidenceResetVerb, EvidenceShowVerb,
};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("doctor")]
pub fn cmd_doctor() -> Result<()> {
    let dummy_matches = clap::Command::new("doctor").get_matches_from(vec!["doctor"]);
    let args = VerbArgs::new(dummy_matches);
    DoctorVerb.run(&args)
}

#[verb("audit")]
pub fn cmd_audit() -> Result<()> {
    let dummy_matches = clap::Command::new("audit").get_matches_from(vec!["audit"]);
    let args = VerbArgs::new(dummy_matches);
    AuditVerb.run(&args)
}

#[verb("show")]
pub fn cmd_show() -> Result<()> {
    let dummy_matches = clap::Command::new("show").get_matches_from(vec!["show"]);
    let args = VerbArgs::new(dummy_matches);
    EvidenceShowVerb.run(&args)
}

#[verb("list")]
pub fn cmd_list() -> Result<()> {
    let dummy_matches = clap::Command::new("list").get_matches_from(vec!["list"]);
    let args = VerbArgs::new(dummy_matches);
    EvidenceListVerb.run(&args)
}

#[verb("reset")]
pub fn cmd_reset() -> Result<()> {
    let dummy_matches = clap::Command::new("reset").get_matches_from(vec!["reset"]);
    let args = VerbArgs::new(dummy_matches);
    EvidenceResetVerb.run(&args)
}
