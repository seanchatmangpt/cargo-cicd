# ADR-013: Embed Oracle Public Key Fingerprint in XES Evidence Traces

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** oracle, cryptography, provenance, key-fingerprint, xes, trust

---

## Context

When a wasm4pm oracle adjudicates process evidence and issues an `Accept` verdict, the receipt records that the adjudication happened. However, there is an unresolved question: **which oracle issued this verdict?**

This matters because:

1. **Key diversity**: In a multi-oracle ecosystem (Phase 2), different oracles hold different private keys. A receipt from Oracle A does not carry the same trust level as a receipt from Oracle B if they have different assurance profiles.

2. **Key compromise**: If an oracle's signing key is compromised, all receipts signed by that key must be considered suspect. Verifiers need to know which key was used to issue each receipt so they can revoke trust selectively.

3. **Key rotation**: Oracles periodically rotate their signing keys. A receipt signed with an old key is still valid if the key was valid at the time of signing. Verifiers need the historical key fingerprint to validate time-bound receipts.

4. **Auditor independence**: A third-party auditor reviewing archived evidence must be able to verify which oracle adjudicated without access to the live oracle or cargo-cicd. The evidence file must be self-describing.

5. **Supply chain integrity**: An attacker who can substitute forged evidence must also forge the oracle signature to pass verification. Embedding the key fingerprint in evidence makes this significantly harder.

### Current State

Currently, the XES evidence files contain the `verdict_claimed` attribute but do not embed oracle identity. The oracle issues a receipt separately (as a JSON file in `receipts/`), but the XES trace does not reference which key signed it.

This creates a gap: the XES evidence and the receipt are linked only by filename conventions, not by cryptographic reference. An auditor must trust the filename convention rather than the evidence itself.

### Alternative A: Key Registry URL in Trace

Embed a URL reference to a key registry:

```xml
<string key="cargoCI:oracle_registry_url"
        value="https://oracle-registry.cargo-cicd.rs/keys/wasm4pm-prod-2026"/>
```

**Pros**: Small payload; oracle registry can update key metadata without changing evidence files.

**Cons**: Network dependency during verification; URL may be unavailable in the future (link rot); does not bind the evidence cryptographically to a specific key — only to a URL that may change.

### Alternative B: Key Embedding in Receipt Only

The full public key (or key fingerprint) is in the receipt JSON, not in the XES file. The XES file references the receipt by filename.

**Pros**: Keeps XES files small; no duplication.

**Cons**: XES and receipt are linked only by filename, not cryptographically. An attacker can substitute a different receipt file. XES evidence is not self-describing — verifiers must locate the companion receipt.

### Alternative C: Full Public Key in XES Trace

Embed the complete Ed25519 public key bytes (32 bytes, base64-encoded = 44 characters) in the XES trace.

**Pros**: XES is fully self-contained for verification.

**Cons**: Key bytes are larger than a fingerprint; if the same key is used across many traces in one file, the duplication is wasteful; a fingerprint is sufficient for key identification.

### Alternative D: SHA-256 Fingerprint of Oracle Public Key (Selected)

Embed the SHA-256 fingerprint of the oracle's public key in the XES trace:

```xml
<string key="cargoCI:oracle_key_fingerprint"
        value="SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0="/>
```

The fingerprint is 44 characters (SHA-256 base64url) and uniquely identifies the key without embedding the full key material. This follows the `ssh-keygen` fingerprint convention.

---

## Decision

**Embed the SHA-256 fingerprint of the adjudicating oracle's public key in every XES trace, in the `cargoCI:oracle_key_fingerprint` attribute.**

### Attribute Specification

```
Key:   cargoCI:oracle_key_fingerprint
Value: "SHA256:<base64url-encoded 32-byte hash>"
       where the hash is SHA-256(oracle_public_key_bytes)
```

Example:
```xml
<trace>
  <string key="concept:name" value="status_show_phase"/>
  <string key="cargoCI:command" value="status show"/>
  <string key="cargoCI:oracle_key_fingerprint"
          value="SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0="/>
  <!-- ... events ... -->
</trace>
```

### Where the Fingerprint Comes From

The fingerprint is provided by the oracle itself as part of the adjudication response. When `wpm audit <file.xes>` is invoked, the oracle's receipt (in `receipts/`) includes:

```json
{
  "verdict": "Accept",
  "oracle_key_fingerprint": "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0=",
  "oracle_version": "wasm4pm/0.9.2",
  "signed_at": "2026-06-17T14:00:01.234Z",
  "signature": "<ed25519 signature of the receipt hash>"
}
```

cargo-cicd reads the fingerprint from the receipt and embeds it into subsequent XES traces that reference this oracle session.

**Note**: The fingerprint is embedded in the XES trace metadata level (trace attributes), not in individual events. All events within a trace are adjudicated by the same oracle session.

### Verification Protocol

A third-party verifier who receives an archived XES file:

1. Extracts `cargoCI:oracle_key_fingerprint` from each `<trace>`.
2. Looks up the fingerprint in the oracle key registry (or a locally cached copy).
3. Finds the companion receipt in `receipts/` (linked by filename convention).
4. Verifies the receipt signature using the public key identified by the fingerprint.
5. If the signature is valid and the key was not revoked at the time of signing, the verdict is confirmed.

This protocol works offline (no live oracle needed) as long as the verifier has a cached copy of the key registry.

### Fingerprint Format Details

```
"SHA256:" + base64url(SHA-256(public_key_bytes))
```

- `SHA-256` produces 32 bytes.
- `base64url` encoding (RFC 4648 §5, no padding) produces 43 characters.
- Prefix "SHA256:" makes the hash algorithm explicit and follows `ssh-keygen -l -E sha256` convention.
- Total length: 50 characters.

Example computation in Rust:

```rust
use sha2::{Sha256, Digest};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

fn oracle_key_fingerprint(public_key_bytes: &[u8]) -> String {
    let hash = Sha256::digest(public_key_bytes);
    format!("SHA256:{}", URL_SAFE_NO_PAD.encode(hash))
}
```

### Pre-Adjudication Placeholder

Before oracle adjudication, the fingerprint attribute is set to `"pending"`:

```xml
<string key="cargoCI:oracle_key_fingerprint" value="pending"/>
```

After oracle adjudication completes, cargo-cicd (via `Wasm4pmShell`) rewrites the XES file with the actual fingerprint. The rewrite is atomic (write to tmp, rename).

If oracle adjudication does not occur (offline environment, `Blocked` verdict), the attribute remains `"pending"` and the evidence is not considered fully adjudicated.

---

## Consequences

### Positive

1. **Self-describing evidence**: An archived XES file carries the identity of the oracle that adjudicated it. No external metadata is needed to identify the oracle.

2. **Key compromise containment**: When an oracle key is compromised, verifiers can query historical evidence for all traces that used the compromised fingerprint and flag them for re-adjudication.

3. **Independent verification**: Third-party auditors can verify oracle signatures without cargo-cicd. They need only the XES file, the companion receipt, and a copy of the oracle public key (retrieved from the key registry by fingerprint).

4. **Audit trail integrity**: The fingerprint is part of the signed evidence content (the XES trace is what the oracle signs over). An attacker who modifies the fingerprint after signing invalidates the oracle signature.

5. **Multi-oracle support**: Phase 2 distributed oracle consensus (see ADR-016) naturally extends this: each trace can carry fingerprints from M oracles, and verification requires M valid signatures.

6. **Key rotation tracking**: Multiple receipts referencing the same evidence can use different key fingerprints if the oracle rotates its key. Historical receipts remain valid for their original key.

### Negative

1. **XES file rewrite**: Embedding the fingerprint after oracle adjudication requires rewriting the XES file. This is more complex than write-once emission. Mitigation: atomic rename makes the rewrite safe.

2. **Pre-adjudication incompleteness**: XES files in the `pending` state are not fully adjudicated. A partial-adjudication crash could leave files in an inconsistent state. Mitigation: `evidence doctor` verb detects and reports pending-state files.

3. **Key registry dependency**: While the fingerprint is in the evidence, verifiers need the public key to verify signatures. The key registry is an external dependency. Mitigation: Keys can be distributed via package metadata in Cargo.toml [evidence] section (see ADR-014).

4. **Fingerprint format stability**: If the fingerprint algorithm (SHA-256) is later deprecated (e.g., SHA-3 preferred), old evidence files will use SHA-256 fingerprints while new ones use SHA-3. Mitigation: The "SHA256:" prefix makes the algorithm explicit; verifiers must handle multiple algorithms.

---

## Key Registry Design (Reference)

The oracle key registry is a simple JSON document served at a stable URL:

```json
{
  "schema": "oracle-key-registry/v1",
  "oracles": [
    {
      "id": "wasm4pm-prod-2026",
      "provider": "wasm4pm.rs",
      "version": "0.9.x",
      "fingerprint": "SHA256:Bz3k9XvR7mNpYqW2LcT5uE8dFsGhJvKlMoNpQrStUv0=",
      "public_key_base64": "<base64-encoded Ed25519 public key>",
      "valid_from": "2026-01-01T00:00:00Z",
      "valid_until": "2027-01-01T00:00:00Z",
      "revoked": false,
      "capabilities": ["process-conformance", "receipt-doctor"]
    }
  ]
}
```

The registry is versioned with a content hash, enabling offline cached copies to be validated for integrity.

---

## Relationship to Other ADRs

| ADR | Relationship |
|-----|-------------|
| ADR-011 (XES v2 Format) | Fingerprint is an attribute within XES traces |
| ADR-012 (Oracle Architecture) | Fingerprint identifies which external oracle adjudicated |
| ADR-016 (Distributed Oracle Consensus) | Multiple fingerprints in one trace for M-of-N oracles |

---

## References

- SSH public key fingerprint convention: `ssh-keygen -l -E sha256`
- Ed25519 key format: RFC 8032
- Base64url encoding: RFC 4648 §5
- XES trace attributes: ISO/IEC 20880:2013 §4.2

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial draft |
