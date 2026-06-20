<!-- BEGIN ggen:command-reference -->
<!-- Rendered from ontology/cargo-cicd-capabilities.ttl. Do not edit by hand. -->

# `cargo cicd certification show`

Displays a summary of certification-readiness across multiple compliance standards: IEC 61508 (functional safety), ISO 26262 (automotive safety), SOC2 Trust Services Criteria, and TOGAF ADM architectural coverage. Prints the set of known certification bodies and emits a `certification:show` ProcessEvent.

**Noun:** `certification` &nbsp;&nbsp; **Verb:** `show`

<!-- END ggen:command-reference -->

<!-- BEGIN custom:synopsis -->
## Synopsis

```sh
cargo cicd certification show
```
<!-- END custom:synopsis -->

<!-- BEGIN custom:description -->
## Description

`cargo cicd certification show` queries each compliance module and renders the results in a single consolidated view. The output always contains the string "cargo-cicd certification summary" and is safe to pipe — plain text, no ANSI codes when stdout is not a TTY.

The command is **read-only**. It does not modify workspace state, write to `cicd.toml`, or install any tooling.

### Standards Covered

| Standard | Domain | What Is Checked |
|---|---|---|
| IEC 61508 | Functional safety (general) | IEC 61508 requirement traceability reported by `iec_61508::compliance_summary()` |
| ISO 26262 | Automotive functional safety | ISO 26262 requirement traceability reported by `iso_26262::compliance_summary()` |
| SOC2 TSC | Trust Services Criteria | CC6.1, CC7.2, A1.1, PI1.1, PI1.4, C1.1 — reported by `soc2::compliance_summary()` |
| TOGAF ADM | Enterprise architecture | ADM phase coverage reported by `togaf::coverage_summary()` |

### SOC2 Criteria Reference

| Criterion | Name |
|---|---|
| CC6.1 | Logical and physical access controls |
| CC7.2 | System operations monitoring |
| A1.1 | System availability commitments |
| PI1.1 | Processing integrity objectives |
| PI1.4 | Processing integrity error detection |
| C1.1 | Confidentiality commitments |

### Known Certification Bodies

The command prints the list returned by `known_cert_bodies()`. At the time of writing this includes:

- **Ferrous Systems** — Rust safety-critical audits
- **TrustInSoft** — Formal verification and safety-case preparation
- **Trail of Bits** — Security and functional-safety assessments
<!-- END custom:description -->

<!-- BEGIN custom:evidence -->
## Evidence Emission

Each invocation emits a `certification:show` ProcessEvent to `target/cargo-cicd/evidence/`. The event follows the standard lifecycle:

1. `start` transition emitted at entry
2. Work executes (compliance module queries)
3. `complete` transition emitted with `verdict_claimed` set to `PASS`

The XES trace element uses `case_id = "certification_show_phase"`.

```
target/cargo-cicd/evidence/
└── evt-certification-show-<timestamp>Z.xes
└── evt-certification-show-<timestamp>Z.jsonl
```
<!-- END custom:evidence -->

<!-- BEGIN custom:exit-codes -->
## Exit Codes

| Code | Meaning |
|---|---|
| 0 | All compliance summaries rendered successfully |
| 1 | An internal error prevented one or more modules from reporting |
| 2 | Invalid workspace — `Cargo.toml` not found |
<!-- END custom:exit-codes -->

<!-- BEGIN custom:examples -->
## Examples

```sh
# Show certification readiness summary
cargo cicd certification show

# Capture plain-text output (pipe disables color automatically)
cargo cicd certification show | tee certification-report.txt
```
<!-- END custom:examples -->
