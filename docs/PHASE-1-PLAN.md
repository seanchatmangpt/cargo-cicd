# Phase 1 Implementation Plan: Foundation (Q3 2026)

**Duration:** 12–16 weeks  
**Start Date:** 2026-06-20 (estimated)  
**End Date:** 2026-09-30 (estimated)  
**Team:** 3 FTE (distributed: core, certification, ecosystem)  

---

## Overview

Phase 1 establishes the process evidence foundation and initial certification infrastructure. The critical path is:

```
XES/JSONL → Oracle Keys → Cargo RFC → Cert Body → Code Provenance
```

Work on non-blocking features (dashboard prototypes, registry design) happens in parallel to keep the team productive while waiting for external responses (Cargo RFC review, cert body onboarding).

---

## Week-by-Week Breakdown

### Week 1: Foundations & RFC Outreach (Jun 20–26)

**Goal:** Begin XES implementation. Initiate external engagement (Cargo RFC, cert bodies).

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **XES 2.0 Specification** — Document ProcessEvent → XES mapping | Core | 1d | `docs/XES-2.0-SPEC.md` (reference) |
| **XES Generation (skeleton)** — Struct and serialization boilerplate | Core | 1.5d | `src/evidence/xes_emitter.rs` (compiling but minimal) |
| **Cargo.toml [evidence] RFC Draft** — Prepare RFC, identify Cargo WG contacts | Ecosystem | 0.5d | Draft RFC posted to rust-lang/cargo issue tracker |
| **Cert Body Outreach** — Email 3–5 candidates (Ferrous, TrustInSoft, Trail of Bits, etc.) | Cert | 0.5d | Outreach emails sent; initial contact established |
| **Ontology Registry UI Mockup** — Static HTML/Figma design | Ecosystem | 1d | Design doc + mockup images |

**Effort:** 5 person-days  
**Dependencies:** None  
**Blockers:** None expected  
**Risks:** Cargo WG may deprioritize RFC; cert bodies slow to respond

---

### Week 2: XES Core + Parallel Design Work (Jun 27–Jul 3)

**Goal:** Complete XES serialization. Start dashboard and registry prototypes.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **XES Serialization (complete)** — Full ProcessEvent → XES with all attributes | Core | 2d | `src/evidence/xes_emitter.rs` (100% feature-complete) |
| **JSONL Companion Format** — Parallel format (JSON lines for streaming parsers) | Core | 1d | `src/evidence/jsonl_emitter.rs` |
| **XES Validation Test Suite** — Unit tests + integration tests parsing generated XES | Core | 1.5d | `tests/xes_validation.rs` (20+ test cases) |
| **Process Mining Dashboard Prototype** — Node.js/React mockup consuming XES | Analytics | 1.5d | GitHub Pages demo site; no backend |
| **Ontology Registry Schema Design** — RDF schema for ontology metadata | Ecosystem | 1d | `docs/ontology-registry-schema.ttl` |

**Effort:** 7 person-days  
**Dependencies:** Week 1 foundations  
**Blockers:** None  
**Risks:** Dashboard prototype timeline (move to Week 3 if needed)

---

### Week 3: Oracle Keys + Cert Body Kickoff (Jul 4–10)

**Goal:** Implement oracle public key embedding. Confirm cert body partnership.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Oracle Public Key Infrastructure** — Key derivation, storage, rotation policy | Core | 2d | `src/evidence/oracle_keys.rs` + key management docs |
| **Receipt Hash Validation** — SHA-256 hashing and verification logic | Core | 1.5d | `src/evidence/receipt_validation.rs` |
| **Cert Body SLA & Technical Onboarding** — Sync with chosen cert body; document requirements | Cert | 2d | Signed SLA + technical integration plan |
| **Test Suite: Oracle Key Rotation** — Verify key lifecycle in CI/CD | Core | 1d | `tests/oracle_key_lifecycle.rs` |
| **Cargo RFC Community Review** — Respond to initial feedback (if any) | Ecosystem | 1d | RFC iteration (GitHub comments) |

**Effort:** 7.5 person-days  
**Dependencies:** Weeks 1–2  
**Blockers:** Cert body selection (should be done by end of Week 2)  
**Risks:** Cert body integration scope creep

---

### Week 4: Cargo.toml [evidence] + Code Provenance (Jul 11–17)

**Goal:** Finalize Cargo.toml [evidence] spec. Begin code provenance tracking.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Cargo.toml [evidence] Field Specification** — Final RFC with examples | Ecosystem | 1.5d | RFC published to rust-lang/cargo; awaiting review |
| **Code Provenance Tracking Framework** — Human/AI/LLM classification in ProcessEvent | Core | 2d | `src/evidence/code_provenance.rs` |
| **Code Provenance Tests** — Unit tests for classification logic | Core | 1d | `tests/code_provenance_detection.rs` |
| **Cargo Metadata Integration (prototype)** — Mock `cargo metadata` output including [evidence] | Ecosystem | 1.5d | Prototype in separate repo; ready for Cargo team feedback |
| **First Cert Body Technical Kickoff** — Detailed technical requirements & API design | Cert | 1.5d | Technical integration plan + API spec |

**Effort:** 8 person-days  
**Dependencies:** Weeks 1–3  
**Blockers:** Cargo WG feedback loop (not blocking; async)  
**Risks:** Code provenance detection complexity (AI detection is hard; scope it carefully)

---

### Week 5: Safety-Critical Registry + First Receipts (Jul 18–24)

**Goal:** Initialize safety-critical crate registry. First cert body issues test receipt.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Safety-Critical Crate Registry Schema** — Metadata structure, versioning, search | Ecosystem | 1.5d | `docs/safety-critical-registry.md` |
| **IEC 61508 Compliance Mapping (initial)** — Level 1–3 process requirements | Cert | 2d | `docs/IEC-61508-MAPPING.md` |
| **First Cert Body Receipt Issuance** — Cert body issues test receipt for cargo-cicd v26.6.2 | Cert | 1.5d | Signed receipt; validated by wpm oracle |
| **Receipt Validation Integration (prod)** — Integrate with cargo-cicd status/publish flow | Core | 1.5d | Receipt lookup, validation, display |
| **Dashboard Data Ingestion** — Accept XES/JSONL from cargo-cicd; display traces | Analytics | 1.5d | Dashboard backend (Node.js Express; no auth) |

**Effort:** 8.5 person-days  
**Dependencies:** Weeks 1–4  
**Blockers:** Cert body technical readiness  
**Risks:** Receipt format compatibility (ensure wpm can parse)

---

### Week 6: ISO 26262 + Public API (Jul 25–31)

**Goal:** Extend compliance mappings. Publish first public API for evidence retrieval.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **ISO 26262 Compliance Mapping** — Functional safety requirements (automotive) | Cert | 2d | `docs/ISO-26262-MAPPING.md` |
| **Public Evidence API (read-only)** — REST API to fetch evidence archives by crate+version | Core | 2d | Prototype API (GitHub Pages + S3 backend) |
| **API Authentication & Rate Limiting** — Prevent abuse; track API consumers | Ecosystem | 1d | Auth token schema; rate limit policy |
| **Evidence Archive Upload Mechanism** — How crates publish evidence to archive service | Core | 1.5d | Upload endpoint + documentation |
| **Dashboard Integration with Public API** — Link dashboard traces to crate evidence | Analytics | 1d | Frontend: crate name → evidence archive |

**Effort:** 7.5 person-days  
**Dependencies:** Weeks 1–5  
**Blockers:** Archive hosting (use S3 or GitHub Releases as interim)  
**Risks:** API scale (start simple; iterate)

---

### Week 7: Test Harness + Ecosystem Validation (Aug 1–7)

**Goal:** Comprehensive test suite. Validate integrations with external tools.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Phase 1 Test Suite (comprehensive)** — 50+ tests covering all P0 features | Core | 3d | `tests/phase1_*.rs` modules (90%+ coverage) |
| **XES Compatibility Testing** — Parse generated XES with ProM/Disco (via vendor APIs) | Core | 1.5d | Test report: ProM ✅, Disco ✅ |
| **Cargo RFC Final Submission** — Submit finalized RFC for Cargo team vote | Ecosystem | 0.5d | RFC merged or in final review |
| **Cert Body Second Receipt** — Issue receipt for a community crate (not cargo-cicd) | Cert | 1.5d | Receipt + validation chain |
| **Public Roadmap & Blog Post** — Announce Vision 2030 to community | Ecosystem | 1d | Blog post + roadmap link on cargo-cicd.rs |

**Effort:** 7.5 person-days  
**Dependencies:** Weeks 1–6  
**Blockers:** None  
**Risks:** Test coverage gaps (iterate in Weeks 8–9)

---

### Week 8: Ontology Registry MVP + Integration Hardening (Aug 8–14)

**Goal:** Launch minimal ontology registry. Harden integrations.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Ontology Registry MVP (read-only)** — Static registry of 3–5 ontologies at `registry.cargo-cicd.rs` | Ecosystem | 2d | Live domain; GitHub Pages-based registry |
| **ggen Integration Documentation** — How to use custom ggen.toml to load ontologies | Ecosystem | 1.5d | Tutorial + examples (`docs/custom-ggen.md`) |
| **Code Provenance Hardening** — Add LLM detection (OpenAI API / local heuristics) | Core | 1.5d | `src/evidence/llm_detection.rs` |
| **Receipt Doctor Enhancement** — Extend to validate code provenance metadata | Core | 1d | `src/integrations/receipt_doctor_v2.rs` |
| **Dashboard Stability Pass** — Performance tuning; error handling | Analytics | 1.5d | Dashboard p95 latency < 500ms |

**Effort:** 8 person-days  
**Dependencies:** Weeks 1–7  
**Blockers:** None  
**Risks:** LLM detection accuracy (caveat emptor in docs)

---

### Week 9: Documentation & Knowledge Transfer (Aug 15–21)

**Goal:** Public documentation. Internal knowledge transfer.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **XES 2.0 Public Specification** — Publish vendor-neutral XES spec (candidate for ISO/IEC) | Core | 1.5d | `docs/XES-2.0-SPECIFICATION.md` (public, with examples) |
| **Code Provenance Documentation** — Limitations, false positive rates, best practices | Core | 1d | `docs/CODE-PROVENANCE.md` |
| **Cert Body Integration Guide** — How to become a certification provider | Cert | 1.5d | `docs/CERT-BODY-INTEGRATION.md` |
| **Ontology Development Tutorial** — Step-by-step guide for teams to write RDF ontologies | Ecosystem | 1.5d | `docs/CUSTOM-ONTOLOGY-GUIDE.md` |
| **Architecture Decision Records (ADRs)** — Record key Phase 1 decisions (XES format, key rotation, etc.) | Core | 1d | `docs/adr/phase1-*.md` (5–10 ADRs) |

**Effort:** 6.5 person-days  
**Dependencies:** Weeks 1–8  
**Blockers:** None  
**Risks:** Documentation quality (allocate time for review/iteration)

---

### Week 10: Cargo Ecosystem Coordination (Aug 22–28)

**Goal:** Align with Cargo/Rust ecosystem. Prepare Phase 2 scope.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Cargo Team Sync** — Present [evidence] section feedback; iterate on RFC | Ecosystem | 2d | RFC approved or major revision complete |
| **Rustlings / Community Outreach** — Announce to Rust community (Twitter, Reddit, URLO) | Ecosystem | 1d | Blog post + social media |
| **Safety-Critical Registry Public Beta** — Open for crate submissions (moderated) | Ecosystem | 1.5d | Registry submission form; first 10 crates onboarded |
| **Phase 2 Planning Kickoff** — Scope distributed oracle, process mining dashboards | Core | 1.5d | Phase 2 sprint plan (shared doc) |

**Effort:** 6 person-days  
**Dependencies:** Weeks 1–9  
**Blockers:** Cargo team availability  
**Risks:** Community expectations (manage scope clearly)

---

### Weeks 11–12: Buffer & Polish (Aug 29–Sep 11)

**Goal:** Contingency for delays. Polish Phase 1 release.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Test Suite Final Pass** — Mutation testing, edge cases, stress testing | Core | 2d | Bug fixes; final test report |
| **Documentation Review & Corrections** — QA pass on all public docs | Ecosystem | 1d | Typo fixes; clarity improvements |
| **Cert Body Case Studies** — 2–3 case studies: how cert bodies use cargo-cicd evidence | Cert | 1.5d | Blog posts + documentation |
| **Phase 1 Release Prep** — Release notes, version bump, git tags | Core | 1d | v27.0.0-phase1 tagged and released |
| **Phase 2 Detailed Design** — Complete design for distributed oracle, dashboards | Core | 2d | Technical design docs (ready for Phase 2 Sprint 1) |

**Effort:** 7.5 person-days (can absorb overruns from earlier weeks)  
**Dependencies:** All  
**Blockers:** None  
**Risks:** Phase 2 scope creep (lock design by end of Week 12)

---

### Weeks 13–16: Contingency & Phase 2 Ramp (Sep 12–30)

**Goal:** Execute contingency work. Begin Phase 2 development.

| Task | Owner | Effort | Deliverable |
|------|-------|--------|-------------|
| **Phase 1 Bug Fixes** — Address any production issues from Phase 1 release | Core | 2–3d | Hotfixes; patch releases |
| **Ontology Registry Expansion** — Add 5–10 more ontologies; improve UX | Ecosystem | 2d | Live updates to registry |
| **Distributed Oracle Design Finalization** — M-of-N threshold signatures (cryptographic detail) | Core | 2d | Technical spec ready for implementation |
| **Phase 2 Sprint 1 Development** — Begin work on 3.1 (Distributed Oracle) | Core | 3–4d | First implementation checkpoint |

**Effort:** 9–11 person-days (partial; many team members move to Phase 2)  
**Dependencies:** Weeks 1–12  
**Blockers:** None  
**Risks:** Context switching between phases

---

## Parallel Workstreams (Ongoing)

| Workstream | Owner | Cadence | Deliverable |
|---|---|---|---|
| **Cert Body Partnership** | Cert | Weekly sync | Progress on technical integration; SLA adherence |
| **Cargo RFC Review** | Ecosystem | Bi-weekly | RFC iteration; feedback synthesis |
| **Dashboard UX Research** | Analytics | Bi-weekly | User feedback; dashboard iteration |
| **Community Feedback Loop** | Ecosystem | Weekly | GitHub issues, URLO threads, Discord |
| **Security & Compliance Audit** | Cert | Monthly | Third-party audit of evidence systems |

---

## Milestones & Go-Live Criteria

### Phase 1 Alpha (Week 7)
- [ ] All P0 features have implementations (not necessarily feature-complete)
- [ ] XES validation passes standard tools (ProM, Disco)
- [ ] First cert body receipt issued

### Phase 1 Beta (Week 10)
- [ ] All P0 + P1 features implemented
- [ ] Cargo RFC in final review (or approved)
- [ ] Safety-critical registry live with 10+ crates
- [ ] Public API operational (read-only)

### Phase 1 Release (Week 12)
- [ ] All tests passing (90%+ coverage)
- [ ] No P0 bugs
- [ ] Cargo RFC approved (or integrated as cargo-cicd patch)
- [ ] Cert body partnership documented & live
- [ ] Public documentation complete
- [ ] v27.0.0 released

### Phase 1 → Phase 2 Gate (End of Week 12)
- [ ] Process evidence emission: 100% of cargo-cicd commands emit valid XES ✅
- [ ] Oracle integration: First cert body receipts issued & validated ✅
- [ ] Cargo ecosystem: [evidence] section recognized by cargo tools ✅
- [ ] Community: 0% adoption (expected; Phase 2 drives adoption) ⏳

---

## Resource Allocation

### Core Team (1.5 FTE)

| Week | Sprint | Focus | Person-Days |
|---|---|---|---|
| 1–6 | Foundation | XES, Oracle Keys, Receipt Validation | 8–9 pd/week |
| 7–9 | Integration | Test Harness, Code Provenance | 8–9 pd/week |
| 10–12 | Polish | Docs, Phase 2 Design, Release Prep | 6–7 pd/week |
| 13–16 | Phase 2 Ramp | Distributed Oracle Design, Phase 2 Sprint 1 | 5–6 pd/week (context switch) |

### Certification Team (1.0 FTE)

| Week | Sprint | Focus | Person-Days |
|---|---|---|---|
| 1–3 | Outreach & Onboarding | Cert body selection, SLA negotiation | 4–5 pd/week |
| 4–6 | Technical Integration | Receipt issuance, compliance mapping | 5–6 pd/week |
| 7–9 | Case Studies & Hardening | IEC 61508, ISO 26262 mapping | 4–5 pd/week |
| 10–12 | Release & Documentation | Case studies, release notes | 3–4 pd/week |
| 13–16 | Phase 2 Prep | Regulatory planning for Phase 2/3 | 2–3 pd/week |

### Ecosystem Team (0.5 FTE)

| Week | Sprint | Focus | Person-Days |
|---|---|---|---|
| 1–2 | RFC Drafting & Design | Cargo RFC, ontology registry design | 2–3 pd/week |
| 3–5 | RFC & Outreach | RFC iteration, cert body liaison | 2–3 pd/week |
| 6–9 | Registry MVP & Documentation | Ontology registry launch, docs | 3–4 pd/week |
| 10–12 | Community Engagement | Public outreach, case studies | 2–3 pd/week |
| 13–16 | Phase 2 Preparation | Registry expansion, ecosystem planning | 2–3 pd/week |

---

## Risk Mitigation

### High Risk: Cargo RFC Rejection
**Impact:** Cargo.toml [evidence] integration blocked until RFC accepted or alternate approach found.  
**Mitigation:** Prepare alternate implementation (cargo-config section, separate manifest). Have 2-week fallback plan ready by Week 4.

### High Risk: Cert Body Delayed Integration
**Impact:** No receipts issued; can't validate Phase 1 go-live criteria.  
**Mitigation:** Prepare internal "test cert body" (mock oracle) by Week 3. Get first cert body receipt by Week 5 (hard deadline).

### Medium Risk: XES Compatibility Issues
**Impact:** Process mining tools can't ingest generated XES; Phase 2 dashboards blocked.  
**Mitigation:** Validate with ProM/Disco by Week 2 (not Week 7). Iterate fast on format issues.

### Medium Risk: Code Provenance Detection Too Complex
**Impact:** Scope creep; LLM detection accuracy unsuitable for production.  
**Mitigation:** Scope narrowly in Week 4. Ship MVP with caveats. Phase 2 can improve detection.

### Low Risk: Team Availability / Illness
**Impact:** Milestone slippage.  
**Mitigation:** Weeks 11–12 buffer absorbs 1–2 week delays. Hire temp contractors by Week 8 if needed.

---

## Sign-Off & Approval

| Role | Approval | Date |
|---|---|---|
| **Project Lead** | ✅ Approved | TBD |
| **Cert Lead** | ✅ Approved | TBD |
| **Core Lead** | ✅ Approved | TBD |
| **Ecosystem Lead** | ✅ Approved | TBD |

---

## Document History

| Date | Version | Status | Changes |
|---|---|---|---|
| 2026-06-17 | 1.0 (draft) | Planning | Initial draft; awaiting team review |
| TBD | 1.0 (final) | Active | Ready for execution |
| — | 1.1 | TBD | Mid-phase review & iteration |

---

## Appendix: Dependencies Between Weeks

```
Week 1 (Foundations)
  ├── Week 2 (XES Core)
  │   ├── Week 3 (Oracle Keys)
  │   │   ├── Week 4 (Code Provenance + Cargo RFC)
  │   │   │   ├── Week 5 (Cert Body Receipts)
  │   │   │   │   ├── Week 6 (API + Public Release)
  │   │   │   │   │   ├── Week 7 (Test Harness + Validation)
  │   │   │   │   │   │   ├── Week 8 (Registry MVP + Hardening)
  │   │   │   │   │   │   │   ├── Week 9 (Documentation)
  │   │   │   │   │   │   │   │   ├── Week 10 (Cargo Ecosystem)
  │   │   │   │   │   │   │   │   │   ├── Weeks 11–12 (Buffer & Polish)
  │   │   │   │   │   │   │   │   │   │   └── Weeks 13–16 (Phase 2 Ramp)

Parallel:
  - Ontology Registry Design (Weeks 1–5, then MVP Weeks 8–9)
  - Dashboard Prototype (Weeks 2–6, then integration Weeks 7–8)
  - Cargo RFC Outreach (Weeks 1–10 async feedback loop)
  - Cert Body Partnership (Weeks 1–16 ongoing)
```

---

**Status:** Ready for team review & execution  
**Next Step:** Team planning meeting (Week 1); finalize resource commitments  
**Questions?** See docs/ROADMAP-2030.md or contact project lead

