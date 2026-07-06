//! Standing schema v1 (`cicd-standing.v1`).
//!
//! Types matching `docs/reference/standing-schema.md` (schema of record).
//! These are the compiled, machine-checked counterpart of that document:
//! keep field names and status tags identical to the schema doc when
//! either side changes.
//!
//! ## Schema id
//!
//! The canonical schema id is [`STANDING_SCHEMA_ID`] (`cicd-standing.v1`).
//! The legacy id `praxis-standing.v1` — used before the standing compiler
//! was collapsed into `cargo-cicd` — is still accepted on read via
//! [`is_recognized_schema_id`] so standing documents emitted by older
//! versions (and by consumers still on the old id) keep parsing. New
//! documents always emit the canonical id ([`default_schema_id`]).

use serde::{Deserialize, Serialize};

/// Canonical standing schema id emitted by this version of cargo-cicd.
pub const STANDING_SCHEMA_ID: &str = "cicd-standing.v1";

/// Legacy schema id, accepted as an alias on read for backward
/// compatibility with standing documents emitted before the schema id
/// rename (previously the *canonical* id, now deprecated in favor of
/// [`STANDING_SCHEMA_ID`]).
pub const STANDING_SCHEMA_ID_ALIAS_PRAXIS: &str = "praxis-standing.v1";

/// Default used by `#[serde(default)]` when deserializing documents that
/// predate the `schema_id` field entirely.
fn default_schema_id() -> String {
    STANDING_SCHEMA_ID.to_string()
}

/// True if `id` is either the canonical schema id or a recognized legacy
/// alias. Used to validate/accept a `schema_id` on read without rejecting
/// still-valid older artifacts.
pub fn is_recognized_schema_id(id: &str) -> bool {
    id == STANDING_SCHEMA_ID || id == STANDING_SCHEMA_ID_ALIAS_PRAXIS
}

/// Errors constructing or validating a [`StandingArtifact`] / [`StandingDocument`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StandingError {
    /// One of the readiness statuses (`PRODUCTION_READY`, `PILOT_READY`,
    /// `PUBLISH_READY`, `PUBLICATION_READY`) is present without a
    /// non-empty `scope` string.
    #[error(
        "artifact `{id}` carries {status:?} without a non-empty scope (scoped-readiness rule)"
    )]
    MissingScope { id: String, status: StandingStatus },
}

/// The 20 standing statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StandingStatus {
    Unseen,
    Discovered,
    Builds,
    Tested,
    LintClean,
    Benchmarked,
    Receipted,
    ReceiptVerified,
    OcelProven,
    Wasm4pmProven,
    ClientVisible,
    PublicationReady,
    PublishReady,
    PilotReady,
    ProductionReady,
    ExternalOperatorSideEffect,
    NonStanding,
    Quarantined,
    Retired,
    Duplicate,
}

impl StandingStatus {
    /// True for the four readiness statuses that require a non-empty `scope`.
    pub fn requires_scope(&self) -> bool {
        matches!(
            self,
            Self::ProductionReady | Self::PilotReady | Self::PublishReady | Self::PublicationReady
        )
    }

    /// This status's rung on the 0-9 readiness ladder, if it sits on the
    /// ladder at all. Statuses off the ladder (e.g. `LintClean`,
    /// `Benchmarked`, `ReceiptVerified`) return `None`.
    pub fn ladder_rung(&self, scope: Option<&str>) -> Option<u8> {
        match self {
            Self::Discovered => Some(0),
            Self::Builds => Some(1),
            Self::Tested => Some(2),
            Self::Receipted => Some(3),
            Self::OcelProven => Some(4),
            Self::Wasm4pmProven => Some(5),
            // REPLAYABLE (rung 6) has no dedicated status in the v1 status
            // list; it is reached implicitly once verified receipts and
            // OCEL/wasm4pm proof are combined by an upstream policy, and is
            // not computed from a single status here.
            Self::PublishReady => Some(7),
            Self::PilotReady => Some(8),
            Self::ProductionReady => {
                // Rung 9 (PRODUCTION_READY_FOR_SCOPE) requires a non-empty scope.
                scope.filter(|s| !s.is_empty()).map(|_| 9).or(
                    Some(8), /* falls back to PILOT_READY's rung if unscoped */
                )
            }
            _ => None,
        }
    }
}

/// The 0-9 readiness ladder, computed from an artifact's `standing` set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LadderLevel(pub u8);

impl std::fmt::Display for LadderLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Compute the readiness ladder level (0-9) from a set of statuses and the
/// artifact's optional scope. This is the max ladder rung among the
/// statuses present; statuses not on the ladder do not contribute.
pub fn compute_ladder_level(standing: &[StandingStatus], scope: Option<&str>) -> u8 {
    standing
        .iter()
        .filter_map(|s| s.ladder_rung(scope))
        .max()
        .unwrap_or(0)
}

/// One piece of evidence backing a standing claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceRef {
    Command {
        command: String,
        exit_code: i32,
        utc: String,
        artifact: Option<String>,
    },
    OcelEvent {
        event_id: String,
        path: String,
    },
    Receipt {
        chain_hash: String,
        path: String,
    },
    Artifact {
        path: String,
        hash: String,
    },
}

/// Kind of artifact being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    RustCrate,
    Client,
    Doc,
    Paper,
    Bench,
    Workflow,
    Ontology,
    Binary,
}

/// A single tracked artifact's standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingArtifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub path: String,
    pub standing: Vec<StandingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub ladder_level: u8,
    pub evidence: Vec<EvidenceRef>,
    pub external_operator_side_effects: Vec<String>,
}

impl StandingArtifact {
    /// Enforce the scoped-readiness validation rule: any of
    /// `PRODUCTION_READY` / `PILOT_READY` / `PUBLISH_READY` /
    /// `PUBLICATION_READY` present in `standing` requires a non-empty
    /// `scope`. Returns a typed error rather than panicking or silently
    /// defaulting.
    pub fn validate(&self) -> Result<(), StandingError> {
        let scope_ok = self.scope.as_deref().is_some_and(|s| !s.is_empty());
        if scope_ok {
            return Ok(());
        }
        for status in &self.standing {
            if status.requires_scope() {
                return Err(StandingError::MissingScope {
                    id: self.id.clone(),
                    status: *status,
                });
            }
        }
        Ok(())
    }
}

/// Top-level compiled standing document for a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingDocument {
    /// Schema id: [`STANDING_SCHEMA_ID`] for documents from this version,
    /// or the accepted legacy alias [`STANDING_SCHEMA_ID_ALIAS_PRAXIS`] for
    /// older/external documents. Defaults to the canonical id when absent
    /// (older documents predating this field).
    #[serde(default = "default_schema_id")]
    pub schema_id: String,
    pub release_id: String,
    pub generated_at_utc: String,
    pub generator: String,
    pub standing_version: String,
    pub artifacts: Vec<StandingArtifact>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graphlaw_example() -> StandingArtifact {
        StandingArtifact {
            id: "praxis-graphlaw".to_string(),
            kind: ArtifactKind::RustCrate,
            path: "crates/praxis-graphlaw".to_string(),
            standing: vec![
                StandingStatus::Builds,
                StandingStatus::Tested,
                StandingStatus::ReceiptVerified,
                StandingStatus::OcelProven,
                StandingStatus::Wasm4pmProven,
                StandingStatus::PublishReady,
            ],
            scope: Some("local release validation and crates.io dry-run".to_string()),
            ladder_level: 7,
            evidence: vec![EvidenceRef::Command {
                command: "cargo test -p praxis-graphlaw".to_string(),
                exit_code: 0,
                utc: "2026-07-06T19:00:00Z".to_string(),
                artifact: None,
            }],
            external_operator_side_effects: vec![
                "real crates.io publish requires operator credentials".to_string(),
            ],
        }
    }

    #[test]
    fn valid_artifact_round_trips_through_serde_json() {
        let artifact = graphlaw_example();
        artifact.validate().expect("valid artifact must validate");
        let json = serde_json::to_string(&artifact).expect("serialize");
        let back: StandingArtifact = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(artifact, back);
        assert!(json.contains("PUBLISH_READY"));
    }

    #[test]
    fn production_ready_without_scope_fails_validate() {
        let mut artifact = graphlaw_example();
        artifact.standing.push(StandingStatus::ProductionReady);
        artifact.scope = None;
        let err = artifact.validate().expect_err("must reject missing scope");
        assert!(matches!(
            err,
            StandingError::MissingScope { ref id, .. } if id == "praxis-graphlaw"
        ));
    }

    #[test]
    fn production_ready_with_empty_scope_fails_validate() {
        let mut artifact = graphlaw_example();
        artifact.standing.push(StandingStatus::ProductionReady);
        artifact.scope = Some(String::new());
        assert!(artifact.validate().is_err());
    }

    #[test]
    fn ladder_level_takes_max_of_ladder_statuses() {
        let level = compute_ladder_level(
            &[
                StandingStatus::Discovered,
                StandingStatus::Builds,
                StandingStatus::Tested,
                StandingStatus::OcelProven,
            ],
            None,
        );
        assert_eq!(level, 4); // OCEL_PROVEN rung, highest present

        let level = compute_ladder_level(&[StandingStatus::Discovered], None);
        assert_eq!(level, 0);

        let level = compute_ladder_level(&[StandingStatus::ProductionReady], Some("prod scope"));
        assert_eq!(level, 9);

        let level = compute_ladder_level(&[StandingStatus::ProductionReady], None);
        assert_eq!(level, 8); // unscoped PRODUCTION_READY falls back to rung 8
    }

    #[test]
    fn non_ladder_statuses_do_not_raise_ladder_level() {
        let level = compute_ladder_level(
            &[
                StandingStatus::Discovered,
                StandingStatus::LintClean,
                StandingStatus::Benchmarked,
                StandingStatus::ReceiptVerified,
            ],
            None,
        );
        assert_eq!(level, 0);
    }

    fn empty_doc_json(schema_id: &str) -> String {
        format!(
            r#"{{"schema_id":"{}","release_id":"r","generated_at_utc":"t","generator":"g","standing_version":"1","artifacts":[]}}"#,
            schema_id
        )
    }

    #[test]
    fn canonical_schema_id_parses() {
        let json = empty_doc_json(STANDING_SCHEMA_ID);
        let doc: StandingDocument = serde_json::from_str(&json).expect("canonical id must parse");
        assert_eq!(doc.schema_id, STANDING_SCHEMA_ID);
        assert!(is_recognized_schema_id(&doc.schema_id));
    }

    #[test]
    fn legacy_praxis_schema_id_still_parses_as_accepted_alias() {
        let json = empty_doc_json(STANDING_SCHEMA_ID_ALIAS_PRAXIS);
        let doc: StandingDocument =
            serde_json::from_str(&json).expect("legacy alias id must still parse");
        assert_eq!(doc.schema_id, STANDING_SCHEMA_ID_ALIAS_PRAXIS);
        assert!(is_recognized_schema_id(&doc.schema_id));
    }

    #[test]
    fn document_missing_schema_id_field_defaults_to_canonical() {
        // Documents written before the `schema_id` field existed at all.
        let json = r#"{"release_id":"r","generated_at_utc":"t","generator":"g","standing_version":"1","artifacts":[]}"#;
        let doc: StandingDocument =
            serde_json::from_str(json).expect("document without schema_id must still parse");
        assert_eq!(doc.schema_id, STANDING_SCHEMA_ID);
    }

    #[test]
    fn unrecognized_schema_id_is_not_recognized() {
        assert!(!is_recognized_schema_id("some-other-schema.v1"));
    }

    #[test]
    fn new_emissions_default_to_canonical_schema_id() {
        let doc = StandingDocument {
            schema_id: default_schema_id(),
            release_id: "r".to_string(),
            generated_at_utc: "t".to_string(),
            generator: "g".to_string(),
            standing_version: "1".to_string(),
            artifacts: vec![],
        };
        assert_eq!(doc.schema_id, STANDING_SCHEMA_ID);
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("cicd-standing.v1"));
    }
}
