# wasm4pm Evidence Gate — Architecture

**cargo-cicd v26.6.2**

**The law:** cargo-cicd emits. wasm4pm adjudicates. Tests assert only the wasm4pm verdict.

---

## Architecture: the inversion of trust

cargo-cicd is untrusted input to wasm4pm. It cannot certify its own process conformance.

**Old shape — self-certifying (REJECTED):**

```
cargo-cicd run → internal assertions pass → gate closed
```

The pipeline declared its own success. No external adjudicator. No process evidence.
Any bug in the runner also corrupts the verdict. Circular.

**New shape — evidence gate (CURRENT):**

```
cargo-cicd run
    │
    │  emit_xes() → XES file on disk
    │
    ▼
WpmEvidenceOracle.audit_xes()
    │
    │  shells out to: wpm audit <path>
    │
    ▼
wasm4pm (external adjudicator)
    │
    │  verdict: Pass | Warn | Partial | Fail | NotAvailable
    │
    ▼
ExpectedWpmVerdict: Accept | Refuse | Blocked
    │
    ▼
assert_wpm_verdict() — the only assertion tests are allowed to make
```

The inversion is structural, not a convention. `emit_xes` returns `Result<()>` — no verdict.
A verdict is only obtainable by constructing a `WpmEvidenceOracle` and calling `audit_xes`.
The type system enforces the separation.

---

## Evidence emission: how ProcessEvent works

`ProcessEvent` is the unit of evidence. Each event represents one cargo-cicd command execution.

**Required fields:**

| Field | Type | Meaning |
|---|---|---|
| `event_id` | `String` | Unique event identifier. Canonical form: `evt-{command}` with spaces replaced by dashes. |
| `timestamp_iso` | `String` | ISO-8601 timestamp. Default: `"2026-06-02T00:00:00.000Z"`. |
| `workspace_id` | `String` | Workspace identifier. Default: `"cargo-cicd-workspace"`. |
| `repo_path` | `String` | Path to the repository root. Default: `"."`. |
| `command` | `String` | The cargo-cicd command executed (e.g. `"status show"`). |
| `verdict_claimed_by_cargo_cicd` | `String` | The outcome cargo-cicd observed (e.g. `"PASS"`, `"DRY-RUN"`). This is a **claim**, not a verdict. wasm4pm determines the verdict. |
| `duration_ms` | `u64` | Execution duration in milliseconds. Default: `0`. |

**Constructor:**

```rust
ProcessEvent::new("status show", "PASS")
```

**Emission:**

```rust
let events = vec![ProcessEvent::new("status show", "PASS")];
let xes_path = dir.path().join("events.xes");
emit_xes(&events, &xes_path).expect("emit_xes must not fail");
```

`emit_xes` writes a complete, self-contained XES 1.0 log. Each event becomes one `<event>` element
inside a single `<trace>`. Parent directories are created if absent. File is overwritten if it exists.

A companion JSONL format is available via `emit_events_jsonl` for downstream tooling (invariant E6).
The canonical evidence directory is `target/cargo-cicd/evidence/` (returned by `evidence_dir()`).

**Confirmed commands (positive gate tests accept these):**

| Command | Claimed Verdict |
|---|---|
| `status show` | `PASS` |
| `target show` | `PASS` |
| `target prune plan` | `DRY-RUN` |
| `test changed` | `PASS` |
| `git close` | `PASS` |
| `publish run` | `PASS` |
| `workspace doctor` | `PASS` |

---

## Oracle setup: how WpmEvidenceOracle.discover() works

`WpmEvidenceOracle::new()` auto-detects the wpm binary via `Wasm4pmShell::detect()` (checks `WPM_PATH`
environment variable or `PATH` lookup).

```rust
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}
```

`audit_xes` verdict mapping:

| wpm result | `ExpectedWpmVerdict` |
|---|---|
| Binary absent | `Blocked` |
| Invocation error | `Refuse` |
| `Pass` | `Accept` |
| `Warn` | `Accept` |
| `Partial` | `Accept` |
| `Fail` | `Refuse` |
| `NotAvailable` | `Blocked` |

---

## Test structure

Every evidence-gate test follows the same three-phase structure:

**Phase 1 — emit evidence:**
```rust
let dir = TempDir::new().unwrap();
let events = vec![ProcessEvent::new("<command>", "<claimed verdict>")];
let xes_path = dir.path().join("events.xes");
emit_xes(&events, &xes_path).expect("emit_xes must not fail");
assert!(xes_path.exists(), "XES file must exist before oracle call");
```

**Phase 2 — construct oracle:**
```rust
let oracle = WpmEvidenceOracle::new();
```

**Phase 3 — assert verdict (the only assertion):**
```rust
if oracle.is_available() {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}
```

**Positive cases** (`tests/wasm4pm_evidence_gate.rs`): emit valid XES, expect `Accept`.

**Negative mutation cases** (`tests/wasm4pm_evidence_mutation.rs`): emit valid XES, then corrupt it, then expect `Refuse`.

Mutation helpers exported `pub` from `wasm4pm_evidence_mutation.rs`:

| Helper | Corruption |
|---|---|
| `corrupt_xes_contradictory_verdict` | Replace `PASS` → `FAIL` in attribute values |
| `corrupt_xes_missing_trace` | Strip `<trace>…</trace>` element entirely |
| `corrupt_xes_no_closing_tag` | Remove `</log>` closing tag |
| `corrupt_xes_empty_file` | Overwrite with zero bytes |
| `corrupt_xes_binary_garbage` | Overwrite with non-XML bytes (`\x00\x01\x02\xff\xfe`) |
| `corrupt_xes_truncated` | Truncate to 20 bytes |
| `corrupt_xes_invalid_attribute` | Inject unescaped `<` inside attribute value |
| `corrupt_xes_wrong_encoding_declaration` | Declare `EBCDIC-US` encoding on UTF-8 content |

---

## The 7 invariants (E1–E7)

These invariants are structural laws of the evidence gate. Violation of any invariant is a defect.

**E1: cargo-cicd NEVER adjudicates its own process conformance.**
All verdicts are issued by the external wasm4pm oracle. `emit_xes` returns `Result<()>` — no verdict
is available without constructing a `WpmEvidenceOracle`. Proven structurally in `evidence_invariant_e1_no_self_certification`.

**E2: Evidence is emitted before adjudication.**
The XES file must exist on disk before `audit_xes` is called. Tests assert `xes_path.exists()`
between emission and oracle invocation. Proven in `evidence_invariant_e2_evidence_required_before_adjudication`.

**E3: If the oracle is unavailable and the expected verdict is not `Blocked`, the evidence gate panics.**
Certification requires the oracle. `assert_wpm_verdict` panics with an E3 violation message if
`actual == Blocked` and `expected != Blocked`. Proven in `evidence_invariant_e3_blocked_is_first_class`.

**E4: Tests assert only wasm4pm verdict, never internal cargo-cicd state.**
cargo-cicd state assertions belong in unit tests. Process conformance assertions belong in
evidence-gate tests. The two must not be mixed.

**E5: XES emission is append-safe.**
Each call to `emit_xes` produces a complete, self-contained log for the event slice passed.
No partial or streaming writes.

**E6: JSONL emission mirrors XES.**
`emit_events_jsonl` produces a JSONL companion for the same event set, with `event_id`, `command`,
and `verdict_claimed_by_cargo_cicd` fields. Machine-readable companion for downstream tooling.

**E7: `ExpectedWpmVerdict::Blocked` is a first-class expectation, not an error state.**
Tests that run without wpm installed MUST declare `Blocked` as their expected verdict.
`oracle.is_available()` guards all evidence-gate tests so they pass cleanly in CI environments
where wpm is not present.

---

## BLOCKED protocol

When `wpm` is unavailable (`oracle.is_available() == false`):

- `oracle.audit_xes(path)` returns `ExpectedWpmVerdict::Blocked` immediately.
- Tests assert `ExpectedWpmVerdict::Blocked` — this is not a skip, it is a first-class assertion (E7).
- `assert_wpm_verdict` with `expected = Blocked` succeeds without invoking the oracle.
- `assert_wpm_verdict` with `expected = Accept` or `Refuse` panics with:

```
BLOCKED: wasm4pm oracle command unavailable — evidence gate cannot certify.
wpm binary not found. Install wasm4pm or set WPM_PATH env var.
Evidence gate invariant E3 violated: external oracle required.
```

---

## GREEN condition

The wasm4pm evidence gate is GREEN when:

> wasm4pm accepts positive evidence and refuses corrupted variants.

Specifically:
1. All 7 positive gate tests in `tests/wasm4pm_evidence_gate.rs` pass with `Accept` (or `Blocked` when wpm unavailable).
2. All mutation tests in `tests/wasm4pm_evidence_mutation.rs` pass with `Refuse` for corrupted inputs (or `Blocked` when wpm unavailable).
3. All refusal cases in `tests/wasm4pm_refusal_cases.rs` pass with `Refuse` (or `Blocked` when wpm unavailable).
4. `evidence_gate_oracle_discover` passes without panicking (E7 compliance).
5. Invariant tests E1, E2, E3 pass structurally.

The gate is not GREEN if any positive case returns `Refuse` or any mutation case returns `Accept`.

---

## Key source paths

| Path | Purpose |
|---|---|
| `src/evidence.rs` | `ProcessEvent`, `WpmEvidenceOracle`, `emit_xes`, `assert_wpm_verdict`, invariant docs |
| `tests/wasm4pm_evidence_gate.rs` | 7 positive acceptance cases + oracle discover |
| `tests/wasm4pm_evidence_mutation.rs` | 5 direct mutation tests + 8 mutation helper functions |
| `tests/wasm4pm_refusal_cases.rs` | 4 refusal cases + 3 invariant structural tests (E1, E2, E3) |
