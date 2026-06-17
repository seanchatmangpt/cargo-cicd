# RFC: Cargo.toml `[evidence]` Section for Supply-Chain Process Evidence

- **Feature Name:** `cargo-evidence-section`
- **Start Date:** 2026-06-17
- **RFC PR:** (to be filed at <https://github.com/rust-lang/rfcs>)
- **Cargo Issue:** (to be filed at <https://github.com/rust-lang/cargo>)
- **Author:** cargo-cicd project, Vision 2030 initiative
- **Status:** Draft

---

## Summary

This RFC proposes a new optional `[evidence]` section in `Cargo.toml` that records
process-evidence metadata for a published crate.  The section links the published
artifact to a cryptographically verifiable evidence archive, an oracle public key,
and a receipt hash so that downstream consumers of the crate can assess the
trustworthiness of the CI/CD process that produced it.

---

## Motivation

### The Supply-Chain Verification Gap

The Rust ecosystem has excellent tooling for auditing declared vulnerabilities
(`cargo audit`), license compliance, and dependency graphs.  What it lacks is any
way to answer the question:

> "Was this crate produced under a process that I can verify?"

Package consumers who operate in regulated environments — automotive (ISO 26262),
industrial safety (IEC 61508), aviation (DO-178C), or medical devices (IEC 62304) —
must be able to demonstrate that the software they incorporate was developed under a
controlled, auditable process.  Today they have no machine-readable way to get this
information from the Rust package registry.

### Why Cargo.toml?

`Cargo.toml` is the canonical, human-readable, version-controlled manifest for a
Rust crate.  It is:

1. **Always present** — every published crate has one.
2. **Versioned with the crate** — immutable for a given crate version.
3. **Parsed by `cargo metadata`** — existing tooling can be extended to surface it.
4. **Indexed by crates.io** — available through the standard registry API.
5. **Familiar to every Rust developer** — no new file format required.

Placing evidence metadata in `Cargo.toml` makes it a first-class attribute of the
crate's public identity, not an afterthought stored in a separate file.

### Prior Art Gap

Other ecosystems have tackled adjacent problems:

- **npm audit** — audits declared vulnerability advisories but does not link to
  CI/CD process evidence.
- **PyPI Provenance (PEP 740)** — links packages to GitHub Actions attestations but
  is scoped to build provenance, not process conformance.
- **Go Module Sum Database** — provides content hashing of modules but does not
  capture process compliance.
- **Sigstore / SLSA** — provides attestation of the build process but focuses on the
  build provenance rather than the broader development process (code review, testing
  policy, change management, etc.).

None of these systems provides a structured way to declare that a package was
produced under a specific, externally-adjudicated process model.  This RFC fills
that gap for the Rust ecosystem.

### Use Cases

**Use Case 1: Regulated-environment dependency selection**

An automotive OEM's software team is evaluating Rust crates for use in an ISO 26262
ASIL-B product.  They want to run `cargo tree --evidence` to see, at a glance,
which of their dependencies have been adjudicated against ASIL-B.  Today this
requires manual investigation of each crate's CI/CD documentation.

**Use Case 2: Automated supply-chain policy enforcement**

A CI/CD pipeline enforces the policy: "all direct dependencies must have a
trustworthiness score >= 0.7".  A new tool, `cargo evidence check`, can evaluate
this policy automatically by reading the `[evidence]` sections of declared
dependencies.

**Use Case 3: Compliance audit traceability**

An auditor reviewing a medical device submission needs to trace each Rust dependency
back to its evidence archive.  With `[evidence]` present in `Cargo.toml`, the
`archive_url` provides a direct link to the evidence tar.gz that can be inspected,
and the `receipt_hash` verifies that the archive has not been tampered with.

**Use Case 4: IDE integration**

A developer's IDE shows a per-dependency trustworthiness badge in the `Cargo.toml`
editor, sourced from the `trustworthiness_score` field of each dependency's
`Cargo.toml`.  Low-scoring dependencies are highlighted for review.

---

## Guide-Level Explanation

### For Crate Maintainers: Adding `[evidence]` to Your `Cargo.toml`

After running your CI/CD pipeline and having the run adjudicated by a process oracle
(e.g., [wasm4pm](https://github.com/wasm4pm/wasm4pm)), you can add an `[evidence]`
section to your `Cargo.toml`:

```toml
[package]
name = "my-crate"
version = "1.0.0"
edition = "2021"
# ... other standard fields ...

[evidence]
version = "1.0"
archive_url = "https://evidence.my-org.com/my-crate/1.0.0/evidence.tar.gz"
oracle_key = "MCowBQYDK2VdAyEAyz..."
receipt_hash = "sha256:4f53c0e7f7c5b9d8..."
timestamp = "2026-06-17T14:30:00.000Z"
trustworthiness_score = 0.9
standards_satisfied = ["IEC 61508 SIL 2"]
```

The `cargo-cicd` tool can generate this block automatically after a successful
`publish run` with wasm4pm adjudication:

```sh
# Generate the [evidence] block and print it:
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence \
    --oracle-key "$(wpm pubkey --base64)" \
    --standard "IEC 61508 SIL 2"

# Or append directly to Cargo.toml:
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence \
    --oracle-key "$(wpm pubkey --base64)" \
    --append Cargo.toml
```

### For Package Consumers: Checking Evidence

With this RFC implemented in Cargo, a consumer can:

```sh
# Show evidence status for all dependencies:
cargo tree --evidence

# Check that all dependencies meet a minimum trustworthiness threshold:
cargo evidence check --min-score 0.7

# Fetch and verify the evidence archive for a specific dependency:
cargo evidence fetch serde@1.0.200
cargo evidence verify serde@1.0.200 --oracle-key <pubkey>
```

### Trustworthiness Score Quick Reference

| Score Range | Interpretation |
|---|---|
| 0.0–0.3 | Unverified — no evidence linked |
| 0.3–0.7 | Partial — some evidence fields present |
| 0.7–0.9 | Verified — receipt, archive, and oracle key all linked |
| 0.9–1.0 | Certified — all fields present, one or more standards satisfied |

---

## Reference-Level Explanation

### `[evidence]` Section Schema

The `[evidence]` section is a single TOML table (not an array table).  It appears
at the top level of `Cargo.toml`, alongside `[package]`, `[dependencies]`, etc.

#### Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `version` | String | Yes | Schema version.  Must be `"1.0"` for this RFC. |
| `archive_url` | String | No | HTTPS URL of the evidence archive (`*.tar.gz`). |
| `oracle_key` | String | No | Base64-encoded DER public key of the adjudication oracle. |
| `receipt_hash` | String | No | `sha256:<hex>` hash of the wpm receipt JSON file. |
| `timestamp` | String | No | ISO-8601 UTC timestamp of adjudication. |
| `trustworthiness_score` | Float | No | Composite score in `[0.0, 1.0]`. |
| `standards_satisfied` | Array of Strings | No | Standards the crate was adjudicated against. |

#### Formal Grammar (TOML-compatible)

```toml
[evidence]
# Required:
version = "<semver-like string, currently '1.0'>"

# Optional:
archive_url = "<https-url>"
oracle_key = "<base64-der-pubkey>"
receipt_hash = "sha256:<64-hex-chars>"
timestamp = "<RFC-3339 UTC timestamp>"
trustworthiness_score = <float in [0.0, 1.0]>
standards_satisfied = ["<standard-name>", ...]
```

#### Schema Version

The `version` field identifies the schema version of this evidence block.  For the
initial release of this RFC the only valid value is `"1.0"`.  Future schema versions
will be additive; Cargo must warn but not error on unknown fields when `version` is
`"1.0"`.  A higher `version` string must cause a `cargo` warning if the version is
not recognized.

#### `archive_url`

The URL where the evidence archive can be fetched.  Requirements:

- Must use the HTTPS scheme.
- The archive at this URL must be a gzip-compressed tar archive (`*.tar.gz`).
- The archive must contain a valid XES event log (`events.xes`) and a wasm4pm
  receipt (`receipts/latest.json`).
- The URL must remain stable for the lifetime of the crate version.

Cargo does not fetch this URL during normal operations.  It is fetched only by
`cargo evidence fetch`.

#### `oracle_key`

The base64-encoded DER-format public key of the process oracle that adjudicated the
evidence.  This field enables consumers to verify that a given evidence archive was
adjudicated by a trusted oracle without having to trust the evidence archive URL.

The oracle key is distributed out-of-band (e.g., in the oracle's published
documentation or its own Cargo.toml `[evidence]` section).

#### `receipt_hash`

A `sha256:<hex>` hash of the wasm4pm receipt JSON file that was produced during
adjudication.  This allows consumers to verify the receipt's integrity after
downloading the archive from `archive_url`:

```sh
sha256sum receipt.json
# Compare with receipt_hash field
```

#### `timestamp`

An RFC-3339 UTC timestamp recording when adjudication was performed.  Intended to
help consumers assess evidence freshness.  Not cryptographically bound to the
evidence archive.

#### `trustworthiness_score`

A single-precision float in `[0.0, 1.0]`.  Computed by a scoring function (see
Trustworthiness Score Specification below) and stored here for fast display without
downloading the full archive.

**Security note:** This field is self-reported by the crate maintainer.  Consumers
who require strong guarantees must recompute the score from the evidence archive
rather than trusting this field.

#### `standards_satisfied`

An array of human-readable standard names that the crate's development process was
adjudicated against.  These strings are not normalized; they are informational.
Canonical identifiers are:

- `"IEC 61508 SIL 1"`, `"IEC 61508 SIL 2"`, `"IEC 61508 SIL 3"`, `"IEC 61508 SIL 4"`
- `"ISO 26262 ASIL A"`, `"ISO 26262 ASIL B"`, `"ISO 26262 ASIL C"`, `"ISO 26262 ASIL D"`
- `"DO-178C Level A"`, `"DO-178C Level B"`, `"DO-178C Level C"`, `"DO-178C Level D"`
- `"IEC 62304 Class A"`, `"IEC 62304 Class B"`, `"IEC 62304 Class C"`
- `"NIST SP 800-218 PL1"`, `"NIST SP 800-218 PL2"`, `"NIST SP 800-218 PL3"`

Cargo does not validate these strings against any normative list.  Validation is the
responsibility of the oracle and any downstream tooling.

---

### Trustworthiness Score Specification

The `trustworthiness_score` field is computed by the following function.  The same
computation is used by `cargo evidence check` when validating that dependencies meet
a minimum score threshold.

```
score = 0.0
if receipt_hash is present:       score += 0.4
if archive_url is present:        score += 0.2
if oracle_key is present:         score += 0.2
for each standard in standards_satisfied (max 2 counted):
    score += 0.1
score = clamp(score, 0.0, 1.0)
```

The rationale for these weights:

- **Receipt hash (0.4):** The receipt is the primary evidence artifact.  Without it,
  there is no externally verifiable record of adjudication.
- **Archive URL (0.2):** Enables consumers to inspect the full evidence trace.
- **Oracle key (0.2):** Enables consumers to verify adjudication signatures.
- **Standards (0.1 each, max 0.2):** Standards provide additional quality signal
  but are capped to prevent gaming via a large list of low-bar standards.

---

### `cargo metadata` Output Changes

`cargo metadata` will include the evidence fields in the package metadata output
under a new `evidence` key:

```json
{
  "packages": [
    {
      "name": "my-crate",
      "version": "1.0.0",
      "evidence": {
        "version": "1.0",
        "archive_url": "https://evidence.my-org.com/my-crate/1.0.0/evidence.tar.gz",
        "oracle_key": "MCowBQYDK2VdAyEAyz...",
        "receipt_hash": "sha256:4f53c0e7f7c5b9d8...",
        "timestamp": "2026-06-17T14:30:00.000Z",
        "trustworthiness_score": 0.9,
        "standards_satisfied": ["IEC 61508 SIL 2"]
      }
    }
  ]
}
```

When no `[evidence]` section is present, the `evidence` key is `null`.

The `cargo metadata` JSON schema is extended with:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema",
  "definitions": {
    "Evidence": {
      "type": ["object", "null"],
      "properties": {
        "version": { "type": "string" },
        "archive_url": { "type": ["string", "null"] },
        "oracle_key": { "type": ["string", "null"] },
        "receipt_hash": { "type": ["string", "null"] },
        "timestamp": { "type": ["string", "null"] },
        "trustworthiness_score": { "type": ["number", "null"], "minimum": 0.0, "maximum": 1.0 },
        "standards_satisfied": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["version"]
    }
  }
}
```

---

### `cargo tree` Display Changes

`cargo tree` gains a `--evidence` flag that annotates each package line with its
evidence status:

```
my-project v0.1.0
├── anyhow v1.0.75 [✓ 0.80]
├── serde v1.0.200 [✓ 0.90 | IEC 61508 SIL 2]
├── toml v0.8.0 [? 0.40]
└── walkdir v2.5.0 [✗ 0.00]
```

Badge legend:

| Badge | Meaning |
|---|---|
| `[✓ <score>]` | `[evidence]` section present; score shown |
| `[? <score>]` | Partial evidence (some fields missing) |
| `[✗ 0.00]` | No `[evidence]` section |

When stdout is not a TTY, ASCII-safe equivalents are used: `[VERIFIED]`,
`[PARTIAL]`, `[UNVERIFIED]`.

---

### `cargo audit` Integration

`cargo audit` is extended to support evidence auditing:

```sh
# Check that all dependencies have evidence:
cargo audit evidence

# Check against a minimum score:
cargo audit evidence --min-score 0.7

# Check that specific standards are satisfied:
cargo audit evidence --require-standard "IEC 61508 SIL 2"
```

`cargo audit evidence` reads the `[evidence]` section from each dependency's
`Cargo.toml` in the local Cargo cache (`~/.cargo/registry/src/`).  It does not
fetch evidence archives unless `--verify` is passed.

Output format:

```
Scanning 42 packages for process evidence...

VERIFIED   (score >= 0.70)
  anyhow 1.0.75          score=0.80
  serde 1.0.200          score=0.90  standards=[IEC 61508 SIL 2]

PARTIAL    (0.30 <= score < 0.70)
  toml 0.8.0             score=0.40  missing=[oracle_key, standards]

UNVERIFIED (score < 0.30)
  walkdir 2.5.0          score=0.00  no [evidence] section

Summary: 2 verified, 1 partial, 1 unverified
```

---

### `cargo publish` Validation

`cargo publish` gains an optional `--require-evidence` flag.  When set:

1. Cargo checks that `[evidence]` is present in `Cargo.toml`.
2. Cargo validates that the `receipt_hash` field is present and correctly prefixed.
3. Cargo emits a warning (not an error) if `trustworthiness_score` < 0.7.

The flag is intentionally **not** enabled by default to preserve backwards
compatibility.  A future RFC or crates.io policy may require it for specific
categories (e.g., crates with safety-critical keywords in their descriptions).

crates.io registry API is extended to include the `evidence` field in package
metadata so it is available without downloading the crate.

---

## Drawbacks

### Additional Complexity in `Cargo.toml`

`Cargo.toml` is already complex.  Adding another table adds cognitive load for crate
authors who do not operate in regulated environments.  Mitigation: the section is
purely optional and invisible to authors who don't use it.

### Risk of Self-Reported Score Gaming

The `trustworthiness_score` field is self-reported by the crate maintainer.  A
malicious author could claim a score of 1.0 without any real evidence.  Mitigation:

1. The score is re-computable from the other fields (consumers can verify it).
2. The receipt oracle's public key (`oracle_key`) lets consumers verify the receipt
   signature independently.
3. crates.io could in the future run server-side score verification.

### Evidence Archive Availability

The `archive_url` depends on an external server that may become unavailable.
Mitigation: this is the same availability risk that affects documentation URLs,
homepage URLs, and repository URLs already in `Cargo.toml`.  The registry does not
follow external URLs by default.

### Namespace Collision

The `[evidence]` table name could conflict with future Cargo features.  Mitigation:
the name is descriptive and unlikely to collide; it can be reserved in Cargo's TOML
grammar.

---

## Rationale and Alternatives

### Why Not a Separate File?

A separate file (e.g., `evidence.toml`) was considered.  Rejected because:

1. It requires a new file format that developers must learn.
2. It is not automatically included in crates.io index metadata.
3. It breaks the single-source-of-truth principle of `Cargo.toml`.

### Why Not `.cargo/config.toml`?

`.cargo/config.toml` is workspace-level, not package-level.  Evidence metadata is
per-version, so it belongs in the per-version manifest.

### Why Not a Sidecar in the Registry?

A sidecar API endpoint (e.g., `crates.io/api/v1/crates/{name}/{version}/evidence`)
was considered.  This would decouple evidence from the crate manifest.  Rejected
for this RFC because it requires registry API changes and prevents offline
verification.  A future RFC could add this as a complementary channel.

### Why Not SLSA?

SLSA (Supply Chain Levels for Software Artifacts) addresses build provenance.  It
answers "was this binary built from this source by this CI system?"  The `[evidence]`
RFC addresses process conformance: "was this crate developed under a controlled,
auditable process?"  These are complementary, not competing, concerns.  A future
extension could embed SLSA provenance data in the `[evidence]` archive.

### Why Not Sigstore?

Sigstore enables keyless signing of artifacts.  It would be an appropriate transport
for the oracle signatures referenced by `oracle_key`.  The `[evidence]` RFC is
transport-agnostic: the `oracle_key` field supports any DER-encoded asymmetric key.
A Sigstore integration could be implemented as an extension of the oracle tooling
without changing this RFC.

---

## Prior Art

### npm provenance (2023)

npm added provenance attestations in 2023, linking published packages to GitHub
Actions workflow runs via Sigstore.  This proves "this package was built in GitHub
Actions" but does not address process quality or compliance with engineering
standards.

### PyPI PEP 740 — Attestations

PEP 740 (accepted 2024) adds a mechanism for PyPI packages to carry signed
attestations about their build environment.  Like npm provenance, it focuses on
build provenance rather than process conformance.

### Go Module Sum Database

The Go module sum database records content hashes of module versions, preventing
tampering.  It has no concept of process evidence or trustworthiness scoring.

### Maven Central's JAPI Compliance Checker

Maven Central requires that submitted artifacts pass a set of quality checks
(metadata completeness, POM validity).  This is a quality gate, not process evidence.

### Crates.io Safety-Critical Working Group

The Rust Foundation's Safety-Critical Rust Consortium is exploring how Rust can
be used in safety-critical domains.  The `[evidence]` RFC is directly aligned with
this initiative and would provide the infrastructure needed for safety-critical crate
authors to demonstrate process conformance.

---

## Unresolved Questions

### 1. Who Can Write `[evidence]`?

Should the `[evidence]` section be writable by anyone, or should it require oracle
signatures before crates.io accepts it?  The current proposal makes it self-reported
with optional verification.

### 2. Minimum Oracle Requirements

What oracles should be recognized?  The RFC currently treats `oracle_key` as an
opaque public key.  Should there be a registry of approved oracles?

### 3. Evidence Archive Format Stability

The evidence archive format (XES + wasm4pm receipt) is defined by wasm4pm.  Should
this RFC specify the archive format, or reference it as an external spec?

### 4. Score Versioning

If the trustworthiness scoring formula changes in a future RFC version, scores
computed under different formulas will be incompatible.  The `version` field provides
a hook for this, but the exact versioning strategy needs to be defined.

### 5. crates.io Index Integration

Should the `[evidence]` fields be included in the crates.io sparse registry index
(`.cargo/registry/index/`) so that `cargo` can read them without downloading the
full crate source?  This would enable fast local evidence checks.

### 6. Workspace-Level Evidence

Should there be a `[workspace.evidence]` table for multi-crate workspaces that
produces or adjudicates evidence at the workspace level rather than per-crate?

### 7. Mandatory vs. Optional for Safety-Critical Categories

Should crates.io require `[evidence]` for crates that self-declare safety-critical
categories?  This would add enforcement teeth but requires defining the category set.

### 8. Evidence Freshness

The `timestamp` field records when evidence was adjudicated, but there is no policy
on maximum evidence age.  Should `cargo evidence check` warn if evidence is older
than N months?

---

## Future Possibilities

### Evidence Archive Notarization

A future RFC could require that the evidence archive be notarized (timestamped by a
trusted third party such as a certificate transparency log) so that the adjudication
date can be cryptographically verified.

### Automated Evidence Refresh on Publish

`cargo publish` could automatically trigger an evidence collection run and oracle
adjudication as part of the publish workflow, filling in the `[evidence]` section
automatically.

### Federated Oracle Registry

A registry of approved process oracles, similar to certificate authorities in the
Web PKI, would allow consumers to trust oracle keys without independent verification.

### Evidence Inheritance

In a workspace where all crates share a common CI/CD process, a child crate could
inherit evidence from the workspace root rather than carrying a separate `[evidence]`
section.

### AI-Assisted Evidence Auditing

LLM-assisted tools could summarize the evidence archive for auditors, automatically
generating natural-language audit reports from the XES event log.

### Marketplace Integration

The crates.io web UI could display a trustworthiness score badge on each crate's
detail page, sourced from the `trustworthiness_score` field.

---

## Reference Implementation

The reference implementation is the `cargo-cicd` tool, available at:

```
https://crates.io/crates/cargo-cicd
```

Relevant source files:

- `src/evidence_manifest.rs` — `EvidenceManifest` struct, `build_manifest()`,
  `compute_trustworthiness()`, `dep_has_evidence()`, `format_evidence_status_table()`
- `src/bin/cicd-evidence-gen.rs` — CLI binary that generates the `[evidence]` block
- `tests/evidence_manifest.rs` — Comprehensive test suite (21 tests)
- `docs/trustworthiness-scoring.md` — Scoring documentation
- `docs/ontology-registry-schema.ttl` — RDF ontology for capability registry

The implementation uses only `std` for the core evidence manifest operations,
ensuring zero additional runtime dependencies for crate authors who import
`cargo-cicd` as a library.

---

## Appendix A: Full `Cargo.toml` Example

```toml
[package]
name = "safety-critical-driver"
version = "2.1.0"
edition = "2021"
description = "CAN bus driver for automotive ECUs"
license = "MIT OR Apache-2.0"
repository = "https://github.com/my-org/safety-critical-driver"
keywords = ["can", "automotive", "embedded"]
categories = ["embedded", "hardware-support"]

[dependencies]
embedded-hal = "1.0.0"
nb = "1.1.0"

[dev-dependencies]
defmt = "0.3"

[evidence]
version = "1.0"
archive_url = "https://evidence.my-org.com/safety-critical-driver/2.1.0/evidence.tar.gz"
oracle_key = "MCowBQYDK2VdAyEAyzXmW9Tnkr7q1UqVQnm2p4R6jw2LvHkMkNQ8pHx3lMs="
receipt_hash = "sha256:7d8f3c2a1b4e5f6a9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b"
timestamp = "2026-06-17T14:30:00.000Z"
trustworthiness_score = 0.9
standards_satisfied = ["ISO 26262 ASIL B", "IEC 61508 SIL 2"]
```

---

## Appendix B: cicd-evidence-gen Reference

```
cicd-evidence-gen <crate-name> <crate-version> [OPTIONS]

ARGUMENTS:
  <crate-name>      The crate name (must match Cargo.toml [package].name)
  <crate-version>   The crate version (must match Cargo.toml [package].version)

OPTIONS:
  --receipt <path>        Path to the wasm4pm receipt JSON file.
                          Used to compute receipt_hash.
  --evidence-dir <dir>    Path to the evidence output directory.
                          Used to derive archive_url.
  --oracle-key <base64>   Base64-encoded oracle public key.
  --standard <name>       Standard satisfied (can be repeated).
  --append <Cargo.toml>   Append the block to this file.
                          If [evidence] already exists, it is replaced.
  --check                 Validate and exit 1 if issues exist.

EXIT CODES:
  0   Success (or --check passed)
  1   --check failed (validation issues found)
  2   Argument parsing error

EXAMPLES:
  # Print [evidence] block to stdout:
  cicd-evidence-gen my-crate 1.0.0

  # Full generation with all fields:
  cicd-evidence-gen my-crate 1.0.0 \
      --receipt target/cargo-cicd/evidence/receipts/latest.json \
      --evidence-dir target/cargo-cicd/evidence \
      --oracle-key "$(wpm pubkey --base64)" \
      --standard "IEC 61508 SIL 2" \
      --standard "ISO 26262 ASIL B"

  # Append to Cargo.toml:
  cicd-evidence-gen my-crate 1.0.0 --receipt latest.json --append Cargo.toml

  # Validate existing manifest:
  cicd-evidence-gen my-crate 1.0.0 --check
```

---

## Appendix C: Worked Example — Generating and Verifying Evidence

This appendix walks through the end-to-end workflow for a crate maintainer adding
the `[evidence]` section to their `Cargo.toml`.

### Step 1: Run the CI/CD Pipeline

```sh
cargo cicd pipeline run
```

This emits process events to `target/cargo-cicd/evidence/events.xes` and
`target/cargo-cicd/evidence/receipts/latest.json`.

### Step 2: Adjudicate with wasm4pm

```sh
wpm audit target/cargo-cicd/evidence/events.xes
# Output: Accept
```

### Step 3: Archive Evidence

```sh
tar czf evidence.tar.gz target/cargo-cicd/evidence/
# Upload to your evidence archive host:
aws s3 cp evidence.tar.gz \
    s3://my-evidence-bucket/my-crate/1.0.0/evidence.tar.gz
```

### Step 4: Generate `[evidence]` Block

```sh
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence \
    --oracle-key "$(wpm pubkey --base64)" \
    --standard "IEC 61508 SIL 2" \
    --append Cargo.toml
```

### Step 5: Publish

```sh
cargo publish
```

### Step 6: Consumer Verification

A consumer of `my-crate` can verify the evidence:

```sh
# Check score:
cargo evidence check my-crate --min-score 0.7

# Download and verify archive:
cargo evidence fetch my-crate@1.0.0
cargo evidence verify my-crate@1.0.0 \
    --oracle-key "MCowBQYDK2VdAyEAyz..."
```

---

*This document is a draft RFC for discussion purposes.  It has not been submitted
to the Rust RFC process.  Feedback is welcome.*
