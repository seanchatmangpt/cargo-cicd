# ADR-021: Conformance-Driven Affidavit Auto-Wiring (the "Process-Mining Seal Loop")

**Status:** Proposed (Phase 2 candidate)
**Date:** 2026-06-21
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee
**Tags:** affidavit, process-mining, conformance, autonomic, provenance, van-der-aalst

---

## 0. Validated Premise (ground truth, not assumption)

Before proposing anything, the current behaviour was verified against source:

| Claim | Evidence | Verdict |
|---|---|---|
| affidavit is **not** a default feature | `Cargo.toml:66` `default = []`; `Cargo.toml:77` `affidavit = ["process-data"]` | CONFIRMED |
| Noun only exists under `#[cfg(feature = "affidavit")]` | `src/main.rs:116-118` (registration), `:39-40` (default verb) | CONFIRMED |
| Nothing auto-enables it at runtime | no `cfg!()` / env / build.rs flips the flag | CONFIRMED |
| Onboarding never mentions it | `.claude/hooks/session-start.sh:27` checks `wpm` only | CONFIRMED |
| Not ontology-manufactured | empty grep over `ontology/` | CONFIRMED |

**Conclusion of validation:** cargo-cicd does **not** auto-wire affidavit. It is opt-in by deliberate design (ADR-019 progressive disclosure). Sealing only happens if a human (a) compiles `--features affidavit`, (b) installs `affi`, and (c) types `cargo cicd affidavit seal`.

This ADR does not contest that design. It asks a sharper question:

> The evidence journal is a process. Sealing is an activity in that process.
> **Why is the decision to seal a manual reflex instead of a conformance verdict?**

---

## 1. The Van Der Aalst Reframing

cargo-cicd already emits XES — Wil van der Aalst's own log format. That is not cosmetic. It means every command produces a **trace**, and the union of traces is an **event log** amenable to the three classical process-mining operations:

1. **Discovery** — learn a model from the log (what process *is* happening).
2. **Conformance** — replay the log against a model (where reality and model *diverge*).
3. **Enhancement** — feed findings back to improve the process.

Affidavit sealing is the *enhancement* step that is currently missing a *conformance* trigger. Today the engine emits evidence (`process-data`), optionally adjudicates it (`wasm4pm`), but the **provenance seal is never recommended by the engine itself** — even though `src/policies/publish_not_adjudicated.rs` proves the exact mechanism already works for receipts.

The innovation is one sentence:

> **Treat "is this evidence sealed?" as a conformance constraint, evaluated by the autonomic layer, with progressively stronger enforcement modes.**

---

## 2. The Auto-Wiring Ladder (implementable → moonshot)

Each rung is independently shippable. Rung *n* does not require rung *n+1*. This is the antidote to vaporware: the bottom rung lands in a day.

### Rung 1 — `affidavit_unsealed` autonomic policy  *(ships today)*

A new `CicdPolicy` that mirrors `publish_not_adjudicated` exactly. It mines the evidence dir and the affidavit receipt dir:

```rust
// src/policies/affidavit_unsealed.rs
impl CicdPolicy for AffidavitUnsealedPolicy {
    fn name(&self) -> &'static str { "affidavit_unsealed" }
    fn mode(&self) -> PolicyMode { PolicyMode::Suggest }   // never destructive

    fn evaluate(&self, _state: &EngineState) -> PolicyResult {
        let journal = Path::new("target/cargo-cicd/evidence");
        let receipt = journal.join("affidavit/receipt.json");

        // Discovery: how many trace events exist vs. how many are covered by a seal?
        let events   = count_journal_events(journal);
        let sealed   = receipt.exists().then(|| sealed_event_count(&receipt)).unwrap_or(0);
        let coverage = if events == 0 { 1.0 } else { sealed as f32 / events as f32 };

        match coverage {
            c if c >= 1.0 => pass(),
            c if c > 0.0  => warn(format!(
                "provenance seal covers {:.0}% of the journal — run `cargo cicd affidavit seal`", c*100.0)),
            _             => alert(
                "evidence journal is unsealed — run `cargo cicd affidavit seal` to certify provenance"),
        }
    }
}
```

`coverage` is literally **fitness** in Van Der Aalst's sense: the fraction of observed events the seal "replays." This rung makes affidavit *discoverable through the engine's own voice* — fixing the onboarding gap (§0) without changing a single default.

**Crucially:** this policy lives behind `#[cfg(feature = "autonomic")]`, NOT `affidavit`. So even a user who has *never heard of* affidavit gets a recommendation to enable it the moment `workspace doctor` runs. That is the auto-wire — not auto-*action*, but auto-*awareness*.

### Rung 2 — Sealedness as a conformance constraint in ADR-020's checker

ADR-020 already specifies `ConformanceChecker` with `ConformanceViolation`. Add one variant:

```rust
pub enum ConformanceViolation {
    MissingRequiredActivity { activity: String },
    OrderingViolation { /* ... */ },
    TemporalViolation { /* ... */ },
    ExceededMaximumCount { /* ... */ },
    UnsealedProvenance { coverage: f32, last_seal_age_hours: f32 },   // NEW
}
```

Now any process model can *declare* a sealing requirement:

```turtle
cc:SealActivity a pm:Activity ;
    pm:model cc:RegulatedReleaseModel ;
    pm:activityName "affidavit-seal" ;
    pm:required true ;
    pm:provenanceConstraint [ pm:minCoverage 1.0 ; pm:maxSealAgeHours 1 ] .
```

A medical-device model (FDA 21 CFR Part 11, cited in ADR-020 §Context) *requires* sealing; a hobby crate's model omits it. **The feature flag stops being the policy; the process model becomes the policy.** This is the real 1000x: affidavit auto-wires per-model, per-org, per-regulatory-framework — not per-binary-build.

### Rung 3 — Declarative trigger (DECLARE-style) → suggest-mode auto-seal hook

Van der Aalst's DECLARE language expresses constraints like `response(publish, seal)` — "if publish happens, seal must eventually happen." Encode these as a thin rule table the autonomic layer can evaluate after every verb:

```
response(publish.run, affidavit.seal)        # publishing obliges a seal
response(git.close,   affidavit.seal)         # closing a phase obliges a seal
precedence(affidavit.verify, affidavit.seal)  # cannot verify what was never sealed
```

When a `response` obligation is unmet, the engine emits a suggest-mode recommendation **and**, if (and only if) the user has opted into `autonomic = "enforce"` mode, schedules `affidavit seal` as the next action. Still no silent surprise — enforcement is an explicit opt-in, consistent with ADR-019's progressive disclosure and the "policies never take destructive action" invariant.

### Rung 4 — Online conformance & the provenance Merkle-DAG  *(moonshot)*

Streaming/online conformance checking (Van der Aalst, Burattin et al.) computes fitness *as events arrive* rather than post-hoc. Combined with affidavit's BLAKE3 receipts (already content-addressed), the journal becomes a **Merkle-DAG of sealed process states**. Two consequences:

- **Cross-workspace provenance federation:** a monorepo's per-crate seals roll up into a workspace-root super-seal; an organization's workspaces roll into an org-root. Provenance becomes a verifiable supply-chain graph (SLSA-aligned, ADR-020 §References).
- **Incremental sealing:** only the unsealed *frontier* of the DAG needs `affi receipt assemble`, making continuous sealing O(Δ) instead of O(journal). This is what makes Rung 3's "seal after every verb" actually cheap.

---

## 3. Why this is the *correct* design, not just a bigger one

| Principle | How this honours it |
|---|---|
| ADR-019 progressive disclosure | Defaults unchanged; lean binary unchanged; every rung is opt-in |
| "Policies never act destructively" | Rungs 1-2 are suggest-only; Rung 3 enforcement is explicit opt-in |
| E1 (never self-adjudicate) | The *seal verdict* still comes from external `affi`; the engine only recommends |
| Stable toolchain | Nothing here links affidavit; `affi` stays a shell-out oracle |
| Ontology-driven manufacturing (ADR-018) | Rung 2 moves the sealing decision *into the ontology*, where capabilities already live |

The current design answers "should affidavit run?" with a compile flag. This ADR answers it with a **conformance verdict computed from the process's own evidence** — which is exactly what a process-data engine exposed as a CI/CD helper should do.

---

## 4. Milestones

| Rung | Effort | Deliverable | Gate |
|---|---|---|---|
| 1 | ~1 day | `src/policies/affidavit_unsealed.rs` + registry wire + test | `cargo test --features autonomic` |
| 2 | ~1 week | `UnsealedProvenance` violation + `pm:provenanceConstraint` schema | conformance unit tests |
| 3 | ~2 weeks | DECLARE rule table + `autonomic="enforce"` mode | end-to-end suggest/enforce tests |
| 4 | Phase 2+ | Online checker + Merkle-DAG federation | research milestone |

---

## 5. References

- van der Aalst, W. *Process Mining: Data Science in Action* (2nd ed., Springer 2016) — discovery/conformance/enhancement triad.
- van der Aalst, W. et al. *Conformance Checking: Relating Processes and Models* (Carmona, van Dongen, Solti, Weidlich, 2018) — alignments, fitness/precision.
- Burattin, A.; van Zelst, S.; et al. *Online Conformance Checking* — streaming fitness (Rung 4).
- DECLARE: Pesic & van der Aalst, *A Declarative Approach for Flexible Business Processes* (2006) — Rung 3 constraint language.
- ADR-018 (ontology manufacturing), ADR-019 (feature strategy), ADR-020 (pluggable process models).
- `src/policies/publish_not_adjudicated.rs` — the working analog Rung 1 mirrors.

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-21 | Vision 2030 Architecture Committee | Initial proposal — conformance-driven affidavit auto-wiring |
