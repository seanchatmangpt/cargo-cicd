// src/certification/mod.rs
//
// Certification body integration, regulatory compliance mappings, and the
// safety-critical crate registry.
//
// Vision 2030 — Phase 1, Weeks 3-7: Safety & Certification.

pub mod iec_61508;
pub mod iso_26262;
pub mod registry;
pub mod soc2;
pub mod togaf;

/// A certification body that can issue process evidence receipts.
///
/// `id` and `submission_url` are read by `cert_body_recommendation`, and
/// `oracle_fingerprint` is reserved for the not-yet-wired receipt-verification
/// path; all three are exercised today only via the
/// `tests/certification_policies.rs` integration suite, which the `cargo
/// build` dead-code scan (rooted at `main()`, not the test crate) cannot see.
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "fields read by cert_body_recommendation, exercised by integration tests, not yet a live CLI path")]
pub struct CertificationBody {
    /// Short identifier, e.g. "ferrous-systems".
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// URL to the submission API or web form.
    pub submission_url: String,
    /// Regulatory compliance standards this body supports.
    pub standards: Vec<ComplianceStandard>,
    /// Oracle public key fingerprint (SHA-256 hex).
    pub oracle_fingerprint: String,
}

/// Supported regulatory compliance standards.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceStandard {
    /// IEC 61508 functional safety — SIL 1 through SIL 4.
    Iec61508 { sil_level: u8 },
    /// ISO 26262 automotive functional safety — ASIL A through ASIL D.
    Iso26262 { asil_level: char },
    /// DO-178C airborne software standard — DAL A through DAL E. No body in
    /// `known_cert_bodies()` registers this yet; kept so `standards_match`'s
    /// DAL-ordering path and future body registrations have a variant to use.
    #[allow(dead_code, reason = "no registered body offers DO-178C yet; taxonomy placeholder for a future known_cert_bodies() entry")]
    Do178c { dal_level: char },
    /// FDA 21 CFR Part 11 electronic records and signatures. Same status as
    /// `Do178c` — modeled, not yet backed by a registered body.
    #[allow(dead_code, reason = "no registered body offers FDA 21 CFR Part 11 yet; taxonomy placeholder for a future known_cert_bodies() entry")]
    Fda21CfrPart11,
    /// Organisation-specific or domain-specific custom standard.
    Custom(String),
}

impl ComplianceStandard {
    /// Full display name of the standard.
    pub fn display_name(&self) -> String {
        match self {
            ComplianceStandard::Iec61508 { sil_level } => {
                format!("IEC 61508 SIL {}", sil_level)
            }
            ComplianceStandard::Iso26262 { asil_level } => {
                format!("ISO 26262 ASIL {}", asil_level)
            }
            ComplianceStandard::Do178c { dal_level } => {
                format!("DO-178C DAL {}", dal_level)
            }
            ComplianceStandard::Fda21CfrPart11 => "FDA 21 CFR Part 11".to_string(),
            ComplianceStandard::Custom(name) => name.clone(),
        }
    }
}

/// Return the registry of known certification bodies.
///
/// This list is seeded with well-known Rust-ecosystem safety specialists.
/// Additional bodies can be registered via the certification body integration
/// guide at `docs/CERT-BODY-INTEGRATION.md`.
pub fn known_cert_bodies() -> Vec<CertificationBody> {
    vec![
        CertificationBody {
            id: "ferrous-systems".to_string(),
            name: "Ferrous Systems GmbH".to_string(),
            submission_url: "https://ferrous-systems.com/ferrocene/".to_string(),
            standards: vec![
                ComplianceStandard::Iec61508 { sil_level: 2 },
                ComplianceStandard::Iec61508 { sil_level: 3 },
                ComplianceStandard::Iso26262 { asil_level: 'B' },
                ComplianceStandard::Iso26262 { asil_level: 'D' },
            ],
            oracle_fingerprint: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
                .to_string(),
        },
        CertificationBody {
            id: "trustinsoft".to_string(),
            name: "TrustInSoft".to_string(),
            submission_url: "https://trust-in-soft.com/tis-analyzer/".to_string(),
            standards: vec![
                ComplianceStandard::Iec61508 { sil_level: 1 },
                ComplianceStandard::Iec61508 { sil_level: 2 },
                ComplianceStandard::Iec61508 { sil_level: 3 },
                ComplianceStandard::Iec61508 { sil_level: 4 },
            ],
            oracle_fingerprint: "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3"
                .to_string(),
        },
        CertificationBody {
            id: "trail-of-bits".to_string(),
            name: "Trail of Bits".to_string(),
            submission_url: "https://www.trailofbits.com/services/security-assessment/".to_string(),
            standards: vec![
                ComplianceStandard::Custom("Trail of Bits Rust Security Assessment".to_string()),
                ComplianceStandard::Custom("SLSA Level 3 Supply Chain Audit".to_string()),
            ],
            oracle_fingerprint: "c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"
                .to_string(),
        },
    ]
}

/// Return all certification bodies that support a given compliance standard.
///
/// Matching is by standard variant type and, for leveled standards, the body
/// must support the requested level or higher.
///
/// Exercised by `tests/certification_policies.rs`; not yet called from a live
/// CLI path, so the `main()`-rooted dead-code scan cannot see that use.
#[allow(dead_code, reason = "exercised by tests/certification_policies.rs, not yet wired into a live CLI path")]
pub fn bodies_for_standard(standard: &ComplianceStandard) -> Vec<CertificationBody> {
    known_cert_bodies()
        .into_iter()
        .filter(|body| body_supports(body, standard))
        .collect()
}

/// Internal: test whether a body supports a given standard.
#[allow(dead_code, reason = "only reachable via bodies_for_standard, see its allow")]
fn body_supports(body: &CertificationBody, target: &ComplianceStandard) -> bool {
    body.standards.iter().any(|s| standards_match(s, target))
}

/// Flexible matching for standard variants: exact match or level-superset.
#[allow(dead_code, reason = "only reachable via bodies_for_standard, see its allow")]
fn standards_match(supported: &ComplianceStandard, requested: &ComplianceStandard) -> bool {
    match (supported, requested) {
        (
            ComplianceStandard::Iec61508 { sil_level: a },
            ComplianceStandard::Iec61508 { sil_level: b },
        ) => a >= b,
        (
            ComplianceStandard::Iso26262 { asil_level: a },
            ComplianceStandard::Iso26262 { asil_level: b },
        ) => asil_gte(*a, *b),
        (
            ComplianceStandard::Do178c { dal_level: a },
            ComplianceStandard::Do178c { dal_level: b },
        ) => dal_gte(*a, *b),
        (ComplianceStandard::Fda21CfrPart11, ComplianceStandard::Fda21CfrPart11) => true,
        (ComplianceStandard::Custom(a), ComplianceStandard::Custom(b)) => a == b,
        _ => false,
    }
}

/// ASIL ordering: A < B < C < D (QM not included in cert body matching).
#[allow(dead_code, reason = "only reachable via bodies_for_standard, see its allow")]
fn asil_gte(a: char, b: char) -> bool {
    fn rank(c: char) -> u8 {
        match c {
            'A' => 1,
            'B' => 2,
            'C' => 3,
            'D' => 4,
            _ => 0,
        }
    }
    rank(a) >= rank(b)
}

/// DAL ordering: E < D < C < B < A (A is most stringent for DO-178C).
#[allow(dead_code, reason = "only reachable via standards_match, see its allow")]
fn dal_gte(a: char, b: char) -> bool {
    fn rank(c: char) -> u8 {
        match c {
            'E' => 1,
            'D' => 2,
            'C' => 3,
            'B' => 4,
            'A' => 5,
            _ => 0,
        }
    }
    rank(a) >= rank(b)
}

/// Format a recommendation for obtaining a receipt from a certification body
/// that covers the requested standard.
///
/// Exercised by `tests/certification_policies.rs`; not yet called from a live
/// CLI path.
#[allow(dead_code, reason = "exercised by tests/certification_policies.rs, not yet wired into a live CLI path")]
pub fn cert_body_recommendation(standard: &ComplianceStandard) -> String {
    let bodies = bodies_for_standard(standard);
    if bodies.is_empty() {
        return format!(
            "No registered certification body supports {}. \
             Refer to docs/CERT-BODY-INTEGRATION.md to register a new provider.",
            standard.display_name()
        );
    }

    let mut out = format!(
        "To obtain a {} certification receipt, contact one of the following bodies:\n",
        standard.display_name()
    );
    for body in &bodies {
        out.push_str(&format!(
            "  - {} ({}): {}\n",
            body.name, body.id, body.submission_url
        ));
    }
    out.push_str(
        "\nSubject your cargo-cicd XES evidence to the body's oracle API and include the \
         resulting receipt JSON in your receipts/ directory.",
    );
    out
}
