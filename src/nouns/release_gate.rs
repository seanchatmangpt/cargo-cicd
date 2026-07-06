//! Release gate: check a release's required `(artifact_id, status)` pairs
//! (`[standing.release_gates]` in `cicd.toml`) against the persisted
//! standing document.
//!
//! Kept as its own noun rather than a verb on `gate` — `gate.rs` is
//! independently evolving and this reads a disjoint config surface
//! (`[standing.release_gates]`), so a separate file avoids an unnecessary
//! coupling between the two.

use cargo_cicd_core::standing::{StandingDocument, StandingStatus};
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct MissingRequirement {
    pub artifact_id: String,
    pub status: StandingStatus,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ReleaseGateReport {
    pub schema: String,
    pub release_id: String,
    pub required: usize,
    pub satisfied: usize,
    pub missing: Vec<MissingRequirement>,
    pub q_release_gate: u8,
}

fn load_standing_document_tolerant(repo_dir: &str) -> StandingDocument {
    let path = Path::new(repo_dir)
        .join("target")
        .join("praxis-standing")
        .join("standing.json");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(StandingDocument {
            schema_id: cargo_cicd_core::standing::STANDING_SCHEMA_ID.to_string(),
            release_id: String::new(),
            generated_at_utc: String::new(),
            generator: String::new(),
            standing_version: "1".to_string(),
            artifacts: vec![],
        })
}

fn missing_requirement(
    req: &crate::cicd_toml::RequiredArtifactStatus,
    doc: &StandingDocument,
) -> Option<MissingRequirement> {
    match doc.artifacts.iter().find(|a| a.id == req.artifact_id) {
        None => Some(MissingRequirement {
            artifact_id: req.artifact_id.clone(),
            status: req.status,
            reason: "artifact not found in standing.json".to_string(),
        }),
        Some(a) if !a.standing.contains(&req.status) => Some(MissingRequirement {
            artifact_id: req.artifact_id.clone(),
            status: req.status,
            reason: format!(
                "artifact standing is {:?}, missing required status",
                a.standing
            ),
        }),
        _ => None,
    }
}

/// Check `release_id`'s required artifact+status pairs against the
/// persisted `standing.json`. Never panics: an absent standing.json or an
/// unconfigured release both resolve to an empty-requirements report
/// (vacuously satisfied) rather than an error, so the gate is safe to run
/// before any standing has ever been compiled.
pub fn check_release_gate(repo_dir: &str, release_id: &str) -> ReleaseGateReport {
    let cfg = crate::cicd_toml::load_or_default().standing;
    let required = cfg
        .release_gates
        .get(release_id)
        .cloned()
        .unwrap_or_default();
    let doc = load_standing_document_tolerant(repo_dir);

    let missing: Vec<MissingRequirement> = required
        .iter()
        .filter_map(|req| missing_requirement(req, &doc))
        .collect();

    ReleaseGateReport {
        schema: "cargo-cicd.standing.release-gate.v1".to_string(),
        release_id: release_id.to_string(),
        required: required.len(),
        satisfied: required.len() - missing.len(),
        q_release_gate: if missing.is_empty() { 1 } else { 0 },
        missing,
    }
}

fn print_release_gate_report(report: &ReleaseGateReport, json: bool) {
    if json {
        println!("{}", serde_json::to_string(report).unwrap_or_default());
        return;
    }
    println!(
        "release-gate {}: {}/{} satisfied",
        report.release_id, report.satisfied, report.required
    );
    for m in &report.missing {
        println!("  MISSING {} {:?}: {}", m.artifact_id, m.status, m.reason);
    }
}

#[verb("check")]
pub fn cmd_check(repo: Option<String>, release_id: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let release_id = release_id.unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));

    let report = check_release_gate(&repo_dir, &release_id);
    print_release_gate_report(&report, json);

    if report.q_release_gate == 1 {
        Ok(())
    } else {
        Err(clap_noun_verb::error::NounVerbError::execution_error(
            "release gate failed",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_cicd_core::standing::{ArtifactKind, StandingArtifact};
    use std::fs;

    fn write_standing_doc(dir: &Path, artifacts: Vec<StandingArtifact>) {
        let out_dir = dir.join("target").join("praxis-standing");
        fs::create_dir_all(&out_dir).unwrap();
        let doc = StandingDocument {
            schema_id: cargo_cicd_core::standing::STANDING_SCHEMA_ID.to_string(),
            release_id: "v26.7.4".to_string(),
            generated_at_utc: "now".to_string(),
            generator: "test".to_string(),
            standing_version: "1".to_string(),
            artifacts,
        };
        fs::write(
            out_dir.join("standing.json"),
            serde_json::to_string(&doc).unwrap(),
        )
        .unwrap();
    }

    fn artifact(id: &str, standing: Vec<StandingStatus>) -> StandingArtifact {
        StandingArtifact {
            id: id.to_string(),
            kind: ArtifactKind::RustCrate,
            path: id.to_string(),
            standing,
            scope: None,
            ladder_level: 0,
            evidence: vec![],
            external_operator_side_effects: vec![],
        }
    }

    #[test]
    fn missing_standing_json_is_vacuously_satisfied_when_no_gate_configured() {
        let dir = tempfile::tempdir().unwrap();
        let report = check_release_gate(dir.path().to_str().unwrap(), "v-unconfigured");
        assert_eq!(report.required, 0);
        assert_eq!(report.q_release_gate, 1);
    }

    #[test]
    fn missing_requirement_flags_absent_artifact() {
        let doc = StandingDocument {
            schema_id: cargo_cicd_core::standing::STANDING_SCHEMA_ID.to_string(),
            release_id: "x".to_string(),
            generated_at_utc: "x".to_string(),
            generator: "x".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![artifact("a", vec![StandingStatus::Tested])],
        };
        let req = crate::cicd_toml::RequiredArtifactStatus {
            artifact_id: "missing-crate".to_string(),
            status: StandingStatus::Tested,
        };
        let result = missing_requirement(&req, &doc);
        assert!(result.is_some());
    }

    #[test]
    fn missing_requirement_flags_wrong_status() {
        let doc = StandingDocument {
            schema_id: cargo_cicd_core::standing::STANDING_SCHEMA_ID.to_string(),
            release_id: "x".to_string(),
            generated_at_utc: "x".to_string(),
            generator: "x".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![artifact("a", vec![StandingStatus::Discovered])],
        };
        let req = crate::cicd_toml::RequiredArtifactStatus {
            artifact_id: "a".to_string(),
            status: StandingStatus::PublishReady,
        };
        assert!(missing_requirement(&req, &doc).is_some());
    }

    #[test]
    fn missing_requirement_none_when_status_present() {
        let doc = StandingDocument {
            schema_id: cargo_cicd_core::standing::STANDING_SCHEMA_ID.to_string(),
            release_id: "x".to_string(),
            generated_at_utc: "x".to_string(),
            generator: "x".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![artifact("a", vec![StandingStatus::Tested])],
        };
        let req = crate::cicd_toml::RequiredArtifactStatus {
            artifact_id: "a".to_string(),
            status: StandingStatus::Tested,
        };
        assert!(missing_requirement(&req, &doc).is_none());
    }

    #[test]
    fn write_standing_doc_helper_is_used_by_future_cli_level_tests() {
        // Smoke-test the fixture helper itself so it doesn't bit-rot unused.
        let dir = tempfile::tempdir().unwrap();
        write_standing_doc(
            dir.path(),
            vec![artifact("a", vec![StandingStatus::Tested])],
        );
        assert!(dir
            .path()
            .join("target/praxis-standing/standing.json")
            .exists());
    }
}
