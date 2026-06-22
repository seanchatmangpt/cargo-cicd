# /evidence — Evidence Gate Workflow

Trigger: user says "evidence", "emit evidence", "adjudicate", "audit XES", or asks about wpm verdicts.
Action: run steps 1–7 in order. Halt on any non-zero exit or Refuse verdict.

---

## Invariants (never violate)

| ID | Rule |
|----|------|
| E1 | cargo-cicd never adjudicates itself — only wpm issues verdicts |
| E2 | XES file must exist on disk before `audit_xes()` is called |
| E3 | Oracle unavailable + expected verdict ≠ Blocked → panic |
| E4 | Tests assert on wpm verdict only, never on internal state |
| E5 | XES groups events by `case_id` into `<trace>` elements |
| E6 | JSONL mirrors XES — same event set |
| E7 | `Blocked` is a first-class expectation, not an error |

---

## Emission Pattern (every noun handler)

```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

FORBIDDEN: hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs.
FORBIDDEN: calling `wpm` on `.xes` files in new code — OCEL only.
FORBIDDEN: adjudicating inside cargo-cicd (E1).
DELETE `src/ocel.rs` if present — replace with wasm4pm-compat imports.
Do not extend `evidence_xes_v2.rs` — legacy, OCEL supersedes it.

Dependency:
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

OCEL 2.0 JSON shape (what wpm expects on disk):
```json
{ "eventTypes": [...], "objectTypes": [...], "events": [...], "objects": [...] }
```

Object types in cargo-cicd domain:
`Workspace` · `Crate` · `TestRun` · `GitCommit` · `Release` · `Receipt` · `EvidenceFile` · `Policy` · `Toolchain`

---

## Step 1 — Verify oracle

```bash
which wpm && wpm --version
```

Not found → build from source or declare `ExpectedWpmVerdict::Blocked` in tests. Release gate requires live `Accept`.

---

## Step 2 — Emit evidence

```bash
cargo cicd evidence doctor
```

Emits to `target/cargo-cicd/evidence/`. On empty output:
```bash
RUST_LOG=debug cargo cicd evidence doctor
```

---

## Step 3 — Audit

```bash
for f in target/cargo-cicd/evidence/*.xes; do
    echo "--- $f ---"
    wpm audit "$f"
done
```

Or via CLI (requires `wasm4pm` feature):
```bash
cargo cicd evidence audit
```

---

## Step 4 — Receipts

```bash
wpm receipt doctor --format json --strict receipts/*.json
```

---

## Verdict Table

| Verdict | Meaning | Action |
|---------|---------|--------|
| `Accept` | Conformant — gate passes | None |
| `Refuse` | Non-conformant — oracle rejected | Diagnose below |
| `Blocked` | Oracle unavailable | Install wpm or use `ExpectedWpmVerdict::Blocked` |

### Refuse — diagnosis

Causes: missing `event_id`/`timestamp`/`lifecycle_transition` · unmatched `start`/`complete` · claimed PASS with error evidence · mutated XES · schema violation.

```bash
cat target/cargo-cicd/evidence/evt-*.xes
cat target/cargo-cicd/evidence/evt-*.jsonl
RUST_LOG=debug cargo cicd evidence audit
```

Fix: re-run the originating verb, re-audit the new file.

---

## Test Pattern

```rust
// CORRECT
assert_eq!(wpm_verdict, WpmVerdict::Accept);

// FORBIDDEN — asserts internal state (violates E4)
assert_eq!(state.target.size, expected_size);
```

```bash
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

---

## Release Checklist

```bash
wpm --version
cargo cicd evidence doctor
ls -la target/cargo-cicd/evidence/
for f in target/cargo-cicd/evidence/*.xes; do echo "--- $f ---"; wpm audit "$f"; done
wpm receipt doctor --format json --strict receipts/*.json
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
cargo make test
```

All steps must exit 0 with `Accept` before the release gate closes.
