//! Compact, agent-facing rendering of the standing document: one line per
//! artifact with its ladder rung, scope, top evidence pointer, and the
//! concrete next action needed to reach the next rung.

use cargo_cicd_core::standing::{StandingArtifact, StandingDocument};
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use std::path::{Path, PathBuf};

/// What closes the gap from `ladder_level` to `ladder_level + 1`.
fn next_action(ladder_level: u8) -> &'static str {
    match ladder_level {
        0 => "build it (cargo build, or the configured client build_command)",
        1 => "get it under test (cargo test) to reach TESTED",
        2 => "mint and verify a receipt (cargo cicd receipt verify) to reach RECEIPTED",
        3 => "produce an OCEL process-validation log with is_conforming=true to reach OCEL_PROVEN",
        4 => "produce a wasm4pm conformance proof to reach WASM4PM_PROVEN",
        5 => "demonstrate a clean replay to reach REPLAYABLE (rung 6)",
        6 => "reach PUBLISH_READY with a non-empty scope",
        7 => "reach PILOT_READY",
        8 => "reach PRODUCTION_READY with a non-empty scope (rung 9)",
        _ => "at rung 9 (PRODUCTION_READY_FOR_SCOPE) — maintain evidence freshness",
    }
}

fn top_evidence_pointer(artifact: &StandingArtifact) -> String {
    match artifact.evidence.first() {
        None => "no evidence recorded".to_string(),
        Some(cargo_cicd_core::standing::EvidenceRef::Command { command, .. }) => {
            format!("command: {command}")
        }
        Some(cargo_cicd_core::standing::EvidenceRef::OcelEvent { path, .. }) => {
            format!("ocel: {path}")
        }
        Some(cargo_cicd_core::standing::EvidenceRef::Receipt { path, .. }) => {
            format!("receipt: {path}")
        }
        Some(cargo_cicd_core::standing::EvidenceRef::Artifact { path, .. }) => {
            format!("artifact: {path}")
        }
    }
}

fn render_context_md(doc: &StandingDocument) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# CLAUDE_CODE_CONTEXT — {} (generated {})\n\n",
        doc.release_id, doc.generated_at_utc
    ));
    for artifact in &doc.artifacts {
        let standing_str = artifact
            .standing
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "- {}: standing=[{}], ladder {}, scope={}, evidence: {}. next: {}\n",
            artifact.id,
            standing_str,
            artifact.ladder_level,
            artifact.scope.clone().unwrap_or_else(|| "none".to_string()),
            top_evidence_pointer(artifact),
            next_action(artifact.ladder_level),
        ));
    }
    out
}

#[verb("show")]
pub fn cmd_show(repo: Option<String>) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let json_path = Path::new(&repo_dir)
        .join("target")
        .join("praxis-standing")
        .join("standing.json");
    let content = std::fs::read_to_string(&json_path).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "no standing.json at {}: {e} (run `standing refresh` first)",
            json_path.display()
        ))
    })?;
    let doc: StandingDocument = serde_json::from_str(&content).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "malformed standing.json: {e}"
        ))
    })?;

    let rendered = render_context_md(&doc);
    let out_path: PathBuf = Path::new(&repo_dir)
        .join("target")
        .join("praxis-standing")
        .join("CLAUDE_CODE_CONTEXT.md");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&out_path, &rendered);

    print!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_cicd_core::standing::{ArtifactKind, EvidenceRef, StandingStatus};

    #[test]
    fn next_action_covers_every_rung() {
        for rung in 0..=9u8 {
            assert!(!next_action(rung).is_empty());
        }
    }

    #[test]
    fn render_context_md_includes_next_action_and_evidence_pointer() {
        let doc = StandingDocument {
            release_id: "v26.7.4".to_string(),
            generated_at_utc: "2026-07-06T00:00:00Z".to_string(),
            generator: "test".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![StandingArtifact {
                id: "praxis-graphlaw".to_string(),
                kind: ArtifactKind::RustCrate,
                path: "crates/praxis-graphlaw".to_string(),
                standing: vec![StandingStatus::Tested],
                scope: None,
                ladder_level: 2,
                evidence: vec![EvidenceRef::Command {
                    command: "cargo test -p praxis-graphlaw".to_string(),
                    exit_code: 0,
                    utc: "unix:0".to_string(),
                    artifact: None,
                }],
                external_operator_side_effects: vec![],
            }],
        };
        let rendered = render_context_md(&doc);
        assert!(rendered.contains("praxis-graphlaw"));
        assert!(rendered.contains("ladder 2"));
        assert!(rendered.contains("mint and verify a receipt"));
        assert!(rendered.contains("command: cargo test -p praxis-graphlaw"));
    }
}
