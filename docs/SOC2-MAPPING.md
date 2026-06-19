# SOC2 Trust Service Criteria — cargo-cicd Evidence Mapping

**Standard:** SOC2 Trust Service Criteria (2022 edition)  
**Revision:** Vision 2030 Phase 1 (2026-06-19)  
**Binary:** `cargo-cicd` v26.6.2  
**Oracle:** wasm4pm (`wpm` binary)

---

## Overview

cargo-cicd process evidence satisfies 6 SOC2 Trust Service Criteria across Security, Availability, Processing Integrity, and Confidentiality categories via OCEL 2.0 event logs adjudicated by wasm4pm. Each criterion maps to one or more `cargo cicd` commands whose emitted evidence, once adjudicated to `Accept` by the wasm4pm oracle, constitutes a documented control activity record.

---

## Trust Service Criteria Mapping

| Criterion | Category | Title | cargo-cicd Coverage | Evidence Command |
|---|---|---|---|---|
| **CC6.1** | Security | Logical access security | `cargo cicd git close`, `cargo cicd affidavit seal` | OCEL 2.0 event log records phase closure and cryptographic seal of artifacts |
| **CC7.2** | Security | Incident detection | `cargo cicd status audit`, `cargo cicd evidence doctor` | OCEL 2.0 events carry `verdict_claimed` = WARN/FAIL for anomaly detection; `evidence doctor` validates event log integrity |
| **A1.1** | Availability | Capacity management | `cargo cicd target show`, `cargo cicd target prune` | Target directory size and reclaimable bytes recorded in OCEL 2.0 event log and cicd.toml `[target]` section |
| **PI1.1** | Processing Integrity | Complete and accurate processing | `cargo cicd pipeline run`, `cargo cicd evidence audit` | `trace_class = "pipeline_run"` aggregates all pipeline stages in a single OCEL 2.0 event log; wasm4pm adjudicates completeness |
| **PI1.4** | Processing Integrity | Output integrity | `cargo cicd affidavit seal`, `cargo cicd affidavit verify` | BLAKE3 receipt sealed at publish time; `affidavit verify` re-checks integrity against receipt — verdict recorded in OCEL 2.0 |
| **C1.1** | Confidentiality | Confidentiality policy | `cargo cicd workspace doctor` | Workspace topology (member list, Rust edition, repo path) recorded in OCEL 2.0 event log; no secrets emitted to evidence files |

---

## Evidence Attributes Used for Compliance

| Attribute | SOC2 Purpose |
|---|---|
| `event_id` | Unique control activity identifier — satisfies audit trail requirements |
| `timestamp_iso` | Temporal ordering of control activities |
| `case_id` | Groups events into logical control categories (OCEL 2.0 event log) |
| `lifecycle_transition` | Marks `start` and `complete` boundaries for each control activity |
| `workspace_id` | Identifies the system under assessment |
| `repo_path` | Configuration item identifier for change management |
| `command` | Maps to the specific control objective implemented |
| `verdict_claimed` | Internal quality gate result (PASS/WARN/FAIL) |
| `verdict_adjudicated` | External oracle verdict (Accept/Refuse/Blocked) — satisfies independence requirement |
| `duration_ms` | Control activity completeness evidence |
| `oracle_command` | Identifies the adjudication tool used for independent verification |

---

## Gaps — What cargo-cicd Does Not Cover

| SOC2 Criterion | Gap | Recommended Supplement |
|---|---|---|
| **CC6.2** — User authentication | cargo-cicd does not manage user access credentials | Identity provider (IdP) audit logs |
| **CC6.3** — Access revocation | cargo-cicd does not manage access lifecycle | Directory service / IAM audit records |
| **CC6.6** — Logical access over transmission | Network-layer controls are out of scope | TLS inspection / network flow logs |
| **CC8.1** — Change management | cargo-cicd tracks git phase but does not enforce change approval workflows | Change management system (ITSM) |
| **A1.2** — Recovery planning | cargo-cicd does not cover backup/restore procedures | Disaster recovery documentation |
| **PI1.2** — Commitments to stakeholders | cargo-cicd produces evidence but does not manage SLA commitments | Service-level agreement documentation |

---

## Example: SOC2 Control Evidence Run

```sh
# CC6.1 — Logical access security: phase closure + artifact seal
cargo cicd git close
# Emits: target/cargo-cicd/evidence/events.ocel.json (lifecycle_transition = complete)

cargo cicd affidavit seal
# Emits: target/cargo-cicd/evidence/events.ocel.json, receipts/<seal>.json

# CC7.2 — Incident detection: status audit + evidence integrity
cargo cicd status audit
# Emits: target/cargo-cicd/evidence/events.ocel.json (verdict_claimed = PASS/WARN/FAIL)

cargo cicd evidence doctor
# Validates: target/cargo-cicd/evidence/events.ocel.json integrity

# A1.1 — Capacity management: target directory metrics
cargo cicd target show
# Emits: target/cargo-cicd/evidence/events.ocel.json (total_size_bytes, reclaimable_bytes)

# PI1.1 — Complete and accurate processing: pipeline aggregate
cargo cicd pipeline run
# Emits: target/cargo-cicd/evidence/events.ocel.json (trace_class=pipeline_run)

# PI1.4 — Output integrity: affidavit verify
cargo cicd affidavit verify
# Emits: target/cargo-cicd/evidence/events.ocel.json (BLAKE3 receipt check)

# C1.1 — Confidentiality policy: workspace topology record
cargo cicd workspace doctor
# Emits: target/cargo-cicd/evidence/events.ocel.json (workspace members, no secrets)

# Adjudicate all evidence
cargo cicd evidence audit
# Invokes: wpm audit target/cargo-cicd/evidence/events.ocel.json
# Produces: receipts/*.json with verdict_adjudicated = Accept

# Receipt integrity
wpm receipt doctor --format json --strict receipts/*.json
# Output: Accept
```

After this sequence, the OCEL 2.0 event log (`target/cargo-cicd/evidence/events.ocel.json`) and JSON receipts in `receipts/` collectively satisfy the SOC2 evidence requirements for criteria CC6.1, CC7.2, A1.1, PI1.1, PI1.4, and C1.1.

---

## Live Compliance Summary

View the live SOC2 compliance summary with:

```sh
cargo cicd certification show
```

This command renders the current status of all 6 Trust Service Criteria coverage based on the most recent OCEL 2.0 event log adjudicated by wasm4pm.

---

## Certification Body Submission

To obtain a SOC2 Type II attestation using cargo-cicd evidence:

1. Run the evidence sequence above continuously over a minimum 6-month period.
2. Collect the OCEL 2.0 event log from `target/cargo-cicd/evidence/events.ocel.json` at each audit interval.
3. Collect all receipts from `receipts/`.
4. Submit to a qualified SOC2 auditor — see `docs/CERT-BODY-INTEGRATION.md` for the list of supported providers.
5. Store the returned attestation report in `receipts/` and register the assessment date in `safety-critical-registry.toml`.
