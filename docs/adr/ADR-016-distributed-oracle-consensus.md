# ADR-016: Phase 2 Distributed M-of-N Oracle Consensus

**Status:** Proposed (Phase 2)  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** oracle, consensus, threshold-signatures, distributed, phase-2, ed25519

---

## Context

ADR-012 established that cargo-cicd never adjudicates itself — an external oracle (wasm4pm) issues all verdicts. This invariant is correct and permanent. However, the current architecture has a critical limitation: **a single oracle is a single point of trust failure**.

### Single-Oracle Failure Modes

1. **Oracle unavailability**: If the wpm binary is unavailable (network outage, binary not installed, corrupt installation), all adjudication is blocked. High-assurance releases cannot proceed.

2. **Oracle compromise**: If the oracle's signing key is compromised, an attacker can issue fraudulent `Accept` verdicts. All receipts signed by the compromised key must be considered suspect.

3. **Oracle vendor lock-in**: If wasm4pm is the only supported oracle, cargo-cicd is dependent on a single vendor. Organizations with regulatory requirements may need to use their own oracle.

4. **Conflict of interest**: If the same organization develops cargo-cicd and wasm4pm, there is a theoretical conflict of interest even with architectural separation. Independent verification requires truly independent parties.

5. **Regulatory gap**: Framework such as DO-178C Supplement DO-330 (tool qualification) requires that safety-critical tool certification involves multiple independent parties. A single oracle does not satisfy this requirement.

### Regulatory Context for M-of-N

Several regulatory frameworks explicitly require multiple independent verifiers for high-assurance software:

- **DO-178C**: For DAL A (highest assurance) software, independent verification activities are required. A single oracle does not constitute independent verification.
- **ISO 26262**: Functional safety in automotive requires at least two independent safety analyses for ASIL D components.
- **IEC 62443**: Industrial cybersecurity standards require defense-in-depth, which includes multiple verification layers.
- **NIST SP 800-218 (SSDF)**: Recommends multiple independent security assessments for critical components.

### Threshold Signature Background

Threshold signature schemes allow a group of N parties to jointly sign a message such that any M-of-N parties can produce a valid signature, but fewer than M parties cannot. For cargo-cicd:

- Each oracle holds a share of a distributed signing key.
- To produce an `Accept` verdict with a valid signature, M oracles must independently adjudicate and agree.
- A single compromised or unavailable oracle cannot produce or block a valid verdict.

Ed25519 threshold signatures (based on Schnorr threshold schemes like FROST — Flexible Round-Optimized Schnorr Threshold Signatures) are the state of the art for this use case.

---

## Decision

**Phase 2 introduces M-of-N threshold oracle consensus for high-assurance adjudication. The default threshold is 2-of-3 (two of three independent oracles must independently agree).**

This is a Phase 2 decision — implementation is planned for Phase 2 Weeks 1-12 (see `docs/PHASE-2-DESIGN.md`). Phase 1 continues with single-oracle architecture.

### Threshold Configuration

```toml
# In cicd.toml or Cargo.toml [evidence]
[evidence.threshold_oracle]
required = 2           # M — minimum agreements required
total = 3              # N — total oracle pool size
algorithm = "FROST-Ed25519"

# Oracle pool members
[[evidence.threshold_oracle.oracles]]
id = "wasm4pm-prod-A"
endpoint = "https://oracle-a.wasm4pm.rs/adjudicate"
fingerprint = "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0="

[[evidence.threshold_oracle.oracles]]
id = "wasm4pm-prod-B"
endpoint = "https://oracle-b.wasm4pm.rs/adjudicate"
fingerprint = "SHA256:Cp4l0YsS8nOqZrX3MdU6vF9eGtHiKwLmNoPqRsTuVw1="

[[evidence.threshold_oracle.oracles]]
id = "org-internal-oracle"
endpoint = "https://oracle.internal.myorg.com/adjudicate"
fingerprint = "SHA256:Dq5m1ZtT9oProYs4NeV7wG0fHuIjLxMnOpQrStUvWx2="
```

### Signing Protocol (FROST-Ed25519)

FROST (Flexible Round-Optimized Schnorr Threshold Signatures, RFC 9591) is used for threshold Ed25519 signatures. The protocol:

**Phase A: Key Generation Ceremony (Offline, Once)**

1. N oracle operators participate in a Distributed Key Generation (DKG) ceremony.
2. Each operator i receives a secret share `sk_i` and the group public key `PK`.
3. Each operator also holds a verification share `VK_i` that allows other parties to verify their partial signatures.
4. The group public key `PK` is published to the oracle key registry.
5. No single party knows the full private key — it never exists in one place.

**Phase B: Per-Adjudication Signing**

1. A client (cargo-cicd) submits XES evidence to all N oracle endpoints.
2. Each oracle independently evaluates the evidence and produces:
   - A local verdict: Accept or Refuse
   - A partial signature over `(verdict, evidence_hash, timestamp)` using its key share `sk_i`
3. The client collects M responses (M ≥ required).
4. If M responses agree on the same verdict, the client uses FROST aggregation to combine the M partial signatures into a single threshold signature over the agreed verdict.
5. The threshold signature is mathematically equivalent to a signature by the group key `PK` — verifiable by any party that knows `PK`.

**Pseudocode**:

```rust
struct ThresholdOracle {
    required: usize,          // M
    oracles: Vec<OracleEndpoint>,  // N endpoints
}

struct OracleResponse {
    oracle_id: String,
    verdict: OracleVerdict,    // Accept | Refuse | Blocked
    partial_sig: Ed25519PartialSig,
    signed_at: DateTime<Utc>,
}

impl ThresholdOracle {
    async fn adjudicate(&self, xes_path: &Path) -> Result<ThresholdVerdict> {
        let evidence_hash = sha256_file(xes_path)?;

        // Submit to all oracles concurrently
        let responses: Vec<OracleResponse> = futures::join_all(
            self.oracles.iter().map(|o| o.adjudicate(xes_path, &evidence_hash))
        ).await.into_iter().filter_map(|r| r.ok()).collect();

        // Count verdicts
        let accepts: Vec<_> = responses.iter().filter(|r| r.verdict == Accept).collect();
        let refuses: Vec<_> = responses.iter().filter(|r| r.verdict == Refuse).collect();

        // Check if consensus is reached
        if accepts.len() >= self.required {
            let sig = frost_aggregate(&accepts[..self.required])?;
            Ok(ThresholdVerdict::Accept { sig, participants: accepts.len() })
        } else if refuses.len() >= self.required {
            let sig = frost_aggregate(&refuses[..self.required])?;
            Ok(ThresholdVerdict::Refuse { sig, participants: refuses.len() })
        } else {
            Ok(ThresholdVerdict::Inconclusive {
                accepts: accepts.len(),
                refuses: refuses.len(),
                blocked: responses.iter().filter(|r| r.verdict == Blocked).count(),
            })
        }
    }
}
```

### Aggregate Receipt Format

The aggregate receipt records all participating oracle responses and the threshold signature:

```json
{
  "schema": "threshold-receipt/v1",
  "algorithm": "FROST-Ed25519",
  "required": 2,
  "total": 3,
  "verdict": "Accept",
  "evidence_hash": "SHA256:abc123...",
  "threshold_signature": "<base64-encoded FROST threshold signature>",
  "group_public_key": "<base64-encoded group Ed25519 public key>",
  "adjudicated_at": "2026-06-17T14:00:02.000Z",
  "participants": [
    {
      "oracle_id": "wasm4pm-prod-A",
      "fingerprint": "SHA256:Bz3k...",
      "verdict": "Accept",
      "partial_sig": "<base64-encoded partial signature>",
      "signed_at": "2026-06-17T14:00:01.800Z"
    },
    {
      "oracle_id": "org-internal-oracle",
      "fingerprint": "SHA256:Dq5m...",
      "verdict": "Accept",
      "partial_sig": "<base64-encoded partial signature>",
      "signed_at": "2026-06-17T14:00:01.950Z"
    }
  ],
  "non_participants": [
    {
      "oracle_id": "wasm4pm-prod-B",
      "reason": "Blocked",
      "attempted_at": "2026-06-17T14:00:01.500Z"
    }
  ]
}
```

### Threshold Verification

Any verifier with the group public key `PK` can verify the aggregate receipt:

```rust
fn verify_threshold_receipt(receipt: &ThresholdReceipt, evidence_hash: &[u8]) -> bool {
    // Verify the threshold signature using group public key
    let message = canonical_message(receipt.verdict, evidence_hash, receipt.adjudicated_at);
    ed25519_verify(&receipt.group_public_key, &message, &receipt.threshold_signature)
}
```

No oracle is needed for verification — the threshold signature is self-contained.

---

## Consequences

### Positive

1. **Eliminates single-oracle failure**: If 1-of-3 oracles is unavailable, adjudication proceeds with the other two (assuming 2-of-3 threshold). Availability improves from 99.9% (single oracle) to 99.99% (any 2 of 3 with 99.9% individual uptime).

2. **Key compromise containment**: An attacker who compromises one oracle's key share cannot produce a valid threshold signature alone. They need M key shares simultaneously.

3. **Regulatory satisfaction**: Multiple independent oracles from different organizations satisfy "independent verification" requirements in DO-178C, ISO 26262, and similar frameworks.

4. **Vendor diversity**: Organizations can include their own internal oracle in the pool, reducing vendor lock-in while still benefiting from community oracles.

5. **Cryptographic verifiability**: The FROST threshold signature is verifiable offline with the group public key. No live oracle access needed post-adjudication.

6. **Backward compatibility**: The threshold oracle is an extension of the single-oracle architecture. Phase 1 receipts (single oracle) remain valid; Phase 2 introduces optional high-assurance mode.

### Negative

1. **Implementation complexity**: FROST-Ed25519 key generation, partial signature exchange, and aggregation are significantly more complex than single-oracle adjudication. Mitigation: Use battle-tested FROST library (frost-ed25519 crate).

2. **Coordination overhead**: M-of-N adjudication requires network round-trips to M oracles. For 2-of-3, latency is determined by the second-fastest oracle. Typical added latency: 100-500ms. Mitigation: Oracles are contacted concurrently; threshold is met as soon as M respond.

3. **DKG ceremony**: The initial key generation ceremony requires all N oracle operators to be present and online simultaneously. This is a one-time cost but logistically complex. Mitigation: FROST supports refreshable key shares (DKG can be re-run periodically for key rotation).

4. **Oracle pool management**: The oracle pool configuration must be kept in sync across all cargo-cicd users. Mitigation: Pool configuration is published in the oracle key registry (see ADR-013).

5. **Threshold consensus failures**: If fewer than M oracles respond or agree, adjudication is inconclusive. Mitigation: `Inconclusive` is a first-class verdict alongside `Accept`/`Refuse`/`Blocked`; the release gate can be configured to retry or require manual intervention.

---

## Comparison with Single-Oracle Architecture

| Property | Single Oracle | M-of-N Threshold |
|----------|--------------|-----------------|
| Availability | Single oracle uptime | Any M of N uptime |
| Key compromise risk | Single key | Requires M simultaneous compromises |
| Regulatory independence | Weaker (single party) | Stronger (M independent parties) |
| Adjudication latency | ~100ms | ~200-500ms (2nd fastest oracle) |
| Implementation complexity | Simple | Complex (FROST) |
| Offline verification | Possible (single sig) | Possible (threshold sig) |
| Phase | Phase 1 (current) | Phase 2 (planned) |

---

## References

- FROST RFC 9591: https://www.rfc-editor.org/rfc/rfc9591
- frost-ed25519 Rust crate: https://crates.io/crates/frost-ed25519
- DO-178C: Software Considerations in Airborne Systems (Section 6.3.3: Independence)
- ISO 26262: Road vehicles — Functional safety
- ADR-012: Oracle Architecture (E1 invariant)
- ADR-013: Oracle Public Key Embedding (fingerprint in traces)
- `docs/distributed-oracle-design.md`: Detailed protocol design

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial Phase 2 proposal |
