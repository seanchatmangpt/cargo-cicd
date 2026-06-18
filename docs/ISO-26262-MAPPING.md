# ISO 26262 — cargo-cicd Evidence Mapping

**Standard:** ISO 26262:2018 — Road vehicles: Functional safety  
**Parts covered:** Part 6 (Software) and Part 8 (Safety management of the safety lifecycle)  
**Revision:** Vision 2030 Phase 1 (2026-06-17)  
**Binary:** `cargo-cicd` v26.6.2  
**Oracle:** wasm4pm (`wpm` binary)

---

## Overview

ISO 26262 requires systematic evidence of software safety activities across the vehicle lifecycle. cargo-cicd emits process evidence in XES format, adjudicated by the wasm4pm oracle. This document maps ISO 26262 Part 6 and Part 8 clauses to the corresponding `cargo cicd` commands and evidence attributes.

---

## Part 6 (Software) Clause-to-Command Mapping

| ISO 26262 Clause | Title | cargo-cicd Coverage | Evidence Attribute |
|---|---|---|---|
| **6.4.2** | Software safety requirements | `cargo cicd status show`, `cargo cicd workspace doctor` | `workspace_id`, `verdict_claimed` in XES; cicd.toml records crate topology as requirement baseline |
| **6.6** | Software unit design and implementation | `cargo cicd workspace doctor` | Workspace member list, crate names, Rust edition — XES `command = "workspace doctor"` |
| **6.7** | Software unit testing | `cargo cicd test changed`, `cargo cicd trybuild changed`, `cargo cicd trybuild full` | `command` field encodes test scope; `verdict_claimed` = PASS/FAIL; changed-file list in cicd.toml |
| **6.8** | Software integration and testing | `cargo cicd pipeline run` | `trace_class = "pipeline_run"`; aggregate verdict in XES trace spanning all integration stages |
| **6.9** | Verification of software safety requirements | `cargo cicd evidence audit` | `verdict_adjudicated` = Accept/Refuse issued by wasm4pm — satisfies independence per 6.9 |

---

## Part 8 (Safety Management) Clause-to-Command Mapping

| ISO 26262 Clause | Title | cargo-cicd Coverage | Evidence Attribute |
|---|---|---|---|
| **8.3** | Software configuration management | `cargo cicd git status`, `cargo cicd git phase`, `cargo cicd git close` | Branch, ahead/behind, dirty-file list; `lifecycle_transition` encodes phase closure |

---

## Evidence Attributes Used for Compliance

| Attribute | ISO 26262 Purpose |
|---|---|
| `event_id` | Work product identifier per Part 8 CM requirements |
| `timestamp_iso` | Temporal ordering of safety activities |
| `case_id` | Groups work products into lifecycle phases (XES `<trace>`) |
| `lifecycle_transition` | Marks activity start/complete per work product structure |
| `workspace_id` | Software item identifier (Part 6 scope) |
| `repo_path` | Configuration item identifier (Part 8.3) |
| `command` | Maps to specific ISO 26262 work product activity |
| `verdict_claimed` | Internal quality gate (PASS/WARN/FAIL) |
| `verdict_adjudicated` | External oracle verdict — satisfies independence requirement of 6.9 |
| `duration_ms` | Activity completeness evidence |
| `oracle_command` | Identifies the verification tool per 6.9 |

---

## Gaps — What cargo-cicd Does Not Cover

| ISO 26262 Clause | Gap | Recommended Supplement |
|---|---|---|
| **6.4.3** — SOTIF analysis | Software of Intended Functionality analysis is out of scope | Dedicated SOTIF toolchain |
| **6.5** — Software safety architecture | cargo-cicd records topology but does not assess architectural safety properties | Architecture safety analysis (e.g., FMEA, FTA) |
| **6.7.3** — MC/DC coverage | cargo-cicd runs tests but does not measure Modified Condition/Decision Coverage | Coverage tools (LLVM coverage, grcov) |
| **6.10** — Software tool qualification | cargo-cicd must be qualified as a software development tool for ASIL C/D | Contact a certification body |
| **7.x** — Hardware-software interface | cargo-cicd operates at the software workspace level | Hardware-in-the-loop (HiL) records |
| **8.2** — Safety management | Organisational safety management is not covered | Quality management system |
| **8.4** — Project configuration management | cargo-cicd tracks git state but does not enforce baseline locks | ASIL-compliant version management tooling |

---

## Example: ASIL B Compliance Evidence Run

```sh
# 1. Software safety requirements baseline (ISO 26262 6.4.2)
cargo cicd workspace doctor
# Emits: target/cargo-cicd/evidence/evt-workspace-doctor-<ts>.xes

# 2. Unit design record (ISO 26262 6.6)
cargo cicd status show
# Emits: target/cargo-cicd/evidence/evt-status-show-<ts>.xes

# 3. Unit testing — changed scope (ISO 26262 6.7)
cargo cicd test changed
# Emits: target/cargo-cicd/evidence/evt-test-changed-<ts>.xes

cargo cicd trybuild changed
# Emits: target/cargo-cicd/evidence/evt-trybuild-changed-<ts>.xes

# 4. Integration testing (ISO 26262 6.8)
cargo cicd pipeline run
# Emits: target/cargo-cicd/evidence/evt-pipeline-run-<ts>.xes (trace_class=pipeline_run)

# 5. Configuration management (ISO 26262 8.3)
cargo cicd git status
# Emits: target/cargo-cicd/evidence/evt-git-status-<ts>.xes (branch, dirty files)

cargo cicd git close
# Emits: target/cargo-cicd/evidence/evt-git-close-<ts>.xes (phase closure)

# 6. Verification (ISO 26262 6.9)
cargo cicd evidence audit
# Invokes: wpm audit target/cargo-cicd/evidence/*.xes
# Produces: receipts/*.json with verdict_adjudicated = Accept

# 7. Receipt integrity
wpm receipt doctor --format json --strict receipts/*.json
# Output: Accept
```

After this sequence, the XES files and JSON receipts satisfy the ISO 26262 ASIL B documentation requirements for clauses 6.4.2, 6.6, 6.7, 6.8, 6.9, and 8.3.

---

## Certification Body Submission

To obtain an ISO 26262 ASIL B or higher receipt:

1. Run the evidence sequence above in your release pipeline.
2. Collect all XES files from `target/cargo-cicd/evidence/`.
3. Collect all receipts from `receipts/`.
4. Submit to a certification body supporting ISO 26262 — see `docs/CERT-BODY-INTEGRATION.md`.
5. Store the returned receipt in `receipts/` and register the crate in `safety-critical-registry.toml`.

---

## Rust API

```rust
use cargo_cicd::certification::iso_26262::{requirements, check_requirement, compliance_summary, Asil};

let asil = Asil::B;
let reqs = requirements();
let event_commands = vec!["test changed".to_string(), "evidence audit".to_string()];

let mut satisfied = Vec::new();
let mut missing = Vec::new();

for req in &reqs {
    if req.min_asil.severity() <= asil.severity() {
        match check_requirement(req, &event_commands) {
            None    => satisfied.push(req.clause.clone()),
            Some(_) => missing.push(req.clause.clone()),
        }
    }
}

println!("{}", compliance_summary(&asil, &satisfied, &missing));
```
