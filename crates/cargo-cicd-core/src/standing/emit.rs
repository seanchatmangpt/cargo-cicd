//! Standing document writers: full JSON/TTL emission plus focused
//! sub-slices (benchmark/receipt/client-surface/claim-index summaries and
//! LSP diagnostics).
//!
//! ## Two distinct OCEL emissions — do not conflate them
//!
//! 1. The append-only, hash-chained `standing_compiled` event JSONL ledger
//!    (`.cargo-cicd/ocel/events.jsonl`) is deliberately **not** built here:
//!    it needs `src/ocel.rs`'s `OcelLog`/`append_ocel_event` from the main
//!    `cargo-cicd` crate, and `cargo-cicd-core` sits below that crate in the
//!    dependency graph (the reverse dependency would be a cycle). That
//!    writer lives in the main crate (`src/nouns/standing.rs`).
//! 2. [`render_standing_ocel_shape_a`] / [`write_standing_ocel_shape_a`]
//!    build a **self-contained Shape-A OCEL 2.0 snapshot**
//!    (`target/praxis-standing/standing.ocel.json`, the
//!    `{eventTypes, objectTypes, events, objects}` shape consumed by
//!    `wasm4pm_compat::ocel::OCEL`) purely from a `StandingDocument` — no
//!    hash chain, no append-only ledger, just "what does the current
//!    standing document look like as one OCEL log". That only needs this
//!    module's own inputs, so it lives here.

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
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Render the standing document as a minimal Turtle graph: one
/// `praxis:StandingArtifact` resource per artifact, with `praxis:standing`
/// (repeated), `praxis:ladderLevel`, and `praxis:evidence` (repeated,
/// stringified) literals. Hand-templated — no RDF crate dependency, per the
/// "smallest diff, reuse first" invariant.
///
/// Deliberately **excludes** `generated_at_utc`: that field is a wall-clock
/// timestamp that changes on every run even when the underlying artifact
/// state is unchanged. Embedding it here would make the TTL non-deterministic
/// (different content hash every run for identical input), which defeats
/// content-addressed caching (e.g. praxis's `ggen.lock`). The timestamp is
/// still recorded — in `standing.json` (the full `StandingDocument` dump) —
/// it is simply not part of this derived, hash-stable TTL projection.
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
        "  praxis:generator \"{}\" .\n\n",
        ttl_escape(&doc.generator)
    ));

    for artifact in &doc.artifacts {
        out.push_str(&format!(
            "praxis:artifact-{} a praxis:StandingArtifact ;\n",
            ttl_local_name(&artifact.id)
        ));
        out.push_str(&format!("  praxis:id \"{}\" ;\n", ttl_escape(&artifact.id)));
        out.push_str(&format!("  praxis:kind \"{:?}\" ;\n", artifact.kind));
        out.push_str(&format!(
            "  praxis:path \"{}\" ;\n",
            ttl_escape(&artifact.path)
        ));
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
            out.push_str(&format!(
                "  praxis:evidence \"{}\" ;\n",
                ttl_escape(&rendered)
            ));
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
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Write the standing document as a Turtle graph.
pub fn write_standing_ttl(doc: &StandingDocument, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_standing_ttl(doc))
}

// ── Shape-A OCEL snapshot ────────────────────────────────────────────────

/// Render the standing document as a self-contained Shape-A OCEL 2.0 log:
/// one `standing_compiled` event per artifact, each linked by an
/// `artifact` qualifier to a `standing_artifact` object carrying that
/// artifact's kind. Matches the field names and casing
/// `wasm4pm_compat::ocel::OCEL` expects (`eventTypes`, `objectTypes`,
/// `events[].type`/`time`/`attributes`/`relationships`, `objects[].type`/
/// `attributes`/`relationships`) so it can be parsed by that type directly.
///
/// Every event/object time is `doc.generated_at_utc` — the one wall-clock
/// reading this snapshot is allowed to carry, since (unlike `standing.ttl`)
/// an OCEL log is expected to be time-stamped and is not asserted to be
/// byte-identical across runs.
pub fn render_standing_ocel_shape_a(doc: &StandingDocument) -> serde_json::Value {
    let event_types = serde_json::json!([{
        "name": "standing_compiled",
        "attributes": [
            {"name": "artifact_id", "type": "string"},
            {"name": "kind", "type": "string"},
            {"name": "standing", "type": "string"},
            {"name": "ladder_level", "type": "integer"},
            {"name": "scope", "type": "string"},
        ],
    }]);
    let object_types = serde_json::json!([{
        "name": "standing_artifact",
        "attributes": [
            {"name": "kind", "type": "string"},
        ],
    }]);

    let events: Vec<serde_json::Value> = doc
        .artifacts
        .iter()
        .map(|a| {
            let standing_str = a
                .standing
                .iter()
                .map(|s| format!("{s:?}"))
                .collect::<Vec<_>>()
                .join(",");
            serde_json::json!({
                "id": format!("standing_compiled:{}", a.id),
                "type": "standing_compiled",
                "time": doc.generated_at_utc,
                "attributes": [
                    {"name": "artifact_id", "value": a.id},
                    {"name": "kind", "value": format!("{:?}", a.kind)},
                    {"name": "standing", "value": standing_str},
                    {"name": "ladder_level", "value": a.ladder_level},
                    {"name": "scope", "value": a.scope.clone().unwrap_or_default()},
                ],
                "relationships": [
                    {"objectId": a.id, "qualifier": "artifact"},
                ],
            })
        })
        .collect();

    let objects: Vec<serde_json::Value> = doc
        .artifacts
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "type": "standing_artifact",
                "attributes": [
                    {"name": "kind", "value": format!("{:?}", a.kind), "time": doc.generated_at_utc},
                ],
                "relationships": [],
            })
        })
        .collect();

    serde_json::json!({
        "eventTypes": event_types,
        "objectTypes": object_types,
        "events": events,
        "objects": objects,
    })
}

/// Write the Shape-A OCEL snapshot as pretty JSON.
pub fn write_standing_ocel_shape_a(doc: &StandingDocument, path: &Path) -> io::Result<()> {
    write_json_pretty(&render_standing_ocel_shape_a(doc), path)
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
    write_json_pretty(
        &build_benchmark_summary(doc),
        &out_dir.join("benchmark-summary.json"),
    )?;
    write_json_pretty(
        &build_receipt_summary(doc),
        &out_dir.join("receipt-summary.json"),
    )?;
    write_json_pretty(
        &build_client_surface_summary(doc),
        &out_dir.join("client-surface-summary.json"),
    )?;
    write_json_pretty(&build_claim_index(doc), &out_dir.join("claim-index.json"))?;
    write_json_pretty(
        &build_lsp_diagnostics(doc),
        &out_dir.join("LSP_DIAGNOSTICS.json"),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standing::model::{EvidenceRef, StandingStatus};

    fn sample_doc() -> StandingDocument {
        StandingDocument {
            schema_id: crate::standing::model::STANDING_SCHEMA_ID.to_string(),
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

    /// The TTL must never embed `generated_at_utc` — that timestamp lives in
    /// `standing.json` only. This is what makes the TTL byte-identical across
    /// runs with unchanged artifact state (see `ttl_is_deterministic_across_runs`).
    #[test]
    fn ttl_does_not_contain_timestamp() {
        let ttl = render_standing_ttl(&sample_doc());
        assert!(!ttl.contains("generatedAtUtc"));
        assert!(!ttl.contains("2026-07-06T00:00:00Z"));
    }

    /// Two renders of the same `StandingDocument`, differing only in
    /// `generated_at_utc`, must produce byte-identical TTL output. This is
    /// the core determinism guarantee that lets consumers (e.g. praxis's
    /// `ggen.lock` content-addressed cache) treat unchanged-input runs as a
    /// no-op instead of needing to `rm -f ggen.lock` before every run.
    #[test]
    fn ttl_is_deterministic_across_runs() {
        let mut doc_a = sample_doc();
        doc_a.generated_at_utc = "2026-07-06T00:00:00Z".to_string();
        let mut doc_b = sample_doc();
        doc_b.generated_at_utc = "2099-01-01T12:34:56Z".to_string();

        let ttl_a = render_standing_ttl(&doc_a);
        let ttl_b = render_standing_ttl(&doc_b);
        assert_eq!(
            ttl_a, ttl_b,
            "TTL output must be independent of generated_at_utc"
        );

        // Also confirm stability across repeated calls with the identical doc.
        let ttl_a2 = render_standing_ttl(&doc_a);
        assert_eq!(ttl_a, ttl_a2);
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
    fn ocel_shape_a_has_one_event_and_object_per_artifact() {
        let doc = sample_doc();
        let ocel = render_standing_ocel_shape_a(&doc);
        assert_eq!(
            ocel["events"].as_array().unwrap().len(),
            doc.artifacts.len()
        );
        assert_eq!(
            ocel["objects"].as_array().unwrap().len(),
            doc.artifacts.len()
        );
        assert_eq!(ocel["eventTypes"][0]["name"], "standing_compiled");
        assert_eq!(ocel["objectTypes"][0]["name"], "standing_artifact");
    }

    #[test]
    fn ocel_shape_a_event_relationship_points_at_matching_object() {
        let ocel = render_standing_ocel_shape_a(&sample_doc());
        let first_event = &ocel["events"][0];
        let first_object = &ocel["objects"][0];
        assert_eq!(
            first_event["relationships"][0]["objectId"],
            first_object["id"]
        );
        assert_eq!(first_event["relationships"][0]["qualifier"], "artifact");
    }

    #[test]
    fn write_standing_ocel_shape_a_round_trips_as_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("standing.ocel.json");
        write_standing_ocel_shape_a(&sample_doc(), &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["events"].is_array());
        assert!(parsed["objects"].is_array());
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
