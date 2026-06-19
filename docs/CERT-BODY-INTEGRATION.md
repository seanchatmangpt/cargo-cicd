# Certification Body Integration Guide

**Audience:** Organisations that want to become a registered cargo-cicd certification provider.  
**Revision:** Vision 2030 Phase 1 (2026-06-17)  
**Version:** 26.6.19

---

## Overview

cargo-cicd emits process evidence in XES (XML Event Stream) format. External certification bodies adjudicate this evidence and issue receipts. This guide describes the technical requirements, SLA commitments, accreditation steps, and receipt format specification for organisations wishing to become certification providers.

---

## Technical Requirements

### Oracle API

Your organisation must expose an oracle endpoint that:

1. **Accepts XES files** — the oracle must parse IEC 62439-compliant XES 2.0 format, as emitted by `cargo cicd evidence doctor`.
2. **Returns a structured verdict** — the response must be machine-parseable (JSON), containing at minimum:
   - `verdict` — one of `"Accept"`, `"Refuse"`, or `"Blocked"`
   - `receipt_hash` — SHA-256 hex of the submitted XES content (prefixed `"sha256:"`)
   - `issued_at` — ISO 8601 timestamp
   - `cert_body_id` — your registered identifier
3. **Signs responses** — responses must be signed with an Ed25519 key. The corresponding public key fingerprint (SHA-256 hex of the DER-encoded public key) is registered in the cargo-cicd certification body registry.
4. **Supports batch submission** — the oracle must handle at minimum 50 XES files per request.

### Receipt Format Specification

The receipt issued by your oracle must be a JSON file conforming to this schema:

```json
{
  "$schema": "https://cargo-cicd.dev/schemas/receipt/v1.json",
  "receipt_version": "1.0",
  "receipt_hash": "sha256:<hex>",
  "cert_body_id": "<your-id>",
  "cert_body_name": "<display name>",
  "issued_at": "<ISO 8601>",
  "verdict": "Accept",
  "evidence_files": [
    {
      "file": "evt-status-show-<ts>.xes",
      "sha256": "<hex>",
      "verdict": "Accept"
    }
  ],
  "standards": ["IEC 61508 SIL 2"],
  "signature": "<base64 Ed25519 signature over receipt JSON minus this field>"
}
```

**Validation:** receipts must pass `wpm receipt doctor --format json --strict <receipt.json>` before being accepted by the cargo-cicd release gate.

### XES Evidence Format

cargo-cicd emits XES 2.0 files in the following structure:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <string key="case_id" value="status_show_phase"/>
    <event>
      <string key="event_id"              value="evt-status-show-20260617120000000Z"/>
      <string key="timestamp"             value="2026-06-17T12:00:00.000Z"/>
      <string key="lifecycle_transition"  value="complete"/>
      <string key="workspace_id"          value="cargo-cicd"/>
      <string key="repo_path"             value="/home/user/cargo-cicd"/>
      <string key="command"               value="status show"/>
      <string key="verdict_claimed"       value="PASS"/>
      <string key="trace_class"           value="live_workspace"/>
    </event>
  </trace>
</log>
```

Your oracle must extract `event_id`, `command`, `verdict_claimed`, `workspace_id`, and `timestamp` when issuing a verdict.

---

## SLA Requirements

| Metric | Minimum requirement |
|---|---|
| **Response time (P99)** | < 30 seconds per XES file |
| **Batch response time (50 files)** | < 5 minutes |
| **Availability** | 99.5% monthly uptime |
| **Blocked verdict TTL** | Blocked verdicts expire after 72 hours; re-submission must be supported |
| **Receipt retention** | Receipts must be retrievable for at least 7 years |
| **Key rotation** | Public key rotation must be announced 90 days in advance via the cargo-cicd registry update process |

---

## Accreditation Process

Follow these steps to register your organisation as a cargo-cicd certification provider:

### Step 1 — Submit a Technical Application

Open a pull request against the `cargo-cicd` repository adding your entry to `src/certification/mod.rs` inside `known_cert_bodies()`:

```rust
CertificationBody {
    id: "your-org-id".to_string(),
    name: "Your Organisation Name".to_string(),
    submission_url: "https://your-org.example.com/cargo-cicd-oracle/".to_string(),
    standards: vec![
        ComplianceStandard::Iec61508 { sil_level: 2 },
        // add further standards you certify
    ],
    oracle_fingerprint: "<SHA-256 hex of your Ed25519 public key DER>".to_string(),
}
```

The pull request must include:
- Your oracle's public key in `receipts/cert-bodies/<your-id>.pub.pem`
- A sample receipt demonstrating the schema above
- Evidence that your oracle can process the canonical XES test fixture at `tests/fixtures/canonical_evidence.xes`

### Step 2 — Oracle Verification

The cargo-cicd maintainers will submit the canonical test fixture to your oracle endpoint. The returned verdict and receipt must:
- Be valid JSON conforming to the receipt schema
- Pass `wpm receipt doctor --format json --strict`
- Have a valid Ed25519 signature over the receipt payload using your registered public key

### Step 3 — Standards Scope Review

For each standard you claim to support (IEC 61508, ISO 26262, DO-178C, FDA 21 CFR Part 11), you must provide:
- Accreditation certificate or equivalent from the relevant standards body
- Description of the adjudication methodology (what checks the oracle performs against the XES evidence)
- Sample acceptance and refusal scenarios with justification

### Step 4 — SLA Commitment

Sign the cargo-cicd Certification Body SLA (available at `docs/CERT-BODY-SLA-TEMPLATE.md`) and submit it alongside your pull request. The SLA covers response time, availability, key rotation, and receipt retention.

### Step 5 — Merge and Activation

Once Steps 1–4 are complete and the pull request is merged, your organisation is active in the registry. Users can discover your body via:

```rust
use cargo_cicd::certification::{bodies_for_standard, ComplianceStandard};

let bodies = bodies_for_standard(&ComplianceStandard::Iec61508 { sil_level: 2 });
for body in &bodies {
    println!("{}: {}", body.name, body.submission_url);
}
```

---

## Oracle Public Key Registration

Your Ed25519 public key must be committed to the repository under `receipts/cert-bodies/`:

```
receipts/cert-bodies/<your-id>.pub.pem
```

Format: PEM-encoded SubjectPublicKeyInfo (SPKI) — the same format produced by:

```sh
openssl genpkey -algorithm ed25519 -out private.pem
openssl pkey -in private.pem -pubout -out receipts/cert-bodies/<your-id>.pub.pem
```

The `oracle_fingerprint` field in your `CertificationBody` entry must equal:

```sh
openssl pkey -in receipts/cert-bodies/<your-id>.pub.pem -pubin -outform DER \
  | openssl dgst -sha256 -hex | awk '{print $2}'
```

---

## Example Integration (Rust)

The following pseudocode illustrates how a certification body oracle can be integrated into a cargo-cicd release pipeline:

```rust
use std::path::Path;

/// Submit XES files to the oracle and collect the receipt.
///
/// In production, replace this with an HTTP client call to your oracle endpoint.
pub fn submit_to_oracle(
    xes_files: &[&Path],
    oracle_url: &str,
    api_key: &str,
) -> Result<OracleReceipt, Box<dyn std::error::Error>> {
    // 1. Read and hash each XES file
    let mut payloads = Vec::new();
    for path in xes_files {
        let content = std::fs::read(path)?;
        let hash = sha256_hex(&content);
        payloads.push(EvidencePayload { path: path.to_path_buf(), sha256: hash, content });
    }

    // 2. POST to oracle endpoint
    // let response = http_client
    //     .post(oracle_url)
    //     .header("Authorization", format!("Bearer {}", api_key))
    //     .json(&payloads)
    //     .send()?;

    // 3. Deserialize and verify receipt signature
    // let receipt: OracleReceipt = response.json()?;
    // verify_ed25519_signature(&receipt, &ORACLE_PUBLIC_KEY)?;

    // 4. Write receipt to receipts/
    // std::fs::write("receipts/receipt.json", serde_json::to_string_pretty(&receipt)?)?;

    todo!("implement HTTP client call")
}

pub struct OracleReceipt {
    pub verdict: String,         // "Accept" | "Refuse" | "Blocked"
    pub receipt_hash: String,    // "sha256:..."
    pub cert_body_id: String,
    pub issued_at: String,
    pub signature: String,       // base64 Ed25519 signature
}

fn sha256_hex(data: &[u8]) -> String {
    // Use a SHA-256 implementation appropriate for your project
    format!("sha256:{}", hex_encode(sha256(data)))
}
```

---

## Contact Process

To begin the accreditation process:

1. Open a GitHub issue titled "Certification Body Application: [Your Org Name]" on the cargo-cicd repository.
2. Include your organisation name, supported standards, oracle endpoint URL (staging), and oracle public key fingerprint.
3. A maintainer will respond within 5 business days to schedule the verification step.
4. For urgent certifications (e.g., safety-critical release deadlines), email the maintainers directly via the address in `Cargo.toml`.

---

## Registered Certification Bodies

See `src/certification/mod.rs` — `known_cert_bodies()` — for the current list of registered providers and their supported standards.

The CLI command to list registered bodies and their submission URLs is:

```sh
# (planned for Vision 2030 Phase 2)
cargo cicd evidence cert-bodies
```
