// src/nouns/sbom.rs — cargo cicd sbom
//
// Thin wrapper delegating to the legacy implementation.
// See CCICD-106 for the deletion milestone.

pub use crate::legacy_nouns::sbom::{SbomNoun, SbomGenerateVerb, SbomShowVerb};
