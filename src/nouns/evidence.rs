use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb("generate")]
pub fn cmd_generate(_repo: Option<String>, _json: bool) -> Result<()> {
    println!(r#"{{"status": "ok"}}"#);
    Ok(())
}
