// src/certification/iso_26262.rs
//
// ISO 26262 automotive safety compliance requirement definitions and evidence mappings.

/// ISO 26262 Automotive Safety Integrity Level.
#[derive(Debug, Clone, PartialEq)]
pub enum Asil {
    /// QM — Quality Management only, no specific safety measures required.
    Qm,
    /// ASIL A — lowest functional safety level.
    A,
    /// ASIL B.
    B,
    /// ASIL C.
    C,
    /// ASIL D — highest functional safety level.
    D,
}

impl Asil {
    /// Short name string, e.g. "ASIL D".
    pub fn name(&self) -> &'static str {
        match self {
            Asil::Qm => "QM",
            Asil::A => "ASIL A",
            Asil::B => "ASIL B",
            Asil::C => "ASIL C",
            Asil::D => "ASIL D",
        }
    }

    /// Numeric severity (QM=0, A=1, B=2, C=3, D=4).
    pub fn severity(&self) -> u8 {
        match self {
            Asil::Qm => 0,
            Asil::A => 1,
            Asil::B => 2,
            Asil::C => 3,
            Asil::D => 4,
        }
    }

    /// Brief description of what this ASIL level means.
    pub fn description(&self) -> &'static str {
        match self {
            Asil::Qm => "Quality Management — standard quality processes, no ISO 26262 safety measures required",
            Asil::A => "Lowest automotive safety integrity level — basic safety measures for low-risk systems",
            Asil::B => "Moderate safety integrity — enhanced safety measures for systems with potential injury risk",
            Asil::C => "High safety integrity — rigorous measures for systems with probable serious injury risk",
            Asil::D => "Highest automotive safety integrity level — comprehensive measures for life-critical systems",
        }
    }

    /// Return true if this ASIL is at least as strict as `other`.
    pub fn at_least(&self, other: &Asil) -> bool {
        self.severity() >= other.severity()
    }
}

/// ISO 26262 work product / requirement.
#[derive(Debug)]
pub struct Iso26262Requirement {
    /// ISO 26262 part number (e.g. 6 = Software, 8 = Safety management).
    pub part: u8,
    /// Clause reference, e.g. "6.7.2".
    pub clause: String,
    /// Short title.
    pub title: String,
    /// How cargo-cicd process evidence satisfies this requirement.
    pub evidence_mapping: String,
    /// Minimum ASIL level this requirement applies to.
    pub min_asil: Asil,
    /// The cargo-cicd command(s) that produce relevant evidence.
    pub covered_by_commands: Vec<String>,
}

/// Return all ISO 26262 requirements that cargo-cicd evidence can partially satisfy.
pub fn requirements() -> Vec<Iso26262Requirement> {
    vec![
        Iso26262Requirement {
            part: 6,
            clause: "6.4.2".to_string(),
            title: "Software safety requirements".to_string(),
            evidence_mapping: "cargo-cicd `status show` and `workspace doctor` emit workspace \
                               state evidence capturing dependency topology, toolchain version, \
                               and requirement traceability via cicd.toml."
                .to_string(),
            min_asil: Asil::A,
            covered_by_commands: vec!["status show".to_string(), "workspace doctor".to_string()],
        },
        Iso26262Requirement {
            part: 6,
            clause: "6.6".to_string(),
            title: "Software unit design and implementation".to_string(),
            evidence_mapping: "cargo-cicd `workspace doctor` records all workspace members, \
                               crate boundaries, and edition metadata as structured XES evidence, \
                               documenting the implemented unit structure."
                .to_string(),
            min_asil: Asil::A,
            covered_by_commands: vec!["workspace doctor".to_string(), "status show".to_string()],
        },
        Iso26262Requirement {
            part: 6,
            clause: "6.7".to_string(),
            title: "Software unit testing".to_string(),
            evidence_mapping: "cargo-cicd `test changed` and `trybuild changed` emit unit-level \
                               test execution events with pass/fail verdicts. Each changed file \
                               is individually tracked in process evidence."
                .to_string(),
            min_asil: Asil::A,
            covered_by_commands: vec![
                "test changed".to_string(),
                "trybuild changed".to_string(),
                "trybuild full".to_string(),
            ],
        },
        Iso26262Requirement {
            part: 6,
            clause: "6.8".to_string(),
            title: "Software integration and testing".to_string(),
            evidence_mapping: "cargo-cicd `pipeline run` sequences all CI/CD activities and \
                               emits integration-level evidence with aggregate pass/fail verdict \
                               adjudicated by wasm4pm."
                .to_string(),
            min_asil: Asil::A,
            covered_by_commands: vec!["pipeline run".to_string(), "evidence audit".to_string()],
        },
        Iso26262Requirement {
            part: 6,
            clause: "6.9".to_string(),
            title: "Verification of software safety requirements".to_string(),
            evidence_mapping: "cargo-cicd evidence gate (wasm4pm) provides independent \
                               adjudication of all process evidence, producing Accept/Refuse \
                               verdicts that serve as verification records."
                .to_string(),
            min_asil: Asil::B,
            covered_by_commands: vec!["evidence audit".to_string(), "evidence doctor".to_string()],
        },
        Iso26262Requirement {
            part: 8,
            clause: "8.3".to_string(),
            title: "Software configuration management".to_string(),
            evidence_mapping: "cargo-cicd `git status`, `git phase`, and `git close` emit \
                               configuration management evidence: branch name, ahead/behind counts, \
                               dirty-file lists, and phase closure events."
                .to_string(),
            min_asil: Asil::A,
            covered_by_commands: vec![
                "git status".to_string(),
                "git phase".to_string(),
                "git close".to_string(),
            ],
        },
    ]
}

/// Check whether a given set of process evidence commands satisfies an ISO 26262 requirement.
///
/// Returns `None` if satisfied, or `Some(reason)` describing the gap.
pub fn check_requirement(
    req: &Iso26262Requirement,
    event_commands: &[String],
) -> Option<String> {
    let covered = req
        .covered_by_commands
        .iter()
        .any(|cmd| event_commands.iter().any(|ev| ev == cmd));

    if covered {
        None
    } else {
        Some(format!(
            "ISO 26262 {} ({}) not satisfied: no evidence found for {:?}",
            req.clause, req.title, req.covered_by_commands
        ))
    }
}

/// Generate a human-readable compliance summary for a given ASIL level.
pub fn compliance_summary(asil: &Asil, satisfied: &[String], missing: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "ISO 26262 Compliance Summary — {}\n",
        asil.name()
    ));
    out.push_str(&format!("Description: {}\n\n", asil.description()));

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
    let pct = if total == 0 {
        0
    } else {
        (satisfied.len() * 100) / total
    };
    out.push_str(&format!("\nCoverage: {}% ({}/{})\n", pct, satisfied.len(), total));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asil_d_severity_is_4() {
        assert_eq!(Asil::D.severity(), 4);
    }

    #[test]
    fn asil_qm_severity_is_0() {
        assert_eq!(Asil::Qm.severity(), 0);
    }

    #[test]
    fn asil_names() {
        assert_eq!(Asil::A.name(), "ASIL A");
        assert_eq!(Asil::B.name(), "ASIL B");
        assert_eq!(Asil::C.name(), "ASIL C");
        assert_eq!(Asil::D.name(), "ASIL D");
        assert_eq!(Asil::Qm.name(), "QM");
    }

    #[test]
    fn requirements_returns_at_least_six() {
        assert!(requirements().len() >= 6);
    }

    #[test]
    fn check_requirement_satisfied_when_command_present() {
        let reqs = requirements();
        let req = reqs.iter().find(|r| r.clause == "6.7").unwrap();
        let gap = check_requirement(req, &["test changed".to_string()]);
        assert!(gap.is_none(), "should be satisfied, got: {:?}", gap);
    }

    #[test]
    fn check_requirement_missing_when_no_commands() {
        let reqs = requirements();
        let req = reqs.iter().find(|r| r.clause == "6.7").unwrap();
        let gap = check_requirement(req, &[]);
        assert!(gap.is_some());
    }

    #[test]
    fn compliance_summary_contains_header() {
        let summary = compliance_summary(&Asil::B, &[], &[]);
        assert!(summary.contains("ISO 26262"));
        assert!(summary.contains("ASIL B"));
    }
}
