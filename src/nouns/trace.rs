use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use std::collections::BTreeMap;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TraceProfileOutput {
    pub schema: String,
    pub repo: String,
    pub profile: String,
    pub recipe: String,
    pub command: Vec<String>,
    pub exit_code: i32,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub git_before: String,
    pub git_after: String,
    pub receipt_path: String,
    pub ocel_event_id: String,
    pub q: i32,
    pub provenance: String,
}

fn get_git_hash(repo_dir: &str) -> String {
    std::process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn simple_hex_hash(data: &[u8]) -> String {
    let mut h: [u64; 4] = [0xcbf29ce484222325, 0x9e3779b97f4a7c15, 0x6c62272e07bb0142, 0x517cc1b727220a95];
    for (i, &b) in data.iter().enumerate() {
        let lane = i % 4;
        h[lane] ^= b as u64;
        h[lane] = h[lane].wrapping_mul(0x00000100000001b3);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h[0], h[1], h[2], h[3])
}

fn get_recipe(profile: &str) -> Result<&'static str> {
    match profile {
        "test" => Ok("test"),
        "check" => Ok("check"),
        "clippy" => Ok("clippy"),
        "dx" => Ok("dx"),
        "dry-run" => Ok("publish-dry-run"),
        _ => Err(clap_noun_verb::error::NounVerbError::execution_error(format!("Unknown profile: {}", profile))),
    }
}

#[verb("profile")]
pub fn cmd_profile(repo: Option<String>, profile: String, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let recipe = get_recipe(&profile)?;
    
    let git_before = get_git_hash(&repo_dir);
    
    let mut cmd = std::process::Command::new("just");
    cmd.arg(recipe).current_dir(&repo_dir);
    
    let output = cmd.output().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("Failed to run just: {}", e)))?;
    let exit_code = output.status.code().unwrap_or(-1);
    
    let git_after = get_git_hash(&repo_dir);
    
    let stdout_digest = simple_hex_hash(&output.stdout);
    let stderr_digest = simple_hex_hash(&output.stderr);
    
    let command_vec = vec!["just".to_string(), recipe.to_string()];
    
    let trace = crate::nouns::receipt::ExecutionTrace {
        command: command_vec.clone(),
        exit_code,
        stdout_digest: stdout_digest.clone(),
        stderr_digest: stderr_digest.clone(),
        git_before: git_before.clone(),
        git_after: git_after.clone(),
        input_artifacts: BTreeMap::new(),
        output_artifacts: BTreeMap::new(),
    };
    
    let receipt = crate::nouns::receipt::Receipt::mint(&trace);
    
    let receipts_dir = Path::new(&repo_dir).join(".cargo-cicd").join("receipts");
    std::fs::create_dir_all(&receipts_dir).unwrap();
    let receipt_path = receipts_dir.join("latest.json");
    std::fs::write(&receipt_path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    
    let ocel_event = crate::ocel::append_ocel_event(&repo_dir, "ReceiptMinted", serde_json::json!({"receipt_digest": receipt.receipt_digest}), "").map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e))?;
    let ocel_event_id = ocel_event.event_id;

    let git_provenance =
        crate::code_provenance::detect_provenance_from_git(std::path::Path::new(&repo_dir));
    let provenance_tag = match &git_provenance {
        crate::code_provenance::CodeProvenance::Unknown => "unknown".to_string(),
        p => {
            let base = p.to_tag();
            if let Some(tool) = p.tool_name() {
                format!("{}:{}", base, tool)
            } else {
                base.to_string()
            }
        }
    };

    let out = TraceProfileOutput {
        schema: "cargo-cicd.trace.v1".to_string(),
        repo: repo_dir.clone(),
        profile,
        recipe: recipe.to_string(),
        command: command_vec,
        exit_code,
        stdout_digest,
        stderr_digest,
        git_before,
        git_after,
        receipt_path: receipt_path.to_string_lossy().to_string(),
        ocel_event_id,
        q: if exit_code == 0 { 1 } else { 0 },
        provenance: provenance_tag,
    };
    
    if json {
        println!("{}", serde_json::to_string(&out).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    }
    
    Ok(())
}
