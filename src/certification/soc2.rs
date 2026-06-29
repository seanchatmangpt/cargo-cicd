// src/certification/soc2.rs
//
// SOC2 Trust Service Criteria evidence mapping for cargo-cicd process evidence.

/// The five SOC2 Trust Service Categories.
#[derive(Debug, Clone)]
pub enum TrustCategory {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

impl TrustCategory {
    /// Short display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            TrustCategory::Security => "Security",
            TrustCategory::Availability => "Availability",
            TrustCategory::ProcessingIntegrity => "Processing Integrity",
            TrustCategory::Confidentiality => "Confidentiality",
            TrustCategory::Privacy => "Privacy",
        }
    }
}

/// A single SOC2 criterion and how cargo-cicd evidence satisfies it.
#[derive(Debug)]
pub struct Soc2Criterion {
    /// Criterion identifier, e.g. "CC6.1".
    pub id: String,
    /// Trust Service Category this criterion belongs to.
    pub category: TrustCategory,
    /// Human-readable description of the criterion.
    pub description: String,
    /// How cargo-cicd process evidence covers this criterion.
    pub evidence_mapping: String,
    /// The cargo-cicd command(s) that produce relevant evidence.
    pub covered_by_commands: Vec<String>,
    /// Whether cargo-cicd evidence satisfies this criterion.
    pub satisfied: bool,
}

/// Return all SOC2 criteria that cargo-cicd process evidence can satisfy.
pub fn criteria() -> Vec<Soc2Criterion> {
    vec![
        Soc2Criterion {
            id: "CC6.1".to_string(),
            category: TrustCategory::Security,
            description: "Logical access security — restrict access to information assets"
                .to_string(),
            evidence_mapping: "git phase tracking (git close) enforces commit access controls; \
                               affidavit BLAKE3 receipts prevent log tampering"
                .to_string(),
            covered_by_commands: vec![
                "cargo cicd git close".to_string(),
                "cargo cicd affidavit seal".to_string(),
            ],
            satisfied: true,
        },
        Soc2Criterion {
            id: "CC7.2".to_string(),
            category: TrustCategory::Security,
            description:
                "Incident detection — detect system events that indicate security incidents"
                    .to_string(),
            evidence_mapping: "wasm4pm oracle adjudicates every process event; \
                               REFUSE verdicts surface anomalies"
                .to_string(),
            covered_by_commands: vec![
                "cargo cicd status audit".to_string(),
                "cargo cicd evidence doctor".to_string(),
            ],
            satisfied: true,
        },
        Soc2Criterion {
            id: "A1.1".to_string(),
            category: TrustCategory::Availability,
            description: "Capacity management — maintain sufficient capacity to meet performance \
                          commitments"
                .to_string(),
            evidence_mapping: "target directory monitoring (target show/prune) prevents disk \
                               exhaustion that would block CI pipelines"
                .to_string(),
            covered_by_commands: vec![
                "cargo cicd target show".to_string(),
                "cargo cicd target prune".to_string(),
            ],
            satisfied: true,
        },
        Soc2Criterion {
            id: "PI1.1".to_string(),
            category: TrustCategory::ProcessingIntegrity,
            description: "Complete and accurate processing — process inputs completely and \
                          accurately"
                .to_string(),
            evidence_mapping: "XES / OCEL event logs trace every pipeline stage from start to \
                               complete with timestamps and verdicts"
                .to_string(),
            covered_by_commands: vec![
                "cargo cicd pipeline run".to_string(),
                "cargo cicd evidence audit".to_string(),
            ],
            satisfied: true,
        },
        Soc2Criterion {
            id: "PI1.4".to_string(),
            category: TrustCategory::ProcessingIntegrity,
            description: "Output integrity — outputs are complete, accurate, and timely"
                .to_string(),
            evidence_mapping: "BLAKE3 cryptographic receipts (affidavit seal/verify) bind outputs \
                               to their provenance trace"
                .to_string(),
            covered_by_commands: vec![
                "cargo cicd affidavit seal".to_string(),
                "cargo cicd affidavit verify".to_string(),
            ],
            satisfied: true,
        },
        Soc2Criterion {
            id: "C1.1".to_string(),
            category: TrustCategory::Confidentiality,
            description: "Confidentiality policy — identify and maintain confidential information"
                .to_string(),
            evidence_mapping: "cargo-cicd emits no secrets or credentials in process evidence; \
                               evidence logs contain only command metadata and verdicts"
                .to_string(),
            covered_by_commands: vec!["cargo cicd workspace doctor".to_string()],
            satisfied: true,
        },
    ]
}

/// Return a formatted multi-line compliance summary string.
pub fn compliance_summary() -> String {
    let all = criteria();
    let satisfied: Vec<_> = all.iter().filter(|c| c.satisfied).collect();
    let missing: Vec<_> = all.iter().filter(|c| !c.satisfied).collect();
    let pct = if all.is_empty() {
        0
    } else {
        satisfied.len() * 100 / all.len()
    };

    let mut out = "SOC2 Trust Service Criteria Coverage\n".to_string();
    out.push_str(&format!(
        "Coverage: {}% ({}/{})\n\n",
        pct,
        satisfied.len(),
        all.len()
    ));

    if !satisfied.is_empty() {
        out.push_str("Covered:\n");
        for c in &satisfied {
            out.push_str(&format!(
                "  [OK] {} ({}) \u{2014} {}\n",
                c.id,
                c.category.name(),
                c.description
            ));
        }
    }
    if !missing.is_empty() {
        out.push_str("\nGaps:\n");
        for c in &missing {
            out.push_str(&format!(
                "  [!!] {} ({}) \u{2014} {}\n",
                c.id,
                c.category.name(),
                c.description
            ));
        }
    }
    out
}
