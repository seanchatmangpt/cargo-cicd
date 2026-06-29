//! IEC 61508 / ISO 26262 compliance summary for cargo-cicd certification.
// src/nouns/certification.rs — cargo cicd certification show
//
// Delegates to the legacy implementation while the full migration is in progress.
// See CCICD-106 for the deletion milestone.

#[allow(deprecated)]
pub use crate::legacy_nouns::certification::{CertificationNoun, CertificationShowVerb};

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[allow(deprecated)]
#[verb("show")]
/// Print IEC 61508 / ISO 26262 compliance summary.
pub fn cmd_show() -> Result<()> {
    CertificationNoun::run_direct()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
