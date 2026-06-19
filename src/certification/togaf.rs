// src/certification/togaf.rs
//
// TOGAF ADM (Architecture Development Method) phase coverage placeholders.

/// A TOGAF ADM phase and its cargo-cicd evidence coverage.
#[derive(Debug)]
pub struct TogafPhase {
    /// Single-letter (or short) phase identifier, e.g. "A", "B", "C-App".
    pub id: String,
    /// Full phase name, e.g. "Architecture Vision".
    pub name: String,
    /// How cargo-cicd covers this phase, or `None` if deferred / not covered.
    pub covered_by: Option<String>,
}

/// Return TOGAF ADM phases with cargo-cicd coverage annotations.
pub fn adm_phases() -> Vec<TogafPhase> {
    vec![
        TogafPhase {
            id: "A".to_string(),
            name: "Architecture Vision".to_string(),
            covered_by: None,
        },
        TogafPhase {
            id: "B".to_string(),
            name: "Business Architecture".to_string(),
            covered_by: Some(
                "ggen ontology (RDF capability map) provides machine-readable Business \
                 Architecture for cargo-cicd capabilities"
                    .to_string(),
            ),
        },
        TogafPhase {
            id: "C-App".to_string(),
            name: "Application Architecture".to_string(),
            covered_by: Some(
                "clap-noun-verb grammar is the Application Architecture layer".to_string(),
            ),
        },
        TogafPhase {
            id: "C-Data".to_string(),
            name: "Data Architecture".to_string(),
            covered_by: Some(
                "OCEL 2.0 event log + BLAKE3 receipts define the Data Architecture".to_string(),
            ),
        },
        TogafPhase {
            id: "D".to_string(),
            name: "Technology Architecture".to_string(),
            covered_by: Some(
                "Rust workspace + wasm4pm oracle define the Technology Architecture".to_string(),
            ),
        },
        TogafPhase {
            id: "E".to_string(),
            name: "Opportunities & Solutions".to_string(),
            covered_by: None,
        },
        TogafPhase {
            id: "F".to_string(),
            name: "Migration Planning".to_string(),
            covered_by: None,
        },
        TogafPhase {
            id: "G".to_string(),
            name: "Implementation Governance".to_string(),
            covered_by: Some(
                "evidence gate (wasm4pm) enforces Implementation Governance via \
                 ACCEPT/REFUSE verdicts"
                    .to_string(),
            ),
        },
        TogafPhase {
            id: "H".to_string(),
            name: "Architecture Change Management".to_string(),
            covered_by: Some(
                "git phase tracking (cargo cicd git close) governs Architecture Change Management"
                    .to_string(),
            ),
        },
    ]
}

/// Return a formatted summary of TOGAF ADM coverage.
pub fn coverage_summary() -> String {
    let phases = adm_phases();
    let covered: Vec<_> = phases.iter().filter(|p| p.covered_by.is_some()).collect();
    let mut out = format!(
        "TOGAF ADM Phase Coverage ({}/{})\n\n",
        covered.len(),
        phases.len()
    );
    for phase in &phases {
        let marker = if phase.covered_by.is_some() {
            "[OK]"
        } else {
            "[  ]"
        };
        out.push_str(&format!(
            "  {} Phase {} \u{2014} {}\n",
            marker, phase.id, phase.name
        ));
        if let Some(desc) = &phase.covered_by {
            out.push_str(&format!("       {}\n", desc));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adm_phases_returns_nine() {
        assert_eq!(adm_phases().len(), 9);
    }

    #[test]
    fn phase_b_is_covered() {
        let phases = adm_phases();
        let b = phases.iter().find(|p| p.id == "B").expect("Phase B must exist");
        assert!(
            b.covered_by.is_some(),
            "Phase B (Business Architecture) should be covered by ggen ontology"
        );
    }

    #[test]
    fn phase_g_is_covered() {
        let phases = adm_phases();
        let g = phases.iter().find(|p| p.id == "G").expect("Phase G must exist");
        assert!(
            g.covered_by.is_some(),
            "Phase G (Implementation Governance) should be covered by wasm4pm"
        );
    }

    #[test]
    fn coverage_summary_contains_togaf() {
        let summary = coverage_summary();
        assert!(
            summary.contains("TOGAF"),
            "expected 'TOGAF' in coverage summary, got:\n{}",
            summary
        );
    }

    #[test]
    fn coverage_summary_lists_all_phases() {
        let phases = adm_phases();
        let summary = coverage_summary();
        for phase in &phases {
            assert!(
                summary.contains(&phase.id),
                "expected phase id '{}' in coverage summary",
                phase.id
            );
        }
    }

    #[test]
    fn phase_ids_are_unique() {
        let phases = adm_phases();
        let mut seen = std::collections::HashSet::new();
        for p in &phases {
            assert!(seen.insert(&p.id), "duplicate phase id: {}", p.id);
        }
    }
}
