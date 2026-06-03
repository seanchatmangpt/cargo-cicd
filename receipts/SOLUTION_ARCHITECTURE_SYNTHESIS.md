# Receipt: Solution Architecture Synthesis

**date:** 2026-06-03
**version:** cargo-cicd v26.6.2
**synthesizer:** Claude Code (claude-sonnet-4-6)

---

## Surfaces Read

| Surface | File | Purpose |
|---------|------|---------|
| Primary architecture | `docs/ARCHITECTURE.md` | Three-tier domain separation, adapter pattern, cicd.toml schema |
| Source law receipt | `receipts/CARGO_CICD_V26_6_2_DOCS_SOURCE_LAW.md` | Diataxis docs, playground results, ggen rules, wasm4pm validation |
| Runtime receipt doctor | `receipts/CARGO_CICD_V26_6_2_RUNTIME_RECEIPT_DOCTOR.md` | Receipt gate architecture, OCEL2 receipt format, wpm commands |
| Declared process model | `receipts/CARGO_CICD_V26_6_2_DECLARED_PROCESS_MODEL.md` | OWL ontology, POWL choice graph, conformance requirements |
| Adjudicated publish gate | `receipts/CARGO_CICD_V26_6_2_ADJUDICATED_PUBLISH_GATE.md` | Publish gate implementation, verdict routing, AndonPull |
| wasm4pm evidence gate | `receipts/CARGO_CICD_V26_6_2_WASM4PM_EVIDENCE_GATE.md` | Oracle commands, positive evidence cases, negative refusal cases |
| Project CLAUDE.md | `CLAUDE.md` | Mission, forbidden terms, feature flags, test hierarchy |

---

## Laws Extracted

| Law | Source | ADR |
|-----|--------|-----|
| Three-crate separation with downward-only imports | ARCHITECTURE.md | ADR-001 |
| Every command emits XES evidence; oracle adjudicates release | CLAUDE.md + evidence gate receipt | ADR-002 |
| ReceiptDoctor is the primary publish gate | Runtime receipt doctor receipt | ADR-003 |
| LSP observer reads state; never acts | CLAUDE.md architecture notes | ADR-004 |
| Receipt lifecycle by keyed subtraction | Runtime receipt doctor receipt | ADR-005 |
| Trailing var-arg is canonical for open-ended positional args | ARCHITECTURE.md (clap-noun-verb section) | ADR-006 |
| Absent verdict keys are errors, not fallback cases | Adjudicated publish gate receipt | ADR-007 |
| Only pipeline traces with declared sequence are admissible | Declared process model receipt | ADR-008 |
| Forbidden terms never appear in public surfaces | CLAUDE.md | ADR-009 |
| Publish requires adjudicated receipt, not internal test pass | All receipts | ADR-010 |

---

## ADRs Written

| ADR | Title | Key Constraint |
|-----|-------|---------------|
| ADR-001 | Three-Crate Separation | Domain never imports CLI |
| ADR-002 | Evidence Gate Invariants | Every command emits; oracle adjudicates |
| ADR-003 | ReceiptDoctor Primary Gate | `wpm receipt doctor --strict` before publish |
| ADR-004 | LSP Observer Not Actor | LspAdapter is read-only |
| ADR-005 | Keyed Subtraction Lifecycle | New receipt replaces prior for same key |
| ADR-006 | Trailing Var-Arg Pattern | All positional lists via `trailing_vararg` |
| ADR-007 | No Silent Fallback on Verdict Keys | Missing `state` key is an error |
| ADR-008 | Pipeline vs. Ambient Trace | Declared sequence required for conformance |
| ADR-009 | Forbidden Terms Public Boundary | 10 terms banned from all public surfaces |
| ADR-010 | Publish Gate Adjudicated Receipt | `RECEIPT_DOCTOR:accepted` required for release |

---

## Anti-Patterns Documented

1. Silent verdict key fallback (`unwrap_or("Admitted")`)
2. Ambient trace admissibility (matching names without declared sequence)
3. Logic accumulation in `run()` methods
4. LSP adapter performing mutations
5. Phantom receipt accumulation without keyed replacement
6. Internal test passage treated as release criterion

---

## Synthesis Verdict

COMPLETE

All 10 laws extracted, all 10 ADRs written, master architecture document created at `docs/SOLUTION_ARCHITECTURE.md`, receipt recorded. No forbidden terms introduced. No destructive operations performed.

Files created:
- `docs/SOLUTION_ARCHITECTURE.md` (master architecture document)
- `docs/adr/ADR-001` through `ADR-010`
- `receipts/SOLUTION_ARCHITECTURE_SYNTHESIS.md` (this file)
- Reference added to `docs/ARCHITECTURE.md`
