use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb("repo")]
pub fn cmd_repo(json: bool) -> Result<()> {
    let _ = json;
    crate::legacy_nouns::workspace::WorkspaceNoun::run_doctor()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

#[verb("evidence")]
pub fn cmd_evidence(json: bool) -> Result<()> {
    crate::legacy_nouns::evidence::EvidenceNoun::run_direct()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
