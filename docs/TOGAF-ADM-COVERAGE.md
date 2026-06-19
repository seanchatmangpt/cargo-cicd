# TOGAF ADM Phase Coverage — cargo-cicd Architecture Evidence

**Framework:** TOGAF 10 — Architecture Development Method (ADM)  
**Revision:** Vision 2030 Phase 1 (2026-06-19)  
**Binary:** `cargo-cicd` v26.6.2  
**Oracle:** wasm4pm (`wpm` binary)

---

## Overview

The cargo-cicd ontology pipeline (ggen RDF → clap-noun-verb grammar → CLI binary) implements 6 of 9 TOGAF ADM phases. Each covered phase has a corresponding architecture artifact produced by the manufacturing pipeline or recorded in the OCEL 2.0 event log adjudicated by wasm4pm. Use `cargo cicd certification show` to view the live phase coverage.

---

## ADM Phase Coverage Table

| Phase | Title | Status | cargo-cicd Architecture Artifact |
|---|---|---|---|
| **Phase A** | Architecture Vision | Deferred | — |
| **Phase B** | Business Architecture | Covered | ggen ontology (RDF capability map in `ontology/cargo-cicd-capabilities.ttl`) |
| **Phase C-App** | Application Architecture | Covered | clap-noun-verb grammar (noun/verb CLI structure manufactured from ontology) |
| **Phase C-Data** | Data Architecture | Covered | OCEL 2.0 event log (`events.ocel.json`) + BLAKE3 receipts (`receipts/*.json`) |
| **Phase D** | Technology Architecture | Covered | Rust workspace + wasm4pm oracle (process evidence adjudication infrastructure) |
| **Phase E** | Opportunities and Solutions | Deferred | — |
| **Phase F** | Migration Planning | Deferred | — |
| **Phase G** | Implementation Governance | Covered | wasm4pm evidence gate Accept/Refuse verdicts (enforced at release boundary) |
| **Phase H** | Architecture Change Management | Covered | `cargo cicd git close` git phase tracking (branch, phase closure, ahead/behind) |

**Coverage: 6 of 9 phases (67%)**

---

## Phase Detail

### Phase B — Business Architecture

**Artifact:** `ontology/cargo-cicd-capabilities.ttl`

The RDF/Turtle capability ontology defines all business capabilities exposed by cargo-cicd. SPARQL inference rules in `queries/` project the capability graph into a clap-noun-verb grammar. This constitutes a machine-executable business architecture in the TOGAF sense: business capabilities are explicitly modelled, not implicit in code.

```sh
# Inspect the business capability model
cat ontology/cargo-cicd-capabilities.ttl
# Or regenerate all downstream artifacts from the ontology:
ggen
```

---

### Phase C-App — Application Architecture

**Artifact:** clap-noun-verb grammar (`src/nouns/`, `src/main.rs`)

The application architecture is manufactured from the ontology via `ggen`. Each noun module in `src/nouns/` implements a `NounCommand`; each verb implements a `VerbCommand`. The CLI grammar is a direct projection of the business capability map — not a hand-authored design.

```sh
# View all manufactured noun-verb pairs
cargo cicd --help
cargo cicd status --help
cargo cicd target --help
```

---

### Phase C-Data — Data Architecture

**Artifact:** `target/cargo-cicd/evidence/events.ocel.json` + `receipts/*.json`

The data architecture is defined by the OCEL 2.0 event log schema and the wasm4pm receipt schema. All process data flows through a single canonical event log. BLAKE3 receipts provide content-addressed integrity proofs for all emitted artifacts.

```sh
# Inspect the current OCEL 2.0 event log
cat target/cargo-cicd/evidence/events.ocel.json

# Inspect receipts
ls receipts/
wpm receipt doctor --format json --strict receipts/*.json
```

---

### Phase D — Technology Architecture

**Artifact:** Rust workspace manifest (`Cargo.toml`) + wasm4pm oracle binary

The technology architecture is the Rust workspace (toolchain, crates, feature flags) and the wasm4pm oracle. The oracle provides an independent adjudication layer separate from the CLI binary itself. Feature flags (`process-data`, `autonomic`, `wasm4pm`, `affidavit`, `advanced`) gate optional technology capabilities.

```sh
# View the technology baseline
cargo cicd workspace doctor
# Emits: target/cargo-cicd/evidence/events.ocel.json

# Verify oracle is available
wpm --version
```

---

### Phase G — Implementation Governance

**Artifact:** wasm4pm Accept/Refuse verdicts in `receipts/*.json`

Implementation governance is enforced at the release boundary: no release may proceed without a wasm4pm `Accept` verdict on the full evidence gate. The `verdict_adjudicated` field in each receipt constitutes the governance record. `Refuse` blocks release; `Blocked` indicates the oracle was unavailable (allowed in offline development, not at release).

```sh
# Run the governance gate
cargo cicd evidence audit
# Invokes: wpm audit target/cargo-cicd/evidence/events.ocel.json
# Blocks release if verdict_adjudicated = Refuse
```

---

### Phase H — Architecture Change Management

**Artifact:** `cargo cicd git close` git phase evidence

Architecture change management is implemented via the git phase lifecycle. `cargo cicd git status` records the current change surface (branch, dirty files, ahead/behind count); `cargo cicd git close` records phase closure. Both emit OCEL 2.0 events with `lifecycle_transition` = `complete` when the phase is cleanly closed.

```sh
# Record current change surface
cargo cicd git status
# Emits: target/cargo-cicd/evidence/events.ocel.json (branch, dirty_files, ahead, behind)

# Close the current phase
cargo cicd git close
# Emits: target/cargo-cicd/evidence/events.ocel.json (lifecycle_transition = complete)
```

---

## Deferred Phases

| Phase | Title | Reason Deferred |
|---|---|---|
| **Phase A** | Architecture Vision | Requires a stakeholder-facing vision document outside the scope of the CLI manufacturing pipeline |
| **Phase E** | Opportunities and Solutions | Solution portfolio management is not yet modelled in the ontology |
| **Phase F** | Migration Planning | Migration planning tooling is planned for a future Vision 2030 phase |

These phases are tracked in `docs/ROADMAP-2030.md` and `docs/PHASE-3-DESIGN.md`.

---

## Live Coverage View

View the live TOGAF ADM phase coverage with:

```sh
cargo cicd certification show
```

This command renders the current phase coverage based on the most recent OCEL 2.0 event log and ontology state.

---

## Gaps and Supplementary Tooling

| TOGAF Concern | Gap | Recommended Supplement |
|---|---|---|
| Architecture Repository | cargo-cicd does not maintain a formal TOGAF Architecture Repository | Enterprise architecture tool (e.g., Archi, LeanIX) |
| Stakeholder Management | No stakeholder register or RACI is generated | Organisation-level TOGAF governance process |
| Architecture Contracts | cargo-cicd enforces process contracts (via wasm4pm) but not formal TOGAF architecture contracts | Contract management system |
| Capability Maturity | Phase coverage is binary (covered/deferred); maturity levels are not assessed | TOGAF maturity model assessment |
