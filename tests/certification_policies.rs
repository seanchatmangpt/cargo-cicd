// tests/certification_policies.rs
//
// Certification infrastructure policy tests.
// Vision 2030 — Phase 1, Weeks 3-7: Safety & Certification.

use cargo_cicd::certification::iec_61508::{self, Sil};
use cargo_cicd::certification::iso_26262::Asil;
use cargo_cicd::certification::registry::{is_certified, SafetyCriticalEntry};
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

// ── 8. is_certified([], "serde", "1.0.0") returns false ──────────────────────

#[test]
fn is_certified_empty_registry_returns_false() {
    let result = is_certified(&[], "serde", "1.0.0");
    assert!(!result, "empty registry must return false for any crate");
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

// ── Additional: cert_body_recommendation ─────────────────────────────────────

#[test]
fn cert_body_recommendation_non_empty_for_known_standard() {
    let rec = cert_body_recommendation(&ComplianceStandard::Iec61508 { sil_level: 2 });
    assert!(!rec.is_empty(), "recommendation must not be empty");
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
