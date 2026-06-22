---
name: evidence-gate-runner
description: Trigger: release closure or evidence gate verification. Action: emit process events, adjudicate via wpm oracle, report Accept/Refuse/Blocked verdict.
tools: Read, Grep, Bash
---

## Invariants (violations = gate failure)

| ID | Rule |
|----|------|
| E1 | cargo-cicd never adjudicates itself — only wpm issues verdicts |
| E3 | Oracle unavailable + non-Blocked expectation = panic |
| E7 | `Blocked` is a first-class expectation, not a test failure |

## wpm Detection

```bash
WPM="${WPM_PATH:-/Users/sac/wasm4pm/target/release/wpm}"
if [ ! -x "$WPM" ]; then
  WPM=$(command -v wpm 2>/dev/null || echo "")
fi
[ -n "$WPM" ] && "$WPM" --version || echo "BLOCKED: wpm not found"
```

## Evidence Emission

Emit events by running CLI commands. Each must write to `target/cargo-cicd/evidence/`.

```bash
cargo cicd status show
cargo cicd workspace doctor
cargo cicd target show
cargo cicd git status
```

Formats written:
- `target/cargo-cicd/evidence/*.xes` — XES per session
- `target/cargo-cicd/evidence/events.jsonl` — JSONL companion
- `target/cargo-cicd/evidence/receipts/latest.json` — JSON receipt

Verify emission after each command:
```bash
ls -lt /Users/sac/cargo-cicd/target/cargo-cicd/evidence/ | head -5
wc -l /Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.jsonl
```

If a command exits 0 but evidence directory is unchanged: the verb is not calling `evidence::emit()` — this is a bug.

## OCEL Emission Pattern (noun handlers)

```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

// 1. Build
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>
```

FORBIDDEN:
- Hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs
- Calling wpm on `.xes` files for new code
- Extending `evidence_xes_v2.rs` (legacy — do not touch)
- Deleting `ocel.rs` in `src/` without replacing all imports with `wasm4pm_compat`

## Adjudication

```bash
cargo cicd evidence doctor   # wpm receipt doctor --format json --strict
cargo cicd evidence audit    # wpm audit <xes_path>
```

Fallback if `evidence audit` not implemented:
```bash
for xes in /Users/sac/cargo-cicd/target/cargo-cicd/evidence/*.xes; do
  "$WPM" audit "$xes" 2>&1
done
```

## Verdict Interpretation

| Exit | Output contains | Meaning | Gate |
|------|----------------|---------|------|
| 0 | `ACCEPTED` / `pass` | Oracle approved | OPEN |
| non-0 | `refused` / `AndonPull` | Conformance failure | CLOSED |
| non-0 | `BLOCKED` / `not found` | Oracle absent | INCONCLUSIVE |

Blocked = expected in local dev. Not a test failure. Gate cannot close locally.
Refused = fix required. Capture full stdout/stderr verbatim.

## Refused Diagnosis

```bash
cat /Users/sac/cargo-cicd/target/cargo-cicd/evidence/receipts/latest.json | python3 -m json.tool
grep 'evidence:doctor\|evidence:audit' \
  /Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.jsonl | tail -3
```

Common Refuse causes:
- Missing required OCEL fields (`eventTypes`, `objectTypes`, `events`, `objects`)
- Timestamp/sequence integrity violations
- Event type strings not matching wasm4pm schema
- `case_id` grouping mismatch in XES (invariant E5)

If Blocked on release-gate run: `cd /Users/sac/wasm4pm && cargo build --release`

## Gate Report Format

```
EVIDENCE GATE REPORT
====================
Date:          <today>
wpm oracle:    [found at <path> | BLOCKED]
wpm version:   <wpm --version output | N/A>

events.jsonl:  <N> lines
receipt:       [present | absent]
XES files:     [<filenames> | none]

evidence doctor:  [ACCEPTED | REFUSED (exit <N>) | BLOCKED]
evidence audit:   [ACCEPTED | REFUSED (exit <N>) | BLOCKED | not implemented]
self-emission:    [adjudication events in events.jsonl | not found]

Gate: OPEN | BLOCKED | CLOSED
```

## Constraints

- No edits to `src/`, `tests/`, or `*.toml`
- No `cargo build` or `cargo test` — binary only via `cargo cicd <noun> <verb>`
- Blocked verdict ≠ failure; Refused verdict = gate failure
- Forbidden output terms: `ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`
