//! Diagnoses repository health against a recorded baseline and reports evidence drift.
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize)]
struct DoctorOutput {
    schema: &'static str,
    q_doctor: u32,
    counterexamples: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DoctorBaseline {
    pub schema: String,
    pub timestamp: String,
    pub git_commit: String,
    pub counterexamples: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DoctorDiffOutput {
    pub schema: String,
    pub baseline_path: String,
    pub baseline_timestamp: String,
    pub baseline_git_commit: String,
    pub current_git_commit: String,
    pub new_counterexamples: Vec<String>,
    pub resolved_counterexamples: Vec<String>,
    pub q_diff: u8,
}

fn get_git_sha(repo_dir: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[verb("repo")]
pub fn cmd_repo(
    repo: Option<String>,
    json: bool,
    write_baseline: Option<String>,
    baseline: Option<String>,
    diff: bool,
) -> Result<()> {
    let repo_path = repo.unwrap_or_else(|| ".".into());
    let repo_dir = Path::new(&repo_path);

    if let Some(baseline_path) = write_baseline {
        write_baseline_logic(repo_dir, &baseline_path)
    } else if diff {
        diff_baseline_logic(repo_dir, baseline)
    } else {
        normal_doctor_logic(repo_dir, json)
    }
}

fn write_baseline_logic(repo_dir: &Path, baseline_path: &str) -> Result<()> {
    let mut counterexamples = do_doctor(repo_dir);
    counterexamples.sort();
    counterexamples.dedup();
    let git_commit = get_git_sha(repo_dir);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    let baseline = DoctorBaseline {
        schema: "cargo-cicd.doctor.baseline.v1".to_string(),
        timestamp,
        git_commit,
        counterexamples,
    };
    let json_str = serde_json::to_string_pretty(&baseline).unwrap();
    fs::write(baseline_path, &json_str)
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
    println!("Baseline written to {}", baseline_path);
    Ok(())
}

fn diff_baseline_logic(repo_dir: &Path, baseline: Option<String>) -> Result<()> {
    let baseline_path = baseline.ok_or_else(|| {
        clap_noun_verb::error::NounVerbError::execution_error(
            "--diff requires --baseline <path>".to_string(),
        )
    })?;
    let baseline_json = fs::read_to_string(&baseline_path)
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
    let baseline_data: DoctorBaseline = serde_json::from_str(&baseline_json)
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

    let mut current = do_doctor(repo_dir);
    current.sort();
    current.dedup();

    let current_git_commit = get_git_sha(repo_dir);

    let baseline_set: std::collections::HashSet<String> =
        baseline_data.counterexamples.iter().cloned().collect();
    let current_set: std::collections::HashSet<String> = current.iter().cloned().collect();

    let mut new_counterexamples: Vec<String> =
        current_set.difference(&baseline_set).cloned().collect();
    new_counterexamples.sort();
    let mut resolved_counterexamples: Vec<String> =
        baseline_set.difference(&current_set).cloned().collect();
    resolved_counterexamples.sort();

    let q_diff: u8 = if new_counterexamples.is_empty() { 1 } else { 0 };

    let diff_output = DoctorDiffOutput {
        schema: "cargo-cicd.doctor.diff.v1".to_string(),
        baseline_path: baseline_path.clone(),
        baseline_timestamp: baseline_data.timestamp,
        baseline_git_commit: baseline_data.git_commit,
        current_git_commit,
        new_counterexamples,
        resolved_counterexamples,
        q_diff,
    };

    println!("{}", serde_json::to_string_pretty(&diff_output).unwrap());

    if q_diff == 0 {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Regressions detected".to_string(),
        ));
    }
    Ok(())
}

fn normal_doctor_logic(repo_dir: &Path, json: bool) -> Result<()> {
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
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Fraud detected".to_string(),
        ));
    }

    Ok(())
}

fn do_doctor(repo_dir: &Path) -> Vec<String> {
    let mut counterexamples: Vec<String> = crate::barrier::detect_barriers(repo_dir)
        .into_iter()
        .map(|ce| format!("{:?}", ce))
        .collect();

    for entry in WalkDir::new(repo_dir) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path
            .components()
            .any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git")
        {
            continue;
        }

        if path.is_file() {
            if path.file_name().and_then(|s| s.to_str()) == Some("doctor.rs") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("AGENTS.md")
                || path.extension().and_then(|s| s.to_str()) == Some("md")
            {
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
                if (content.contains("std::fs::write") || content.contains("File::create"))
                    && content.contains("\"cargo-cicd.receipt")
                {
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
                if (content.contains("\"git_before\": \"") || content.contains("git_before: \""))
                    && !content.contains("git_before: String")
                {
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
                if content.contains("Command::new(\"sh\")")
                    || content.contains("Command::new(\"bash\")")
                {
                    counterexamples.push("shell_called_by_agent".to_string());
                }
            }
        }
    }

    let ocel_path = repo_dir.join(".cargo-cicd/ocel/events.jsonl");
    if ocel_path.exists() {
        if let Ok(content) = fs::read_to_string(ocel_path) {
            for line in content.lines() {
                if line.contains("\"command\":[\"cargo\"")
                    || line.contains("\"command\": [\"cargo\"")
                {
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

#[verb("diff")]
/// Diff current doctor results against a saved baseline.
pub fn cmd_diff(repo: Option<String>, baseline: Option<String>) -> Result<()> {
    let repo_path = repo.unwrap_or_else(|| ".".into());
    let repo_dir = Path::new(&repo_path);
    diff_baseline_logic(repo_dir, baseline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_baseline_for_dir(dir: &std::path::Path, counterexamples: Vec<String>) -> String {
        let baseline = DoctorBaseline {
            schema: "cargo-cicd.doctor.baseline.v1".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            git_commit: "abc123".to_string(),
            counterexamples,
        };
        let path = dir.join("baseline.json");
        fs::write(&path, serde_json::to_string(&baseline).unwrap()).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn doctor_diff_detects_new_counterexample() {
        let tmp = TempDir::new().unwrap();
        let repo_dir = tmp.path();

        // Write clean baseline (no counterexamples)
        let baseline_path = write_baseline_for_dir(repo_dir, vec![]);

        // Add a file that triggers fake_test counterexample
        let src_dir = repo_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            src_dir.join("fake_test.rs"),
            "fn foo() { assert!(true); }\n",
        )
        .unwrap();

        // Run do_doctor and compute diff manually
        let mut current = do_doctor(repo_dir);
        current.sort();
        current.dedup();

        let baseline_json = fs::read_to_string(&baseline_path).unwrap();
        let baseline_data: DoctorBaseline = serde_json::from_str(&baseline_json).unwrap();

        let baseline_set: std::collections::HashSet<String> =
            baseline_data.counterexamples.iter().cloned().collect();
        let current_set: std::collections::HashSet<String> = current.iter().cloned().collect();

        let mut new_counterexamples: Vec<String> =
            current_set.difference(&baseline_set).cloned().collect();
        new_counterexamples.sort();
        let resolved_counterexamples: Vec<String> =
            baseline_set.difference(&current_set).cloned().collect();

        let q_diff: u8 = if new_counterexamples.is_empty() { 1 } else { 0 };

        assert!(
            new_counterexamples.iter().any(|s| s.contains("fake_test")),
            "expected fake_test in new_counterexamples, got: {:?}",
            new_counterexamples
        );
        assert_eq!(q_diff, 0, "q_diff should be 0 when regressions found");
        let _ = resolved_counterexamples;
    }

    #[test]
    fn doctor_diff_detects_resolved() {
        let tmp = TempDir::new().unwrap();
        let repo_dir = tmp.path();

        // Get baseline counterexamples (including natural empty directory counterexamples)
        let mut baseline_list = do_doctor(repo_dir);
        baseline_list.push("fake_test".to_string());
        let baseline_path = write_baseline_for_dir(repo_dir, baseline_list);

        // Current repo has no fake_test file → no counterexample
        let mut current = do_doctor(repo_dir);
        current.sort();
        current.dedup();

        let baseline_json = fs::read_to_string(&baseline_path).unwrap();
        let baseline_data: DoctorBaseline = serde_json::from_str(&baseline_json).unwrap();

        let baseline_set: std::collections::HashSet<String> =
            baseline_data.counterexamples.iter().cloned().collect();
        let current_set: std::collections::HashSet<String> = current.iter().cloned().collect();

        let new_counterexamples: Vec<String> =
            current_set.difference(&baseline_set).cloned().collect();
        let mut resolved_counterexamples: Vec<String> =
            baseline_set.difference(&current_set).cloned().collect();
        resolved_counterexamples.sort();

        let q_diff: u8 = if new_counterexamples.is_empty() { 1 } else { 0 };

        assert!(
            !resolved_counterexamples.is_empty(),
            "expected resolved_counterexamples to be non-empty"
        );
        assert_eq!(q_diff, 1, "q_diff should be 1 when no regressions");
    }
}
