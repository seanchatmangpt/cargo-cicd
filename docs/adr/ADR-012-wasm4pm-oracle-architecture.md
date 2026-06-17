# ADR-012: External Oracle Architecture — cargo-cicd Never Adjudicates Itself

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** oracle, adjudication, evidence, invariants, wasm4pm, separation-of-concerns

---

## Context

cargo-cicd executes CI/CD operations (status checks, test runs, publish gates, git phase closures) and emits process evidence recording what happened and what outcome was claimed. A natural implementation would have cargo-cicd examine its own evidence and declare whether the execution met quality standards — self-certification.

This ADR addresses why self-certification is structurally forbidden in cargo-cicd and why an external oracle (wasm4pm) adjudicates all process evidence.

### The Self-Certification Problem

Consider a tool that checks whether tests pass and then certifies the test results. If the tool itself has a bug:

1. **Scenario A — Honest bug**: The tool fails to detect a test failure and claims `PASS`. The certification is wrong. Users ship broken code.
2. **Scenario B — Intentional manipulation**: The tool is modified to always claim `PASS` regardless of test outcomes. Self-certification provides no protection against this.
3. **Scenario C — Conflict of interest**: The tool author has incentive to make the tool look good. Self-reported metrics are not auditable by third parties.

In software supply chain contexts, self-certification is equivalent to "trust me." For regulatory compliance (DO-178C, FDA 21 CFR Part 11, ISO 26262), self-certification is explicitly insufficient — an independent verifier is required.

### What wasm4pm Provides

wasm4pm (`wpm` binary) is an external process evidence oracle that:

1. **Operates independently**: wpm is a separate binary compiled from a separate codebase with separate release cycles.
2. **Has no interest in cargo-cicd outcomes**: wpm adjudicates based on the evidence structure, not on cargo-cicd's claims.
3. **Is deterministic**: Given the same XES input, wpm always produces the same verdict.
4. **Is versioned separately**: wpm version changes are tracked independently. A specific wpm version can be pinned for certification.
5. **Can be replaced**: The oracle interface is abstract; a different adjudicator could be substituted without changing cargo-cicd's emission code.

### Current State (Pre-Vision 2030)

The invariant is enforced in code via `E1` in `src/evidence.rs`:

```rust
// E1: cargo-cicd never adjudicates itself.
// Only wasm4pm issues verdicts. We emit; oracle adjudicates.
//
// INVARIANT: No function in this module may return a WpmVerdict.
// Only Wasm4pmShell::audit_xes() may obtain a WpmVerdict, and only
// by invoking the external wpm binary.
```

And in the test suite (`tests/invariants.rs`):

```rust
#[test]
fn invariant_cargo_cicd_never_self_adjudicates() {
    // Verify that no source file outside integrations/wasm4pm_shell.rs
    // calls any function that returns a WpmVerdict directly.
    // The only legitimate source of WpmVerdict is wpm binary invocation.
}
```

### Vision 2030 Implications

As cargo-cicd scales to ecosystem platform status, the oracle architecture must handle:

1. **Multiple oracles**: Different compliance frameworks may require different oracles (NIST SP 800-218, DO-178C verifier, custom organizational policy).
2. **Distributed oracle consensus**: High-assurance contexts require M-of-N independent oracle agreement (see ADR-016).
3. **Oracle provenance**: Which oracle signed which receipt must be cryptographically verifiable (see ADR-013).
4. **Oracle versioning**: Receipts must record the exact oracle version used, so future audits can reproduce the adjudication.

---

## Decision

**Hard Invariant E1: cargo-cicd never adjudicates itself. Only wasm4pm issues verdicts.**

This invariant is permanent, non-negotiable, and enforced at multiple levels:

### Level 1: Architectural Constraint

The system is structured so that `verdict_adjudicated` can only be set via the external oracle path:

```
cargo-cicd emits ProcessEvent { verdict_claimed = "PASS" }
    ↓
Serializes to XES in target/cargo-cicd/evidence/
    ↓
[cargo-cicd halts — it does not touch verdict_adjudicated]
    ↓
External invocation: wpm audit <evidence.xes>
    ↓
wpm returns: "Accept" | "Refuse" | "Blocked"
    ↓
Test harness reads wpm output → asserts verdict
```

cargo-cicd's `verdict_claimed` is a claim, not a verdict. The distinction is permanent.

### Level 2: Code Structure

```rust
pub struct ProcessEvent {
    /// What cargo-cicd claims happened. This is a CLAIM, not a verdict.
    /// Only wasm4pm can produce a verdict.
    pub verdict_claimed: String,

    /// Set only by Wasm4pmShell::audit_xes() after oracle invocation.
    /// None until oracle adjudicates.
    pub verdict_adjudicated: Option<String>,

    /// The exact oracle command that produced verdict_adjudicated.
    /// Enables reproduction of the adjudication.
    pub oracle_command: Option<String>,
}
```

No setter for `verdict_adjudicated` is exposed in the public API except through `Wasm4pmShell`.

### Level 3: Test Assertions

Tests are explicitly forbidden from asserting on `verdict_claimed`:

```rust
// FORBIDDEN — asserting on cargo-cicd's own claim:
assert_eq!(event.verdict_claimed, "PASS");  // ❌ WRONG

// REQUIRED — asserting on the oracle's verdict:
let verdict = Wasm4pmShell::audit_xes(&xes_path)?;
assert_eq!(verdict, WpmVerdict::Accept);     // ✅ CORRECT
```

The `tests/invariants.rs` suite enforces this pattern. Any test asserting on `verdict_claimed` directly (without going through the oracle) fails the invariant check.

### Level 4: Compile-Time Gate

The `wasm4pm` feature flag gates the oracle integration seam:

```rust
#[cfg(feature = "wasm4pm")]
mod wasm4pm_shell {
    pub fn audit_xes(path: &Path) -> Result<WpmVerdict> {
        // Only place in the codebase that invokes wpm binary
        let output = Command::new("wpm")
            .arg("audit")
            .arg(path)
            .output()?;
        WpmVerdict::from_str(&String::from_utf8_lossy(&output.stdout).trim())
    }
}
```

Without the `wasm4pm` feature, the oracle path is not compiled in. Tests that require oracle adjudication must declare `ExpectedWpmVerdict::Blocked` when the feature is disabled.

### Oracle Interface Contract

The oracle contract is defined by the wpm CLI interface, not by cargo-cicd:

```
Input:  A well-formed XES file at a given path
Output: One of three tokens printed to stdout:
          "Accept"  — evidence is conformant; claim is adjudicated
          "Refuse"  — evidence is non-conformant; claim is rejected
          "Blocked" — oracle cannot adjudicate (offline, corrupted, etc.)
Exit:   0 for Accept/Refuse, non-zero for Blocked
```

This interface is stable and versioned by wasm4pm's semver. cargo-cicd pins a minimum wpm version in its documentation and test matrix.

### Multiple Oracle Support (Phase 2)

Under Vision 2030 Phase 2, M-of-N oracle consensus (see ADR-016) introduces multiple oracles. The invariant still holds: cargo-cicd never adjudicates — it collects N verdicts from N external oracles and presents the aggregate. The aggregate is still external to cargo-cicd's own logic.

```rust
// Phase 2: ThresholdOracle collects verdicts from multiple independent oracles
// cargo-cicd's role: emit evidence, submit to ThresholdOracle, record aggregate
// cargo-cicd's role is NOT: decide whether the aggregate means "pass"
// The ThresholdOracle decides that; cargo-cicd records the ThresholdVerdict.
struct ThresholdVerdict {
    required: usize,
    accepts: usize,
    refuses: usize,
    blocked: usize,
    consensus: ThresholdConsensus,  // Set by ThresholdOracle, not cargo-cicd
}
```

---

## Consequences

### Positive

1. **Conflict of interest elimination**: cargo-cicd cannot manipulate its own certification outcome. The evidence and the verdict are produced by independent systems.

2. **Auditability**: Third parties can run `wpm audit <evidence.xes>` against archived evidence to reproduce verdicts. No cargo-cicd binary is needed for post-hoc audit.

3. **Regulatory compliance path**: Separation of emitter (cargo-cicd) and adjudicator (wasm4pm) satisfies the "independent verification" requirement of DO-178C, FDA 21 CFR Part 11, and similar frameworks. See `docs/PHASE-3-DESIGN.md`.

4. **Oracle substitution**: Organizations with their own compliance requirements can substitute a different oracle (their internal certification tool) as long as it implements the same CLI interface. cargo-cicd's evidence emission doesn't change.

5. **Trust hierarchy clarity**: Users know exactly what cargo-cicd claims vs. what is independently verified. The distinction is explicit in every ProcessEvent.

6. **Ecosystem scalability**: As more tools adopt the XES + oracle pattern, a shared trust infrastructure emerges. cargo-cicd evidence can be adjudicated by any compatible oracle.

### Negative

1. **wpm dependency for release**: Any release gate that requires oracle adjudication requires the wpm binary. Teams without wpm installed get `Blocked` verdicts rather than `Accept`. Mitigation: `Blocked` is a first-class verdict; local development can proceed with it; full certification requires wpm.

2. **Increased test complexity**: Tests must explicitly handle three verdict states (Accept, Refuse, Blocked) rather than simple pass/fail. Mitigation: `ExpectedWpmVerdict` enum documents intent clearly.

3. **Process latency**: Every oracle adjudication requires a subprocess invocation (wpm binary). For high-frequency command execution, this adds 50-200ms per command. Mitigation: Oracle invocation is gated behind the `wasm4pm` feature flag; development builds skip it.

4. **Oracle version management**: The wpm binary version must be tracked alongside cargo-cicd versions. A wpm upgrade might change adjudication behavior for identical evidence. Mitigation: Oracle command and version are recorded in the receipt.

### Neutral

- The `verdict_claimed` field in ProcessEvent is not meaningless — it records cargo-cicd's self-assessment, which is audited against the oracle verdict. Persistent divergence between `verdict_claimed` and oracle verdict would indicate a systematic bias in cargo-cicd's self-assessment.

---

## Relationship to Other ADRs

| ADR | Relationship |
|-----|-------------|
| ADR-011 (XES v2 Format) | XES is the format submitted to the oracle |
| ADR-013 (Oracle Public Key Embedding) | Oracle identity is embedded in evidence traces |
| ADR-015 (JSONL Companion) | JSONL is not submitted to oracle; XES is authoritative |
| ADR-016 (Distributed Oracle Consensus) | M-of-N extends this invariant to multiple oracles |

---

## Invariants Summary

| Code | Invariant | Enforcement |
|------|-----------|-------------|
| E1 | cargo-cicd never adjudicates itself | Code structure, test suite |
| E2 | XES file must exist before oracle invocation | Precondition check in Wasm4pmShell |
| E3 | If oracle unavailable and verdict expected, panic | Runtime check |
| E4 | Tests assert oracle verdict, never claimed verdict | Test suite invariant |
| E7 | `Blocked` is first-class, not an error | WpmVerdict enum |

---

## References

- cargo-cicd invariants: `src/evidence.rs` (comments E1-E7)
- Oracle integration: `src/integrations/wasm4pm_shell.rs`
- Test assertions: `tests/wasm4pm_evidence_gate.rs`
- DO-178C, Section 12: Software Quality Assurance
- FDA 21 CFR Part 11: Electronic Records; Electronic Signatures

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial draft for Phase 1 Weeks 9-12 |
