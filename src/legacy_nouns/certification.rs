// src/nouns/certification.rs — cargo cicd certification show
//
// Prints an IEC 61508 / ISO 26262 compliance summary using the
// certification body registry.

use crate::certification::iec_61508::{self, Sil};
use crate::certification::iso_26262::{self, Asil};
use crate::certification::known_cert_bodies;
use crate::legacy_nouns::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::theme::{self, Role};
use crate::ui::{panel, symbols};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct CertificationNoun;

impl CertificationNoun {
    pub fn new() -> Self {
        Self
    }

    pub fn run_direct() -> anyhow::Result<()> {
        CertificationShowVerb.execute()
    }
}

impl Default for CertificationNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for CertificationNoun {
    fn name(&self) -> &'static str {
        "certification"
    }
    fn about(&self) -> &'static str {
        "IEC 61508 / ISO 26262 compliance summary"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(CertificationShowVerb)]
    }
}

pub struct CertificationShowVerb;

impl CertificationShowVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("certification:show");

        println!("{}", panel::header("cargo-cicd certification summary"));

        // ── IEC 61508 summary at SIL 1 (baseline) ───────────────────────────
        let sil = Sil::new(1);
        let reqs = iec_61508::requirements();
        // With no prior event log present, every SIL-1 requirement is pending.
        let all_cmds: Vec<String> = vec![];
        let mut satisfied: Vec<String> = vec![];
        let mut missing: Vec<String> = vec![];
        for req in &reqs {
            if req.min_sil.level() <= sil.level() {
                match iec_61508::check_requirement(req, &all_cmds) {
                    None => satisfied.push(format!("{} \u{2014} {}", req.number, req.text)),
                    Some(_) => missing.push(format!("{} \u{2014} {}", req.number, req.text)),
                }
            }
        }
        let iec_summary = iec_61508::compliance_summary(&sil, &satisfied, &missing);
        println!("{}", theme::paint(&iec_summary, Role::Value));

        // ── ISO 26262 summary at ASIL A (baseline) ──────────────────────────
        let asil = Asil::A;
        let iso_reqs = iso_26262::requirements();
        let mut iso_satisfied: Vec<String> = vec![];
        let mut iso_missing: Vec<String> = vec![];
        for req in &iso_reqs {
            if req.min_asil.severity() <= asil.severity() {
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

        finish_evidence(
            start_evt,
            t0,
            case_id,
            "PASS",
            "certification:show",
            &evidence_dir,
        );

        Ok(())
    }
}

impl VerbCommand for CertificationShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Print IEC 61508 and ISO 26262 compliance summary"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}
