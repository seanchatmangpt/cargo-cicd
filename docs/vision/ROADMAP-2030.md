# Vision 2030 Implementation Roadmap

**Prepared:** 2026-06-17  
**Status:** Phase 1 Planning (Q3 2026)  
**Audience:** cargo-cicd core team, ecosystem partners  

---

## Executive Summary

Vision 2030 is a strategic multi-year initiative to evolve cargo-cicd from a workspace health tool into a process-evidence ecosystem platform. The vision consists of **34 features** across **5 theme areas**, organized into **3 phases** spanning 18+ months (Q3 2026 → Q4 2027+).

**Critical Path:** Phases are sequentially dependent. Phase 1 (Q3 2026, 12–16 weeks) establishes the process evidence foundation and certification infrastructure. Phase 2 (Q4 2026 → Q1 2027, 12–16 weeks) scales to ecosystem adoption and process mining. Phase 3 (Q2 2027+, ongoing) refines the ontology-driven future and regulatory compliance.

---

## Phase 1: Foundation (Q3 2026, 12–16 weeks)

**Theme:** Process Evidence as First-Class Artifact + Certification Infrastructure  
**Go-Live Criteria:** Process evidence emission validated by wasm4pm. Published Cargo.toml [evidence] schema. First certification body onboarded.

### P0 Features (Blocking)

| # | Feature | Effort | Owner | Dependencies |
|---|---------|--------|-------|---|
| 1.1 | XES Event Stream Generation (XES 2.0 compliance) | M | Core | None |
| 1.2 | JSONL Companion Format (Process mining companion) | S | Core | 1.1 |
| 1.3 | Oracle Public Key Embedding in Evidence | M | Core | 1.1 |
| 1.4 | Receipt Hash Validation Framework | M | Core | 1.3 |
| 1.5 | Cargo.toml [evidence] Section Schema (RFC) | M | Ecosystem | 1.1, 1.3, 1.4 |
| 1.6 | Certification Body Integration (First body) | L | Cert | 1.1, 1.3, 1.4 |
| 1.7 | IEC 61508 / ISO 26262 Compliance Mapping | M | Cert | 1.6 |
| 1.8 | Process Conformance Evidence Collection | M | Core | 1.1 |
| 1.9 | Safety-Critical Crate Metadata Registry | L | Ecosystem | 1.6, 1.7 |

**Phase 1 Effort:** 80–100 person-days (~5–6 weeks FTE, 2-person team)

### P1 Features (Recommended, within Phase 1)

| # | Feature | Effort | Owner | Dependencies |
|---|---------|--------|-------|---|
| 2.1 | Audit Trail for Process State Changes | M | Core | 1.1, 1.8 |
| 2.2 | Evidence Archive URL & Metadata Fields | M | Ecosystem | 1.5 |
| 2.3 | Receipt Doctor Tool Enhancement | S | Core | 1.4 |
| 2.4 | Code Provenance Tracking (Human/AI/LLM flags) | M | Core | 1.1 |

**Phase 1 P1 Effort:** 40–50 person-days (~2.5–3 weeks FTE)

### Deliverables

1. **Process Evidence RFC** — Formal specification of XES 2.0 compliance, JSONL format, process mining compatibility.
2. **Cargo.toml [evidence] Specification** — RFC or enhancement proposal for Cargo ecosystem adoption.
3. **Certification Body SLA** — First certification provider onboarded; compliance mapping documented (IEC 61508, ISO 26262).
4. **Code Provenance Framework** — Mechanism to flag human-authored vs AI-generated code in XES traces.
5. **Test Suite** — 50+ tests validating:
   - XES generation and validity
   - JSONL format correctness
   - Receipt hash validation
   - Process conformance detection
   - Multi-oracle consensus (prepare for Phase 2)

### Go-Live Criteria (Phase 1 → Phase 2)

- [ ] All P0 features implemented and tested
- [ ] `cargo cicd <noun> <verb>` emits valid XES to `target/cargo-cicd/evidence/`
- [ ] `wpm audit <xes-file>` accepts 100% of generated XES (no validation errors)
- [ ] Cargo.toml [evidence] section documented and `cargo metadata` outputs it
- [ ] First certification body (e.g., Ferrous Systems, TrustInSoft) integrates and issues receipts
- [ ] IEC 61508 / ISO 26262 compliance checklist published
- [ ] Code provenance tracking integrated into evidence emission
- [ ] Safety-critical crate registry initialized with 10+ entries

---

## Phase 2: Adoption & Scale (Q4 2026 → Q1 2027, 12–16 weeks)

**Theme:** Ecosystem Adoption, Process Mining, Distributed Oracle Adjudication  
**Go-Live Criteria:** 20% of crates.io adopters publish evidence. Process mining dashboards operational. Multi-oracle consensus in production.

### P0 Features (Blocking)

| # | Feature | Effort | Owner | Dependencies |
|---|---------|--------|-------|---|
| 3.1 | Distributed Oracle Adjudication (M-of-N consensus) | L | Core | Phase 1 |
| 3.2 | Process Mining Dashboards (Disco/ProM integration) | L | Analytics | Phase 1 |
| 3.3 | Ecosystem Health Analytics Service | L | Analytics | Phase 1, 3.2 |
| 3.4 | Bottleneck Detection & Recommendation Engine | M | Analytics | 3.3 |
| 3.5 | Pluggable Process Model Architecture | L | Core | Phase 1 |
| 3.6 | Custom Ontology Support in ggen | M | Core | 3.5 |
| 3.7 | cargo tree VERIFIED Badge Display | M | Ecosystem | Phase 1 |
| 3.8 | cargo audit Receipt Verification | M | Ecosystem | Phase 1 |

**Phase 2 Effort:** 90–120 person-days (~6–7.5 weeks FTE, 2-person team)

### P1 Features (Recommended)

| # | Feature | Effort | Owner | Dependencies |
|---|---------|--------|-------|---|
| 4.1 | Reproducible Build Evidence Integration | M | Core | Phase 1 |
| 4.2 | Supply Chain Defense Scanning (dependency audit receipts) | M | Core | Phase 1 |
| 4.3 | Ontology Registry (SKOS-based capability marketplace) | L | Ecosystem | 3.6 |
| 4.4 | Process Model Export (vendor-neutral format) | M | Ecosystem | 3.5 |

**Phase 2 P1 Effort:** 50–60 person-days (~3–4 weeks FTE)

### Deliverables

1. **Distributed Oracle Consensus Protocol** — M-of-N threshold signature scheme. Specification, reference implementation (wasm4pm enhancement).
2. **Process Mining Dashboard** — Real-time/historical visualization of workspace process traces. Disco/ProM import/export.
3. **Ecosystem Analytics Service** — Public API (read-only) exposing:
   - Overall adoption metrics (% of crates.io with evidence)
   - Per-policy success rates
   - Bottleneck rankings (slowest stages across ecosystem)
   - Safety-critical certification status (by crate)
4. **Ontology Registry** — Web application at `registry.cargo-cicd.rs` supporting:
   - Search / discovery by keyword
   - Versioning & dependency resolution
   - Adoption metrics per ontology
5. **Test Suite** — 80+ tests validating:
   - Multi-oracle consensus correctness
   - Process mining compatibility
   - Dashboard data integrity
   - Custom ontology generation
   - Reproducible build evidence

### Go-Live Criteria (Phase 2 → Phase 3)

- [ ] All P0 features implemented and tested
- [ ] Distributed oracle consensus in production; 3+ oracles integrated
- [ ] Process mining dashboards display accurate workspace traces
- [ ] Ecosystem health analytics service stable (99.5% uptime)
- [ ] cargo tree shows VERIFIED/UNVERIFIED badges; cargo audit validates receipts
- [ ] Ontology registry operational; 5+ published ontologies
- [ ] 20%+ of new crates.io releases include [evidence] metadata
- [ ] Supply chain defense scanning detects and flags pinning violations

---

## Phase 3: Refinement & Regulation (Q2 2027+, ongoing)

**Theme:** Regulatory Compliance, Ontology-Driven Future, Ecosystem Deepening  
**Focus:** Long-term sustainability, regulatory framework alignment, community governance.

### P0 Features (Strategic)

| # | Feature | Effort | Owner | Dependencies |
|---|---------|--------|-------|---|
| 5.1 | Extended ggen Pipeline for Custom Capabilities | L | Core | Phase 2 |
| 5.2 | DO-178C Compliance Integration (Avionics) | L | Cert | Phase 1, Phase 2 |
| 5.3 | Certified Rust Library Registry (ecosystem curated) | M | Ecosystem | Phase 1, Phase 2 |
| 5.4 | Process Model Versioning & Evolution | M | Core | 5.1 |
| 5.5 | Anti-Pattern Detection (ML-based process mining) | L | Analytics | Phase 2, 5.1 |
| 5.6 | Distributed Trust & Multi-Provider Ecosystem | L | Ecosystem | Phase 2 |

**Phase 3 Effort:** 120+ person-days (ongoing); phases in parallel.

### Deliverables

1. **Extended ggen Capability Language** — Allow teams to define custom process compliance rules in RDF/Turtle; scaffold implementations.
2. **Regulatory Compliance Mappings** — IEC 61508, ISO 26262, DO-178C, FDA 21 CFR Part 11.
3. **Certified Rust Library Registry** — Community-curated list of crates that have undergone certification body audits.
4. **Anti-Pattern Detection Service** — ML model trained on ecosystem evidence archives. Detects:
   - Slow test cycles
   - High publish failure rates
   - Unusual process deviations (security signals)
5. **Community Governance Model** — Process mining committee, ontology review board, certification body accreditation.

### Phase 3 Success Metrics

- 50%+ of crates.io releases include process evidence
- 10+ regulatory compliance mappings published
- 100+ crates in Certified Rust Registry
- Anti-pattern detection catches 5+ security issues per quarter
- 3+ independent certification bodies operating

---

## Consolidated Feature Matrix

**Total Features:** 34  
**Total Effort:** 300–350 person-days (~18–22 weeks FTE, 2-person core team)  
**Calendar Time:** Q3 2026 → Q4 2027+ (18+ months)

| Theme | Count | Phase 1 | Phase 2 | Phase 3 |
|-------|-------|---------|---------|---------|
| Publication & Distribution | 10 | 5 | 4 | 1 |
| Process Infrastructure | 10 | 6 | 3 | 1 |
| Safety & Certification | 14 | 4 | 3 | 7 |
| AI/Code Quality | 6 | 3 | 2 | 1 |
| Ecosystem & Tooling | 14 | 4 | 5 | 5 |
| **Total** | **34** | **22** | **17** | **15** |

---

## Critical Dependencies & Sequencing

### Hard Dependencies (Must Complete Before)

1. **XES Event Stream (1.1)** → All phase 1 process-related features
2. **Oracle Public Key (1.3)** → Receipt validation, certification body integration
3. **Cargo.toml [evidence] (1.5)** → Ecosystem adoption, cargo tree/audit integration
4. **Distributed Oracle (3.1)** → Multi-provider ecosystem (Phase 3)
5. **Custom Ontology Support (3.6)** → Extended ggen (Phase 3)

### Parallel Work (Can Start Independently)

- JSONL Companion (1.2) — parallel with XES (1.1)
- Code Provenance (2.4) — parallel with XES (1.1)
- First Certification Body (1.6) — parallel with foundational evidence work (1.1, 1.3, 1.4)
- Process Mining Dashboards (3.2) — begins after Phase 1 XES validation

### Long-Lead Items (Start Early)

- **Cargo.toml [evidence] RFC** — Propose to Cargo team immediately (Week 1)
- **First Certification Body Outreach** — Ferrous Systems, TrustInSoft, others (Week 2–4)
- **Ontology Registry Design** — Parallel to Phase 1 execution (Week 1–8)

---

## Resource Plan

### Phase 1 Team

- **Core (Rust, architecture):** 1.5 FTE
- **Certification/Compliance:** 1.0 FTE
- **Ecosystem/RFC:** 0.5 FTE
- **Total:** 3.0 FTE (can be 2-person team with split focus)

### Phase 2 Team

- **Core (Rust, distributed systems):** 1.5 FTE
- **Analytics / Dashboarding:** 1.0 FTE
- **Ecosystem / Registry:** 0.5 FTE
- **Total:** 3.0 FTE

### Phase 3 Team

- **Core (ggen, ontology):** 1.0 FTE
- **ML / Anti-pattern Detection:** 1.0 FTE
- **Compliance/Regulatory:** 1.0 FTE
- **Community Governance:** 0.5 FTE
- **Total:** 3.5 FTE (governance-heavy)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Cargo RFC rejected or delayed | Medium | High | Propose alternative (package-level, cargo config) |
| Certification body integration slower than expected | Medium | Medium | Parallel approach: first internal cert, then external |
| Process mining tools incompatibility | Low | Medium | Validate XES format against ProM/Disco early |
| Adoption slower than 20% target (Phase 2) | Medium | High | Aggressive marketing, incentive structure (e.g., Registry badge) |
| Multi-oracle consensus correctness | Low | Critical | Formal verification, extensive testing |
| Regulatory mapping errors (Phase 3) | Low | Critical | External audit of compliance claims |

---

## Success Metrics (Per Phase)

### Phase 1

- ✅ XES validation: 100% of cargo-cicd events emit valid XES
- ✅ Certification: First certification body onboarded
- ✅ RFC acceptance: Cargo.toml [evidence] specification published
- ✅ Test coverage: 50+ new tests, 90%+ code coverage
- ✅ Zero P0 bugs in evidence emission

### Phase 2

- ✅ Adoption: 20%+ of crates.io have [evidence] section
- ✅ Oracle integration: 3+ independent oracles operational
- ✅ Analytics: Dashboard processes 1000+ traces/day
- ✅ Registry: 5+ ontologies published, 30+ teams adopting
- ✅ Performance: Dashboard queries < 500ms p95

### Phase 3

- ✅ Community: 100+ crates in Certified Rust Registry
- ✅ Regulatory: 5+ compliance mappings (IEC, ISO, DO, FDA)
- ✅ Adoption: 50%+ of crates.io releases
- ✅ Ecosystem: 10+ independent certification bodies

---

## Next Immediate Actions (Week 1 of Phase 1)

1. **Propose Cargo.toml [evidence] RFC** to rust-lang/cargo working group
2. **Reach out to first certification body** (Ferrous Systems, TrustInSoft)
3. **Formalize XES specification** (vendor-neutral, publish externally)
4. **Prototype extended ggen pipeline** for custom capabilities
5. **Design distributed oracle consensus protocol** (M-of-N threshold scheme)
6. **Establish process mining baseline** (validate Disco/ProM XES compatibility)

---

## Document History

| Date | Version | Status | Author |
|------|---------|--------|--------|
| 2026-06-17 | 1.0 | Phase 1 Planning | cargo-cicd core team |
| — | 1.1 (draft) | Planning | TBD |

---

## Appendix: Feature Quick Reference

### By Theme

**Publication & Distribution** (Features 1–10)
- Process Evidence as Cargo.toml Metadata (1.1)
- XES Event Stream Generation (1.2)
- Oracle Public Key Embedding (1.3)
- Receipt Hash Validation (1.4)
- Cargo.toml [evidence] Section Schema (1.5)
- cargo tree VERIFIED Badge (1.6)
- cargo audit Receipt Verification (1.7)
- Evidence Archive URLs & Metadata (1.8)
- Trustworthiness Scoring (1.9)
- Unverified Package Flagging (1.10)

**Process Infrastructure** (Features 2–11)
- XES Event Stream Generation (2.1)
- JSONL Companion Format (2.2)
- Process Mining Compatibility (2.3)
- Evidence Archive URLs (2.4)
- Oracle Public Key Embedding (2.5)
- Receipt Hash Validation (2.6)
- Process State Versioning (2.7)
- Evidence Collection & Storage (2.8)
- cicd.toml as State Carrier (2.9)
- Audit Trail for State Changes (2.10)

**Safety & Certification** (Features 3–16)
- Certification Body Integration (3.1)
- IEC 61508 / ISO 26262 Compliance Mapping (3.2)
- Certified Rust Library Registry (3.3)
- Process Conformance Evidence (3.4)
- Safety-Critical Crate Metadata (3.5)
- Audit Trail (3.6)
- Receipt Doctor (3.7)
- Compliance Report Export (3.8)
- Distributed Trust (Multi-Oracle) (3.9)
- Safety Policies (3.10)
- Dependency Supply Chain Scanning (3.11)
- Reproducible Builds (3.12)
- Process Model Definition (3.13)
- AI-Generated Code Flagging (3.14)
- DO-178C Compliance (3.15)
- FDA 21 CFR Part 11 (3.16)

**AI/Code Quality** (Features 4–9)
- Anti-LLM Code Detection (4.1)
- Code Provenance Tracking (4.2)
- Human Review Gates for AI Code (4.3)
- LLM Linting Rules (4.4)
- Community Tooling / Registry Visibility (4.5)
- Evidence Fields for Code Provenance (4.6)

**Ecosystem & Tooling** (Features 5–18)
- Ontology-Driven CLI Capability Generation (5.1)
- Extended ggen Pipeline (5.2)
- Capability Ontology Standard & Registry (5.3)
- Pluggable Process Models (5.4)
- Distributed Oracle Adjudication (5.5)
- Process Mining Dashboards (5.6)
- Analytics Services (5.7)
- Bottleneck Detection (5.8)
- Vendor-Neutral Process Export (5.9)
- Custom Process Model Languages (5.10)
- Distributed Trust Ecosystem (5.11)
- Anti-Pattern Detection (5.12)
- Process Model Versioning (5.13)
- Community Governance (5.14)
- Ecosystem Health Metrics (5.15)
- Regulatory Compliance Mappings (5.16)
- Certified Rust Registry (5.17)
- Process Mining Committee (5.18)

---

**Prepared by:** Claude Code (Phase 1 planning synthesis)  
**Last Updated:** 2026-06-17  
**Next Review:** End of Phase 1 (Q4 2026)
