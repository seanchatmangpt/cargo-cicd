# IEC 61508 — cargo-cicd Evidence Mapping

**Standard:** IEC 61508:2010 — Functional Safety of Electrical/Electronic/Programmable Electronic Safety-Related Systems  
**Revision:** Vision 2030 Phase 1 (2026-06-17)  
**Binary:** `cargo-cicd` v26.6.2  
**Oracle:** wasm4pm (`wpm` binary)

---

## Overview

IEC 61508 requires that each phase of the software safety lifecycle produce documented evidence of activities performed. cargo-cicd emits process evidence in XES (XML Event Stream) format, adjudicated by the wasm4pm oracle. This document maps each IEC 61508 clause to the corresponding `cargo cicd` command and evidence attribute.

---

## Clause-to-Command Mapping

| IEC 61508 Clause | Requirement Title | cargo-cicd Coverage | Evidence Attribute |
|---|---|---|---|
| **5.2.4** | Safety lifecycle documentation | `cargo cicd evidence doctor`, `cargo cicd evidence audit` | `event_id`, `timestamp_iso`, `verdict_claimed`, `verdict_adjudicated` in XES trace |
| **7.4.2** | Software requirements specification | `cargo cicd status show`, `cargo cicd workspace doctor` | `workspace_id`, `command`, `verdict_claimed` — workspace name and member topology in cicd.toml |
| **7.4.3** | Software architecture design | `cargo cicd workspace doctor` | `repo_path`, workspace member list, Rust edition in cicd.toml `[workspace]` section |
| **7.4.5** | Software module testing | `cargo cicd test changed`, `cargo cicd trybuild changed`, `cargo cicd trybuild full` | `command` field encodes changed-file scope; `verdict_claimed` = PASS/FAIL per run |
| **7.4.6** | Software integration testing | `cargo cicd pipeline run` | `trace_class = "pipeline_run"`, aggregate verdict across all pipeline stages in a single XES trace |
| **7.4.7** | Software verification | `cargo cicd evidence audit` | `verdict_adjudicated` = Accept/Refuse issued by wasm4pm oracle — constitutes independent verification record |
| **7.4.9** | Software validation | `cargo cicd publish run`, `cargo cicd evidence audit` | `command = "publish run"`, `verdict_adjudicated` from wasm4pm must be Accept before release |
| **8.4.6** | Software modification | `cargo cicd test changed`, `cargo cicd git status` | `changed_files` in cicd.toml; XES events scoped to changed-file set |

---

## Evidence Attributes Used for Compliance

Each XES event emitted by cargo-cicd carries the following attributes relevant to IEC 61508:

| Attribute | IEC 61508 Purpose |
|---|---|
| `event_id` | Unique activity identifier per clause 5.2.4 traceability |
| `timestamp_iso` | Temporal ordering evidence per clause 5.2.4 |
| `case_id` | Groups events into lifecycle phases (XES `<trace>`) per clause 5.2.4 |
| `lifecycle_transition` | Marks `start` and `complete` boundaries per clause 5.2.4 |
| `workspace_id` | Identifies the software item under assessment per clause 7.4.2 |
| `repo_path` | Configuration item identifier per clause 8.4.6 |
| `command` | Maps to specific safety lifecycle activity per clauses 7.4.2–7.4.9 |
| `verdict_claimed` | Internal quality gate result (PASS/WARN/FAIL) |
| `verdict_adjudicated` | External oracle verdict (Accept/Refuse/Blocked) — satisfies independence requirement |
| `duration_ms` | Temporal completeness evidence per clause 5.2.4 |
| `oracle_command` | Identifies the adjudication tool used per clause 7.4.7 |

---

## Gaps — What cargo-cicd Does Not Cover

The following IEC 61508 activities require supplementary tooling beyond cargo-cicd:

| IEC 61508 Clause | Gap | Recommended Supplement |
|---|---|---|
| **7.4.2** — FMEA/FTA | cargo-cicd does not perform failure mode analysis | Use dedicated FMEA tools; attach reports to receipts/ |
| **7.4.3** — Architecture safety analysis | cargo-cicd records topology but does not verify safety properties of the architecture | MISRA static analysis or formal verification tools |
| **7.4.4** — Support tools qualification | cargo-cicd itself must be qualified as a support tool for SIL 2+ projects | Contact a certification body (see `docs/CERT-BODY-INTEGRATION.md`) |
| **7.4.8** — Software integration with hardware | cargo-cicd operates at the software workspace level; hardware integration is out of scope | Hardware-in-the-loop (HiL) test records |
| **8.2** — Functional safety management | Organisational safety management is not covered | Quality management system (e.g., ISO 9001) |
| **8.2.3** — Independence of V&V | wasm4pm oracle provides process-level independence; human reviewer independence is not enforced | Formal code review records per your QMS |

---

## Example: SIL 2 Compliance Evidence Run

The following shows a complete sequence producing SIL 2-compliant evidence:

```sh
# 1. Workspace health (IEC 61508 7.4.2, 5.2.4)
cargo cicd status show
# Emits: target/cargo-cicd/evidence/evt-status-show-<ts>.xes

# 2. Architecture record (IEC 61508 7.4.3)
cargo cicd workspace doctor
# Emits: target/cargo-cicd/evidence/evt-workspace-doctor-<ts>.xes

# 3. Module testing — changed files only (IEC 61508 7.4.5, 8.4.6)
cargo cicd test changed
# Emits: target/cargo-cicd/evidence/evt-test-changed-<ts>.xes

# 4. Trybuild: compiler error snapshot tests (IEC 61508 7.4.5)
cargo cicd trybuild changed
# Emits: target/cargo-cicd/evidence/evt-trybuild-changed-<ts>.xes

# 5. Integration pipeline (IEC 61508 7.4.6)
cargo cicd pipeline run
# Emits: target/cargo-cicd/evidence/evt-pipeline-run-<ts>.xes (trace_class=pipeline_run)

# 6. Publish gate (IEC 61508 7.4.9)
cargo cicd publish run
# Emits: target/cargo-cicd/evidence/evt-publish-run-<ts>.xes

# 7. Evidence adjudication (IEC 61508 7.4.7)
cargo cicd evidence audit
# Invokes: wpm audit target/cargo-cicd/evidence/*.xes
# Produces: receipts/*.json with verdict_adjudicated = Accept

# 8. Receipt integrity check (IEC 61508 5.2.4)
wpm receipt doctor --format json --strict receipts/*.json
# Output: Accept (all receipts valid)
```

After this sequence, the XES files in `target/cargo-cicd/evidence/` and the JSON receipts in `receipts/` collectively satisfy the IEC 61508 SIL 2 documentation requirements for clauses 5.2.4, 7.4.2, 7.4.3, 7.4.5, 7.4.6, 7.4.7, 7.4.9, and 8.4.6.

---

## Certification Body Submission

To obtain an IEC 61508 SIL 2 receipt from a certification body:

1. Run the evidence sequence above in your release pipeline.
2. Collect all XES files from `target/cargo-cicd/evidence/`.
3. Collect all receipts from `receipts/`.
4. Submit to a certification body — see `docs/CERT-BODY-INTEGRATION.md` for the list of supported providers.
5. Store the returned receipt in `receipts/` and register the crate in `safety-critical-registry.toml`.

---

## Rust API

```rust
use cargo_cicd::certification::iec_61508::{requirements, check_requirement, compliance_summary, Sil};

let sil = Sil::new(2);
let reqs = requirements();
let event_commands = vec!["test changed".to_string(), "evidence audit".to_string()];

let mut satisfied = Vec::new();
let mut missing = Vec::new();

for req in &reqs {
    if req.min_sil <= sil {
        match check_requirement(req, &event_commands) {
            None    => satisfied.push(req.number.clone()),
            Some(_) => missing.push(req.number.clone()),
        }
    }
}

println!("{}", compliance_summary(&sil, &satisfied, &missing));
```
