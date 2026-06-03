# DAY3_RECOMMENDATION — cargo-cicd v26.6.2

**Date:** 2026-06-03

---

## Recommended Day 3 Target

**LSP editor diagnostics proof**

---

## Rationale

The LSP explain feature has the highest FruitScore (12.0) of all Day 3 candidates. The run logic is implemented. The CICD_CATALOG lookup is implemented with 22 entries. The only gap is a single clap wiring issue: the `code` positional arg is declared in `additional_args()` but is not forwarded through `build_command()` in clap-noun-verb 26.6.2. This makes the `code` arg unreachable at runtime.

Fixing this wiring is:
- **Bounded** — changes are local to `build_command()` and the explain command
- **Low risk** — no schema changes, no binary dependencies, no external service calls
- **High value** — makes all 22 CICD_CATALOG codes externally usable via CLI
- **Provable** — proof is a single JSON receipt from `cargo cicd lsp explain CICD-GIT-001`

---

## Why Not the Other Candidates

**CICD-WPM-004 + regression fixture (FruitScore 6.0):** Next-best candidate after LSP explain is completed. Requires the wpm PATH situation to be resolved for CI or the `WPM_PATH` env var to be documented. Not a Day 3 opener.

**Publish gate as adjudicated receipt (FruitScore 5.33):** Requires defining a new receipt schema before any code can be written. Adds scope. Not a Day 3 opener.

**Conformance 1.0 feedback closure (FruitScore 0.625):** Root cause of the 0.9636 vs 0.8194 discrepancy is uninvestigated. The precision metric is absent and undocumented. Too much unknown surface to open on Day 3.

**Spec Kit integration (FruitScore 0.9):** Entirely greenfield. No existing code, no schema, no fixtures. Not a Day 3 target.

---

## First Step

Locate `build_command()` in the clap-noun-verb 26.6.2 crate and add positional arg forwarding from `additional_args()`. Verify with:

```
cargo cicd lsp explain CICD-GIT-001
```

Expected result: JSON receipt containing the diagnostic explanation for CICD-GIT-001.

---

## Day 3 Sequence (if first step completes)

1. Wire `code` positional arg through `build_command()` — verify CICD-GIT-001 reachable
2. Confirm at least 7 catalog codes return valid explanations
3. Emit JSON receipt as proof
4. (Optional, if time allows) Wire CICD-WPM-004 into `analyzers/runtime_court.rs` and add regression fixture

---

## Open Risks

| Risk | Probability | Mitigation |
|---|---|---|
| LSP server does not start in editor | Medium | Initialize test; verify binary on PATH |
| wpm binary path changes | Low | Set WPM_PATH env var |
| Conformance regresses | Medium | pipeline run test on canonical event log |
| Publish without adjudication | Low | Receipt doctor gate |
| Private term leak in public docs | Low | Public boundary scan on every ggen render |
