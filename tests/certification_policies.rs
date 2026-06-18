// tests/certification_policies.rs
//
// Certification infrastructure policy tests.
// Vision 2030 — Phase 1, Weeks 3-7: Safety & Certification.

use cargo_cicd::certification::iec_61508::{self, Sil};
use cargo_cicd::certification::iso_26262::{self, Asil};
use cargo_cicd::certification::registry::{
    format_registry_listing, is_certified, SafetyCriticalEntry,
};
use cargo_cicd::certification::{
    bodies_for_standard, cert_body_recommendation, known_cert_bodies, ComplianceStandard,
};

// ── 1. known_cert_bodies() ────────────────────────────────────────────────────

#[test]
fn known_cert_bodies_returns_at_least_three() {
    let bodies = known_cert_bodies();
    assert!(
        bodies.len() >= 3,
        "expected at least 3 certification bodies, got {}",
        bodies.len()
    );
}

// ── 2. bodies_for_standard(IEC 61508 SIL 2) ──────────────────────────────────

#[test]
fn bodies_for_iec_61508_sil2_non_empty() {
    let standard = ComplianceStandard::Iec61508 { sil_level: 2 };
    let bodies = bodies_for_standard(&standard);
    assert!(
        !bodies.is_empty(),
        "at least one body must support IEC 61508 SIL 2"
    );
}

// ── 3. Sil::new(2).name() == "SIL 2" ─────────────────────────────────────────

#[test]
fn sil_new_2_name_is_sil_2() {
    let sil = Sil::new(2);
    assert_eq!(
        sil.name(),
        "SIL 2",
        "Sil::new(2).name() must return \"SIL 2\""
    );
}

// ── 4. iec_61508::requirements() returns at least 6 items ────────────────────

#[test]
fn iec_61508_requirements_at_least_six() {
    let reqs = iec_61508::requirements();
    assert!(
        reqs.len() >= 6,
        "expected at least 6 IEC 61508 requirements, got {}",
        reqs.len()
    );
}

// ── 5. check_requirement returns None when relevant noun is present ───────────

#[test]
fn iec_61508_check_requirement_none_when_command_present() {
    let reqs = iec_61508::requirements();
    // 7.4.5 covers "test changed"
    let req = reqs
        .iter()
        .find(|r| r.number == "7.4.5")
        .expect("requirement 7.4.5 must exist");

    let commands = vec!["test changed".to_string()];
    let gap = iec_61508::check_requirement(req, &commands);
    assert!(
        gap.is_none(),
        "requirement 7.4.5 should be satisfied by 'test changed', got: {:?}",
        gap
    );
}

// ── 6. Asil::D.severity() == 4 ───────────────────────────────────────────────

#[test]
fn asil_d_severity_is_4() {
    assert_eq!(Asil::D.severity(), 4, "ASIL D must have severity 4");
}

// ── 7. iso_26262::requirements() returns at least 5 items ────────────────────

#[test]
fn iso_26262_requirements_at_least_five() {
    let reqs = iso_26262::requirements();
    assert!(
        reqs.len() >= 5,
        "expected at least 5 ISO 26262 requirements, got {}",
        reqs.len()
    );
}

// ── 8. is_certified([], "serde", "1.0.0") returns false ──────────────────────

#[test]
fn is_certified_empty_registry_returns_false() {
    let result = is_certified(&[], "serde", "1.0.0");
    assert!(!result, "empty registry must return false for any crate");
}

// ── 9. SafetyCriticalEntry can be constructed ─────────────────────────────────

#[test]
fn safety_critical_entry_can_be_constructed() {
    let entry = SafetyCriticalEntry::new(
        "example-safety-crate",
        "0.1.0",
        "ferrous-systems",
        vec!["IEC 61508 SIL 2".to_string()],
        "2026-06-17",
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert_eq!(entry.crate_name, "example-safety-crate");
    assert_eq!(entry.version, "0.1.0");
    assert_eq!(entry.cert_body_id, "ferrous-systems");
    assert_eq!(entry.certified_at, "2026-06-17");
    assert!(entry.evidence_url.is_none());
}

// ── 10. format_registry_listing([]) returns a header line ─────────────────────

#[test]
fn format_registry_listing_empty_has_header_line() {
    let listing = format_registry_listing(&[]);
    assert!(
        listing.contains("Safety-Critical Crate Registry"),
        "format_registry_listing([]) must contain a header line, got: {}",
        listing
    );
}

// ── Additional: ComplianceStandard display_name ───────────────────────────────

#[test]
fn compliance_standard_display_name_iec_61508() {
    let s = ComplianceStandard::Iec61508 { sil_level: 3 };
    assert_eq!(s.display_name(), "IEC 61508 SIL 3");
}

#[test]
fn compliance_standard_display_name_iso_26262() {
    let s = ComplianceStandard::Iso26262 { asil_level: 'D' };
    assert_eq!(s.display_name(), "ISO 26262 ASIL D");
}

#[test]
fn compliance_standard_short_code() {
    assert_eq!(
        ComplianceStandard::Fda21CfrPart11.short_code(),
        "FDA-21CFR11"
    );
}

// ── Additional: IEC 61508 compliance_summary ──────────────────────────────────

#[test]
fn iec_61508_compliance_summary_contains_sil_name() {
    let sil = Sil::new(2);
    let summary = iec_61508::compliance_summary(
        &sil,
        &["7.4.5 — module testing".to_string()],
        &["7.4.7 — verification".to_string()],
    );
    assert!(summary.contains("SIL 2"), "summary must mention SIL 2");
    assert!(
        summary.contains("Satisfied"),
        "summary must list satisfied items"
    );
    assert!(
        summary.contains("Missing"),
        "summary must list missing items"
    );
}

// ── Additional: ISO 26262 compliance_summary ──────────────────────────────────

#[test]
fn iso_26262_compliance_summary_contains_asil_name() {
    let summary = iso_26262::compliance_summary(&Asil::B, &[], &[]);
    assert!(summary.contains("ASIL B"), "summary must mention ASIL B");
}

// ── Additional: cert_body_recommendation ─────────────────────────────────────

#[test]
fn cert_body_recommendation_non_empty_for_known_standard() {
    let rec = cert_body_recommendation(&ComplianceStandard::Iec61508 { sil_level: 2 });
    assert!(!rec.is_empty(), "recommendation must not be empty");
}

#[test]
fn cert_body_recommendation_fallback_for_unknown_standard() {
    let rec = cert_body_recommendation(&ComplianceStandard::Custom(
        "Hypothetical-Ultra-Standard-9999".to_string(),
    ));
    // Should contain guidance even when no body matches
    assert!(!rec.is_empty());
}

// ── Additional: registry with an entry ────────────────────────────────────────

#[test]
fn is_certified_found_in_registry() {
    let entry = SafetyCriticalEntry::new(
        "safe-lib",
        "1.0.0",
        "ferrous-systems",
        vec!["IEC 61508 SIL 2".to_string()],
        "2026-06-17",
        "sha256:abcdef",
    );
    assert!(is_certified(&[entry], "safe-lib", "1.0.0"));
}

#[test]
fn format_registry_listing_shows_entry_fields() {
    let entry = SafetyCriticalEntry::new(
        "my-critical-crate",
        "2.3.4",
        "trustinsoft",
        vec!["IEC 61508 SIL 1".to_string()],
        "2026-01-15",
        "sha256:deadbeef",
    )
    .with_evidence_url("https://evidence.example.com/my-critical-crate/");

    let listing = format_registry_listing(&[entry]);
    assert!(listing.contains("my-critical-crate"));
    assert!(listing.contains("trustinsoft"));
    assert!(listing.contains("2026-01-15"));
}
