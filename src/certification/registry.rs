// src/certification/registry.rs
//
// Safety-critical crate registry — query surface.
//
// This registry's construction/query surface is exercised by the
// `tests/certification_policies.rs` integration suite; it is not yet wired
// into a live `cargo cicd certification show` code path, so `cargo build`
// (which does not see the integration-test crate) reports it as dead.

/// Entry in the safety-critical crate registry.
#[derive(Debug, Clone)]
#[allow(
    dead_code,
    reason = "constructed only by tests/certification_policies.rs today; real fields read via is_certified"
)]
pub struct SafetyCriticalEntry {
    /// Crate name on crates.io.
    pub crate_name: String,
    /// Version that was certified.
    pub version: String,
    /// Certification body that issued the receipt.
    pub cert_body_id: String,
    /// Standards satisfied, e.g. `["IEC 61508 SIL 2", "ISO 26262 ASIL B"]`.
    pub standards: Vec<String>,
    /// Date of certification (YYYY-MM-DD).
    pub certified_at: String,
    /// Receipt hash (SHA-256 hex, prefixed with "sha256:").
    pub receipt_hash: String,
    /// Link to a public evidence archive, if available.
    pub evidence_url: Option<String>,
}

impl SafetyCriticalEntry {
    /// Construct a new entry with all required fields.
    #[allow(
        dead_code,
        reason = "constructed only by tests/certification_policies.rs today"
    )]
    pub fn new(
        crate_name: impl Into<String>,
        version: impl Into<String>,
        cert_body_id: impl Into<String>,
        standards: Vec<String>,
        certified_at: impl Into<String>,
        receipt_hash: impl Into<String>,
    ) -> Self {
        SafetyCriticalEntry {
            crate_name: crate_name.into(),
            version: version.into(),
            cert_body_id: cert_body_id.into(),
            standards,
            certified_at: certified_at.into(),
            receipt_hash: receipt_hash.into(),
            evidence_url: None,
        }
    }
}

/// Check if a specific crate + version appears in the registry.
#[allow(
    dead_code,
    reason = "exercised only by tests/certification_policies.rs today"
)]
pub fn is_certified(registry: &[SafetyCriticalEntry], crate_name: &str, version: &str) -> bool {
    registry
        .iter()
        .any(|e| e.crate_name == crate_name && e.version == version)
}
