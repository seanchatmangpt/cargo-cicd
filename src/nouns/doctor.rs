use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use serde::Serialize;

#[derive(Serialize)]
struct DoctorOutput {
    schema: &'static str,
    q_doctor: u32,
    counterexamples: Vec<String>,
}

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool) -> Result<()> {
    let repo_path = repo.unwrap_or_else(|| ".".into());
    let repo_dir = Path::new(&repo_path);
    
    let mut counterexamples = do_doctor(repo_dir);
    counterexamples.sort();
    counterexamples.dedup();
    
    let q_doctor = if counterexamples.is_empty() { 1 } else { 0 };
    
    let output = DoctorOutput {
        schema: "cargo-cicd.doctor.v1",
        q_doctor,
        counterexamples,
    };
    
    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
    
    if q_doctor == 0 {
        return Err(clap_noun_verb::error::NounVerbError::execution_error("Fraud detected".to_string()));
    }
    
    Ok(())
}

fn do_doctor(repo_dir: &Path) -> Vec<String> {
    let mut counterexamples = Vec::new();
    
    for entry in WalkDir::new(repo_dir) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        let path = entry.path();
        
        if path.components().any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git") {
            continue;
        }

        if path.is_file() {
            if path.file_name().and_then(|s| s.to_str()) == Some("doctor.rs") {
                continue; 
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("AGENTS.md") || path.extension().and_then(|s| s.to_str()) == Some("md") {
                continue; 
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            
            if ext == "py" {
                counterexamples.push("python_called_by_agent".to_string());
            } else if ext == "sh" {
                counterexamples.push("shell_called_by_agent".to_string());
            }
            
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            
            if ext == "rs" {
                if (content.contains("std::fs::write") || content.contains("File::create")) && content.contains("\"cargo-cicd.receipt") {
                    counterexamples.push("manual_receipt_json".to_string());
                }
                if content.contains("assert!(true)") {
                    counterexamples.push("fake_test".to_string());
                }
                if content.contains("\"q_release\": 1") || content.contains("\"q_release\":1") {
                    counterexamples.push("dummy_gate".to_string());
                }
                if content.contains("\"cargo-cicd.gate.v1\"") && content.contains("\"token\"") {
                    counterexamples.push("token_gate".to_string());
                }
                if (content.contains("\"git_before\": \"") || content.contains("git_before: \"")) && !content.contains("git_before: String") {
                    counterexamples.push("hardcoded_commitment".to_string());
                }
                if content.to_lowercase().contains("placeholder") {
                    if content.to_lowercase().contains("gate") {
                        counterexamples.push("placeholder_authority".to_string());
                    }
                    if content.to_lowercase().contains("ocel") {
                        counterexamples.push("ocel_replay_placeholder".to_string());
                    }
                }
                if content.contains("Command::new(\"python") {
                    counterexamples.push("python_called_by_agent".to_string());
                }
                if content.contains("Command::new(\"sh\")") || content.contains("Command::new(\"bash\")") {
                    counterexamples.push("shell_called_by_agent".to_string());
                }
            }
        }
    }
    
    let ocel_path = repo_dir.join(".cargo-cicd/ocel/events.jsonl");
    if ocel_path.exists() {
        if let Ok(content) = fs::read_to_string(ocel_path) {
            for line in content.lines() {
                if line.contains("\"command\":[\"cargo\"") || line.contains("\"command\": [\"cargo\"") {
                    counterexamples.push("raw_cargo_used_by_agent".to_string());
                }
                if line.contains("\"just\"") && line.contains("\"agent_command\"") {
                    counterexamples.push("just_called_by_agent".to_string());
                }
            }
        }
    }
    
    counterexamples
}

#[verb("evidence")]
pub fn cmd_evidence(_json: bool) -> Result<()> {
    crate::legacy_nouns::evidence::EvidenceNoun::run_direct()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
