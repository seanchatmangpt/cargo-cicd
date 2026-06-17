# ADR-014: Propose [evidence] Section for Cargo.toml (RFC Process)

**Status:** Proposed (Awaiting Cargo RFC Review)  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, proposed to Cargo WG  
**Tags:** cargo, toml, evidence, metadata, ecosystem, rfc

---

## Context

cargo-cicd emits process evidence (XES files) that is adjudicated by the wasm4pm oracle. The oracle produces receipts (JSON files in `receipts/`). Currently, this evidence and receipt metadata lives entirely outside Cargo's package metadata system. It is managed by cargo-cicd's own `cicd.toml` carrier.

This creates a fundamental discoverability problem: when a crate is published to crates.io, there is no standard place to declare:

1. **That this crate uses process evidence**: Other tools (IDEs, CI systems, dependency auditors) cannot discover that a crate participates in the cargo-cicd ecosystem.
2. **Which oracle adjudicated the last release**: Downstream consumers who care about supply chain integrity cannot verify the adjudication without cargo-cicd-specific tooling.
3. **Certification claims**: If a crate declares compliance with a process standard (SLSA Level 3, NIST SP 800-218), there is no standard place to record this claim with a verifiable receipt.
4. **Minimum oracle version**: A crate may require that downstream users use a minimum wpm version to verify its evidence. This cannot be expressed in `Cargo.toml` today.

### Existing Workaround: `package.metadata.evidence`

Cargo.toml's `[package.metadata]` table is an unofficial extension mechanism:

```toml
[package.metadata.evidence]
oracle = "wasm4pm"
oracle_version = ">=0.9"
last_adjudicated_at = "2026-06-17T14:00:00Z"
last_verdict = "Accept"
receipt_path = "receipts/publish-2026-06-17.json"
```

**Pros**: Works today; no Cargo RFC needed; cargo does not validate `package.metadata` content.

**Cons**:
- Unofficial: `package.metadata` is a black box to Cargo. No tooling automatically processes it.
- No schema validation: Any tool can write any keys; interoperability is coincidental.
- No `cargo publish` integration: Cargo does not read `package.metadata.evidence` during publish.
- No crates.io exposure: `package.metadata` is not indexed or searchable on crates.io.
- No badge generation: crates.io cannot display an "evidence-adjudicated" badge based on `package.metadata`.

### Vision 2030 Requirement

For cargo-cicd to achieve 50%+ crates.io adoption by 2030, evidence metadata must be:

1. **Standardized** in Cargo's own schema (recognized by `cargo publish`, `cargo metadata`, `cargo audit`).
2. **Visible on crates.io** as a structured metadata field, enabling badge display and search filtering.
3. **Queryable** by `cargo tree` for supply chain verification (`cargo tree VERIFIED` badge, see `docs/PHASE-2-DESIGN.md`).
4. **Validated** by `cargo publish` (at minimum, syntactic validation of the oracle fingerprint format).

---

## Decision

**Propose a `[evidence]` top-level section in Cargo.toml through the official Cargo RFC process.**

This is a proposed decision — it requires Cargo WG acceptance, which cargo-cicd does not control. The ADR documents the proposal and rationale; actual implementation depends on RFC outcome.

### Proposed Cargo.toml Schema

```toml
[package]
name = "my-crate"
version = "1.2.3"
# ...

# Proposed [evidence] section
[evidence]
# Oracle that adjudicates process evidence for this crate
oracle = "wasm4pm"

# Minimum oracle version required to verify evidence
oracle_version = ">=0.9"

# Fingerprint of the oracle's public key (see ADR-013)
oracle_key_fingerprint = "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0="

# Timestamp of last adjudicated release
last_adjudicated_at = "2026-06-17T14:00:00Z"

# Verdict from last adjudication
last_verdict = "Accept"

# Path to receipt file (relative to workspace root)
receipt_path = "receipts/publish-20260617T140000Z.json"

# Process standards this crate claims conformance with
standards = ["SLSA-L3", "NIST-SP-800-218"]

# Evidence format version
evidence_format = "xes/2.0"
```

### RFC Strategy

The Cargo RFC (Request for Comments) process is documented at https://github.com/rust-lang/rfcs. The cargo-cicd proposal will:

1. **Scope narrowly**: Propose only `[evidence.oracle]`, `[evidence.oracle_version]`, and `[evidence.last_verdict]` in the initial RFC. More fields can be added in follow-up RFCs.

2. **Show ecosystem demand**: Document that cargo-cicd, cargo-audit, and supply chain security tooling all need a standard evidence metadata location.

3. **Propose crates.io badge**: Evidence-adjudicated crates would display a badge on crates.io, incentivizing adoption.

4. **Align with SLSA**: The Supply-chain Levels for Software Artifacts (SLSA) framework from Google/CNCF is gaining traction. An `[evidence]` section aligns cargo with SLSA provenance attestation.

### Fallback: `package.metadata.evidence` (Interim Standard)

Until the Cargo RFC is accepted (or if it is rejected), cargo-cicd will document and use `package.metadata.evidence` as an interim standard. The schema is identical to the proposed `[evidence]` section.

```toml
[package.metadata.evidence]
oracle = "wasm4pm"
oracle_version = ">=0.9"
oracle_key_fingerprint = "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0="
last_adjudicated_at = "2026-06-17T14:00:00Z"
last_verdict = "Accept"
receipt_path = "receipts/publish-20260617T140000Z.json"
standards = ["SLSA-L3"]
evidence_format = "xes/2.0"
```

cargo-cicd will parse both `[evidence]` (if accepted) and `[package.metadata.evidence]` (interim), with `[evidence]` taking precedence.

### cargo-cicd Implementation

Regardless of RFC outcome, cargo-cicd will:

1. **Write `[package.metadata.evidence]`** when `cargo cicd publish run` completes with an adjudicated receipt.
2. **Read `[package.metadata.evidence]`** when `cargo cicd workspace doctor` checks evidence status.
3. **Display evidence status** in `cargo cicd status show` output.
4. **Migrate automatically** if `[evidence]` section becomes official Cargo schema.

```rust
// src/adapters/manifest_parser.rs
pub struct EvidenceMetadata {
    pub oracle: Option<String>,
    pub oracle_version: Option<String>,
    pub oracle_key_fingerprint: Option<String>,
    pub last_adjudicated_at: Option<String>,
    pub last_verdict: Option<String>,
    pub receipt_path: Option<String>,
    pub standards: Vec<String>,
    pub evidence_format: Option<String>,
}

impl ManifestParser {
    /// Read evidence metadata from Cargo.toml.
    /// Checks [evidence] first, then [package.metadata.evidence].
    pub fn evidence_metadata(manifest_path: &Path) -> EvidenceMetadata {
        // ... parse both locations, prefer [evidence] ...
    }
}
```

---

## Alternatives Considered

### Alternative A: Custom cicd.toml Section

Keep all evidence metadata in `cicd.toml` and never involve `Cargo.toml`.

**Pros**: No Cargo RFC needed; cargo-cicd fully controls the schema.

**Cons**: Evidence metadata is not visible at package publish time; not discoverable via `cargo metadata`; requires separate tooling to inspect.

**Rejection reason**: Does not achieve ecosystem-level discoverability. Vision 2030 requires crates.io integration.

### Alternative B: Separate `evidence.toml` File

Introduce a new `evidence.toml` file at the workspace root:

```toml
# evidence.toml
[publish]
oracle = "wasm4pm"
last_verdict = "Accept"
receipt_path = "receipts/..."
```

**Pros**: No Cargo.toml schema change needed; cargo-cicd fully controls the format.

**Cons**: Another file for developers to manage; not linked to the Cargo ecosystem; same discoverability problem as cicd.toml.

**Rejection reason**: Proliferating configuration files is an anti-pattern. Cargo.toml is the authoritative source of package metadata.

### Alternative C: Cargo Audit Integration

Extend `cargo-audit` (the existing security audit tool) to carry evidence metadata rather than proposing a Cargo.toml change.

**Pros**: cargo-audit already has crates.io integration and community trust.

**Cons**: cargo-audit is focused on vulnerability advisories, not process evidence. The mental model doesn't fit.

**Rejection reason**: Scope mismatch. Process conformance evidence is distinct from vulnerability audit results.

### Alternative D: SLSA Attestation Format (Adopted for Standards Claims)

Use the SLSA attestation format (an in-toto attestation) for standards claims and reference it from `package.metadata.evidence`:

```toml
[package.metadata.evidence]
slsa_attestation_path = "attestations/slsa-l3-20260617.json"
oracle = "wasm4pm"
```

**Status**: This hybrid approach is recommended alongside the `[evidence]` proposal. SLSA attestations are machine-readable and verifiable. cargo-cicd can generate SLSA attestations for high-assurance releases.

---

## RFC Draft Outline

The Cargo RFC draft (to be submitted to https://github.com/rust-lang/rfcs) will contain:

### Summary
Add a `[evidence]` top-level table to Cargo.toml for declaring process evidence metadata, enabling crates to express oracle-adjudicated quality certifications.

### Motivation
- Supply chain security requires verifiable build provenance
- Process conformance (SLSA, NIST SP 800-218) requires evidence metadata
- No current standard place for evidence metadata in the Cargo ecosystem

### Guide-Level Explanation
Developers add `[evidence]` to `Cargo.toml` after running `cargo cicd publish run`:
```toml
[evidence]
oracle = "wasm4pm"
last_verdict = "Accept"
```

### Reference-Level Explanation
Formal TOML schema with types, constraints, and cargo-validate integration.

### Drawbacks
- Adds complexity to Cargo.toml
- Oracle ecosystem is early-stage; may need schema revisions

### Rationale
- Aligns with SLSA provenance framework
- Enables crates.io badge display
- Provides standard discovery mechanism for ecosystem tooling

### Prior Art
- `pyproject.toml` [tool.*] sections in Python packaging
- `package.json` fields in npm (homepage, funding, etc.)
- SLSA attestation format from Google

---

## Success Criteria

| Milestone | Criteria |
|-----------|----------|
| RFC Submitted | Cargo RFC filed on GitHub with ≥50 community thumbs-up |
| RFC Accepted | Cargo WG accepts RFC into tracking issue |
| Implementation | `cargo metadata` outputs `evidence` section |
| crates.io | crates.io displays "evidence-adjudicated" badge for qualifying crates |
| Adoption | ≥100 crates published with `[evidence]` or `[package.metadata.evidence]` |

---

## References

- Cargo RFC process: https://github.com/rust-lang/rfcs
- SLSA provenance: https://slsa.dev/provenance/
- in-toto attestation: https://in-toto.io/
- NIST SP 800-218: Secure Software Development Framework (SSDF)
- `cargo audit`: https://github.com/rustsec/rustsec/tree/main/cargo-audit

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial proposal draft |
