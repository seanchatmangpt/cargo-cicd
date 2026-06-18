# Product Requirements Document: Vision 2030
## Ontology-Driven Ecosystem, Process Mining, and Distributed Adjudication

**Document Version:** 1.0  
**Prepared:** 2026-06-17  
**Scope:** Vision 2030 strategic initiatives from cargo-cicd thesis (sections 5.2, 5.3, deeper vision)

---

## Overview

This PRD structures product requirements from the Vision 2030 section of the cargo-cicd thesis. It covers ecosystem-scale architectural capabilities: ontology-driven capability generation, pluggable process models, distributed oracle adjudication, process mining analytics, anti-pattern detection, and ecosystem health metrics. All requirements are ecosystem-level features that build on or extend the current cargo-cicd architecture.

---

## FEATURE AREA 1: Ontology-Driven CLI Capability Manufacturing

### 1.1 Extended ggen Pipeline for Custom Capability Definition

**Title:** Extensible Capability Manufacturing — Developer-Declared Process Models

**User Story:**
- As a **project maintainer**
- I want to **declare custom compliance capabilities in RDF Turtle** without writing Rust boilerplate
- So that **my team's process conformance rules are formally specified, code-generated, and testable**

**Description:**
The ggen pipeline currently manufactures cargo-cicd's own noun-verb grammar from `ontology/cargo-cicd-capabilities.ttl`. Vision 2030 extends this to allow *teams* to define their own process conformance rules in an ontology and receive scaffolded implementations.

Example use case:
- Team defines: "Our process requires that all trybuild fixtures be updated before publication"
- Team extends `cicd-capabilities.ttl` with a `compliance:trybuild-freshness` concept
- Team runs `ggen --project-config myproject.ggen.toml`
- Team receives: `src/nouns/compliance.rs`, test scaffolding, help text, evidence emission code

**Acceptance Criteria:**
1. Project can specify a custom ggen.toml with project-specific ontology paths
2. ggen pipeline accepts RDF Turtle input defining custom nouns and verbs
3. Custom nouns are generated into `src/nouns/<custom>.rs` with NounCommand trait implementation
4. Custom verbs are scaffolded with evidence emission (ProcessEvent, XES, JSONL)
5. Generated code compiles without modification
6. Integration tests are scaffolded in `tests/cli/test_<custom>.rs`
7. Help text and README reference sections are generated
8. Regeneration is idempotent (running ggen twice produces identical output)

**Dependencies:**
- Current ggen implementation (working baseline)
- Ontology validation framework (RDF schema constraint checking)
- Tera template reusability across projects

**Effort:** L (Large)  
**Priority:** P1

**Success Metrics:**
- A second Rust team uses custom ggen to define their own compliance check
- Custom-generated verbs emit valid XES and pass wasm4pm audit
- Regeneration cycle takes < 5 seconds for typical ontologies

---

### 1.2 Capability Ontology Standard and Registry

**Title:** Shared Ontology Repository for Cross-Project Capability Reuse

**User Story:**
- As a **team lead across multiple Rust projects**
- I want to **discover and reuse common process compliance patterns** from other teams
- So that **my process definitions are consistent with ecosystem best practices and don't repeat work**

**Description:**
Vision 2030 includes a registry of published, community-maintained ontologies defining common CI/CD and process compliance patterns. Teams contribute ontologies; others import and extend them. This mirrors the crates.io ecosystem but for process definitions.

Example patterns (to be published):
- `crates:safety-critical-process` — Aerospace/medical device publication gating
- `crates:supply-chain-defense` — Dependency audit and pinning rules
- `crates:reproducible-build` — Toolchain lock and artifact hashing
- `crates:accessible-ci` — Process accessibility requirements

**Acceptance Criteria:**
1. Registry accessible at `registry.cargo-cicd.rs` (or similar)
2. Ontologies versioned in Turtle format with SemVer
3. Teams can declare ontology dependencies in `ggen.toml`: `requires = ["safety-critical-process/2.0"]`
4. ggen merges imported ontologies with local definitions (no conflicts)
5. Imported capabilities are marked as inherited in generated code comments
6. Team can override imported capability definitions (with warning)
7. Registry supports search by keyword (SKOS labeling)
8. Registry tracks adoption metrics (number of projects using each capability)

**Dependencies:**
- Custom capability manufacturing (Feature 1.1)
- ggen semantic validation (checking for namespace conflicts, circular imports)

**Effort:** L (Large)  
**Priority:** P1 (enables ecosystem coordination)

**Success Metrics:**
- 5+ published ontologies in registry within 12 months
- 30% of cargo-cicd adopters using at least one imported ontology
- No namespace collision issues reported in 100+ published projects

---

## FEATURE AREA 2: Pluggable Process Conformance Models

### 2.1 Process Model Declaration and Runtime Enforcement

**Title:** Declarative Process Conformance Models — Team-Defined "What Constitutes Done"

**User Story:**
- As a **compliance officer at a safety-critical company**
- I want to **specify exactly which process steps must occur before a release** (e.g., "two independent evidence adjudications", "target pressure < 10%", "all trybuild fixtures updated")
- So that **the oracle enforces our process automatically, and violations are caught before publication**

**Description:**
Currently, wasm4pm is a generic oracle that validates basic XES structure. Vision 2030 introduces a process model format that teams declare as part of their evidence adjudication. The oracle receives both the evidence (XES) and the declared process model, then checks conformance.

Process model is a formal spec (in a language TBD: BPMN, Petri net, or custom DSL) that declares:
- Required workflow steps and their sequence
- Mandatory evidence properties (e.g., "verdict_claimed must be PASS for publish")
- Conditional branches (e.g., "if changes > 100 files, require code review evidence")
- Threshold rules (e.g., "target_pressure must be < 10%")

Example process model (pseudocode):
```
process PublishGate {
  step1: status show → verdict PASS
  step2: test changed → verdict PASS or WARN
  conditional: if changed_files > 100 → require_code_review_evidence
  threshold: target_pressure < 10_000_000_000 bytes
  sequence: step1 before step2 before publish
}
```

**Acceptance Criteria:**
1. Team declares process model in project-specific TOML or YAML
2. Process model is embedded in `cicd.toml` under `[conformance]` section
3. wasm4pm accepts process model as second input to `audit` command
4. wasm4pm validates evidence against declared process model
5. Verdict includes conformance check results (e.g., "Accept: all steps present and in order")
6. Violation reports include which step/rule failed and evidence location
7. Process model versioning: `cicd.toml` tracks which model version was used for each evidence batch
8. Backward compatibility: wasm4pm accepts evidence without a process model (generic validation)

**Dependencies:**
- wasm4pm oracle architecture (must support model input)
- ProcessEvent structure (must carry sufficient metadata for model checking)
- Evidence emission pattern (already stable)

**Effort:** XL (Extra Large — requires new oracle capability)  
**Priority:** P1 (foundational for safety-critical adoption)

**Success Metrics:**
- 3+ teams define and publish custom process models
- wasm4pm rejects 100% of evidence violating declared models (false-accept rate = 0)
- Model validation time < 1 second per evidence set

---

### 2.2 Custom Evidence Properties and Claim Extensions

**Title:** Extensible ProcessEvent Schema for Team-Specific Verdicts

**User Story:**
- As a **Rust team building medical device firmware**
- I want to **emit custom evidence properties** (e.g., "code_review_approval", "safety_audit_passed", "fuzz_test_duration_seconds")
- So that **our process model can enforce rules over these properties and the oracle can include them in conformance checks**

**Description:**
ProcessEvent currently has a fixed schema. Vision 2030 allows teams to extend it with custom claims. The oracle understands these extensions and enforces model rules over them.

Example extension:
```rust
let mut event = ProcessEvent::new("publish run", "PASS");
event.add_claim("code_review_approval", "reviewer:alice:2026-06-14T10:30:00Z");
event.add_claim("fuzz_test_iterations", "10000");
event.add_claim("safety_audit_passed", "true");
```

**Acceptance Criteria:**
1. ProcessEvent has `claims: HashMap<String, String>` field
2. Claims are preserved in XES serialization (custom `<string>` elements under event)
3. Claims are preserved in JSONL (separate "claims" object)
4. Schema validation: teams declare expected claim keys in process model
5. wasm4pm rejects evidence if declared claims are missing
6. Process model can reference claims in threshold rules: `safety_audit_passed == "true"`
7. Backward compatibility: evidence without claims passes validation (claims optional by default)
8. Claims are immutable once emitted (part of evidence hash)

**Dependencies:**
- ProcessEvent structure changes
- XES schema extension (custom elements)
- JSONL schema extension (claims object)
- wasm4pm model evaluation (claim-aware rule checking)

**Effort:** M (Medium)  
**Priority:** P2 (enhances P1 but not blocking)

**Success Metrics:**
- 5+ custom claims in use across published evidence
- wasm4pm claim validation has 0 false negatives
- Claim validation adds < 50ms to audit time

---

## FEATURE AREA 3: Distributed Adjudication and Threshold Signatures

### 3.1 Multi-Oracle Adjudication with Threshold Signing

**Title:** Distributed Consensus Verdicts — No Single Point of Trust Failure

**User Story:**
- As a **crates.io governance body**
- I want to **require that critical publications be adjudicated by N independent oracles**, with at least M signatures required before publication
- So that **no single oracle compromise can poison the ecosystem**

**Description:**
Instead of a single wasm4pm oracle, Vision 2030 supports submitting evidence to a network of independent oracles. Each returns a signed verdict. Publication requires threshold of valid signatures (e.g., 3 of 5). This is threshold cryptography applied to CI/CD.

Flow:
1. `cargo cicd publish run` emits evidence to `target/cargo-cicd/evidence/`
2. Developer submits evidence to oracle network (configurable endpoints)
3. Each oracle independently validates evidence against its declared model
4. Each oracle signs its verdict with its private key
5. Developer collects M of N signatures
6. Receipt aggregates all N verdicts; only M signatures are required for publication

Example receipt:
```json
{
  "evidence_hash": "sha256:abc123...",
  "process_model_version": "v1.2",
  "verdicts": [
    {"oracle_id": "o1.cargo-cicd.rs", "verdict": "Accept", "signature": "..."},
    {"oracle_id": "o2.cargo-cicd.rs", "verdict": "Accept", "signature": "..."},
    {"oracle_id": "o3.cargo-cicd.rs", "verdict": "Refuse", "signature": "..."}
  ],
  "threshold_config": {"m": 2, "n": 3},
  "threshold_met": true
}
```

**Acceptance Criteria:**
1. `cicd.toml` has `[oracle_network]` section with endpoint list and threshold config
2. `cargo cicd publish run` can submit to multiple oracles concurrently
3. Each oracle endpoint is contacted via HTTPS with mutual TLS
4. Each oracle response includes: verdict, timestamp, signature, public key
5. Signature verification uses Ed25519 or similar (team decision)
6. Threshold validation: receipt is valid only if M of N signatures are present and valid
7. Timeout handling: if N-M+1 oracles respond, publication can proceed (fault tolerance)
8. Backward compatibility: single-oracle mode still works (m=1, n=1)

**Dependencies:**
- wasm4pm oracle endpoint exposure (HTTP API)
- Signature scheme design (Ed25519, secp256k1, or Threshold BLS for better properties)
- Receipt format extension (aggregating multiple verdicts)
- Key distribution infrastructure (publishing oracle public keys)

**Effort:** XL (Extra Large — requires oracle network architecture)  
**Priority:** P1 (strategic for ecosystem trust)

**Success Metrics:**
- 3 independent oracle implementations deployed
- No single oracle signature validates 100% of receipts
- Threshold consensus time < 30 seconds for 3 oracles
- False-reject rate across network < 0.1% (consensus is reliable)

---

### 3.2 Oracle Identity and Key Rotation Management

**Title:** Oracle Key Lifecycle and Accountability Infrastructure

**User Story:**
- As a **cargo-cicd foundation stakeholder**
- I want to **establish oracle identities, track key versions, and support emergency key rotation**
- So that **oracle compromise is detectable, localized to a time window, and recovery is automated**

**Description:**
Distributed adjudication requires managing oracle identities and keys. Vision 2030 includes a key management infrastructure:

- Each oracle has a stable identity (`oracle_id`) and public key
- Keys are versioned; each key has a validity period
- Old keys remain usable for verifying old receipts
- Key rotation is announced in advance and executed with no downtime
- Compromised keys are revoked and all receipts signed with them are flagged
- Key directory is published and updated weekly

Example key directory entry:
```toml
[[oracles]]
id = "o1.cargo-cicd.rs"
name = "cargo-cicd Official Oracle 1"
public_key = "ed25519:..."
valid_from = "2026-01-01T00:00:00Z"
valid_until = "2027-01-01T00:00:00Z"
status = "active"
```

**Acceptance Criteria:**
1. Key directory published at `https://registry.cargo-cicd.rs/oracles/keys.toml`
2. Each key has validity period (not open-ended)
3. Key rotation: new key must be published 30 days before old key expires
4. Revocation: compromised keys are marked `status = "revoked"` with timestamp
5. Receipts include oracle key version used for signing
6. Verification tool checks key validity period and revocation status
7. Historical receipts remain valid if key was valid at signing time
8. Revocation notifications are published in a tamper-evident log (Merkle tree of hashes)

**Dependencies:**
- Oracle identity system (DNS-like structure or cryptographic commitment)
- Key versioning and rotation procedures
- Tamper-evident log infrastructure (optional but recommended for accountability)

**Effort:** M (Medium)  
**Priority:** P2 (operational security, follows P1 network setup)

**Success Metrics:**
- Key directory updated daily with no false positives
- Key rotation executed with zero publication delays
- Revocation detection < 5 minutes after announcement
- Historical receipt verification works for all receipts > 2 years old

---

## FEATURE AREA 4: Process Mining Analytics and Dashboards

### 4.1 Ecosystem-Wide Process Mining Analytics Service

**Title:** XES Evidence Aggregation and Anti-Pattern Detection

**User Story:**
- As a **Rust ecosystem researcher**
- I want to **aggregate anonymized XES evidence from published projects** and run process mining queries
- So that **I can identify anti-patterns, bottlenecks, and failure modes affecting Rust development**

**Description:**
Vision 2030 includes a centralized analytics service that collects XES evidence from projects that opt in. The service runs standard process mining algorithms to extract insights:

- **Bottleneck Detection:** Which process steps have the longest duration? Which are most frequently failing?
- **Anti-Pattern Detection:** "Projects that ship > 3 hours after last evidence adjudication have 3x post-publication issue rate"
- **Process Drift:** Are projects deviating from their declared process models?
- **Correlation Analysis:** "Projects that skip trybuild fixture updates have 2x higher test-failure rate in next release"

Examples of insights published weekly:
```
Top 5 anti-patterns this week:
1. Shipping without updating workspace health snapshot (24% of releases)
2. Target pressure > 5GB at publish time (18% of releases)
3. Skipping trybuild changed verification (12% of releases)
...

Median process metrics:
- status show duration: 0.8s
- test changed duration: 12.3s
- publish run duration: 2.1s
- Average time between evidence adjudication and publication: 6.4 hours

Ecosystem health score: 7.2/10 (up from 6.8 last week)
```

**Acceptance Criteria:**
1. Analytics service ingests XES/JSONL evidence via HTTPS API (opt-in)
2. Evidence is stored with workspace/project anonymized (hash of project name)
3. Process mining library (ProM or similar) processes evidence
4. Weekly report published at `analytics.cargo-cicd.rs/report`
5. Report includes: bottleneck analysis, anti-pattern detection, drift detection
6. Per-pattern insights show correlation with post-publication issues (if data available)
7. Ecosystem health score calculated from multiple metrics
8. Individual projects can see their metrics compared to percentiles

**Dependencies:**
- XES collection infrastructure (endpoint, authentication)
- Process mining library integration (ProM or python-pm4py wrapper)
- Data anonymization procedures (privacy-preserving hashing)
- Analytics dashboard frontend

**Effort:** L (Large — significant infrastructure)  
**Priority:** P2 (valuable but not blocking)

**Success Metrics:**
- 100+ projects submitting evidence within first year
- Weekly reports published consistently
- Reported anti-patterns are actionable (teams report implementing recommendations)
- Ecosystem health score correlates with GitHub Rust language trends (validation)

---

### 4.2 Process Conformance Scoring and Reputation System

**Title:** Project-Level Conformance Score Based on Evidence History

**User Story:**
- As a **Cargo dependency resolver**
- I want to **compute a conformance score for each crate version** based on process evidence
- So that **crates with higher conformance scores bubble up in search results and `cargo audit` reports**

**Description:**
Building on ecosystem analytics, Vision 2030 assigns each published crate version a conformance score (0–100) based on the evidence that accompanied its publication:

- Did the release go through the full process (all declared steps)?
- Was the process compliant with declared model (no deviations)?
- What was target pressure, dirty files, toolchain state?
- How long after evidence adjudication was the release published?
- Were all dependencies themselves published from conformant processes?

Example scoring:
```
conformance_score(log4rs v0.13.3) = 94
  - Full process followed: +30 points
  - Declared model conformant: +25 points
  - Target pressure < 5%: +15 points
  - Published < 1 hour after adjudication: +15 points
  - All dependencies high-conformance: +9 points
  - Total: 94/100
```

**Acceptance Criteria:**
1. Conformance score published in `Cargo.toml` metadata (or separate manifest)
2. Score calculated by analytics service from evidence
3. Calculation formula is published and stable across releases
4. Cargo dependency resolver surfaces conformance score in `cargo tree` output
5. `cargo audit` reports conformance scores alongside vulnerability data
6. Score history available (graph showing trend over last 10 releases)
7. Scores are reproducible (same evidence always produces same score)
8. Benchmarking: individual scores compared to ecosystem percentile

**Dependencies:**
- Ecosystem analytics service (Feature 4.1)
- Evidence adjudication completeness (all verbs emit evidence)
- Cargo metadata extension (conformance_score field)

**Effort:** M (Medium)  
**Priority:** P2

**Success Metrics:**
- 80% of published crates have conformance scores
- Crates with scores > 90 report 50% fewer post-publication issues
- Developers report using score in dependency selection
- No manipulation: developers cannot artificially inflate scores

---

### 4.3 Interactive Process Mining Dashboard and Visualization

**Title:** Web Dashboard for Process Mining Queries and Exploration

**User Story:**
- As a **project maintainer**
- I want to **explore my project's process history graphically** (timeline, bottlenecks, variances)
- So that **I can identify where time is being spent and which steps are failing**

**Description:**
Vision 2030 includes a web dashboard (`dashboard.cargo-cicd.rs`) where maintainers can upload or link their evidence archive and explore it visually:

- **Process Timeline:** Gantt chart showing all events in a release cycle
- **Bottleneck View:** Which steps take longest? Which are slowest across all runs?
- **Failure Analysis:** Which events lead to FAIL verdicts? Common error patterns?
- **Conformance View:** How does my actual process compare to my declared model?
- **Trend Analysis:** Is my median publish time increasing or decreasing?
- **Dependency Graph:** Which steps depend on which? Critical path analysis?

Example dashboard views:
- "Last 10 releases: median process duration 2.3 hours, std dev 0.8 hours"
- "Target pressure is the bottleneck: median 8.2GB, ranges from 2.4GB to 15.6GB"
- "Publish run started, but test changed failed 3 times; issue in changed file detection"

**Acceptance Criteria:**
1. Dashboard at `dashboard.cargo-cicd.rs` accepts project link or file upload
2. XES file parsed and events visualized in timeline
3. Bottleneck analysis: sorting events by duration, identifying outliers
4. Failure path analysis: tracing which events precede FAIL verdicts
5. Conformance comparison: overlaying actual process vs. declared model
6. Trend analysis: computing statistics over last N releases
7. Export: users can export analysis as JSON/CSV for reporting
8. Privacy: projects are never indexed; URLs are unlisted, unguessable

**Dependencies:**
- XES parsing library (Python or Rust)
- Web dashboard frontend (React or similar)
- Process mining algorithms (bottleneck, critical path, conformance)

**Effort:** L (Large)  
**Priority:** P3 (nice-to-have, valuable for users but not blocking)

**Success Metrics:**
- 50+ projects using dashboard within first year
- Average session time > 10 minutes (indicates value)
- Users report implementing bottleneck recommendations
- Dashboard remains available with > 99.5% uptime

---

## FEATURE AREA 5: Process Model Definition Language and Standardization

### 5.1 Declarative Process Model Language (DPML)

**Title:** Domain-Specific Language for Process Conformance Rules

**User Story:**
- As a **process engineer**
- I want to **define conformance rules in a human-readable DSL** without writing JSON or BPMN XML
- So that **process definitions are easy to review, version control, and evolve**

**Description:**
Vision 2030 includes a simple, declarative language for defining process conformance rules. This language is not a general-purpose programming language — it is a constraint system designed for CI/CD process definitions.

Example DPML (pseudocode; exact syntax TBD):
```
process PublishGate v1.2 {
  
  # Define required steps and their order
  step status_show is "status show" {
    required_verdict: PASS
    timeout: 60s
  }
  
  step test_changed is "test changed" {
    required_verdict: PASS or WARN
    timeout: 300s
  }
  
  step publish_run is "publish run" {
    required_verdict: PASS
    timeout: 60s
  }
  
  # Define constraints over evidence properties
  constraint target_pressure {
    when: step publish_run
    rule: workspace.target_pressure_bytes < 10_000_000_000  # 10GB
    severity: BLOCK
  }
  
  constraint no_dirty_files {
    when: step publish_run
    rule: git.dirty_files.count == 0
    severity: BLOCK
  }
  
  constraint trybuild_fresh {
    when: step publish_run
    rule: claim("trybuild_updated") == "true"
    severity: WARN
  }
  
  # Define workflow sequence
  sequence {
    status_show
    then test_changed
    then publish_run
  }
  
  # Define conditional branches
  if changed_files > 100 {
    require_step "code_review_evidence"
  }
}
```

**Acceptance Criteria:**
1. DPML syntax defined and documented
2. Parser converts DPML → internal representation (AST or constraint graph)
3. wasm4pm interpreter evaluates DPML against evidence
4. DPML supports: step definitions, constraints, sequences, conditionals
5. Constraint language supports: comparisons, boolean logic, string matching, claim access
6. Error messages point to specific DPML line numbers on constraint violation
7. DPML files are versionable; `cicd.toml` tracks which version was used
8. DPML → JSON conversion for tools that prefer JSON

**Dependencies:**
- Language specification (formal grammar)
- Parser implementation (Rust or WASM)
- Interpreter implementation (wasm4pm integration)
- Validation and error reporting

**Effort:** L (Large)  
**Priority:** P1 (foundation for custom models)

**Success Metrics:**
- 10+ process definitions published in DPML
- No syntax errors in > 95% of first submissions
- DPML models evaluate faster than JSON (< 10ms per evaluation)
- Community contributes process templates

---

### 5.2 Process Model Library and Standards

**Title:** Published Process Model Templates for Common Scenarios

**User Story:**
- As a **project maintainer without a compliance team**
- I want to **use a pre-built process model** for "standard Rust publishing"
- So that **I can start emitting compliant evidence immediately without designing my own process**

**Description:**
Building on the DPML language, Vision 2030 includes a library of published, peer-reviewed process models for common scenarios:

- `standard-rust-publishing` — Basic process: status → test → publish
- `safety-critical-publishing` — Two-person review, threshold signatures
- `no-std-optimization` — Specialized for embedded/no-std crates
- `high-frequency-release` — Lightweight process for rapid iteration
- `ci-cd-heavy` — Comprehensive: all eight dimensions, multiple adjudications
- `enterprise-governance` — SOX/ISO requirements, audit trail

Each template includes:
- DPML process definition
- Rationale and assumptions
- Recommended conformance score thresholds
- Test evidence archive (example runs that pass/fail)

Example template entry:
```toml
[[process_models]]
name = "standard-rust-publishing"
version = "1.0"
authors = ["cargo-cicd-foundation"]
description = "Baseline process for all Rust releases"
dpml_url = "https://registry.cargo-cicd.rs/models/standard.dpml"
target_audience = "all"
maintenance_status = "stable"

[process_models.defaults]
required_verdict = "PASS"
target_pressure_threshold_bytes = 5_000_000_000  # 5GB
timeout_seconds = 600
```

**Acceptance Criteria:**
1. Model library published at `registry.cargo-cicd.rs/models/`
2. Each model includes: DPML, documentation, test evidence
3. Models versioned independently (SemVer)
4. Projects reference models by name: `process_model = "standard-rust-publishing/1.0"`
5. Model adoption tracked (number of projects using each)
6. Community can propose new models via PR
7. Models are maintainable (deprecated versions are marked, with migration path)
8. Model testing: community can submit evidence against model and see conformance

**Dependencies:**
- DPML language (Feature 5.1)
- Model registry infrastructure
- Community governance guidelines

**Effort:** M (Medium)  
**Priority:** P2

**Success Metrics:**
- 8+ process models published within first year
- 50% of publishing projects use one of the standard models
- Model maintenance cycle < 2 weeks (issues → resolution)
- Zero disputes about model correctness

---

## FEATURE AREA 6: Advanced Analytics and Ecosystem Intelligence

### 6.1 Vendor-Neutral Process Mining Tool Compatibility

**Title:** Export XES Evidence in Formats Compatible with Industry Process Mining Tools

**User Story:**
- As a **large enterprise adopting Rust with existing process mining infrastructure**
- I want to **feed cargo-cicd evidence into ProM, Celonis, or other industry tools**
- So that **my organization's existing analytics pipeline can analyze Rust process data without custom integration**

**Description:**
XES is an industry standard, but real-world process mining tools have variant interpretations and extensions. Vision 2030 ensures cargo-cicd evidence is compatible with major tools by:

1. **XES Strict Mode:** Validation that emitted XES conforms to the published XES standard
2. **Tool-Specific Exporters:** Plugins that reformat cargo-cicd evidence for specific tools
3. **Extension Mappings:** Standard mappings of cargo-cicd custom fields → tool-specific extensions
4. **Roundtrip Testing:** Evidence exported to tool format and re-imported must be lossless

Supported tools (target list):
- ProM (academic process mining, widely used)
- Celonis (enterprise process intelligence)
- ARIS (SAP process modeling)
- Signavio (cloud-based, collaborative)

**Acceptance Criteria:**
1. XES output passes `xesame` validator (XES reference implementation)
2. Each tool has an exporter producing tool-specific format
3. Exporters are tested with real tool instances
4. Custom fields are mapped to tool extensions (e.g., `verdict_claimed` → ProM `org:resource`)
5. Exporter preserves causality and timestamps (no information loss)
6. Documentation for each tool integration (how to import, what features are available)
7. Performance: exporting to any format < 500ms for typical evidence archive

**Dependencies:**
- XES validation library (or reference implementation)
- Tool-specific export plugins
- Integration testing with actual tool instances

**Effort:** M (Medium — per tool)  
**Priority:** P2

**Success Metrics:**
- Evidence successfully imported into ProM, Celonis, and at least one other tool
- Tool-specific queries run correctly on exported evidence
- Roundtrip testing: 100% information preservation
- Enterprise teams report using integration

---

### 6.2 Performance Bottleneck Detection and Recommendation Engine

**Title:** Automated Identification of Slow Process Steps and Optimization Suggestions

**User Story:**
- As a **project lead trying to reduce CI/CD cycle time**
- I want to **automatically identify which process steps are slowest** and get actionable recommendations
- So that **I can prioritize optimization efforts based on data, not guesses**

**Description:**
The analytics service (Feature 4.1) processes evidence to identify performance bottlenecks. Vision 2030 adds a recommendation engine that suggests optimizations:

- "Your test changed step takes 45s; ecosystem median is 12s. Suggestion: enable test parallelization or use `cargo nextest`"
- "Target pressure is 12GB; usually indicates cached build artifacts. Suggestion: add `cargo cicd target prune` to pre-release checklist"
- "Your publish run takes 3 minutes; requires manual review? Suggestion: enable automation for low-risk releases"

**Acceptance Criteria:**
1. Analytics service computes percentiles for each process step duration
2. For each step, recommendations are keyed by: language, workspace size, test count
3. Recommendations are evidence-based (derived from ecosystem data, not heuristics)
4. Project receives personalized recommendations based on its metrics
5. Each recommendation includes: problem description, suggestion, expected impact, effort level
6. Recommendations are ranked by potential time savings
7. Projects can opt in to weekly recommendation reports
8. Recommendation effectiveness is tracked (did projects implement suggestions? did it help?)

**Dependencies:**
- Ecosystem analytics service (Feature 4.1)
- Bottleneck detection algorithms
- Recommendation generation (heuristics based on aggregate data)

**Effort:** M (Medium)  
**Priority:** P3

**Success Metrics:**
- 50+ projects receive recommendations per week
- 30% of projects implement top-ranked recommendations
- Average time savings reported: 2–5 minutes per release (cumulative effect)
- Recommendation accuracy > 80% (teams rate suggestions as useful)

---

## FEATURE AREA 7: Safety-Critical and Compliance Infrastructure

### 7.1 Rust Ecosystem Certification Program

**Title:** Formal Certification Framework Using Process Evidence as Technical Basis

**User Story:**
- As a **safety engineer in aerospace**
- I want to **certify Rust libraries as meeting functional safety standards** (e.g., DO-178C) using process evidence
- So that **I can use community Rust crates in safety-critical systems without building equivalents in-house**

**Description:**
Vision 2030 envisions a Rust certification program (analogous to Common Criteria for security, DO-178C for functional safety) where libraries are certified based on process conformance evidence, not on code review alone.

Certification bodies (third-party organizations) can:
1. Define custom process models encoding their certification requirements
2. Accept evidence from library authors
3. Verify conformance using wasm4pm
4. Issue signed certificates for conformant libraries
5. Publish certified crate lists

Example certificate:
```
Library: std-no-alloc v2.1.0
Standard: DO-178C Level C (Safety-Critical)
Certifying Body: Rust-SAI (Rust Safety Institute)
Certified: 2026-06-14
Valid Until: 2027-06-14
Process Evidence: [XES hash]
Conformance Score: 98/100
Certificate: [cryptographically signed]
```

**Acceptance Criteria:**
1. Framework for third-party certification bodies (identity, key management)
2. Certification bodies define process models encoding their requirements
3. Authors submit evidence; certification body validates it
4. Certification is recorded in a tamper-evident log
5. Certificates are cryptographically signed and verifiable
6. Certificate includes: crate/version, standard, conformance evidence hash, valid date range
7. Certification bodies publish certified crate lists
8. Cargo resolver can surface certification status (e.g., `cargo tree --show-certified`)

**Dependencies:**
- wasm4pm oracle (for conformance checking)
- Third-party identity infrastructure (DNS-based or PKI)
- Certificate format (JSON + signature)
- Tamper-evident log (optional but recommended)

**Effort:** XL (Extra Large — requires ecosystem governance)  
**Priority:** P1 (strategic for safety-critical adoption)

**Success Metrics:**
- 3+ certification bodies operational within 18 months
- 50+ crates certified within first year
- Aerospace/medical companies report using certified Rust crates in production
- Certification improves crate adoption among regulated industries

---

### 7.2 AI-Generated Code Admissibility Tracking

**Title:** Evidence-Based Detection and Labeling of LLM-Generated Code

**User Story:**
- As a **safety-critical system integrator**
- I want to **know which parts of dependencies were generated by LLMs** and **require additional review for AI-generated code**
- So that **I can make informed risk decisions about AI assistance in dependencies**

**Description:**
Vision 2030 integrates `anti-llm-cheat` (from `lsp-max` crate) as a process evidence step. When code is generated by LLMs, the evidence record captures:

- Which files contain AI-generated code
- LLM model identifier and parameters
- Whether human review occurred (evidence claim: `llm_code_reviewed`)
- Percentage of LLM-generated vs. human-written code

This information flows into the process conformance model:
```
constraint llm_code_disclosure {
  when: step publish_run
  rule: if files_with_llm_code > 0 then claim("llm_code_reviewed") == "true"
  severity: BLOCK
}
```

And into the conformance score:
```
llm_code_score_adjustment = if llm_code_reviewed then -5 else -20
# High-quality LLM code with review: small penalty
# Unreviewed LLM code: large penalty
```

**Acceptance Criteria:**
1. `anti-llm-cheat` library integrated as optional adapter
2. Detects code blocks generated by major LLM models (GPT-4, Copilot, Claude, Gemini)
3. Evidence claim `llm_code_generated` tracks: files, model, percentage
4. Evidence claim `llm_code_reviewed` can be set by humans to attest review
5. ProcessEvent includes: `llm_generated_percentage: f32`
6. Process model rules can enforce review requirements for AI code
7. Conformance scorer applies penalty/bonus based on review status
8. `cargo cicd lsp check` command provides real-time feedback on AI code

**Dependencies:**
- `anti-llm-cheat` library (external, integrate as optional dependency)
- ProcessEvent claims extension (Feature 2.2)
- Process model rules (Feature 2.1)
- DPML language support for LLM-specific constraints

**Effort:** M (Medium)  
**Priority:** P2 (important for trust, but not blocking)

**Success Metrics:**
- 80% of new crates disclose AI-generated code in evidence
- Teams report using admissibility tracking in dependency decisions
- No false positives in LLM detection (< 1% false positive rate)
- Human-reviewed AI code scores 20+ points higher on conformance metrics

---

## Dependency Graph and Sequencing

```
Feature 1.1 (Custom Capability Manufacturing)
    ↓
Feature 1.2 (Ontology Registry) — depends on 1.1

Feature 2.1 (Process Model Declaration) — depends on 1.1
Feature 2.2 (Custom Evidence Claims) — depends on 2.1

Feature 3.1 (Multi-Oracle Adjudication) — depends on 2.1
Feature 3.2 (Oracle Key Management) — depends on 3.1

Feature 4.1 (Ecosystem Analytics) — depends on nothing (orthogonal)
Feature 4.2 (Conformance Scoring) — depends on 4.1
Feature 4.3 (Analytics Dashboard) — depends on 4.1

Feature 5.1 (DPML Language) — depends on 2.1
Feature 5.2 (Process Model Library) — depends on 5.1

Feature 6.1 (Tool Compatibility) — depends on nothing (orthogonal)
Feature 6.2 (Bottleneck Detection) — depends on 4.1

Feature 7.1 (Certification Program) — depends on 3.1, 5.2
Feature 7.2 (AI Code Tracking) — depends on 2.2, 5.1
```

---

## Release Planning Recommendation

**Phase 1 (Vision 2030 Year 1):**
- Feature 1.1 (Custom Capabilities) — P1, XL effort
- Feature 2.1 (Process Models) — P1, XL effort
- Feature 5.1 (DPML Language) — P1, L effort
- Feature 3.1 (Multi-Oracle Network) — P1, XL effort
- Feature 4.1 (Ecosystem Analytics) — P2, L effort

**Phase 2 (Vision 2030 Year 2):**
- Feature 1.2 (Ontology Registry) — P1, L effort
- Feature 2.2 (Custom Claims) — P2, M effort
- Feature 3.2 (Key Management) — P2, M effort
- Feature 5.2 (Model Library) — P2, M effort
- Feature 4.2 (Conformance Scoring) — P2, M effort
- Feature 6.1 (Tool Compatibility) — P2, M effort

**Phase 3 (Vision 2030 Year 3):**
- Feature 4.3 (Analytics Dashboard) — P3, L effort
- Feature 6.2 (Bottleneck Detection) — P3, M effort
- Feature 7.1 (Certification Program) — P1 (critical), XL effort
- Feature 7.2 (AI Code Tracking) — P2, M effort

---

## Success Metrics: Vision 2030 Overall

By 2030, success is measured by:

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Ecosystem Coverage** | 50% of published Rust crates carry wasm4pm receipts | Primary adoption metric |
| **Process Evidence Volume** | 1M+ XES events per day aggregated | Indicates active use |
| **Certification Programs** | 5+ active third-party certification bodies | Demonstrates ecosystem maturity |
| **Conformance Adoption** | 70% of adopters using custom process models | Shows extensibility value |
| **Bottleneck Reduction** | Teams report 20% median reduction in CI/CD cycle time | Practical benefit realization |
| **Supply Chain Trust** | Industry publications cite cargo-cicd as supply chain defense | Market recognition |
| **Safety-Critical Adoption** | 10+ aerospace/medical projects using certified Rust | Regulatory acceptance |
| **Community Models** | 20+ published process model templates | Ecosystem richness |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-06-17 | Claude Code | Initial extraction from thesis Vision 2030 sections |

