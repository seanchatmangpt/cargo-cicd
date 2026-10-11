//! src/nouns/certification.rs — cargo cicd certification verbs (inlined from the retired legacy layer; see CCICD-106)

// src/nouns/certification.rs — cargo cicd certification show
//
// Prints an IEC 61508 / ISO 26262 compliance summary using the
// certification body registry.

use crate::certification::iec_61508::{self, Sil};
use crate::certification::iso_26262::{self, Asil};
use crate::certification::known_cert_bodies;
use crate::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::theme::{self, Role};
use crate::ui::{panel, symbols};

pub struct CertificationShowVerb;

/// Classify IEC 61508 requirements against observed evidence commands.
/// Fail-closed: when no verifiable commands exist, every requirement is
/// UNVERIFIED — never satisfied.
fn classify_iec(
    sil: &Sil,
    all_cmds: &[String],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let reqs = iec_61508::requirements();
    let mut satisfied = vec![];
    let mut missing = vec![];
    let mut unverified = vec![];
    for req in &reqs {
        if req.min_sil.level() <= sil.level() {
            if all_cmds.is_empty() {
                unverified.push(format!("{} \u{2014} {}", req.number, req.text));
            } else {
                match iec_61508::check_requirement(req, all_cmds) {
                    None => satisfied.push(format!("{} \u{2014} {}", req.number, req.text)),
                    Some(_) => missing.push(format!("{} \u{2014} {}", req.number, req.text)),
                }
            }
        }
    }
    (satisfied, missing, unverified)
}

impl CertificationShowVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("certification:show");

        println!("{}", panel::header("cargo-cicd certification summary"));

        // ── IEC 61508 summary at SIL 1 (baseline) ───────────────────────────
        let sil = Sil::new(1);
        // With no prior event log present, no requirement is checkable.
        // Honest absence: uncheckable requirements render UNVERIFIED, not
        // satisfied — fail-closed.
        let all_cmds: Vec<String> = vec![];
        let (satisfied, missing, unverified) = classify_iec(&sil, &all_cmds);
        if !unverified.is_empty() {
            println!(
                "{} UNVERIFIED ({}): no verifiable commands available",
                symbols::arrow(),
                unverified.len()
            );
            for u in &unverified {
                println!("  [??] {u}");
            }
        }
        let iec_summary = iec_61508::compliance_summary(&sil, &satisfied, &missing);
        println!("{}", theme::paint(&iec_summary, Role::Value));

        // ── ISO 26262 summary at ASIL A (baseline) ──────────────────────────
        let asil = Asil::A;
        let iso_reqs = iso_26262::requirements();
        let mut iso_satisfied: Vec<String> = vec![];
        let mut iso_missing: Vec<String> = vec![];
        let mut iso_unverified: Vec<String> = vec![];
        for req in &iso_reqs {
            if req.min_asil.severity() <= asil.severity() {
                if all_cmds.is_empty() {
                    iso_unverified.push(format!("{} \u{2014} {}", req.clause, req.title));
                } else {
                    match iso_26262::check_requirement(req, &all_cmds) {
                        None => {
                            iso_satisfied.push(format!("{} \u{2014} {}", req.clause, req.title))
                        }
                        Some(_) => {
                            iso_missing.push(format!("{} \u{2014} {}", req.clause, req.title))
                        }
                    }
                }
            }
        }
        if !iso_unverified.is_empty() {
            println!(
                "{} UNVERIFIED ({}): no verifiable commands available",
                symbols::arrow(),
                iso_unverified.len()
            );
            for u in &iso_unverified {
                println!("  [??] {u}");
            }
        }
        let iso_summary = iso_26262::compliance_summary(&asil, &iso_satisfied, &iso_missing);
        println!("{}", theme::paint(&iso_summary, Role::Value));

        // ── SOC2 Trust Service Criteria ──────────────────────────────────────
        let soc2_summary = crate::certification::soc2::compliance_summary();
        println!("\n{}", theme::paint(&soc2_summary, Role::Value));

        // ── TOGAF ADM phase coverage ─────────────────────────────────────────
        let togaf_summary = crate::certification::togaf::coverage_summary();
        println!("\n{}", theme::paint(&togaf_summary, Role::Value));

        // ── Registered certification bodies ─────────────────────────────────
        let bodies = known_cert_bodies();
        if !bodies.is_empty() {
            println!("{}", panel::header("registered certification bodies"));
            let rows: Vec<(String, String)> = bodies
                .iter()
                .map(|b| {
                    let standards: Vec<String> =
                        b.standards.iter().map(|s| s.display_name()).collect();
                    let value = format!("{} {}", symbols::arrow(), standards.join(", "));
                    (b.name.clone(), value)
                })
                .collect();
            let row_refs: Vec<(&str, &str)> = rows
                .iter()
                .map(|(k, v): &(String, String)| (k.as_str(), v.as_str()))
                .collect();
            println!("{}", panel::kv(&row_refs));
        }

        // Fail-closed: any UNVERIFIED requirement means the summary does
        // not pass. Honest absence is recorded as UNVERIFIED, never PASS.
        let unverified_count = unverified.len() + iso_unverified.len();
        let verdict = if unverified_count > 0 {
            println!(
                "\nverdict: NON-PASSING — {} requirement(s) UNVERIFIED (no verifiable evidence commands)",
                unverified_count
            );
            "UNVERIFIED"
        } else {
            "PASS"
        };

        finish_evidence(
            start_evt,
            t0,
            case_id,
            verdict,
            "certification:show",
            &evidence_dir,
        );

        if unverified_count > 0 {
            anyhow::bail!(
                "certification summary non-passing: {} requirement(s) UNVERIFIED",
                unverified_count
            );
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Fail-closed: with zero verifiable commands, NO requirement may be
    /// reported satisfied — honest absence renders UNVERIFIED.
    #[test]
    fn empty_command_set_leaves_every_requirement_unverified() {
        let sil = Sil::new(1);
        let (satisfied, _missing, unverified) = classify_iec(&sil, &[]);
        assert!(satisfied.is_empty(), "no requirement is satisfiable without evidence");
        assert!(!unverified.is_empty(), "requirements must render UNVERIFIED, not satisfied");
    }

    /// Requirements actually covered by recorded commands classify as
    /// satisfied (real adjudication path, not inverted).
    #[test]
    fn covered_commands_satisfy_requirements() {
        let sil = Sil::new(1);
        let reqs = iec_61508::requirements();
        let cmds: Vec<String> = reqs
            .iter()
            .filter(|r| r.min_sil.level() <= 1)
            .flat_map(|r| r.covered_by_commands.clone())
            .collect();
        let (satisfied, _missing, unverified) = classify_iec(&sil, &cmds);
        assert!(!satisfied.is_empty());
        assert!(unverified.is_empty());
    }
}

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("show")]
/// Print IEC 61508 / ISO 26262 compliance summary.
pub fn cmd_show() -> Result<()> {
    CertificationShowVerb
        .execute()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
