//! Reports on and prunes stale build artefacts in the Cargo target directory.
use crate::legacy_nouns::target::{TargetPruneVerb, TargetShowVerb};
use clap_noun_verb::Result;
use clap_noun_verb::{VerbArgs, VerbCommand};
use clap_noun_verb_macros::verb;

#[verb("show")]
pub fn cmd_show() -> Result<()> {
    let dummy_matches = clap::Command::new("show").get_matches_from(vec!["show"]);
    let args = VerbArgs::new(dummy_matches);
    TargetShowVerb.run(&args)
}

#[verb("prune")]
pub fn cmd_prune(apply: bool) -> Result<()> {
    let mut args_vec = vec!["prune".to_string()];
    if apply {
        args_vec.push("--apply".to_string());
    }
    let dummy_matches = TargetPruneVerb.build_command().get_matches_from(args_vec);
    let args = VerbArgs::new(dummy_matches);
    TargetPruneVerb.run(&args)
}
