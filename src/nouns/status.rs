use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use crate::legacy_nouns::status::StatusNoun;

#[verb("show")]
pub fn cmd_show() -> Result<()> {
    StatusNoun::run_direct().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
