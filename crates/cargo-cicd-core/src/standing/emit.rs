//! Standing document writers: full JSON/TTL emission plus focused
//! sub-slices (benchmark/receipt/client-surface/claim-index summaries and
//! LSP diagnostics).
//!
//! OCEL emission (`standing_compiled` events) is deliberately **not** here:
//! it needs `src/ocel.rs`'s `OcelLog`/`append_ocel_event` from the main
//! `cargo-cicd` crate, and `cargo-cicd-core` sits below that crate in the
//! dependency graph (the reverse dependency would be a cycle). That writer
//! lives in the main crate (`src/nouns/standing.rs`), calling
//! `append_ocel_event` directly against the `StandingDocument` this module
//! produces — the OCEL *writer* is still reused as-is, just from the layer
//! that already owns it.

use crate::standing::model::{ArtifactKind, StandingArtifact, StandingDocument};
use std::io;
use std::path::Path;

fn write_json_pretty<T: serde::Serialize>(value: &T, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Write the full standing document as pretty JSON.
pub fn write_standing_json(doc: &StandingDocument, path: &Path) -> io::Result<()> {
    write_json_pretty(doc, path)
}

/// Escape a string for embedding in a Turtle string literal.
fn ttl_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

/// Render the standing document as a minimal Turtle graph: one
/// `praxis:StandingArtifact` resource per artifact, with `praxis:standing`
/// (repeated), `praxis:ladderLevel`, and `praxis:evidence` (repeated,
/// stringified) literals. Hand-templated — no RDF crate dependency, per the
/// "smallest diff, reuse first" invariant.
pub fn render_standing_ttl(doc: &StandingDocument) -> String {
    let mut out = String::new();
    out.push_str("@prefix praxis: <https://praxis.dev/ontology/standing#> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

    out.push_str(&format!(
        "praxis:StandingDocument-{} a praxis:StandingDocument ;\n",
        ttl_local_name(&doc.release_id)
    ));
    out.push_str(&format!(
        "  praxis:releaseId \"{}\" ;\n",
        ttl_escape(&doc.release_id)
    ));
    out.push_str(&format!(
        "  praxis:generatedAtUtc \"{}\" ;\n",
        ttl_escape(&doc.generated_at_utc)
    ));
    out.push_str(&format!(
        "  praxis:generator \"{}\" .\n\n",
        ttl_escape(&doc.generator)
    ));

    for artifact in &doc.artifacts {
        out.push_str(&format!(
            "praxis:artifact-{} a praxis:StandingArtifact ;\n",
            ttl_local_name(&artifact.id)
        ));
        out.push_str(&format!("  praxis:id \"{}\" ;\n", ttl_escape(&artifact.id)));
        out.push_str(&format!(
            "  praxis:kind \"{:?}\" ;\n",
            artifact.kind
        ));
        out.push_str(&format!("  praxis:path \"{}\" ;\n", ttl_escape(&artifact.path)));
        for status in &artifact.standing {
            out.push_str(&format!("  praxis:standing \"{:?}\" ;\n", status));
        }
        out.push_str(&format!(
            "  praxis:ladderLevel \"{}\"^^xsd:integer ;\n",
            artifact.ladder_level
        ));
        if let Some(scope) = &artifact.scope {
            out.push_str(&format!("  praxis:scope \"{}\" ;\n", ttl_escape(scope)));
        }
        for evidence in &artifact.evidence {
            let rendered = serde_json::to_string(evidence).unwrap_or_default();
            out.push_str(&format!("  praxis:evidence \"{}\" ;\n", ttl_escape(&rendered)));
        }
        // Replace the trailing " ;\n" with " .\n\n" to close the resource.
        if out.ends_with(" ;\n") {
            out.truncate(out.len() - 3);
            out.push_str(" .\n\n");
        }
    }

    out
}

/// Turtle-safe local name: alphanumerics, `-`, `_` only.
fn ttl_local_name(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Write the standing document as a Turtle graph.
pub fn write_standing_ttl(doc: &StandingDocument, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_standing_ttl(doc))
}

// ── Focused sub-slices ───────────────────────────────────────────────────

/// Artifacts carrying `BENCHMARKED` in their standing set.
pub fn build_benchmark_summary(doc: &StandingDocument) -> serde_json::Value {
    let artifacts: Vec<&StandingArtifact> = doc
        .artifacts
        .iter()
        .filter(|a| {
            a.standing
                .contains(&crate::standing::model::StandingStatus::Benchmarked)
        })
        .collect();
    serde_json::json!({
        "schema": "cargo-cicd.standing.benchmark-summary.v1",
        "release_id": doc.release_id,
        "count": artifacts.len(),
        "artifacts": artifacts,
    })
}

/// Artifacts carrying `RECEIPTED` or `RECEIPT_VERIFIED`.
pub fn build_receipt_summary(doc: &StandingDocument) -> serde_json::Value {
    use crate::standing::model::StandingStatus;
    let artifacts: Vec<&StandingArtifact> = doc
        .artifacts
        .iter()
        .filter(|a| {
            a.standing.contains(&StandingStatus::Receipted)
                || a.standing.contains(&StandingStatus::ReceiptVerified)
        })
        .collect();
    serde_json::json!({
        "schema": "cargo-cicd.standing.receipt-summary.v1",
        "release_id": doc.release_id,
        "count": artifacts.len(),
        "artifacts": artifacts,
    })
}

/// Artifacts of kind `client`.
pub fn build_client_surface_summary(doc: &StandingDocument) -> serde_json::Value {
    let artifacts: Vec<&StandingArtifact> = doc
        .artifacts
        .iter()
        .filter(|a| matches!(a.kind, ArtifactKind::Client))
        .collect();
    serde_json::json!({
        "schema": "cargo-cicd.standing.client-surface-summary.v1",
        "release_id": doc.release_id,
        "count": artifacts.len(),
        "artifacts": artifacts,
    })
}

/// Artifacts ingested from claim tables (id prefixed `claim:` by
/// `ingest_claim_tables`). Explicitly informational, per the ingestor's
/// contract — never used as authoritative standing.
pub fn build_claim_index(doc: &StandingDocument) -> serde_json::Value {
    let artifacts: Vec<&StandingArtifact> = doc
        .artifacts
        .iter()
        .filter(|a| a.id.starts_with("claim:"))
        .collect();
    serde_json::json!({
        "schema": "cargo-cicd.standing.claim-index.v1",
        "release_id": doc.release_id,
        "authoritative": false,
        "count": artifacts.len(),
        "artifacts": artifacts,
    })
}

/// One line per artifact whose claimed standing lacks any evidence at all.
pub fn build_lsp_diagnostics(doc: &StandingDocument) -> serde_json::Value {
    let diagnostics: Vec<serde_json::Value> = doc
        .artifacts
        .iter()
        .filter(|a| !a.standing.is_empty() && a.evidence.is_empty())
        .map(|a| {
            serde_json::json!({
                "artifact_id": a.id,
                "issue": "standing_without_evidence",
                "detail": format!(
                    "artifact claims {:?} but carries zero evidence entries",
                    a.standing
                ),
            })
        })
        .collect();
    serde_json::Value::Array(diagnostics)
}

/// Write all four focused sub-slices plus `LSP_DIAGNOSTICS.json` into
/// `out_dir`, using the canonical filenames.
pub fn write_summaries(doc: &StandingDocument, out_dir: &Path) -> io::Result<()> {
    write_json_pretty(&build_benchmark_summary(doc), &out_dir.join("benchmark-summary.json"))?;
    write_json_pretty(&build_receipt_summary(doc), &out_dir.join("receipt-summary.json"))?;
    write_json_pretty(
        &build_client_surface_summary(doc),
        &out_dir.join("client-surface-summary.json"),
    )?;
    write_json_pretty(&build_claim_index(doc), &out_dir.join("claim-index.json"))?;
    write_json_pretty(&build_lsp_diagnostics(doc), &out_dir.join("LSP_DIAGNOSTICS.json"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standing::model::{EvidenceRef, StandingStatus};

    fn sample_doc() -> StandingDocument {
        StandingDocument {
            release_id: "v26.7.4".to_string(),
            generated_at_utc: "2026-07-06T00:00:00Z".to_string(),
            generator: "cargo-cicd-standing".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![
                StandingArtifact {
                    id: "praxis-graphlaw".to_string(),
                    kind: ArtifactKind::RustCrate,
                    path: "crates/praxis-graphlaw".to_string(),
                    standing: vec![StandingStatus::Benchmarked, StandingStatus::Receipted],
                    scope: None,
                    ladder_level: 3,
                    evidence: vec![EvidenceRef::Receipt {
                        chain_hash: "abc".to_string(),
                        path: "ledger.jsonl".to_string(),
                    }],
                    external_operator_side_effects: vec![],
                },
                StandingArtifact {
                    id: "claim:CLAIM_PROMOTION_TABLE".to_string(),
                    kind: ArtifactKind::Doc,
                    path: "docs/CLAIM_PROMOTION_TABLE.md".to_string(),
                    standing: vec![StandingStatus::Discovered],
                    scope: None,
                    ladder_level: 0,
                    evidence: vec![],
                    external_operator_side_effects: vec![],
                },
                StandingArtifact {
                    id: "web-client".to_string(),
                    kind: ArtifactKind::Client,
                    path: "clients/web".to_string(),
                    standing: vec![StandingStatus::Discovered, StandingStatus::Builds],
                    scope: None,
                    ladder_level: 1,
                    evidence: vec![],
                    external_operator_side_effects: vec![],
                },
            ],
        }
    }

    #[test]
    fn write_standing_json_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("standing.json");
        write_standing_json(&sample_doc(), &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let back: StandingDocument = serde_json::from_str(&content).unwrap();
        assert_eq!(back.artifacts.len(), 3);
    }

    #[test]
    fn ttl_contains_ladder_and_standing() {
        let ttl = render_standing_ttl(&sample_doc());
        assert!(ttl.contains("praxis:StandingArtifact"));
        assert!(ttl.contains("praxis:ladderLevel \"3\"^^xsd:integer"));
        assert!(ttl.contains("Benchmarked") || ttl.contains("Receipted"));
    }

    #[test]
    fn benchmark_summary_filters_correctly() {
        let summary = build_benchmark_summary(&sample_doc());
        assert_eq!(summary["count"], 1);
    }

    #[test]
    fn receipt_summary_filters_correctly() {
        let summary = build_receipt_summary(&sample_doc());
        assert_eq!(summary["count"], 1);
    }

    #[test]
    fn client_surface_summary_filters_by_kind() {
        let summary = build_client_surface_summary(&sample_doc());
        assert_eq!(summary["count"], 1);
    }

    #[test]
    fn claim_index_is_marked_non_authoritative() {
        let index = build_claim_index(&sample_doc());
        assert_eq!(index["authoritative"], false);
        assert_eq!(index["count"], 1);
    }

    #[test]
    fn lsp_diagnostics_flags_standing_without_evidence() {
        let diagnostics = build_lsp_diagnostics(&sample_doc());
        let arr = diagnostics.as_array().unwrap();
        // "claim:..." and "web-client" both have non-empty standing + empty evidence.
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn write_summaries_creates_all_five_files() {
        let dir = tempfile::tempdir().unwrap();
        write_summaries(&sample_doc(), dir.path()).unwrap();
        for name in [
            "benchmark-summary.json",
            "receipt-summary.json",
            "client-surface-summary.json",
            "claim-index.json",
            "LSP_DIAGNOSTICS.json",
        ] {
            assert!(dir.path().join(name).exists(), "missing {name}");
        }
    }
}
