use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb("verify")]
pub fn cmd_verify(_repo: Option<String>, _json: bool) -> Result<()> {
    Ok(())
}

#[verb("audit")]
pub fn cmd_audit(_repo: Option<String>, _json: bool) -> Result<()> {
    Ok(())
}
