//! Verifies and audits signed execution receipts recording command provenance.
use clap_noun_verb::Result;
use clap_noun_verb::error::NounVerbError;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RECEIPT_SCHEMA: &str = "cargo-cicd.receipt.v1";
pub const AUDIT_SCHEMA: &str = "cargo-cicd.receipt.audit.v1";

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
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis().to_string())
            .unwrap_or_else(|_| "0".to_string());
        let mut r = Self {
            schema: RECEIPT_SCHEMA.to_string(),
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
        blake3::hash(data.as_bytes()).to_hex().to_string()
    }

    pub fn is_valid(&self) -> bool {
        self.schema == RECEIPT_SCHEMA && self.receipt_digest == self.compute_hash()
    }
}

/// blake3 hex digest of arbitrary bytes (e.g. captured stdout/stderr).
pub fn content_digest(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

fn receipts_dir(repo_dir: &str) -> PathBuf {
    Path::new(repo_dir).join(".cargo-cicd").join("receipts")
}

/// Classify one receipt file's content. Ok = valid, Err = counterexample reason.
fn judge_receipt(content: &str) -> std::result::Result<(), String> {
    if let Ok(receipt) = serde_json::from_str::<Receipt>(content) {
        if receipt.is_valid() {
            return Ok(());
        }
        return Err("receipt_hashes_itself".to_string());
    }
    let v: serde_json::Value =
        serde_json::from_str(content).unwrap_or(serde_json::Value::Null);
    if v.get("schema").is_some() {
        if v.get("command").is_none() {
            Err("receipt_missing_command".to_string())
        } else if v.get("exit_code").is_none() {
            Err("receipt_missing_exit_code".to_string())
        } else if v.get("stdout_digest").is_none() {
            Err("receipt_missing_stdout_digest".to_string())
        } else if v.get("stderr_digest").is_none() {
            Err("receipt_missing_stderr_digest".to_string())
        } else if v.get("git_before").is_none() {
            Err("receipt_missing_git_before".to_string())
        } else if v.get("git_after").is_none() {
            Err("receipt_missing_git_after".to_string())
        } else {
            Err("manual_receipt_json".to_string())
        }
    } else {
        Err("manual_receipt_json".to_string())
    }
}

#[derive(Serialize, Debug)]
pub struct AuditEntry {
    pub file: String,
    pub valid: bool,
    pub reason: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct AuditReport {
    pub valid_count: usize,
    pub invalid_count: usize,
    pub entries: Vec<AuditEntry>,
}

/// Scan the receipts dir and classify every `.json` receipt.
/// Missing receipts dir is a typed error, never vacuous success.
fn do_audit(repo_dir: &str) -> Result<AuditReport> {
    let dir = receipts_dir(repo_dir);
    if !dir.exists() {
        return Err(NounVerbError::execution_error(format!(
            "receipts directory not found: {}",
            dir.display()
        )));
    }
    let mut report = AuditReport {
        valid_count: 0,
        invalid_count: 0,
        entries: vec![],
    };
    for entry in std::fs::read_dir(&dir)
        .map_err(|e| NounVerbError::execution_error(format!("read_dir {}: {}", dir.display(), e)))?
    {
        let entry = entry
            .map_err(|e| NounVerbError::execution_error(format!("dir entry: {}", e)))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let content = std::fs::read_to_string(&path).map_err(|e| {
            NounVerbError::execution_error(format!("read {}: {}", path.display(), e))
        })?;
        match judge_receipt(&content) {
            Ok(()) => {
                report.valid_count += 1;
                report.entries.push(AuditEntry {
                    file: name,
                    valid: true,
                    reason: None,
                });
            }
            Err(reason) => {
                report.invalid_count += 1;
                report.entries.push(AuditEntry {
                    file: name,
                    valid: false,
                    reason: Some(reason),
                });
            }
        }
    }
    Ok(report)
}

/// Legacy verify shape: (valid count, flat counterexample list).
fn do_verify(repo_dir: &str) -> Result<(usize, Vec<String>)> {
    let report = do_audit(repo_dir)?;
    let counterexamples = report
        .entries
        .iter()
        .filter(|e| !e.valid)
        .map(|e| e.reason.clone().unwrap_or_else(|| "invalid_receipt".to_string()))
        .collect();
    Ok((report.valid_count, counterexamples))
}

#[verb("verify")]
pub fn cmd_verify(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let (valid_count, counterexamples) = do_verify(&repo_dir)?;

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
pub fn cmd_audit(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let report = do_audit(&repo_dir)?;

    if json {
        let output = serde_json::json!({
            "schema": AUDIT_SCHEMA,
            "repo": repo_dir,
            "valid_receipts": report.valid_count,
            "invalid_receipts": report.invalid_count,
            "receipts": report.entries,
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        println!(
            "Valid receipts: {}, invalid receipts: {}",
            report.valid_count, report.invalid_count
        );
        for e in &report.entries {
            if let Some(reason) = &e.reason {
                println!("  INVALID {}: {}", e.file, reason);
            }
        }
    }

    if report.invalid_count > 0 {
        return Err(NounVerbError::execution_error(format!(
            "receipt audit failed: {} invalid receipt(s)",
            report.invalid_count
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn trace(cmd: &str, out: &[u8]) -> ExecutionTrace {
        ExecutionTrace {
            command: vec!["just".to_string(), cmd.to_string()],
            exit_code: 0,
            stdout_digest: content_digest(out),
            stderr_digest: content_digest(b""),
            git_before: "aaa".into(),
            git_after: "aaa".into(),
            input_artifacts: BTreeMap::new(),
            output_artifacts: BTreeMap::new(),
        }
    }

    #[test]
    fn minted_receipt_is_valid_and_digest_is_blake3() {
        let r = Receipt::mint(&trace("test", b"hello"));
        assert!(r.is_valid());
        // blake3 hex is 64 chars; FNV 4-lane was 64 chars too, so pin a known vector.
        assert_eq!(r.receipt_digest.len(), 64);
        let known = blake3::hash(b"hello").to_hex().to_string();
        assert_eq!(content_digest(b"hello"), known);
    }

    #[test]
    fn tampered_receipt_fails_validation() {
        let mut r = Receipt::mint(&trace("test", b"hello"));
        assert!(r.is_valid());
        r.exit_code = 1;
        assert!(!r.is_valid());
    }

    #[test]
    fn different_stdout_yields_different_digest() {
        let a = Receipt::mint(&trace("test", b"one"));
        let b = Receipt::mint(&trace("test", b"two"));
        assert_ne!(a.stdout_digest, b.stdout_digest);
        assert_ne!(a.receipt_digest, b.receipt_digest);
    }

    #[test]
    fn audit_missing_dir_is_typed_error_not_vacuous_success() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = do_audit(tmp.path().to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("receipts directory not found"));
    }

    #[test]
    fn audit_reports_valid_and_corrupt_receipts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join(".cargo-cicd").join("receipts");
        fs::create_dir_all(&dir).unwrap();

        let good = Receipt::mint(&trace("test", b"ok"));
        fs::write(dir.join("good.json"), serde_json::to_string(&good).unwrap()).unwrap();

        let mut bad = Receipt::mint(&trace("test", b"ok"));
        bad.exit_code = 7; // tamper without re-minting digest
        fs::write(dir.join("bad.json"), serde_json::to_string(&bad).unwrap()).unwrap();

        fs::write(dir.join("garbage.json"), "not json at all").unwrap();

        let report = do_audit(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.invalid_count, 2);
        let reasons: Vec<&str> = report
            .entries
            .iter()
            .filter(|e| !e.valid)
            .map(|e| e.reason.as_deref().unwrap())
            .collect();
        assert!(reasons.contains(&"receipt_hashes_itself"));
        assert!(reasons.contains(&"manual_receipt_json"));
    }

    #[test]
    fn audit_all_valid_passes() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join(".cargo-cicd").join("receipts");
        fs::create_dir_all(&dir).unwrap();
        let good = Receipt::mint(&trace("check", b"ok"));
        fs::write(dir.join("good.json"), serde_json::to_string(&good).unwrap()).unwrap();
        let report = do_audit(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.invalid_count, 0);
    }
}
