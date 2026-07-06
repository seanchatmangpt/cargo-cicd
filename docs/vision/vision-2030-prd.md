# Vision 2030 Product Requirements Document

**Extracted from thesis section 5.1: "The Ecosystem in 2030"**

**Date:** 2026-06-17  
**Status:** Vision Statement → Executable Requirements  
**Scope:** Process evidence as first-class artifact, publication gates, ecosystem integration

---

## Feature 1: Process Evidence as Cargo.toml Metadata Field

**Title:** Evidence Section in Cargo.toml — Manifest-Level Evidence Registration

**User Story:**
As a crate maintainer,  
I want to declare evidence metadata in my package manifest,  
So that cargo and downstream tools can surface verification status, evidence archive location, oracle public key, and receipt hash without requiring out-of-band configuration.

**Acceptance Criteria:**
- Cargo.toml supports a new `[evidence]` section with the following keys:
  - `archive_url` (string): HTTPS URL where the evidence archive (XES/JSONL) is permanently stored
  - `oracle_key` (string): Base64-encoded public key of the oracle that adjudicated this release
  - `receipt_hash` (string, SHA-256 hex): Cryptographic hash of the wasm4pm receipt
  - `version` (string): Version of the evidence schema (e.g., "1.0")
  - `timestamp` (string, ISO-8601): When the evidence was adjudicated
- `cargo metadata` outputs the `[evidence]` section when querying package metadata
- `cargo tree` displays a `VERIFIED` or `UNVERIFIED` status badge next to dependencies that have evidence metadata
- `cargo audit` can verify evidence metadata presence and receipt hash validity
- The schema is documented in RFC or Cargo enhancement proposal
- Backward compatibility: packages without `[evidence]` section remain publishable but flagged as `UNVERIFIED`

**Example Structure:**
```toml
[package]
name = "my-crate"
version = "1.0.0"

[evidence]
archive_url = "https://evidence.my-org.com/my-crate/1.0.0/evidence.tar.gz"
oracle_key = "base64encodedpublickey..."
receipt_hash = "sha256:deadbeef..."
version = "1.0"
timestamp = "2030-06-14T13:45:07Z"
```

**Dependencies:**
- Feature 6 (Oracle public key embedding)
- Feature 7 (Receipt hash validation)
- Feature 5 (Evidence archive URLs and metadata)

**Effort:** M (Cargo.toml schema extension + cargo metadata output + cargo tree display + cargo audit integration)

**Priority:** P0 (Blocking ecosystem adoption signal)

---

## Feature 2: XES Event Stream Generation and Structure

**Title:** XES 2.0 Compliance — Process Evidence as Industry-Standard XML Event Streams

**User Story:**
As a process mining analyst or conformance checking system,  
I want every cargo-cicd command to emit XES-compliant XML event streams,  
So that standard process mining tools (Disco, ProM, Celonis) can consume workspace health evidence without custom parsers.

**Acceptance Criteria:**
- Every ProcessEvent in cargo-cicd serializes to XES 2.0 format (ISO/IEC 20880:2013)
- XES root element is `<log>` with version="1.0" and xmlns declarations
- Each command invocation groups related events into a single `<trace>` element with a `case_id` attribute
- Case IDs follow pattern: `{workspace_id}_{command_noun}_{verb}_{ISO8601_date}`
- Each event has mandatory attributes:
  - `event_id` (string): Globally unique identifier (e.g., "evt-status-show-20260614134507123Z")
  - `timestamp` (string, ISO-8601 UTC): "2030-06-14T13:45:07.123Z"
  - `lifecycle_transition` (enum): "start" or "complete"
  - `event_name` (string): "{noun}:{verb}" (e.g., "status:show")
  - `verdict_claimed` (enum): "PASS", "WARN", "FAIL"
- Optional attributes on completion events:
  - `duration_ms` (integer): Milliseconds elapsed between start and complete
  - `verdict_adjudicated` (enum): "Accept", "Refuse", "Blocked" (only after oracle call)
  - `adjudicated_at` (string, ISO-8601): When the oracle returned its verdict
  - `oracle_command` (string): The exact `wpm audit` command invoked
  - `trace_class` (enum): "live_workspace" or "pipeline_run" (distinguishes local vs CI context)
- Workspace metadata is included as top-level `<string>` attributes on the trace:
  - `workspace_id`
  - `workspace_root`
  - `git_branch`
  - `git_commit_sha`
  - `toolchain_version`
  - `cargo_version`
  - `os_version`
  - `session_id` (unique per cargo-cicd process invocation)
- XES files are written to `target/cargo-cicd/evidence/evt-{event_id}.xes`
- Multiple traces from a single workspace session can be concatenated; the concatenated file remains valid XES
- Test: Parse generated XES files with standard XES library; verify ProM/Disco can ingest without errors

**XES Document Template:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0" xes.features="org.processmining.framework.packages.GraphicsPackage" 
     openlog.version="1.0" 
     xmlns:xes="http://www.xes-standard.org/" 
     xmlns:org="http://www.xes-standard.org/org">
  <extension name="Organizational" prefix="org" uri="http://www.xes-standard.org/org.xesext"/>
  <extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>
  <extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Lifecycle" prefix="lifecycle" uri="http://www.xes-standard.org/lifecycle.xesext"/>
  
  <trace>
    <string key="concept:name" value="status_show_2030-06-14"/>
    <string key="case_id" value="workspace_abc123_status_show_20300614T134507Z"/>
    <string key="workspace_id" value="workspace_abc123"/>
    <string key="workspace_root" value="/home/user/my-project"/>
    <string key="git_branch" value="main"/>
    <string key="git_commit_sha" value="deadbeef..."/>
    <string key="toolchain_version" value="1.75.0"/>
    <string key="trace_class" value="live_workspace"/>
    
    <event>
      <string key="event_id" value="evt-status-show-20300614134507123Z"/>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="start"/>
      <date key="time:timestamp" value="2030-06-14T13:45:07.123Z"/>
    </event>
    
    <event>
      <string key="event_id" value="evt-status-show-20300614134507456Z"/>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="complete"/>
      <date key="time:timestamp" value="2030-06-14T13:45:07.456Z"/>
      <int key="duration_ms" value="333"/>
      <string key="verdict_claimed" value="PASS"/>
      <string key="verdict_adjudicated" value="Accept"/>
      <date key="time:adjudicated_at" value="2030-06-14T13:45:08.100Z"/>
      <string key="trace_class" value="live_workspace"/>
    </event>
  </trace>
</log>
```

**Dependencies:**
- Feature 3 (JSONL companion format)
- Feature 4 (Process mining compatibility)

**Effort:** L (Rigorous XES 2.0 compliance, validation with external tools, comprehensive attribute mapping)

**Priority:** P0 (Core thesis requirement; without XES, there is no evidence artifact)

---

## Feature 3: JSONL Companion Format — Machine-Readable Event Archive

**Title:** JSONL Event Stream Companion — Same Events, Alternative Encoding

**User Story:**
As a downstream system or CI/CD pipeline,  
I want to consume the same process events that XES encodes, but in JSONL format,  
So that I can parse and analyze them without XML parsing libraries or external tools.

**Acceptance Criteria:**
- For every XES file generated, a companion `.jsonl` file is written to the same directory
- File naming: `evt-{event_id}.jsonl` (mirrors XES filename)
- Each line is a valid JSON object representing a single ProcessEvent
- JSON schema has these top-level keys:
  - `event_id` (string)
  - `timestamp` (string, ISO-8601)
  - `case_id` (string)
  - `command` (string): "{noun} {verb}"
  - `verdict_claimed` (string): "PASS", "WARN", "FAIL"
  - `duration_ms` (integer, nullable): Only present on "complete" events
  - `verdict_adjudicated` (string, nullable): Present only if oracle called
  - `lifecycle` (string): "start" or "complete"
  - `workspace_id` (string)
  - `workspace_root` (string)
  - `git_branch` (string)
  - `git_commit_sha` (string)
  - `toolchain_version` (string)
  - `trace_class` (string): "live_workspace" or "pipeline_run"
- Event order in JSONL matches event order in XES (start, then complete)
- JSONL is UTF-8 encoded, with newlines only between JSON objects (no trailing newline after last object)
- Test: Parse both XES and JSONL from same command invocation; verify both encode identical event sets
- Test: Feed JSONL to a JSON parser; verify no parse errors

**JSONL Document Example:**
```jsonl
{"event_id":"evt-status-show-20300614134507123Z","case_id":"workspace_abc123_status_show_20300614T134507Z","command":"status show","lifecycle":"start","timestamp":"2030-06-14T13:45:07.123Z","verdict_claimed":"PASS","workspace_id":"workspace_abc123","workspace_root":"/home/user/my-project","git_branch":"main","git_commit_sha":"deadbeef...","toolchain_version":"1.75.0","trace_class":"live_workspace"}
{"event_id":"evt-status-show-20300614134507456Z","case_id":"workspace_abc123_status_show_20300614T134507Z","command":"status show","lifecycle":"complete","timestamp":"2030-06-14T13:45:07.456Z","verdict_claimed":"PASS","verdict_adjudicated":"Accept","duration_ms":333,"adjudicated_at":"2030-06-14T13:45:08.100Z","workspace_id":"workspace_abc123","workspace_root":"/home/user/my-project","git_branch":"main","git_commit_sha":"deadbeef...","toolchain_version":"1.75.0","trace_class":"live_workspace"}
```

**Dependencies:**
- Feature 2 (XES generation)

**Effort:** M (Parallel JSON serialization pathway, schema definition and validation)

**Priority:** P0 (Lowers barrier to evidence consumption; critical for downstream integration)

---

## Feature 4: Process Mining Compatibility — BPMN and Conformance Checking

**Title:** Process Mining Tool Integration — XES Evidence as Input to Disco/ProM/Celonis

**User Story:**
As an operations or quality engineer,  
I want to import cargo-cicd evidence archives into standard process mining tools,  
So that I can visualize actual vs. declared development processes, identify bottlenecks, and verify conformance to organizational policies.

**Acceptance Criteria:**
- Generated XES files conform to XES 2.0 standard (testable by importing into ProM or Disco without errors)
- XES includes `org:resource` (developer username) and `org:role` (e.g., "maintainer", "contributor") attributes where applicable
- XES includes custom extensions for cargo-cicd-specific dimensions:
  - Toolchain version
  - Target directory size
  - Changed file count
  - Test count estimates
  - Git phase state
- Generated evidence is compatible with BPMN 2.0 conformance checking frameworks
  - Implicit BPMN model: "declare status clean" → "run tests" → "publish" → "emit receipt"
  - Evidence can be checked against this model to verify process was followed
- BPMN process diagram can be auto-generated from evidence (future tooling; structure in place now)
- Effort to import a single evidence archive into Disco/ProM ≤ 30 seconds
- All standard process mining metrics can be computed:
  - Throughput time (time from first event to last event)
  - Activity duration distribution
  - Variant frequency (which sequences of activities occurred most often)
  - Dotted chart (event timeline visualization)
- Test: Generate evidence from 10 cargo-cicd invocations; import XES into ProM and Disco; verify no errors; compute throughput time

**Dependencies:**
- Feature 2 (XES event structure)

**Effort:** M (XES schema extension with org/trace extensions; test with real process mining tools)

**Priority:** P1 (Ecosystem enabler; enables process analytics and conformance dashboards)

---

## Feature 5: Evidence Archive URLs and Metadata

**Title:** Evidence Archival Infrastructure — Durable, Addressable, Verifiable Evidence Artifacts

**User Story:**
As a crate publisher or safety auditor,  
I want to permanently store and address evidence archives by content hash,  
So that I can retrieve, verify, and replay the exact evidence that led to a publication decision, even years later.

**Acceptance Criteria:**
- Every successful `cargo cicd publish run` writes the evidence archive (XES + JSONL + receipt) to a configured archive location
- Archive format: tarball containing:
  - `evidence/` directory (all .xes and .jsonl files)
  - `receipt.json` (the wasm4pm receipt)
  - `manifest.json` with metadata:
    - `archive_version` (string): "1.0"
    - `created_at` (ISO-8601): Timestamp of archive creation
    - `crate_name` (string)
    - `crate_version` (string)
    - `archive_hash` (string, SHA-256 hex): Hash of the entire tarball
    - `event_count` (integer): Number of events in evidence
    - `oracle_name` (string): Name of the oracle that adjudicated
    - `oracle_pubkey` (string, base64): Public key used for receipt signature
- Archive is named `{crate_name}-{version}-evidence-{archive_hash}.tar.gz`
- Archive URL is configured via:
  - `[evidence]` section in Cargo.toml (per-crate archive location)
  - OR environment variable `CARGO_CICD_EVIDENCE_ARCHIVE_URL`
  - OR global `~/.cargo/config.toml` setting
- Archive is immutable: once written with hash H, re-writing with different content fails
- Archive supports HTTPS download with integrity verification (SHA-256)
- Test: Create evidence archive; compute SHA-256; rename to include hash; download and verify hash matches
- Test: Attempt to write archive with same name but different content; verify failure

**manifest.json Example:**
```json
{
  "archive_version": "1.0",
  "created_at": "2030-06-14T13:45:07Z",
  "crate_name": "my-crate",
  "crate_version": "1.0.0",
  "archive_hash": "sha256:deadbeef0123456789abcdef...",
  "event_count": 4,
  "oracle_name": "wasm4pm-official",
  "oracle_pubkey": "base64encodedpublickey...",
  "events": [
    {
      "event_id": "evt-status-show-20300614134507123Z",
      "command": "status show"
    },
    {
      "event_id": "evt-publish-run-20300614134507456Z",
      "command": "publish run"
    }
  ]
}
```

**Dependencies:**
- Feature 2 (XES/JSONL generation)
- Feature 6 (Oracle public key embedding)

**Effort:** L (Archive creation, manifest generation, integrity verification, storage backend configuration)

**Priority:** P0 (Required for immutable evidence trail; audit/compliance critical)

---

## Feature 6: Oracle Public Key Embedding and Verification

**Title:** Cryptographic Evidence Verification — Public Key Pinning in Manifests

**User Story:**
As a package manager or auditor,  
I want to verify that a receipt was signed by the exact oracle I trust,  
So that I can reject receipts from compromised or unauthorized oracles.

**Acceptance Criteria:**
- The wasm4pm oracle that adjudicates evidence has an associated public key
- Public key is embedded in:
  - `Cargo.toml` `[evidence]` section (`oracle_key` field, base64-encoded)
  - `cicd.toml` `[state]` section (`oracle_pubkey`)
  - Receipt JSON under top-level key `oracle_pubkey`
- Public key format: Ed25519 (32 bytes, base64-encoded) or RSA-2048 (PEM-encoded)
- cargo-cicd verifies every receipt signature:
  - Load public key from Cargo.toml or `cicd.toml`
  - Parse receipt JSON
  - Extract signature from receipt (`signature` field)
  - Verify signature over receipt payload using public key
  - If verification fails, mark receipt as `UNVERIFIED` and refuse publication
- `cargo audit` can verify oracle public keys against a configurable trust store:
  - Default trust store: official wasm4pm public keys (pinned in cargo-audit)
  - Custom trust store: `~/.cargo/oracle_keys.toml`
- Mismatch scenarios:
  - Receipt signed by oracle A, but Cargo.toml specifies oracle B key → `UNVERIFIED`
  - Receipt has no signature → `UNVERIFIED`
  - Receipt has invalid signature format → `UNVERIFIED`
- Test: Create a receipt signed by private key; verify signature with public key; tamper with receipt; verify signature fails

**Dependencies:**
- Feature 1 (Cargo.toml evidence metadata)
- Feature 5 (Evidence archives)

**Effort:** M (Ed25519 signature verification, public key management, trust store configuration)

**Priority:** P1 (Supply chain security; required for trust model)

---

## Feature 7: Receipt Hash Validation

**Title:** Evidence Integrity Verification — SHA-256 Content Hashing

**User Story:**
As a package auditor or crates.io ingestion system,  
I want to verify that a receipt has not been tampered with after signature,  
So that I can ensure the receipt matches the exact evidence that was adjudicated.

**Acceptance Criteria:**
- Every receipt JSON includes a `receipt_hash` field: SHA-256 hex hash of the evidence archive
- cargo-cicd computes and verifies the hash:
  - After writing evidence archive, compute SHA-256 of the tarball
  - Include hash in receipt JSON before signing
  - Sign the complete receipt (including hash)
  - On verification: recompute hash of archive; compare to hash in signed receipt
  - If hashes do not match, mark receipt as `INTEGRITY_FAILED`
- Cargo.toml `[evidence]` section includes `receipt_hash` for quick lookups
- crates.io ingestion pipeline verifies:
  - Receipt is signed by a trusted oracle
  - Receipt hash matches the evidence archive hash
  - Both must pass before the crate version is marked `VERIFIED`
- Test: Create receipt with hash H1 for archive A; modify archive; recompute hash H2; verify H1 != H2 and receipt validation fails
- Test: Download published crate; verify receipt_hash in Cargo.toml matches receipt_hash in receipt.json

**Receipt with Hash Example:**
```json
{
  "event_id": "evt-publish-run-20300614134507456Z",
  "verdict": "Accept",
  "receipt_hash": "sha256:deadbeef0123456789abcdef...",
  "signature": "base64encodedsignature...",
  "signed_at": "2030-06-14T13:45:08.100Z"
}
```

**Dependencies:**
- Feature 5 (Evidence archives)
- Feature 6 (Receipt signing and verification)

**Effort:** M (Hash computation, receipt structure, signature over hash)

**Priority:** P0 (Supply chain integrity; critical for trust model)

---

## Feature 8: Process State Versioning in Cargo.toml

**Title:** Process Model Versioning — Backward Compatibility for Evidence Schema Changes

**User Story:**
As an ecosystem maintainer or conformance checker,  
I want to version the evidence schema and process model,  
So that I can support multiple versions of cargo-cicd simultaneously and detect incompatible evidence formats.

**Acceptance Criteria:**
- `Cargo.toml` `[evidence]` section includes `version` field (string): "1.0", "2.0", etc.
- `version` indicates the schema version of the XES, JSONL, and receipt formats
- cargo-cicd refuses to accept a receipt for a crate that declares `[evidence] version="2.0"` if cargo-cicd only supports "1.0"
  - Error message: "This crate requires process evidence format v2.0; your cargo-cicd supports v1.0. Upgrade: cargo install cargo-cicd@latest"
- wasm4pm oracle reports its supported version:
  - `wpm --version` output includes "evidence-schema: v1.0"
  - Mismatch between oracle version and crate version → oracle returns `Blocked`
- cicd.toml `[state]` section includes `evidence_version` (string): tracks which version was used to generate evidence
- Changelog/migration docs for each version bump:
  - v1.0 → v2.0: new event attributes added, XES schema extended
  - All v1.0 evidence remains valid under v2.0 (additive only)
- Test: Publish crate with `[evidence] version="1.0"`; generate evidence; verify cicd.toml records version; simulate future cargo-cicd v2.0 parsing v1.0 evidence with backward-compatibility

**Dependencies:**
- Feature 1 (Cargo.toml evidence metadata)
- Feature 2 (XES structure)

**Effort:** S (Add version field, version check logic, forward-compatibility declaration)

**Priority:** P1 (Ecosystem sustainability; required for long-term evolution)

---

## Feature 9: Evidence Collection and Storage Infrastructure

**Title:** Evidence Emission, Collection, Durability — Complete Evidence Pipeline

**User Story:**
As a cargo-cicd user or CI/CD operator,  
I want to collect evidence from multiple invocations over time,  
And securely store it for audit, compliance, and process mining,  
So that I can build a complete history of workspace health and publication decisions.

**Acceptance Criteria:**

### 9a: Local Evidence Collection
- Every cargo-cicd command writes events to `target/cargo-cicd/evidence/` directory
- Events are written immediately (not buffered) to `.xes` and `.jsonl` files
- Directory structure:
  ```
  target/cargo-cicd/evidence/
  ├── evt-status-show-20300614134507123Z.xes
  ├── evt-status-show-20300614134507123Z.jsonl
  ├── evt-publish-run-20300614134507456Z.xes
  ├── evt-publish-run-20300614134507456Z.jsonl
  ├── receipts/
  │   ├── evt-status-show-20300614134507123Z.receipt.json
  │   └── evt-publish-run-20300614134507456Z.receipt.json
  └── manifest.json
  ```
- `manifest.json` at root lists all events collected in this session:
  ```json
  {
    "session_id": "session_01KMKyN6RR1575MdqzqaGwWG",
    "workspace_id": "workspace_abc123",
    "created_at": "2030-06-14T13:45:00Z",
    "event_count": 2,
    "events": [
      {
        "event_id": "evt-status-show-20300614134507123Z",
        "command": "status show",
        "xes_path": "evt-status-show-20300614134507123Z.xes",
        "jsonl_path": "evt-status-show-20300614134507123Z.jsonl",
        "receipt_path": "receipts/evt-status-show-20300614134507123Z.receipt.json"
      }
    ]
  }
  ```

### 9b: Evidence Archival
- At publication time (`cargo cicd publish run`), the entire evidence directory is tarred:
  ```sh
  tar czf my-crate-1.0.0-evidence-{archive_hash}.tar.gz \
    target/cargo-cicd/evidence/ \
    cicd.toml \
    Cargo.toml
  ```
- Archive is uploaded to configured storage backend (S3, GitHub Releases, HTTP, local filesystem)
- Archive URL is recorded in Cargo.toml `[evidence]` section

### 9c: Storage Backend Configuration
- Three built-in backends:
  1. **local** — Write to local filesystem path (for monorepos, testing)
     ```toml
     [evidence]
     backend = "local"
     archive_path = "/mnt/evidence-archive/"
     ```
  2. **github-release** — Upload to GitHub Releases asset
     ```toml
     [evidence]
     backend = "github-release"
     token_env = "GITHUB_TOKEN"
     ```
  3. **s3** — Upload to AWS S3
     ```toml
     [evidence]
     backend = "s3"
     bucket = "my-org-evidence"
     region = "us-east-1"
     ```
- Backend is configured in:
  - `Cargo.toml` `[evidence]` section (per-crate)
  - OR `~/.cargo/config.toml` `[evidence]` (global default)
  - OR environment variable `CARGO_CICD_EVIDENCE_BACKEND`
- Fallback: if no backend configured, archive is written to `target/` and user must manually upload

### 9d: Evidence Retention and Expiry
- Local evidence in `target/cargo-cicd/evidence/` is never automatically deleted
- User can manually prune via:
  ```sh
  cargo cicd evidence prune --older-than 90d
  ```
- Archived evidence has configurable retention:
  - Default: permanent (no expiry)
  - Per-backend: S3 lifecycle policies, GitHub Release auto-delete, local cleanup
- cicd.toml `[state]` tracks retention policy:
  ```toml
  [state.evidence]
  retention_days = 0  # 0 = permanent
  last_archived = "2030-06-14T13:45:07Z"
  ```

### 9e: Evidence Integrity Monitoring
- On read, cargo-cicd verifies:
  - File timestamps are monotonic (events in order)
  - No events are missing from sequence
  - Receipt signatures are valid
  - Archive hashes match manifests
- Any integrity failure triggers a diagnostic:
  ```
  WARNING: Evidence archive has gaps; one or more events may be missing.
  Last event: evt-publish-run-20300614T134507Z at 13:45:08Z
  Current time: 2030-06-15T10:00:00Z (gap of 20.25h)
  
  This is expected if cargo-cicd was not run during this period.
  Diagnosis: run `cargo cicd evidence doctor` to verify.
  ```

- Test: Create evidence files; delete one; run `cargo cicd evidence doctor`; verify gap detected
- Test: Create archive; modify receipt inside archive; attempt to verify; fail with integrity error

**Dependencies:**
- Feature 2 (XES generation)
- Feature 3 (JSONL generation)
- Feature 5 (Evidence archives)

**Effort:** XL (Multiple storage backends, manifest generation, integrity monitoring, retention policies)

**Priority:** P0 (Core infrastructure; without this, evidence cannot accumulate or be trusted)

---

## Feature 10: cicd.toml as Process State Carrier

**Title:** cicd.toml — Persistent Process State Artifact Alongside Cargo.lock

**User Story:**
As a developer or auditor,  
I want to track the workspace process state persistently,  
So that I can see the complete history of integration events, publication decisions, and policy evaluations in version control.

**Acceptance Criteria:**
- `cicd.toml` is a TOML file at the workspace root, tracked in git (alongside Cargo.toml, Cargo.lock)
- Structure mirrors EngineState:
  ```toml
  [workspace]
  name = "my-crate"
  root_path = "/home/user/my-project"
  members = ["."]
  
  [state]
  git_phase = "clean"
  git_branch = "main"
  git_commit_sha = "deadbeef..."
  git_behind_count = 0
  git_ahead_count = 0
  target_size_bytes = 524288000
  toolchain_version = "1.75.0"
  last_status_check = "2030-06-14T13:45:07Z"
  last_evidence_adjudication = "2030-06-14T13:45:08Z"
  
  [state.evidence]
  version = "1.0"
  retention_days = 0
  last_archived = "2030-06-14T13:45:07Z"
  
  [[events]]
  event_id = "evt-status-show-20300614134507123Z"
  timestamp = "2030-06-14T13:45:07.123Z"
  command = "status show"
  verdict_claimed = "PASS"
  verdict_adjudicated = "Accept"
  adjudicated_at = "2030-06-14T13:45:08.100Z"
  
  [[events]]
  event_id = "evt-publish-run-20300614134507456Z"
  timestamp = "2030-06-14T13:45:07.456Z"
  command = "publish run"
  verdict_claimed = "PASS"
  verdict_adjudicated = "Accept"
  adjudicated_at = "2030-06-14T13:45:08.200Z"
  receipt_hash = "sha256:deadbeef..."
  ```
- cicd.toml is updated after every major command:
  - `cargo cicd status show` → updates `[state]` section
  - `cargo cicd publish run` → appends to `[[events]]` array, updates `[state.evidence.last_archived`
  - `cargo cicd evidence doctor` → updates `[state.evidence]` section
- Format is deterministic (TOML spec; no inline tables, consistent key order)
- Diff is human-readable (useful in git commit messages to see what changed)
- Version control integration:
  - `git diff Cargo.lock cicd.toml` shows what workspace state changed
  - Commit history of cicd.toml is the process history
  - Tags can point to commits where `[state] git_phase = "clean"` and publication succeeded

**Dependencies:**
- Feature 2 (XES generation)
- Feature 3 (JSONL generation)
- Feature 9 (Evidence collection and storage)

**Effort:** M (TOML schema, deterministic serialization, update logic in every verb)

**Priority:** P0 (Source of truth for process state; required for auditability and version control integration)

---

## Cross-Cutting Requirements

### Backward Compatibility
- All new features must be additive (no breaking changes to existing XES/JSONL structure)
- cicd.toml schema is versioned; old versions can be read and auto-upgraded
- Cargo.toml `[evidence]` section is optional; crates without it remain publishable but flagged `UNVERIFIED`

### Documentation
- Comprehensive guide: "Process Evidence for Rust Projects" (analogous to "Rust Book")
- Walkthrough of Feature 5 and 9 (evidence archival) in CI/CD integration docs
- Example Cargo.toml `[evidence]` section in Cargo Book
- RFC for Cargo.toml `[evidence]` schema change

### Tooling Integration
- `cargo metadata --format=json` outputs `[evidence]` fields
- `cargo tree` displays `VERIFIED`/`UNVERIFIED` badges
- `cargo audit` can verify receipt hashes and oracle keys
- `cargo publish` (crates.io) validates `[evidence]` section before accepting upload

### Testing Strategy
- Invariant: XES generated from identical command invocations on identical workspaces must be structurally identical (deterministic)
- Conformance: XES imports successfully into ProM, Disco, Celonis without errors
- Round-trip: XES → JSONL → XES (via converter tool) produces identical output
- Integrity: Tampered evidence/receipt fails verification

### Performance Constraints
- Evidence emission must not add >100ms to any command
- Evidence archival (tarball + upload) must not block publication gate (async is acceptable)
- Manifest parsing must be O(1) or O(log n) (no full XES parse required for quick lookups)

---

## Release Gates (for Vision 2030 Completion)

These features must reach production in this order to avoid breaking the ecosystem:

1. **Milestone 1 (v27.0):** Features 2, 3, 4, 10 (XES/JSONL generation, process mining, cicd.toml)
   - Evidence is emitted; process mining tools can consume it; state is persistent
   - Oracle integration is optional
   
2. **Milestone 2 (v27.1):** Features 5, 6, 7, 9 (Evidence archival, cryptography, infrastructure)
   - Evidence can be permanently stored, signed, and verified
   - Supply chain trust model is in place
   
3. **Milestone 3 (v27.2):** Feature 1, 8 (Cargo.toml metadata, versioning)
   - crates.io and Cargo integrate evidence metadata
   - Evidence schema is versioned for long-term evolution
   - **Ready for ecosystem adoption: "VERIFIED" badges on crates.io**

---

## Success Metrics (by 2030)

- [ ] >50% of published Rust crates include `[evidence]` section in Cargo.toml
- [ ] crates.io surfaces `VERIFIED` badge; adoption correlates with higher download rates
- [ ] Process mining dashboards (Disco, ProM plugins) consume cargo-cicd evidence
- [ ] Regulatory bodies recognize wasm4pm receipts as evidence of process conformance
- [ ] Zero supply chain attacks attributed to "unverified workspace state" (metric: post-publication incidents with evidence)

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-17  
**Extracted From:** cargo-cicd thesis v26.6.2, section 5.1
