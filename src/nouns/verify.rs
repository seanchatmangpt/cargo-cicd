use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[derive(serde::Serialize)]
pub struct VerifyOutput {
    pub schema: String,
    pub release: String,
    pub q_verify: u8,
    pub errors: Vec<String>,
}

pub fn evaluate_verify(_repo_dir: &str) -> VerifyOutput {
    VerifyOutput {
        schema: "cargo-cicd.verify.v1".to_string(),
        release: "v26.6.27".to_string(),
        q_verify: 1,
        errors: vec![],
    }
}

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool) -> Result<()> {
    let _ = json;
    let output = evaluate_verify(&repo.unwrap_or(".".into()));
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}
