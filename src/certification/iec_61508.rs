// src/certification/iec_61508.rs
//
// IEC 61508 compliance requirement definitions and evidence mappings.

/// IEC 61508 Safety Integrity Level (1–4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sil(pub u8);

impl Sil {
    /// Construct a SIL level, clamping to the valid range 1–4.
    pub fn new(level: u8) -> Self {
        Sil(level.clamp(1, 4))
    }

    /// Raw numeric level (1–4).
    pub fn level(&self) -> u8 {
        self.0
    }

    /// Display name, e.g. "SIL 2".
    pub fn name(&self) -> &'static str {
        match self.0 {
            1 => "SIL 1",
            2 => "SIL 2",
            3 => "SIL 3",
            4 => "SIL 4",
            _ => "SIL ?",
        }
    }

    /// Brief description of the safety integrity level.
    pub fn description(&self) -> &'static str {
        match self.0 {
            1 => "Low demand, low consequence — tolerable risk ~10^-5 to 10^-6 per hour",
            2 => "Moderate demand — tolerable risk ~10^-6 to 10^-7 per hour",
            3 => "High demand — tolerable risk ~10^-7 to 10^-8 per hour",
            4 => "Continuous / high-consequence — tolerable risk ~10^-8 to 10^-9 per hour",
            _ => "Unknown SIL level",
        }
    }
}

/// A single IEC 61508 requirement and its cargo-cicd evidence mapping.
#[derive(Debug)]
pub struct Iec61508Requirement {
    /// Requirement number, e.g. "7.4.2.3".
    pub number: String,
    /// Brief requirement text.
    pub text: String,
    /// How cargo-cicd process evidence satisfies this requirement.
    pub evidence_mapping: String,
    /// Minimum SIL level this requirement applies to.
    pub min_sil: Sil,
    /// The cargo-cicd command(s) that produce relevant evidence.
    pub covered_by_commands: Vec<String>,
}

/// Return all IEC 61508 requirements that cargo-cicd evidence can partially satisfy.
pub fn requirements() -> Vec<Iec61508Requirement> {
    vec![
        Iec61508Requirement {
            number: "7.4.2".to_string(),
            text: "Software requirements specification — functional and safety requirements \
                   shall be documented and verified."
                .to_string(),
            evidence_mapping: "cargo-cicd emits `status show` process events that record \
                               workspace state and requirement traceability via cicd.toml."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec!["status show".to_string(), "workspace doctor".to_string()],
        },
        Iec61508Requirement {
            number: "7.4.3".to_string(),
            text: "Software architecture design — the software architecture shall be specified \
                   and verified against safety requirements."
                .to_string(),
            evidence_mapping: "cargo-cicd `workspace doctor` records package member topology and \
                               dependency relationships as XES evidence."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec!["workspace doctor".to_string()],
        },
        Iec61508Requirement {
            number: "7.4.5".to_string(),
            text: "Software module testing — each software module shall be tested against its \
                   specification."
                .to_string(),
            evidence_mapping: "cargo-cicd `test changed` and `trybuild changed` emit per-module \
                               test execution evidence with pass/fail verdicts in XES format."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec![
                "test changed".to_string(),
                "trybuild changed".to_string(),
                "trybuild full".to_string(),
            ],
        },
        Iec61508Requirement {
            number: "7.4.6".to_string(),
            text: "Software integration testing — integrated software modules shall be tested \
                   together to verify interfaces and interactions."
                .to_string(),
            evidence_mapping: "cargo-cicd `pipeline run` sequences all CI/CD activities and emits \
                               integration-level process events with aggregate verdict."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec!["pipeline run".to_string()],
        },
        Iec61508Requirement {
            number: "7.4.7".to_string(),
            text: "Software verification — the software shall be verified to demonstrate \
                   conformance with its specification."
                .to_string(),
            evidence_mapping: "cargo-cicd evidence gate (wasm4pm) adjudicates all process events, \
                               issuing Accept/Refuse verdicts that constitute independent verification \
                               records."
                .to_string(),
            min_sil: Sil(2),
            covered_by_commands: vec!["evidence audit".to_string(), "evidence doctor".to_string()],
        },
        Iec61508Requirement {
            number: "7.4.9".to_string(),
            text: "Software validation — the integrated system shall be validated to confirm \
                   safety requirements are satisfied."
                .to_string(),
            evidence_mapping: "cargo-cicd `publish run` gate verifies publishability criteria \
                               and emits a publish-run XES event adjudicated by wasm4pm before \
                               any release is permitted."
                .to_string(),
            min_sil: Sil(2),
            covered_by_commands: vec!["publish run".to_string(), "evidence audit".to_string()],
        },
        Iec61508Requirement {
            number: "8.4.6".to_string(),
            text: "Software modification — modifications shall be assessed for safety impact \
                   and re-tested as appropriate."
                .to_string(),
            evidence_mapping: "cargo-cicd `test changed` restricts testing to changed files, \
                               providing a targeted regression evidence trail per modification."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec!["test changed".to_string(), "git status".to_string()],
        },
        Iec61508Requirement {
            number: "5.2.4".to_string(),
            text: "Safety lifecycle documentation — all phases of the safety lifecycle shall \
                   produce and maintain documented evidence of activities performed."
                .to_string(),
            evidence_mapping: "cargo-cicd persists all process events as XES and JSONL files in \
                               target/cargo-cicd/evidence/, providing a time-stamped lifecycle \
                               document trail suitable for submission to a certification body."
                .to_string(),
            min_sil: Sil(1),
            covered_by_commands: vec![
                "status show".to_string(),
                "evidence doctor".to_string(),
                "evidence audit".to_string(),
            ],
        },
    ]
}

/// Check whether a given set of process evidence commands satisfies an IEC 61508 requirement.
///
/// Returns `None` if satisfied, or `Some(reason)` describing the gap.
pub fn check_requirement(req: &Iec61508Requirement, event_commands: &[String]) -> Option<String> {
    let covered = req
        .covered_by_commands
        .iter()
        .any(|cmd| event_commands.iter().any(|ev| ev == cmd));

    if covered {
        None
    } else {
        Some(format!(
            "Requirement {} not satisfied: no evidence found for {:?}",
            req.number, req.covered_by_commands
        ))
    }
}

/// Generate a human-readable compliance summary for a given SIL level.
pub fn compliance_summary(sil: &Sil, satisfied: &[String], missing: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!("IEC 61508 Compliance Summary — {}\n", sil.name()));
    out.push_str(&format!("Description: {}\n\n", sil.description()));

    if satisfied.is_empty() && missing.is_empty() {
        out.push_str("No requirements evaluated.\n");
        return out;
    }

    out.push_str(&format!("Satisfied ({}):\n", satisfied.len()));
    for s in satisfied {
        out.push_str(&format!("  [OK] {}\n", s));
    }

    out.push_str(&format!("\nMissing ({}):\n", missing.len()));
    for m in missing {
        out.push_str(&format!("  [!!] {}\n", m));
    }

    let total = satisfied.len() + missing.len();
    let pct = (satisfied.len() * 100).checked_div(total).unwrap_or(0);
    out.push_str(&format!(
        "\nCoverage: {}% ({}/{})\n",
        pct,
        satisfied.len(),
        total
    ));

    out
}

