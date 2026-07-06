# Phase 3 Regulatory, Governance, and Ecosystem Maturation

**Vision 2030 — Phase 3: Regulatory Integration and Ecosystem Leadership**  
**Document Type:** Technical Design  
**Status:** Proposed  
**Date:** 2026-06-17  
**Audience:** cargo-cicd core team, regulatory affairs leads, ecosystem governance council  

---

## Overview

Phase 3 (target completion: 24-36 months post-Phase 2) transforms cargo-cicd from an ecosystem platform into a regulatory compliance infrastructure. The key themes are:

1. **Domain-specific regulatory mappings**: DO-178C (aviation), FDA 21 CFR Part 11 (medical devices), ISO 26262 (automotive), IEC 62443 (industrial cybersecurity).
2. **Community governance**: Process mining committee, ontology review board, certification body accreditation.
3. **Extended ggen pipeline**: Custom ontologies compiled into domain-specific CLI verifiers.
4. **Anti-pattern detection**: ML model (LSTM on XES sequences) detecting non-conformant process patterns.
5. **2030 Adoption Vision**: What 50%+ crates.io adoption looks like operationally.

**Phase 3 Target**: 50%+ of published crates.io crates with active development teams using cargo-cicd evidence by 2030.

---

## Table of Contents

1. [Extended ggen Pipeline](#1-extended-ggen-pipeline)
2. [DO-178C Integration](#2-do-178c-integration)
3. [FDA 21 CFR Part 11 Mapping](#3-fda-21-cfr-part-11-mapping)
4. [Community Governance Model](#4-community-governance-model)
5. [Certification Body Accreditation](#5-certification-body-accreditation)
6. [Anti-Pattern Detection](#6-anti-pattern-detection)
7. [2030 Adoption Vision](#7-2030-adoption-vision)

---

## 1. Extended ggen Pipeline

### 1.1 Current Pipeline (Phase 1)

```
ontology/cargo-cicd-capabilities.ttl
    ↓ [SPARQL inference]
    ↓ [Tera templates]
    ↓
src/nouns/<noun>.rs   (Rust CLI modules)
tests/cli/            (Test scaffolding)
docs/reference/       (Documentation)
```

### 1.2 Phase 3 Extended Pipeline

Phase 3 extends ggen to produce domain-specific compiled verifiers:

```
ontology/cargo-cicd-capabilities.ttl
ontology/regulatory/<domain>.ttl        ← NEW: Regulatory ontology
ontology/custom/<org>-capabilities.ttl  ← NEW: Org-specific
    ↓ [SPARQL inference with regulatory rules]
    ↓ [Extended Tera templates]
    ↓
src/nouns/<noun>.rs                    ← Existing
tests/cli/                             ← Existing
docs/reference/                        ← Existing
verifiers/<domain>-verifier/           ← NEW: Domain verifier binary
  ├── src/main.rs                      ← Standalone verifier binary
  ├── Cargo.toml                       ← Self-contained crate
  └── src/rules/                       ← Compiled regulatory rules
```

### 1.3 Domain Verifier Architecture

A domain verifier is a standalone Rust binary that can verify cargo-cicd evidence against regulatory requirements without requiring cargo-cicd itself:

```rust
// verifiers/do178c-verifier/src/main.rs

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// XES evidence file to verify
    #[arg(short, long)]
    evidence: PathBuf,
    
    /// DAL (Design Assurance Level): A, B, C, D
    #[arg(short, long, default_value = "C")]
    dal: DalLevel,
    
    /// Output format: human, json, junit
    #[arg(short, long, default_value = "human")]
    format: OutputFormat,
}

fn main() {
    let args = Args::parse();
    let evidence = XesParser::parse(&args.evidence).expect("Failed to parse XES");
    let checker = Do178cChecker::new(args.dal);
    let result = checker.verify(&evidence);
    
    match args.format {
        OutputFormat::Human => print_human_readable(result),
        OutputFormat::Json => print_json(result),
        OutputFormat::Junit => print_junit_xml(result),
    }
    
    if !result.compliant {
        std::process::exit(1);
    }
}
```

**Key property**: The domain verifier is generated from the regulatory ontology by ggen. It is not handwritten. The ontology is the authoritative source of the regulatory rules; the verifier is manufactured.

### 1.4 ggen.toml Extended Configuration

```toml
# ggen.toml (Phase 3)
[ontology]
path = "ontology/cargo-cicd-capabilities.ttl"

[[ontology.imports]]
path = "ontology/regulatory/do178c.ttl"
namespace = "https://cargo-cicd.rs/ontology/regulatory/do178c#"
prefix = "do178c"

# Generate domain verifier
[[outputs]]
template = "templates/domain_verifier.rs.tera"
output = "verifiers/{regulatory_domain}-verifier/src/main.rs"
per = "regulatory_domain"

[[outputs]]
template = "templates/domain_verifier_cargo.toml.tera"
output = "verifiers/{regulatory_domain}-verifier/Cargo.toml"
per = "regulatory_domain"
```

---

## 2. DO-178C Integration

DO-178C is the standard for airborne software ("Software Considerations in Airborne Systems and Equipment, Radio Communication Equipment, and Ground Support Equipment"). It defines Software Design Assurance Levels (DAL) A through E.

### 2.1 DAL-Specific Requirements Mapping

| DO-178C Section | Requirement | DAL A | DAL B | DAL C | DAL D |
|----------------|-------------|-------|-------|-------|-------|
| §6.3 Software Verification Process | Independent verification | Required | Required | Recommended | Optional |
| §6.4.4 Reviews and Analyses | Code review documented | Required | Required | Required | Optional |
| §9.1 Software Plans | Plans reviewed | Required | Required | Required | Optional |
| §12.1 Software Quality Assurance | QA records | Required | Required | Required | Optional |
| §11 Software Configuration Mgmt | Change records | Required | Required | Required | Required |

### 2.2 DO-178C Ontology

```turtle
# ontology/regulatory/do178c.ttl
@prefix do178c: <https://cargo-cicd.rs/ontology/regulatory/do178c#> .
@prefix cc: <https://cargo-cicd.rs/ontology/capabilities#> .

# DAL Levels
do178c:DalA a do178c:DesignAssuranceLevel ;
    rdfs:label "DAL A" ;
    do178c:description "Failure condition is catastrophic" .

do178c:DalB a do178c:DesignAssuranceLevel ;
    rdfs:label "DAL B" ;
    do178c:description "Failure condition is hazardous" .

# Section mapping: what activities satisfy DO-178C §6.3
do178c:Section63Requirement a do178c:Requirement ;
    do178c:dalLevel do178c:DalA, do178c:DalB ;
    do178c:requiredActivity cc:evidence-audit ;
    do178c:independentVerification true ;
    do178c:description "Software verification must be performed by an independent party" .

# The gate check activity is required for all DALs
do178c:SectionQARequirement a do178c:Requirement ;
    do178c:dalLevel do178c:DalA, do178c:DalB, do178c:DalC ;
    do178c:requiredActivity cc:publish-run ;
    do178c:requiredEvidence "Accept" ;
    do178c:maxEvidenceAgeDays 30 ;
    do178c:description "Software release requires oracle-adjudicated evidence not older than 30 days" .
```

### 2.3 DO-178C Verifier Output

```
$ do178c-verifier --evidence target/cargo-cicd/evidence/evt-*.xes --dal B

DO-178C DAL B Compliance Verification
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Section  Requirement                    Evidence            Status
─────────────────────────────────────────────────────────────────
§6.3     Independent verification       Accept (2026-06-15) PASS
§9.1     Plans reviewed                 No evidence found   FAIL
§12.1    QA records present             Accept (2026-06-10) PASS
§11      Configuration management       WARN: no CM records WARN
─────────────────────────────────────────────────────────────────
Overall: FAIL (1 failure, 1 warning)

Failure: §9.1 requires documented review of software plans.
  → Run `cargo cicd evidence audit --standard do178c-dal-b` to see required activities.
```

### 2.4 DO-330 Tool Qualification

DO-330 (Tool Qualification Considerations) requires that tools used in the development of safety-critical software be qualified to a level commensurate with the DAL of the software they support.

For cargo-cicd as a tool used in DO-178C contexts:

1. **Tool qualification plan**: cargo-cicd must have a documented Tool Qualification Plan (TQP).
2. **Verification of tool output**: XES evidence provides the audit trail for tool qualification.
3. **Operational requirements**: cargo-cicd's deterministic behavior (same input → same output) must be documented and tested.

Phase 3 deliverable: DO-330 Tool Qualification Package for cargo-cicd, generated from the ontology and evidence logs.

---

## 3. FDA 21 CFR Part 11 Mapping

FDA 21 CFR Part 11 ("Electronic Records; Electronic Signatures") governs electronic records in FDA-regulated activities (medical device software, pharmaceutical manufacturing).

### 3.1 Part 11 Requirements for Cargo-cicd Evidence

| 21 CFR Part 11 Section | Requirement | cargo-cicd Mapping |
|------------------------|-------------|-------------------|
| §11.10(a) | Validate systems to ensure accuracy, reliability | XES schema validation; wpm oracle adjudication |
| §11.10(b) | Ability to generate accurate and complete copies | XES + JSONL export endpoints |
| §11.10(c) | Protection of records to enable accurate retrieval | Append-only evidence directory; no deletion |
| §11.10(d) | Limit system access to authorized individuals | `--confirm` flag for destructive operations |
| §11.10(e) | Secure computer-generated audit trails | XES timestamp integrity; oracle signature |
| §11.50(a) | Signed electronic records must include signer's name | Oracle key fingerprint (ADR-013) |
| §11.50(b) | Electronic signatures must be permanently linked | Threshold receipt cryptographic binding |
| §11.70 | Electronic signatures are equivalent to handwritten | Oracle threshold signature (FROST Ed25519) |

### 3.2 Part 11 Compliance Ontology

```turtle
# ontology/regulatory/fda-part11.ttl
@prefix fda: <https://cargo-cicd.rs/ontology/regulatory/fda-part11#> .

fda:Section1110a a fda:Requirement ;
    fda:title "System Validation" ;
    fda:requiredEvidence true ;
    fda:requiredOracleAdjudication true ;
    fda:minOracleVersion "0.9.0" ;
    fda:description "Systems that create, modify, or transmit electronic records must be validated" .

fda:Section1110e a fda:Requirement ;
    fda:title "Audit Trail" ;
    fda:requiredXesIntegrity true ;
    fda:requiredTimestampAuthority true ;
    fda:description "Computer-generated audit trails must be computer generated, date and time stamped" .

fda:Section1150 a fda:Requirement ;
    fda:title "Electronic Signatures" ;
    fda:requiredSignerIdentity true ;  # oracle key fingerprint
    fda:requiredSignatureLinking true ; # threshold receipt binding
    fda:description "Electronic signatures must include the printed name of the signer" .
```

### 3.3 21 CFR Part 11 Audit Package

Phase 3 deliverable: `cargo cicd audit fda-part11` generates an audit package containing:

1. **Audit trail export**: All XES evidence for a specified period, exported in timestamped order.
2. **Oracle identity certificate**: Oracle key fingerprint with chain of trust to key registry.
3. **System validation record**: cargo-cicd version, test suite results, feature configuration.
4. **Electronic signature binding**: Threshold receipt binding evidence to oracle signatures.
5. **Access control log**: `--confirm` flag usage log (who authorized destructive operations).

```sh
$ cargo cicd audit fda-part11 --from 2026-01-01 --to 2026-06-17 --output fda-audit-2026.zip

Generating FDA 21 CFR Part 11 Audit Package...
  ✓ Evidence export: 847 XES files (2026-01-01 to 2026-06-17)
  ✓ Oracle certificates: wasm4pm/0.9.2 key chain verified
  ✓ System validation: cargo-cicd v26.6.2 (all tests pass)
  ✓ Threshold receipts: 43 publish operations with Accept verdict
  ✓ Access control: 0 unauthorized destructive operations
  
Audit package: fda-audit-2026.zip (12.4MB)
  Use with FDA's ERES (Electronic Records/Electronic Signatures) audit tool.
```

---

## 4. Community Governance Model

### 4.1 Governance Structure

As cargo-cicd becomes regulatory infrastructure, a community governance model is essential to prevent single-vendor control and ensure long-term sustainability.

**Governance Bodies**:

1. **Technical Steering Committee (TSC)**: 7 members elected annually. Responsible for technical direction, release planning, and breaking changes. Minimum 3 different organizations represented.

2. **Process Mining Committee (PMC)**: 5 members with process mining expertise. Reviews and approves new process model templates. Ensures conformance checking algorithms are sound.

3. **Ontology Review Board (ORB)**: 5 members with RDF/ontology expertise. Reviews third-party ontology submissions to the registry. Ensures namespace hygiene and semantic correctness.

4. **Certification Advisory Council (CAC)**: 9 members representing regulated industries (aviation, medical, automotive, industrial). Reviews certification body accreditation requests.

5. **Security Council**: 3 members with cryptography and security expertise. Reviews oracle key ceremonies, threshold signature implementations, and key revocation decisions.

### 4.2 Decision-Making Process

Decisions follow the Apache Software Foundation model:

- **Lazy consensus**: Minor changes (bug fixes, documentation) go in if no objections within 72 hours.
- **Supermajority (2/3)**: Significant changes (new features, breaking changes) require 2/3 of TSC to approve.
- **Unanimous**: Security decisions (key revocation, oracle compromise response) require unanimous Security Council.

### 4.3 Process Mining Committee Charter

The PMC is responsible for:

1. **Algorithm review**: Reviewing conformance checking algorithms proposed for inclusion. Ensuring they are mathematically sound and correctly implemented.
2. **Reference model publication**: Publishing reference process models for common use cases (basic release, SLSA-L3, DO-178C DAL-C).
3. **Research liaison**: Maintaining relationships with academic process mining community (University of Eindhoven ProM group, IEEE Task Force on Process Mining).
4. **Fitness metric standards**: Defining what constitutes "conformant" (e.g., fitness ≥ 0.95).

### 4.4 Ontology Review Board Charter

The ORB is responsible for:

1. **Namespace registration**: Maintaining the namespace registry to prevent collisions.
2. **Quality standards**: Ensuring submitted ontologies meet quality standards (valid Turtle, SKOS compliance, required documentation).
3. **Deprecation policy**: Managing deprecated capability ontologies (grace period, migration guidance).
4. **Cross-org alignment**: Identifying when two organizations have defined semantically equivalent capabilities and proposing consolidation.

---

## 5. Certification Body Accreditation

### 5.1 What Is Certification Body Accreditation?

A certification body is an organization that provides independent verification of compliance claims. For cargo-cicd, a certified certification body:

1. Operates one or more oracles that meet cargo-cicd's oracle interface contract.
2. Has been audited by the Certification Advisory Council.
3. Has completed the DKG ceremony for at least one threshold group.
4. Agrees to the certification body code of conduct.

**Examples**:
- Compliance vendor offering DO-178C tool qualification services.
- National standards body (NIST, DIN, BSI) operating an oracle for national standards.
- Academic institution providing research-oriented oracle for process mining research.

### 5.2 Accreditation Process

**Step 1: Application**

The candidate certification body submits:
- Organization profile and legal standing
- Technical capability assessment (oracle infrastructure, key management, security practices)
- Scope declaration (which standards/regulations they can adjudicate)
- References from at least 2 organizations they have previously certified

**Step 2: Technical Audit**

The Security Council conducts a technical audit:
- Oracle implementation review (does it correctly implement the interface contract?)
- Key management practices (hardware security modules, ceremony procedures, revocation capability)
- Infrastructure security (penetration testing report required)
- Uptime and incident response procedures

**Step 3: Ceremony Participation**

The candidate participates in a DKG ceremony for a test threshold group. The ceremony transcript is published and reviewable.

**Step 4: CAC Decision**

The Certification Advisory Council votes. Approval requires 2/3 majority. Approved bodies are added to the oracle registry with accreditation metadata.

**Step 5: Ongoing Compliance**

Accredited bodies:
- Undergo annual re-audit.
- Notify the Security Council immediately upon any key compromise or security incident.
- Publish a public log of oracle verdicts (aggregated, not individual workspace data).

### 5.3 Accreditation Levels

| Level | Requirements | Trust |
|-------|-------------|-------|
| Community | Self-attestation, open source | Lowest |
| Verified | Technical audit | Medium |
| Accredited | Full CAC approval + annual audit | High |
| Regulatory | Accredited + industry body recognition | Highest |

Only Accredited and Regulatory bodies can participate in threshold groups for regulated software contexts.

---

## 6. Anti-Pattern Detection

### 6.1 Problem Statement

Process conformance checking (fitness calculation) tells you whether a trace conforms to a declared model. Anti-pattern detection is different: it detects suspicious or anomalous patterns in evidence logs that may indicate:

- Process gaming (circumventing required steps)
- Evidence tampering attempts
- Systematic non-compliance
- Emerging quality problems before they manifest as failures

### 6.2 ML Model Architecture

Phase 3 introduces an LSTM (Long Short-Term Memory) model trained on XES event sequences to detect anomalous patterns.

**Why LSTM?**

Process event logs are sequential data with temporal dependencies. LSTM excels at:
- Long-range dependencies (an event from 2 hours ago may be relevant to the current event)
- Variable-length sequences (different traces have different numbers of events)
- Pattern recognition in time series

**Input Representation**

Each event in a trace is encoded as a feature vector:

```rust
struct EventFeatureVector {
    // Categorical features (one-hot encoded)
    command_id: u32,            // Index into command vocabulary
    verdict_id: u8,             // 0=PASS, 1=WARN, 2=FAIL
    lifecycle_id: u8,           // 0=start, 1=complete
    
    // Numerical features (normalized 0-1)
    duration_ms_normalized: f32, // duration / max_observed_duration
    hour_of_day_normalized: f32, // hour / 23.0
    day_of_week_normalized: f32, // day / 6.0
    
    // Boolean features
    oracle_adjudicated: bool,
    is_pipeline_run: bool,
}
```

**LSTM Architecture**

```
Input sequence: [EventFeatureVector]  (variable length)
    ↓
Embedding layer: d_model = 64
    ↓
LSTM layer 1: hidden_size = 128, dropout = 0.2
    ↓
LSTM layer 2: hidden_size = 64, dropout = 0.2
    ↓
Linear layer: 64 → 16
    ↓
ReLU activation
    ↓
Linear layer: 16 → 1
    ↓
Sigmoid
    ↓
Output: anomaly_score ∈ [0, 1]
```

**Anomaly Score Interpretation**:
- 0.0-0.3: Normal pattern
- 0.3-0.6: Slightly unusual (worth monitoring)
- 0.6-0.85: Suspicious (alert generated)
- 0.85-1.0: Highly anomalous (immediate investigation)

**Implementation** (using `candle-core` for pure-Rust LSTM):

```rust
// src/advanced/anti_pattern/lstm_model.rs

use candle_core::{Tensor, Device, DType};
use candle_nn::{Module, LSTM, Linear, VarMap};

pub struct AnomalyDetector {
    embedding: Linear,
    lstm1: LSTM,
    lstm2: LSTM,
    fc1: Linear,
    fc2: Linear,
    device: Device,
}

impl AnomalyDetector {
    pub fn load(weights_path: &Path) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        // Load pre-trained weights
        var_map.load(weights_path)?;
        // Build model from var_map...
        todo!("Initialize layers from var_map")
    }

    pub fn detect(&self, trace: &XesTrace) -> AnomalyResult {
        let features: Vec<EventFeatureVector> = trace.events.iter()
            .map(|e| EventFeatureVector::from_event(e))
            .collect();

        let input = self.encode_sequence(&features)?;
        let score = self.forward(&input)?;
        
        AnomalyResult {
            anomaly_score: score,
            is_anomalous: score > 0.6,
            explanation: self.explain(score, &features),
        }
    }
}
```

### 6.3 Training Dataset

The LSTM is trained on:

**Positive examples** (normal patterns): 
- All evidence logs from workspaces that have operator-verified `Accept` receipts.
- Synthetically generated conformant traces following the reference process model.

**Negative examples** (anomalous patterns):
- Known gaming patterns (test result timestamp backdating, evidence file substitution).
- Synthetically generated non-conformant traces.
- Traces with known process violations (from the conformance checker).

**Dataset scale** (Phase 3 target):
- 10M+ traces from 1,000+ workspaces (from analytics service, opt-in)
- 50K labeled anomalous patterns (from security research)
- Quarterly retraining cycle

### 6.4 Anti-Pattern Categories

The LSTM is trained to detect these specific anti-patterns:

| Pattern | Description | Detection Signal |
|---------|-------------|-----------------|
| Timestamp backdating | Events with timestamps earlier than the binary timestamp | Timing correlation |
| Skip-and-publish | `publish run` without preceding `test changed` | Ordering violation |
| Evidence flooding | 100+ evidence events emitted in < 1 second | Volume spike |
| Oracle shopping | Multiple oracle rejections before one acceptance | Oracle disagreement pattern |
| Verdict inflation | Claiming `PASS` immediately followed by `FAIL` | Verdict inconsistency |
| Phantom test run | `test changed` with 0ms duration | Duration anomaly |
| Command sequence reversal | `git close` before `test changed` | Ordering violation |

### 6.5 Anti-Pattern API Integration

Anti-pattern detection is available via the dashboard API:

```
GET /api/v1/anti-patterns?workspace=<id>&from=<iso>&sensitivity=medium

Response:
{
  "workspace_id": "...",
  "period": "7d",
  "total_traces": 200,
  "flagged_traces": 3,
  "patterns": [
    {
      "trace_id": "publish_run_phase_047",
      "anomaly_score": 0.78,
      "pattern": "skip-and-publish",
      "explanation": "publish run occurred 0.3s after status show with no test-changed event",
      "recommended_action": "Review publish trace and re-run with full pipeline"
    }
  ]
}
```

---

## 7. 2030 Adoption Vision

### 7.1 Target: 50%+ crates.io Adoption

By 2030, cargo-cicd's evidence emission should be present in 50%+ of actively maintained crates on crates.io. "Active" means a crate with at least one publish operation in the past 12 months.

**Current state (Phase 1, 2026)**: ~500 crates using cargo-cicd  
**Phase 2 target (2027)**: ~5,000 crates  
**Phase 3 target (2030)**: ~50%+ of ~200,000 active crates = ~100,000 crates

### 7.2 How Adoption Reaches 50%

**Adoption driver 1: Developer experience** (Phase 1)
- `cargo install cargo-cicd` → immediate value from `status show`
- No configuration required for basic use
- Zero-dependency binary (default build)

**Adoption driver 2: Cargo integration** (Phase 2)
- `cargo tree VERIFIED` badge for adjudicated crates
- `cargo add` warns about unadjudicated crates (opt-in)
- crates.io displays "Process Verified" badge

**Adoption driver 3: Regulatory requirements** (Phase 3)
- Aerospace and medical device companies require evidence for vendor crates
- Supply chain security frameworks (SLSA L3+) require evidence
- Government procurement contracts specify evidence requirements

**Adoption driver 4: Community tooling** (Phase 3)
- cargo-audit integrates receipt verification
- IDEs display certification status inline
- CI/CD systems (GitHub Actions, GitLab CI) have native cargo-cicd steps
- Dependabot / Renovate check evidence status before auto-merging upgrades

### 7.3 Operational Infrastructure at 50%+ Adoption

At 50% of 200,000 crates using cargo-cicd with evidence emission:

**Volume estimates**:
- 100,000 active crates × avg 2 releases/year = 200,000 publish operations/year
- 200,000 × 10 command executions/release = 2,000,000 evidence events/year
- 2,000,000 / 365 = ~5,500 evidence events/day across the ecosystem

This is manageable at low infrastructure cost:
- 5,500 events/day → 500MB/day XES storage (compressed)
- Analytics service: ClickHouse handles this at commodity hardware costs
- Oracle adjudication: 200,000 oracle calls/year = 550/day (trivial load)

**Infrastructure requirements at 50% adoption**:

| Component | Specification | Monthly Cost |
|-----------|--------------|-------------|
| Analytics ClickHouse | 8 vCPU, 32GB RAM, 10TB NVMe | ~$400/month |
| Oracle registry | GitHub Pages (static) | $0 |
| Collector (per workspace) | Embedded in cargo-cicd | $0 |
| Dashboard (self-hosted) | SQLite, no server needed | $0 |
| Dashboard (hosted) | 4 vCPU, 8GB RAM, PostgreSQL | ~$150/month |

### 7.4 Operational Runbook

At 50%+ adoption, the following operational procedures are required:

**Daily**:
- Monitor oracle key registry for revocation announcements
- Check analytics API health score (alert if < 80)
- Review anti-pattern detection alerts (high-severity)

**Weekly**:
- Oracle key registry backup and integrity check
- Analytics service capacity planning review
- TSC office hours (community Q&A)

**Monthly**:
- Oracle availability SLO review
- Anti-pattern model retraining trigger (if new patterns discovered)
- Certification body audit status review

**Quarterly**:
- TSC member election cycle
- ORB ontology submission queue review
- LSTM model retraining with latest 90-day evidence dataset
- Security Council key rotation review

**Annually**:
- Full certification body re-audit
- Regulatory ontology updates (DO-178C amendment tracking, FDA guidance updates)
- Phase N+1 planning kickoff

### 7.5 2030 Success Criteria

| Criteria | Target | Measurement |
|----------|--------|-------------|
| crates.io adoption | ≥50% active crates | crates.io stats API |
| Oracle adjudication coverage | ≥80% of publish operations | Analytics API |
| Regulatory framework coverage | DO-178C, FDA Part 11, ISO 26262, IEC 62443 | Ontology registry |
| Certification bodies accredited | ≥10 | CAC registry |
| Third-party ontologies | ≥50 | Registry count |
| Anti-pattern detection accuracy | ≥95% precision | Security audit |
| Ecosystem health score | ≥90/100 | Analytics API |
| Research publications | ≥20 papers citing cargo-cicd XES dataset | Google Scholar |

---

*Document version 1.0 — 2026-06-17*  
*See also: `docs/PHASE-2-DESIGN.md`, `docs/distributed-oracle-design.md`, `docs/process-mining-architecture.md`*
