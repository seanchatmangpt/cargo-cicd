# Distributed M-of-N Oracle Consensus Protocol Design

**Document Type:** Technical Design  
**Status:** Proposed (Phase 2)  
**Date:** 2026-06-17  
**Audience:** cargo-cicd core engineers, security architects, integration partners  
**Companion ADR:** `docs/adr/ADR-016-distributed-oracle-consensus.md`

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [M-of-N Threshold Signature Scheme](#2-m-of-n-threshold-signature-scheme)
3. [Oracle Registry](#3-oracle-registry)
4. [Receipt Aggregation](#4-receipt-aggregation)
5. [Failure Modes and Recovery](#5-failure-modes-and-recovery)
6. [Phase 2 Implementation Plan](#6-phase-2-implementation-plan)

---

## 1. Problem Statement

### 1.1 Single Oracle Failure Modes

cargo-cicd's current architecture (Phase 1) relies on a single wasm4pm oracle (`wpm` binary) to adjudicate all process evidence. This creates a **single point of trust failure** with several distinct failure modes:

**Failure Mode 1: Oracle Unavailability**

The `wpm` binary may be unavailable due to:
- Network failure (if oracle is remote)
- Binary not installed on the CI runner
- Corrupt binary installation
- Oracle service outage
- Air-gapped environment

When the oracle is unavailable, adjudication returns `Blocked`. All evidence remains unadjudicated. High-assurance releases cannot proceed. Entire CI/CD pipelines stall.

Probability estimate: 0.1-1% per release attempt in typical CI environments. At 10 releases/day, this means 1-36 blocked releases per year.

**Failure Mode 2: Key Compromise**

If the oracle's signing private key is stolen:
- An attacker can forge `Accept` verdicts for malicious packages
- All historical `Accept` verdicts from the compromised key must be considered suspect
- Key rotation requires re-adjudicating all affected evidence
- There is no way to detect forged verdicts post-hoc without independent verification

This is a supply chain attack vector. A compromised oracle key is equivalent to a compromised Certificate Authority — the attacker can impersonate the oracle indefinitely until the key is revoked.

**Failure Mode 3: Oracle Vendor Lock-In**

With a single oracle vendor (wasm4pm):
- Teams cannot substitute an equivalent oracle from a different provider
- If wasm4pm is acquired, changes licensing, or is discontinued, cargo-cicd's evidence gate is broken
- Organizations with strict vendor diversity requirements cannot use cargo-cicd for regulated contexts
- A malicious oracle vendor could selectively issue `Refuse` verdicts to competitors

**Failure Mode 4: Conflict of Interest**

Even with architectural separation (ADR-012: cargo-cicd never adjudicates itself), if the same entity controls both cargo-cicd and wasm4pm:
- There is no independent verification party
- Regulatory frameworks that require "independent" verification may not be satisfied
- Trust in the ecosystem depends on a single organization's integrity

### 1.2 Regulatory Requirements for Independent Verification

Several regulatory frameworks explicitly require multiple independent verifiers:

**DO-178C (Airborne Software)**

DO-178C §6.3.3 (Independence) requires that for DAL A and DAL B software, certain verification activities must be performed by a party independent of the development organization. A single oracle controlled by one entity does not satisfy independence for safety-critical aviation software.

**ISO 26262 (Automotive Functional Safety)**

ISO 26262-8 §6.4.4 requires that for ASIL D components, independent safety analyses are performed. "Independent" means performed by a person not involved in development, using different methods and possibly different tools.

**IEC 62443-4-1 (Industrial Cybersecurity)**

IEC 62443-4-1 §12.2 (Independent review) requires that security design reviews be conducted by parties independent of the design team. For IACS (Industrial Automation and Control Systems), single-party verification is insufficient.

**NIST SP 800-218 (SSDF)**

NIST SP 800-218 PV.1.3 recommends: "Use multiple verification mechanisms to detect tampering or weaknesses in software." A single oracle is a single verification mechanism.

### 1.3 Supply Chain Attack Scenarios

**Attack Scenario A: Oracle Key Theft**

1. Attacker gains access to oracle signing key (via employee compromise, infrastructure breach, or side-channel attack).
2. Attacker creates malicious Rust package with backdoor.
3. Attacker uses stolen key to generate forged `Accept` receipt for malicious package.
4. Package is published to crates.io with a valid-appearing receipt.
5. Downstream projects that trust oracle receipts adopt the malicious package.

**Detection**: Impossible with single oracle. There is no second opinion.
**M-of-N mitigation**: Attacker must compromise M of N oracle keys simultaneously. If oracles are held by different independent organizations, this is significantly harder.

**Attack Scenario B: Oracle Binary Substitution**

1. Attacker gains write access to the CI runner where `wpm` is installed.
2. Attacker replaces `wpm` binary with a trojanized version that always returns `Accept`.
3. All subsequent releases are "adjudicated" by the malicious binary with forged verdicts.

**Detection**: Possible only if another party can verify the receipt signature independently.
**M-of-N mitigation**: Only one oracle has been substituted. The other oracles (on different systems, controlled by different parties) issue genuine verdicts. The attacker's forged `Accept` is outvoted by `Refuse` verdicts from other oracles.

**Attack Scenario C: Process Evidence Tampering**

1. Attacker intercepts evidence files before they reach the oracle.
2. Attacker modifies evidence (e.g., changes `verdict_claimed` from `FAIL` to `PASS`).
3. Modified evidence is submitted to the oracle, which issues `Accept`.

**Detection**: The oracle must sign over the evidence content hash. If it does, tampered evidence produces a signature mismatch.
**M-of-N mitigation**: Each oracle independently hashes the evidence and signs the hash. An attacker must submit the same tampered evidence to all M oracles simultaneously, and all M must independently accept the tampered evidence. Each oracle's independent evaluation reduces the attacker's success probability.

---

## 2. M-of-N Threshold Signature Scheme

### 2.1 FROST-Ed25519 Overview

cargo-cicd uses **FROST** (Flexible Round-Optimized Schnorr Threshold Signatures, RFC 9591) with Ed25519 as the underlying signature scheme.

FROST is chosen over alternatives (Shamir's Secret Sharing + multi-sig, BLS threshold signatures, RSA threshold signatures) for:

- **Ed25519 base**: Ed25519 is already used in SSH, TLS 1.3, and age encryption. It is battle-tested and has Rust implementations (`ed25519-dalek`).
- **Non-interactive aggregation**: FROST allows threshold signatures to be aggregated without further communication between signers, unlike some earlier threshold schemes.
- **Compact signatures**: FROST threshold signatures are the same size as a regular Ed25519 signature (64 bytes). The verification cost is identical.
- **RFC standardization**: RFC 9591 (2024) provides a stable specification. Interoperability between implementations is testable.
- **Rust ecosystem**: The `frost-ed25519` crate implements RFC 9591.

### 2.2 Key Generation Ceremony

**Participants**: N oracle operators (e.g., 3 for a 2-of-3 configuration).

**Requirements**:
- All N operators participate in the same ceremony session.
- The ceremony must be performed offline (no internet connectivity during key material generation).
- Each operator holds exactly one secret share; the group key never exists on any single machine.

**Ceremony Steps (Pedersen's DKG, as specified in FROST RFC 9591 Appendix C)**:

**Round 1: Commitment**

1. Each operator $i$ generates a polynomial $f_i(x) = a_{i,0} + a_{i,1}x + ... + a_{i,t-1}x^{t-1}$ of degree $t-1$ (where $t = M$, the threshold).
2. Each operator computes commitments $\phi_{i,j} = a_{i,j} \cdot G$ for each coefficient.
3. Each operator broadcasts their commitments $\{\phi_{i,j}\}$ to all other operators.

**Round 2: Share Exchange**

1. Each operator $i$ computes a secret share for each other operator $j$: $f_i(j)$.
2. Shares are exchanged over encrypted, authenticated channels.

**Round 3: Verification and Combination**

1. Each operator $i$ verifies the shares received from all other operators against their commitments.
2. Each operator computes their final secret share: $s_i = \sum_j f_j(i) \pmod{q}$.
3. The group public key is: $PK = \sum_i \phi_{i,0}$.

**Outputs**:
- Each operator $i$ holds secret share $s_i$ (stored offline, never transmitted after ceremony).
- All operators know the group public key $PK$ (public, distributed via key registry).
- Each operator knows all others' verification shares $VK_i = s_i \cdot G$ (used to verify partial signatures).

**Ceremony Record**: A signed ceremony transcript is published so that third parties can audit that the ceremony was conducted correctly. The transcript contains all commitments and verification shares but no secret material.

### 2.3 Per-Adjudication Signing Protocol

When cargo-cicd submits evidence for adjudication:

**Step 1: Evidence Submission**

```
cargo-cicd → Oracle A: POST /adjudicate { xes_content, nonce }
cargo-cicd → Oracle B: POST /adjudicate { xes_content, nonce }
cargo-cicd → Oracle C: POST /adjudicate { xes_content, nonce }
```

The `nonce` is a 32-byte random value generated by cargo-cicd. It prevents replay attacks.

**Step 2: Independent Evaluation**

Each oracle independently:
1. Parses the XES evidence.
2. Evaluates the evidence against its process model.
3. Produces a verdict: `Accept` or `Refuse`.
4. Signs the message: `sign(sk_i, Hash(verdict || evidence_hash || nonce || timestamp))`.

No oracle communicates with other oracles during evaluation.

**Step 3: Partial Signature Response**

Each oracle returns:
```json
{
  "oracle_id": "wasm4pm-prod-A",
  "verdict": "Accept",
  "evidence_hash": "SHA256:abc123...",
  "nonce_echo": "<nonce>",
  "timestamp": "2026-06-17T14:00:01.800Z",
  "partial_sig": "<base64-encoded FROST partial signature>",
  "verification_share": "<base64-encoded VK_i>"
}
```

**Step 4: Aggregation (Client-Side)**

cargo-cicd (or a dedicated aggregator service) collects responses and:

1. Verifies that `nonce_echo` matches the submitted nonce (prevents replay).
2. Verifies each partial signature against the oracle's verification share: $VK_i$.
3. Checks that M or more oracles agree on the same verdict.
4. If consensus: aggregates M partial signatures into a threshold signature.

```rust
struct ThresholdOracle {
    required: usize,                  // M
    oracles: Vec<OracleEndpoint>,     // N endpoints
    group_public_key: GroupPublicKey, // PK from DKG ceremony
}

#[derive(Debug)]
enum ThresholdVerdict {
    Accept {
        threshold_sig: Ed25519Signature,   // Aggregated FROST signature
        participants: Vec<String>,         // Which oracle IDs contributed
        consensus_count: usize,            // How many oracles agreed
    },
    Refuse {
        threshold_sig: Ed25519Signature,
        participants: Vec<String>,
        consensus_count: usize,
    },
    Inconclusive {
        accepts: usize,
        refuses: usize,
        blocked: usize,
        detail: String,
    },
}

impl ThresholdOracle {
    async fn adjudicate(&self, xes_path: &Path) -> Result<ThresholdVerdict> {
        let xes_bytes = std::fs::read(xes_path)?;
        let evidence_hash = sha256(&xes_bytes);
        let nonce = random_bytes(32);

        // Submit to all oracles concurrently
        let responses: Vec<Option<OracleResponse>> =
            futures::future::join_all(
                self.oracles.iter().map(|oracle| {
                    oracle.adjudicate(&xes_bytes, &evidence_hash, &nonce)
                })
            ).await;

        // Separate accepts from refuses
        let valid_responses: Vec<_> = responses.iter()
            .filter_map(|r| r.as_ref())
            .filter(|r| r.nonce_echo == nonce)
            .collect();

        let accepts: Vec<_> = valid_responses.iter()
            .filter(|r| r.verdict == OracleVerdict::Accept)
            .collect();

        let refuses: Vec<_> = valid_responses.iter()
            .filter(|r| r.verdict == OracleVerdict::Refuse)
            .collect();

        if accepts.len() >= self.required {
            // Aggregate M Accept partial signatures
            let sigs_to_aggregate: Vec<_> = accepts[..self.required]
                .iter()
                .map(|r| (&r.partial_sig, &r.oracle_id))
                .collect();

            let threshold_sig = frost_aggregate(
                &sigs_to_aggregate,
                &evidence_hash,
                &nonce,
                &self.group_public_key,
            )?;

            Ok(ThresholdVerdict::Accept {
                threshold_sig,
                participants: accepts.iter().map(|r| r.oracle_id.clone()).collect(),
                consensus_count: accepts.len(),
            })
        } else if refuses.len() >= self.required {
            let sigs_to_aggregate: Vec<_> = refuses[..self.required]
                .iter()
                .map(|r| (&r.partial_sig, &r.oracle_id))
                .collect();

            let threshold_sig = frost_aggregate(
                &sigs_to_aggregate,
                &evidence_hash,
                &nonce,
                &self.group_public_key,
            )?;

            Ok(ThresholdVerdict::Refuse {
                threshold_sig,
                participants: refuses.iter().map(|r| r.oracle_id.clone()).collect(),
                consensus_count: refuses.len(),
            })
        } else {
            Ok(ThresholdVerdict::Inconclusive {
                accepts: accepts.len(),
                refuses: refuses.len(),
                blocked: self.oracles.len() - valid_responses.len(),
                detail: format!(
                    "Need {} agreements; got {} accepts, {} refuses, {} blocked",
                    self.required, accepts.len(), refuses.len(),
                    self.oracles.len() - valid_responses.len()
                ),
            })
        }
    }
}
```

### 2.4 Verification Algorithm

Any verifier can check a threshold signature using only the group public key:

```rust
fn verify_threshold_verdict(
    verdict: &ThresholdVerdict,
    evidence_hash: &[u8; 32],
    nonce: &[u8; 32],
    group_public_key: &GroupPublicKey,
) -> bool {
    match verdict {
        ThresholdVerdict::Accept { threshold_sig, .. } |
        ThresholdVerdict::Refuse { threshold_sig, .. } => {
            let message = canonical_message(
                verdict.verdict_str(),
                evidence_hash,
                nonce,
            );
            ed25519_verify(group_public_key, &message, threshold_sig)
        },
        ThresholdVerdict::Inconclusive { .. } => false,
    }
}

fn canonical_message(verdict: &str, evidence_hash: &[u8; 32], nonce: &[u8; 32]) -> Vec<u8> {
    // Deterministic serialization for signing:
    // version (1 byte) || verdict_len (1 byte) || verdict || evidence_hash (32) || nonce (32)
    let mut msg = Vec::new();
    msg.push(1u8);  // protocol version
    msg.push(verdict.len() as u8);
    msg.extend_from_slice(verdict.as_bytes());
    msg.extend_from_slice(evidence_hash);
    msg.extend_from_slice(nonce);
    msg
}
```

The verification is identical to a single Ed25519 signature verification — no special FROST tooling needed for verification. This is a key property of threshold signatures: the verifier is unaware that multiple parties signed.

---

## 3. Oracle Registry

### 3.1 Oracle Registration

The oracle registry is a publicly readable, append-only JSON document published at a stable URL:

```
https://registry.cargo-cicd.rs/oracles/v1.json
```

**Registry Entry Schema**:

```json
{
  "schema": "oracle-registry/v1",
  "updated_at": "2026-06-17T00:00:00Z",
  "oracles": [
    {
      "id": "wasm4pm-prod-2026",
      "display_name": "wasm4pm Production Oracle 2026",
      "provider": {
        "name": "wasm4pm.rs",
        "url": "https://wasm4pm.rs",
        "contact": "security@wasm4pm.rs"
      },
      "version": "0.9.x",
      "fingerprint": "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0=",
      "public_key_base64": "<base64-encoded Ed25519 public key>",
      "endpoint": "https://oracle-a.wasm4pm.rs/adjudicate",
      "protocols": ["single-oracle/v1", "frost-ed25519/v1"],
      "capabilities": [
        "process-conformance",
        "receipt-doctor",
        "cargo-cicd-v26"
      ],
      "standards": ["SLSA-L3", "NIST-SP-800-218"],
      "valid_from": "2026-01-01T00:00:00Z",
      "valid_until": "2027-01-01T00:00:00Z",
      "revoked": false,
      "revocation_reason": null,
      "ceremony_transcript_url": "https://registry.cargo-cicd.rs/ceremonies/wasm4pm-prod-2026-dkg.json"
    }
  ],
  "threshold_groups": [
    {
      "id": "standard-2of3-2026",
      "description": "Standard 2-of-3 threshold group for 2026",
      "required": 2,
      "total": 3,
      "group_public_key_base64": "<base64-encoded group Ed25519 public key>",
      "members": ["wasm4pm-prod-2026", "wasm4pm-prod-B-2026", "community-oracle-2026"],
      "valid_from": "2026-01-01T00:00:00Z",
      "valid_until": "2027-01-01T00:00:00Z",
      "ceremony_transcript_url": "https://registry.cargo-cicd.rs/ceremonies/standard-2of3-2026-dkg.json"
    }
  ]
}
```

### 3.2 Oracle Discovery

cargo-cicd discovers available oracles through:

**Method 1: Configured (cicd.toml)**:
```toml
[oracle_registry]
url = "https://registry.cargo-cicd.rs/oracles/v1.json"
cache_path = "target/cargo-cicd/oracle-registry.json"
cache_expiry_hours = 24
```

**Method 2: Bundled (default)**:
A default registry snapshot is bundled with cargo-cicd at build time. This enables oracle discovery without network access. It is updated with each cargo-cicd release.

**Method 3: Environment variable (CI override)**:
```bash
export CARGO_CICD_ORACLE_REGISTRY=/path/to/local-oracle-registry.json
```

**Filtering by standard**:
```sh
# Find all oracles that support SLSA-L3
cargo cicd evidence doctor --oracle-standard SLSA-L3

# Find oracles for a specific threshold group
cargo cicd evidence doctor --threshold-group standard-2of3-2026
```

### 3.3 Key Rotation

When an oracle rotates its signing key:

1. **New key generation**: The oracle operator generates a new Ed25519 keypair.
2. **Registry update**: The registry appends a new entry with the new key and updates `valid_until` on the old entry.
3. **Re-ceremony (for threshold groups)**: If the oracle participates in threshold groups, a new DKG ceremony is scheduled to produce new group keys.
4. **Grace period**: The old key remains valid for a configurable grace period (default: 30 days) to allow existing receipts to be validated.

**Rotation Record Format**:
```json
{
  "oracle_id": "wasm4pm-prod-2026",
  "old_fingerprint": "SHA256:Bz3k...",
  "new_fingerprint": "SHA256:Eq6n...",
  "rotation_at": "2027-01-01T00:00:00Z",
  "old_valid_until": "2027-02-01T00:00:00Z",   // 30-day grace period
  "signed_by_old_key": "<self-signed rotation announcement>"
}
```

The rotation announcement is signed by the old key, proving continuity of the oracle identity.

**Revocation** (emergency): If a key is compromised before its scheduled rotation:

```json
{
  "oracle_id": "wasm4pm-prod-2026",
  "fingerprint": "SHA256:Bz3k...",
  "revoked": true,
  "revocation_reason": "Key material compromised 2026-11-15",
  "revoked_at": "2026-11-15T18:00:00Z",
  "signed_by": "wasm4pm-prod-2027"   // New key signs the revocation
}
```

All `Accept` verdicts issued by the revoked key after the compromise date must be considered invalid. Before the compromise date, verdicts remain valid (assuming the key was uncompromised).

---

## 4. Receipt Aggregation

### 4.1 Individual Oracle Receipts

Each participating oracle issues an individual receipt:

```json
{
  "schema": "oracle-receipt/v1",
  "oracle_id": "wasm4pm-prod-A",
  "oracle_version": "0.9.2",
  "oracle_fingerprint": "SHA256:Bz3k...",
  "evidence_hash": "SHA256:xyz789...",
  "nonce": "<base64-encoded 32-byte nonce>",
  "verdict": "Accept",
  "evaluated_at": "2026-06-17T14:00:01.800Z",
  "partial_sig": "<base64-encoded FROST partial signature>",
  "verification_share": "<base64-encoded verification share VK_i>",
  "process_model": "basic-release/v1.0",
  "conformance_result": {
    "fitness": 1.0,
    "violations": []
  }
}
```

Individual receipts are stored in `receipts/partial/` during aggregation and deleted (or archived) after aggregation completes.

### 4.2 Aggregate Receipt Format

The aggregate receipt combines individual receipts into a single verifiable document:

```json
{
  "schema": "threshold-receipt/v2",
  "receipt_id": "rcpt-20260617T140002Z-abc123",
  
  "threshold": {
    "algorithm": "FROST-Ed25519",
    "group_id": "standard-2of3-2026",
    "group_public_key": "<base64-encoded group Ed25519 public key>",
    "required": 2,
    "total": 3
  },
  
  "verdict": "Accept",
  "adjudicated_at": "2026-06-17T14:00:02.000Z",
  
  "evidence": {
    "hash": "SHA256:xyz789...",
    "xes_path": "target/cargo-cicd/evidence/evt-publish-run-20260617T140000Z.xes",
    "command": "publish run",
    "workspace": "cargo-cicd@/home/user/cargo-cicd"
  },
  
  "threshold_signature": "<base64-encoded aggregated FROST signature>",
  
  "participants": [
    {
      "oracle_id": "wasm4pm-prod-A",
      "fingerprint": "SHA256:Bz3k...",
      "verdict": "Accept",
      "evaluated_at": "2026-06-17T14:00:01.800Z",
      "partial_sig": "<base64-encoded partial signature>",
      "process_model": "basic-release/v1.0",
      "conformance_fitness": 1.0
    },
    {
      "oracle_id": "org-internal-oracle",
      "fingerprint": "SHA256:Dq5m...",
      "verdict": "Accept",
      "evaluated_at": "2026-06-17T14:00:01.950Z",
      "partial_sig": "<base64-encoded partial signature>",
      "process_model": "basic-release/v1.0",
      "conformance_fitness": 1.0
    }
  ],
  
  "non_participants": [
    {
      "oracle_id": "wasm4pm-prod-B",
      "reason": "Blocked",
      "attempted_at": "2026-06-17T14:00:01.500Z",
      "error": "Connection timeout after 5000ms"
    }
  ],
  
  "nonce": "<base64-encoded 32-byte nonce>",
  "cargo_cicd_version": "26.6.2",
  "threshold_receipt_version": "2"
}
```

### 4.3 Storing the Aggregate Receipt in Cargo.toml

After aggregation, the receipt is stored in `receipts/` and referenced from `Cargo.toml [package.metadata.evidence]`:

```toml
[package.metadata.evidence]
oracle = "threshold:standard-2of3-2026"
threshold_required = 2
threshold_total = 3
last_verdict = "Accept"
receipt_path = "receipts/rcpt-20260617T140002Z-abc123.json"
last_adjudicated_at = "2026-06-17T14:00:02.000Z"
group_public_key_fingerprint = "SHA256:GroupKey..."
```

### 4.4 Aggregate Receipt Verification

A third party verifies the aggregate receipt:

```rust
fn verify_aggregate_receipt(
    receipt: &ThresholdReceipt,
    group_public_key: &GroupPublicKey,
) -> VerificationResult {
    // 1. Check threshold is met (participants.len() >= required)
    if receipt.participants.len() < receipt.threshold.required {
        return VerificationResult::InsufficientParticipants;
    }

    // 2. Verify the threshold signature over the canonical message
    let evidence_hash = decode_hash(&receipt.evidence.hash)?;
    let nonce = decode_base64(&receipt.nonce)?;
    let message = canonical_message(&receipt.verdict, &evidence_hash, &nonce);

    if !ed25519_verify(group_public_key, &message, &receipt.threshold_signature) {
        return VerificationResult::InvalidThresholdSignature;
    }

    // 3. Verify individual partial signatures (optional, for deeper audit)
    for participant in &receipt.participants {
        let vk = lookup_verification_share(&participant.oracle_id)?;
        let partial_message = canonical_message(&participant.verdict, &evidence_hash, &nonce);
        if !ed25519_verify_partial(&vk, &partial_message, &participant.partial_sig) {
            return VerificationResult::InvalidPartialSignature(participant.oracle_id.clone());
        }
    }

    VerificationResult::Valid
}
```

---

## 5. Failure Modes and Recovery

### 5.1 Oracle Unavailability (1 of 3 Offline)

**Scenario**: In a 2-of-3 configuration, one oracle is temporarily unavailable (network timeout, maintenance).

**Behavior**:
1. cargo-cicd contacts all 3 oracles with a 5-second timeout.
2. 2 respond with `Accept`; 1 responds with `Blocked` (timeout).
3. 2 `Accept` responses meet the threshold (2-of-3 required).
4. Aggregation proceeds with 2 participants.
5. `non_participants` in the receipt records the failed oracle.

**Outcome**: `Accept` receipt issued. No disruption to release pipeline.

**Recovery**: None needed. The aggregate receipt is valid. When the unavailable oracle comes back online, it can be used in future adjudications.

### 5.2 Oracle Disagreement (Different Verdicts)

**Scenario**: In a 2-of-3 configuration, two oracles return `Accept` but one returns `Refuse`.

**Behavior**:
1. cargo-cicd collects: 2 `Accept`, 1 `Refuse`.
2. The `Accept` threshold (2-of-3) is met.
3. cargo-cicd issues an `Accept` threshold signature with the 2 accepting oracles.
4. The `Refuse` from the third oracle is recorded in `non_participants` with reason "Dissenting".

**Outcome**: `Accept` receipt issued (majority rule). The dissenting `Refuse` is recorded for audit.

**Alternative policy (unanimous required)**:
```toml
[evidence.threshold_oracle]
required = 3    # Unanimous: all 3 must agree
total = 3
unanimous = true    # If set, any dissent = Inconclusive
```

With `unanimous = true`, any dissent results in `Inconclusive` and requires human review.

**Operator notification**: When verdict divergence exceeds a threshold (e.g., 1 in 10 adjudications has a dissenting oracle), operators are notified to investigate the dissenting oracle.

### 5.3 Key Compromise and Emergency Revocation

**Scenario**: One oracle's signing key is stolen. An attacker uses it to issue forged `Accept` partial signatures.

**Detection**:
- Other oracle operators observe that their oracle did not participate in specific adjudications but receipts claim it did.
- Anomalous patterns in adjudication timing or participant lists.
- Direct discovery of key material theft.

**Immediate Response**:
1. Compromised oracle operator publishes an emergency revocation to the registry.
2. Registry marks the fingerprint as `revoked` with a compromise date.
3. cargo-cicd clients that cache the registry are updated within their cache expiry period (default: 24 hours).

**Impact Assessment**:
- All `Accept` receipts that include the compromised oracle as a participant **after** the compromise date are suspect.
- Receipts from before the compromise date are assumed valid.
- `Accept` receipts that include 2 non-compromised oracles (for 2-of-3) remain valid even if the compromised oracle participated, because the threshold signature requires M partial signatures and M-1 uncompromised sigs remain valid.

**Re-adjudication**:
Packages whose `Accept` receipts are compromised must be re-adjudicated using an uncorrupted oracle pool.

### 5.4 Fewer Than M Oracles Available

**Scenario**: In a 2-of-3 configuration, only 1 oracle is available (2 are down simultaneously).

**Behavior**:
1. cargo-cicd contacts all 3; only 1 responds.
2. 1 `Accept` < required threshold of 2.
3. Result: `Inconclusive`.
4. Release pipeline blocks.

**Outcomes by configured policy**:

| Policy | Action |
|--------|--------|
| `on_inconclusive = "block"` (default) | Release fails. Human intervention required. |
| `on_inconclusive = "single_fallback"` | Fall back to single-oracle mode using the one available oracle. |
| `on_inconclusive = "defer"` | Queue for retry when oracles come back online. |
| `on_inconclusive = "warn_and_continue"` | Issue release with `WARN:inconclusive_oracle` verdict. Not for regulated contexts. |

**Availability calculation**:
With 3 independent oracles each at 99.9% uptime:
- P(all 3 available) = 99.9%^3 ≈ 99.7%
- P(at least 2 available for 2-of-3) = 1 - P(2+ unavailable) ≈ 1 - 3×(0.001)^2 ≈ 99.9997%
- Effective availability for 2-of-3 is higher than single oracle.

---

## 6. Phase 2 Implementation Plan

### 6.1 Weeks 1-4: Individual Oracle Protocol Extension

**Goal**: Extend the existing single-oracle protocol to emit nonces, evidence hashes, and support partial signature responses.

**Deliverables**:

- [ ] Define `oracle-receipt/v1` JSON schema.
- [ ] Update `Wasm4pmShell` to include nonce in adjudication requests.
- [ ] Update `wpm` oracle to return evidence hash and partial signature (FROST Round 1).
- [ ] Add `OracleRegistry` struct that loads from JSON.
- [ ] Add `cargo cicd evidence doctor --oracle standard-2of3-2026` command variant.

**Test Scenarios** (Weeks 1-4):
1. Single oracle adjudication with nonce echoing (backward compatible).
2. Oracle registry loading from local file.
3. Oracle discovery by standard (`SLSA-L3`).
4. Registry caching and expiry.
5. Registry URL offline (fallback to bundled snapshot).

### 6.2 Weeks 5-8: Threshold Aggregation Library

**Goal**: Implement FROST-Ed25519 threshold aggregation as a standalone Rust library.

**Deliverables**:

- [ ] `frost-aggregator` library crate at `crates/frost-aggregator/`.
- [ ] Implements FROST RFC 9591 aggregation.
- [ ] Partial signature verification.
- [ ] Threshold signature generation.
- [ ] Threshold signature verification (using group public key only).
- [ ] `ThresholdReceipt` JSON serialization/deserialization.
- [ ] DKG ceremony tool (offline, for oracle operators).

**Test Scenarios** (Weeks 5-8):
6. 2-of-3 aggregation: all 3 respond Accept.
7. 2-of-3 aggregation: 2 Accept, 1 Blocked.
8. 2-of-3 aggregation: 2 Accept, 1 Refuse (Accept wins).
9. 2-of-3 aggregation: 1 Accept, 2 Refuse (Refuse wins).
10. 2-of-3 aggregation: 1 Accept, 1 Refuse, 1 Blocked (Inconclusive).
11. Threshold signature verification (valid).
12. Threshold signature verification (tampered — should fail).
13. Threshold signature verification with revoked key (should fail after compromise date).
14. Partial signature verification for each oracle.
15. DKG ceremony produces valid group key.

### 6.3 Weeks 9-12: Integration with cargo-cicd Publish Flow

**Goal**: Integrate threshold oracle into the `publish run` verb flow.

**Deliverables**:

- [ ] `ThresholdOracle` struct in `src/integrations/threshold_oracle.rs`.
- [ ] `publish run` invokes `ThresholdOracle::adjudicate()` when `threshold_oracle` is configured.
- [ ] Aggregate receipt stored in `receipts/`.
- [ ] `Cargo.toml [package.metadata.evidence]` updated after aggregation.
- [ ] `evidence audit --threshold` command: verifies aggregate receipt offline.
- [ ] `cargo cicd status show` displays threshold oracle status.

**Test Scenarios** (Weeks 9-12):
16. `publish run` with 2-of-3 threshold produces aggregate receipt.
17. Aggregate receipt stored in `receipts/` with correct schema.
18. `Cargo.toml [package.metadata.evidence]` updated with `group_public_key_fingerprint`.
19. `evidence audit --threshold` verifies receipt offline.
20. `status show` displays "Threshold: 2-of-3 Accept (2026-06-17)".
21. Oracle revocation detected during `evidence audit`.
22. `on_inconclusive = "single_fallback"` policy.
23. Threshold oracle timeout (5s) with fallback to available oracles.
24. Emergency revocation (compromise date after receipt — receipt still valid).
25. Emergency revocation (compromise date before receipt — receipt invalid).

### 6.4 Total Test Coverage Summary

| Category | Test Count | Target Pass Rate |
|----------|-----------|-----------------|
| Individual Oracle Protocol | 5 | 100% |
| Threshold Aggregation | 10 | 100% |
| Publish Flow Integration | 10 | 100% |
| **Total** | **25** | **100%** |

All test scenarios are automated in `tests/threshold_oracle/` and included in `cargo make test`.

---

## Appendix A: Dependency Requirements

```toml
# For frost-aggregator library
[dependencies]
frost-ed25519 = "2.0"        # FROST RFC 9591 Ed25519 implementation
ed25519-dalek = "2.1"        # Ed25519 base implementation
rand = "0.8"                 # CSPRNG for nonce generation
sha2 = "0.10"                # SHA-256 for evidence hashing
base64ct = "1.6"             # URL-safe base64 encoding
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# For cargo-cicd threshold oracle integration
[dependencies]
tokio = { version = "1.0", features = ["rt-multi-thread", "time"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

---

## Appendix B: FROST Algorithm Reference

FROST (RFC 9591) Aggregation (pseudocode):

```
Input:
  - signers: [(oracle_id, partial_sig, verification_share)]  (M signers)
  - message: &[u8]
  - group_key: GroupPublicKey

Output:
  - threshold_signature: Ed25519Signature

Steps:
1. Compute binding factors:
   For each (id, partial_sig, VK_i) in signers:
     binding_factor_i = H(id, message, signers_list)
   
2. Compute group commitment:
   R = sum_i (R_i * binding_factor_i)  // R_i from partial_sig

3. Compute Lagrange coefficients:
   For each (id, ...) in signers:
     lambda_i = product_{j != i} (j / (j - i))  // over scalar field

4. Aggregate partial signatures:
   s = sum_i (s_i * lambda_i)  // s_i from partial_sig

5. Output (R, s) as the threshold signature
```

The output (R, s) is a standard Ed25519 signature verifiable with the group public key.

---

*Document version 1.0 — 2026-06-17*  
*See also: `docs/adr/ADR-016-distributed-oracle-consensus.md`*
