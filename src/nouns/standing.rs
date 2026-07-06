//! The standing compiler: ingest evidence lying around the workspace,
//! score each artifact's readiness ladder, and write a `praxis-standing.v1`
//! document plus focused sub-slices under `target/praxis-standing/`.

use cargo_cicd_core::standing::{emit, score, sources, StandingArtifact, StandingDocument};
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn standing_out_dir(repo_dir: &str) -> PathBuf {
    Path::new(repo_dir).join("target").join("praxis-standing")
}

fn standing_json_path(repo_dir: &str) -> PathBuf {
    standing_out_dir(repo_dir).join("standing.json")
}

/// Read `target/praxis-standing/standing.json`, tolerating a missing or
/// unparseable file by returning an empty document rather than erroring.
/// Shared with `gate release`, which needs the same persisted document
/// without wanting to duplicate the read/parse path.
pub fn load_standing_document_tolerant(repo_dir: &str) -> StandingDocument {
    std::fs::read_to_string(standing_json_path(repo_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| StandingDocument {
            release_id: String::new(),
            generated_at_utc: String::new(),
            generator: String::new(),
            standing_version: "1".to_string(),
            artifacts: vec![],
        })
}

/// Run every configured ingestor and return the merged, scored artifact list.
fn ingest_all(repo_dir: &str, cfg: &crate::cicd_toml::StandingConfig) -> Vec<StandingArtifact> {
    let cwd_guard = std::env::current_dir().ok();
    // Ingestors that shell out (doctor, clients) run relative to repo_dir.
    if cwd_guard.as_deref() != Some(Path::new(repo_dir)) {
        let _ = std::env::set_current_dir(repo_dir);
    }

    let mut artifacts = vec![];
    artifacts.extend(sources::ingest_doctor_json(cfg.doctor_command.as_deref()));

    // `ocel_logs` and `process_validation` both feed the same
    // process-validation ingestor: two config knobs, one evidence shape.
    let mut ocel_globs = cfg.ocel_logs.clone();
    ocel_globs.extend(cfg.process_validation.clone());
    if !ocel_globs.is_empty() || (cfg.ocel_logs.is_empty() && cfg.process_validation.is_empty()) {
        artifacts.extend(sources::ingest_ocel_process_validation(&ocel_globs));
    }

    artifacts.extend(sources::ingest_receipt_ledgers(&cfg.receipt_ledgers));
    artifacts.extend(sources::ingest_plan_runs(cfg.plan_runs_glob.as_deref()));
    artifacts.extend(sources::ingest_bench_raw(cfg.bench_raw_glob.as_deref()));
    artifacts.extend(sources::ingest_claim_tables(&cfg.claim_tables));
    artifacts.extend(sources::ingest_client_builds(
        &cfg.clients,
        Duration::from_secs(300),
    ));

    if let Some(prev) = cwd_guard {
        let _ = std::env::set_current_dir(prev);
    }

    score::score_all(&mut artifacts);
    artifacts
}

fn build_document(release_id: &str, artifacts: Vec<StandingArtifact>) -> StandingDocument {
    StandingDocument {
        release_id: release_id.to_string(),
        generated_at_utc: crate::evidence::now_iso8601(),
        generator: format!("cargo-cicd-standing/{}", env!("CARGO_PKG_VERSION")),
        standing_version: "1".to_string(),
        artifacts,
    }
}

/// Mint a receipt for the refresh via the existing `Receipt::mint` path,
/// writing it into `.cargo-cicd/receipts/` where `receipt verify` expects it.
fn mint_refresh_receipt(repo_dir: &str, standing_json: &Path) -> std::io::Result<PathBuf> {
    use crate::nouns::receipt::{ExecutionTrace, Receipt};
    use std::collections::BTreeMap;

    let bytes = std::fs::read(standing_json).unwrap_or_default();
    let digest = crate::ocel::blake3_hex(&bytes);
    let git_head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HEAD-not-resolved".to_string());

    let mut output_artifacts = BTreeMap::new();
    output_artifacts.insert(
        "standing.json".to_string(),
        standing_json.to_string_lossy().to_string(),
    );

    let trace = ExecutionTrace {
        command: vec!["cargo".to_string(), "cicd".to_string(), "standing".to_string(), "refresh".to_string()],
        exit_code: 0,
        stdout_digest: digest,
        stderr_digest: String::new(),
        git_before: git_head.clone(),
        git_after: git_head,
        input_artifacts: BTreeMap::new(),
        output_artifacts,
    };
    let receipt = Receipt::mint(&trace);

    let receipts_dir = Path::new(repo_dir).join(".cargo-cicd").join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = receipts_dir.join(format!("standing-refresh-{ts}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap_or_default())?;
    Ok(path)
}

/// Emit one `standing_compiled` OCEL event per artifact, reusing
/// `src/ocel.rs`'s hash-chained JSONL writer as-is.
fn emit_standing_ocel(repo_dir: &str, doc: &StandingDocument) {
    for artifact in &doc.artifacts {
        let _ = crate::ocel::append_ocel_event(
            repo_dir,
            "standing_compiled",
            serde_json::json!({
                "artifact_id": artifact.id,
                "kind": format!("{:?}", artifact.kind),
                "standing": artifact.standing,
                "ladder_level": artifact.ladder_level,
                "scope": artifact.scope,
            }),
            "",
        );
    }
}

fn write_all_outputs(repo_dir: &str, doc: &StandingDocument) -> std::io::Result<PathBuf> {
    let out_dir = standing_out_dir(repo_dir);
    std::fs::create_dir_all(&out_dir)?;
    let json_path = out_dir.join("standing.json");
    emit::write_standing_json(doc, &json_path)?;
    emit::write_standing_ttl(doc, &out_dir.join("standing.ttl"))?;
    emit::write_summaries(doc, &out_dir)?;
    emit_standing_ocel(repo_dir, doc);
    Ok(json_path)
}

#[verb("refresh")]
pub fn cmd_refresh(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let cfg = crate::cicd_toml::load_or_default().standing;
    let release_id = format!("v{}", env!("CARGO_PKG_VERSION"));

    let artifacts = ingest_all(&repo_dir, &cfg);
    let validation_errors: Vec<String> = artifacts
        .iter()
        .filter_map(|a| a.validate().err().map(|e| e.to_string()))
        .collect();

    let doc = build_document(&release_id, artifacts);

    let json_path = write_all_outputs(&repo_dir, &doc).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "failed to write standing outputs: {e}"
        ))
    })?;

    let receipt_path = mint_refresh_receipt(&repo_dir, &json_path).ok();

    let summary = serde_json::json!({
        "schema": "cargo-cicd.standing.refresh.v1",
        "release_id": doc.release_id,
        "artifact_count": doc.artifacts.len(),
        "validation_errors": validation_errors,
        "standing_json": json_path.to_string_lossy(),
        "receipt": receipt_path.map(|p| p.to_string_lossy().to_string()),
    });

    if json {
        println!("{}", serde_json::to_string(&summary).unwrap_or_default());
    } else {
        println!(
            "standing refresh: {} artifact(s) -> {}",
            doc.artifacts.len(),
            json_path.display()
        );
        if !validation_errors.is_empty() {
            println!("validation errors:");
            for e in &validation_errors {
                println!("  - {e}");
            }
        }
    }

    Ok(())
}

/// Load the artifact list persisted by a previous `refresh`, or an empty
/// list if none exists yet (everything fresh then reads as "added").
fn load_persisted_artifacts(repo_dir: &str) -> Vec<StandingArtifact> {
    std::fs::read_to_string(standing_json_path(repo_dir))
        .ok()
        .and_then(|s| serde_json::from_str::<StandingDocument>(&s).ok())
        .map(|d| d.artifacts)
        .unwrap_or_default()
}

fn drift_entry_for(artifact: &StandingArtifact, persisted: &[StandingArtifact]) -> Option<serde_json::Value> {
    match persisted.iter().find(|p| p.id == artifact.id) {
        None => Some(serde_json::json!({"artifact_id": artifact.id, "kind": "added"})),
        Some(prev) if prev.path != artifact.path || prev.standing != artifact.standing => {
            Some(serde_json::json!({
                "artifact_id": artifact.id,
                "kind": "changed",
                "was_path": prev.path,
                "now_path": artifact.path,
            }))
        }
        _ => None,
    }
}

fn removed_entries(persisted: &[StandingArtifact], fresh: &[StandingArtifact]) -> Vec<serde_json::Value> {
    persisted
        .iter()
        .filter(|prev| !fresh.iter().any(|a| a.id == prev.id))
        .map(|prev| serde_json::json!({"artifact_id": prev.id, "kind": "removed"}))
        .collect()
}

/// Diff a fresh ingestion against the persisted standing document:
/// `added` (in fresh, not persisted), `changed` (path/standing differs),
/// `removed` (in persisted, not fresh).
fn compute_drift(persisted: &[StandingArtifact], fresh: &[StandingArtifact]) -> Vec<serde_json::Value> {
    let mut drift: Vec<serde_json::Value> = fresh
        .iter()
        .filter_map(|a| drift_entry_for(a, persisted))
        .collect();
    drift.extend(removed_entries(persisted, fresh));
    drift
}

fn print_drift_report(repo_dir: &str, drift: &[serde_json::Value], json: bool) {
    if json {
        let report = serde_json::json!({
            "schema": "cargo-cicd.standing.verify.v1",
            "repo": repo_dir,
            "drift_count": drift.len(),
            "drift": drift,
        });
        println!("{}", serde_json::to_string(&report).unwrap_or_default());
        return;
    }
    println!("standing verify: {} drifted artifact(s)", drift.len());
    for d in drift {
        println!("  - {d}");
    }
}

#[verb("verify")]
pub fn cmd_verify(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let cfg = crate::cicd_toml::load_or_default().standing;

    let persisted = load_persisted_artifacts(&repo_dir);
    let fresh = ingest_all(&repo_dir, &cfg);
    let drift = compute_drift(&persisted, &fresh);

    print_drift_report(&repo_dir, &drift, json);

    if drift.is_empty() {
        Ok(())
    } else {
        Err(clap_noun_verb::error::NounVerbError::execution_error(
            "standing drift detected",
        ))
    }
}

fn print_report_table(doc: &StandingDocument) {
    println!("{:<28} {:<10} {:<40} {:<6} scope", "id", "kind", "standing", "ladder");
    for artifact in &doc.artifacts {
        println!("{}", format_report_row(artifact));
    }
}

fn format_report_row(artifact: &StandingArtifact) -> String {
    let standing_str = artifact
        .standing
        .iter()
        .map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{:<28} {:<10?} {:<40} {:<6} {}",
        artifact.id,
        artifact.kind,
        standing_str,
        artifact.ladder_level,
        artifact.scope.clone().unwrap_or_default(),
    )
}

fn load_standing_document(repo_dir: &str) -> Result<StandingDocument> {
    let path = standing_json_path(repo_dir);
    let content = std::fs::read_to_string(&path).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "no standing.json at {}: {e} (run `standing refresh` first)",
            path.display()
        ))
    })?;
    serde_json::from_str(&content).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "malformed standing.json: {e}"
        ))
    })
}

#[verb("report")]
pub fn cmd_report(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let doc = load_standing_document(&repo_dir)?;

    if json {
        println!("{}", serde_json::to_string(&doc).unwrap_or_default());
    } else {
        print_report_table(&doc);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_cicd_core::standing::{ArtifactKind, StandingStatus};

    #[test]
    fn ingest_all_with_empty_config_never_panics() {
        let cfg = crate::cicd_toml::StandingConfig::default();
        let dir = tempfile::tempdir().unwrap();
        let artifacts = ingest_all(dir.path().to_str().unwrap(), &cfg);
        // Every ingestor contributes at least its fallback artifact.
        assert!(artifacts.len() >= 6);
        assert!(artifacts
            .iter()
            .all(|a| !a.standing.is_empty(), ));
    }

    #[test]
    fn build_document_stamps_schema_fields() {
        let artifacts = vec![StandingArtifact {
            id: "x".to_string(),
            kind: ArtifactKind::Doc,
            path: "x".to_string(),
            standing: vec![StandingStatus::Discovered],
            scope: None,
            ladder_level: 0,
            evidence: vec![],
            external_operator_side_effects: vec![],
        }];
        let doc = build_document("v26.7.4", artifacts);
        assert_eq!(doc.standing_version, "1");
        assert_eq!(doc.release_id, "v26.7.4");
        assert_eq!(doc.artifacts.len(), 1);
    }
}
