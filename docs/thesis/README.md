# PhD Thesis: cargo-cicd — A Level 5 Process-Data Engine for Rust Workspace CI/CD Automation

**Author:** Research Team  
**Date:** June 2026  
**Version:** v26.6.2

---

## Abstract

cargo-cicd is a Level 5 process-data engine that treats process conformance as a first-class property of every software release. Rather than relying on self-referential CI/CD pipelines to certify their own correctness, cargo-cicd manufactures a noun-verb command grammar from a formal RDF ontology, emits structured XES process evidence after every command, and delegates all verdict authority to an external wasm4pm oracle. This thesis presents the system's architecture, evidence model, testing strategy, and a Vision 2030 roadmap for future research directions.

---

## Thesis Structure

| Chapter | Title | Words | File |
|---------|-------|-------|------|
| Abstract + Ch. 1–2 | Introduction & Background | 6,033 | [chapter1_introduction.md](./chapter1_introduction.md) |
| Ch. 3 | System Architecture & Design | 5,899 | [chapter3_architecture.md](./chapter3_architecture.md) |
| Ch. 4 | Evidence Emission & Process Mining | 7,691 | [chapter4_evidence.md](./chapter4_evidence.md) |
| Ch. 5 | Verification, Testing & Autonomic Policies | 7,254 | [chapter5_testing.md](./chapter5_testing.md) |
| Ch. 6 + Appendix A | Conclusions & Vision 2030 Roadmap | 6,559 | [chapter6_conclusions_vision2030.md](./chapter6_conclusions_vision2030.md) |
| **Total** | | **~33,436** | |

---

## Chapter Summaries

### Chapter 1–2: Introduction & Background
Covers motivation, problem statement (P1–P4), research objectives (O1–O4), six primary contributions, and comprehensive background in process mining, CI/CD theory, Rust ecosystem, event-driven architectures, and formal verification. Includes 47 academic citations.

### Chapter 3: System Architecture & Design
Traces the full grammar manufacturing pipeline (ontology → ggen → clap-noun-verb → cargo-cicd), the EngineState aggregate root with eleven state dimensions, the four adapter contracts (A1–A4), cicd.toml state carrier lifecycle, feature flag dependency lattice, and seven cross-cutting design principles. Grounded in actual Turtle, SPARQL, and Rust source excerpts.

### Chapter 4: Evidence Emission & Process Mining
Presents the formal ProcessEvent model as a 13-tuple, all seven evidence invariants (E1–E7) with mathematical specifications, the four-stage emission pipeline, wasm4pm oracle independence principle, mutation testing with 8 XES + 5 JSONL corruption operators, OCEL 2.0 receipt artifacts, and the Dung Gate model with fitness engineering analysis.

### Chapter 5: Verification, Testing & Autonomic Policies
Covers test stratification theory (Tier 1 non-closing vs. Tier 2 evidence gate), all seven public boundary invariants with formal specifications, feature projection tests, the suggest-mode autonomic policy layer with all seven policy implementations, mutation testing strategy, changed-file-driven test selection, and the trybuild conservative mode invariant.

### Chapter 6 + Vision 2030: Conclusions & Strategic Roadmap
Summarizes four primary contributions, identifies six limitations of v26.6.2 (L1–L6), analyzes threats to validity, proposes six future research directions, and presents a 17-milestone Vision 2030 roadmap spanning:
- **2026 H2:** WASM Component Model oracle, LSP wiring, distributed monorepo support, advanced module activation
- **2027:** Declarative pipeline DSL, multi-oracle consensus, real-time evidence streaming, SPARQL-driven test assignment
- **2028:** ISO/IEC 33001 conformance, federated evidence ledger, ML build-time prediction, declarative policy language
- **2029–2030:** Lean 4 formal verification, zero-trust supply chain, cross-language grammar, autonomous remediation, W3C/ISO standard proposal

---

## Key Concepts

| Term | Definition |
|------|-----------|
| Level 5 engine | Process-data engine that manufactures its own grammar from a formal ontology |
| XES | XML Event Stream — IEEE Std 1849-2016 format for process event logs |
| wasm4pm | External WebAssembly oracle that adjudicates all process verdicts |
| EngineState | Aggregate root collecting all workspace state dimensions |
| Dung Gate | Release quality gate requiring oracle-adjudicated evidence |
| E1–E7 | Seven formal invariants governing evidence emission correctness |
| ggen | Grammar generation tool: ontology → CLI grammar + docs + tests |
| cicd.toml | Persistent state carrier written after each major operation |

---

## Reading Order

For a linear read: Ch. 1 → Ch. 2 → Ch. 3 → Ch. 4 → Ch. 5 → Ch. 6 → Appendix A (Vision 2030).

For architecture-first: Ch. 3 → Ch. 4 → Ch. 1 → Ch. 5 → Ch. 6.

For roadmap-only: Appendix A in [chapter6_conclusions_vision2030.md](./chapter6_conclusions_vision2030.md).
