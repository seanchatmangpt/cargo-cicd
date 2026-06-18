# Vision 2030: Strategic Initiative Index

**Prepared:** 2026-06-17  
**Status:** Phase 1 Planning — Ready for Execution  
**Scope:** Evolving cargo-cicd from workspace health tool into process-evidence ecosystem platform

---

## Quick Start

**New to Vision 2030?** Start here:

1. **Read the Thesis** (`docs/thesis.md`) — 331-line manifesto explaining the "why"
   - Diagnosis: CI/CD theater, fragmented state
   - Solution: Level 5 engine, evidence-based verdicts
   - Vision: Ecosystem-scale process mining, regulatory compliance, distributed trust

2. **Review the Roadmap** (`docs/ROADMAP-2030.md`) — 3-phase strategic plan spanning 18+ months
   - Phase 1 (Q3 2026): Process evidence foundation + certification
   - Phase 2 (Q4 2026–Q1 2027): Ecosystem adoption + dashboards
   - Phase 3 (Q2 2027+): Regulatory compliance + community governance
   - All 34 features mapped to phases, efforts, and dependencies

3. **Phase 1 Team Kickoff** (`docs/PHASE-1-PLAN.md`) — Week-by-week sprint plan
   - 16-week execution plan (12–16 weeks = Q3 2026)
   - Resource allocation: 3 FTE (core, cert, ecosystem)
   - Weekly deliverables, milestones, go-live criteria
   - Risk mitigation strategies

---

## Documents by Purpose

### Strategic & Vision

| Document | Purpose | Audience | Length |
|----------|---------|----------|--------|
| `docs/thesis.md` | Manifesto: "Why every Rust project should use process evidence" | Decision-makers, architects | 331 lines |
| `docs/ROADMAP-2030.md` | Three-phase roadmap (18+ months), 34 features, 300–350 person-days | Project managers, engineers | 384 lines |
| `docs/prd-vision-2030.md` | Comprehensive PRD: 34 features × 8 acceptance criteria each | Product managers, engineers | 943 lines |
| `docs/vision-2030-prd.md` | Alternate PRD structure: process infrastructure, safety, ecosystem | Product teams | 691 lines |

**Use `docs/ROADMAP-2030.md` as the source of truth for prioritization and sequencing.**

### Execution

| Document | Purpose | Audience | Length |
|----------|---------|----------|--------|
| `docs/PHASE-1-PLAN.md` | Week-by-week execution plan (Weeks 1–16, Q3 2026) | Core team, engineers | 397 lines |
| `docs/PHASE-1-PLAN.md` (Appendix) | Risk mitigation, resource allocation, dependencies | Project lead, managers | —— |

### Technical (To Be Created in Phase 1)

| Document | Purpose | Owner | Milestone |
|----------|---------|-------|-----------|
| `docs/XES-2.0-SPECIFICATION.md` | Vendor-neutral XES format specification | Core | Week 1 (draft), Week 9 (public) |
| `docs/CODE-PROVENANCE.md` | Human/AI/LLM classification; limitations & best practices | Core | Week 9 |
| `docs/CERT-BODY-INTEGRATION.md` | How to become a certification provider | Cert | Week 9 |
| `docs/CUSTOM-ONTOLOGY-GUIDE.md` | Step-by-step guide for teams to write RDF ontologies | Ecosystem | Week 9 |
| `docs/adr/phase1-*.md` | Architecture Decision Records (5–10 ADRs) | Core | Week 9 |

---

## Feature Catalog

### By Theme

**Theme 1: Publication & Distribution (10 features)**
- Cargo.toml [evidence] section (manifest-level evidence registration)
- cargo tree VERIFIED badge (dependency verification status)
- cargo audit receipt verification
- Evidence archive URLs & metadata
- Trustworthiness scoring
- Unverified package flagging
- [5 more features in PRD]

**Theme 2: Process Infrastructure (10 features)**
- XES Event Stream Generation (XES 2.0 compliance)
- JSONL companion format (streaming parsers)
- Oracle public key embedding
- Receipt hash validation
- Process state versioning
- Evidence collection & storage
- [4 more features in PRD]

**Theme 3: Safety & Certification (14 features)**
- Certification body integration
- IEC 61508 / ISO 26262 compliance mapping
- Certified Rust library registry
- Process conformance evidence
- Safety-critical crate metadata
- Distributed trust (multi-oracle consensus)
- DO-178C compliance (avionics)
- [7 more features in PRD]

**Theme 4: AI/Code Quality (6 features)**
- Anti-LLM code detection (gated via anti-llm-cheat feature)
- Code provenance tracking
- Human review gates for AI code
- LLM linting rules
- [2 more features in PRD]

**Theme 5: Ecosystem & Tooling (14 features)**
- Ontology-driven CLI capability generation (ggen extensibility)
- Custom ontology support
- Pluggable process models
- Distributed oracle adjudication (M-of-N consensus)
- Process mining dashboards
- Ecosystem health analytics
- Bottleneck detection
- [7 more features in PRD]

**Total:** 34 features across 5 themes  
**See:** `docs/prd-vision-2030.md` or `docs/vision-2030-prd.md` for full feature specifications

### By Priority

**P0 (Blocking, must complete before phase transition):**
- Phase 1: 9 features (XES, Oracle, Cargo RFC, Cert Body, Code Provenance, IEC 61508, Process Conformance, Safety-Critical Registry)
- Phase 2: 8 features (Distributed Oracle, Process Mining, Analytics, Bottleneck Detection, Pluggable Models, Custom Ontology, Cargo Tree Badge, Cargo Audit Integration)
- Phase 3: 6 features (Extended ggen, DO-178C, Certified Registry, Process Model Versioning, Anti-Pattern Detection, Distributed Ecosystem)

**P1 (Recommended, within phase):**
- Phase 1: 4 features (Audit Trail, Archive URLs, Receipt Doctor, Code Provenance)
- Phase 2: 4 features (Reproducible Builds, Supply Chain Scanning, Ontology Registry, Export)

**P2 (Nice-to-have, deferred):**
- Various Phase 3 features, community governance items

### By Phase

| Phase | Start | Duration | Features | Effort |
|-------|-------|----------|----------|--------|
| **Phase 1** | 2026-Q3 | 12–16w | 22 (9 P0 + 4 P1 + 9 misc) | 80–100 pd |
| **Phase 2** | 2026-Q4 | 12–16w | 17 (8 P0 + 4 P1 + 5 misc) | 90–120 pd |
| **Phase 3** | 2027-Q2 | Ongoing | 15 (6 P0 + ongoing) | 120+ pd |

**Total Effort:** 300–350 person-days over 18+ months  
**Team Size:** 3 FTE per phase (distributed: core, cert, ecosystem)

---

## Key Concepts

### Process Evidence as First-Class Artifact

Every cargo-cicd command emits structured evidence (ProcessEvent → XES → receipt):

```
cargo cicd <noun> <verb>
  ↓
ProcessEvent::started("noun:verb")
  ↓
[work]
  ↓
ProcessEvent::completed("noun:verb", verdict)
  ↓
Serialize to target/cargo-cicd/evidence/evt-*.xes (XES 2.0)
  ↓
Serialize to target/cargo-cicd/evidence/evt-*.jsonl (JSONL)
  ↓
Call wasm4pm oracle: wpm audit <evt-*.xes>
  ↓
Oracle returns Accept/Refuse/Blocked
  ↓
Cert body issues receipt (if approved)
  ↓
Crate publishes evidence archive + receipt URL in Cargo.toml [evidence]
```

**Vision 2030 enforces this pipeline at ecosystem scale.**

### Certification Infrastructure

Certification bodies (Ferrous Systems, TrustInSoft, Trail of Bits, others) validate process evidence and issue receipts:

- **Receipt:** Cryptographically signed artifact claiming "evidence is valid per IEC 61508 / ISO 26262 / DO-178C"
- **Receipt Hash:** SHA-256 commitment stored in Cargo.toml [evidence]
- **Multi-Oracle:** Phase 2 introduces M-of-N consensus (e.g., 2-of-3 oracles must agree)
- **Public Key:** Oracle's public key embedded in evidence or fetched from registry

### Ecosystem Health Metrics

Phase 2 introduces analytics service that tracks:
- % of crates.io with process evidence
- % with valid receipts
- Bottleneck stages (slowest test, longest review, etc.)
- Anti-patterns (high publish failure rate, unusual process deviations)
- Supply chain risk (unverified dependencies, pinning violations)

### Regulatory Compliance

Phase 3 maps process evidence to regulatory frameworks:
- **IEC 61508** (functional safety, industrial automation) — Phase 1 mapping
- **ISO 26262** (automotive functional safety) — Phase 1 mapping
- **DO-178C** (avionics certification) — Phase 3
- **FDA 21 CFR Part 11** (medical device software) — Phase 3

**Vision 2030 allows safety-critical projects to prove compliance via evidence archives.**

---

## Critical Path & Dependencies

```
XES 2.0 Spec (Week 1)
  ↓
Oracle Public Key (Week 3)
  ↓
Cargo.toml [evidence] RFC (Week 4)
  ↓
First Cert Body Receipt (Week 5)
  ↓
Code Provenance Tracking (Week 4–5)
  ↓
Phase 1 Release (Week 12)
  ↓
Distributed Oracle (Phase 2, Week 1)
  ↓
Process Mining Dashboards (Phase 2, Week 4)
  ↓
Ecosystem Analytics (Phase 2, Week 6)
  ↓
Phase 2 Release (Week 16)
  ↓
Extended ggen (Phase 3, Week 1)
  ↓
Regulatory Mappings (Phase 3, ongoing)
  ↓
Phase 3 Release (2027 mid-year)
```

**Parallel streams** (don't block critical path):
- Dashboard prototyping (Weeks 2–6, Phase 1)
- Ontology registry design (Weeks 1–8, Phase 1)
- Cargo RFC review (async feedback loop, Weeks 1–10, Phase 1)

---

## Team Structure

### Core Engineering (1.5 FTE per phase)
Owns: XES generation, oracle integration, process infrastructure, distributed consensus, evidence emission, ggen extensions.

**Phase 1 Focus:**
- XES 2.0 serialization + validation
- Oracle public key infrastructure
- Receipt hash validation
- Code provenance tracking
- Comprehensive test suite

**Phase 2 Focus:**
- Distributed oracle M-of-N consensus
- Process mining data pipeline
- Custom ontology support
- Pluggable process models

**Phase 3 Focus:**
- Extended ggen pipeline
- ML-based anti-pattern detection
- Regulatory compliance verification
- Community governance infrastructure

### Certification & Compliance (1.0 FTE per phase)
Owns: Cert body partnerships, regulatory mappings, safety-critical metadata, audit trails, compliance verification.

**Phase 1 Focus:**
- Cert body outreach & onboarding (Week 1–3)
- First receipt issuance (Week 5)
- IEC 61508 / ISO 26262 mappings (Weeks 6–9)
- Safety-critical registry initialization (Week 5)

**Phase 2 Focus:**
- Multi-oracle consensus validation
- Extended compliance mappings
- Certified Rust registry curation
- Audit trail hardening

**Phase 3 Focus:**
- DO-178C integration
- FDA 21 CFR Part 11 mapping
- Regulatory committee coordination
- Accreditation standards

### Ecosystem & Product (0.5 FTE per phase)
Owns: Cargo RFC, ecosystem adoption, community engagement, registry, documentation, roadmap communication.

**Phase 1 Focus:**
- Cargo.toml [evidence] RFC (Weeks 1–10)
- Ontology registry MVP (Weeks 8–9)
- Public documentation (Week 9)
- Community outreach (Week 10)

**Phase 2 Focus:**
- Cargo tree / cargo audit integration (Weeks 1–4)
- Ecosystem analytics service (Weeks 6–10)
- Registry expansion (10+ ontologies)
- Process model export formats

**Phase 3 Focus:**
- Community governance
- Registry curation
- Ecosystem partnership programs
- International adoption

---

## Success Metrics

### Phase 1 (Q3 2026)
- ✅ XES validation: 100% of cargo-cicd events emit valid XES
- ✅ Cert integration: First certification body onboarded
- ✅ Cargo RFC: [evidence] section specification published
- ✅ Test coverage: 50+ new tests, 90%+ code coverage
- ✅ Zero P0 bugs in evidence emission

### Phase 2 (Q4 2026 – Q1 2027)
- ✅ Adoption: 20%+ of crates.io have [evidence] metadata
- ✅ Oracles: 3+ independent certification bodies operational
- ✅ Analytics: Dashboard processes 1000+ traces/day
- ✅ Registry: 5+ published ontologies, 30+ teams adopting
- ✅ Performance: Dashboard queries < 500ms p95

### Phase 3 (Q2 2027+)
- ✅ Community: 100+ crates in Certified Rust Registry
- ✅ Regulatory: 5+ compliance mappings (IEC, ISO, DO, FDA)
- ✅ Adoption: 50%+ of crates.io releases
- ✅ Ecosystem: 10+ independent certification bodies

---

## Getting Started (Next Week)

### For Architects & Decision-Makers

1. Read `docs/thesis.md` (20 min) — Understand the strategic rationale
2. Skim `docs/ROADMAP-2030.md` (30 min) — Review 3-phase plan, effort estimates, dependencies
3. Attend kick-off meeting (1h) — Approve Phase 1 plan, commit resources

### For Engineering Teams

1. Read `docs/PHASE-1-PLAN.md` (45 min) — Understand week-by-week sprint plan
2. Review `docs/prd-vision-2030.md` or `docs/vision-2030-prd.md` (1h) — Study Phase 1 P0 features in detail
3. Set up local dev environment, fork cargo-cicd repo
4. Attend tech design sync (2h) — Finalize XES spec, Oracle key strategy, Cargo RFC approach

### For Cert Bodies / Regulators

1. Read `docs/thesis.md` sections IV–V (15 min) — Understand ecosystem vision
2. Review relevant compliance mappings:
   - IEC 61508: `docs/ROADMAP-2030.md` → Phase 1 deliverables (TBD)
   - ISO 26262: `docs/ROADMAP-2030.md` → Phase 1 deliverables (TBD)
   - DO-178C: `docs/ROADMAP-2030.md` → Phase 3 features (TBD)
3. Contact cert team (Week 1 of Phase 1) — Discuss partnership SLA and technical requirements

### For Community / Adopters

1. Read `docs/thesis.md` (20 min) — Understand the vision
2. Watch for Phase 1 release (expected Q3 2026, Week 12)
3. Try XES generation in local workspace: `cargo cicd status show` → check `target/cargo-cicd/evidence/`
4. Consider publishing evidence archive: Cargo.toml `[evidence]` section (Phase 1 release)

---

## FAQ

### When does Phase 1 start?
**Expected:** 2026-06-20 (estimated) — Pending team approval and resource commitment.

### Can I use Vision 2030 features now?
**Partial:** cargo-cicd v26.6.2 already emits ProcessEvent / XES. Anti-LLM code detection available behind `anti-llm-cheat` feature flag.  
**Full:** Wait for Phase 1 release (Q3 2026, Week 12) when [evidence] section and Cargo RFC are finalized.

### What happens to existing cargo-cicd users?
**Backward compatible:** Phase 1 changes are additive. Existing `cargo cicd status`, `cargo cicd target`, etc. work unchanged.  
**New capability:** Teams opt-in to [evidence] metadata publication. Old crates remain usable but flagged as UNVERIFIED.

### How does this relate to supply chain security?
Vision 2030 enables **provable supply chain compliance** via process evidence:
- Dependency pinning verification
- License compliance audits (stored in evidence archives)
- Build reproducibility proofs
- Anti-pattern detection (unusual behavior = security signal)

See `docs/ROADMAP-2030.md` → Phase 2 → Feature 4.2 (Supply Chain Defense Scanning).

### What's the cost to adopt?
**For maintainers:** Minimal. cargo-cicd is free, open-source. Optional Cargo.toml edits.  
**For safety-critical projects:** Certification body fees (TBD per body). Estimated $10k–$100k per certification cycle depending on scope.

### Can I use a different oracle (not wasm4pm)?
**Phase 1:** wasm4pm only (reference implementation).  
**Phase 2:** Multiple oracles supported (M-of-N consensus). Pluggable oracle interface (TBD).  
**Phase 3:** Community oracles welcome (accreditation process).

### How do I participate?
1. **Engineering:** Join core team or contribute features (GitHub issues / PRs)
2. **Cert body:** Contact us to discuss partnership (Ferrous Systems model)
3. **Ecosystem:** Publish an ontology in the registry (Phase 2)
4. **Feedback:** URLO threads, GitHub discussions, async design docs

---

## Related Documents

- **CLAUDE.md** — Architecture & design patterns for cargo-cicd core
- **README.md** — User guide for cargo-cicd v26.6.2
- **Cargo.toml** — Dependency declarations, feature flags
- **CHANGELOG.md** — Release notes and version history
- **tests/** — Integration test suites (invariants, CLI, evidence gates)

---

## Contact & Governance

**Vision 2030 Lead:** TBD (assign Week 1)  
**Cargo RFC Champion:** TBD (assign Week 1)  
**Cert Body Liaison:** TBD (assign Week 1)  
**Community Manager:** TBD (assign Phase 2)

**Communication Channels:**
- GitHub Issues: `vision-2030-*` labels
- Design Docs: Google Docs (linked in PHASE-1-PLAN.md)
- Weekly Syncs: TBD (calendar invite Week 1)
- Async Updates: Slack `#vision-2030` (TBD)

---

## License & Attribution

All Vision 2030 documentation and code are part of cargo-cicd, licensed under MIT + Apache 2.0.

**Contributors:**
- cargo-cicd core team (engineering, design, architecture)
- Claude Code agents (research, synthesis, initial PRDs)
- Community feedback (early adopters, partners)

---

## Version History

| Date | Version | Status | Notes |
|------|---------|--------|-------|
| 2026-06-17 | 1.0 (draft) | Planning | Initial documentation complete. Awaiting team review. |
| TBD | 1.0 (final) | Active | Ready for Phase 1 execution kickoff. |
| — | 1.1+ | Future | Iterative updates during Phase 1 (weekly). |

---

**Last Updated:** 2026-06-17  
**Next Review:** End of Phase 1 kickoff meeting (Week 1)  
**Status:** Ready for Review → Execution

---

## Quick Navigation

```
Start Here:
├── docs/thesis.md                    ← Why (Vision & rationale)
├── docs/ROADMAP-2030.md              ← What & When (3-phase plan)
├── docs/PHASE-1-PLAN.md              ← How (execution roadmap)
│
For Deep Dives:
├── docs/prd-vision-2030.md           ← Full PRD (34 features)
├── docs/vision-2030-prd.md           ← Alternate PRD structure
│
Technical (Phase 1 deliverables, TBD):
├── docs/XES-2.0-SPECIFICATION.md     ← Vendor-neutral XES spec
├── docs/CODE-PROVENANCE.md           ← Human/AI/LLM classification
├── docs/CERT-BODY-INTEGRATION.md     ← How to partner
├── docs/CUSTOM-ONTOLOGY-GUIDE.md     ← RDF ontology tutorial
├── docs/adr/phase1-*.md              ← Architecture decisions
│
Status & Governance:
└── CLAUDE.md                         ← Architecture & patterns
```

