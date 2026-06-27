use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool) -> Result<()> {
    let _ = json;
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let mut cmd = std::process::Command::new("just");
    cmd.arg("gate").current_dir(repo_dir);
    let status = cmd.status()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("Failed to run just gate: {}", e)))?;
        
    if !status.success() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error("just gate failed"));
    }
    
    Ok(())
}
