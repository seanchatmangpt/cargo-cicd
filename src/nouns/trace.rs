use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb("profile")]
pub fn cmd_profile(repo: Option<String>, profile: String, json: bool) -> Result<()> {
    let _ = json;
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let recipe = get_recipe(&profile)?;
    
    let mut cmd = std::process::Command::new("just");
    cmd.arg(recipe).current_dir(repo_dir);
    
    let status = cmd.status()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("Failed to run just: {}", e)))?;
        
    if !status.success() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(format!("Profile {} failed", profile)));
    }
    
    Ok(())
}

fn get_recipe(profile: &str) -> Result<&'static str> {
    match profile {
        "test" => Ok("test"),
        "clippy" => Ok("clippy"),
        "dx" => Ok("dx"),
        "dry-run" => Ok("publish-dry-run"),
        _ => Err(clap_noun_verb::error::NounVerbError::execution_error(format!("Unknown profile: {}", profile))),
    }
}
