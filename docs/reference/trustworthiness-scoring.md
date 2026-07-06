# Trustworthiness Scoring

This document explains the trustworthiness score computed for a crate's `[evidence]`
section in `Cargo.toml`.  The score is a single `f32` value in the range `[0.0, 1.0]`
that summarises how much independently verifiable evidence the crate's maintainer has
provided about its development process.

---

## What Is the Trustworthiness Score?

The trustworthiness score is a **composite metric** that answers the question:

> "How much evidence exists that this crate was produced under a controlled,
> independently-auditable process?"

It is NOT a measure of:

- The correctness or safety of the crate's code.
- The absence of security vulnerabilities.
- The quality of the crate's public API.

Those concerns are addressed by `cargo audit`, fuzzing, formal verification, and
peer review.  The trustworthiness score addresses a different, complementary concern:
the *process* that produced the crate.

---

## Score Computation

The score is computed by the following formula, implemented in
`src/evidence_manifest.rs::compute_trustworthiness()`:

```
score = 0.0

if receipt_hash is present and non-empty:
    score += 0.4

if archive_url is present and non-empty:
    score += 0.2

if oracle_key is present and non-empty:
    score += 0.2

for each standard in standards_satisfied (counting at most 2):
    score += 0.1

score = clamp(score, 0.0, 1.0)
```

### Factor Weights and Rationale

| Factor | Weight | Rationale |
|---|---|---|
| Receipt hash | 0.4 | The receipt is the primary artifact of adjudication. Without it there is no record of external review. |
| Archive URL | 0.2 | Enables consumers to download and inspect the full evidence trace independently. |
| Oracle key | 0.2 | Enables consumers to cryptographically verify that the receipt was issued by a trusted oracle. |
| Standards (first) | 0.1 | Demonstrates the process was evaluated against a recognized external standard. |
| Standards (second) | 0.1 | Additional standards signal multi-domain compliance. Only two are counted to prevent gaming. |

### Why a Maximum of Two Standards?

Counting more than two standards at 0.1 each would allow a crate to reach score 1.0
purely by listing many low-bar standards, without providing a receipt, archive, or
oracle key.  The cap ensures that the core evidence fields (receipt + archive +
oracle) dominate the score.

---

## Score Interpretation

### Score Bands

| Band | Range | Label | Meaning |
|---|---|---|---|
| 1 | 0.00–0.30 | **Unverified** | No evidence linked. The crate may have been produced under a sound process, but there is no way to verify it. |
| 2 | 0.30–0.70 | **Partial** | Some evidence fields are present, but the picture is incomplete. Consumers should investigate before relying on regulated-environment requirements. |
| 3 | 0.70–0.90 | **Verified** | Receipt, archive, and oracle key are all present. The evidence can be independently verified. |
| 4 | 0.90–1.00 | **Certified** | All evidence fields are present and at least two recognized standards are satisfied. Suitable for use in regulated environments that accept this oracle. |

### Example Scores

| Scenario | Score |
|---|---|
| No `[evidence]` section | 0.00 |
| Only `receipt_hash` | 0.40 |
| `receipt_hash` + `archive_url` | 0.60 |
| `receipt_hash` + `archive_url` + `oracle_key` | 0.80 |
| All fields + one standard | 0.90 |
| All fields + two or more standards | 1.00 |

---

## How to Improve Your Score

### Step 1: Run the CI/CD Pipeline and Get a Receipt (score: 0.00 → 0.40)

Run your full CI/CD pipeline under cargo-cicd and have it adjudicated by wasm4pm:

```sh
# Run the full pipeline:
cargo cicd pipeline run

# Adjudicate the evidence:
cargo cicd evidence audit

# The receipt is now at:
ls target/cargo-cicd/evidence/receipts/latest.json
```

Generate the `[evidence]` block with the receipt:

```sh
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json
```

This gives you a score of **0.40**.

### Step 2: Archive and Host the Evidence (score: 0.40 → 0.60)

Create and upload the evidence archive:

```sh
tar czf evidence.tar.gz target/cargo-cicd/evidence/

# Upload to your hosting provider:
aws s3 cp evidence.tar.gz \
    s3://my-evidence-bucket/my-crate/1.0.0/evidence.tar.gz
```

Add `--evidence-dir` to your generation command:

```sh
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence
```

This gives you a score of **0.60**.

### Step 3: Include the Oracle Key (score: 0.60 → 0.80)

Export the wasm4pm oracle's public key:

```sh
wpm pubkey --base64 > oracle-pubkey.b64
```

Add `--oracle-key` to your generation command:

```sh
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence \
    --oracle-key "$(cat oracle-pubkey.b64)"
```

This gives you a score of **0.80**.

### Step 4: Declare Standards Satisfied (score: 0.80 → 1.00)

If your process has been evaluated against recognized standards, declare them:

```sh
cicd-evidence-gen my-crate 1.0.0 \
    --receipt target/cargo-cicd/evidence/receipts/latest.json \
    --evidence-dir target/cargo-cicd/evidence \
    --oracle-key "$(cat oracle-pubkey.b64)" \
    --standard "IEC 61508 SIL 2" \
    --standard "ISO 26262 ASIL B" \
    --append Cargo.toml
```

Two standards give you a score of **1.00**.

---

## How Scores Are Displayed

### In `cargo cicd publish run`

```
Publish gate
  [PASS] Cargo.toml has required fields
  [PASS] Git state is clean
  [INFO] Trustworthiness score: 0.80
         receipt_hash: sha256:7d8f3c2a...
         archive_url:  https://evidence.my-org.com/my-crate/1.0.0/...
         oracle_key:   MCowBQY...
         standards:    (none declared — add --standard to reach 0.90+)
```

### In `cargo tree --evidence` (proposed)

```
my-project v0.1.0
├── anyhow v1.0.75 [✓ 0.80]
├── serde v1.0.200 [✓ 0.90 | IEC 61508 SIL 2]
├── toml v0.8.0 [? 0.40]
└── walkdir v2.5.0 [✗ 0.00]
```

Plain-text (non-TTY) output:

```
my-project v0.1.0
|- anyhow v1.0.75 [VERIFIED 0.80]
|- serde v1.0.200 [VERIFIED 0.90 | IEC 61508 SIL 2]
|- toml v0.8.0 [PARTIAL 0.40]
`- walkdir v2.5.0 [UNVERIFIED 0.00]
```

### In `cargo evidence check`

```sh
cargo evidence check --min-score 0.7
```

```
Checking evidence for 42 dependencies...
  ✓ serde@1.0.200        score=0.90  >= 0.70
  ✓ anyhow@1.0.75        score=0.80  >= 0.70
  ✗ walkdir@2.5.0        score=0.00  BELOW minimum 0.70

1 dependency below minimum score 0.70.
Run 'cargo evidence fetch walkdir@2.5.0' to investigate.
```

---

## Frequently Asked Questions

### Q: Is the trustworthiness score cryptographically verifiable?

The score stored in `[evidence]` is **self-reported** by the crate maintainer.  A
consumer must recompute it from the other fields to verify it.  The score computation
function is deterministic and open: see `src/evidence_manifest.rs::compute_trustworthiness()`.

The underlying evidence (receipt hash, archive URL, oracle key) *can* be
cryptographically verified:

- `receipt_hash` lets you verify the receipt file has not been tampered with.
- `oracle_key` lets you verify the receipt was signed by the declared oracle.

### Q: Can I trust a score of 1.0?

A score of 1.0 means all evidence fields are present and at least two standards are
declared.  It does NOT mean:

- The oracle has verified the evidence in the archive at `archive_url`.
- The standards listed have been independently confirmed by the standards body.
- The crate code is correct or safe.

For high-assurance use cases, download the archive and independently verify it
against the oracle using `cargo evidence verify`.

### Q: What if the archive URL is unavailable?

A score computed from a manifest with `archive_url` set is still 0.60 even if the
URL is currently unreachable.  The score reflects *declared* evidence, not
*verified* evidence.  Consumers who need to verify should check URL reachability
separately.

### Q: How often should I re-generate the evidence?

Evidence should be regenerated on every release.  If your crate releases frequently,
consider automating evidence generation as part of your CI/CD pipeline.  The
`cicd-evidence-gen` binary can be called from a GitHub Actions workflow step after
successful adjudication.

### Q: Do all my dependencies need a high score?

That depends on your requirements.  For most projects, there are no score
requirements.  For safety-critical or regulated-environment projects, you should
define a minimum score policy (e.g., all direct dependencies must have score >= 0.7)
and enforce it with `cargo evidence check --min-score 0.7`.

### Q: What does score 0.0 mean for crates that predate this RFC?

A score of 0.0 (no `[evidence]` section) simply means the crate has not yet
participated in the Vision 2030 evidence initiative.  It says nothing about the
crate's actual quality.  Most high-quality crates in the ecosystem will have a score
of 0.0 until their maintainers add the `[evidence]` section.

### Q: Can I add `[evidence]` to a crate I didn't publish?

No.  The `[evidence]` section is embedded in the crate's `Cargo.toml` and is
immutable for a given published version.  Only the crate maintainer can add it,
by publishing a new version with the section included.

### Q: What is the minimum useful score?

For a score to be independently verifiable by a consumer, all three core fields
must be present: `receipt_hash`, `archive_url`, and `oracle_key`.  This gives a
score of **0.80**.  A score below 0.80 means some verification steps are not possible.

### Q: Does `cargo publish` validate the score?

Not by default.  The `--require-evidence` flag (proposed in the RFC) would validate
the presence of `[evidence]` and the `receipt_hash` format.  In the current
implementation, the `[evidence]` section is informational and not validated by
`cargo publish`.

### Q: How is `trustworthiness_score` stored in `Cargo.toml`?

As a TOML float with one decimal of precision.  The `cicd-evidence-gen` binary
writes it as `trustworthiness_score = 0.9` (one decimal place) to avoid TOML
floating-point representation noise.

---

## Implementation Reference

| Item | Location |
|---|---|
| Score computation | `src/evidence_manifest.rs::compute_trustworthiness()` |
| Manifest struct | `src/evidence_manifest.rs::EvidenceManifest` |
| TOML block generation | `src/evidence_manifest.rs::EvidenceManifest::to_toml_block()` |
| CLI tool | `src/bin/cicd-evidence-gen.rs` |
| Integration tests | `tests/evidence_manifest.rs` |
| RFC | `docs/CARGO-EVIDENCE-RFC.md` |
| Ontology schema | `docs/ontology-registry-schema.ttl` |
