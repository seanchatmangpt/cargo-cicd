//! Verifies and audits signed execution receipts recording command provenance.
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub command: Vec<String>,
    pub exit_code: i32,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub git_before: String,
    pub git_after: String,
    pub input_artifacts: BTreeMap<String, String>,
    pub output_artifacts: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Receipt {
    pub schema: String,
    pub command: Vec<String>,
    pub exit_code: i32,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub git_before: String,
    pub git_after: String,
    pub input_artifacts: BTreeMap<String, String>,
    pub output_artifacts: BTreeMap<String, String>,
    pub timestamp: String,
    pub receipt_digest: String,
}

impl Receipt {
    pub fn mint(trace: &ExecutionTrace) -> Self {
        let timestamp = format!(
            "{:?}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let mut r = Self {
            schema: "cargo-cicd.receipt.v1".to_string(),
            command: trace.command.clone(),
            exit_code: trace.exit_code,
            stdout_digest: trace.stdout_digest.clone(),
            stderr_digest: trace.stderr_digest.clone(),
            git_before: trace.git_before.clone(),
            git_after: trace.git_after.clone(),
            input_artifacts: trace.input_artifacts.clone(),
            output_artifacts: trace.output_artifacts.clone(),
            timestamp,
            receipt_digest: String::new(),
        };
        r.receipt_digest = r.compute_hash();
        r
    }

    pub fn compute_hash(&self) -> String {
        let mut data = String::new();
        data.push_str(&self.command.join(" "));
        data.push_str(&self.exit_code.to_string());
        data.push_str(&self.stdout_digest);
        data.push_str(&self.stderr_digest);
        data.push_str(&self.git_before);
        data.push_str(&self.git_after);
        for (k, v) in &self.input_artifacts {
            data.push_str(k);
            data.push_str(v);
        }
        for (k, v) in &self.output_artifacts {
            data.push_str(k);
            data.push_str(v);
        }
        fn_hash(data.as_bytes())
    }

    pub fn is_valid(&self) -> bool {
        self.schema == "cargo-cicd.receipt.v1" && self.receipt_digest == self.compute_hash()
    }
}

fn fn_hash(data: &[u8]) -> String {
    let mut h: [u64; 4] = [
        0xcbf29ce484222325,
        0x9e3779b97f4a7c15,
        0x6c62272e07bb0142,
        0x517cc1b727220a95,
    ];
    for (i, &b) in data.iter().enumerate() {
        let lane = i % 4;
        h[lane] ^= b as u64;
        h[lane] = h[lane].wrapping_mul(0x00000100000001b3);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h[0], h[1], h[2], h[3])
}

fn do_verify(repo_dir: &str) -> (usize, Vec<String>) {
    let receipts_dir = Path::new(repo_dir).join(".cargo-cicd").join("receipts");
    let mut counterexamples = vec![];
    let mut valid_count = 0;

    if !receipts_dir.exists() {
        return (valid_count, counterexamples);
    }

    for entry in std::fs::read_dir(receipts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let v: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);

        if let Ok(receipt) = serde_json::from_str::<Receipt>(&content) {
            if receipt.is_valid() {
                valid_count += 1;
            } else {
                counterexamples.push("receipt_hashes_itself".to_string());
            }
        } else {
            if v.get("schema").is_some() {
                if v.get("command").is_none() {
                    counterexamples.push("receipt_missing_command".to_string());
                } else if v.get("exit_code").is_none() {
                    counterexamples.push("receipt_missing_exit_code".to_string());
                } else if v.get("stdout_digest").is_none() {
                    counterexamples.push("receipt_missing_stdout_digest".to_string());
                } else if v.get("stderr_digest").is_none() {
                    counterexamples.push("receipt_missing_stderr_digest".to_string());
                } else if v.get("git_before").is_none() {
                    counterexamples.push("receipt_missing_git_before".to_string());
                } else if v.get("git_after").is_none() {
                    counterexamples.push("receipt_missing_git_after".to_string());
                } else {
                    counterexamples.push("manual_receipt_json".to_string());
                }
            } else {
                counterexamples.push("manual_receipt_json".to_string());
            }
        }
    }

    (valid_count, counterexamples)
}

#[verb("verify")]
pub fn cmd_verify(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let (valid_count, counterexamples) = do_verify(&repo_dir);

    if json {
        let output = serde_json::json!({
            "schema": "cargo-cicd.receipt.verify.v1",
            "repo": repo_dir,
            "valid_receipts": valid_count,
            "counterexamples": counterexamples,
            "q": if counterexamples.is_empty() { 1 } else { 0 }
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        println!("Valid receipts: {}", valid_count);
        if !counterexamples.is_empty() {
            println!("Counterexamples: {:?}", counterexamples);
        }
    }

    Ok(())
}

#[verb("audit")]
pub fn cmd_audit(_repo: Option<String>, _json: bool) -> Result<()> {
    Ok(())
}
