# /evidence — Evidence Gate Workflow

Guide through the full cargo-cicd evidence gate: emit process evidence, adjudicate with the wasm4pm oracle, and interpret Accept/Refuse/Blocked verdicts.

---

## Evidence Gate Overview

cargo-cicd emits process evidence (XES + JSONL) for every major operation. The `wpm` oracle (`wasm4pm`) adjudicates that evidence externally. **cargo-cicd never adjudicates itself** (Invariant E1).

Evidence flow:

```
cargo-cicd executes a verb
    |
    v
ProcessEvent emitted (verdict_claimed = "PASS" | "WARN" | "FAIL")
    |
    v
Serialized to XES:  target/cargo-cicd/evidence/evt-*.xes
Serialized to JSONL: target/cargo-cicd/evidence/evt-*.jsonl
    |
    v
wpm oracle called:  wpm audit <evt-*.xes>
    |
    v
Oracle returns:     Accept | Refuse | Blocked
    |
    v
Tests assert on the oracle verdict — never on cargo-cicd internal state
```

---

## Step-by-Step Workflow

### Step 1 — Check the wpm oracle is available

```bash
which wpm
wpm --version
```

If `wpm` is not found:

```bash
# Build wasm4pm from source
cd /path/to/wasm4pm
cargo build --release
export PATH="/path/to/wasm4pm/target/release:$PATH"

# Verify
wpm --version
```

If wpm cannot be made available, proceed with `ExpectedWpmVerdict::Blocked` (see Blocked section below). You cannot close the release gate without a live oracle.

---

### Step 2 — Run the evidence doctor

```bash
cargo cicd evidence doctor
```

This verb:
- Populates `EngineState` from all adapters
- Emits a `ProcessEvent` with `lifecycle_transition = "complete"`
- Writes XES and JSONL files to `target/cargo-cicd/evidence/`
- Reports any structural problems with existing evidence

---

### Step 3 — List emitted XES files

```bash
ls -la target/cargo-cicd/evidence/
```

Expected output shows `.xes` and `.jsonl` pairs:

```
evt-evidence-doctor-20260614134507123Z.xes
evt-evidence-doctor-20260614134507123Z.jsonl
evt-status-show-20260614130000000Z.xes
evt-status-show-20260614130000000Z.jsonl
```

If the directory is empty or missing, evidence was not emitted. Re-run the verb and check for errors:

```bash
RUST_LOG=debug cargo cicd evidence doctor
```

---

### Step 4 — Audit each XES file with the oracle

```bash
wpm audit target/cargo-cicd/evidence/evt-evidence-doctor-*.xes
```

Audit all files at once:

```bash
for f in target/cargo-cicd/evidence/*.xes; do
    echo "--- $f ---"
    wpm audit "$f"
done
```

Or use the evidence audit verb (runs wpm internally when `wasm4pm` feature is enabled):

```bash
cargo cicd evidence audit
```

---

### Step 5 — Check receipts

```bash
ls -la receipts/
wpm receipt doctor --format json --strict receipts/*.json
```

If `receipts/` is empty, no receipts have been written yet. Receipts are written after adjudication in release runs.

To validate a specific receipt:

```bash
wpm receipt doctor --format json --strict receipts/evt-evidence-doctor-20260614134507123Z.json
```

---

### Step 6 — Interpret each verdict

| Verdict | Meaning | Action |
|---------|---------|--------|
| `Accept` | Evidence is conformant; process is certified | Nothing required — gate passes |
| `Refuse` | Evidence is non-conformant; oracle rejected it | Diagnose and fix (see below) |
| `Blocked` | Oracle was unavailable; verdict was skipped | Install wpm or declare Blocked in test |

---

## Evidence Invariants (E1–E7)

| Invariant | Rule |
|-----------|------|
| **E1** | cargo-cicd never adjudicates itself; only wasm4pm issues verdicts |
| **E2** | XES file must exist on disk before `audit_xes()` is called |
| **E3** | If oracle unavailable and expected verdict is not `Blocked`, panic — certification requires oracle |
| **E4** | Tests assert only on the wasm4pm verdict, never on internal cargo-cicd state |
| **E5** | XES groups events by `case_id` into `<trace>` elements |
| **E6** | JSONL emission mirrors XES — same event set, machine-readable |
| **E7** | `Blocked` is a first-class expectation, not an error, for offline test environments |

---

## XES Format Reference

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <string key="case_id" value="status_show_phase"/>
    <event>
      <string key="event_id" value="evt-status-show-20260614134507123Z"/>
      <string key="timestamp" value="2026-06-14T13:45:07.123Z"/>
      <string key="lifecycle_transition" value="complete"/>
      <string key="verdict_claimed" value="PASS"/>
      <string key="trace_class" value="live_workspace"/>
    </event>
  </trace>
</log>
```

Key fields:

- `event_id` — Unique identifier: `evt-<command>-<timestamp>Z`
- `lifecycle_transition` — `start` (before work) or `complete` (after work)
- `verdict_claimed` — `PASS`, `WARN`, or `FAIL` as claimed by cargo-cicd
- `trace_class` — `live_workspace` (interactive run) or `pipeline_run` (CI pipeline)
- `case_id` — Groups related events into a single `<trace>`; used by wpm for correlation

---

## JSONL Format Reference

Each line is a JSON object mirroring the XES event:

```jsonl
{"event_id":"evt-status-show-20260614134507123Z","timestamp":"2026-06-14T13:45:07.123Z","command":"status show","verdict_claimed":"PASS","lifecycle_transition":"complete","trace_class":"live_workspace"}
```

JSONL files are machine-readable companions to XES. Use them for downstream tooling or log aggregation.

---

## Verdict Meanings in Detail

### Accept

The oracle examined the XES trace and found the process conforms to the declared model. The claimed verdict (`PASS`, `WARN`, or `FAIL`) is consistent with the evidence structure. The gate passes.

### Refuse

The oracle rejected the evidence. Possible causes:

- **Missing required fields** — `event_id`, `timestamp`, or `lifecycle_transition` absent
- **Lifecycle mismatch** — `start` event without a matching `complete`, or vice versa
- **Verdict inconsistency** — claimed `PASS` but evidence shows error conditions
- **Mutated or corrupted XES** — file was modified after emission
- **Schema violation** — XES structure does not match the expected format

To diagnose a Refuse:

```bash
# Inspect the XES file directly
cat target/cargo-cicd/evidence/evt-*.xes

# Check the JSONL companion
cat target/cargo-cicd/evidence/evt-*.jsonl

# Re-run with debug logging
RUST_LOG=debug cargo cicd evidence audit
```

Fix by re-running the originating verb cleanly and re-auditing the new XES file.

### Blocked

The oracle binary was not found on PATH. This is a first-class expectation (`ExpectedWpmVerdict::Blocked`), not a failure mode. In offline CI environments without wpm installed, `Blocked` is the correct response.

Implications:
- Tests that declare `Blocked` pass in offline environments
- `Blocked` does **not** close the release gate — a live `Accept` is required for release
- To unblock: install wpm, add it to PATH, and re-run

---

## Running the Tier 2 Evidence Gate Tests

```bash
# Happy path: normal evidence -> Accept
cargo test --test wasm4pm_evidence_gate -- --nocapture

# Corruption cases: mutated evidence -> Refuse
cargo test --test wasm4pm_evidence_mutation

# Edge cases: oracle unavailable, malformed XES
cargo test --test wasm4pm_refusal_cases
```

These tests assert on the wasm4pm verdict only:

```rust
// Correct pattern (asserts oracle verdict)
assert_eq!(wpm_verdict, WpmVerdict::Accept);

// Incorrect pattern (never do this — asserts internal state)
assert_eq!(state.target.size, expected_size);
```

---

## Full Release Evidence Checklist

```bash
# 1. Verify oracle
wpm --version

# 2. Emit evidence
cargo cicd evidence doctor

# 3. List emitted files
ls -la target/cargo-cicd/evidence/

# 4. Audit all XES files
for f in target/cargo-cicd/evidence/*.xes; do
    echo "--- $f ---"
    wpm audit "$f"
done

# 5. Check receipts
wpm receipt doctor --format json --strict receipts/*.json

# 6. Run all evidence gate tests
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

# 7. Run full test suite
cargo make test
```

All steps must produce `Accept` (or `0` exit code) before the release gate is considered closed.
